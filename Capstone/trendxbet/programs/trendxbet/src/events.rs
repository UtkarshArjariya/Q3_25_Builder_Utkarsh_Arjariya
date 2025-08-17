use anchor_lang::prelude::*;

/// Event emitted when the platform is initialized
#[event]
pub struct PlatformInitialized {
    pub admin: Pubkey,
    pub house_edge: u16,
    pub min_bet_amount: u64,
    pub max_bet_amount: u64,
    pub timestamp: i64,
}

/// Event emitted when a user profile is created
#[event]
pub struct UserProfileCreated {
    pub user: Pubkey,
    pub username: String,
    pub timestamp: i64,
}

/// Event emitted when a user profile is updated
#[event]
pub struct UserProfileUpdated {
    pub user: Pubkey,
    pub new_username: Option<String>,
    pub timestamp: i64,
}

/// Event emitted when a user deposits funds
#[event]
pub struct FundsDeposited {
    pub user: Pubkey,
    pub amount: u64,
    pub new_balance: u64,
    pub timestamp: i64,
}

/// Event emitted when a user withdraws funds
#[event]
pub struct FundsWithdrawn {
    pub user: Pubkey,
    pub amount: u64,
    pub new_balance: u64,
    pub timestamp: i64,
}

/// Event emitted when a new match is created
#[event]
pub struct MatchCreated {
    pub match_id: Pubkey,
    pub team1: String,
    pub team2: String,
    pub start_time: i64,
    pub end_time: i64,
    pub description: String,
    pub creator: Pubkey,
    pub timestamp: i64,
}

/// Event emitted when match status is updated
#[event]
pub struct MatchStatusUpdated {
    pub match_id: Pubkey,
    pub old_status: u8,
    pub new_status: u8,
    pub timestamp: i64,
}

/// Event emitted when betting is closed for a match
#[event]
pub struct MatchBettingClosed {
    pub match_id: Pubkey,
    pub total_pool: u64,
    pub team1_pool: u64,
    pub team2_pool: u64,
    pub total_bets: u64,
    pub timestamp: i64,
}

/// Event emitted when a bet is placed
#[event]
pub struct BetPlaced {
    pub bet_id: Pubkey,
    pub bettor: Pubkey,
    pub match_id: Pubkey,
    pub amount: u64,
    pub predicted_team: u8,
    pub odds_at_time: u64,
    pub potential_payout: u64,
    pub timestamp: i64,
}

/// Event emitted when a bet is cancelled
#[event]
pub struct BetCancelled {
    pub bet_id: Pubkey,
    pub bettor: Pubkey,
    pub match_id: Pubkey,
    pub amount: u64,
    pub refund_amount: u64,
    pub timestamp: i64,
}

/// Event emitted when a bet is settled
#[event]
pub struct BetSettled {
    pub bet_id: Pubkey,
    pub bettor: Pubkey,
    pub match_id: Pubkey,
    pub amount: u64,
    pub won: bool,
    pub payout_amount: u64,
    pub timestamp: i64,
}

/// Event emitted when winnings are claimed
#[event]
pub struct WinningsClaimed {
    pub bettor: Pubkey,
    pub bet_id: Pubkey,
    pub match_id: Pubkey,
    pub payout_amount: u64,
    pub timestamp: i64,
}

/// Event emitted when an oracle is registered
#[event]
pub struct OracleRegistered {
    pub oracle_authority: Pubkey,
    pub match_id: Pubkey,
    pub timestamp: i64,
}

/// Event emitted when match result is updated by oracle
#[event]
pub struct MatchResultUpdated {
    pub match_id: Pubkey,
    pub oracle_authority: Pubkey,
    pub winning_team: u8,
    pub final_score: String,
    pub timestamp: i64,
}

/// Event emitted when oracle update is validated
#[event]
pub struct OracleUpdateValidated {
    pub match_id: Pubkey,
    pub oracle_authority: Pubkey,
    pub confirmations: u8,
    pub is_final: bool,
    pub timestamp: i64,
}

/// Event emitted when platform configuration is updated
#[event]
pub struct PlatformConfigUpdated {
    pub admin: Pubkey,
    pub old_house_edge: Option<u16>,
    pub new_house_edge: Option<u16>,
    pub old_min_bet: Option<u64>,
    pub new_min_bet: Option<u64>,
    pub old_max_bet: Option<u64>,
    pub new_max_bet: Option<u64>,
    pub timestamp: i64,
}

/// Event emitted when platform fees are withdrawn
#[event]
pub struct PlatformFeesWithdrawn {
    pub admin: Pubkey,
    pub amount: u64,
    pub remaining_fees: u64,
    pub timestamp: i64,
}

/// Event emitted when platform is paused
#[event]
pub struct PlatformPaused {
    pub admin: Pubkey,
    pub timestamp: i64,
}

/// Event emitted when platform is unpaused
#[event]
pub struct PlatformUnpaused {
    pub admin: Pubkey,
    pub timestamp: i64,
}

/// Event emitted when emergency withdrawal is performed
#[event]
pub struct EmergencyWithdrawal {
    pub admin: Pubkey,
    pub amount: u64,
    pub reason: String,
    pub timestamp: i64,
}

/// Event emitted for treasury operations
#[event]
pub struct TreasuryOperation {
    pub operation_type: String, // "deposit", "withdrawal", "fee_collection"
    pub amount: u64,
    pub new_balance: u64,
    pub authority: Pubkey,
    pub timestamp: i64,
}

/// Event emitted when match result consensus is reached
#[event]
pub struct ConsensusReached {
    pub match_id: Pubkey,
    pub winning_team: u8,
    pub total_confirmations: u8,
    pub timestamp: i64,
}

/// Event emitted for audit trail
#[event]
pub struct AdminAction {
    pub admin: Pubkey,
    pub action: String,
    pub target: Option<Pubkey>,
    pub timestamp: i64,
}
