use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    system_instruction::transfer,
};
use spl_token::state::Account as TokenAccount;

use crate::state::vault_state::VaultState;

pub fn deposit(_program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let depositor = next_account_info(accounts_iter)?;
    let vault_state_pda = next_account_info(accounts_iter)?;
    let source_token_account = next_account_info(accounts_iter)?;
    let destination_token_account = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    if !depositor.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let vault_state = VaultState::try_from_slice(&vault_state_pda.data.borrow())?;

    if destination_token_account.key != &vault_state.token_account {
        return Err(ProgramError::InvalidAccountData);
    }

    if vault_state.is_native {
        if system_program.key != &solana_program::system_program::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        invoke(
            &transfer(depositor.key, destination_token_account.key, amount),
            &[
                depositor.clone(),
                destination_token_account.clone(),
                system_program.clone(),
            ],
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

        if &source_token.owner != depositor.key {
            return Err(ProgramError::InvalidAccountOwner);
        }

        if &destination_token.owner != vault_state_pda.key {
            return Err(ProgramError::InvalidAccountOwner);
        }

        if source_token.mint != destination_token.mint {
            return Err(ProgramError::InvalidAccountData);
        }

        let transfer_ix = spl_token::instruction::transfer(
            token_program.key,
            source_token_account.key,
            destination_token_account.key,
            depositor.key,
            &[],
            amount,
        )?;

        invoke(
            &transfer_ix,
            &[
                source_token_account.clone(),
                destination_token_account.clone(),
                depositor.clone(),
                token_program.clone(),
            ],
        )?;
    }

    msg!("Deposited {} tokens", amount);

    Ok(())
}
