use crate::constants::*;
use crate::error::TrendXBetError;
use anchor_lang::prelude::*;

pub struct MathUtils;

impl MathUtils {
    /// Calculate betting odds based on pool distribution
    /// Returns odds in basis points (10000 = 1:1 odds)
    pub fn calculate_odds(team_pool: u64, total_pool: u64) -> Result<u64> {
        if total_pool == 0 {
            return Ok(BASIS_POINTS); // 1:1 odds if no bets placed
        }

        if team_pool == 0 {
            return Ok(BASIS_POINTS * 100); // Very high odds if no bets on this team
        }

        // Odds = (total_pool - team_pool) / team_pool
        // Multiply by BASIS_POINTS for precision
        let opposing_pool = total_pool
            .checked_sub(team_pool)
            .ok_or(TrendXBetError::MathematicalOverflow)?;

        let odds = opposing_pool
            .checked_mul(BASIS_POINTS)
            .ok_or(TrendXBetError::MathematicalOverflow)?
            .checked_div(team_pool)
            .ok_or(TrendXBetError::DivisionByZero)?;

        Ok(odds)
    }

    /// Calculate potential payout for a bet
    pub fn calculate_payout(bet_amount: u64, odds: u64, house_edge: u16) -> Result<u64> {
        // Apply house edge first
        let house_edge_amount = bet_amount
            .checked_mul(house_edge as u64)
            .ok_or(TrendXBetError::MathematicalOverflow)?
            .checked_div(BASIS_POINTS)
            .ok_or(TrendXBetError::DivisionByZero)?;

        let effective_bet = bet_amount
            .checked_sub(house_edge_amount)
            .ok_or(TrendXBetError::MathematicalOverflow)?;

        // Calculate payout: bet_amount + (effective_bet * odds / BASIS_POINTS)
        let winnings = effective_bet
            .checked_mul(odds)
            .ok_or(TrendXBetError::MathematicalOverflow)?
            .checked_div(BASIS_POINTS)
            .ok_or(TrendXBetError::DivisionByZero)?;

        let total_payout = bet_amount
            .checked_add(winnings)
            .ok_or(TrendXBetError::MathematicalOverflow)?;

        Ok(total_payout)
    }

    /// Calculate platform fee from bet amount
    pub fn calculate_platform_fee(bet_amount: u64, house_edge: u16) -> Result<u64> {
        bet_amount
            .checked_mul(house_edge as u64)
            .ok_or(TrendXBetError::MathematicalOverflow)?
            .checked_div(BASIS_POINTS)
            .ok_or(TrendXBetError::DivisionByZero.into())
    }

    /// Calculate proportional payout for winning bets in parimutuel system
    pub fn calculate_parimutuel_payout(
        bet_amount: u64,
        winning_pool: u64,
        total_pool: u64,
        house_edge: u16,
    ) -> Result<u64> {
        if winning_pool == 0 {
            return Err(TrendXBetError::DivisionByZero.into());
        }

        // Calculate total fees
        let total_fees = Self::calculate_platform_fee(total_pool, house_edge)?;

        // Net pool after fees
        let net_pool = total_pool
            .checked_sub(total_fees)
            .ok_or(TrendXBetError::MathematicalOverflow)?;

        // Payout = (bet_amount / winning_pool) * net_pool
        let payout = bet_amount
            .checked_mul(net_pool)
            .ok_or(TrendXBetError::MathematicalOverflow)?
            .checked_div(winning_pool)
            .ok_or(TrendXBetError::DivisionByZero)?;

        Ok(payout)
    }
}

pub struct ValidationUtils;

impl ValidationUtils {
    /// Validate username format and length
    pub fn validate_username(username: &str) -> Result<()> {
        if username.len() > MAX_USERNAME_LENGTH {
            return Err(TrendXBetError::UsernameTooLong.into());
        }

        if username.trim().is_empty() {
            return Err(TrendXBetError::InvalidAccount.into());
        }

        // Check for valid characters (alphanumeric and underscore only)
        if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(TrendXBetError::InvalidAccount.into());
        }

