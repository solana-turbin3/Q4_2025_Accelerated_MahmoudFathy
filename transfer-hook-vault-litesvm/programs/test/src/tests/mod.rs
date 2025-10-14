#[cfg(test)]
mod test {
    use {
        anchor_lang::{prelude::msg, AccountDeserialize, InstructionData, ToAccountMetas}, anchor_spl::{
            associated_token::{self, spl_associated_token_account},
            token::Mint,
            token_interface::TokenAccount,
        }, litesvm::{types::{FailedTransactionMetadata, TransactionMetadata}, LiteSVM}, litesvm_token::{
            spl_token::ID as TOKEN_PROGRAM_ID, CreateAssociatedTokenAccount, CreateMint, MintTo,
        }, solana_account::Account, solana_instruction::Instruction, solana_keypair::Keypair, solana_message::Message, solana_native_token::LAMPORTS_PER_SOL, solana_pubkey::Pubkey, solana_sdk::compute_budget, solana_sdk_ids::system_program::ID as SYSTEM_PROGRAM_ID, solana_signer::Signer, solana_transaction::Transaction, spl_token_2022::instruction::{initialize_mint2, mint_to}, std::{path::PathBuf, str::FromStr}, transfer_hook_vault_litesvm as transfer_hook
    };
    
    static ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";
    static TRANSFER_HOOK_PROGRAM_ID: Pubkey = transfer_hook_vault_litesvm::ID;
    static VAULT_PROGRAM_ID: Pubkey = vault::ID;

    fn send_signed_transaction(
        program: &mut LiteSVM,
        payer: &Keypair,
        ix: Instruction,
        signers: Option<&Keypair>,
    ) -> Result<TransactionMetadata, FailedTransactionMetadata> {
        let message = Message::new(&[ix], Some(&payer.pubkey()));
        let recent_blockhash = program.latest_blockhash();
        let tx = match signers {
            Some(ks) => {
                Transaction::new(&[&payer, &ks], message, recent_blockhash)
            }
            None => {
                Transaction::new(&[payer], message, recent_blockhash)
            }
        };
        program.send_transaction(tx)
    }

