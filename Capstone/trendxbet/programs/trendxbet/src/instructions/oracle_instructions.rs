use anchor_lang::prelude::*;
use crate::state::*;
use crate::constants::*;
use crate::error::TrendXBetError;
use crate::utils::{ValidationUtils, TimeUtils};
use crate::events::*;

// Register Oracle
#[derive(Accounts)]
pub struct RegisterOracle<'info> {
    #[account(
        init,
        payer = authority,
        space = OracleState::LEN,
        seeds = [ORACLE_SEED, oracle_authority.key().as_ref(), match_id.key().as_ref()],
        bump
    )]
    pub oracle_state: Account<'info, OracleState>,
    
    #[account(
        seeds = [PLATFORM_SEED],
        bump = global_state.bump,
        has_one = admin @ TrendXBetError::Unauthorized
    )]
    pub global_state: Account<'info, GlobalState>,
    
    #[account(
        seeds = [MATCH_SEED, match_id.key().as_ref()],
        bump = match_state.bump
    )]
    pub match_state: Account<'info, MatchState>,
    
    /// CHECK: Oracle authority
    pub oracle_authority: UncheckedAccount<'info>,
    
    /// CHECK: Match identifier
    pub match_id: UncheckedAccount<'info>,
    
    pub admin: Signer<'info>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn register_oracle(ctx: Context<RegisterOracle>, oracle_authority: Pubkey) -> Result<()> {
    let oracle_state = &mut ctx.accounts.oracle_state;
    let current_time = TimeUtils::get_current_timestamp();
    
    // Initialize oracle state
    oracle_state.initialize(
        oracle_authority,
        ctx.accounts.match_id.key(),
        ctx.bumps.oracle_state,
    );
    
    // Emit event
    emit!(OracleRegistered {
        oracle_authority,
        match_id: ctx.accounts.match_id.key(),
        timestamp: current_time,
    });
    
    msg!("Oracle registered for match: {}", ctx.accounts.match_id.key());
    Ok(())
}

// Update Match Result
#[derive(Accounts)]
pub struct UpdateMatchResult<'info> {
    #[account(
        mut,
        seeds = [ORACLE_SEED, oracle_authority.key().as_ref(), match_id.key().as_ref()],
        bump = oracle_state.bump,
        constraint = oracle_state.is_authorized_oracle(&oracle_authority.key()) @ TrendXBetError::InvalidOracleAuthority
    )]
    pub oracle_state: Account<'info, OracleState>,
    
    #[account(
        mut,
        seeds = [MATCH_SEED, match_id.key().as_ref()],
        bump = match_state.bump,
        constraint = match_state.status == MatchStatus::Ended @ TrendXBetError::InvalidMatchStatus
    )]
    pub match_state: Account<'info, MatchState>,
    
    #[account(
        seeds = [PLATFORM_SEED],
        bump = global_state.bump,
        constraint = global_state.is_operational() @ TrendXBetError::PlatformPaused
    )]
    pub global_state: Account<'info, GlobalState>,
    
    /// CHECK: Oracle authority
    pub oracle_authority: UncheckedAccount<'info>,
    
    /// CHECK: Match identifier
    pub match_id: UncheckedAccount<'info>,
    
    pub authority: Signer<'info>,
}

pub fn update_match_result(
    ctx: Context<UpdateMatchResult>,
    winning_team: u8,
    final_score: String,
) -> Result<()> {
    let oracle_state = &mut ctx.accounts.oracle_state;
    let match_state = &mut ctx.accounts.match_state;
    let current_time = TimeUtils::get_current_timestamp();
    
    // Validate inputs
    ValidationUtils::validate_team_selection(winning_team)?;
    require!(final_score.len() <= MAX_SCORE_LENGTH, TrendXBetError::DescriptionTooLong);
    
    // Check if oracle update is within valid time window
    require!(
        oracle_state.is_update_valid(match_state.end_time, current_time),
        TrendXBetError::OracleUpdateExpired
    );
    
    // Submit or update result
    if oracle_state.reported_result.is_none() {
        oracle_state.submit_result(winning_team, final_score.clone(), current_time)?;
    } else {
        oracle_state.update_result(winning_team, final_score.clone(), current_time)?;
    }
    
    // If this oracle has minimum confirmations, update match result
    if oracle_state.has_minimum_confirmations() {
        match_state.set_result(winning_team, final_score.clone(), current_time);
    }
    
    // Emit event
    emit!(MatchResultUpdated {
        match_id: ctx.accounts.match_id.key(),
        oracle_authority: ctx.accounts.oracle_authority.key(),
        winning_team,
        final_score,
        timestamp: current_time,
    });
    
    msg!("Match result updated by oracle: team {} wins", winning_team);
    Ok(())
}

// Validate Oracle Update
#[derive(Accounts)]
pub struct ValidateOracleUpdate<'info> {
    #[account(
        mut,
        seeds = [ORACLE_SEED, oracle_authority.key().as_ref(), match_id.key().as_ref()],
        bump = oracle_state.bump
    )]
    pub oracle_state: Account<'info, OracleState>,
    
    #[account(
        mut,
        seeds = [MATCH_SEED, match_id.key().as_ref()],
        bump = match_state.bump
    )]
    pub match_state: Account<'info, MatchState>,
    
    #[account(
        seeds = [PLATFORM_SEED],
        bump = global_state.bump,
        has_one = admin @ TrendXBetError::Unauthorized
    )]
    pub global_state: Account<'info, GlobalState>,
    
    /// CHECK: Oracle authority
    pub oracle_authority: UncheckedAccount<'info>,
    
    /// CHECK: Match identifier
    pub match_id: UncheckedAccount<'info>,
    
    pub admin: Signer<'info>,
}

pub fn validate_oracle_update(ctx: Context<ValidateOracleUpdate>) -> Result<()> {
    let oracle_state = &mut ctx.accounts.oracle_state;
    let match_state = &mut ctx.accounts.match_state;
    let current_time = TimeUtils::get_current_timestamp();
    
    // Validate the oracle result
    oracle_state.validate(current_time)?;
    
    // If not already settled and oracle is ready, settle the match
    if !match_state.is_settled && oracle_state.is_ready() {
        if let (Some(winning_team), Some(final_score)) = (
            oracle_state.get_validated_result(),
            oracle_state.get_validated_score(),
        ) {
            match_state.set_result(winning_team, final_score, current_time);
            match_state.settle();
        }
    }
    
    // Emit event
    emit!(OracleUpdateValidated {
        match_id: ctx.accounts.match_id.key(),
        oracle_authority: ctx.accounts.oracle_authority.key(),
        confirmations: oracle_state.confirmations,
        is_final: oracle_state.is_ready(),
        timestamp: current_time,
    });
    
    // Check for consensus
    if oracle_state.confirmations >= MIN_ORACLE_CONFIRMATIONS {
        if let Some(winning_team) = oracle_state.get_validated_result() {
            emit!(ConsensusReached {
                match_id: ctx.accounts.match_id.key(),
                winning_team,
                total_confirmations: oracle_state.confirmations,
                timestamp: current_time,
            });
        }
    }
    
    msg!("Oracle update validated for match: {}", ctx.accounts.match_id.key());
    Ok(())
}
