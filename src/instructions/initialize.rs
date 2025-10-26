use bytemuck::{Pod, Zeroable};


use pinocchio::{
    program_error::ProgramError,
    account_info::AccountInfo,
    instruction::{Seed, Signer},

    log, msg,

    pubkey::{self, log},
    sysvars::{self, rent::Rent, Sysvar},
    ProgramResult,
};
// use pinocchio_log::log;
use pinocchio_pubkey::derive_address;
use pinocchio_system::instructions::CreateAccount;

use alloc::vec::Vec;

use crate::{fundraiser, state::Fundraiser};

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, Debug, PartialEq)]
pub struct InitializeInstruction {
    pub amount_to_raise: u64,
    pub duration: u64,
}


impl InitializeInstruction {
    pub fn to_bytes(&self) -> Vec<u8> {
        bytemuck::bytes_of(self).to_vec()
    }
}

pub fn process_initialize_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [maker, mint, fundraiser, vault, system_program, token_program, associated_token_program, rent_sysvar @ ..] =

        accounts
    else {
        return Err(pinocchio::program_error::ProgramError::NotEnoughAccountKeys);
    };

    // checks
    // check that maker is a signer ✅
    if !&maker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let ix_data = bytemuck::try_pod_read_unaligned::<InitializeInstruction>(data)
        .map_err(|_| pinocchio::program_error::ProgramError::InvalidInstructionData)?;


    let fundraiser_seeds = [b"fundraiser".as_ref(), maker.key().as_ref()];
    let (fundraiser_pda, bump) = pubkey::find_program_address(&fundraiser_seeds, &crate::ID);

    // Validate that derived == account 
    if fundraiser_pda != *fundraiser.key() {
        return Err(ProgramError::InvalidAccountData);
    }

    let vault_state = pinocchio_token::state::TokenAccount::from_account_info(&vault)?;
    // Account fundraiser should be the authority over vault account 
    if vault_state.owner() != fundraiser.key() {
        return Err(ProgramError::IllegalOwner);
    }

    // check that mint is created ✅
    let mint_state = pinocchio_token::state::Mint::from_account_info(&mint)?;
    if !mint_state.is_initialized() {
        return Err(ProgramError::UninitializedAccount);  // YO! mint does not exist
    }
    // check that vault mint is mint ✅
    if vault_state.mint() != mint.key() {
        return Err(ProgramError::InvalidAccountData);    // "Yo!, You provided wrong mint address"
    }
        


    // create fundraiser account
    let bump = [bump.to_le()];
    let seed = [
        Seed::from(b"fundraiser"),
        Seed::from(maker.key()),
        Seed::from(&bump),
    ];
    let signer_seeds = Signer::from(&seed);
    CreateAccount {
        from: maker,
        lamports: Rent::get()?.minimum_balance(Fundraiser::LEN),
        owner: &crate::ID,
        space: Fundraiser::LEN as u64,
        to: fundraiser,
    }
    .invoke_signed(&[signer_seeds])?;

    let data = &mut fundraiser.try_borrow_mut_data()?;
    let fundraiser_state = &mut bytemuck::from_bytes_mut::<Fundraiser>(data);

    fundraiser_state.amount_to_raise = ix_data.amount_to_raise.to_le_bytes();
    fundraiser_state.bump = bump;
    fundraiser_state.current_amount = 0u64.to_le_bytes();
    fundraiser_state.duration = ix_data.duration.to_le_bytes();
    fundraiser_state.maker = *maker.key();
    fundraiser_state.mint = *mint.key();
    fundraiser_state.time_started =
        (sysvars::clock::Clock::get()?.unix_timestamp as u64).to_le_bytes();

    Ok(())
}
