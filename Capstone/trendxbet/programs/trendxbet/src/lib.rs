#![allow(ambiguous_glob_reexports)]
#![allow(unexpected_cfgs)]
#![allow(deprecated)]

pub mod constants;
pub mod error;
pub mod events;
pub mod instructions;
pub mod state;
pub mod utils;

use anchor_lang::prelude::*;

pub use constants::*;
pub use error::*;
pub use events::*;
pub use instructions::*;
pub use state::*;
pub use utils::*;

declare_id!("EBpoYA6LknxbYKJ2MM4PZsPqkGYqGKPrV74XxLXcWtBn");

#[program]
pub mod trendxbet {
    use super::*;

    // Platform Initialization
    pub fn initialize(ctx: Context<Initialize>, admin: Pubkey) -> Result<()> {
        instructions::initialize::handler(ctx, admin)
    }

    // User Management Instructions
    pub fn create_user_profile(ctx: Context<CreateUserProfile>, username: String) -> Result<()> {
        instructions::user_instructions::create_user_profile(ctx, username)
    }

    pub fn update_user_profile(
        ctx: Context<UpdateUserProfile>,
        username: Option<String>,
    ) -> Result<()> {
        instructions::user_instructions::update_user_profile(ctx, username)
    }

    pub fn deposit_funds(ctx: Context<DepositFunds>, amount: u64) -> Result<()> {
        instructions::user_instructions::deposit_funds(ctx, amount)
    }

    pub fn withdraw_funds(ctx: Context<WithdrawFunds>, amount: u64) -> Result<()> {
        instructions::user_instructions::withdraw_funds(ctx, amount)
    }

    // Match Management Instructions
    pub fn create_match(
        ctx: Context<CreateMatch>,
        team1: String,
        team2: String,
        start_time: i64,
        end_time: i64,
        description: String,
    ) -> Result<()> {
        instructions::match_instructions::create_match(
            ctx,
            team1,
            team2,
            start_time,
            end_time,
            description,
        )
    }

    pub fn update_match_status(ctx: Context<UpdateMatchStatus>, status: MatchStatus) -> Result<()> {
        instructions::match_instructions::update_match_status(ctx, status)
    }

    pub fn close_match_betting(ctx: Context<CloseMatchBetting>) -> Result<()> {
        instructions::match_instructions::close_match_betting(ctx)
    }

    // Betting Instructions
    pub fn place_bet(
        ctx: Context<PlaceBet>,
        amount: u64,
        predicted_team: u8,
        odds_accepted: u64,
    ) -> Result<()> {
        instructions::bet_instructions::place_bet(ctx, amount, predicted_team, odds_accepted)
    }

    pub fn cancel_bet(ctx: Context<CancelBet>) -> Result<()> {
        instructions::bet_instructions::cancel_bet(ctx)
    }

    pub fn settle_bet(ctx: Context<SettleBet>) -> Result<()> {
        instructions::bet_instructions::settle_bet(ctx)
    }

    pub fn claim_winnings(ctx: Context<ClaimWinnings>) -> Result<()> {
        instructions::bet_instructions::claim_winnings(ctx)
    }

    // Oracle Instructions
    pub fn register_oracle(ctx: Context<RegisterOracle>, oracle_authority: Pubkey) -> Result<()> {
        instructions::oracle_instructions::register_oracle(ctx, oracle_authority)
    }

    pub fn update_match_result(
        ctx: Context<UpdateMatchResult>,
        winning_team: u8,
        final_score: String,
    ) -> Result<()> {
        instructions::oracle_instructions::update_match_result(ctx, winning_team, final_score)
    }

    pub fn validate_oracle_update(ctx: Context<ValidateOracleUpdate>) -> Result<()> {
        instructions::oracle_instructions::validate_oracle_update(ctx)
    }

    // Admin Instructions
    pub fn update_platform_config(
        ctx: Context<UpdatePlatformConfig>,
        house_edge: Option<u16>,
        min_bet_amount: Option<u64>,
        max_bet_amount: Option<u64>,
    ) -> Result<()> {
        instructions::admin_instructions::update_platform_config(
            ctx,
            house_edge,
            min_bet_amount,
            max_bet_amount,
        )
    }

    pub fn withdraw_platform_fees(ctx: Context<WithdrawPlatformFees>, amount: u64) -> Result<()> {
        instructions::admin_instructions::withdraw_platform_fees(ctx, amount)
    }

    pub fn pause_platform(ctx: Context<PausePlatform>) -> Result<()> {
        instructions::admin_instructions::pause_platform(ctx)
    }

    pub fn unpause_platform(ctx: Context<UnpausePlatform>) -> Result<()> {
        instructions::admin_instructions::unpause_platform(ctx)
    }

    pub fn emergency_withdraw(ctx: Context<EmergencyWithdraw>, amount: u64) -> Result<()> {
        instructions::admin_instructions::emergency_withdraw(ctx, amount)
    }
}
