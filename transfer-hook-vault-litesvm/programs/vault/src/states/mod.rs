use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace, Debug)]
pub struct Vault {
    pub admin: Pubkey,
    pub underlying_token: Pubkey,
    pub bump: u8,
}
