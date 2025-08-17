use anchor_lang::prelude::*;

#[error_code]
pub enum TrendXBetError {
    #[msg("Platform is currently paused")]
    PlatformPaused,

    #[msg("Unauthorized access")]
    Unauthorized,

    #[msg("Invalid bet amount - below minimum")]
    BetAmountTooLow,

    #[msg("Invalid bet amount - above maximum")]
    BetAmountTooHigh,

    #[msg("Insufficient user balance")]
    InsufficientBalance,

    #[msg("Invalid team selection")]
    InvalidTeam,

    #[msg("Betting is closed for this match")]
    BettingClosed,

    #[msg("Match has not started yet")]
    MatchNotStarted,

    #[msg("Match has already ended")]
    MatchEnded,

    #[msg("Match has already been settled")]
    MatchAlreadySettled,

    #[msg("Invalid match status")]
    InvalidMatchStatus,

    #[msg("Bet not found or already settled")]
    BetNotFound,

    #[msg("Cannot cancel bet - match has started")]
    CannotCancelBet,

    #[msg("Oracle update window expired")]
    OracleUpdateExpired,

    #[msg("Insufficient oracle confirmations")]
    InsufficientOracleConfirmations,

    #[msg("Oracle deviation too high")]
    OracleDeviationTooHigh,

    #[msg("Invalid oracle authority")]
    InvalidOracleAuthority,

    #[msg("Oracle result already submitted")]
    OracleResultExists,

    #[msg("Username too long")]
    UsernameTooLong,

    #[msg("Team name too long")]
    TeamNameTooLong,

    #[msg("Description too long")]
    DescriptionTooLong,

    #[msg("Invalid time configuration")]
    InvalidTimeConfiguration,

    #[msg("Start time must be in the future")]
    StartTimeInPast,

    #[msg("End time must be after start time")]
    EndTimeBeforeStart,

    #[msg("Treasury insufficient funds")]
    TreasuryInsufficientFunds,

    #[msg("Withdrawal amount exceeds daily limit")]
    WithdrawalLimitExceeded,

    #[msg("Invalid house edge - must be between 0 and 50%")]
    InvalidHouseEdge,

    #[msg("Mathematical overflow")]
    MathematicalOverflow,

    #[msg("Division by zero")]
    DivisionByZero,

    #[msg("Invalid account provided")]
    InvalidAccount,

    #[msg("Account not initialized")]
    AccountNotInitialized,

    #[msg("Already initialized")]
    AlreadyInitialized,

    #[msg("Invalid program address")]
    InvalidProgramAddress,

    #[msg("Token transfer failed")]
    TokenTransferFailed,

    #[msg("Invalid timestamp")]
    InvalidTimestamp,

    #[msg("User profile not found")]
    UserProfileNotFound,

    #[msg("Match not found")]
    MatchNotFound,

    #[msg("Payout calculation failed")]
    PayoutCalculationFailed,

    #[msg("Settlement period not yet reached")]
    SettlementPeriodNotReached,

    #[msg("Emergency withdrawal not authorized")]
    EmergencyWithdrawalNotAuthorized,
}
