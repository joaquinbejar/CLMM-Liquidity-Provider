use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::account::Account;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use std::sync::Arc;

pub struct SolanaRpcAdapter {
    pub client: Arc<RpcClient>,
}

impl SolanaRpcAdapter {
    pub fn new(rpc_url: &str) -> Self {
        Self {
            client: Arc::new(RpcClient::new(rpc_url.to_string())),
        }
    }

    pub async fn get_account_data(&self, address: &str) -> Result<Vec<u8>> {
        let pubkey = Pubkey::from_str(address).map_err(|_| anyhow::anyhow!("Invalid pubkey"))?;
        let account = self.client.get_account(&pubkey).await?;
        Ok(account.data)
    }

    pub async fn get_multiple_accounts(
        &self,
        addresses: &[String],
    ) -> Result<Vec<Option<Account>>> {
        let pubkeys: Vec<Pubkey> = addresses
            .iter()
            .filter_map(|s| Pubkey::from_str(s).ok())
            .collect();

        if pubkeys.is_empty() {
            return Ok(vec![]);
        }

        let accounts = self.client.get_multiple_accounts(&pubkeys).await?;
        Ok(accounts)
    }
}
