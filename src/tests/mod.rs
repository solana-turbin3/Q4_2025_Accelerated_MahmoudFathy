#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use litesvm::{types::{FailedTransactionMetadata, TransactionMetadata, TransactionResult}, LiteSVM};
    use litesvm_token::{
        spl_token::{
            self,
            solana_program::
            {msg, rent::Rent, sysvar::SysvarId}
        }, 
        CreateAssociatedTokenAccount,
        CreateMint,
        MintTo
    };
    
    use solana_instruction::{AccountMeta, Instruction};
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_native_token::LAMPORTS_PER_SOL;
    use solana_pubkey::Pubkey;
    use solana_sdk_ids::system_program::ID as system_program_id;
    use solana_signer::Signer;
    use solana_transaction::Transaction;
    use pinocchio_token::state::TokenAccount;

    use crate::{fundraiser, InitializeInstruction};

    const PROGRAM_ID: &str = "EiJfMHkdFRYVts5Kvxg6ooBaZ1TV6qEiY41xjZuSFLSw";
    const TOKEN_PROGRAM_ID: Pubkey = spl_token::ID;
    const ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey =  Pubkey::from_str_const("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

    trait Pipe {
        fn pipe<F, R>(self, f: F) -> R
        where
            F: FnOnce(Self) -> R,
            Self: Sized,
        {
            f(self)
        }
    }

    impl<T> Pipe for T {}


    fn get_program_id() -> Pubkey {
        Pubkey::from(crate::ID)
    }

    // Helper: Send the transaction
    fn send_singed_tx(
        svm: &mut LiteSVM,
        ix: Instruction,
        payer: Keypair
    ) -> TransactionResult {
        let message = Message::new(&[ix], Some(&payer.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[&payer], message, recent_blockhash);

        svm.send_transaction(transaction)
    }

    fn setup() -> (LiteSVM, Keypair, Keypair, Keypair, Pubkey, Pubkey, Pubkey, Pubkey) {

        let mut svm = LiteSVM::new();
        let mint_authority = Keypair::new();
        let maker = Keypair::new();   // also payer 
        let owner = Keypair::new();

        svm
            .airdrop(&maker.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");
        svm
            .airdrop(&owner.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");
        svm
            .airdrop(&mint_authority.pubkey(), 100 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");


        // Load program SO file
        msg!("[SETUP] The path is!! {}", env!("CARGO_MANIFEST_DIR"));
        let so_path = PathBuf::from("./target/sbf-solana-solana/release/fundraiser.so");
        msg!("[SETUP] The path is!! {:?}", so_path);

        msg!("[SETUP] Maker pubkey: {:?}", &maker.pubkey());
        msg!("[SETUP] Taker pubkey: {:?}", &owner.pubkey());
    
        let program_data = std::fs::read(so_path).expect("Failed to read program SO file");
    
        svm.add_program(get_program_id(), &program_data);

        let mint = CreateMint::new(&mut svm, &mint_authority)
            .decimals(6)
            .authority(&mint_authority.pubkey())
            .send()
            .unwrap();
        msg!("[SETUP] Mint A: {}", mint);


        // Create the maker's associated token account for Mint
        let maker_ata = CreateAssociatedTokenAccount::new(&mut svm, &maker, &mint)
            .owner(&maker.pubkey())
            .send()
            .unwrap();
        msg!("[SETUP] Maker ATA: {}\n", maker_ata);

        // Derive the PDA for the escrow account using the maker's public key and a seed value
        let (fundraiser, _bump) = Pubkey::find_program_address(
            &[b"fundraiser".as_ref(), maker.pubkey().as_ref()],
            &get_program_id(),
        );
        msg!("[SETUP] Fundraiser PDA: {}", fundraiser );

        // Derive the PDA for the vault associated token account using the escrow PDA and Mint A
        let vault = spl_associated_token_account::get_associated_token_address(
            &fundraiser,   // 
            &mint,         // mint
        );

        msg!("[SETUP] Vault PDA: {}\n", vault);

        // Fund accounts
        {
            MintTo::new(&mut svm, &mint_authority, &mint, &maker_ata, 1_000_000_000_000)
                .send()
                .unwrap();
        }
        litesvm_token::CreateAssociatedTokenAccount::new(&mut svm, &maker, &mint)
            .owner(&fundraiser)
            .token_program_id(&TOKEN_PROGRAM_ID)
            .send()
            .unwrap();

        (svm, maker, owner, mint_authority, mint, maker_ata, fundraiser, vault)
    }

    #[test]
    pub fn test_initialize_instruction() {
        let (
            mut svm,
            maker,
            _,
            _,
            mint,
            _,
            fundraiser,
            vault
        ) = setup();
        let amount_to_raise : u64 = 4_000_000_000_000; // 4_000_000 tokens with 6 decimal places
        let duration: u64 = 2 * 7 * 24 * 60 * 60;    // 2 weeks


        let init_data: InitializeInstruction = InitializeInstruction {
            amount_to_raise,
            duration,
        };


        let init_data_bytes = init_data.to_bytes();

        let init_data_ix = [
            vec![crate::instructions::FundraiserInstructions::Initialize as u8],
            init_data_bytes, 
        ]
        .concat();

        let init_ix = Instruction {
            program_id: get_program_id(),
            accounts: vec![
                AccountMeta::new(maker.pubkey(), true),
                AccountMeta::new(mint, false),
                AccountMeta::new(fundraiser, false),
                AccountMeta::new(vault, false),
                AccountMeta::new(system_program_id, false),
                AccountMeta::new(TOKEN_PROGRAM_ID, false),
                AccountMeta::new(ASSOCIATED_TOKEN_PROGRAM_ID, false),
                AccountMeta::new(Rent::id(), false),
            ],
            data: init_data_ix,
        };

        let tx = 
            send_singed_tx(&mut svm, init_ix, maker)
            .map_err(|e| format!("[test_initialize_instruction] Initialize Transaction Failed: {:?}", e) )
            .unwrap();

        msg!("[test_initialize_instruction] tx logs: {:#?}", tx.logs);
        msg!("[test_initialize_instruction] CUs Consumed: {}", tx.compute_units_consumed);

        // POSTCOND
        let fundraiser_pda_state = svm.get_account(&fundraiser)
            .ok_or("[test_initialize_instruction] Could not retrieve fundraiser PDA")
            .unwrap();


        let s_fundraiser =
            bytemuck::try_from_bytes::<crate::state::Fundraiser>(&fundraiser_pda_state.data).unwrap();

        assert_eq!(
            s_fundraiser.amount_to_raise.pipe(u64::from_le_bytes ),
            amount_to_raise,
            "[test_initialize_instruction] Verification failed; fundraiser account is not initialized properly"
        );
    }

    #[test]
    fn test_contribute_instruction() {
        let (
            mut svm,
            maker,
            owner,
            mint_authority,
            mint,
            maker_ata,
            fundraiser,
            vault
        ) = setup();

        // PRECOND
        {
            let amount_to_raise : u64 = 4_000_000_000_000; // 4_000_000 tokens with 6 decimal places
            let duration: u64 = 2 * 7 * 24 * 60 * 60;    // 2 weeks


            let init_data: InitializeInstruction = InitializeInstruction {
                amount_to_raise,
                duration,
            };


            let init_data_bytes = init_data.to_bytes();

            let init_data_ix = [
                vec![crate::instructions::FundraiserInstructions::Initialize as u8],
                init_data_bytes, 
            ]
            .concat();

            let init_ix = Instruction {
                program_id: get_program_id(),
                accounts: vec![
                    AccountMeta::new(maker.pubkey(), true),
                    AccountMeta::new(mint, false),
                    AccountMeta::new(fundraiser, false),
                    AccountMeta::new(vault, false),
                    AccountMeta::new(system_program_id, false),
                    AccountMeta::new(TOKEN_PROGRAM_ID, false),
                    AccountMeta::new(ASSOCIATED_TOKEN_PROGRAM_ID, false),
                    AccountMeta::new(Rent::id(), false),
                ],
                data: init_data_ix,
            };

            send_singed_tx(&mut svm, init_ix, maker)
                .map_err(|e| format!("[test_initialize_instruction] Initialize Transaction Failed: {:?}", e) )
                .unwrap();
        }


        let user = Keypair::new();

        svm.airdrop(&user.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");

        let user_ata = CreateAssociatedTokenAccount::new(&mut svm, &user, &mint)
            .owner(&user.pubkey())
            .send()
            .unwrap();

        msg!("[test_contribute_instruction] User ATA: {}\n", &user_ata);


        MintTo::new(&mut svm, &mint_authority, &mint, &user_ata, 1_000_000_000)
            .send()
            .unwrap();


        let (contributor_pda, _bump) = Pubkey::find_program_address(
            &[b"contributor".as_ref(), user.pubkey().as_ref()],
            &get_program_id(),
        );
        msg!("[test_contribute_instruction] Contributor PDA: {}\n", &contributor_pda);


        let contribute_data_ix = [
            vec![crate::instructions::FundraiserInstructions::Contribute as u8],
            10_000_000u64.to_le_bytes().to_vec(), // Discriminator for "Make" instruction
        ]
        .concat();

        let contribute_ix = Instruction {
            program_id: get_program_id(),

            accounts: vec![
                AccountMeta::new(user.pubkey(), true),
                AccountMeta::new(mint, false),
                AccountMeta::new(fundraiser, false),
                AccountMeta::new(vault, false),
                AccountMeta::new(user_ata, false),
                AccountMeta::new(contributor_pda, false),
                AccountMeta::new_readonly(system_program_id, false),

                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false),

                AccountMeta::new_readonly(Rent::id(), false),
            ],
            data: contribute_data_ix,
        };
        let tx = send_singed_tx(&mut svm, contribute_ix, user)
            .map_err(|e| format!("[test_contribute_instruction] Contribute Transaction Failed: {:?}", e) )
            .unwrap();


        msg!("tx logs: {:#?}", tx.logs);
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
    }
}
