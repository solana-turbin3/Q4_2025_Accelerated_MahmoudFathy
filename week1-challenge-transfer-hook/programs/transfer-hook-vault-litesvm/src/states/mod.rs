use anchor_lang::prelude::*;

mod transfer_permit_list;
pub use transfer_permit_list::*;

// @note: The restriction on the token account in transfers
#[account]
#[derive(InitSpace)]
pub struct WhitelistedAccount {
    pub is_restricted: bool,
    pub is_whitelisted: bool,
    pub bump: u8,
}

