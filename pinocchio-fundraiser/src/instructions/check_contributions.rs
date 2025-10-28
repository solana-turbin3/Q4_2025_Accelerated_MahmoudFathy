use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey, ProgramResult,
};
use pinocchio_token::{instructions::Transfer, state::TokenAccount};

use crate::state::Fundraiser;

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum CheckContributionsError {
    TargetNotReached = 0,
}

impl From<CheckContributionsError> for ProgramError {
    fn from(e: CheckContributionsError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

// @dev maker can check contributions so far
pub fn process_check_contributions_instruction(
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let [maker, mint, fundraiser, vault, maker_ata, _others @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !maker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Validate the Fundraiser account
    if fundraiser.owner() != &crate::ID {
        return Err(ProgramError::InvalidAccountOwner);
    }
    let fundraiser_data = fundraiser.try_borrow_data().unwrap();
    let fundraiser_state =
        bytemuck::try_pod_read_unaligned::<Fundraiser>(&fundraiser_data).unwrap();
    // Validating the mint
    if fundraiser_state.mint != *mint.key() {
        return Err(ProgramError::InvalidAccountData);
    }

    // validating the vault owner
    let vault_state = TokenAccount::from_account_info(vault)?;
    if vault_state.mint() != mint.key() {
        return Err(ProgramError::InvalidAccountData);
    }
    if vault_state.owner() != fundraiser.key() {
        return Err(ProgramError::InvalidAccountOwner);
    }

    // Check if the target amount has been met
    let amount_to_raise = u64::from_le_bytes(fundraiser_state.amount_to_raise);
    if vault_state.amount() >= amount_to_raise {
        return Err(CheckContributionsError::TargetNotReached.into());
    }

    // Validating the maker's token account
    let maker_ata_state = TokenAccount::from_account_info(maker_ata)?;
    if maker_ata_state.mint() != mint.key() {
        return Err(ProgramError::InvalidAccountData);
    }
    if maker_ata_state.owner() != maker.key() {
        return Err(ProgramError::InvalidAccountOwner);
    }

    // Validating the fundraiser account
    let (fundraiser_pda, bump) =
        pubkey::find_program_address(&[b"fundraiser", &maker.key().as_ref()], &crate::ID);
    if fundraiser.key() != &fundraiser_pda {
        return Err(ProgramError::InvalidAccountData);
    }

    // Define the fundraiser PDA
    let bump = &[u8::from_le_bytes(fundraiser_state.bump)];
    let signer_seeds = [
        Seed::from(b"fundraiser"),
        Seed::from(maker.key().as_ref()),
        Seed::from(bump),
    ];
    let signer = Signer::from(&signer_seeds);

    // Transfe the contributions to the maker
    Transfer {
        from: vault,
        authority: fundraiser,
        to: maker_ata,
        amount: vault_state.amount(),
    }
    .invoke_signed(&[signer])?;

    Ok(())
}
