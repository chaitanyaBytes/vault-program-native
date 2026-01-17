use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction::{self, create_account},
    sysvar::Sysvar,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};

use crate::state::vault_state::VaultState;

pub fn initialize(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    vault_bump: u8,
    state_bump: u8,
    is_native: bool,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let authority = next_account_info(accounts_iter)?;
    let vault_state = next_account_info(accounts_iter)?;
    let token_mint = next_account_info(accounts_iter)?;
    let token_account = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    let associated_token_program = next_account_info(accounts_iter)?;

    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (expected_vault_state_pda, _) =
        Pubkey::find_program_address(&[b"state", authority.key.as_ref()], program_id);

    if vault_state.key != &expected_vault_state_pda {
        return Err(ProgramError::InvalidAccountData);
    }

    // Verify vault state account is uninitialized
    if vault_state.data_len() > 0 || vault_state.owner == program_id {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let space_required = VaultState::space();
    let min_lamports = Rent::get()?.minimum_balance(space_required);

    invoke_signed(
        &system_instruction::create_account(
            authority.key,
            vault_state.key,
            min_lamports,
            space_required as u64,
            program_id,
        ),
        &[
            authority.clone(),
            vault_state.clone(),
            system_program.clone(),
        ],
        &[&[b"state", authority.key.as_ref(), &[state_bump]]],
    )?;

    if !is_native {
        if token_program.key != &spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        if associated_token_program.key != &spl_associated_token_account::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        let expected_ata = get_associated_token_address(vault_state.key, token_mint.key);

        if token_account.key != &expected_ata {
            return Err(ProgramError::InvalidAccountData);
        }

        invoke(
            &create_associated_token_account(
                authority.key,
                vault_state.key,
                token_mint.key,
                token_program.key,
            ),
            &[
                authority.clone(),
                vault_state.clone(),
                token_account.clone(),
                token_mint.clone(),
                token_program.clone(),
                system_program.clone(),
                associated_token_program.clone(),
            ],
        )?;
    } else {
        let (vault_account_pda, _) =
            Pubkey::find_program_address(&[b"vault", vault_state.key.as_ref()], program_id);

        if *token_account.key != vault_account_pda {
            return Err(ProgramError::InvalidAccountData);
        }

        invoke_signed(
            &create_account(
                authority.key,
                token_account.key,
                Rent::get()?.minimum_balance(0),
                0,
                system_program.key,
            ),
            &[
                authority.clone(),
                token_account.clone(),
                system_program.clone(),
            ],
            &[&[b"vault", vault_state.key.as_ref(), &[vault_bump]]],
        )?
    }

    let vault_state_data = VaultState {
        authority: *authority.key,
        token_mint: *token_mint.key,
        token_account: *token_account.key,
        vault_bump,
        state_bump,
        is_native,
    };

    let mut account_data = &mut vault_state.data.borrow_mut()[..];
    vault_state_data.serialize(&mut account_data)?;

    msg!("Vault initialized: {:?}", vault_state.key);

    Ok(())
}
