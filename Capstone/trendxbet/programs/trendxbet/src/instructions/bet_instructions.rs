use anchor_lang::prelude::*;
use crate::state::*;
use crate::constants::*;
use crate::error::TrendXBetError;
use crate::utils::{ValidationUtils, TimeUtils, MathUtils};
use crate::events::*;

// Place Bet
#[derive(Accounts)]
pub struct PlaceBet<'info> {
    #[account(
        init,
        payer = authority,
        space = BetState::LEN,
        seeds = [BET_SEED, bettor.key().as_ref(), match_id.key().as_ref()],
        bump
    )]
    pub bet_state: Account<'info, BetState>,
    
    #[account(
        mut,
        seeds = [USER_SEED, bettor.key().as_ref()],
        bump = user_state.bump,
        has_one = authority @ TrendXBetError::Unauthorized
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(
        mut,
        seeds = [MATCH_SEED, match_id.key().as_ref()],
        bump = match_state.bump
    )]
    pub match_state: Account<'info, MatchState>,
    
    #[account(
        seeds = [PLATFORM_SEED],
        bump = global_state.bump,
        constraint = global_state.is_operational() @ TrendXBetError::PlatformPaused
    )]
    pub global_state: Account<'info, GlobalState>,
    
    #[account(
        mut,
        seeds = [TREASURY_SEED],
        bump = treasury.bump
    )]
    pub treasury: Account<'info, TreasuryState>,
    
    /// CHECK: Match identifier
    pub match_id: UncheckedAccount<'info>,
    
    /// CHECK: Bettor account
    pub bettor: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn place_bet(
    ctx: Context<PlaceBet>,
    amount: u64,
    predicted_team: u8,
    odds_accepted: u64,
) -> Result<()> {
    let bet_state = &mut ctx.accounts.bet_state;
    let user_state = &mut ctx.accounts.user_state;
    let match_state = &mut ctx.accounts.match_state;
    let global_state = &ctx.accounts.global_state;
    let treasury = &mut ctx.accounts.treasury;
    let current_time = TimeUtils::get_current_timestamp();
    
    // Validate inputs
    ValidationUtils::validate_bet_amount(amount, global_state.min_bet_amount, global_state.max_bet_amount)?;
    ValidationUtils::validate_team_selection(predicted_team)?;
    
    // Check if betting is allowed
    require!(
        match_state.is_betting_allowed(current_time),
        TrendXBetError::BettingClosed
    );
    
    // Check user has sufficient balance
    require!(
        user_state.has_sufficient_balance(amount),
        TrendXBetError::InsufficientBalance
    );
    
    // Calculate current odds
    let current_odds = match_state.get_team_odds(predicted_team);
    
    // Check odds haven't moved too much (slippage protection)
    require!(
        current_odds >= odds_accepted,
        TrendXBetError::OracleDeviationTooHigh
    );
    
    // Calculate potential payout
    let potential_payout = MathUtils::calculate_payout(
        amount,
        current_odds,
        global_state.house_edge,
    )?;
    
    // Calculate platform fee
    let platform_fee = MathUtils::calculate_platform_fee(amount, global_state.house_edge)?;
    
    // Update user balance
    user_state.subtract_balance(amount)?;
    user_state.record_bet_placed(amount)?;
    
    // Update match pools
    match_state.add_bet(amount, predicted_team)?;
    
    // Update treasury and global state
    treasury.add_platform_fees(platform_fee)?;
    
    // Initialize bet state
    bet_state.initialize(
        ctx.accounts.bettor.key(),
        ctx.accounts.match_id.key(),
        amount,
        predicted_team,
        current_odds,
        potential_payout,
        ctx.bumps.bet_state,
        current_time,
    );
    
    // Emit event
    emit!(BetPlaced {
        bet_id: bet_state.key(),
        bettor: ctx.accounts.bettor.key(),
        match_id: ctx.accounts.match_id.key(),
        amount,
        predicted_team,
        odds_at_time: current_odds,
        potential_payout,
        timestamp: current_time,
    });
    
    msg!("Bet placed: {} lamports on team {}", amount, predicted_team);
    Ok(())
}

// Cancel Bet
#[derive(Accounts)]
pub struct CancelBet<'info> {
    #[account(
        mut,
        seeds = [BET_SEED, bettor.key().as_ref(), match_id.key().as_ref()],
        bump = bet_state.bump,
        has_one = bettor @ TrendXBetError::Unauthorized
    )]
    pub bet_state: Account<'info, BetState>,
    
    #[account(
        mut,
        seeds = [USER_SEED, bettor.key().as_ref()],
        bump = user_state.bump
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(
        mut,
        seeds = [MATCH_SEED, match_id.key().as_ref()],
        bump = match_state.bump
    )]
    pub match_state: Account<'info, MatchState>,
    
    /// CHECK: Match identifier
    pub match_id: UncheckedAccount<'info>,
    
    /// CHECK: Bettor account
    pub bettor: UncheckedAccount<'info>,
    
    pub authority: Signer<'info>,
}

