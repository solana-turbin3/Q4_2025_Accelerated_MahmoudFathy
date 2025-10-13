#![allow(unexpected_cfgs)]
#![allow(deprecated)]

pub mod instructions;
pub mod states;


use anchor_lang::prelude::*;


pub use instructions::*;
pub use states::*;

declare_id!("11111111111111111111111111111111");

#[constant]
pub const SEED: &str = "anchor";

#[program]
pub mod vault {
    use super::*;

    pub fn init_vault(ctx: Context<Init>) -> Result<()> {
        ctx.accounts.init_vault(&ctx.bumps)?;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)?;
        Ok(())

    }

    // pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    //     let bumps = ctx.bumps;
    //     ctx.accounts.withdraw(amount, &bumps)?;
    //     Ok(())
    // }
}

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,
}
