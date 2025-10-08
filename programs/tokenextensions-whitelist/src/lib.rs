#![allow(unexpected_cfgs)]
#![allow(deprecated)]
use anchor_lang::prelude::*;

mod instructions;
mod states;

use instructions::*;

use spl_discriminator::SplDiscriminate;
use spl_transfer_hook_interface::{
    instruction::{
        ExecuteInstruction, 
        InitializeExtraAccountMetaListInstruction

    },
};
use spl_tlv_account_resolution::state::ExtraAccountMetaList;

declare_id!("E5iLMqv3rqSzpu743Vz7uhWzcw476F1cj6VQatgoJRs9");

#[program]
pub mod tokenextensions_whitelist {
    use super::*;

    pub fn initialize_whitelist(ctx: Context<InitializeWhitelist>) -> Result<()> {
        ctx.accounts.initialize_whitelist(ctx.bumps)
    }


    pub fn add_to_whitelist(ctx: Context<WhitelistOperations>, user: Pubkey) -> Result<()> {
        ctx.accounts.add_to_whitelist(user)
    }

    pub fn remove_from_whitelist(ctx: Context<WhitelistOperations>, user: Pubkey) -> Result<()> {
        ctx.accounts.remove_from_whitelist(user)
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
