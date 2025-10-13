use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::states::Vault;

#[derive(Accounts)]
pub struct Init<'info> {

    #[account(mut)]
    pub admin: Signer<'info>,
    pub underlying_token: InterfaceAccount<'info, Mint>,
    #[account(
        init,
        payer = admin,
        seeds = [b"vault"],
        bump,
        space = 8 + Vault::INIT_SPACE,
    )]
    pub vault: Account<'info, Vault>,
    #[account(
        init,
        payer = admin,
        associated_token::mint = underlying_token,
        associated_token::authority = vault,

    )]
    pub vault_underlying_ata: InterfaceAccount<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Init<'info> {
    pub fn init_vault(&mut self, bumps: &InitBumps) -> Result<()> {

        self.vault.set_inner(Vault {
            admin: self.admin.key(),
            underlying_token: self.underlying_token.key(),
            bump: bumps.vault,
        });

        Ok(())
    }
}
