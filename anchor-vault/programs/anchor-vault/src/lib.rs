#![allow(unexpected_cfgs)]
#![allow(deprecated)]

use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

declare_id!("368FmB85aVd8mPxVhE89LrS8bDTjEfNGNiPyDBggXmFm");

#[program]
pub mod anchor_vault {
    use super::*;

    // Initializes the vault for a user.
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.initialize(&ctx.bumps)
    }

    // Deposits lamports into the vault.
    pub fn deposit(ctx: Context<Payment>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)
    }

    // Withdraws lamports from the vault.
    pub fn withdraw(ctx: Context<Payment>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)
    }

    // Closes the vault and returns funds to the user.
    pub fn close(ctx: Context<Close>) -> Result<()> {
        ctx.accounts.close()
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub user: Signer<'info>, // The user initializing the vault, must sign and be mutable.

    #[account(
        init,                                    
        payer = user,                            
        space = VaultState::INIT_SPACE,          
        seeds = [b"vault", user.key().as_ref()], 
        bump
    )]
    pub vault_state: Account<'info, VaultState>, 

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>, 

    pub system_program: Program<'info, System>, 
}

impl<'info> Initialize<'info> {
    // Handles vault initialization part of the program.
    pub fn initialize(&mut self, bumps: &InitializeBumps) -> Result<()> {
        let rent_exempt: u64 =
            Rent::get()?.minimum_balance(self.vault.to_account_info().data_len()); // Calculates rent-exempt minimum.

        let cpi_program: AccountInfo<'_> = self.system_program.to_account_info(); 

        let cpi_accounts = Transfer {
            from: self.user.to_account_info(),
            to: self.vault.to_account_info(), 
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer(cpi_ctx, rent_exempt)?;                                // Transfers rent-exempt lamports to vault.

        self.vault_state.vault_bump = bumps.vault; 
        self.vault_state.state_bump = bumps.vault_state; 

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Payment<'info> {
    pub user: Signer<'info>, // User making the payment.

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump
    )]
    pub vault: SystemAccount<'info>, 

    #[account(
        seeds = [b"state", user.key().as_ref()],
        bump = vault_state.state_bump
    )]
    pub vault_state: Account<'info, VaultState>, 

    pub system_program: Program<'info, System>, 
}

impl<'info> Payment<'info> {
    // Handles deposit part of the program.
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        let cpi_program: AccountInfo<'_> = self.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.user.to_account_info(), 
            to: self.vault.to_account_info(),  
        };

        let cpi_ctx: CpiContext<'_, '_, '_, '_, _> = CpiContext::new(cpi_program, cpi_accounts);

        transfer(cpi_ctx, amount) 
    }

    // Handles withdraw part of the program.
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        let cpi_program: AccountInfo<'_> = self.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(), 
            to: self.user.to_account_info(),    
        };

        let seeds: &[&[u8]; 3] = &[
            b"vault",
            self.vault_state.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump],
        ]; 

        let signer_seeds: &[&[&[u8]]; 1] = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        transfer(cpi_ctx, amount)
    }
}

#[derive(Accounts)]
pub struct Close<'info> {
    #[account(mut)]
    pub user: Signer<'info>, // User closing the vault.

    #[account(
        mut,
        seeds = [b"vault", user.key().as_ref()],
        bump = vault_state.vault_bump
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [b"state", user.key().as_ref()],
        bump = vault_state.state_bump,
        close = user                                // Closes account and sends rent to user.
    )]
    pub vault_state: Account<'info, VaultState>,

    pub system_program: Program<'info, System>,
}

impl Close<'_> {
    pub fn close(&mut self) -> Result<()> {
        let cpi_program: AccountInfo<'_> = self.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.user.to_account_info(),
        };

        let seeds: &[&[u8]; 3] = &[
            b"vault",
            self.vault_state.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump],
        ];

        let signer_seeds: &[&[&[u8]]; 1] = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        transfer(cpi_ctx, self.vault.lamports())
    }
}

#[account]
pub struct VaultState {
    pub vault_bump: u8,
    pub state_bump: u8,
}

impl Space for VaultState {
    const INIT_SPACE: usize = 8 + 1 * 2;
}
