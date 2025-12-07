//! Prelude module for convenient imports.
//!
//! This module re-exports the most commonly used types from the crate.
//!
//! # Example
//!
//! ```rust
//! use clmm_lp_execution::prelude::*;
//! ```

// Alerts
pub use crate::alerts::{
    Alert, AlertData, AlertLevel, AlertRule, AlertType, ConsoleNotifier, FileNotifier,
    MultiNotifier, Notifier, RuleCondition, RuleContext, RulesEngine, WebhookNotifier,
};

// Emergency
pub use crate::emergency::{
    CircuitBreaker, CircuitBreakerConfig, CircuitBreakerStats, CircuitState, EmergencyExitConfig,
    EmergencyExitManager, ExitResult, ExitStatus,
};

// Lifecycle
pub use crate::lifecycle::{
    AggregateStats, CloseReason, EventData, FeesCollectedData, LifecycleEvent, LifecycleEventType,
    LifecycleTracker, LiquidityChangeData, PositionClosedData, PositionOpenedData, PositionSummary,
    RebalanceData, RebalanceReason,
};

// Monitor
pub use crate::monitor::{
    MonitorConfig, MonitoredPosition, PnLResult, PnLTracker, PortfolioMetrics, PositionEntry,
    PositionMonitor, PositionPnL, ReconcileResult, StateSynchronizer, SyncState,
};

// Scheduler
pub use crate::scheduler::{Schedule, ScheduleBuilder, ScheduledTask, Scheduler, TaskEvent};

// Strategy
pub use crate::strategy::{
    Decision, DecisionConfig, DecisionContext, DecisionEngine, ExecutorConfig, ProfitabilityCheck,
    RebalanceConfig, RebalanceExecutor, RebalanceParams, RebalanceResult, StrategyExecutor,
};

// Sync
pub use crate::sync::{
    AccountListener, AccountListenerConfig, AccountState, AccountUpdate, ReconcileStatus,
    Reconciler, ReconcilerConfig, Subscription, SubscriptionType,
};

// Transaction
pub use crate::transaction::{
    PriorityLevel, SimulationResult, TransactionBuilder, TransactionConfig, TransactionManager,
    TransactionResult, TransactionStatus,
};

// Wallet
pub use crate::wallet::{Wallet, WalletManager};
