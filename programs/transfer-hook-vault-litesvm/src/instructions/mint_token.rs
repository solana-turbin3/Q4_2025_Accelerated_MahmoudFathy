use anchor_lang::{ 
    prelude::*, 
};
use anchor_spl::token_interface::{
    Mint, 
    TokenInterface,
};

use crate::states::RestrictedAccount;

#[derive(Accounts)]
pub struct TokenFactory<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        payer = user,
        mint::token_program = token_program,
        mint::decimals = 9,
        mint::authority = user,
        extensions::transfer_hook::authority = user,
        extensions::transfer_hook::program_id = crate::ID,
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    /// CHECK: ExtraAccountMetaList Account, will be checked by the transfer hook
    #[account(mut)]
    pub extra_account_meta_list: UncheckedAccount<'info>,

    #[account(
        seeds = [b"whitelist", mint.key().as_ref()], 
        bump
    )]
    pub blocklist: Account<'info, RestrictedAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> TokenFactory<'info> {
    pub fn init_mint(
        &mut self,
        _bumps: &TokenFactoryBumps,
        _decimals: u8,
        _mint_authority: Pubkey,
    ) -> Result<()> {
        Ok(())
    }
}
