use anchor_lang::prelude::*;

/// The Escrow account holds the state for an escrow transaction.
/// It stores the maker, the mints involved, the amount to receive, and a bump for PDA.
#[account]
#[derive(InitSpace)]
pub struct Escrow {
    /// Unique seed for the escrow account
    pub seed: u64,
    /// The public key of the maker (initializer) of the escrow
    pub maker: Pubkey,
    /// The mint address of token A
    pub mint_a: Pubkey,
    /// The mint address of token B
    pub mint_b: Pubkey,
    /// Amount to be received in the escrow
    pub recieve: u64,
    /// Bump seed for the PDA
    pub bump: u8,
}
