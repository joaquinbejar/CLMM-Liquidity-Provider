//! Circuit breaker for automated trading safety.

use rust_decimal::Decimal;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{error, info};

/// Circuit breaker state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed (normal operation).
    Closed,
    /// Circuit is open (operations blocked).
    Open,
    /// Circuit is half-open (testing recovery).
    HalfOpen,
}

/// Configuration for the circuit breaker.
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Maximum consecutive failures before opening circuit.
    pub max_failures: u32,
    /// Maximum loss percentage before opening circuit.
    pub max_loss_pct: Decimal,
    /// Maximum priority fee in lamports before opening circuit.
    pub max_priority_fee_lamports: u64,
    /// Time to wait before attempting recovery in seconds.
    pub recovery_timeout_secs: u64,
    /// Number of successful operations to close circuit.
    pub success_threshold: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            max_failures: 3,
            max_loss_pct: Decimal::new(10, 2),      // 10%
            max_priority_fee_lamports: 100_000_000, // 0.1 SOL
            recovery_timeout_secs: 300,             // 5 minutes
            success_threshold: 2,
        }
    }
}

/// Circuit breaker for protecting against cascading failures.
pub struct CircuitBreaker {
    /// Current state.
    state: Arc<RwLock<CircuitState>>,
    /// Configuration.
    config: CircuitBreakerConfig,
    /// Consecutive failure count.
    failure_count: AtomicU32,
    /// Consecutive success count (for half-open state).
    success_count: AtomicU32,
    /// Time when circuit was opened.
    opened_at: Arc<RwLock<Option<Instant>>>,
    /// Manual trip flag.
    manually_tripped: AtomicBool,
    /// Callback for state changes.
    #[allow(dead_code)]
    on_state_change: Option<Box<dyn Fn(CircuitState) + Send + Sync>>,
}

