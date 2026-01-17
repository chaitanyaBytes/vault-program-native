use std::str::FromStr;

use borsh::BorshDeserialize;
use litesvm::LiteSVM;

use native_vault::instructions::VaultInstruction;
use solana_program::program_pack::Pack;
use solana_sdk::{
    message::{AccountMeta, Instruction},
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    rent::Rent,
    signature::Keypair,
    signer::Signer,
    sysvar::Sysvar,
    transaction::Transaction,
};
use solana_system_interface::{instruction::create_account, program};
use spl_associated_token_account::ID as ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID;
use spl_token::{state::Mint, ID as TOKEN_PROGRAM_ID};
use spl_token_interface::instruction::initialize_mint;

#[test]
pub fn test_vault_sol() {
    let mut svm = LiteSVM::new();

    let authority = Keypair::new();
    svm.airdrop(&authority.pubkey(), 5 * LAMPORTS_PER_SOL)
        .expect("Failed to airdrop");

    let program_id = Pubkey::from_str("BwzUsvj7pXh8h2fEWCmawbSaGXjzi4yV1ftnztBJq3Ba").unwrap();
    let program_bytes = include_bytes!("../../target/deploy/vault_native.so");
    svm.add_program(program_id, program_bytes)
        .expect("faield to laod program");

    let token_program_id = Pubkey::try_from_slice(TOKEN_PROGRAM_ID.as_ref()).unwrap();
    let associated_token_program_id: Pubkey =
        Pubkey::try_from_slice(ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID.as_ref()).unwrap();

    let (vault_state_pda, state_bump) =
        Pubkey::find_program_address(&[b"state", authority.pubkey().as_ref()], &program_id);

    let (vault_account_pda, vault_bump) =
        Pubkey::find_program_address(&[b"vault", vault_state_pda.as_ref()], &program_id);

    let token_mint = solana_system_interface::program::ID;

    // 1. initialise the vault
    let ix_data = borsh::to_vec(&VaultInstruction::Initialize {
        vault_bump,
        state_bump,
        is_native: true,
    })
    .expect("Failed to serialize");

    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(authority.pubkey(), true),
            AccountMeta::new(vault_state_pda, false),
            AccountMeta::new(token_mint, false),
            AccountMeta::new(vault_account_pda, false),
            AccountMeta::new_readonly(token_program_id, false),
            AccountMeta::new_readonly(program::ID, false),
            AccountMeta::new_readonly(associated_token_program_id, false),
        ],
        data: ix_data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&authority.pubkey()),
        &[&authority],
        svm.latest_blockhash(),
    );

    let result = svm.send_transaction(tx);
    assert!(result.is_ok());

    let logs = result.unwrap().pretty_logs();
    println!("{}", logs);

    let vault_state = svm.get_account(&vault_state_pda).unwrap();
    println!("vault state pda lamports = {}", vault_state.lamports);

    // 2. deposit in the vault
    let ix_data = borsh::to_vec(&VaultInstruction::Deposit {
        amount: 1 * LAMPORTS_PER_SOL,
    })
    .expect("Failed to serialize");

    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(authority.pubkey(), true),
            AccountMeta::new(vault_state_pda, false),
            AccountMeta::new(authority.pubkey(), false),
            AccountMeta::new(vault_account_pda, false),
            AccountMeta::new_readonly(token_program_id, false),
            AccountMeta::new_readonly(program::ID, false),
        ],
        data: ix_data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&authority.pubkey()),
        &[&authority],
        svm.latest_blockhash(),
    );

    let result = svm.send_transaction(tx);
    assert!(result.is_ok());

    let logs = result.unwrap().pretty_logs();
    println!("{}", logs);

    let vault_account = svm.get_account(&vault_account_pda).unwrap();
    println!("vault account pda lamports = {}", vault_account.lamports);

    // 3. withdraw from the vault
    let ix_data = borsh::to_vec(&VaultInstruction::Withdraw {
        amount: 1 * LAMPORTS_PER_SOL,
    })
    .expect("Failed to serialize");

    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(authority.pubkey(), true),
            AccountMeta::new(vault_state_pda, false),
            AccountMeta::new(vault_account_pda, false),
            AccountMeta::new(authority.pubkey(), false),
            AccountMeta::new_readonly(token_program_id, false),
            AccountMeta::new_readonly(program::ID, false),
        ],
        data: ix_data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&authority.pubkey()),
        &[&authority],
        svm.latest_blockhash(),
    );

    let result = svm.send_transaction(tx);
    assert!(result.is_ok());

    let logs = result.unwrap().pretty_logs();
    println!("{}", logs);

    let vault_account = svm.get_account(&vault_account_pda).unwrap();
    println!("vault account pda lamports = {}", vault_account.lamports);

    // 3. close the vault
    let ix_data = borsh::to_vec(&VaultInstruction::Close).expect("Failed to serialize");

    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(authority.pubkey(), true),
            AccountMeta::new(vault_state_pda, false),
            AccountMeta::new(vault_account_pda, false),
            AccountMeta::new(authority.pubkey(), false),
            AccountMeta::new_readonly(token_program_id, false),
            AccountMeta::new_readonly(program::ID, false),
        ],
        data: ix_data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&authority.pubkey()),
        &[&authority],
        svm.latest_blockhash(),
    );

    let result = svm.send_transaction(tx);

    // Print logs regardless of success/failure to see where execution stopped
    if let Ok(response) = &result {
        let logs = response.pretty_logs();
        println!("Transaction logs:\n{}", logs);
    } else if let Err(e) = &result {
        // Try to get logs from error if available
        eprintln!("Transaction failed: {:?}", e);
    }

    assert!(result.is_ok());

    let logs = result.unwrap().pretty_logs();
    println!("{}", logs);
}

