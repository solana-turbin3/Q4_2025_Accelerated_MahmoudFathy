use anchor_lang::prelude::*;

mod transfer_permit_list;
pub use transfer_permit_list::*;

#[account]
#[derive(InitSpace)]
pub struct RestrictedAccount {
    pub is_restricted: bool,
    pub bump: u8,
}