        Ok(())
    }

    /// Validate team name format and length
    pub fn validate_team_name(team_name: &str) -> Result<()> {
        if team_name.len() > MAX_TEAM_NAME_LENGTH {
            return Err(TrendXBetError::TeamNameTooLong.into());
        }

        if team_name.trim().is_empty() {
            return Err(TrendXBetError::InvalidAccount.into());
        }

        Ok(())
    }

    /// Validate description format and length
    pub fn validate_description(description: &str) -> Result<()> {
        if description.len() > MAX_DESCRIPTION_LENGTH {
            return Err(TrendXBetError::DescriptionTooLong.into());
        }

        Ok(())
    }

    /// Validate bet amount
    pub fn validate_bet_amount(amount: u64, min_amount: u64, max_amount: u64) -> Result<()> {
        if amount < min_amount {
            return Err(TrendXBetError::BetAmountTooLow.into());
        }

        if amount > max_amount {
            return Err(TrendXBetError::BetAmountTooHigh.into());
        }

        Ok(())
    }

    /// Validate house edge (0-50%)
    pub fn validate_house_edge(house_edge: u16) -> Result<()> {
        if house_edge > 5000 {
            // 50% in basis points
            return Err(TrendXBetError::InvalidHouseEdge.into());
        }

        Ok(())
    }

    /// Validate time configuration for matches
    pub fn validate_match_times(start_time: i64, end_time: i64, current_time: i64) -> Result<()> {
        if start_time <= current_time {
            return Err(TrendXBetError::StartTimeInPast.into());
        }

        if end_time <= start_time {
            return Err(TrendXBetError::EndTimeBeforeStart.into());
        }

        let duration = end_time - start_time;
        if duration < MIN_BETTING_DURATION || duration > MAX_BETTING_DURATION {
            return Err(TrendXBetError::InvalidTimeConfiguration.into());
        }

        Ok(())
    }

    /// Validate team selection (0 = team1, 1 = team2)
    pub fn validate_team_selection(team: u8) -> Result<()> {
        if team > 1 {
            return Err(TrendXBetError::InvalidTeam.into());
        }

        Ok(())
    }
}

pub struct TimeUtils;

impl TimeUtils {
    /// Get current timestamp
    pub fn get_current_timestamp() -> i64 {
        Clock::get().unwrap().unix_timestamp
    }

    /// Check if current time is within betting window
    pub fn is_betting_open(start_time: i64, end_time: i64) -> bool {
        let current_time = Self::get_current_timestamp();
        current_time >= start_time && current_time <= end_time
    }

    /// Check if match has started
    pub fn has_match_started(start_time: i64) -> bool {
        let current_time = Self::get_current_timestamp();
        current_time >= start_time
    }

    /// Check if match has ended
    pub fn has_match_ended(end_time: i64) -> bool {
        let current_time = Self::get_current_timestamp();
        current_time >= end_time
    }

    /// Check if oracle update is within valid window
    pub fn is_oracle_update_valid(match_end_time: i64, update_time: i64) -> bool {
        update_time >= match_end_time && update_time <= (match_end_time + ORACLE_UPDATE_WINDOW)
    }

    /// Check if settlement window has passed
    pub fn is_settlement_period_reached(match_end_time: i64) -> bool {
        let current_time = Self::get_current_timestamp();
        current_time >= (match_end_time + SETTLEMENT_WINDOW)
    }
}

pub struct SecurityUtils;

impl SecurityUtils {
    /// Generate PDA for user account
    pub fn generate_user_pda(authority: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[USER_SEED, authority.as_ref()], program_id)
    }

    /// Generate PDA for match account
    pub fn generate_match_pda(match_id: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[MATCH_SEED, match_id.as_ref()], program_id)
    }

    /// Generate PDA for bet account
    pub fn generate_bet_pda(user: &Pubkey, match_id: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[BET_SEED, user.as_ref(), match_id.as_ref()], program_id)
    }

    /// Generate PDA for treasury account
    pub fn generate_treasury_pda(program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[TREASURY_SEED], program_id)
    }

    /// Generate PDA for oracle account
    pub fn generate_oracle_pda(
        oracle_authority: &Pubkey,
        match_id: &Pubkey,
        program_id: &Pubkey,
    ) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[ORACLE_SEED, oracle_authority.as_ref(), match_id.as_ref()],
            program_id,
        )
    }

    /// Generate PDA for global state account
    pub fn generate_platform_pda(program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[PLATFORM_SEED], program_id)
    }
}
