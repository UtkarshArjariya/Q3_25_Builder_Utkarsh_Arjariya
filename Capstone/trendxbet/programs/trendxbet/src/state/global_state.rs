use crate::constants::*;
use anchor_lang::prelude::*;

/// Global platform state account
#[account]
pub struct GlobalState {
    /// Admin authority for the platform
    pub admin: Pubkey,

    /// House edge in basis points (500 = 5%)
    pub house_edge: u16,

    /// Minimum bet amount allowed
    pub min_bet_amount: u64,

    /// Maximum bet amount allowed
    pub max_bet_amount: u64,

    /// Total volume processed by platform
    pub total_volume: u64,

    /// Total fees collected by platform
    pub total_fees_collected: u64,

    /// Timestamp when platform was created
    pub platform_created_at: i64,

    /// Whether the platform is paused
    pub is_paused: bool,

    /// Bump seed for PDA
    pub bump: u8,
}

impl GlobalState {
    pub const LEN: usize = GLOBAL_STATE_SPACE;

    /// Initialize the global state
    pub fn initialize(
        &mut self,
        admin: Pubkey,
        house_edge: u16,
        min_bet_amount: u64,
        max_bet_amount: u64,
        bump: u8,
        current_time: i64,
    ) {
        self.admin = admin;
        self.house_edge = house_edge;
        self.min_bet_amount = min_bet_amount;
        self.max_bet_amount = max_bet_amount;
        self.total_volume = 0;
        self.total_fees_collected = 0;
        self.platform_created_at = current_time;
        self.is_paused = false;
        self.bump = bump;
    }

    /// Update platform configuration
    pub fn update_config(
        &mut self,
        house_edge: Option<u16>,
        min_bet_amount: Option<u64>,
        max_bet_amount: Option<u64>,
    ) {
        if let Some(edge) = house_edge {
            self.house_edge = edge;
        }
        if let Some(min_amount) = min_bet_amount {
            self.min_bet_amount = min_amount;
        }
        if let Some(max_amount) = max_bet_amount {
            self.max_bet_amount = max_amount;
        }
    }

    /// Add volume to total
    pub fn add_volume(&mut self, amount: u64) -> Result<()> {
        self.total_volume = self
            .total_volume
            .checked_add(amount)
            .ok_or(error!(crate::error::TrendXBetError::MathematicalOverflow))?;
        Ok(())
    }

    /// Add fees to total collected
    pub fn add_fees(&mut self, amount: u64) -> Result<()> {
        self.total_fees_collected = self
            .total_fees_collected
            .checked_add(amount)
            .ok_or(error!(crate::error::TrendXBetError::MathematicalOverflow))?;
        Ok(())
    }

    /// Pause the platform
    pub fn pause(&mut self) {
        self.is_paused = true;
    }

    /// Unpause the platform
    pub fn unpause(&mut self) {
        self.is_paused = false;
    }

    /// Check if platform is operational
    pub fn is_operational(&self) -> bool {
        !self.is_paused
    }
}
