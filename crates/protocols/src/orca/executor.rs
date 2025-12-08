//! Whirlpool executor for on-chain operations.
//!
//! Provides functionality to execute LP operations on Orca Whirlpools:
//! - Open positions
//! - Increase/decrease liquidity
//! - Collect fees
//! - Close positions

use crate::rpc::RpcProvider;
use anyhow::{Context, Result};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Signature,
    signer::Signer,
    transaction::Transaction,
};
use std::str::FromStr;
use std::sync::Arc;
use tracing::{debug, info};

/// Orca Whirlpool program ID (mainnet).
pub const WHIRLPOOL_PROGRAM_ID: &str = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";

/// Token program ID.
pub const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

/// Associated token program ID.
pub const ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

/// System program ID.
pub const SYSTEM_PROGRAM_ID: &str = "11111111111111111111111111111111";

/// Parameters for opening a new position.
#[derive(Debug, Clone)]
pub struct OpenPositionParams {
    /// Pool address.
    pub pool: Pubkey,
    /// Lower tick bound.
    pub tick_lower: i32,
    /// Upper tick bound.
    pub tick_upper: i32,
    /// Amount of token A to deposit.
    pub amount_a: u64,
    /// Amount of token B to deposit.
    pub amount_b: u64,
    /// Slippage tolerance in basis points.
    pub slippage_bps: u16,
}

/// Parameters for increasing liquidity.
#[derive(Debug, Clone)]
pub struct IncreaseLiquidityParams {
    /// Position address.
    pub position: Pubkey,
    /// Pool address.
    pub pool: Pubkey,
    /// Liquidity amount to add.
    pub liquidity_amount: u128,
    /// Maximum token A amount.
    pub token_max_a: u64,
    /// Maximum token B amount.
    pub token_max_b: u64,
}

/// Parameters for decreasing liquidity.
#[derive(Debug, Clone)]
pub struct DecreaseLiquidityParams {
    /// Position address.
    pub position: Pubkey,
    /// Pool address.
    pub pool: Pubkey,
    /// Liquidity amount to remove.
    pub liquidity_amount: u128,
    /// Minimum token A amount.
    pub token_min_a: u64,
    /// Minimum token B amount.
    pub token_min_b: u64,
}

/// Result of an execution operation.
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Transaction signature.
    pub signature: Signature,
    /// Whether the transaction was successful.
    pub success: bool,
    /// Slot at which the transaction was confirmed.
    pub slot: Option<u64>,
    /// Error message if failed.
    pub error: Option<String>,
}

impl ExecutionResult {
    /// Creates a successful result.
    #[must_use]
    pub fn success(signature: Signature, slot: u64) -> Self {
        Self {
            signature,
            success: true,
            slot: Some(slot),
            error: None,
        }
    }

    /// Creates a failed result.
    #[must_use]
    pub fn failure(signature: Signature, error: String) -> Self {
        Self {
            signature,
            success: false,
            slot: None,
            error: Some(error),
        }
    }
}

/// Executor for Orca Whirlpool operations.
pub struct WhirlpoolExecutor {
    /// RPC provider for blockchain interaction.
    provider: Arc<RpcProvider>,
    /// Whirlpool program ID.
    program_id: Pubkey,
    /// Token program ID.
    token_program: Pubkey,
    /// Associated token program ID.
    ata_program: Pubkey,
    /// System program ID.
    system_program: Pubkey,
}

impl WhirlpoolExecutor {
    /// Creates a new WhirlpoolExecutor.
    pub fn new(provider: Arc<RpcProvider>) -> Self {
        Self {
            provider,
            program_id: Pubkey::from_str(WHIRLPOOL_PROGRAM_ID).expect("Invalid program ID"),
            token_program: Pubkey::from_str(TOKEN_PROGRAM_ID).expect("Invalid token program ID"),
            ata_program: Pubkey::from_str(ASSOCIATED_TOKEN_PROGRAM_ID)
                .expect("Invalid ATA program ID"),
            system_program: Pubkey::from_str(SYSTEM_PROGRAM_ID).expect("Invalid system program ID"),
        }
    }

