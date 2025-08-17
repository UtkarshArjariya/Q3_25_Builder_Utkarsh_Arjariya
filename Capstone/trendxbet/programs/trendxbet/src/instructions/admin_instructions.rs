use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
use crate::state::*;
use crate::constants::*;
use crate::error::TrendXBetError;
use crate::utils::{ValidationUtils, TimeUtils};
use crate::events::*;

// Update Platform Config
#[derive(Accounts)]
pub struct UpdatePlatformConfig<'info> {
    #[account(
        mut,
        seeds = [PLATFORM_SEED],
        bump = global_state.bump,
        has_one = admin @ TrendXBetError::Unauthorized
    )]
    pub global_state: Account<'info, GlobalState>,
    
    pub admin: Signer<'info>,
}

pub fn update_platform_config(
    ctx: Context<UpdatePlatformConfig>,
    house_edge: Option<u16>,
    min_bet_amount: Option<u64>,
    max_bet_amount: Option<u64>,
) -> Result<()> {
    let global_state = &mut ctx.accounts.global_state;
    let current_time = TimeUtils::get_current_timestamp();
    
    // Store old values for event
    let old_house_edge = if house_edge.is_some() { Some(global_state.house_edge) } else { None };
    let old_min_bet = if min_bet_amount.is_some() { Some(global_state.min_bet_amount) } else { None };
    let old_max_bet = if max_bet_amount.is_some() { Some(global_state.max_bet_amount) } else { None };
    
    // Validate new values
    if let Some(edge) = house_edge {
        ValidationUtils::validate_house_edge(edge)?;
    }
    
    if let Some(min_amount) = min_bet_amount {
        require!(min_amount > 0, TrendXBetError::BetAmountTooLow);
    }
    
    if let Some(max_amount) = max_bet_amount {
        let min_amount = min_bet_amount.unwrap_or(global_state.min_bet_amount);
        require!(max_amount > min_amount, TrendXBetError::InvalidTimeConfiguration);
    }
    
    // Update configuration
    global_state.update_config(house_edge, min_bet_amount, max_bet_amount);
    
    // Emit event
    emit!(PlatformConfigUpdated {
        admin: ctx.accounts.admin.key(),
        old_house_edge,
        new_house_edge: house_edge,
        old_min_bet,
        new_min_bet: min_bet_amount,
        old_max_bet,
        new_max_bet: max_bet_amount,
        timestamp: current_time,
    });
    
    msg!("Platform configuration updated by admin");
    Ok(())
}

// Withdraw Platform Fees
#[derive(Accounts)]
pub struct WithdrawPlatformFees<'info> {
    #[account(
        seeds = [PLATFORM_SEED],
        bump = global_state.bump,
        has_one = admin @ TrendXBetError::Unauthorized
    )]
    pub global_state: Account<'info, GlobalState>,
    
    #[account(
        mut,
        seeds = [TREASURY_SEED],
        bump = treasury.bump
    )]
    pub treasury: Account<'info, TreasuryState>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    /// CHECK: Treasury account to send funds from
    #[account(mut)]
    pub treasury_account: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn withdraw_platform_fees(ctx: Context<WithdrawPlatformFees>, amount: u64) -> Result<()> {
    let treasury = &mut ctx.accounts.treasury;
    let current_time = TimeUtils::get_current_timestamp();
    
    require!(amount > 0, TrendXBetError::BetAmountTooLow);
    require!(treasury.available_platform_fees() >= amount, TrendXBetError::TreasuryInsufficientFunds);
    
    // Transfer SOL from treasury to admin
    let treasury_seeds = &[TREASURY_SEED, &[treasury.bump]];
    let signer_seeds = &[&treasury_seeds[..]];
    
    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.system_program.to_account_info(),
        Transfer {
            from: ctx.accounts.treasury_account.to_account_info(),
            to: ctx.accounts.admin.to_account_info(),
        },
        signer_seeds,
    );
    transfer(transfer_ctx, amount)?;
    
    // Update treasury
    treasury.withdraw_platform_fees(amount)?;
    treasury.update_fee_collection_time(current_time);
    
    // Emit event
    emit!(PlatformFeesWithdrawn {
        admin: ctx.accounts.admin.key(),
        amount,
        remaining_fees: treasury.available_platform_fees(),
        timestamp: current_time,
    });
    
    emit!(TreasuryOperation {
        operation_type: "fee_withdrawal".to_string(),
        amount,
        new_balance: treasury.net_balance() as u64,
        authority: ctx.accounts.admin.key(),
        timestamp: current_time,
    });
    
    msg!("Platform fees withdrawn: {} lamports", amount);
    Ok(())
}

