use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    sysvars::{clock::Clock, Sysvar},
    ProgramResult,
};

use pinocchio_token::instructions::Transfer;

use crate::state::{Contributor, Fundraiser};

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum RefundError {
    TimeNotElapsed = 0,
    TargetAlreadyReached = 1,
    NoContribution = 2,
}

impl From<RefundError> for ProgramError {
    fn from(e: RefundError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

pub fn process_refund_instruction(accounts: &[AccountInfo], _data: &[u8]) -> ProgramResult {
    let [user, creator, mint, fundraiser, vault, contributor_ata, contributor_pda, _system_program, _token_program, _associated_token_program, _rent_sysvar @ ..] =
        accounts
    else {
        return Err(pinocchio::program_error::ProgramError::NotEnoughAccountKeys);
    };

    // check that user is signer ✅
    if !&user.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let bump = {
        // Fundraiser
        let data = fundraiser.try_borrow_data().unwrap();
        let state = bytemuck::try_pod_read_unaligned::<Fundraiser>(&data).unwrap();

        // check that duration has elapsed
        let clock = Clock::get();
        let current_time = clock?.unix_timestamp as u64;
        if u64::from_le_bytes(state.duration) > current_time {
            return Err(RefundError::TimeNotElapsed.into());
        }

        let vault_state = pinocchio_token::state::TokenAccount::from_account_info(vault).unwrap();
        // Target Already met ?
        if u64::from_le_bytes(state.amount_to_raise) <= vault_state.amount() {
            return Err(RefundError::TargetAlreadyReached.into());
        }

        // // Validate fundraiser is owner of vault
        // if vault_state.owner() == fundraiser.key() {
        //     return Err(ProgramError::IllegalOwner);
        // }

        // check that vault is of the mint
        if vault_state.mint() != mint.key() {
            return Err(ProgramError::InvalidAccountData);
        }

        u8::from_le_bytes(state.bump)
    };

    let contributor_amount = {
        let data = contributor_pda.try_borrow_data().unwrap();
        let state = bytemuck::try_pod_read_unaligned::<Contributor>(&data).unwrap();

        // Ensure pda actually exists
        if contributor_pda.data_is_empty() {
            return Err(ProgramError::UninitializedAccount);
        }
        // Ensure contributor has deposited an amount
        if u64::from_le_bytes(state.amount) == 0 {
            return Err(RefundError::NoContribution.into());
        }

        state.amount
    };

    {
        let contributor_ata_state =
            pinocchio_token::state::TokenAccount::from_account_info(contributor_ata).unwrap();
        if contributor_ata_state.mint() != mint.key() {
            return Err(ProgramError::InvalidAccountData);
        }
    }

    let bump = &[bump];
    let seed = [
        Seed::from(b"fundraiser"),
        Seed::from(creator.key()),
        Seed::from(bump),
    ];
    let seeds = Signer::from(&seed);
    // transfer amount back to contributro [refund]
    Transfer {
        amount: u64::from_le_bytes(contributor_amount),
        authority: fundraiser,
        from: vault,
        to: contributor_ata,
    }
    .invoke_signed(&[seeds])?;

    Ok(())
}
