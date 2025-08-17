use crate::constants::*;
use anchor_lang::prelude::*;

/// Treasury state account for managing platform funds
#[account]
pub struct TreasuryState {
    /// Authority that can manage treasury (admin)
    pub authority: Pubkey,

    /// Total deposits made to treasury
    pub total_deposits: u64,

    /// Total withdrawals from treasury
    pub total_withdrawals: u64,

    /// Accumulated platform fees
    pub platform_fees: u64,

    /// Amount pending for payouts to winners
    pub pending_payouts: u64,

    /// Timestamp of last fee collection
    pub last_fee_collection: i64,

    /// Bump seed for PDA
    pub bump: u8,

    /// Reserved space for future fields
    pub reserved: [u8; 64],
}

impl TreasuryState {
    pub const LEN: usize = TREASURY_STATE_SPACE;

    /// Initialize treasury
    pub fn initialize(&mut self, authority: Pubkey, bump: u8, current_time: i64) {
        self.authority = authority;
        self.total_deposits = 0;
        self.total_withdrawals = 0;
        self.platform_fees = 0;
        self.pending_payouts = 0;
        self.last_fee_collection = current_time;
        self.bump = bump;
        self.reserved = [0; 64];
    }

    /// Record a deposit to treasury
    pub fn record_deposit(&mut self, amount: u64) -> Result<()> {
        self.total_deposits = self
            .total_deposits
            .checked_add(amount)
            .ok_or(error!(crate::error::TrendXBetError::MathematicalOverflow))?;
        Ok(())
    }

    /// Record a withdrawal from treasury
    pub fn record_withdrawal(&mut self, amount: u64) -> Result<()> {
        self.total_withdrawals = self
            .total_withdrawals
            .checked_add(amount)
            .ok_or(error!(crate::error::TrendXBetError::MathematicalOverflow))?;
        Ok(())
    }

    /// Add platform fees
    pub fn add_platform_fees(&mut self, amount: u64) -> Result<()> {
        self.platform_fees = self
            .platform_fees
            .checked_add(amount)
            .ok_or(error!(crate::error::TrendXBetError::MathematicalOverflow))?;
        Ok(())
    }

    /// Withdraw platform fees
    pub fn withdraw_platform_fees(&mut self, amount: u64) -> Result<()> {
        if self.platform_fees < amount {
            return Err(error!(
                crate::error::TrendXBetError::TreasuryInsufficientFunds
            ));
        }

        self.platform_fees = self
            .platform_fees
            .checked_sub(amount)
            .ok_or(error!(crate::error::TrendXBetError::MathematicalOverflow))?;

        self.record_withdrawal(amount)?;
        Ok(())
    }

    /// Add to pending payouts
    pub fn add_pending_payout(&mut self, amount: u64) -> Result<()> {
        self.pending_payouts = self
            .pending_payouts
            .checked_add(amount)
            .ok_or(error!(crate::error::TrendXBetError::MathematicalOverflow))?;
        Ok(())
    }

    /// Remove from pending payouts (when paid out)
    pub fn remove_pending_payout(&mut self, amount: u64) -> Result<()> {
        if self.pending_payouts < amount {
            return Err(error!(crate::error::TrendXBetError::MathematicalOverflow));
        }

        self.pending_payouts = self
            .pending_payouts
            .checked_sub(amount)
            .ok_or(error!(crate::error::TrendXBetError::MathematicalOverflow))?;
        Ok(())
    }

    /// Update last fee collection timestamp
    pub fn update_fee_collection_time(&mut self, timestamp: i64) {
        self.last_fee_collection = timestamp;
    }

    /// Calculate net treasury balance
    pub fn net_balance(&self) -> i64 {
        (self.total_deposits as i64) - (self.total_withdrawals as i64)
    }

    /// Calculate available balance (excluding pending payouts)
    pub fn available_balance(&self) -> i64 {
        self.net_balance() - (self.pending_payouts as i64)
    }

    /// Check if treasury has sufficient funds for a withdrawal
    pub fn has_sufficient_funds(&self, amount: u64) -> bool {
        self.available_balance() >= (amount as i64)
    }

    /// Get total platform fees available for withdrawal
    pub fn available_platform_fees(&self) -> u64 {
        self.platform_fees
    }

    /// Calculate treasury utilization ratio (pending payouts / total deposits)
    pub fn utilization_ratio(&self) -> f64 {
        if self.total_deposits == 0 {
            return 0.0;
        }
        (self.pending_payouts as f64) / (self.total_deposits as f64)
    }

    /// Check if treasury is healthy (low utilization, positive balance)
    pub fn is_healthy(&self) -> bool {
        self.net_balance() > 0 && self.utilization_ratio() < 0.8 // Less than 80% utilization
    }

    /// Calculate total fees collected over time
    pub fn total_fees_lifetime(&self) -> u64 {
        // This would be platform_fees + already withdrawn fees
        // For now, we track current platform_fees
        self.platform_fees
    }

    /// Get days since last fee collection
    pub fn days_since_last_collection(&self, current_time: i64) -> u64 {
        let seconds_diff = current_time - self.last_fee_collection;
        if seconds_diff < 0 {
            0
        } else {
            (seconds_diff / 86_400) as u64 // Convert to days
        }
    }
}