#[test]
pub fn test_vault_spl() {
    let mut svm = LiteSVM::new();

    let authority = Keypair::new();
    svm.airdrop(&authority.pubkey(), 5 * LAMPORTS_PER_SOL)
        .expect("Failed to airdrop");

    let program_id = Pubkey::from_str("BwzUsvj7pXh8h2fEWCmawbSaGXjzi4yV1ftnztBJq3Ba").unwrap();
    let program_bytes = include_bytes!("../../target/deploy/vault_native.so");
    svm.add_program(program_id, program_bytes)
        .expect("faield to laod program");

    let token_program_id = Pubkey::try_from_slice(TOKEN_PROGRAM_ID.as_ref()).unwrap();
    let associated_token_program_id: Pubkey =
        Pubkey::try_from_slice(ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID.as_ref()).unwrap();

    let (vault_state_pda, state_bump) =
        Pubkey::find_program_address(&[b"state", authority.pubkey().as_ref()], &program_id);

    let token_mint = Keypair::new();
    let mint_authority = Keypair::new();
    svm.airdrop(&mint_authority.pubkey(), LAMPORTS_PER_SOL)
        .expect("failed to airdrop mint auth");

    let create_mint_ix = create_account(
        &mint_authority.pubkey(),
        &token_mint.pubkey(),
        Rent::default().minimum_balance(Mint::LEN),
        Mint::LEN as u64,
        &token_program_id,
    );

    let init_mint_ix = initialize_mint(
        &token_program_id,
        &token_mint.pubkey(),
        &mint_authority.pubkey(),
        None,
        9,
    );

    let (vault_token_account, vault_bump) = Pubkey::find_program_address(
        &[
            &vault_state_pda.to_bytes(),
            &token_program_id.to_bytes(),
            &token_mint.pubkey().to_bytes(),
        ],
        &associated_token_program_id,
    );

    let (authority_token_account, _) = Pubkey::find_program_address(
        &[
            &authority.pubkey().to_bytes(),
            &token_program_id.to_bytes(),
            &token_mint.pubkey().to_bytes(),
        ],
        &associated_token_program_id,
    );

    // 1. initialise the vault
    let ix_data = borsh::to_vec(&VaultInstruction::Initialize {
        vault_bump,
        state_bump,
        is_native: false,
    })
    .expect("Failed to serialize");

    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(authority.pubkey(), true),
            AccountMeta::new(vault_state_pda, false),
            AccountMeta::new(token_mint.pubkey(), false),
            AccountMeta::new(vault_token_account, false),
            AccountMeta::new_readonly(token_program_id, false),
            AccountMeta::new_readonly(program::ID, false),
            AccountMeta::new_readonly(associated_token_program_id, false),
        ],
        data: ix_data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&authority.pubkey()),
        &[&authority],
        svm.latest_blockhash(),
    );

    let result = svm.send_transaction(tx);

    if let Ok(response) = &result {
        let logs = response.pretty_logs();
        println!("Transaction logs:\n{}", logs);
    } else if let Err(e) = &result {
        // Try to get logs from error if available
        eprintln!("Transaction failed: {:?}", e);
    }

    assert!(result.is_ok());

    let logs = result.unwrap().pretty_logs();
    println!("{}", logs);

    let vault_state = svm.get_account(&vault_state_pda).unwrap();
    println!("vault state pda lamports = {}", vault_state.lamports);

    // 2. deposit in the vault
    let ix_data = borsh::to_vec(&VaultInstruction::Deposit {
        amount: 1 * LAMPORTS_PER_SOL,
    })
    .expect("Failed to serialize");

    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(authority.pubkey(), true),
            AccountMeta::new(vault_state_pda, false),
            AccountMeta::new(authority_token_account, false),
            AccountMeta::new(vault_token_account, false),
            AccountMeta::new_readonly(token_program_id, false),
            AccountMeta::new_readonly(program::ID, false),
        ],
        data: ix_data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&authority.pubkey()),
        &[&authority],
        svm.latest_blockhash(),
    );

    let result = svm.send_transaction(tx);
    assert!(result.is_ok());

    let logs = result.unwrap().pretty_logs();
    println!("{}", logs);

    let vault_account = svm.get_account(&vault_token_account).unwrap();
    println!("vault account pda lamports = {}", vault_account.lamports);

    // 3. withdraw from the vault
    let ix_data = borsh::to_vec(&VaultInstruction::Withdraw {
        amount: 1 * LAMPORTS_PER_SOL,
    })
    .expect("Failed to serialize");

    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(authority.pubkey(), true),
            AccountMeta::new(vault_state_pda, false),
            AccountMeta::new(vault_token_account, false),
            AccountMeta::new(authority_token_account, false),
            AccountMeta::new_readonly(token_program_id, false),
            AccountMeta::new_readonly(program::ID, false),
        ],
        data: ix_data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&authority.pubkey()),
        &[&authority],
        svm.latest_blockhash(),
    );

    let result = svm.send_transaction(tx);
    assert!(result.is_ok());

    let logs = result.unwrap().pretty_logs();
    println!("{}", logs);

    let vault_account = svm.get_account(&vault_token_account).unwrap();
    println!("vault account pda lamports = {}", vault_account.lamports);

    // 3. close the vault
    let ix_data = borsh::to_vec(&VaultInstruction::Close).expect("Failed to serialize");

    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(authority.pubkey(), true),
            AccountMeta::new(vault_state_pda, false),
            AccountMeta::new(vault_token_account, false),
            AccountMeta::new(authority_token_account, false),
            AccountMeta::new_readonly(token_program_id, false),
            AccountMeta::new_readonly(program::ID, false),
        ],
        data: ix_data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&authority.pubkey()),
        &[&authority],
        svm.latest_blockhash(),
    );

    let result = svm.send_transaction(tx);

    // Print logs regardless of success/failure to see where execution stopped
    if let Ok(response) = &result {
        let logs = response.pretty_logs();
        println!("Transaction logs:\n{}", logs);
    } else if let Err(e) = &result {
        // Try to get logs from error if available
        eprintln!("Transaction failed: {:?}", e);
    }

    assert!(result.is_ok());

    let logs = result.unwrap().pretty_logs();
    println!("{}", logs);
}
