use crate::constants::*;
use anchor_lang::prelude::*;

/// Bet status enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum BetStatus {
    /// Bet is active and waiting for match result
    Active,
    /// Bet won and payout available
    Won,
    /// Bet lost
    Lost,
    /// Bet was cancelled and refunded
    Cancelled,
    /// Bet payout has been claimed
    Claimed,
}

impl Default for BetStatus {
    fn default() -> Self {
        BetStatus::Active
    }
}

/// Individual bet state account
#[account]
pub struct BetState {
    /// Address of the bettor
    pub bettor: Pubkey,

    /// Match this bet is for
    pub match_id: Pubkey,

    /// Amount of the bet in lamports
    pub amount: u64,

    /// Predicted winning team (0 = team1, 1 = team2)
    pub predicted_team: u8,

    /// Odds at the time bet was placed (in basis points)
    pub odds_at_time: u64,

    /// Potential payout if bet wins
    pub potential_payout: u64,

    /// Current status of the bet
    pub status: BetStatus,

    /// Timestamp when bet was placed
    pub bet_placed_at: i64,

    /// Timestamp when bet was settled (optional)
    pub settled_at: Option<i64>,

    /// Actual payout amount (optional, set when settled)
    pub payout_amount: Option<u64>,

    /// Bump seed for PDA
    pub bump: u8,

    /// Reserved space for future fields
    pub reserved: [u8; 32],
}

impl BetState {
    pub const LEN: usize = BET_STATE_SPACE;

    /// Initialize new bet
    pub fn initialize(
        &mut self,
        bettor: Pubkey,
        match_id: Pubkey,
        amount: u64,
        predicted_team: u8,
        odds_at_time: u64,
        potential_payout: u64,
        bump: u8,
        current_time: i64,
    ) {
        self.bettor = bettor;
        self.match_id = match_id;
        self.amount = amount;
        self.predicted_team = predicted_team;
        self.odds_at_time = odds_at_time;
        self.potential_payout = potential_payout;
        self.status = BetStatus::Active;
        self.bet_placed_at = current_time;
        self.settled_at = None;
        self.payout_amount = None;
        self.bump = bump;
        self.reserved = [0; 32];
    }

    /// Cancel the bet and mark for refund
    pub fn cancel(&mut self, current_time: i64) {
        self.status = BetStatus::Cancelled;
        self.settled_at = Some(current_time);
        self.payout_amount = Some(self.amount); // Full refund
    }

    /// Settle bet as won
    pub fn settle_as_won(&mut self, payout_amount: u64, current_time: i64) {
        self.status = BetStatus::Won;
        self.settled_at = Some(current_time);
        self.payout_amount = Some(payout_amount);
    }

    /// Settle bet as lost
    pub fn settle_as_lost(&mut self, current_time: i64) {
        self.status = BetStatus::Lost;
        self.settled_at = Some(current_time);
        self.payout_amount = Some(0); // No payout for lost bets
    }

    /// Mark payout as claimed
    pub fn claim_payout(&mut self) -> Result<()> {
        if self.status != BetStatus::Won && self.status != BetStatus::Cancelled {
            return Err(error!(crate::error::TrendXBetError::BetNotFound));
        }

        self.status = BetStatus::Claimed;
        Ok(())
    }

    /// Check if bet can be cancelled
    pub fn can_be_cancelled(&self, match_start_time: i64, current_time: i64) -> bool {
        self.status == BetStatus::Active && current_time < match_start_time
    }

    /// Check if bet has winnings to claim
    pub fn has_claimable_winnings(&self) -> bool {
        matches!(self.status, BetStatus::Won | BetStatus::Cancelled)
            && self.payout_amount.unwrap_or(0) > 0
    }

    /// Get claimable amount
    pub fn get_claimable_amount(&self) -> u64 {
        if self.has_claimable_winnings() {
            self.payout_amount.unwrap_or(0)
        } else {
            0
        }
    }

    /// Check if bet won
    pub fn is_winning_bet(&self, winning_team: u8) -> bool {
        self.predicted_team == winning_team
    }

    /// Calculate actual payout using parimutuel system
    pub fn calculate_parimutuel_payout(
        &self,
        winning_pool: u64,
        total_pool: u64,
        house_edge: u16,
    ) -> Result<u64> {
        use crate::utils::MathUtils;

        MathUtils::calculate_parimutuel_payout(self.amount, winning_pool, total_pool, house_edge)
    }

    /// Get bet profit/loss
    pub fn get_profit_loss(&self) -> i64 {
        match self.status {
            BetStatus::Won | BetStatus::Claimed => {
                (self.payout_amount.unwrap_or(0) as i64) - (self.amount as i64)
            }
            BetStatus::Lost => -(self.amount as i64),
            BetStatus::Cancelled => 0, // Refunded
            BetStatus::Active => 0,    // Not yet determined
        }
    }

    /// Get effective odds based on actual payout
    pub fn get_effective_odds(&self) -> Option<u64> {
        if let Some(payout) = self.payout_amount {
            if payout > self.amount {
                let profit = payout - self.amount;
                Some((profit * crate::constants::BASIS_POINTS) / self.amount)
            } else {
                Some(0)
            }
        } else {
            None
        }
    }
}
