use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{
    states::Vault, 
    VAULT_AUTH_SEED, 
    VAULT_STATE_SEED, 
    VaultError
};

/// Initializes Vault state + PDA authority + PDA ATA
#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: PDA authority (no data)
    #[account(seeds=[VAULT_AUTH_SEED, mint.key().as_ref()], bump)]
    pub vault_authority: UncheckedAccount<'info>,

    #[account(
        init,
        payer = payer,

        space = 8 + Vault::INIT_SPACE,
        seeds = [VAULT_STATE_SEED, mint.key().as_ref()],
        bump
    )]

    pub vault_state: Account<'info, Vault>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = payer,
        associated_token::token_program = token_program,
        associated_token::mint = mint,
        associated_token::authority = vault_authority
    )]
    pub vault_ata: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeVault<'info> {
    pub fn inititialize_vault(ctx: Context<Self>, owner: Pubkey) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.token_program.key(),
            anchor_spl::token_2022::ID,
            VaultError::InvalidTokenProgram
        );
        require_keys_eq!(
            *ctx.accounts.mint.to_account_info().owner,
            ctx.accounts.token_program.key(),
            VaultError::InvalidTokenProgram
        );


        let bump = ctx.bumps.vault_authority;
        let st = &mut ctx.accounts.vault_state;

        st.mint = ctx.accounts.mint.key();
        st.vault_bump = bump;
        st.owner = owner;
        Ok(())
    }
}

