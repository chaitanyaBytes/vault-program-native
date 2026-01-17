use borsh::BorshDeserialize;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::instructions::{
    close::close, deposit::deposit, initialize::initialize, withdraw::withdraw, VaultInstruction,
};

pub struct Processor {}

impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = VaultInstruction::try_from_slice(&instruction_data)?;

        match instruction {
            VaultInstruction::Initialize {
                vault_bump,
                state_bump,
                is_native,
            } => initialize(program_id, accounts, vault_bump, state_bump, is_native),
            VaultInstruction::Deposit { amount } => deposit(program_id, accounts, amount),
            VaultInstruction::Withdraw { amount } => withdraw(program_id, accounts, amount),
            VaultInstruction::Close => close(program_id, accounts),
        }
    }
}
