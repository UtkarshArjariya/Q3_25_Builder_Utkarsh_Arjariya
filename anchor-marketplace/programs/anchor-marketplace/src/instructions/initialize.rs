use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::metadata::Metadata;
use anchor_spl::token_interface::{Mint, TokenInterface};

// Accounts required to initialize a marketplace.
#[derive(Accounts)]
#[instruction(name: String)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>, // The signer who will be the admin of the marketplace.

    #[account(
        init,
        payer = admin,
        seeds = [b"marketplace", name.as_bytes()],
        bump,
        space = 8 + Marketplace::InitSpace
    )]
    pub marketplace: Account<'info, Marketplace>, // Main marketplace PDA derived from name.

    #[account(
        seeds = [b"treasury", marketplace.key().as_ref()],
        bump,
    )]
    pub treasury: SystemAccount<'info>, //  Treasury PDA to collect marketplace fees.

    #[account(
        init,
        payer = admin,
        seeds = [b"rewards", marketplace.key().as_ref()],
        bump,
        mint::decimals = 6,
        mint::authority = marketplace,
    )]
    pub rewards_mint: InterfaceAccount<'info, Mint>, // Reward token mint

    pub system_program: Program<'info, System>, // Required for creating accounts.
    pub token_program: Interface<'info, TokenInterface>, // Required for token operations.
    pub associated_token_program: Program<'info, AssociatedToken>, // Required for associated token accounts.
    pub metadata_program: Program<'info, Metadata>, // Required for metadata operations.
}

impl<'info> Initialize<'info> {
    // Initialize the marketplace with the provided name and fee.
    pub fn init(&mut self, name: String, fee: u16, bumps: &InitializeBumps) -> Result<()> {
        require!(
            !name.is_empty() && name.len() < 32,
            MarketplaceError::NameToLong
        );

        // Set marketplace account data
        self.marketplace.set_inner(Marketplace {
            name,
            admin: self.admin.key(),
            bump: bumps.marketplace,
            treasury_bump: bumps.treasury,
            rewards_bump: bumps.rewards_mint,
            fee,
        });

        Ok(())
    }
}
