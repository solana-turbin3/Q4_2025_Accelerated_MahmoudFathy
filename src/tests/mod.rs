#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use litesvm::LiteSVM;
    use litesvm_token::{spl_token::{self, solana_program::{msg, rent::Rent, sysvar::SysvarId}}, CreateAssociatedTokenAccount, CreateMint, MintTo};
    
    use solana_instruction::{AccountMeta, Instruction};
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_native_token::LAMPORTS_PER_SOL;
    use solana_pubkey::Pubkey;
    use solana_signer::Signer;
    use solana_transaction::Transaction;
    use pinocchio_token::state::TokenAccount;

    const PROGRAM_ID: &str = "RURxJrgqHSoJgqFbHyntxm1VSZxZugEveRewzvjVm5V";
    const TOKEN_PROGRAM_ID: Pubkey = spl_token::ID;
    const ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";
    
    fn program_id() -> Pubkey {
        Pubkey::from(crate::ID)
    }

    fn setup() -> (LiteSVM, Keypair, Keypair) {

        let mut svm = LiteSVM::new();
        let payer = Keypair::new();   // also maker
        let taker = Keypair::new();

        svm
            .airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");
        svm
            .airdrop(&taker.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");

        // Load program SO file
        msg!("The path is!! {}", env!("CARGO_MANIFEST_DIR"));
        let so_path = PathBuf::from("./target/sbf-solana-solana/release/escrow.so");
        msg!("The path is!! {:?}", so_path);

        msg!("Maker pubkey: {:?}", &payer.pubkey());
        msg!("Taker pubkey: {:?}", &taker.pubkey());
    
        let program_data = std::fs::read(so_path).expect("Failed to read program SO file");
    
        svm.add_program(program_id(), &program_data);

        (svm, payer, taker)
        
    }

    #[test]
    #[ignore]
    pub fn test_make_instruction() {
        let (mut svm, payer, _) = setup();

        let program_id = program_id();

        assert_eq!(program_id.to_string(), PROGRAM_ID);

        let mint_a = CreateMint::new(&mut svm, &payer)
            .decimals(6)
            .authority(&payer.pubkey())
            .send()
            .unwrap();
        msg!("Mint A: {}", mint_a);

        let mint_b = CreateMint::new(&mut svm, &payer)
            .decimals(6)
            .authority(&payer.pubkey())
            .send()
            .unwrap();
        msg!("Mint B: {}", mint_b);

        // Create the maker's associated token account for Mint A
        let maker_ata_a = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint_a)
            .owner(&payer.pubkey()).send().unwrap();
        msg!("Maker ATA A: {}\n", maker_ata_a);

        // Derive the PDA for the escrow account using the maker's public key and a seed value
        let escrow = Pubkey::find_program_address(
            &[b"escrow".as_ref(), payer.pubkey().as_ref()],
            &PROGRAM_ID.parse().unwrap(),
        );
        msg!("Escrow PDA: {}\n", escrow.0);

        // Derive the PDA for the vault associated token account using the escrow PDA and Mint A
        let vault = spl_associated_token_account::get_associated_token_address(
            &escrow.0,  // owner will be the escrow PDA
            &mint_a     // mint
        );
        msg!("Vault PDA: {}\n", vault);

        // Define program IDs for associated token program, token program, and system program
        let asspciated_token_program = ASSOCIATED_TOKEN_PROGRAM_ID.parse::<Pubkey>().unwrap();
        let token_program = TOKEN_PROGRAM_ID;
        let system_program = solana_sdk_ids::system_program::ID;

        // Mint 1,000 tokens (with 6 decimal places) of Mint A to the maker's associated token account
        MintTo::new(&mut svm, &payer, &mint_a, &maker_ata_a, 1000000000)
            .send()
            .unwrap();

        let amount_to_receive: u64 = 100000000; // 100 tokens with 6 decimal places
        let amount_to_give: u64 = 500000000;    // 500 tokens with 6 decimal places
        let bump: u8 = escrow.1;

        msg!("Bump: {}", bump);

        // Create the "Make" instruction to deposit tokens into the escrow
        let make_data = [
            vec![0u8],              // Discriminator for "Make" instruction
            bump.to_le_bytes().to_vec(),
            amount_to_receive.to_le_bytes().to_vec(),
            amount_to_give.to_le_bytes().to_vec(),
        ].concat();
        let make_ix = Instruction {
            program_id: program_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(mint_a, false),
                AccountMeta::new(mint_b, false),
                AccountMeta::new(escrow.0, false),
                AccountMeta::new(maker_ata_a, false),
                AccountMeta::new(vault, false),
                AccountMeta::new(system_program, false),
                AccountMeta::new(token_program, false),
                AccountMeta::new(asspciated_token_program, false),
                AccountMeta::new(Rent::id(), false),
            ],
            data: make_data,
        };

        // Create and send the transaction containing the "Make" instruction
        let message = Message::new(&[make_ix], Some(&payer.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[&payer], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx = svm.send_transaction(transaction).unwrap();

        // Log transaction details
        msg!("\n\nMake transaction sucessfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
    }


    #[test]
    pub fn test_take_instruction() {
        let (mut svm, payer, taker) = setup();

        let program_id = program_id();

        assert_eq!(program_id.to_string(), PROGRAM_ID);

        let mint_a = CreateMint::new(&mut svm, &payer)
            .decimals(6)
            .authority(&payer.pubkey())
            .send()
            .unwrap();
        msg!("Mint A: {}", mint_a);

        let mint_b = CreateMint::new(&mut svm, &payer)
            .decimals(6)
            .authority(&payer.pubkey())
            .send()
            .unwrap();
        msg!("Mint B: {}", mint_b);

        // Create the maker's associated token account for Mint A
        let maker_ata_a = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint_a)
            .owner(&payer.pubkey()).send().unwrap();
        msg!("Maker ATA A: {}\n", maker_ata_a);

        // Create the maker's associated token account for Mint B
        let maker_ata_b = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint_b)
            .owner(&payer.pubkey()).send().unwrap();
        msg!("Maker ATA B: {}\n", maker_ata_b);
        // Create the taker's associated token account for Mint A
        let taker_ata_a = CreateAssociatedTokenAccount::new(&mut svm, &taker, &mint_a)
            .owner(&taker.pubkey()).send().unwrap();
        msg!("Taker ATA A: {}\n", taker_ata_a);
        // Create the taker's associated token account for Mint B
        let taker_ata_b = CreateAssociatedTokenAccount::new(&mut svm, &taker, &mint_b)
            .owner(&taker.pubkey()).send().unwrap();
        msg!("Taker ATA B: {}\n", taker_ata_b);



        // Derive the PDA for the escrow account using the maker's public key and a seed value
        let escrow = Pubkey::find_program_address(
            &[b"escrow".as_ref(), payer.pubkey().as_ref()],
            &PROGRAM_ID.parse().unwrap(),
        );
        msg!("Escrow PDA: {}\n", escrow.0);

        // Derive the PDA for the vault associated token account using the escrow PDA and Mint A
        let vault = spl_associated_token_account::get_associated_token_address(
            &escrow.0,  // owner will be the escrow PDA
            &mint_a     // mint
        );
        msg!("Vault PDA: {}\n", vault);

        // Define program IDs for associated token program, token program, and system program
        let asspciated_token_program = ASSOCIATED_TOKEN_PROGRAM_ID.parse::<Pubkey>().unwrap();
        let token_program = TOKEN_PROGRAM_ID;
        let system_program = solana_sdk_ids::system_program::ID;


        let amount_to_receive: u64 = 100000000; // 100 tokens with 6 decimal places
        let amount_to_give: u64 = 500000000;    // 500 tokens with 6 decimal places
        let bump: u8 = escrow.1;

        // Mint 1,000 tokens (with 6 decimal places) of Mint A to the maker's associated token account
        MintTo::new(&mut svm, &payer, &mint_a, &maker_ata_a, 1000000000)
            .send()
            .unwrap();
        // Mint 1,000 tokens (with 6 decimal places) of Mint B to the taker's associated token account
        MintTo::new(&mut svm, &payer, &mint_b, &taker_ata_b, 1000000000)
            .send()
            .unwrap();


        msg!("Bump: {}", bump);

        // Create the "Make" instruction to deposit tokens into the escrow
        let make_data = [
            vec![0u8],              // Discriminator for "Make" instruction
            bump.to_le_bytes().to_vec(),
            amount_to_receive.to_le_bytes().to_vec(),
            amount_to_give.to_le_bytes().to_vec(),
        ].concat();
        let make_ix = Instruction {
            program_id: program_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(mint_a, false),
                AccountMeta::new(mint_b, false),
                AccountMeta::new(escrow.0, false),
                AccountMeta::new(maker_ata_a, false),
                AccountMeta::new(vault, false),
                AccountMeta::new(system_program, false),
                AccountMeta::new(token_program, false),
                AccountMeta::new(asspciated_token_program, false),
                AccountMeta::new(Rent::id(), false),
            ],
            data: make_data,
        };

        // Create and send the transaction containing the "Make" instruction
        let message = Message::new(&[make_ix], Some(&payer.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[&payer], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx = svm.send_transaction(transaction).unwrap();

        // Log transaction details
        msg!("\n\nMake transaction sucessfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);


        // Take 
        // Create the "Take" instruction to deposit tokens into the escrow
        let take_ix_data = [
            vec![1u8],              // Discriminator for "Take" instruction
            amount_to_receive.to_le_bytes().to_vec(),    // Amount received by maker from taker
            amount_to_give.to_le_bytes().to_vec(),       // Amount given by maker to taker
        ].concat();
        let take_ix = Instruction {
            program_id: program_id,
            accounts: vec![
                AccountMeta::new(taker.pubkey(), true),
                AccountMeta::new(mint_a, false),
                AccountMeta::new(mint_b, false),
                AccountMeta::new(escrow.0, false),
                AccountMeta::new(maker_ata_b, false),
                AccountMeta::new(taker_ata_b, false),
                AccountMeta::new(taker_ata_a, false),
                AccountMeta::new(vault, false),
                AccountMeta::new(system_program, false),
                AccountMeta::new(token_program, false),
                AccountMeta::new(asspciated_token_program, false),
                AccountMeta::new(Rent::id(), false),
            ],
            data: take_ix_data,
        };

        // Create and send the transaction containing the "Make" instruction
        let message = Message::new(&[take_ix], Some(&taker.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[&taker], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx = svm.send_transaction(transaction).unwrap();

        // Log transaction details
        msg!("\n\nTake transaction sucessfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);

        // POSTCONDITIONS
        // Get the account data
        let maker_ata_b_account = svm.get_account(&maker_ata_b).unwrap();

        // Deserialize to TokenAccount
        let token_account_data = unsafe { TokenAccount::from_bytes_unchecked(&maker_ata_b_account.data) };

        // Get the balance
        let balance = token_account_data.amount();
        assert_eq!(balance, amount_to_receive, "Maker did not receive their mint_b");
        
        // Get the account data
        let taker_ata_a_account = svm.get_account(&taker_ata_a).unwrap();

        // Deserialize to TokenAccount
        let token_account_data = unsafe { TokenAccount::from_bytes_unchecked(&taker_ata_a_account.data) };

        // Get the balance
        let balance = token_account_data.amount();

        assert_eq!(balance, amount_to_give, "Taker did not receive their mint_a");

    }
}
