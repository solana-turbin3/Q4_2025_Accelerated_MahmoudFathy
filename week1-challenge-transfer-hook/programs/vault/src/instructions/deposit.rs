use anchor_lang::prelude::*;
use anchor_spl::{
    token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked},
};

use crate::{
    states::Vault,
    VAULT_AUTH_SEED,
    VAULT_STATE_SEED,
    VaultError
};

/// User → Vault ATA
#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds=[VAULT_STATE_SEED, mint.key().as_ref()],
        bump,
        has_one = mint
    )]
    pub vault_state: Account<'info, Vault>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::token_program = token_program,
        associated_token::mint = mint,
        associated_token::authority = user
    )]

    pub user_ata: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: PDA authority
    #[account(
        seeds=[VAULT_AUTH_SEED, mint.key().as_ref()],
        bump = vault_state.vault_bump
    )]
    pub vault_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::token_program = token_program,
        associated_token::mint = mint,
        associated_token::authority = vault_authority

    )]
    pub vault_ata: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: ExtraAccountMetaList PDA of the hook program (["extra-account-metas", mint])

    pub extra_account_meta_list: UncheckedAccount<'info>,
    /// CHECK: Whitelist PDA of the hook program (["whitelist", mint, source])
    pub source_token_whitelist_state: UncheckedAccount<'info>,
    /// CHECK: Whitelist PDA of the hook program (["whitelist", mint, destination])
    pub destination_token_whitelist_state: UncheckedAccount<'info>,

    pub token_program: Interface<'info, TokenInterface>,
}


impl<'info> Deposit<'info> {

    pub fn deposit(ctx: Context<Self>, amount: u64) -> Result<()> {
        require!(amount > 0, VaultError::InvalidAmountZero);
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

        let cpi = CpiContext::new(

            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.user_ata.to_account_info(),
                to:   ctx.accounts.vault_ata.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
            }
        )
        .with_remaining_accounts(vec![
            ctx.accounts.extra_account_meta_list.to_account_info(),
            ctx.accounts.source_token_whitelist_state.to_account_info(),
            ctx.accounts.destination_token_whitelist_state.to_account_info(),
        ]);

        transfer_checked(cpi, amount, ctx.accounts.mint.decimals)
    }
}
