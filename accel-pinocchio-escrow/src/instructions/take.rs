use pinocchio::{
    account_info::AccountInfo, instruction::{Seed, Signer}, msg, pubkey::log, sysvars::{rent::Rent, Sysvar}, ProgramResult
};
use pinocchio_pubkey::derive_address;
use pinocchio_system::instructions::CreateAccount;
use pinocchio::log::sol_log_64;

use crate::state::Escrow;


/**
 * @dev mint_a:  transfer {from: escrow_ata, to: taker_ata_mint_a}
 * @dev mint_b:  transfer {from: taker_ata_mint_b, to: maker_ata_mint_b}
 */
pub fn process_take_instruction(
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {

    msg!("Processing Take instruction");

    let [
        taker,
        mint_a,
        mint_b,
        escrow_account,
        maker_ata,              // mint_b
        taker_ata_mint_b,       // mint_b
        taker_ata_mint_a,       // mint_a
        escrow_ata,             // mint_a
        _system_program,
        _token_program,
        _associated_token_program,
        _rent_sysvar @ ..
    ] = accounts else {
        return Err(pinocchio::program_error::ProgramError::NotEnoughAccountKeys);
    };
    
    // No need for bump, therefore arguments start at 0
    let amount_to_receive = unsafe{ *(data.as_ptr().add(0) as *const u64) };
    let amount_to_give = unsafe{ *(data.as_ptr().add(8) as *const u64) };

    {

        pinocchio_token::instructions::Transfer {
            from: &taker_ata_mint_b,
            to: &maker_ata,
            authority: &taker,
            amount: amount_to_receive,       // This is amount to be received by maker
        }.invoke()?;
    }

    let maker_ata_state = 
        pinocchio_token::state::TokenAccount::from_account_info(&maker_ata)?;

    let maker_key = maker_ata_state.owner();
    let escrow_state = Escrow::from_account_info(&escrow_account)?;
    let bump = escrow_state.bump;


    {
    
    
        let taker_ata_state_mint_a = pinocchio_token::state::TokenAccount::from_account_info(&taker_ata_mint_a)?;
        if taker_ata_state_mint_a.owner() != taker.key() {
            return Err(pinocchio::program_error::ProgramError::IllegalOwner);
        }
        let taker_ata_state_mint_b = pinocchio_token::state::TokenAccount::from_account_info(&taker_ata_mint_b)?;
        if taker_ata_state_mint_b.owner() != taker.key() {
            return Err(pinocchio::program_error::ProgramError::IllegalOwner);
        }
        if taker_ata_state_mint_a.mint() != mint_a.key() {
            return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
        }
        if taker_ata_state_mint_b.mint() != mint_b.key() {
            return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
        }
        //
        if maker_ata_state.mint() != mint_b.key() {
            return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
        }


        let seed = [b"escrow".as_ref(), maker_key.as_slice(), &[bump]];

        let escrow_account_pda = derive_address(&seed, None, &crate::ID);
        log(&escrow_account_pda);
        log(escrow_account.key());

        // If this holds it also implies maker key is the right one
        if escrow_account_pda != *escrow_account.key() {
            return Err (pinocchio::program_error::ProgramError::InvalidAccountData); 
        }
        //
    }
    let bump = [bump];
    let seed = [Seed::from(b"escrow"), Seed::from(maker_key), Seed::from(&bump)];
    let signer_seeds = Signer::from(&seed);

            // {
            //     let escrow_state = Escrow::from_account_info(escrow_account)?;
            //
            //     escrow_state.set_maker(maker.key());
            //     escrow_state.set_mint_a(mint_a.key());
            //     escrow_state.set_mint_b(mint_b.key());
            //     escrow_state.set_amount_to_receive(amount_to_receive);
            //     escrow_state.set_amount_to_give(amount_to_give);  
            //     escrow_state.bump = data[0];
            // }

    pinocchio_token::instructions::Transfer {
        from: &escrow_ata,
        to: &taker_ata_mint_a,
        authority: &escrow_account,
        amount: amount_to_give,       // This is amount given by maker
    }.invoke_signed(&[signer_seeds])?;



    Ok(())
}
