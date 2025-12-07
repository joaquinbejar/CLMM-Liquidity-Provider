//! WebSocket account listener for real-time updates.

use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tracing::{debug, error, info, warn};

/// Account update event.
#[derive(Debug, Clone)]
pub struct AccountUpdate {
    /// Account address.
    pub address: Pubkey,
    /// Slot of the update.
    pub slot: u64,
    /// Account data.
    pub data: Vec<u8>,
    /// Lamports balance.
    pub lamports: u64,
    /// Owner program.
    pub owner: Pubkey,
}

/// Subscription type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubscriptionType {
    /// Pool account.
    Pool,
    /// Position account.
    Position,
    /// Token account.
    TokenAccount,
}

/// Subscription info.
#[derive(Debug, Clone)]
pub struct Subscription {
    /// Account address.
    pub address: Pubkey,
    /// Subscription type.
    pub sub_type: SubscriptionType,
    /// WebSocket subscription ID (if connected).
    pub ws_subscription_id: Option<u64>,
    /// Whether subscription is active.
    pub active: bool,
}

/// Configuration for account listener.
#[derive(Debug, Clone)]
pub struct AccountListenerConfig {
    /// WebSocket URL.
    pub ws_url: String,
    /// Reconnect delay in seconds.
    pub reconnect_delay_secs: u64,
    /// Maximum reconnect attempts.
    pub max_reconnect_attempts: u32,
    /// Commitment level for subscriptions.
    pub commitment: String,
}

impl Default for AccountListenerConfig {
    fn default() -> Self {
        Self {
            ws_url: "wss://api.mainnet-beta.solana.com".to_string(),
            reconnect_delay_secs: 5,
            max_reconnect_attempts: 10,
            commitment: "confirmed".to_string(),
        }
    }
}

/// Listener for account changes via WebSocket.
pub struct AccountListener {
    /// Configuration.
    config: AccountListenerConfig,
    /// Active subscriptions.
    subscriptions: Arc<RwLock<HashMap<Pubkey, Subscription>>>,
    /// Update sender.
    update_tx: mpsc::Sender<AccountUpdate>,
    /// Update receiver.
    update_rx: Option<mpsc::Receiver<AccountUpdate>>,
    /// Connection status.
    connected: Arc<RwLock<bool>>,
    /// Reconnect attempts.
    reconnect_attempts: Arc<RwLock<u32>>,
}

impl AccountListener {
    /// Creates a new account listener.
    pub fn new(config: AccountListenerConfig) -> Self {
        let (tx, rx) = mpsc::channel(1000);
        Self {
            config,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            update_tx: tx,
            update_rx: Some(rx),
            connected: Arc::new(RwLock::new(false)),
            reconnect_attempts: Arc::new(RwLock::new(0)),
        }
    }

    /// Takes the update receiver.
    pub fn take_receiver(&mut self) -> Option<mpsc::Receiver<AccountUpdate>> {
        self.update_rx.take()
    }

    /// Subscribes to an account.
    pub async fn subscribe(&self, address: Pubkey, sub_type: SubscriptionType) {
        let subscription = Subscription {
            address,
            sub_type,
            ws_subscription_id: None,
            active: false,
        };

        self.subscriptions
            .write()
            .await
            .insert(address, subscription);

        info!(
            address = %address,
            sub_type = ?sub_type,
            "Added subscription"
        );

        // If connected, activate subscription
        if *self.connected.read().await {
            self.activate_subscription(&address).await;
        }
    }

    /// Unsubscribes from an account.
    pub async fn unsubscribe(&self, address: &Pubkey) {
        if let Some(sub) = self.subscriptions.write().await.remove(address) {
            if sub.active {
                self.deactivate_subscription(&sub).await;
            }
            info!(address = %address, "Removed subscription");
        }
    }

    /// Starts the listener.
    pub async fn start(&self) {
        info!(ws_url = %self.config.ws_url, "Starting account listener");

        loop {
            match self.connect().await {
                Ok(()) => {
                    *self.reconnect_attempts.write().await = 0;
                    self.run_event_loop().await;
                }
                Err(e) => {
                    error!(error = %e, "WebSocket connection failed");
                }
            }

            // Check reconnect attempts
            let attempts = {
                let mut attempts = self.reconnect_attempts.write().await;
                *attempts += 1;
                *attempts
            };

            if attempts >= self.config.max_reconnect_attempts {
                error!("Max reconnect attempts reached, stopping listener");
                break;
            }

            warn!(
                attempts = attempts,
                delay_secs = self.config.reconnect_delay_secs,
                "Reconnecting..."
            );

            tokio::time::sleep(std::time::Duration::from_secs(
                self.config.reconnect_delay_secs,
            ))
            .await;
        }
    }

    /// Connects to the WebSocket.
    async fn connect(&self) -> anyhow::Result<()> {
        // Note: In a real implementation, this would establish a WebSocket connection
        // using tokio-tungstenite or similar
        info!("Would connect to WebSocket: {}", self.config.ws_url);

        *self.connected.write().await = true;

        // Activate all subscriptions
        let addresses: Vec<Pubkey> = self.subscriptions.read().await.keys().copied().collect();
        for address in addresses {
            self.activate_subscription(&address).await;
        }

        Ok(())
    }

    /// Runs the event loop.
    async fn run_event_loop(&self) {
        // Note: In a real implementation, this would process WebSocket messages
        debug!("Running event loop");

        // Simulate running for a while
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;

        *self.connected.write().await = false;
    }

    /// Activates a subscription.
    async fn activate_subscription(&self, address: &Pubkey) {
        if let Some(sub) = self.subscriptions.write().await.get_mut(address) {
            // Note: In a real implementation, this would send a subscription request
            sub.active = true;
            sub.ws_subscription_id = Some(1); // Placeholder
            debug!(address = %address, "Activated subscription");
        }
    }

    /// Deactivates a subscription.
    async fn deactivate_subscription(&self, sub: &Subscription) {
        // Note: In a real implementation, this would send an unsubscribe request
        debug!(address = %sub.address, "Deactivated subscription");
    }

    /// Checks if connected.
    pub async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }

    /// Gets all subscriptions.
    pub async fn get_subscriptions(&self) -> Vec<Subscription> {
        self.subscriptions.read().await.values().cloned().collect()
    }

    /// Gets subscription count.
    pub async fn subscription_count(&self) -> usize {
        self.subscriptions.read().await.len()
    }

    /// Simulates an account update (for testing).
    pub async fn simulate_update(&self, update: AccountUpdate) {
        if let Err(e) = self.update_tx.send(update).await {
            error!(error = %e, "Failed to send simulated update");
        }
    }
}

impl Default for AccountListener {
    fn default() -> Self {
        Self::new(AccountListenerConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_account_listener_subscribe() {
        let listener = AccountListener::default();
        let address = Pubkey::new_unique();

        listener.subscribe(address, SubscriptionType::Pool).await;

        let subs = listener.get_subscriptions().await;
        assert_eq!(subs.len(), 1);
        assert_eq!(subs[0].address, address);
    }

    #[tokio::test]
    async fn test_account_listener_unsubscribe() {
        let listener = AccountListener::default();
        let address = Pubkey::new_unique();

        listener.subscribe(address, SubscriptionType::Pool).await;
        listener.unsubscribe(&address).await;

        assert_eq!(listener.subscription_count().await, 0);
    }
}
