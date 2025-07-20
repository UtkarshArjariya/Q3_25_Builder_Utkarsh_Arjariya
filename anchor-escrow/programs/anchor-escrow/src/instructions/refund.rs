use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },
};

use crate::Escrow;

#[derive(Accounts)]
pub struct Refund<'info> {
    // The user initializing the escrow, must sign the transaction
    #[account(mut)]
    pub maker: Signer<'info>,

    // The mint for token A, using the specified token program
    #[account(
        mint::token_program = token_program,
    )]
    pub mint_a: InterfaceAccount<'info, Mint>,

    // The maker's associated token account for mint A, must be mutable
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,

    // The escrow account, must be mutable and will be closed to the maker after refund.
    // Ensures it is associated with the correct mint and maker, and is derived from PDA seeds.
    #[account(
        mut,
        close = maker,
        has_one = mint_a,
        has_one = maker,
        seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, Escrow>,

    // The vault token account holding the deposited tokens, owned by the escrow PDA.
    #[account(
        mut,
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

impl<'info> Refund<'info> {
    /// Refunds the tokens from the escrow vault back to the maker's associated token account and closes the vault.
    pub fn refund_and_close_vault(&mut self) -> Result<()> {
        // We will be passing the seeds here
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"escrow",
            self.maker.to_account_info().key.as_ref(),
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump],
        ]];

        // Close the vault account and transfer its balance to the maker's ATA
        let transfer_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.maker_ata_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        // Perform the transfer using the CPI context with the signer seeds
        let transfer_cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            transfer_accounts,
            &signer_seeds,
        );

        // Execute the transfer_checked CPI to refund tokens
        transfer_checked(transfer_cpi_ctx, self.vault.amount, self.mint_a.decimals)?;

        // Close the vault account and transfer ownership to the maker
        let close_accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        // Create the CPI context for closing the vault account with signer seeds
        let close_cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            close_accounts,
            &signer_seeds,
        );

        // Execute the close_account CPI to close the vault
        close_account(close_cpi_ctx)?;

        Ok(())
    }
}
