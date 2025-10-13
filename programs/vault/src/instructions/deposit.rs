use std::vec;

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        Mint, TokenAccount, TokenInterface,
    },
};

use crate::states::Vault;

#[derive(Accounts)]
pub struct Deposit<'info> {
    /// user signs the tx and pays for ATA if needed
    #[account(mut)]
    pub user: Signer<'info>,

    /// interface mint types (works with Token v1 and Token-2022)
    pub underlying_token: InterfaceAccount<'info, Mint>,

    /// vault PDA (used as mint authority). must be mut because it is used as CPI signer
    #[account(mut, seeds = [b"vault"], bump)]
    pub vault: Account<'info, Vault>,

    /// user's ATA for underlying token (source of transfer)
    #[account(
        mut,
        associated_token::mint = underlying_token,

        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub user_underlying_ata: InterfaceAccount<'info, TokenAccount>,

    /// vault's ATA for underlying token (destination of transfer)
    #[account(
        mut,

        associated_token::mint = underlying_token,
        associated_token::authority = vault,
        associated_token::token_program = token_program
    )]
    pub vault_underlying_ata: InterfaceAccount<'info, TokenAccount>,

    ///CHECK: ExtraAccountMetaList Account, will be checked by the transfer hook
    #[account(mut)]
    pub extra_account_meta_list: UncheckedAccount<'info>,

    ///CHECK: Whitelist Account, will be checked by the transfer hook
    pub whitelist: UncheckedAccount<'info>,



    pub associated_token_program: Program<'info, AssociatedToken>,

    /// CHECK: Transfer Hook Program Account, checked in CPI
    pub transfer_hook_program: UncheckedAccount<'info>,

    /// use Interface for Token program so token_interface CPIs are consistent
    pub token_program: Interface<'info, TokenInterface>,

    pub system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    /// Transfer `amount` of underlying token from user -> vault ATA
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        // Build the transfer_checked instruction manually to have full control over accounts
        let mut transfer_idx = anchor_spl::token_2022::spl_token_2022::instruction::transfer_checked(
            &anchor_spl::token_2022::spl_token_2022::id(),
            &self.user_underlying_ata.key(),
            &self.underlying_token.key(),
            &self.vault_underlying_ata.key(),
            &self.user.key(),
            &[&self.user.key()],
            amount,
            self.underlying_token.decimals,
        ).unwrap();

        // Add the transfer hook accounts in the correct order
        transfer_idx.accounts.push(anchor_lang::prelude::AccountMeta::new_readonly(
            self.transfer_hook_program.key(),
            false,

        ));


        transfer_idx.accounts.push(anchor_lang::prelude::AccountMeta::new_readonly(
            self.extra_account_meta_list.key(),
            false,
        ));

        transfer_idx.accounts.push(anchor_lang::prelude::AccountMeta::new_readonly(

            self.whitelist.key(),
            false,
        ));

        // Invoke the instruction with all required accounts
        anchor_lang::solana_program::program::invoke(
            &transfer_idx,
            &[

                self.token_program.to_account_info(),
                self.user_underlying_ata.to_account_info(),
                self.underlying_token.to_account_info(),
                self.vault_underlying_ata.to_account_info(),
                self.user.to_account_info(),
                self.transfer_hook_program.to_account_info(),
                self.extra_account_meta_list.to_account_info(),

                self.whitelist.to_account_info(),

            ],
        )?;

        Ok(())
    }

}
