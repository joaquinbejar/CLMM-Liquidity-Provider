use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Protocol {
    Raydium,
    OrcaWhirlpools,
    OrcaLegacy,
    MeteoraDLMM,
    MeteoraStable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PoolType {
    ConstantProduct,
    ConcentratedLiquidity,
    StableSwap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionStatus {
    Open,
    Closed,
    OutOfRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationObjective {
    MaximizeFeeYield,
    MinimizeImpermanentLoss,
    MaximizeSharpeRatio,
    MaximizeNetReturn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeHorizon {
    Days(u32),
    Weeks(u32),
    Months(u32),
}
