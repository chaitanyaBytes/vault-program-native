# Native Vault

A Solana program for managing vaults that can hold either native SOL or SPL tokens.

## Features

- Initialize a new vault with an authority
- Deposit tokens (SOL or SPL) into the vault
- Withdraw tokens from the vault (authority only)
- Close the vault and reclaim rent

## Instructions

### Initialize

Creates a new vault with a specified authority. Can be configured for native SOL or SPL tokens.

### Deposit

Deposits tokens from a user's account into the vault.

### Withdraw

Withdraws tokens from the vault to a recipient account. Only the vault authority can withdraw.

### Close

Closes the vault, transfers remaining tokens to the authority, and reclaims rent.

## Building

```bash
cargo build-sbf
```

## Testing

Uses litesvm for testing. Run tests with:

```bash
cargo test-sbf
```
