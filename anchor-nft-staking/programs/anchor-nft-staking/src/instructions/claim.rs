use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
};

use crate::state::{StakeConfig, UserAccount};

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,

    #[account(
        mut,
        seeds = [b"rewards".as_ref(), config.key().as_ref()],
        bump = config.rewards_bump,
    )]
    pub rewards_mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = rewards_mint,
        associated_token::authority = user,
    )]
    pub rewards_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"config".as_ref()],
        bump = config.bump,
    )]
    pub config: Account<'info, StakeConfig>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Claim<'info> {
    pub fn claim(&mut self) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();

        let seeds = &[b"rewards".as_ref(), &[self.config.bump]];

        let signer_seeds = &[&seeds[..]];

        let cpi_accounts = MintTo {
            authority: self.config.to_account_info(),
            mint: self.rewards_mint.to_account_info(),
            to: self.rewards_ata.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        let amount_to_mint =
            self.user_account.points as u64 * 10_u64.pow(self.rewards_mint.decimals as u32);

        mint_to(cpi_ctx, amount_to_mint);

        self.user_account.points = 0;

        Ok(())
    }
}