    /// Opens a new position in a Whirlpool.
    ///
    /// # Arguments
    /// * `params` - Position parameters
    /// * `payer` - Transaction payer and position owner
    ///
    /// # Returns
    /// Execution result with transaction signature.
    pub async fn open_position<S: Signer>(
        &self,
        params: &OpenPositionParams,
        payer: &S,
    ) -> Result<ExecutionResult> {
        info!(
            pool = %params.pool,
            tick_lower = params.tick_lower,
            tick_upper = params.tick_upper,
            "Opening new position"
        );

        // Derive position mint PDA
        let position_mint =
            self.derive_position_mint(&params.pool, params.tick_lower, params.tick_upper)?;

        // Derive position PDA
        let (position_pda, _bump) =
            Pubkey::find_program_address(&[b"position", position_mint.as_ref()], &self.program_id);

        // Build open position instruction
        let open_ix = self.build_open_position_instruction(
            params,
            &payer.pubkey(),
            &position_mint,
            &position_pda,
        )?;

        // Build increase liquidity instruction
        let increase_ix = self.build_increase_liquidity_instruction(
            &position_pda,
            &params.pool,
            &payer.pubkey(),
            params.amount_a,
            params.amount_b,
        )?;

        // Create and send transaction
        let instructions = vec![open_ix, increase_ix];
        self.send_transaction(&instructions, payer).await
    }

    /// Increases liquidity in an existing position.
    pub async fn increase_liquidity<S: Signer>(
        &self,
        params: &IncreaseLiquidityParams,
        payer: &S,
    ) -> Result<ExecutionResult> {
        info!(
            position = %params.position,
            liquidity = params.liquidity_amount,
            "Increasing liquidity"
        );

        let ix = self.build_increase_liquidity_instruction(
            &params.position,
            &params.pool,
            &payer.pubkey(),
            params.token_max_a,
            params.token_max_b,
        )?;

        self.send_transaction(&[ix], payer).await
    }

    /// Decreases liquidity from an existing position.
    pub async fn decrease_liquidity<S: Signer>(
        &self,
        params: &DecreaseLiquidityParams,
        payer: &S,
    ) -> Result<ExecutionResult> {
        info!(
            position = %params.position,
            liquidity = params.liquidity_amount,
            "Decreasing liquidity"
        );

        let ix = self.build_decrease_liquidity_instruction(
            &params.position,
            &params.pool,
            &payer.pubkey(),
            params.liquidity_amount,
            params.token_min_a,
            params.token_min_b,
        )?;

        self.send_transaction(&[ix], payer).await
    }

    /// Collects fees from a position.
    pub async fn collect_fees<S: Signer>(
        &self,
        position: &Pubkey,
        pool: &Pubkey,
        payer: &S,
    ) -> Result<ExecutionResult> {
        info!(position = %position, "Collecting fees");

        let ix = self.build_collect_fees_instruction(position, pool, &payer.pubkey())?;

        self.send_transaction(&[ix], payer).await
    }

    /// Closes a position.
    pub async fn close_position<S: Signer>(
        &self,
        position: &Pubkey,
        pool: &Pubkey,
        payer: &S,
    ) -> Result<ExecutionResult> {
        info!(position = %position, "Closing position");

        // First decrease all liquidity
        let decrease_ix = self.build_decrease_liquidity_instruction(
            position,
            pool,
            &payer.pubkey(),
            u128::MAX, // All liquidity
            0,         // Min token A
            0,         // Min token B
        )?;

        // Collect any remaining fees
        let collect_ix = self.build_collect_fees_instruction(position, pool, &payer.pubkey())?;

        // Close the position
        let close_ix = self.build_close_position_instruction(position, &payer.pubkey())?;

        let instructions = vec![decrease_ix, collect_ix, close_ix];
        self.send_transaction(&instructions, payer).await
    }

    /// Simulates a transaction without broadcasting.
    pub async fn simulate_transaction<S: Signer>(
        &self,
        instructions: &[Instruction],
        payer: &S,
    ) -> Result<bool> {
        debug!(
            "Simulating transaction with {} instructions",
            instructions.len()
        );

        let recent_blockhash = self
            .provider
            .get_latest_blockhash()
            .await
            .context("Failed to get recent blockhash")?;

        let transaction = Transaction::new_signed_with_payer(
            instructions,
            Some(&payer.pubkey()),
            &[payer],
            recent_blockhash,
        );

        let result = self
            .provider
            .simulate_transaction(&transaction)
            .await
            .context("Failed to simulate transaction")?;

        if let Some(err) = result.err {
            debug!("Simulation failed: {:?}", err);
            return Ok(false);
        }

        debug!("Simulation successful");
        Ok(true)
    }

