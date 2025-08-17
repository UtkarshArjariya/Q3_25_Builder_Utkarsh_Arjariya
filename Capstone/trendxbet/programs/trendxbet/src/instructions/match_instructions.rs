use anchor_lang::prelude::*;
use crate::state::*;
use crate::constants::*;
use crate::error::TrendXBetError;
use crate::utils::{ValidationUtils, TimeUtils};
use crate::events::*;

// Create Match
#[derive(Accounts)]
pub struct CreateMatch<'info> {
    #[account(
        init,
        payer = authority,
        space = MatchState::LEN,
        seeds = [MATCH_SEED, match_id.key().as_ref()],
        bump
    )]
    pub match_state: Account<'info, MatchState>,
    
    #[account(
        seeds = [PLATFORM_SEED],
        bump = global_state.bump,
        constraint = global_state.is_operational() @ TrendXBetError::PlatformPaused
    )]
    pub global_state: Account<'info, GlobalState>,
    
    /// CHECK: Unique identifier for the match
    pub match_id: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn create_match(
    ctx: Context<CreateMatch>,
    team1: String,
    team2: String,
    start_time: i64,
    end_time: i64,
    description: String,
) -> Result<()> {
    let match_state = &mut ctx.accounts.match_state;
    let current_time = TimeUtils::get_current_timestamp();
    
    // Validate inputs
    ValidationUtils::validate_team_name(&team1)?;
    ValidationUtils::validate_team_name(&team2)?;
    ValidationUtils::validate_description(&description)?;
    ValidationUtils::validate_match_times(start_time, end_time, current_time)?;
    
    // Ensure teams are different
    require!(team1 != team2, TrendXBetError::InvalidTeam);
    
    // Initialize match
    match_state.initialize(
        ctx.accounts.match_id.key(),
        team1.clone(),
        team2.clone(),
        description.clone(),
        start_time,
        end_time,
        ctx.bumps.match_state,
        current_time,
    );
    
    // Emit event
    emit!(MatchCreated {
        match_id: ctx.accounts.match_id.key(),
        team1,
        team2,
        start_time,
        end_time,
        description,
        creator: ctx.accounts.authority.key(),
        timestamp: current_time,
    });
    
    msg!("Match created: {} vs {}", match_state.team1, match_state.team2);
    Ok(())
}

// Update Match Status
#[derive(Accounts)]
pub struct UpdateMatchStatus<'info> {
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
    
    /// CHECK: Match identifier
    pub match_id: UncheckedAccount<'info>,
    
    pub admin: Signer<'info>,
}

pub fn update_match_status(ctx: Context<UpdateMatchStatus>, status: MatchStatus) -> Result<()> {
    let match_state = &mut ctx.accounts.match_state;
    let current_time = TimeUtils::get_current_timestamp();
    let old_status = match_state.status.clone();
    
    // Validate status transition
    match (&old_status, &status) {
        (MatchStatus::Scheduled, MatchStatus::Live) => {
            require!(TimeUtils::has_match_started(match_state.start_time), TrendXBetError::MatchNotStarted);
        },
        (MatchStatus::Live, MatchStatus::Ended) => {
            require!(TimeUtils::has_match_ended(match_state.end_time), TrendXBetError::MatchNotStarted);
        },
        (MatchStatus::Ended, MatchStatus::Settled) => {
            require!(match_state.winning_team.is_some(), TrendXBetError::OracleResultExists);
        },
        (_, MatchStatus::Cancelled) => {
            // Admin can cancel at any time
        },
        _ => return Err(error!(TrendXBetError::InvalidMatchStatus)),
    }
    
    // Update status
    match_state.update_status(status.clone());
    
    // If transitioning to Live, close betting
    if status == MatchStatus::Live {
        match_state.close_betting();
    }
    
    // Emit event
    emit!(MatchStatusUpdated {
        match_id: ctx.accounts.match_id.key(),
        old_status: match old_status {
            MatchStatus::Scheduled => 0,
            MatchStatus::Live => 1,
            MatchStatus::Ended => 2,
            MatchStatus::Settled => 3,
            MatchStatus::Cancelled => 4,
        },
        new_status: match status {
            MatchStatus::Scheduled => 0,
            MatchStatus::Live => 1,
            MatchStatus::Ended => 2,
            MatchStatus::Settled => 3,
            MatchStatus::Cancelled => 4,
        },
        timestamp: current_time,
    });
    
    msg!("Match status updated for: {}", ctx.accounts.match_id.key());
    Ok(())
}

// Close Match Betting
#[derive(Accounts)]
pub struct CloseMatchBetting<'info> {
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
    
    /// CHECK: Match identifier
    pub match_id: UncheckedAccount<'info>,
    
    pub admin: Signer<'info>,
}

pub fn close_match_betting(ctx: Context<CloseMatchBetting>) -> Result<()> {
    let match_state = &mut ctx.accounts.match_state;
    let current_time = TimeUtils::get_current_timestamp();
    
    require!(
        match_state.status == MatchStatus::Scheduled,
        TrendXBetError::InvalidMatchStatus
    );
    
    // Close betting
    match_state.close_betting();
    
    // Emit event
    emit!(MatchBettingClosed {
        match_id: ctx.accounts.match_id.key(),
        total_pool: match_state.total_pool,
        team1_pool: match_state.team1_pool,
        team2_pool: match_state.team2_pool,
        total_bets: match_state.total_bets,
        timestamp: current_time,
    });
    
    msg!("Betting closed for match: {}", ctx.accounts.match_id.key());
    Ok(())
}
