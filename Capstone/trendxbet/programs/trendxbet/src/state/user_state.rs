use crate::constants::*;
use anchor_lang::prelude::*;

/// User profile state account
#[account]
pub struct UserState {
    /// User's wallet authority
    pub authority: Pubkey,

    /// Username chosen by user
    pub username: String,

    /// User's current balance in lamports
    pub balance: u64,

    /// Total number of bets placed by user
    pub total_bets_placed: u64,

    /// Total number of bets won by user
    pub total_bets_won: u64,

    /// Total volume of bets placed by user
    pub total_volume: u64,

    /// Total winnings earned by user
    pub total_winnings: u64,

    /// Timestamp when profile was created
    pub profile_created_at: i64,

    /// Bump seed for PDA
    pub bump: u8,

    /// Reserved space for future fields
    pub reserved: [u8; 32],
}

impl UserState {
    pub const LEN: usize = USER_STATE_SPACE;

    /// Initialize user profile
    pub fn initialize(&mut self, authority: Pubkey, username: String, bump: u8, current_time: i64) {
        self.authority = authority;
        self.username = username;
        self.balance = 0;
        self.total_bets_placed = 0;
        self.total_bets_won = 0;
        self.total_volume = 0;
        self.total_winnings = 0;
        self.profile_created_at = current_time;
        self.bump = bump;
        self.reserved = [0; 32];
    }

    /// Update username
    pub fn update_username(&mut self, new_username: String) {
        self.username = new_username;
    }

    /// Add funds to user balance
    pub fn add_balance(&mut self, amount: u64) -> Result<()> {
        self.balance = self
            .balance
            .checked_add(amount)
            .ok_or(error!(crate::error::TrendXBetError::MathematicalOverflow))?;
        Ok(())
    }

    /// Subtract funds from user balance
    pub fn subtract_balance(&mut self, amount: u64) -> Result<()> {
        if self.balance < amount {
            return Err(error!(crate::error::TrendXBetError::InsufficientBalance));
        }

        self.balance = self
            .balance
            .checked_sub(amount)
            .ok_or(error!(crate::error::TrendXBetError::MathematicalOverflow))?;
        Ok(())
    }

    /// Record a new bet placed
    pub fn record_bet_placed(&mut self, amount: u64) -> Result<()> {
        self.total_bets_placed = self
            .total_bets_placed
            .checked_add(1)
            .ok_or(error!(crate::error::TrendXBetError::MathematicalOverflow))?;

        self.total_volume = self
            .total_volume
            .checked_add(amount)
            .ok_or(error!(crate::error::TrendXBetError::MathematicalOverflow))?;

        Ok(())
    }

    /// Record a bet won
    pub fn record_bet_won(&mut self, winnings: u64) -> Result<()> {
        self.total_bets_won = self
            .total_bets_won
            .checked_add(1)
            .ok_or(error!(crate::error::TrendXBetError::MathematicalOverflow))?;

        self.total_winnings = self
            .total_winnings
            .checked_add(winnings)
            .ok_or(error!(crate::error::TrendXBetError::MathematicalOverflow))?;

        Ok(())
    }

    /// Calculate user's win rate
    pub fn win_rate(&self) -> f64 {
        if self.total_bets_placed == 0 {
            return 0.0;
        }
        (self.total_bets_won as f64) / (self.total_bets_placed as f64)
    }

    /// Calculate user's profit/loss
    pub fn net_profit(&self) -> i64 {
        (self.total_winnings as i64) - (self.total_volume as i64)
    }

    /// Check if user has sufficient balance
    pub fn has_sufficient_balance(&self, amount: u64) -> bool {
        self.balance >= amount
    }
}
