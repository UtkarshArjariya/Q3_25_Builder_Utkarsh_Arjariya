use crate::constants::*;
use crate::error::TrendXBetError;
use crate::events::*;
use crate::state::*;
use crate::utils::{TimeUtils, ValidationUtils};
use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

// Create User Profile
#[derive(Accounts)]
#[instruction(username: String)]
pub struct CreateUserProfile<'info> {
    #[account(
        init,
        payer = user,
        space = UserState::LEN,
        seeds = [USER_SEED, user.key().as_ref()],
        bump
    )]
    pub user_state: Account<'info, UserState>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn create_user_profile(ctx: Context<CreateUserProfile>, username: String) -> Result<()> {
    let user_state = &mut ctx.accounts.user_state;
    let current_time = TimeUtils::get_current_timestamp();

    // Validate username
    ValidationUtils::validate_username(&username)?;

    // Initialize user state
    user_state.initialize(
        ctx.accounts.user.key(),
        username.clone(),
        ctx.bumps.user_state,
        current_time,
    );

    // Emit event
    emit!(UserProfileCreated {
        user: ctx.accounts.user.key(),
        username,
        timestamp: current_time,
    });

    msg!("User profile created for: {}", ctx.accounts.user.key());
    Ok(())
}

// Update User Profile
#[derive(Accounts)]
pub struct UpdateUserProfile<'info> {
    #[account(
        mut,
        seeds = [USER_SEED, user.key().as_ref()],
        bump = user_state.bump,
        has_one = authority @ TrendXBetError::Unauthorized
    )]
    pub user_state: Account<'info, UserState>,

    /// CHECK: This is the user's authority
    pub authority: Signer<'info>,

    /// CHECK: This is the user account that owns the profile
    pub user: UncheckedAccount<'info>,
}

pub fn update_user_profile(
    ctx: Context<UpdateUserProfile>,
    username: Option<String>,
) -> Result<()> {
    let user_state = &mut ctx.accounts.user_state;
    let current_time = TimeUtils::get_current_timestamp();

    if let Some(new_username) = &username {
        ValidationUtils::validate_username(new_username)?;
        user_state.update_username(new_username.clone());
    }

    // Emit event
    emit!(UserProfileUpdated {
        user: ctx.accounts.user.key(),
        new_username: username,
        timestamp: current_time,
    });

    msg!("User profile updated for: {}", ctx.accounts.user.key());
    Ok(())
}

// Deposit Funds
#[derive(Accounts)]
pub struct DepositFunds<'info> {
    #[account(
        mut,
        seeds = [USER_SEED, user.key().as_ref()],
        bump = user_state.bump,
        has_one = authority @ TrendXBetError::Unauthorized
    )]
    pub user_state: Account<'info, UserState>,

    #[account(
        mut,
        seeds = [TREASURY_SEED],
        bump = treasury.bump
    )]
    pub treasury: Account<'info, TreasuryState>,

    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: This is the user account
    pub user: UncheckedAccount<'info>,

    /// CHECK: Treasury account to receive funds
    #[account(mut)]
    pub treasury_account: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn deposit_funds(ctx: Context<DepositFunds>, amount: u64) -> Result<()> {
    let user_state = &mut ctx.accounts.user_state;
    let treasury = &mut ctx.accounts.treasury;
    let current_time = TimeUtils::get_current_timestamp();

    require!(amount > 0, TrendXBetError::BetAmountTooLow);

    // Transfer SOL from user to treasury
    let transfer_ctx = CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        Transfer {
            from: ctx.accounts.authority.to_account_info(),
            to: ctx.accounts.treasury_account.to_account_info(),
        },
    );
    transfer(transfer_ctx, amount)?;

    // Update user balance
    user_state.add_balance(amount)?;

    // Update treasury
    treasury.record_deposit(amount)?;

    // Emit event
    emit!(FundsDeposited {
        user: ctx.accounts.user.key(),
        amount,
        new_balance: user_state.balance,
        timestamp: current_time,
    });

    msg!(
        "User {} deposited {} lamports",
        ctx.accounts.user.key(),
        amount
    );
    Ok(())
}

// Withdraw Funds
#[derive(Accounts)]
pub struct WithdrawFunds<'info> {
    #[account(
        mut,
        seeds = [USER_SEED, user.key().as_ref()],
        bump = user_state.bump,
        has_one = authority @ TrendXBetError::Unauthorized
    )]
    pub user_state: Account<'info, UserState>,

    #[account(
        mut,
        seeds = [TREASURY_SEED],
        bump = treasury.bump
    )]
    pub treasury: Account<'info, TreasuryState>,

    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: This is the user account
    pub user: UncheckedAccount<'info>,

    /// CHECK: Treasury account to send funds from
    #[account(mut)]
    pub treasury_account: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn withdraw_funds(ctx: Context<WithdrawFunds>, amount: u64) -> Result<()> {
    let user_state = &mut ctx.accounts.user_state;
    let treasury = &mut ctx.accounts.treasury;
    let current_time = TimeUtils::get_current_timestamp();

    require!(amount > 0, TrendXBetError::BetAmountTooLow);
    require!(
        user_state.has_sufficient_balance(amount),
        TrendXBetError::InsufficientBalance
    );
    require!(
        treasury.has_sufficient_funds(amount),
        TrendXBetError::TreasuryInsufficientFunds
    );

    // Transfer SOL from treasury to user
    let treasury_seeds = &[TREASURY_SEED, &[treasury.bump]];
    let signer_seeds = &[&treasury_seeds[..]];

    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.system_program.to_account_info(),
        Transfer {
            from: ctx.accounts.treasury_account.to_account_info(),
            to: ctx.accounts.authority.to_account_info(),
        },
        signer_seeds,
    );
    transfer(transfer_ctx, amount)?;

    // Update user balance
    user_state.subtract_balance(amount)?;

    // Update treasury
    treasury.record_withdrawal(amount)?;

    // Emit event
    emit!(FundsWithdrawn {
        user: ctx.accounts.user.key(),
        amount,
        new_balance: user_state.balance,
        timestamp: current_time,
    });

    msg!(
        "User {} withdrew {} lamports",
        ctx.accounts.user.key(),
        amount
    );
    Ok(())
}