pub fn cancel_bet(ctx: Context<CancelBet>) -> Result<()> {
    let bet_state = &mut ctx.accounts.bet_state;
    let user_state = &mut ctx.accounts.user_state;
    let match_state = &mut ctx.accounts.match_state;
    let current_time = TimeUtils::get_current_timestamp();
    
    // Check if bet can be cancelled
    require!(
        bet_state.can_be_cancelled(match_state.start_time, current_time),
        TrendXBetError::CannotCancelBet
    );
    
    let amount = bet_state.amount;
    let predicted_team = bet_state.predicted_team;
    
    // Cancel the bet
    bet_state.cancel(current_time);
    
    // Refund user
    user_state.add_balance(amount)?;
    
    // Remove from match pools
    match_state.remove_bet(amount, predicted_team)?;
    
    // Emit event
    emit!(BetCancelled {
        bet_id: bet_state.key(),
        bettor: ctx.accounts.bettor.key(),
        match_id: ctx.accounts.match_id.key(),
        amount,
        refund_amount: amount,
        timestamp: current_time,
    });
    
    msg!("Bet cancelled and refunded: {} lamports", amount);
    Ok(())
}

// Settle Bet
#[derive(Accounts)]
pub struct SettleBet<'info> {
    #[account(
        mut,
        seeds = [BET_SEED, bettor.key().as_ref(), match_id.key().as_ref()],
        bump = bet_state.bump
    )]
    pub bet_state: Account<'info, BetState>,
    
    #[account(
        mut,
        seeds = [USER_SEED, bettor.key().as_ref()],
        bump = user_state.bump
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(
        seeds = [MATCH_SEED, match_id.key().as_ref()],
        bump = match_state.bump,
        constraint = match_state.is_settled @ TrendXBetError::MatchNotStarted
    )]
    pub match_state: Account<'info, MatchState>,
    
    #[account(
        seeds = [PLATFORM_SEED],
        bump = global_state.bump
    )]
    pub global_state: Account<'info, GlobalState>,
    
    #[account(
        mut,
        seeds = [TREASURY_SEED],
        bump = treasury.bump
    )]
    pub treasury: Account<'info, TreasuryState>,
    
    /// CHECK: Match identifier
    pub match_id: UncheckedAccount<'info>,
    
    /// CHECK: Bettor account
    pub bettor: UncheckedAccount<'info>,
    
    pub authority: Signer<'info>,
}

pub fn settle_bet(ctx: Context<SettleBet>) -> Result<()> {
    let bet_state = &mut ctx.accounts.bet_state;
    let user_state = &mut ctx.accounts.user_state;
    let match_state = &ctx.accounts.match_state;
    let global_state = &ctx.accounts.global_state;
    let treasury = &mut ctx.accounts.treasury;
    let current_time = TimeUtils::get_current_timestamp();
    
    require!(bet_state.status == BetStatus::Active, TrendXBetError::BetNotFound);
    
    let winning_team = match_state.winning_team.ok_or(TrendXBetError::MatchNotStarted)?;
    let winning_pool = match_state.get_winning_pool().ok_or(TrendXBetError::PayoutCalculationFailed)?;
    
    let won = bet_state.is_winning_bet(winning_team);
    
    if won {
        // Calculate payout using parimutuel system
        let payout = bet_state.calculate_parimutuel_payout(
            winning_pool,
            match_state.total_pool,
            global_state.house_edge,
        )?;
        
        // Settle as won
        bet_state.settle_as_won(payout, current_time);
        
        // Update user stats
        user_state.record_bet_won(payout)?;
        
        // Add to pending payouts
        treasury.add_pending_payout(payout)?;
    } else {
        // Settle as lost
        bet_state.settle_as_lost(current_time);
    }
    
    // Emit event
    emit!(BetSettled {
        bet_id: bet_state.key(),
        bettor: ctx.accounts.bettor.key(),
        match_id: ctx.accounts.match_id.key(),
        amount: bet_state.amount,
        won,
        payout_amount: bet_state.payout_amount.unwrap_or(0),
        timestamp: current_time,
    });
    
    msg!("Bet settled: {} - payout: {}", won, bet_state.payout_amount.unwrap_or(0));
    Ok(())
}

// Claim Winnings
#[derive(Accounts)]
pub struct ClaimWinnings<'info> {
    #[account(
        mut,
        seeds = [BET_SEED, bettor.key().as_ref(), match_id.key().as_ref()],
        bump = bet_state.bump,
        has_one = bettor @ TrendXBetError::Unauthorized
    )]
    pub bet_state: Account<'info, BetState>,
    
    #[account(
        mut,
        seeds = [USER_SEED, bettor.key().as_ref()],
        bump = user_state.bump
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(
        mut,
        seeds = [TREASURY_SEED],
        bump = treasury.bump
    )]
    pub treasury: Account<'info, TreasuryState>,
    
    /// CHECK: Match identifier
    pub match_id: UncheckedAccount<'info>,
    
    /// CHECK: Bettor account
    pub bettor: UncheckedAccount<'info>,
    
    pub authority: Signer<'info>,
}

pub fn claim_winnings(ctx: Context<ClaimWinnings>) -> Result<()> {
    let bet_state = &mut ctx.accounts.bet_state;
    let user_state = &mut ctx.accounts.user_state;
    let treasury = &mut ctx.accounts.treasury;
    let current_time = TimeUtils::get_current_timestamp();
    
    require!(bet_state.has_claimable_winnings(), TrendXBetError::BetNotFound);
    
    let payout_amount = bet_state.get_claimable_amount();
    
    // Mark as claimed
    bet_state.claim_payout()?;
    
    // Add winnings to user balance
    user_state.add_balance(payout_amount)?;
    
    // Remove from pending payouts
    treasury.remove_pending_payout(payout_amount)?;
    
    // Emit event
    emit!(WinningsClaimed {
        bettor: ctx.accounts.bettor.key(),
        bet_id: bet_state.key(),
        match_id: ctx.accounts.match_id.key(),
        payout_amount,
        timestamp: current_time,
    });
    
    msg!("Winnings claimed: {} lamports", payout_amount);
    Ok(())
}
