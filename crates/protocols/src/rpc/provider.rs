//! RPC provider with automatic failover and retry logic.

use super::{HealthChecker, RpcConfig};
use anyhow::{Context, Result};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::account::Account;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::{debug, info, warn};

/// RPC provider with automatic failover and health checking.
pub struct RpcProvider {
    /// Configuration.
    config: RpcConfig,
    /// Health checker.
    health: Arc<HealthChecker>,
    /// Current active endpoint index.
    current_endpoint_idx: Arc<RwLock<usize>>,
}

impl RpcProvider {
    /// Creates a new RPC provider with the given configuration.
    #[must_use]
    pub fn new(config: RpcConfig) -> Self {
        Self {
            config,
            health: Arc::new(HealthChecker::new()),
            current_endpoint_idx: Arc::new(RwLock::new(0)),
        }
    }

    /// Creates a new RPC provider for mainnet with default settings.
    #[must_use]
    pub fn mainnet() -> Self {
        Self::new(RpcConfig::default())
    }

    /// Creates a new RPC provider for devnet.
    #[must_use]
    pub fn devnet() -> Self {
        Self::new(RpcConfig::devnet())
    }

    /// Creates a new RPC provider for localhost.
    #[must_use]
    pub fn localhost() -> Self {
        Self::new(RpcConfig::localhost())
    }

    /// Returns the current active endpoint.
    pub async fn current_endpoint(&self) -> String {
        let idx = *self.current_endpoint_idx.read().await;
        let endpoints = self.config.all_endpoints();
        endpoints.get(idx).unwrap_or(&endpoints[0]).to_string()
    }

    /// Gets an RPC client for the current endpoint.
    async fn get_client(&self) -> RpcClient {
        let endpoint = self.current_endpoint().await;
        RpcClient::new_with_timeout(endpoint, self.config.timeout)
    }

    /// Rotates to the next healthy endpoint.
    async fn rotate_endpoint(&self) {
        let endpoints = self.config.all_endpoints();
        let mut idx = self.current_endpoint_idx.write().await;

        for i in 1..=endpoints.len() {
            let next_idx = (*idx + i) % endpoints.len();
            let endpoint = endpoints[next_idx];

            if self.health.is_healthy(endpoint).await {
                info!(
                    from = endpoints[*idx],
                    to = endpoint,
                    "Rotating to new RPC endpoint"
                );
                *idx = next_idx;
                return;
            }
        }

        // All endpoints unhealthy, try the next one anyway
        *idx = (*idx + 1) % endpoints.len();
        warn!("All endpoints unhealthy, rotating anyway");
    }

    /// Executes a request with retry and failover logic.
    async fn execute_with_retry<T, F, Fut>(&self, operation: F) -> Result<T>
    where
        F: Fn(RpcClient) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut last_error = None;
        let mut retry_count = 0;

