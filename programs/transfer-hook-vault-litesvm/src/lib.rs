#![allow(unexpected_cfgs)]
#![allow(deprecated)]
use anchor_lang::prelude::*;

mod instructions;
mod states;

use instructions::*;
use states::*;

use spl_discriminator::SplDiscriminate;
use spl_transfer_hook_interface::{
    instruction::{
        ExecuteInstruction, 
        InitializeExtraAccountMetaListInstruction

    },
};
use spl_tlv_account_resolution::state::ExtraAccountMetaList;

declare_id!("3ZktYLqFLBuLdzsnH66siTLjZctKzj1aZHvG9KWyaF1x");

#[program]
pub mod transfer_hook_vault_litesvm {
    use super::*;
    pub fn add_restricted_account(ctx: Context<WhitelistOperations>, user: Pubkey) -> Result<()> {
        ctx.accounts.add_restricted_account(ctx.bumps)
    }
    pub fn remove_restricted_account(ctx: Context<WhitelistOperations>, user: Pubkey) -> Result<()> {
        ctx.accounts.remove_restricted_account(ctx.bumps)
    }

    pub fn init_mint(ctx: Context<TokenFactory>, decimals: u8, mint_authority: Pubkey) -> Result<()> {
        ctx.accounts.init_mint(&ctx.bumps, decimals, mint_authority)
    }

    #[instruction(discriminator = InitializeExtraAccountMetaListInstruction::SPL_DISCRIMINATOR_SLICE)]
    pub fn initialize_transfer_hook(ctx: Context<InitializeExtraAccountMetaList>) -> Result<()> {

        // Get the extra account metas for the transfer hook
        let extra_account_metas = InitializeExtraAccountMetaList::extra_account_metas()?;

        // initialize ExtraAccountMetaList account with extra accounts
        ExtraAccountMetaList::init::<ExecuteInstruction>(
            &mut ctx.accounts.extra_account_meta_list.try_borrow_mut_data()?,
            &extra_account_metas
        )?;

        Ok(())
    }

    #[instruction(discriminator = ExecuteInstruction::SPL_DISCRIMINATOR_SLICE)]
    pub fn transfer_hook(ctx: Context<TransferHook>, amount: u64) -> Result<()> {
        // Call the transfer hook logic
        ctx.accounts.transfer_hook(amount)

    }
}





