use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    system_instruction::transfer,
};
use spl_token::state::Account as TokenAccount;

use crate::state::vault_state::VaultState;

pub fn withdraw(_program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let authority = next_account_info(accounts_iter)?;
    let vault_state_pda = next_account_info(accounts_iter)?;
    let source_token_account = next_account_info(accounts_iter)?;
    let destination_token_account = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let vault_state = VaultState::try_from_slice(&vault_state_pda.data.borrow())?;

    if &vault_state.authority != authority.key {
        return Err(ProgramError::IllegalOwner);
    }

    if source_token_account.key != &vault_state.token_account {
        return Err(ProgramError::InvalidAccountData);
    }

    if vault_state.is_native {
        if system_program.key != &solana_program::system_program::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        if source_token_account.owner != system_program.key {
            return Err(ProgramError::InvalidAccountOwner);
        }

        invoke_signed(
            &transfer(
                source_token_account.key,
                destination_token_account.key,
                amount,
            ),
            &[
                source_token_account.clone(),
                destination_token_account.clone(),
                system_program.clone(),
            ],
            &[&[
                b"vault",
                vault_state_pda.key.as_ref(),
                &[vault_state.vault_bump],
            ]],
        )?
    } else {
        if token_program.key != &spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        if source_token_account.owner != token_program.key {
            return Err(ProgramError::InvalidAccountOwner);
        }

        if destination_token_account.owner != token_program.key {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let source_token = TokenAccount::unpack(&source_token_account.data.borrow())?;

        let destination_token = TokenAccount::unpack(&destination_token_account.data.borrow())?;

        if &source_token.owner != vault_state_pda.key {
            return Err(ProgramError::InvalidAccountOwner);
        }

        if &destination_token.owner != authority.key {
            return Err(ProgramError::InvalidAccountOwner);
        }

        if source_token.mint != destination_token.mint {
            return Err(ProgramError::InvalidAccountData);
        }

        let transfer_ix = spl_token::instruction::transfer(
            token_program.key,
            source_token_account.key,
            destination_token_account.key,
            vault_state_pda.key,
            &[],
            amount,
        )?;

        invoke_signed(
            &transfer_ix,
            &[
                source_token_account.clone(),
                destination_token_account.clone(),
                vault_state_pda.clone(),
                token_program.clone(),
            ],
            &[&[
                b"vault",
                vault_state_pda.key.as_ref(),
                &[vault_state.vault_bump],
            ]],
        )?;
    }

    msg!("Withdrawn {} tokens", amount);

    Ok(())
}
