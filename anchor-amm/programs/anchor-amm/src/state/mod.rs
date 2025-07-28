use anchor_lang::prelude::{borsh::de, *};

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub seed: u64,
    pub authority: Option<Pubkey>, // In a case we want to unlock a pool and remove the authority and set it to null, if authority set to null we can't do anything
    pub mint_x: Pubkey,
    pub mint_y: Pubkey,
    pub fee: u16,
    pub locked: bool,
    pub config_bump: u8,
    pub lp_bump: u8,
}
