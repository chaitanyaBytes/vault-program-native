use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct VaultState {
    pub authority: Pubkey,     // who controls the vault
    pub token_mint: Pubkey,    // SPL token mint (or native mint)
    pub token_account: Pubkey, // asssoicated token account for holding tokens
    pub state_bump: u8,        // pda bump seed
    pub vault_bump: u8,
    pub is_native: bool, // true if SOL vault, false if SPL token vault
}

impl VaultState {
    pub fn space() -> usize {
        32 + 32 + 32 + 1 + 1 + 1
    }
}