impl CircuitBreaker {
    /// Creates a new circuit breaker.
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            config,
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
            opened_at: Arc::new(RwLock::new(None)),
            manually_tripped: AtomicBool::new(false),
            on_state_change: None,
        }
    }

    /// Checks if operations are allowed.
    pub async fn is_allowed(&self) -> bool {
        // Check manual trip first
        if self.manually_tripped.load(Ordering::SeqCst) {
            return false;
        }

        let state = *self.state.read().await;

        match state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if recovery timeout has passed
                if let Some(opened_at) = *self.opened_at.read().await {
                    let elapsed = opened_at.elapsed();
                    if elapsed >= Duration::from_secs(self.config.recovery_timeout_secs) {
                        // Transition to half-open
                        self.transition_to(CircuitState::HalfOpen).await;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Records a successful operation.
    pub async fn record_success(&self) {
        self.failure_count.store(0, Ordering::SeqCst);

        let state = *self.state.read().await;
        if state == CircuitState::HalfOpen {
            let count = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;
            if count >= self.config.success_threshold {
                self.transition_to(CircuitState::Closed).await;
                self.success_count.store(0, Ordering::SeqCst);
                info!("Circuit breaker closed after successful recovery");
            }
        }
    }

    /// Records a failed operation.
    pub async fn record_failure(&self) {
        let count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
        self.success_count.store(0, Ordering::SeqCst);

        let state = *self.state.read().await;

        match state {
            CircuitState::Closed => {
                if count >= self.config.max_failures {
                    self.trip("consecutive failures exceeded threshold").await;
                }
            }
            CircuitState::HalfOpen => {
                // Any failure in half-open state reopens the circuit
                self.trip("failure during recovery").await;
            }
            CircuitState::Open => {
                // Already open, nothing to do
            }
        }
    }

    /// Checks if a loss exceeds the threshold.
    pub async fn check_loss(&self, loss_pct: Decimal) -> bool {
        if loss_pct.abs() > self.config.max_loss_pct {
            self.trip(&format!("loss exceeded threshold: {}%", loss_pct))
                .await;
            false
        } else {
            true
        }
    }

    /// Checks if priority fee exceeds the threshold.
    pub async fn check_priority_fee(&self, fee_lamports: u64) -> bool {
        if fee_lamports > self.config.max_priority_fee_lamports {
            self.trip(&format!(
                "priority fee exceeded threshold: {} lamports",
                fee_lamports
            ))
            .await;
            false
        } else {
            true
        }
    }

    /// Manually trips the circuit breaker.
    pub async fn manual_trip(&self, reason: &str) {
        self.manually_tripped.store(true, Ordering::SeqCst);
        self.trip(&format!("manual trip: {}", reason)).await;
    }

    /// Resets the manual trip flag.
    pub fn reset_manual_trip(&self) {
        self.manually_tripped.store(false, Ordering::SeqCst);
        info!("Manual trip flag reset");
    }

    /// Trips the circuit breaker.
    async fn trip(&self, reason: &str) {
        error!(reason = reason, "Circuit breaker tripped");
        self.transition_to(CircuitState::Open).await;
        *self.opened_at.write().await = Some(Instant::now());
        self.failure_count.store(0, Ordering::SeqCst);
    }

    /// Transitions to a new state.
    async fn transition_to(&self, new_state: CircuitState) {
        let mut state = self.state.write().await;
        let old_state = *state;

        if old_state != new_state {
            *state = new_state;
            info!(
                old_state = ?old_state,
                new_state = ?new_state,
                "Circuit breaker state changed"
            );
        }
    }

    /// Gets the current state.
    pub async fn state(&self) -> CircuitState {
        *self.state.read().await
    }

    /// Resets the circuit breaker to closed state.
    pub async fn reset(&self) {
        self.transition_to(CircuitState::Closed).await;
        self.failure_count.store(0, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        self.manually_tripped.store(false, Ordering::SeqCst);
        *self.opened_at.write().await = None;
        info!("Circuit breaker reset");
    }

    /// Gets circuit breaker statistics.
    pub async fn stats(&self) -> CircuitBreakerStats {
        CircuitBreakerStats {
            state: *self.state.read().await,
            failure_count: self.failure_count.load(Ordering::SeqCst),
            success_count: self.success_count.load(Ordering::SeqCst),
            manually_tripped: self.manually_tripped.load(Ordering::SeqCst),
            opened_at: *self.opened_at.read().await,
        }
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }
}

/// Statistics for the circuit breaker.
#[derive(Debug, Clone)]
pub struct CircuitBreakerStats {
    /// Current state.
    pub state: CircuitState,
    /// Current failure count.
    pub failure_count: u32,
    /// Current success count.
    pub success_count: u32,
    /// Whether manually tripped.
    pub manually_tripped: bool,
    /// When circuit was opened.
    pub opened_at: Option<Instant>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_closed() {
        let cb = CircuitBreaker::default();
        assert!(cb.is_allowed().await);
        assert_eq!(cb.state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_trips_on_failures() {
        let config = CircuitBreakerConfig {
            max_failures: 2,
            ..Default::default()
        };
        let cb = CircuitBreaker::new(config);

        cb.record_failure().await;
        assert!(cb.is_allowed().await);

        cb.record_failure().await;
        assert!(!cb.is_allowed().await);
        assert_eq!(cb.state().await, CircuitState::Open);
    }

    #[tokio::test]
    async fn test_circuit_breaker_manual_trip() {
        let cb = CircuitBreaker::default();

        cb.manual_trip("test").await;
        assert!(!cb.is_allowed().await);

        cb.reset_manual_trip();
        // Still open due to trip, but manual flag is cleared
        assert!(!cb.is_allowed().await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_reset() {
        let cb = CircuitBreaker::default();

        cb.manual_trip("test").await;
        assert!(!cb.is_allowed().await);

        cb.reset().await;
        assert!(cb.is_allowed().await);
    }
}