        while retry_count <= self.config.max_retries {
            let endpoint = self.current_endpoint().await;
            let client = self.get_client().await;
            let start = Instant::now();

            match operation(client).await {
                Ok(result) => {
                    let elapsed = start.elapsed().as_millis() as f64;
                    self.health.record_success(&endpoint, elapsed).await;
                    return Ok(result);
                }
                Err(e) => {
                    warn!(
                        endpoint = endpoint,
                        retry = retry_count,
                        error = %e,
                        "RPC request failed"
                    );
                    self.health.record_failure(&endpoint).await;
                    last_error = Some(e);

                    // Rotate endpoint on failure
                    self.rotate_endpoint().await;

                    // Exponential backoff
                    if retry_count < self.config.max_retries {
                        let delay = calculate_backoff(
                            retry_count,
                            self.config.retry_base_delay_ms,
                            self.config.retry_max_delay_ms,
                        );
                        debug!(delay_ms = delay, "Waiting before retry");
                        sleep(Duration::from_millis(delay)).await;
                    }

                    retry_count += 1;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Unknown error")))
    }

    /// Gets the current slot.
    pub async fn get_slot(&self) -> Result<u64> {
        self.execute_with_retry(|client| async move {
            client.get_slot().await.context("Failed to get slot")
        })
        .await
    }

    /// Gets the current block height.
    pub async fn get_block_height(&self) -> Result<u64> {
        self.execute_with_retry(|client| async move {
            client
                .get_block_height()
                .await
                .context("Failed to get block height")
        })
        .await
    }

    /// Gets account data for a given address.
    pub async fn get_account(&self, address: &Pubkey) -> Result<Account> {
        let addr = *address;
        self.execute_with_retry(|client| async move {
            client
                .get_account(&addr)
                .await
                .context("Failed to get account")
        })
        .await
    }

    /// Gets account data by address string.
    pub async fn get_account_by_address(&self, address: &str) -> Result<Account> {
        let pubkey = Pubkey::from_str(address).context("Invalid pubkey")?;
        self.get_account(&pubkey).await
    }

    /// Gets multiple accounts.
    pub async fn get_multiple_accounts(
        &self,
        addresses: &[Pubkey],
    ) -> Result<Vec<Option<Account>>> {
        let addrs = addresses.to_vec();
        self.execute_with_retry(|client| {
            let addrs = addrs.clone();
            async move {
                client
                    .get_multiple_accounts(&addrs)
                    .await
                    .context("Failed to get multiple accounts")
            }
        })
        .await
    }

    /// Gets the balance of an account in lamports.
    pub async fn get_balance(&self, address: &Pubkey) -> Result<u64> {
        let addr = *address;
        self.execute_with_retry(|client| async move {
            client
                .get_balance(&addr)
                .await
                .context("Failed to get balance")
        })
        .await
    }

    /// Gets the latest blockhash.
    pub async fn get_latest_blockhash(&self) -> Result<solana_sdk::hash::Hash> {
        self.execute_with_retry(|client| async move {
            client
                .get_latest_blockhash()
                .await
                .context("Failed to get latest blockhash")
        })
        .await
    }

    /// Gets transaction status.
    pub async fn get_signature_status(
        &self,
        signature: &Signature,
    ) -> Result<Option<solana_sdk::transaction::TransactionError>> {
        let sig = *signature;
        self.execute_with_retry(|client| async move {
            let statuses = client
                .get_signature_statuses(&[sig])
                .await
                .context("Failed to get signature status")?;

            Ok(statuses
                .value
                .first()
                .and_then(|s| s.as_ref().and_then(|status| status.err.clone())))
        })
        .await
    }

    /// Gets the health status of all endpoints.
    pub async fn get_health_status(
        &self,
    ) -> std::collections::HashMap<String, super::EndpointHealth> {
        self.health.get_all_health().await
    }

    /// Performs a health check on all endpoints.
    pub async fn check_all_endpoints(&self) {
        let endpoints = self.config.all_endpoints();
        for endpoint in endpoints {
            let _ = self.health.check_endpoint(endpoint).await;
        }
    }

    /// Simulates a transaction without broadcasting.
    pub async fn simulate_transaction(
        &self,
        transaction: &solana_sdk::transaction::Transaction,
    ) -> Result<solana_client::rpc_response::RpcSimulateTransactionResult> {
        let tx = transaction.clone();
        self.execute_with_retry(|client| {
            let tx = tx.clone();
            async move {
                let response = client
                    .simulate_transaction(&tx)
                    .await
                    .context("Failed to simulate transaction")?;
                Ok(response.value)
            }
        })
        .await
    }

    /// Sends and confirms a transaction.
    pub async fn send_and_confirm_transaction(
        &self,
        transaction: &solana_sdk::transaction::Transaction,
    ) -> Result<Signature> {
        let tx = transaction.clone();
        self.execute_with_retry(|client| {
            let tx = tx.clone();
            async move {
                client
                    .send_and_confirm_transaction(&tx)
                    .await
                    .context("Failed to send and confirm transaction")
            }
        })
        .await
    }

    /// Sends a transaction without waiting for confirmation.
    pub async fn send_transaction(
        &self,
        transaction: &solana_sdk::transaction::Transaction,
    ) -> Result<Signature> {
        let tx = transaction.clone();
        self.execute_with_retry(|client| {
            let tx = tx.clone();
            async move {
                client
                    .send_transaction(&tx)
                    .await
                    .context("Failed to send transaction")
            }
        })
        .await
    }
}

/// Calculates exponential backoff delay.
fn calculate_backoff(retry: u32, base_ms: u64, max_ms: u64) -> u64 {
    let delay = base_ms * 2u64.pow(retry);
    delay.min(max_ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_backoff() {
        assert_eq!(calculate_backoff(0, 100, 5000), 100);
        assert_eq!(calculate_backoff(1, 100, 5000), 200);
        assert_eq!(calculate_backoff(2, 100, 5000), 400);
        assert_eq!(calculate_backoff(3, 100, 5000), 800);
        assert_eq!(calculate_backoff(10, 100, 5000), 5000); // Capped at max
    }

    #[tokio::test]
    async fn test_provider_creation() {
        let provider = RpcProvider::mainnet();
        let endpoint = provider.current_endpoint().await;
        assert!(endpoint.contains("mainnet"));
    }

    #[tokio::test]
    async fn test_devnet_provider() {
        let provider = RpcProvider::devnet();
        let endpoint = provider.current_endpoint().await;
        assert!(endpoint.contains("devnet"));
    }
}
