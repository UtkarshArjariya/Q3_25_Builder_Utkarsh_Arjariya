use crate::constants::*;
use anchor_lang::prelude::*;

/// Oracle state account for managing match result reporting
#[account]
pub struct OracleState {
    /// Authority of the oracle provider
    pub oracle_authority: Pubkey,

    /// Match this oracle report is for
    pub match_id: Pubkey,

    /// Reported result (0 = team1 wins, 1 = team2 wins)
    pub reported_result: Option<u8>,

    /// Reported final score
    pub reported_score: Option<String>,

    /// Timestamp when result was reported
    pub report_time: Option<i64>,

    /// Whether this oracle report has been validated
    pub is_validated: bool,

    /// Timestamp when validation occurred
    pub validation_time: Option<i64>,

    /// Number of confirmations received
    pub confirmations: u8,

    /// Bump seed for PDA
    pub bump: u8,

    /// Reserved space for future fields
    pub reserved: [u8; 32],
}

impl OracleState {
    pub const LEN: usize = ORACLE_STATE_SPACE;

    /// Initialize oracle state
    pub fn initialize(&mut self, oracle_authority: Pubkey, match_id: Pubkey, bump: u8) {
        self.oracle_authority = oracle_authority;
        self.match_id = match_id;
        self.reported_result = None;
        self.reported_score = None;
        self.report_time = None;
        self.is_validated = false;
        self.validation_time = None;
        self.confirmations = 0;
        self.bump = bump;
        self.reserved = [0; 32];
    }

    /// Submit oracle result
    pub fn submit_result(
        &mut self,
        winning_team: u8,
        final_score: String,
        current_time: i64,
    ) -> Result<()> {
        // Check if result already exists
        if self.reported_result.is_some() {
            return Err(error!(crate::error::TrendXBetError::OracleResultExists));
        }

        self.reported_result = Some(winning_team);
        self.reported_score = Some(final_score);
        self.report_time = Some(current_time);
        self.confirmations = 1; // First confirmation

        Ok(())
    }

    /// Update oracle result (for corrections)
    pub fn update_result(
        &mut self,
        winning_team: u8,
        final_score: String,
        current_time: i64,
    ) -> Result<()> {
        // Only allow updates if not yet validated
        if self.is_validated {
            return Err(error!(crate::error::TrendXBetError::OracleResultExists));
        }

        self.reported_result = Some(winning_team);
        self.reported_score = Some(final_score);
        self.report_time = Some(current_time);

        Ok(())
    }

    /// Add confirmation from another oracle
    pub fn add_confirmation(&mut self) -> Result<()> {
        self.confirmations = self
            .confirmations
            .checked_add(1)
            .ok_or(error!(crate::error::TrendXBetError::MathematicalOverflow))?;
        Ok(())
    }

    /// Validate the oracle result
    pub fn validate(&mut self, current_time: i64) -> Result<()> {
        if self.confirmations < MIN_ORACLE_CONFIRMATIONS {
            return Err(error!(
                crate::error::TrendXBetError::InsufficientOracleConfirmations
            ));
        }

        self.is_validated = true;
        self.validation_time = Some(current_time);
        Ok(())
    }

    /// Check if oracle result is ready for use
    pub fn is_ready(&self) -> bool {
        self.is_validated && self.reported_result.is_some()
    }

    /// Check if oracle update is within valid time window
    pub fn is_update_valid(&self, match_end_time: i64, current_time: i64) -> bool {
        use crate::utils::TimeUtils;

        if let Some(report_time) = self.report_time {
            TimeUtils::is_oracle_update_valid(match_end_time, report_time)
        } else {
            // For new reports, check current time
            TimeUtils::is_oracle_update_valid(match_end_time, current_time)
        }
    }

    /// Get reported result with validation check
    pub fn get_validated_result(&self) -> Option<u8> {
        if self.is_ready() {
            self.reported_result
        } else {
            None
        }
    }

    /// Get reported score with validation check
    pub fn get_validated_score(&self) -> Option<String> {
        if self.is_ready() {
            self.reported_score.clone()
        } else {
            None
        }
    }

    /// Check if oracle has minimum confirmations
    pub fn has_minimum_confirmations(&self) -> bool {
        self.confirmations >= MIN_ORACLE_CONFIRMATIONS
    }

    /// Calculate confidence score based on confirmations
    pub fn confidence_score(&self) -> f64 {
        if self.confirmations == 0 {
            return 0.0;
        }

        // Simple confidence calculation: min(confirmations / min_required, 1.0)
        let score = (self.confirmations as f64) / (MIN_ORACLE_CONFIRMATIONS as f64);
        score.min(1.0)
    }

    /// Check if result matches another oracle's result
    pub fn matches_result(&self, other_result: u8) -> bool {
        if let Some(result) = self.reported_result {
            result == other_result
        } else {
            false
        }
    }

    /// Get time since report submission
    pub fn time_since_report(&self, current_time: i64) -> Option<i64> {
        self.report_time
            .map(|report_time| current_time - report_time)
    }

    /// Check if oracle authority is authorized for this report
    pub fn is_authorized_oracle(&self, authority: &Pubkey) -> bool {
        self.oracle_authority == *authority
    }
}
