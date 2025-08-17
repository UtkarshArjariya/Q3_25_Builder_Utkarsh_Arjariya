// Platform Constants
pub const PLATFORM_SEED: &[u8] = b"platform";
pub const USER_SEED: &[u8] = b"user";
pub const MATCH_SEED: &[u8] = b"match";
pub const BET_SEED: &[u8] = b"bet";
pub const TREASURY_SEED: &[u8] = b"treasury";
pub const ORACLE_SEED: &[u8] = b"oracle";

// Platform Configuration
pub const MAX_USERNAME_LENGTH: usize = 32;
pub const MAX_TEAM_NAME_LENGTH: usize = 64;
pub const MAX_DESCRIPTION_LENGTH: usize = 256;
pub const MAX_SCORE_LENGTH: usize = 32;

// Betting Configuration
pub const DEFAULT_HOUSE_EDGE: u16 = 500; // 5% (in basis points)
pub const MIN_BET_AMOUNT: u64 = 1_000_000; // 0.001 SOL
pub const MAX_BET_AMOUNT: u64 = 100_000_000_000; // 100 SOL
pub const BASIS_POINTS: u64 = 10_000;

// Oracle Configuration
pub const MIN_ORACLE_CONFIRMATIONS: u8 = 2;
pub const ORACLE_UPDATE_WINDOW: i64 = 3600; // 1 hour in seconds
pub const MAX_ORACLE_DEVIATION: u64 = 1000; // 10% in basis points

// Time Configuration
pub const MIN_BETTING_DURATION: i64 = 3600; // 1 hour minimum betting period
pub const MAX_BETTING_DURATION: i64 = 2_592_000; // 30 days maximum betting period
pub const SETTLEMENT_WINDOW: i64 = 86_400; // 24 hours settlement window

// Account Space Calculations
pub const GLOBAL_STATE_SPACE: usize = 8 + // discriminator
    32 + // admin
    2 + // house_edge
    8 + // min_bet_amount
    8 + // max_bet_amount
    8 + // total_volume
    8 + // total_fees_collected
    8 + // platform_created_at
    1 + // is_paused
    1; // bump

pub const USER_STATE_SPACE: usize = 8 + // discriminator
    32 + // authority
    4 + MAX_USERNAME_LENGTH + // username
    8 + // balance
    8 + // total_bets_placed
    8 + // total_bets_won
    8 + // total_volume
    8 + // total_winnings
    8 + // profile_created_at
    1 + // bump
    32; // reserved

pub const MATCH_STATE_SPACE: usize = 8 + // discriminator
    32 + // match_id (Pubkey)
    4 + MAX_TEAM_NAME_LENGTH + // team1
    4 + MAX_TEAM_NAME_LENGTH + // team2
    4 + MAX_DESCRIPTION_LENGTH + // description
    8 + // start_time
    8 + // end_time
    1 + // status
    8 + // total_pool
    8 + // team1_pool
    8 + // team2_pool
    8 + // total_bets
    1 + // winning_team (optional)
    4 + MAX_SCORE_LENGTH + // final_score (optional)
    8 + // oracle_result_time
    1 + // is_betting_closed
    1 + // is_settled
    8 + // match_created_at
    1 + // bump
    64; // reserved

pub const BET_STATE_SPACE: usize = 8 + // discriminator
    32 + // bettor
    32 + // match_id
    8 + // amount
    1 + // predicted_team
    8 + // odds_at_time
    8 + // potential_payout
    1 + // status
    8 + // bet_placed_at
    8 + // settled_at (optional)
    8 + // payout_amount (optional)
    1 + // bump
    32; // reserved

pub const TREASURY_STATE_SPACE: usize = 8 + // discriminator
    32 + // authority
    8 + // total_deposits
    8 + // total_withdrawals
    8 + // platform_fees
    8 + // pending_payouts
    8 + // last_fee_collection
    1 + // bump
    64; // reserved

pub const ORACLE_STATE_SPACE: usize = 8 + // discriminator
    32 + // oracle_authority
    32 + // match_id
    1 + // reported_result
    4 + MAX_SCORE_LENGTH + // reported_score
    8 + // report_time
    1 + // is_validated
    8 + // validation_time
    4 + // confirmations
    1 + // bump
    32; // reserved
