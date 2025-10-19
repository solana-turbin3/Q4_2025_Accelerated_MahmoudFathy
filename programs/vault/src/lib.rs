#![allow(unexpected_cfgs)]
#![allow(deprecated)]

pub mod instructions;
pub mod states;

use anchor_lang::prelude::*;


pub use instructions::*;
pub use states::*;


declare_id!("B5ufbureiHZzc2WTW6CVqsdpQnNPyWLbBUUa9KweTxYJ");

pub const VAULT_AUTH_SEED: &[u8] = b"vault";
pub const VAULT_STATE_SEED: &[u8] = b"vault_state";

#[error_code]
pub enum VaultError {
    InvalidTokenProgram,
    InvalidAmountZero,
    VaultAlreadyExist,
}

#[program]
pub mod vault {
    use super::*;

    pub fn initialize(ctx: Context<InitializeVault>, owner: Pubkey) -> Result<()> {
        InitializeVault::inititialize_vault(ctx, owner)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        Deposit::deposit(ctx, amount)
    }


    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        Withdraw::withdraw(ctx, amount)

    }
}

