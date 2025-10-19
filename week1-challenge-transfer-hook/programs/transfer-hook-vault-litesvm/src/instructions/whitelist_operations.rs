use anchor_lang::{
    prelude::*,
};

use crate::states::WhitelistedAccount;


#[derive(Accounts)]
#[instruction(user: Pubkey)]
pub struct WhitelistOperations<'info> {
    #[account(
        mut,
        //address = 
    )]
    pub admin: Signer<'info>,
    /// CHECK:
    pub mint: UncheckedAccount<'info>,
    // do that
    #[account(
        init_if_needed,
        payer = admin,
        seeds = [b"whitelist", mint.key().as_ref(), user.key().as_ref()],
        space = 8 + WhitelistedAccount::INIT_SPACE,
        bump,
    )]
    pub whitelisted_account: Account<'info, WhitelistedAccount>,
    pub system_program: Program<'info, System>,
}


impl<'info> WhitelistOperations<'info> {
    pub fn add_restricted_account(&mut self, bumps: WhitelistOperationsBumps) -> Result<()> {
        if !self.whitelisted_account.is_restricted {
            self.whitelisted_account.is_restricted = true;
            self.whitelisted_account.bump = bumps.whitelisted_account;
        }
        Ok(())
    }

    pub fn remove_restricted_account(&mut self, _bumps: WhitelistOperationsBumps) -> Result<()> {
        if self.whitelisted_account.is_restricted {
            self.whitelisted_account.is_restricted = false;
        }
        Ok(())
    }

    pub fn add_whitelisted_account(&mut self, bumps: WhitelistOperationsBumps) -> Result<()> {
        if !self.whitelisted_account.is_whitelisted{
            self.whitelisted_account.is_whitelisted = true;
            self.whitelisted_account.bump = bumps.whitelisted_account;
        }
        Ok(())
    }

    pub fn remove_whitelisted_account (&mut self, _bumps: WhitelistOperationsBumps) -> Result<()> {
        if self.whitelisted_account.is_whitelisted {
            self.whitelisted_account.is_restricted = false;
        }
        Ok(())
    }
}
