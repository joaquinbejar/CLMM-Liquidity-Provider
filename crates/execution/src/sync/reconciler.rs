//! State reconciler for ensuring consistency.

use super::AccountUpdate;
use clmm_lp_protocols::prelude::RpcProvider;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Reconciliation status for an account.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconcileStatus {
    /// Account is in sync.
    InSync,
    /// Account needs update.
    NeedsUpdate,
    /// Account is being updated.
    Updating,
    /// Reconciliation failed.
    Failed,
}

/// State for a tracked account.
#[derive(Debug, Clone)]
pub struct AccountState {
    /// Account address.
    pub address: Pubkey,
    /// Last known slot.
    pub last_slot: u64,
    /// Last update time.
    pub last_update: Instant,
    /// Reconciliation status.
    pub status: ReconcileStatus,
    /// Number of failed reconciliations.
    pub failure_count: u32,
}

/// Configuration for the reconciler.
#[derive(Debug, Clone)]
pub struct ReconcilerConfig {
    /// Maximum age before forcing reconciliation in seconds.
    pub max_age_secs: u64,
    /// Reconciliation interval in seconds.
    pub reconcile_interval_secs: u64,
    /// Maximum failures before marking account as failed.
    pub max_failures: u32,
}

impl Default for ReconcilerConfig {
    fn default() -> Self {
        Self {
            max_age_secs: 60,
            reconcile_interval_secs: 30,
            max_failures: 3,
        }
    }
}

/// Reconciler for keeping local state in sync with on-chain.
pub struct Reconciler {
    /// RPC provider.
    provider: Arc<RpcProvider>,
    /// Configuration.
    config: ReconcilerConfig,
    /// Tracked accounts.
    accounts: Arc<RwLock<HashMap<Pubkey, AccountState>>>,
    /// Current slot.
    current_slot: Arc<RwLock<u64>>,
}

impl Reconciler {
    /// Creates a new reconciler.
    pub fn new(provider: Arc<RpcProvider>, config: ReconcilerConfig) -> Self {
        Self {
            provider,
            config,
            accounts: Arc::new(RwLock::new(HashMap::new())),
            current_slot: Arc::new(RwLock::new(0)),
        }
    }

    /// Tracks an account for reconciliation.
    pub async fn track_account(&self, address: Pubkey) {
        let state = AccountState {
            address,
            last_slot: 0,
            last_update: Instant::now(),
            status: ReconcileStatus::NeedsUpdate,
            failure_count: 0,
        };

        self.accounts.write().await.insert(address, state);
        debug!(address = %address, "Tracking account for reconciliation");
    }

    /// Stops tracking an account.
    pub async fn untrack_account(&self, address: &Pubkey) {
        self.accounts.write().await.remove(address);
        debug!(address = %address, "Stopped tracking account");
    }

    /// Processes an account update from WebSocket.
    pub async fn process_update(&self, update: AccountUpdate) {
        let mut accounts = self.accounts.write().await;

        if let Some(state) = accounts.get_mut(&update.address) {
            state.last_slot = update.slot;
            state.last_update = Instant::now();
            state.status = ReconcileStatus::InSync;
            state.failure_count = 0;

            debug!(
                address = %update.address,
                slot = update.slot,
                "Processed account update"
            );
        }
    }

    /// Runs a reconciliation cycle.
    pub async fn reconcile(&self) -> ReconcileResult {
        let current_slot = self.fetch_current_slot().await;
        *self.current_slot.write().await = current_slot;

        let mut result = ReconcileResult::default();
        let now = Instant::now();

        let addresses: Vec<Pubkey> = self.accounts.read().await.keys().copied().collect();

        for address in addresses {
            let needs_reconcile = {
                let accounts = self.accounts.read().await;
                if let Some(state) = accounts.get(&address) {
                    let age = now.duration_since(state.last_update);
                    age > Duration::from_secs(self.config.max_age_secs)
                        || state.status == ReconcileStatus::NeedsUpdate
                } else {
                    false
                }
            };

            if needs_reconcile {
                match self.reconcile_account(&address).await {
                    Ok(()) => {
                        result.reconciled += 1;
                    }
                    Err(e) => {
                        warn!(address = %address, error = %e, "Reconciliation failed");
                        result.failed += 1;

                        // Update failure count
                        let mut accounts = self.accounts.write().await;
                        if let Some(state) = accounts.get_mut(&address) {
                            state.failure_count += 1;
                            if state.failure_count >= self.config.max_failures {
                                state.status = ReconcileStatus::Failed;
                            }
                        }
                    }
                }
            } else {
                result.in_sync += 1;
            }
        }

        result.current_slot = current_slot;
        result
    }