    fn init_vault(program: &mut LiteSVM, payer: &Keypair, collateral: &Pubkey) -> (Pubkey) {
        let bin_filepath =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/deploy/vault.so");
        let vault_so = std::fs::read(bin_filepath).expect("Failed to read .so file");
        program.add_program(VAULT_PROGRAM_ID, &vault_so);

        let vault: Pubkey = Pubkey::find_program_address(&[b"vault"], &VAULT_PROGRAM_ID).0;

        let vault_underlying_ata: Pubkey =
            associated_token::get_associated_token_address_with_program_id(
                &vault,
                &*collateral,
                &spl_token_2022::id(),
            );


        let init_ix = Instruction {
            program_id: VAULT_PROGRAM_ID,
            accounts: vault::accounts::Init {
                admin: payer.pubkey(),
                underlying_token: collateral.to_owned(),
                vault,
                vault_underlying_ata,
                associated_token_program: Pubkey::from_str(ASSOCIATED_TOKEN_PROGRAM_ID).unwrap(),

                token_program: spl_token_2022::ID,
                system_program: SYSTEM_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: vault::instruction::InitVault {}.data(),
        };

        send_signed_transaction(program, &payer, init_ix, None);
        vault
    }

    fn init_token_ext_program(
        program: &mut LiteSVM,
        payer: &Keypair,
        user: Pubkey,
        token_collateral: &Keypair,
        vault: Pubkey,
    ) -> (Pubkey, Pubkey) {
        let bin_filepath =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/deploy/transfer_hook_vault_litesvm.so");
        let transfer_hook_program = std::fs::read(bin_filepath).expect("Failed to read .so file");
        program.add_program(TRANSFER_HOOK_PROGRAM_ID, &transfer_hook_program);

        // Init the extra account meta list

        let extra_account_meta_list = Pubkey::find_program_address(
            &[b"extra-account-metas", token_collateral.pubkey().as_ref()],
            &TRANSFER_HOOK_PROGRAM_ID,
        ).0;

        let ix = Instruction {
            program_id: TRANSFER_HOOK_PROGRAM_ID,
            accounts: transfer_hook::accounts::TokenFactory {
                user: payer.pubkey(),
                mint: token_collateral.pubkey(),
                extra_account_meta_list,
                system_program: SYSTEM_PROGRAM_ID,
                token_program: spl_token_2022::ID,

            }
            .to_account_metas(None),
            data: transfer_hook::instruction::InitMint {}.data(),
        };
        send_signed_transaction(program, payer, ix, Some(&token_collateral));

        let ix = Instruction {
            program_id: TRANSFER_HOOK_PROGRAM_ID,
            accounts: transfer_hook::accounts::InitializeExtraAccountMetaList {
                payer: payer.pubkey(),
                extra_account_meta_list,
                mint: token_collateral.pubkey(),
                system_program: SYSTEM_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: transfer_hook::instruction::InitializeTransferHook {}.data(),
        };
        send_signed_transaction(program, payer, ix, None);

        let whitelist =
            Pubkey::find_program_address(&[b"restricted_account", user.as_ref()], &TRANSFER_HOOK_PROGRAM_ID).0;


        // let ix = Instruction {
        //     program_id: TRANSFER_HOOK_PROGRAM_ID,
        //     accounts: transfer_hook::accounts::InitializeWhitelist {
        //         admin: payer.pubkey(),
        //
        //         whitelist,
        //         system_program: SYSTEM_PROGRAM_ID,
        //     }
        //     .to_account_metas(None),
        //     data: transfer_hook::instruction::InitializeWhitelist { token_owner: user }.data(),
        // };
        //
        // send_ix_in_tx(program, payer, ix, None);
        //
        // let ix = Instruction {
        //     program_id: TRANSFER_HOOK_PROGRAM_ID,
        //     accounts: transfer_hook::accounts::WhitelistOperations {
        //         admin: payer.pubkey(),
        //         restricted_account: whitelist,
        //         system_program: SYSTEM_PROGRAM_ID,
        //     }
        //     .to_account_metas(None),
        //     data: transfer_hook::instruction::AddRestrictedAccount { vault: user }.data(),
        // };
        // send_signed_transaction(program, payer, ix, None);

        // Return the LiteSVM instance and payer keypair
        (extra_account_meta_list, whitelist)
    }

    fn do_deposit(
        payer: &Keypair,
        user: &Keypair,
        program: &mut LiteSVM,
        collateral: Pubkey,
        amount: u64,
        extra_account_meta_list: Pubkey,
        whitelist: Pubkey,
    ) -> (Pubkey, Pubkey) {
        // User deposits collateral token into vault
        let vault = Pubkey::find_program_address(&[b"vault"], &VAULT_PROGRAM_ID).0;
        let vault_underlying_ata: Pubkey =
            associated_token::get_associated_token_address_with_program_id(
                &vault,
                &collateral,
                &spl_token_2022::id(),
            );
        let user_underlying_ata: Pubkey =
            CreateAssociatedTokenAccount::new(program, &user, &collateral)
                .token_program_id(&spl_token_2022::ID)
                .owner(&user.pubkey())
                .send()
                .unwrap();

        let ix = mint_to(
            &spl_token_2022::id(),
            &collateral,    // mint
            &user_underlying_ata, // destination ATA
            &payer.pubkey(),      // authority
            &[],                  // no multisig signers
            amount,               // amount
        )
        .unwrap();

        send_signed_transaction(program, &payer, ix, None);

        let init_ix = Instruction {
            program_id: VAULT_PROGRAM_ID,
            accounts: vault::accounts::Deposit {
                user: user.pubkey(),
                underlying_token: collateral,
                vault,
                vault_underlying_ata,
                extra_account_meta_list,
                whitelist,
                user_underlying_ata,
                transfer_hook_program: TRANSFER_HOOK_PROGRAM_ID,
                associated_token_program: spl_associated_token_account::ID,
                token_program: spl_token_2022::ID,

                system_program: SYSTEM_PROGRAM_ID,

            }
            .to_account_metas(None),
            data: vault::instruction::Deposit { amount }.data(),
        };

        send_signed_transaction(program, &user, init_ix, None);

        (user_underlying_ata, vault_underlying_ata)
    }

    fn setup() -> (LiteSVM, Keypair, Keypair, Pubkey, Pubkey, Pubkey, Pubkey) {
        // Initialize LiteSVM and payer
        let mut program = LiteSVM::new();
        // Set a custom compute unit limit for the next transaction
        // FIXME: Attempt to fix compute limit issue on test run 
        // program.with_compute_budget(|c: compute_budget::ComputeBudgetInstruction| {
        //     c.set_compute_unit_limit(1_000_000); // 1 million CUs
        // });
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
        
        let (extra_account_meta_list, whitelist) = init_token_ext_program(
            &mut program,
            &payer,
            user.pubkey(),
            &underlying_token_keypair,
            vault,
        );
        
        // FIXME: Test failes here due to compute limit
        let vault = init_vault(&mut program, &payer, &underlying_token_keypair.pubkey()); 

        // Return the LiteSVM instance and payer keypair
        (
            program,
            payer,
            user,
            underlying_token_keypair.pubkey(),
            vault,
            extra_account_meta_list,
            whitelist,
        )
    }

    #[test]
    fn test_deposit() {
        let (mut program, payer, user, underlying_token, vault, extra_account_meta_list, whitelist ) =
            setup();
        let amount = 1_000_000u64;
        let (user_underlying_ata, vault_underlying_ata) = do_deposit(
            &payer,
            &user,
            &mut program,
            underlying_token,
            amount,
            extra_account_meta_list,
            whitelist,
        );
    }

}
