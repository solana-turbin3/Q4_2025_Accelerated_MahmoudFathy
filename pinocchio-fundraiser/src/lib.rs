#![cfg_attr(not(test), no_std)]
use pinocchio::{
    nostd_panic_handler,
    account_info::AccountInfo,
    entrypoint,
    pubkey::Pubkey,
    ProgramResult
};

use crate::instructions::*;

#[cfg(test)]
extern crate std;

extern crate alloc;
pub use alloc::vec::Vec;

// Use the no_std panic handler.
#[cfg(target_os = "solana")]
nostd_panic_handler!();

#[cfg(test)]
mod tests;

mod state;
mod instructions;

pub use instructions::*;
pub use state::*;


#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);

pinocchio_pubkey::declare_id!("EiJfMHkdFRYVts5Kvxg6ooBaZ1TV6qEiY41xjZuSFLSw");

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {

    assert_eq!(program_id, &ID);

    let (discriminator, data) = instruction_data.split_first()
        .ok_or(pinocchio::program_error::ProgramError::InvalidInstructionData)?;

    match FundraiserInstructions::try_from(discriminator)? {
        FundraiserInstructions::Initialize => process_initialize_instruction(accounts, data)?,
        FundraiserInstructions::Contribute=> process_contribute_instruction(accounts, data)?,
        _ => return Err(pinocchio::program_error::ProgramError::InvalidInstructionData),
    }
    Ok(())
}
