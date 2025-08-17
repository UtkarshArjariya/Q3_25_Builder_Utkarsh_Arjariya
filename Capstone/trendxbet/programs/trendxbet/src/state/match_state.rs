use crate::constants::*;
use anchor_lang::prelude::*;

/// Match status enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum MatchStatus {
    /// Match is scheduled and betting is open
    Scheduled,
    /// Match is live and betting is closed
    Live,
    /// Match has ended, waiting for oracle result
    Ended,
    /// Match has been settled with final results
    Settled,
    /// Match was cancelled
    Cancelled,
}

impl Default for MatchStatus {
    fn default() -> Self {
        MatchStatus::Scheduled
    }
}

/// Match state account
#[account]
pub struct MatchState {
    /// Unique identifier for the match
    pub match_id: Pubkey,

    /// Name of team 1
    pub team1: String,

    /// Name of team 2
    pub team2: String,

    /// Match description
    pub description: String,

    /// Match start time (unix timestamp)
    pub start_time: i64,

    /// Match end time (unix timestamp)
    pub end_time: i64,

    /// Current status of the match
    pub status: MatchStatus,

    /// Total pool across both teams
    pub total_pool: u64,

    /// Pool for team 1 bets
    pub team1_pool: u64,

    /// Pool for team 2 bets
    pub team2_pool: u64,

    /// Total number of bets placed
    pub total_bets: u64,

    /// Winning team (0 = team1, 1 = team2, None if not determined)
    pub winning_team: Option<u8>,

    /// Final score string (optional)
    pub final_score: Option<String>,

    /// Timestamp when oracle provided result
    pub oracle_result_time: Option<i64>,

    /// Whether betting is closed for this match
    pub is_betting_closed: bool,

    /// Whether the match has been settled
    pub is_settled: bool,

    /// Timestamp when match was created
    pub match_created_at: i64,

    /// Bump seed for PDA
    pub bump: u8,

    /// Reserved space for future fields
    pub reserved: [u8; 64],
}

impl MatchState {
    pub const LEN: usize = MATCH_STATE_SPACE;

    /// Initialize new match
    pub fn initialize(
        &mut self,
        match_id: Pubkey,
        team1: String,
        team2: String,
        description: String,
        start_time: i64,
        end_time: i64,
        bump: u8,
        current_time: i64,
    ) {
        self.match_id = match_id;
        self.team1 = team1;
        self.team2 = team2;
        self.description = description;
        self.start_time = start_time;
        self.end_time = end_time;
        self.status = MatchStatus::Scheduled;
        self.total_pool = 0;
        self.team1_pool = 0;
        self.team2_pool = 0;
        self.total_bets = 0;
        self.winning_team = None;
        self.final_score = None;
        self.oracle_result_time = None;
        self.is_betting_closed = false;
        self.is_settled = false;
        self.match_created_at = current_time;
        self.bump = bump;
        self.reserved = [0; 64];
    }

    /// Update match status
    pub fn update_status(&mut self, new_status: MatchStatus) {
        self.status = new_status;
    }

    /// Add bet to the pools
    pub fn add_bet(&mut self, amount: u64, team: u8) -> Result<()> {
        // Add to total pool
        self.total_pool = self
            .total_pool
            .checked_add(amount)
            .ok_or(error!(crate::error::TrendXBetError::MathematicalOverflow))?;

        // Add to team-specific pool
        if team == 0 {
            self.team1_pool = self
                .team1_pool
                .checked_add(amount)
                .ok_or(error!(crate::error::TrendXBetError::MathematicalOverflow))?;
        } else {
            self.team2_pool = self
                .team2_pool
                .checked_add(amount)
                .ok_or(error!(crate::error::TrendXBetError::MathematicalOverflow))?;
        }

        // Increment bet count
        self.total_bets = self
            .total_bets
            .checked_add(1)
            .ok_or(error!(crate::error::TrendXBetError::MathematicalOverflow))?;

        Ok(())
    }

    /// Remove bet from the pools (for cancellations)
    pub fn remove_bet(&mut self, amount: u64, team: u8) -> Result<()> {
        // Remove from total pool
        self.total_pool = self
            .total_pool
            .checked_sub(amount)
            .ok_or(error!(crate::error::TrendXBetError::MathematicalOverflow))?;

        // Remove from team-specific pool
        if team == 0 {
            self.team1_pool = self
                .team1_pool
                .checked_sub(amount)
                .ok_or(error!(crate::error::TrendXBetError::MathematicalOverflow))?;
        } else {
            self.team2_pool = self
                .team2_pool
                .checked_sub(amount)
                .ok_or(error!(crate::error::TrendXBetError::MathematicalOverflow))?;
        }

        // Decrement bet count
        self.total_bets = self
            .total_bets
            .checked_sub(1)
            .ok_or(error!(crate::error::TrendXBetError::MathematicalOverflow))?;

        Ok(())
    }

    /// Close betting for this match
    pub fn close_betting(&mut self) {
        self.is_betting_closed = true;
    }

    /// Set match result
    pub fn set_result(&mut self, winning_team: u8, final_score: String, oracle_time: i64) {
        self.winning_team = Some(winning_team);
        self.final_score = Some(final_score);
        self.oracle_result_time = Some(oracle_time);
        self.status = MatchStatus::Ended;
    }

    /// Mark match as settled
    pub fn settle(&mut self) {
        self.is_settled = true;
        self.status = MatchStatus::Settled;
    }

    /// Cancel the match
    pub fn cancel(&mut self) {
        self.status = MatchStatus::Cancelled;
    }

    /// Check if betting is currently allowed
    pub fn is_betting_allowed(&self, current_time: i64) -> bool {
        !self.is_betting_closed
            && self.status == MatchStatus::Scheduled
            && current_time >= self.start_time
            && current_time <= self.end_time
    }

    /// Check if match has started
    pub fn has_started(&self, current_time: i64) -> bool {
        current_time >= self.start_time
    }

    /// Check if match has ended
    pub fn has_ended(&self, current_time: i64) -> bool {
        current_time >= self.end_time
    }

    /// Get odds for a team (in basis points)
    pub fn get_team_odds(&self, team: u8) -> u64 {
        use crate::utils::MathUtils;

        if self.total_pool == 0 {
            return crate::constants::BASIS_POINTS; // 1:1 odds
        }

        let team_pool = if team == 0 {
            self.team1_pool
        } else {
            self.team2_pool
        };

        MathUtils::calculate_odds(team_pool, self.total_pool)
            .unwrap_or(crate::constants::BASIS_POINTS)
    }

    /// Get the winning pool amount
    pub fn get_winning_pool(&self) -> Option<u64> {
        self.winning_team.map(|team| {
            if team == 0 {
                self.team1_pool
            } else {
                self.team2_pool
            }
        })
    }

    /// Get the losing pool amount
    pub fn get_losing_pool(&self) -> Option<u64> {
        self.winning_team.map(|team| {
            if team == 0 {
                self.team2_pool
            } else {
                self.team1_pool
            }
        })
    }
}
