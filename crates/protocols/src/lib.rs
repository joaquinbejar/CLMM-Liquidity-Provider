pub mod orca;
pub mod parsers;
pub mod raydium;
pub mod rpc;
pub mod solana_client; // Whirlpools

use amm_domain::entities::pool::Pool;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait PoolFetcher {
    async fn fetch_pool(&self, pool_address: &str) -> Result<Pool>;
}
