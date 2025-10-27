use bytemuck::{Pod, Zeroable};


use pinocchio::{
    account_info::AccountInfo, instruction::{Account, Seed, Signer}, log::{self, sol_log_64}, msg, program_error::ProgramError, pubkey::{self, find_program_address, log}, sysvars::{self, rent::Rent, Sysvar}, ProgramResult
};
// use pinocchio_log::log;
use pinocchio_pubkey::derive_address;
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::instructions::Transfer;

use crate::{fundraiser, state::{Contributor, Fundraiser}};


pub fn process_contribute_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [contributor, mint, fundraiser, vault, contributor_ata, contributor_pda, system_program, token_program, associated_token_program, rent_sysvar @ ..] =
        accounts
    else {
        return Err(pinocchio::program_error::ProgramError::NotEnoughAccountKeys);
    };
    let amount = data;

    // check that contributor is signer ✅
    // check that maker is a signer ✅
    if !&contributor.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Validate that this program owns fundraiser 
    // @dev this can also be chacked using the seeds that derived the fundraiser
    if !&fundraiser.is_owned_by(&crate::ID) {
        return Err(ProgramError::IllegalOwner);
    }

    {
        // check that fundraiser exists 
        let data = &mut fundraiser.try_borrow_mut_data()?;
        let fundraiser_state = &mut bytemuck::from_bytes_mut::<Fundraiser>(data);

        // check that the mint is correct in fundraiser field ✅
        if mint.key() != &fundraiser_state.mint {
            return Err(ProgramError::InvalidAccountData);
        }

        // check that provided vault is owned by fundraiser state
        let vault_state = pinocchio_token::state::TokenAccount::from_account_info(&vault)?;
        if vault_state.owner() != fundraiser.key() {
            return Err(ProgramError::IllegalOwner);
        }

        // check that contributor has enough amount to transfer
        let contributor_ata_state =
            pinocchio_token::state::TokenAccount::from_account_info(&*contributor_ata)?;

        if contributor_ata_state.amount() < u64::from_le_bytes(amount.try_into().unwrap() ) {
            return Err(ProgramError::InvalidArgument);
        }

        // check that contributor is sending above minimum
        if u64::from_le_bytes(amount.try_into().unwrap()) < fundraiser_state.min_sendable() {
            return Err(ProgramError::InvalidArgument);  // amount less than minimum
        }

        // check that contributor is sending below maximum
        if u64::from_le_bytes(amount.try_into().unwrap()) > fundraiser_state.max_sendable() {
            return Err(ProgramError::InvalidArgument);
        }
    }


    // @dev create contributor pda if it's not created already
    // @dev [init-if-needed]
    let contributor_seeds: &[&[u8]] = &[b"contributor".as_ref(), contributor.key().as_ref()];

    if contributor_pda.lamports() == 0 || contributor_pda.data_is_empty() {
        let (contributor_pda_derived, bump) = find_program_address(&contributor_seeds, &crate::ID);

        if contributor_pda.key() != &contributor_pda_derived{
            return Err(ProgramError::InvalidAccountData);
        }

        // create the account
        let bump = [bump.to_le()];
        let seed = [
            Seed::from(b"contributor"),
            Seed::from(contributor.key()),
            Seed::from(&bump),
        ];
        let signer_seeds = Signer::from(&seed);
        {
            CreateAccount {
                from: contributor,
                lamports: Rent::get()?.minimum_balance(Contributor::LEN),
                owner: &crate::ID,
                space: Contributor::LEN as u64,
                to: contributor_pda,
            }
            .invoke_signed(&[signer_seeds])?;
        }
        {
            // deposit to the vault
            Transfer {
                amount: u64::from_le_bytes(amount.try_into().unwrap()),
                authority: contributor,
                from: contributor_ata,
                to: vault,
            }
            .invoke()?;
        }

        // increase contributor amount by how much was deposited

        let raw_account_data = &mut contributor_pda.try_borrow_mut_data()?;
        let contributor_pda_state = bytemuck::from_bytes_mut::<Contributor>(raw_account_data);

        contributor_pda_state.amount =
            (u64::from_le_bytes(amount.try_into().unwrap()))
            .to_le_bytes()
    } else {
        let raw_account_data = &mut contributor_pda.try_borrow_mut_data()?;
        let contributor_pda_state = bytemuck::from_bytes_mut::<Contributor>(raw_account_data);

        contributor_pda_state.amount =
            (u64::from_le_bytes(contributor_pda_state.amount)
                + u64::from_le_bytes(amount.try_into().unwrap()))
            .to_le_bytes();

        // deposit to the vault
        // increase contributor amount by quantity being deposited
        {
            // deposit to the vault
            Transfer {
                amount: u64::from_le_bytes(amount.try_into().unwrap()),
                authority: contributor,
                from: contributor_ata,
                to: vault,
            }
            .invoke()?;
        }
    }

    Ok(())

}