// Pause Platform
#[derive(Accounts)]
pub struct PausePlatform<'info> {
    #[account(
        mut,
        seeds = [PLATFORM_SEED],
        bump = global_state.bump,
        has_one = admin @ TrendXBetError::Unauthorized
    )]
    pub global_state: Account<'info, GlobalState>,
    
    pub admin: Signer<'info>,
}

pub fn pause_platform(ctx: Context<PausePlatform>) -> Result<()> {
    let global_state = &mut ctx.accounts.global_state;
    let current_time = TimeUtils::get_current_timestamp();
    
    require!(global_state.is_operational(), TrendXBetError::PlatformPaused);
    
    // Pause the platform
    global_state.pause();
    
    // Emit event
    emit!(PlatformPaused {
        admin: ctx.accounts.admin.key(),
        timestamp: current_time,
    });
    
    emit!(AdminAction {
        admin: ctx.accounts.admin.key(),
        action: "pause_platform".to_string(),
        target: None,
        timestamp: current_time,
    });
    
    msg!("Platform paused by admin");
    Ok(())
}

// Unpause Platform
#[derive(Accounts)]
pub struct UnpausePlatform<'info> {
    #[account(
        mut,
        seeds = [PLATFORM_SEED],
        bump = global_state.bump,
        has_one = admin @ TrendXBetError::Unauthorized
    )]
    pub global_state: Account<'info, GlobalState>,
    
    pub admin: Signer<'info>,
}

pub fn unpause_platform(ctx: Context<UnpausePlatform>) -> Result<()> {
    let global_state = &mut ctx.accounts.global_state;
    let current_time = TimeUtils::get_current_timestamp();
    
    require!(!global_state.is_operational(), TrendXBetError::AlreadyInitialized);
    
    // Unpause the platform
    global_state.unpause();
    
    // Emit event
    emit!(PlatformUnpaused {
        admin: ctx.accounts.admin.key(),
        timestamp: current_time,
    });
    
    emit!(AdminAction {
        admin: ctx.accounts.admin.key(),
        action: "unpause_platform".to_string(),
        target: None,
        timestamp: current_time,
    });
    
    msg!("Platform unpaused by admin");
    Ok(())
}

// Emergency Withdraw
#[derive(Accounts)]
pub struct EmergencyWithdraw<'info> {
    #[account(
        seeds = [PLATFORM_SEED],
        bump = global_state.bump,
        has_one = admin @ TrendXBetError::Unauthorized
    )]
    pub global_state: Account<'info, GlobalState>,
    
    #[account(
        mut,
        seeds = [TREASURY_SEED],
        bump = treasury.bump
    )]
    pub treasury: Account<'info, TreasuryState>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    /// CHECK: Treasury account to send funds from
    #[account(mut)]
    pub treasury_account: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn emergency_withdraw(ctx: Context<EmergencyWithdraw>, amount: u64) -> Result<()> {
    let treasury = &mut ctx.accounts.treasury;
    let current_time = TimeUtils::get_current_timestamp();
    
    require!(amount > 0, TrendXBetError::BetAmountTooLow);
    require!(treasury.has_sufficient_funds(amount), TrendXBetError::TreasuryInsufficientFunds);
    
    // This is an emergency function - should only be used in critical situations
    // Consider adding additional checks or multi-sig requirements
    
    // Transfer SOL from treasury to admin
    let treasury_seeds = &[TREASURY_SEED, &[treasury.bump]];
    let signer_seeds = &[&treasury_seeds[..]];
    
    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.system_program.to_account_info(),
        Transfer {
            from: ctx.accounts.treasury_account.to_account_info(),
            to: ctx.accounts.admin.to_account_info(),
        },
        signer_seeds,
    );
    transfer(transfer_ctx, amount)?;
    
    // Update treasury
    treasury.record_withdrawal(amount)?;
    
    // Emit events
    emit!(EmergencyWithdrawal {
        admin: ctx.accounts.admin.key(),
        amount,
        reason: "Emergency withdrawal executed".to_string(),
        timestamp: current_time,
    });
    
    emit!(AdminAction {
        admin: ctx.accounts.admin.key(),
        action: "emergency_withdrawal".to_string(),
        target: None,
        timestamp: current_time,
    });
    
    emit!(TreasuryOperation {
        operation_type: "emergency_withdrawal".to_string(),
        amount,
        new_balance: treasury.net_balance() as u64,
        authority: ctx.accounts.admin.key(),
        timestamp: current_time,
    });
    
    msg!("Emergency withdrawal executed: {} lamports", amount);
    Ok(())
}
