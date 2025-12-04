pub mod rpc;
pub mod solana_client;
pub mod parsers;
pub mod raydium;
pub mod orca; // Whirlpools

use async_trait::async_trait;
use amm_domain::entities::pool::Pool;
use anyhow::Result;

#[async_trait]
pub trait PoolFetcher {
    async fn fetch_pool(&self, pool_address: &str) -> Result<Pool>;
}
