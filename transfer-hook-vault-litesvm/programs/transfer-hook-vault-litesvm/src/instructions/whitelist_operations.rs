use anchor_lang::{
    prelude::*,
};

use crate::states::RestrictedAccount;


#[derive(Accounts)]
#[instruction(vault: Pubkey)]
pub struct WhitelistOperations<'info> {
    #[account(
        mut,
        //address = 
    )]
    pub admin: Signer<'info>,
    // At this point anyone can set a vault to be restricted account TODO: Vault Admin only should
    // do that
    #[account(
        init_if_needed,
        payer = admin,
        seeds = [b"restricted_account", vault.key().as_ref()],
        space = 8 + RestrictedAccount::INIT_SPACE,
        bump,
    )]
    pub restricted_account: Account<'info, RestrictedAccount>,
    pub system_program: Program<'info, System>,
}


impl<'info> WhitelistOperations<'info> {
    pub fn add_restricted_account(&mut self, bumps: WhitelistOperationsBumps) -> Result<()> {
        // Challenge:
        if !self.restricted_account.is_restricted {
            self.restricted_account.set_inner(RestrictedAccount {
                is_restricted: true,
                bump: bumps.restricted_account
            });
        }
        Ok(())
    }

    pub fn remove_restricted_account(&mut self, bumps: WhitelistOperationsBumps) -> Result<()> {
        if self.restricted_account.is_restricted{
            self.restricted_account.set_inner(RestrictedAccount{
                is_restricted: false,
                bump: bumps.restricted_account
            });
        }
        Ok(())
    }
}
