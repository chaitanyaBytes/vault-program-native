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
use spl_token::instruction::close_account;
use spl_token::state::Account as TokenAccount;

use crate::state::vault_state::VaultState;

pub fn close(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_iter = &mut accounts.iter();

    let authority = next_account_info(account_iter)?;
    let vault_state_pda = next_account_info(account_iter)?;
    let vault_token_account = next_account_info(account_iter)?;
    let authority_token_account = next_account_info(account_iter)?;
    let token_program = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let vault_state = VaultState::try_from_slice(&vault_state_pda.data.borrow())?;

    if &vault_state.authority != authority.key {
        return Err(ProgramError::IllegalOwner);
    }

    if vault_token_account.key != &vault_state.token_account {
        return Err(ProgramError::InvalidAccountData);
    }

    if vault_state_pda.owner != program_id {
        return Err(ProgramError::InvalidAccountOwner);
    }

    if vault_state.is_native {
        if system_program.key != &solana_program::system_program::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        if vault_token_account.owner != system_program.key {
            return Err(ProgramError::InvalidAccountOwner);
        }

        invoke_signed(
            &transfer(
                vault_token_account.key,
                authority_token_account.key,
                vault_token_account.lamports(),
            ),
            &[
                vault_token_account.clone(),
                authority_token_account.clone(),
                system_program.clone(),
            ],
            &[&[
                b"vault",
                vault_state_pda.key.as_ref(),
                &[vault_state.vault_bump],
            ]],
        )?;

        invoke_signed(
            &transfer(
                vault_state_pda.key,
                authority.key,
                vault_state_pda.lamports(),
            ),
            &[
                vault_state_pda.clone(),
                authority.clone(),
                system_program.clone(),
            ],
            &[&[b"state", authority.key.as_ref(), &[vault_state.state_bump]]],
        )?
    } else {
        let vault_token = TokenAccount::unpack(&vault_token_account.data.borrow())?;
        if vault_token.amount != 0 {
            return Err(ProgramError::InvalidAccountData);
        }

        if token_program.key != &spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        if system_program.key != &solana_program::system_program::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        // 2. Close token account
        let close_ix = close_account(
            token_program.key,
            vault_token_account.key,
            authority.key,       // rent destination
            vault_state_pda.key, // authority
            &[],
        )?;

        invoke_signed(
            &close_ix,
            &[
                vault_token_account.clone(),
                authority.clone(),
                vault_state_pda.clone(),
                token_program.clone(),
            ],
            &[&[
                b"vault",
                vault_state_pda.key.as_ref(),
                &[vault_state.vault_bump],
            ]],
        )?;

        // 3. Close state PDA
        invoke_signed(
            &transfer(
                vault_state_pda.key,
                authority.key,
                vault_state_pda.lamports(),
            ),
            &[
                vault_state_pda.clone(),
                authority.clone(),
                system_program.clone(),
            ],
            &[&[b"state", authority.key.as_ref(), &[vault_state.state_bump]]],
        )?;
    }

    msg!("vault closed");

    Ok(())
}
