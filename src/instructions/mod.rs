pub mod close;
pub mod deposit;
pub mod initialize;
pub mod withdraw;

use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize, Debug, Clone)]
pub enum VaultInstruction {
    /// Initialize a new vault
    /// Accounts required:
    /// 0. [signer] Authority (who will control the vault)
    /// 1. [writable] Vault state account (PDA)
    /// 2. [] Token mint (if SPL token vault, else can be system program)
    /// 3. [writable] Token account (ATA for holding tokens)
    /// 4. [] Token program (if SPL token)
    /// 5. [] System program
    Initialize {
        vault_bump: u8,
        state_bump: u8,
        is_native: bool,
    },

    /// Deposit tokens into the vault
    /// Accounts expected:
    /// 0. [signer] Depositor
    /// 1. [writable] Vault state account
    /// 2. [writable] Depositor's token account (source)
    /// 3. [writable] Vault's token account (destination)
    /// 4. [] Token program (if SPL token)
    /// 5. [] System program (if native SOL)
    Deposit { amount: u64 },

    /// Withdraw tokens from the vault
    /// Accounts expected:
    /// 0. [signer] Authority (must be vault authority)
    /// 1. [writable] Vault state account
    /// 2. [writable] Vault's token account (source)
    /// 3. [writable] Recipient's token account (destination)
    /// 4. [] Token program (if SPL token)
    /// 5. [] System program (if native SOL)
    Withdraw { amount: u64 },

    /// Close the vault and reclaim rent
    /// Accounts expected:
    /// 0. [signer] Authority (must be vault authority)
    /// 1. [writable] Vault state account
    /// 2. [writable] Vault's token account (to close)
    /// 3. [writable] Authority's token account (to receive tokens)
    /// 4. [] Token program (if SPL token)
    /// 5. [] System program
    Close,
}