    // Private helper methods

    fn derive_position_mint(
        &self,
        pool: &Pubkey,
        tick_lower: i32,
        tick_upper: i32,
    ) -> Result<Pubkey> {
        let (mint, _bump) = Pubkey::find_program_address(
            &[
                b"position_mint",
                pool.as_ref(),
                &tick_lower.to_le_bytes(),
                &tick_upper.to_le_bytes(),
            ],
            &self.program_id,
        );
        Ok(mint)
    }

    fn build_open_position_instruction(
        &self,
        params: &OpenPositionParams,
        owner: &Pubkey,
        position_mint: &Pubkey,
        position: &Pubkey,
    ) -> Result<Instruction> {
        // Whirlpool OpenPosition instruction discriminator
        let discriminator: [u8; 8] = [0x87, 0x80, 0x2f, 0x4d, 0x0f, 0x98, 0xf0, 0x31];

        let mut data = Vec::with_capacity(24);
        data.extend_from_slice(&discriminator);
        data.extend_from_slice(&params.tick_lower.to_le_bytes());
        data.extend_from_slice(&params.tick_upper.to_le_bytes());

        // Derive position token account
        let position_token_account = self.derive_ata(owner, position_mint)?;

        let accounts = vec![
            AccountMeta::new(*owner, true),                        // funder
            AccountMeta::new_readonly(*owner, false),              // owner
            AccountMeta::new(*position, false),                    // position
            AccountMeta::new(*position_mint, true),                // position_mint
            AccountMeta::new(position_token_account, false),       // position_token_account
            AccountMeta::new_readonly(params.pool, false),         // whirlpool
            AccountMeta::new_readonly(self.token_program, false),  // token_program
            AccountMeta::new_readonly(self.system_program, false), // system_program
            AccountMeta::new_readonly(solana_sdk::sysvar::rent::ID, false), // rent
            AccountMeta::new_readonly(self.ata_program, false),    // associated_token_program
        ];

        Ok(Instruction {
            program_id: self.program_id,
            accounts,
            data,
        })
    }

    fn build_increase_liquidity_instruction(
        &self,
        position: &Pubkey,
        pool: &Pubkey,
        owner: &Pubkey,
        token_max_a: u64,
        token_max_b: u64,
    ) -> Result<Instruction> {
        // Whirlpool IncreaseLiquidity instruction discriminator
        let discriminator: [u8; 8] = [0x2e, 0x9c, 0xf3, 0x76, 0x0d, 0xc6, 0x1e, 0x84];

        let mut data = Vec::with_capacity(40);
        data.extend_from_slice(&discriminator);
        data.extend_from_slice(&0u128.to_le_bytes()); // liquidity_amount (calculated by program)
        data.extend_from_slice(&token_max_a.to_le_bytes());
        data.extend_from_slice(&token_max_b.to_le_bytes());

        let accounts = vec![
            AccountMeta::new(*pool, false),                       // whirlpool
            AccountMeta::new_readonly(self.token_program, false), // token_program
            AccountMeta::new_readonly(*owner, true),              // position_authority
            AccountMeta::new(*position, false),                   // position
                                                                  // Additional accounts would be derived from pool state
                                                                  // token_owner_account_a, token_owner_account_b, token_vault_a, token_vault_b, tick_array_lower, tick_array_upper
        ];

        Ok(Instruction {
            program_id: self.program_id,
            accounts,
            data,
        })
    }

    fn build_decrease_liquidity_instruction(
        &self,
        position: &Pubkey,
        pool: &Pubkey,
        owner: &Pubkey,
        liquidity_amount: u128,
        token_min_a: u64,
        token_min_b: u64,
    ) -> Result<Instruction> {
        // Whirlpool DecreaseLiquidity instruction discriminator
        let discriminator: [u8; 8] = [0xa0, 0x26, 0xd0, 0x6f, 0x68, 0x5b, 0x2c, 0x01];

        let mut data = Vec::with_capacity(40);
        data.extend_from_slice(&discriminator);
        data.extend_from_slice(&liquidity_amount.to_le_bytes());
        data.extend_from_slice(&token_min_a.to_le_bytes());
        data.extend_from_slice(&token_min_b.to_le_bytes());

        let accounts = vec![
            AccountMeta::new(*pool, false),                       // whirlpool
            AccountMeta::new_readonly(self.token_program, false), // token_program
            AccountMeta::new_readonly(*owner, true),              // position_authority
            AccountMeta::new(*position, false),                   // position
                                                                  // Additional accounts derived from pool state
        ];

        Ok(Instruction {
            program_id: self.program_id,
            accounts,
            data,
        })
    }