    /// Reconciles a single account.
    async fn reconcile_account(&self, address: &Pubkey) -> anyhow::Result<()> {
        // Mark as updating
        {
            let mut accounts = self.accounts.write().await;
            if let Some(state) = accounts.get_mut(address) {
                state.status = ReconcileStatus::Updating;
            }
        }

        // Fetch account from RPC
        let account = self.provider.get_account(address).await?;

        // Update state
        {
            let mut accounts = self.accounts.write().await;
            if let Some(state) = accounts.get_mut(address) {
                state.last_slot = *self.current_slot.read().await;
                state.last_update = Instant::now();
                state.status = ReconcileStatus::InSync;
                state.failure_count = 0;
            }
        }

        debug!(
            address = %address,
            data_len = account.data.len(),
            "Reconciled account"
        );

        Ok(())
    }

    /// Fetches the current slot.
    async fn fetch_current_slot(&self) -> u64 {
        self.provider.get_slot().await.unwrap_or(0)
    }

    /// Gets the status of all tracked accounts.
    pub async fn get_status(&self) -> HashMap<Pubkey, AccountState> {
        self.accounts.read().await.clone()
    }

    /// Gets accounts that need attention.
    pub async fn get_stale_accounts(&self) -> Vec<Pubkey> {
        let now = Instant::now();
        let max_age = Duration::from_secs(self.config.max_age_secs);

        self.accounts
            .read()
            .await
            .iter()
            .filter(|(_, state)| {
                now.duration_since(state.last_update) > max_age
                    || state.status != ReconcileStatus::InSync
            })
            .map(|(addr, _)| *addr)
            .collect()
    }

    /// Gets failed accounts.
    pub async fn get_failed_accounts(&self) -> Vec<Pubkey> {
        self.accounts
            .read()
            .await
            .iter()
            .filter(|(_, state)| state.status == ReconcileStatus::Failed)
            .map(|(addr, _)| *addr)
            .collect()
    }

    /// Starts the reconciliation loop.
    pub async fn start(&self) {
        info!(
            interval_secs = self.config.reconcile_interval_secs,
            "Starting reconciler"
        );

        let mut interval =
            tokio::time::interval(Duration::from_secs(self.config.reconcile_interval_secs));

        loop {
            interval.tick().await;

            let result = self.reconcile().await;
            debug!(
                in_sync = result.in_sync,
                reconciled = result.reconciled,
                failed = result.failed,
                "Reconciliation cycle complete"
            );
        }
    }
}

/// Result of a reconciliation cycle.
#[derive(Debug, Clone, Default)]
pub struct ReconcileResult {
    /// Current slot.
    pub current_slot: u64,
    /// Accounts already in sync.
    pub in_sync: u32,
    /// Accounts reconciled.
    pub reconciled: u32,
    /// Accounts that failed reconciliation.
    pub failed: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clmm_lp_protocols::prelude::RpcConfig;

    #[tokio::test]
    async fn test_reconciler_track_account() {
        let config = RpcConfig::default();
        let provider = Arc::new(RpcProvider::new(config));
        let reconciler = Reconciler::new(provider, ReconcilerConfig::default());

        let address = Pubkey::new_unique();
        reconciler.track_account(address).await;

        let status = reconciler.get_status().await;
        assert!(status.contains_key(&address));
    }
}
