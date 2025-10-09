use anchor_lang::{
    prelude::*,
};

use crate::states::Whitelist;


#[derive(Accounts)]
#[instruction(user: Pubkey)]
pub struct WhitelistOperations<'info> {
    #[account(
        mut,
        //address = 
    )]
    pub admin: Signer<'info>,
    #[account(
        init_if_needed,
        payer = admin,
        seeds = [b"whitelist", user.key().as_ref()],
        space = 8 + Whitelist::INIT_SPACE,
        bump,
    )]
    pub whitelist: Account<'info, Whitelist>,
    pub system_program: Program<'info, System>,
}


impl<'info> WhitelistOperations<'info> {
    pub fn add_to_whitelist(&mut self, bumps: WhitelistOperationsBumps) -> Result<()> {
        // Challenge:
        if !self.whitelist.is_whitelisted {
            self.whitelist.set_inner(Whitelist {
                is_whitelisted: true,
                bump: bumps.whitelist
            });
        }
        Ok(())
    }

    pub fn remove_from_whitelist(&mut self, bumps: WhitelistOperationsBumps) -> Result<()> {
        if self.whitelist.is_whitelisted {
            self.whitelist.set_inner(Whitelist {
                is_whitelisted: false,
                bump: bumps.whitelist
            });
        }
        Ok(())
    }

    // pub fn realloc_whitelist(&self, is_adding: bool) -> Result<()> {
    //     // Get the account info for the whitelist
    //     let account_info = self.whitelist.to_account_info();
    //
    //     if is_adding {  // Adding to whitelist
    //         let new_account_size = account_info.data_len() + std::mem::size_of::<Pubkey>();
    //         // Calculate rent required for the new account size
    //         let lamports_required = (Rent::get()?).minimum_balance(new_account_size);
    //         // Determine additional rent required
    //
    //         let rent_diff = lamports_required - account_info.lamports();
    //
    //
    //         // Perform transfer of additional rent
    //         let cpi_program = self.system_program.to_account_info();
    //         let cpi_accounts = system_program::Transfer{
    //             from: self.admin.to_account_info(), 
    //             to: account_info.clone(),
    //         };
    //         let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
    //         system_program::transfer(cpi_context,rent_diff)?;
    //
    //         // Reallocate the account
    //         account_info.resize(new_account_size)?;
    //         msg!("Account Size Updated: {}", account_info.data_len());
    //
    //     } else {        // Removing from whitelist
    //         let new_account_size = account_info.data_len() - std::mem::size_of::<Pubkey>();
    //         // Calculate rent required for the new account size
    //
    //         let lamports_required = (Rent::get()?).minimum_balance(new_account_size);
    //         // Determine additional rent to be refunded
    //         let rent_diff = account_info.lamports() - lamports_required;
    //
    //         // Reallocate the account
    //         account_info.resize(new_account_size)?;
    //         msg!("Account Size Downgraded: {}", account_info.data_len());
    //
    //         // Perform transfer to refund additional rent
    //         **self.admin.to_account_info().try_borrow_mut_lamports()? += rent_diff;
    //
    //         **self.whitelist.to_account_info().try_borrow_mut_lamports()? -= rent_diff;
    //     }
    //
    //     Ok(())
    // }
}
