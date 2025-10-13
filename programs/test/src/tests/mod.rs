#[cfg(test)]
mod test {
    use {
        anchor_lang::{prelude::msg, AccountDeserialize, InstructionData, ToAccountMetas},
        anchor_spl::{
            associated_token::{self, spl_associated_token_account},
            token::Mint,
            token_interface::TokenAccount,
        },
        litesvm::LiteSVM,
        litesvm_token::{
            spl_token::ID as TOKEN_PROGRAM_ID, CreateAssociatedTokenAccount, CreateMint, MintTo,
        },
        solana_account::Account,
        solana_instruction::Instruction,

        solana_keypair::Keypair,
        solana_message::Message,
        solana_native_token::LAMPORTS_PER_SOL,
        solana_pubkey::Pubkey,

        solana_sdk_ids::system_program::ID as SYSTEM_PROGRAM_ID,
        solana_signer::Signer,
        solana_transaction::Transaction,
        spl_token_2022::instruction::{initialize_mint2, mint_to},
        std::path::PathBuf,
        std::str::FromStr,
        // vault_project::{init, vault},
    };
    use transfer_hook_vault_litesvm;

    static TRANSFER_HOOK_PROGRAM_ID: Pubkey = transfer_hook_vault_litesvm::ID;
    static VAULT_PROGRAM_ID: Pubkey = vault::ID;

    fn setup() -> (LiteSVM, Keypair, Keypair, Pubkey, Pubkey) {
        // Initialize LiteSVM and payer
        let mut program = LiteSVM::new();
        let payer = Keypair::new();
        let user = Keypair::new();
        let underlying_token_keypair = Keypair::new();

        // Airdrop some SOL to the payer keypair

        program
            .airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to airdrop SOL to payer");


        program
            .airdrop(&user.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to airdrop SOL to user");

        let (vault, _vault_bump) = Pubkey::find_program_address(&[b"vault"], &VAULT_PROGRAM_ID);
        
        // let (extra_account_meta_list, whitelist) = setup_transfer_hook(
        //     &mut program,
        //     &payer,
        //     user.pubkey(),
        //     &underlying_token_keypair,
        //     vault,
        // );

        // let (vault) = setup_vault(&mut program, &payer, &underlying_token_keypair.pubkey());

        // Return the LiteSVM instance and payer keypair
        (
            program,
            payer,
            user,
            underlying_token_keypair.pubkey(),
            vault,
            // extra_account_meta_list,
            // whitelist,
        )
    }

    #[test]
    fn test_deposit() {
        let (mut program, payer, user, underlying_token, vault ) =
            setup();
    }

}
