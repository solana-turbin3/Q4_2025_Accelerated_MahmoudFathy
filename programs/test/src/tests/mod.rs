#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use {
        anchor_lang::{
            prelude::msg,
            InstructionData,
            ToAccountMetas,
        },
        anchor_spl::{
            associated_token,
        },
        solana_sdk::{
            instruction::{Instruction},
            transaction::Transaction,
            pubkey::Pubkey,
            message::Message,
            signature::{Keypair, Signer},
            native_token::LAMPORTS_PER_SOL,
            system_instruction,
            hash::Hash,
            system_program,
        },
        litesvm::LiteSVM,
        litesvm_token::{
            CreateAssociatedTokenAccount
        },
        solana_program_pack::Pack,
        spl_token_2022::{
            instruction::initialize_mint2,
            ID as TOKEN_22_PROGRAM_ID,
        },
        std::path::PathBuf,

        transfer_hook_vault_litesvm as transfer_hook,
    };

    static TRANSFER_HOOK_PROGRAM_ID: Pubkey = transfer_hook::ID;
    static VAULT_PROGRAM_ID: Pubkey = vault::ID;
    
    fn create_token_with_extension_22(
        svm: &mut LiteSVM,
        payer: &Keypair,
        mint: &Keypair,
        mint_authority: &Pubkey,
        decimals: u8,
    ) -> Result<(), String> {
        let minimum_required_rent = svm.minimum_balance_for_rent_exemption(spl_token_2022::state::Mint::LEN);
        let create_account_instruction = system_instruction::create_account(
            &payer.pubkey(),
            &mint.pubkey(),
            minimum_required_rent,
            spl_token_2022::state::Mint::LEN as u64,
            &TOKEN_22_PROGRAM_ID,
        );
        let init_mint_instruction = initialize_mint2(
            &TOKEN_22_PROGRAM_ID,
            &mint.pubkey(),
            mint_authority,
            None,
            decimals,
        ).unwrap();


        let message = Message::new(
            &[create_account_instruction, init_mint_instruction],
            Some(&payer.pubkey()),
        );
        let recent_blockhash = svm.latest_blockhash();
        let transaction = Transaction::new(&[payer, mint], message, Hash::from(recent_blockhash.to_bytes()) );
        
        svm.send_transaction(  transaction )
            .expect("Failed to send transaction create token 2022");

        Ok(())
    }

    fn setup() -> (
        LiteSVM,
        Keypair,
        Keypair,
        Keypair,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey
    ) {
        
        let mut svm = LiteSVM::new();
        let payer = Keypair::new();

        svm.airdrop(&payer.pubkey(), 8 * LAMPORTS_PER_SOL)
            .expect("Airdrop Failed!!");

        let transfer_hook_so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/deploy/transfer_hook_vault_litesvm.so");
        let transfer_hook_data =
            std::fs::read(transfer_hook_so_path).expect("Failed to read transfer_hook SO file");
        svm.add_program(TRANSFER_HOOK_PROGRAM_ID, &transfer_hook_data)
            .expect("Failed to add transfer_hook program");

        let vault_so_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/deploy/vault.so");
        let vault_data = std::fs::read(vault_so_path).expect("Failed to read week1_challenge SO file");
        svm.add_program(VAULT_PROGRAM_ID, &vault_data)
            .expect("Failed to add week1_challenge program");
        

        // Initialize Accounts
        let mint_authority = payer.pubkey();
        let user = Keypair::new();

        svm.airdrop(&user.pubkey(), 2 * LAMPORTS_PER_SOL)
            .expect("Airdrop Failed!!");

        let mint_keypair = Keypair::new();

        msg!("\n1: Creating token22 Mint");

        create_token_with_extension_22(
            &mut svm,
            &payer,
            &mint_keypair,
            &mint_authority,
            9,
        )
        .expect("Failed to create Token22 with Extension");
        //
        let mint = mint_keypair.pubkey();

        let (whitelist_user_pda, _) = Pubkey::find_program_address(&[b"whitelist", mint.as_ref(), user.pubkey().as_ref()], &TRANSFER_HOOK_PROGRAM_ID);

        let (vault_authority, _) = Pubkey::find_program_address(&[b"vault", mint.as_ref()], &VAULT_PROGRAM_ID);

        let (vault_state, _) = Pubkey::find_program_address(&[b"vault_state", mint.as_ref()], &VAULT_PROGRAM_ID);

        let (whitelist_vault_pda, _) = Pubkey::find_program_address(&[b"whitelist", mint.as_ref(), vault_authority.as_ref()], &TRANSFER_HOOK_PROGRAM_ID);

        let (extra_account_meta_list, _) = Pubkey::find_program_address(
            &[b"extra-account-metas", mint.as_ref()],
            &TRANSFER_HOOK_PROGRAM_ID,
        );

        //
        msg!("\n2: Initialize Meta list for transfer hook");

        let init_extra_account_metalist_ix = Instruction {
            program_id: TRANSFER_HOOK_PROGRAM_ID,
            accounts: transfer_hook::accounts::InitializeExtraAccountMetaList {
                payer: payer.pubkey(),
                extra_account_meta_list,
                mint,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
            data: transfer_hook::instruction::InitializeTransferHook {}.data(),

        };
        //
        let message = Message::new(&[init_extra_account_metalist_ix], Some(&payer.pubkey()));
        let recent_blockhash = svm.latest_blockhash();
        let transaction = Transaction::new(&[&payer], message, recent_blockhash);

        svm.send_transaction(transaction).unwrap();
        msg!("Extra account meta list initialized");
        //
        //
        msg!("\n3: Initializing vault");
        let vault_ata = associated_token::get_associated_token_address_with_program_id(
            &vault_authority,
            &mint,
            &TOKEN_22_PROGRAM_ID,
        );

        let create_vault_ix = Instruction {
            program_id: VAULT_PROGRAM_ID,
            accounts: vault::accounts::InitializeVault {
                payer: payer.pubkey(),
                vault_authority,
                vault_state,
                mint,
                vault_ata,
                token_program: TOKEN_22_PROGRAM_ID,
                associated_token_program: associated_token::ID,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
            data: vault::instruction::Initialize {
                owner: payer.pubkey(),
            }
            .data(),
        };
        //
        let message = Message::new(&[create_vault_ix], Some(&payer.pubkey()));
        let recent_blockhash = svm.latest_blockhash();
        let transaction = Transaction::new(&[&payer], message, recent_blockhash);
        svm.send_transaction(transaction).unwrap();
        msg!("Vault created");

        //
        //
        msg!("\n3: Create user ata");
        let user_ata = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint)
            .owner(&user.pubkey())
            .token_program_id(&TOKEN_22_PROGRAM_ID)
            .send()
            .expect("Failed to create user ATA");

        // Mint tokens to user
        let fund_user_ix = spl_token_2022::instruction::mint_to(
            &TOKEN_22_PROGRAM_ID,
            &mint,
            &user_ata,
            &mint_authority,
            &[],
            10_000_000_000, 
        )
        .unwrap();

        //
        let message = Message::new(&[fund_user_ix], Some(&payer.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[&payer], message, recent_blockhash);
        svm.send_transaction(transaction).unwrap();
        msg!("Minted tokens to user");
        ( 
            svm,
            payer,
            user,
            mint_keypair,
            vault_state,
            user_ata,
            vault_authority,
            vault_ata,
            extra_account_meta_list,
            whitelist_user_pda,
            whitelist_vault_pda
        )
    }


    #[test]
    fn test_valid_token_transfer() {
        let (
            mut svm,
            payer,
            user,
            mint_keypair,
            vault_state,
            user_ata,
            vault_authority,
            vault_ata,
            extra_account_meta_list,
            whitelist_user_pda,
            whitelist_vault_pda
        ) = setup();
        let mint = mint_keypair.pubkey();

        msg!("\n4: Add user to special list");
        //
        let add_to_whitelist_ix = Instruction {
            program_id: TRANSFER_HOOK_PROGRAM_ID,
            accounts: transfer_hook::accounts::WhitelistOperations {
                admin: payer.pubkey(),
                mint: mint,
                whitelisted_account: whitelist_user_pda,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
            data: transfer_hook::instruction::AddWhitelistedAccount{
                user: user.pubkey(),
            }
            .data(),
        };

        let message = Message::new(&[add_to_whitelist_ix], Some(&payer.pubkey()));
        let recent_blockhash = svm.latest_blockhash();
        let transaction = Transaction::new(&[&payer], message, recent_blockhash);
        svm.send_transaction(transaction).unwrap();
        msg!("User added to Whitelist");

        msg!("\n5: Add vault to special list");
        let add_to_restrictedlist_ix = Instruction {
            program_id: TRANSFER_HOOK_PROGRAM_ID,
            accounts: transfer_hook::accounts::WhitelistOperations {
                admin: payer.pubkey(),
                mint: mint,
                whitelisted_account: whitelist_vault_pda,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
            data: transfer_hook::instruction::AddRestrictedAccount{
                user: vault_authority,
            }
            .data(),
        };

        let message = Message::new(&[add_to_restrictedlist_ix], Some(&payer.pubkey()));
        let recent_blockhash = svm.latest_blockhash();
        let transaction = Transaction::new(&[&payer], message, recent_blockhash);
        svm.send_transaction(transaction).unwrap();
        msg!("Vault added to Restricted list");


        msg!("\n6: Deposit to vault");
        //
        let deposit_ix = Instruction {
            program_id: VAULT_PROGRAM_ID,
            accounts: vault::accounts::Deposit {
                user: user.pubkey(),
                vault_state,
                mint,
                user_ata,
                vault_authority,
                vault_ata,
                extra_account_meta_list,
                source_token_whitelist_state: whitelist_user_pda,
                destination_token_whitelist_state: whitelist_vault_pda,
                token_program: TOKEN_22_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: vault::instruction::Deposit { amount: 1_000_000_000 }.data(),
        };
        //
        let message = Message::new(&[deposit_ix], Some(&user.pubkey()));
        let recent_blockhash = svm.latest_blockhash();
        let transaction = Transaction::new(&[&user], message, recent_blockhash);
        let result = svm.send_transaction(transaction);
        assert!(
            result.is_ok(),
            "Expected deposit to succeed, but it failed"
        );
    }

    #[test]
    #[ignore]
    fn test_invalid_token_transfer_expect_revert() {
        let (
            mut svm,
            payer,
            user,
            mint_keypair,
            vault_state,
            user_ata,
            vault_authority,
            vault_ata,
            extra_account_meta_list,
            whitelist_user_pda,
            whitelist_vault_pda,
        ) = setup();
        let mint = mint_keypair.pubkey();

        msg!("\n3: Add vault to restricted list");
        //
        let add_to_restrictedlist_ix = Instruction {
            program_id: TRANSFER_HOOK_PROGRAM_ID,
            accounts: transfer_hook::accounts::WhitelistOperations {
                admin: payer.pubkey(),
                mint: mint,
                whitelisted_account: whitelist_vault_pda,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
            data: transfer_hook::instruction::AddRestrictedAccount{
                user: vault_authority,
            }
            .data(),
        };

        let message = Message::new(&[add_to_restrictedlist_ix], Some(&payer.pubkey()));
        let recent_blockhash = svm.latest_blockhash();
        let transaction = Transaction::new(&[&payer], message, recent_blockhash);
        svm.send_transaction(transaction).unwrap();
        msg!("Vault added to restricted list");
        //
        // Other party is not added to a whitelist, therefore deposit should fail

        msg!("\n4: Deposit to vault");
        //
        let deposit_ix = Instruction {
            program_id: VAULT_PROGRAM_ID,
            accounts: vault::accounts::Deposit {
                user: user.pubkey(),
                vault_state,
                mint,
                user_ata,
                vault_authority,
                vault_ata,
                extra_account_meta_list,
                source_token_whitelist_state: whitelist_user_pda,
                destination_token_whitelist_state: whitelist_vault_pda,
                token_program: TOKEN_22_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: vault::instruction::Deposit { amount: 1_000_000_000 }.data(),
        };
        //
        let message = Message::new(&[deposit_ix], Some(&user.pubkey()));
        let recent_blockhash = svm.latest_blockhash();
        let transaction = Transaction::new(&[&user], message, recent_blockhash);
        let result = svm.send_transaction(transaction);
        assert!(
            result.is_err(),
            "Expected deposit to fail for restricted vault, but it succeeded"
        );
    }

}

