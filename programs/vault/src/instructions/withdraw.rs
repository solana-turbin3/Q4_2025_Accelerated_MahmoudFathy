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

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut, address = vault_state.owner)]
    pub owner: Signer<'info>,


    #[account(
        seeds=[VAULT_STATE_SEED, mint.key().as_ref()],
        bump,
        has_one = mint
    )]
    pub vault_state: Account<'info, Vault>,

    pub mint: InterfaceAccount<'info, Mint>,

    /// CHECK: PDA signer
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

    #[account(
        mut,
        associated_token::token_program = token_program,
        associated_token::mint = mint,
        associated_token::authority = owner
    )]
    pub owner_ata: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: ExtraAccountMetaList PDA of the hook program
    pub extra_account_meta_list: UncheckedAccount<'info>,

    /// CHECK: Whitelist PDA of the hook program (["whitelist", mint, source])
    pub source_token_whitelist_state: UncheckedAccount<'info>,
    /// CHECK: Whitelist PDA of the hook program (["whitelist", mint, destination])
    pub destination_token_whitelist_state: UncheckedAccount<'info>,

    pub token_program: Interface<'info, TokenInterface>,

}


impl<'info> Withdraw<'info> {
    pub fn withdraw(ctx: Context<Self>, amount: u64) -> Result<()> {

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

        let mint_key = ctx.accounts.mint.key();
        // PDA seed array for the vault authority
        let seeds: &[&[u8]] = &[
            VAULT_AUTH_SEED,
            mint_key.as_ref(),
            &[ctx.accounts.vault_state.vault_bump],
        ];


        let signer = &[seeds];
        let cpi = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.vault_ata.to_account_info(),
                to:   ctx.accounts.owner_ata.to_account_info(),
                authority: ctx.accounts.vault_authority.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
            },
            signer,
        )
        .with_remaining_accounts(vec![

            ctx.accounts.extra_account_meta_list.to_account_info(),
            ctx.accounts.source_token_whitelist_state.to_account_info(),
            ctx.accounts.destination_token_whitelist_state.to_account_info(),
        ]);

        transfer_checked(cpi, amount, ctx.accounts.mint.decimals)
    }
}
