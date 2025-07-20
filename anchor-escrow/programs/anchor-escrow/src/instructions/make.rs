use anchor_lang::{accounts::program, prelude::*};

use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::Escrow;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Make<'info> {
    // The user initializing the escrow, must sign the transaction
    #[account(mut)]
    pub maker: Signer<'info>,

    // The mint for token A, using the specified token program
    #[account(
        mint::token_program = token_program,
    )]
    pub mint_a: InterfaceAccount<'info, Mint>,

    // The mint for token B, using the specified token program
    #[account(
        mint::token_program = token_program,
    )]
    pub mint_b: InterfaceAccount<'info, Mint>,

    // The maker's associated token account for mint A, must be mutable
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,

    // The escrow account, initialized with a PDA using a seed and bump
    #[account(
        init,
        payer = maker,
        seeds = [b"escrow", maker.key().as_ref(), seed.to_le_bytes().as_ref()],
        space = 8 + Escrow::INIT_SPACE,
        bump,
    )]
    pub escrow: Account<'info, Escrow>,

    // The vault token account, owned by the escrow, to hold deposited tokens
    #[account(
        init,
        payer = maker,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    // The token program interface (SPL Token 2022 or compatible)
    pub token_program: Interface<'info, TokenInterface>,
    // The system program (for account creation)
    pub system_program: Program<'info, System>,
    // The associated token program (for creating ATAs)
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Make<'info> {
    // Initialize the escrow account with provided parameters
    pub fn init_escrow(&mut self, seed: u64, recieve: u64, bumps: &MakeBumps) -> Result<()> {
        self.escrow.set_inner(Escrow {
            seed,                      // Store the seed used for PDA
            maker: self.maker.key(),   // Store the maker's public key
            mint_a: self.mint_a.key(), // Store mint A's public key
            mint_b: self.mint_b.key(), // Store mint B's public key
            recieve,                   // Amount to receive in the trade
            bump: bumps.escrow,        // Store the bump for PDA
        });

        Ok(())
    }

    // Deposit tokens from the maker's ATA to the vault
    pub fn deposit(&mut self, deposit: u64) -> Result<()> {
        // Prepare the accounts required for the transfer_checked CPI
        let transfer_accounts = TransferChecked {
            from: self.maker_ata_a.to_account_info(), // Source account (maker's ATA)
            mint: self.mint_a.to_account_info(),      // Mint account
            to: self.vault.to_account_info(),         // Destination account (vault)
            authority: self.maker.to_account_info(),  // Authority (maker)
        };

        // Create the CPI context for the token transfer
        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), transfer_accounts);

        // Perform the transfer_checked CPI to move tokens
        transfer_checked(cpi_ctx, deposit, self.mint_a.decimals)
    }
}