    fn build_collect_fees_instruction(
        &self,
        position: &Pubkey,
        pool: &Pubkey,
        owner: &Pubkey,
    ) -> Result<Instruction> {
        // Whirlpool CollectFees instruction discriminator
        let discriminator: [u8; 8] = [0xa4, 0x98, 0xcf, 0x63, 0x1e, 0xba, 0x13, 0x7a];

        let data = discriminator.to_vec();

        let accounts = vec![
            AccountMeta::new(*pool, false),          // whirlpool
            AccountMeta::new_readonly(*owner, true), // position_authority
            AccountMeta::new(*position, false),      // position
            AccountMeta::new_readonly(self.token_program, false), // token_program
                                                     // Additional accounts: token_owner_account_a, token_owner_account_b, token_vault_a, token_vault_b
        ];

        Ok(Instruction {
            program_id: self.program_id,
            accounts,
            data,
        })
    }

    fn build_close_position_instruction(
        &self,
        position: &Pubkey,
        owner: &Pubkey,
    ) -> Result<Instruction> {
        // Whirlpool ClosePosition instruction discriminator
        let discriminator: [u8; 8] = [0x7b, 0x86, 0x51, 0x0c, 0x31, 0x5b, 0xfc, 0x00];

        let data = discriminator.to_vec();

        let accounts = vec![
            AccountMeta::new_readonly(*owner, true), // position_authority
            AccountMeta::new(*owner, false),         // receiver
            AccountMeta::new(*position, false),      // position
                                                     // position_mint, position_token_account, token_program
        ];

        Ok(Instruction {
            program_id: self.program_id,
            accounts,
            data,
        })
    }

    fn derive_ata(&self, owner: &Pubkey, mint: &Pubkey) -> Result<Pubkey> {
        let (ata, _bump) = Pubkey::find_program_address(
            &[owner.as_ref(), self.token_program.as_ref(), mint.as_ref()],
            &self.ata_program,
        );
        Ok(ata)
    }

    async fn send_transaction<S: Signer>(
        &self,
        instructions: &[Instruction],
        payer: &S,
    ) -> Result<ExecutionResult> {
        let recent_blockhash = self
            .provider
            .get_latest_blockhash()
            .await
            .context("Failed to get recent blockhash")?;

        let transaction = Transaction::new_signed_with_payer(
            instructions,
            Some(&payer.pubkey()),
            &[payer],
            recent_blockhash,
        );

        debug!("Sending transaction...");

        match self
            .provider
            .send_and_confirm_transaction(&transaction)
            .await
        {
            Ok(signature) => {
                info!(signature = %signature, "Transaction confirmed");
                // Get slot from transaction status
                let slot = self.provider.get_slot().await.unwrap_or(0);
                Ok(ExecutionResult::success(signature, slot))
            }
            Err(e) => {
                let signature = transaction.signatures.first().copied().unwrap_or_default();
                Ok(ExecutionResult::failure(signature, e.to_string()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_program_ids() {
        assert!(Pubkey::from_str(WHIRLPOOL_PROGRAM_ID).is_ok());
        assert!(Pubkey::from_str(TOKEN_PROGRAM_ID).is_ok());
        assert!(Pubkey::from_str(ASSOCIATED_TOKEN_PROGRAM_ID).is_ok());
    }

    #[test]
    fn test_execution_result() {
        let sig = Signature::default();

        let success = ExecutionResult::success(sig, 12345);
        assert!(success.success);
        assert_eq!(success.slot, Some(12345));
        assert!(success.error.is_none());

        let failure = ExecutionResult::failure(sig, "test error".to_string());
        assert!(!failure.success);
        assert!(failure.slot.is_none());
        assert_eq!(failure.error, Some("test error".to_string()));
    }
}
