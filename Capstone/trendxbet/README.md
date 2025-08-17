# TrendXBet - Decentralized Cricket Betting Platform

TrendXBet is a sophisticated decentralized betting platform built on Solana using the Anchor framework, specifically designed for cricket match betting with enterprise-grade security, oracle integration, and comprehensive treasury management.

## 🏏 Platform Overview

TrendXBet enables users to place bets on cricket matches using a secure, transparent, and decentralized system. The platform implements a parimutuel betting model with oracle-based result validation and comprehensive risk management.

### Key Features

- **Live Betting System**: Unique live betting during match execution (not traditional pre-match betting)
- **Decentralized Architecture**: Place bets on cricket matches with transparent odds calculation
- **Oracle Integration**: Multi-oracle validation system for accurate match results
- **Treasury Management**: Secure fund management with admin controls and emergency procedures
- **User Profiles**: Comprehensive user management with betting history and statistics
- **Admin Dashboard**: Platform configuration, fee management, and emergency controls
- **Parimutuel System**: Fair odds calculation based on pool distribution
- **Security First**: Multiple layers of security and validation
- **Clean Compilation**: Warning-free build with optimized performance

### Betting Model

**🚨 Important**: TrendXBet implements **live betting during matches**, not traditional pre-match betting:

- **Match Creation**: Matches are created with future start times
- **Betting Window**: Betting is allowed after match starts but before status changes to "Live"
- **Live Betting Period**: Users can place bets during the early phase of ongoing matches
- **Betting Closure**: Betting automatically closes when match status transitions to "Live"

This design enables dynamic, real-time betting experiences during cricket matches.

## 🏗️ Architecture

### Program Structure

```
trendxbet/
├── programs/trendxbet/src/
│   ├── lib.rs                 # Main program entry point
│   ├── constants.rs           # Platform constants and configurations
│   ├── error.rs              # Custom error definitions
│   ├── utils.rs              # Utility functions for calculations and validation
│   ├── events.rs             # Event definitions for blockchain logging
│   ├── instructions/          # Instruction handlers
│   │   ├── initialize.rs      # Platform initialization
│   │   ├── user_instructions.rs    # User profile management
│   │   ├── match_instructions.rs   # Match creation and management
│   │   ├── bet_instructions.rs     # Betting functionality
│   │   ├── oracle_instructions.rs  # Oracle management and result updates
│   │   └── admin_instructions.rs   # Administrative functions
│   └── state/                # On-chain state definitions
│       ├── global_state.rs    # Platform-wide configuration
│       ├── user_state.rs      # User profiles and balances
│       ├── match_state.rs     # Match details and pools
│       ├── bet_state.rs       # Individual bet records
│       ├── treasury_state.rs  # Treasury and fee management
│       └── oracle_state.rs    # Oracle state and validations
```

### State Management

#### Global State

- Platform configuration (house edge, bet limits)
- Admin authority and pause controls
- Platform-wide statistics and metrics

#### User State

- User profiles with usernames and authorities
- Balance management and betting history
- Win/loss statistics and total volume

#### Match State

- Cricket match details (teams, timing, description)
- Betting pools (team1_pool, team2_pool, total_pool)
- Match status and settlement information

#### Bet State

- Individual bet records with amounts and predictions
- Odds at placement time and potential payouts
- Settlement status and actual payouts

#### Treasury State

- Platform fund management and fee collection
- Deposit/withdrawal tracking
- Pending payout management

#### Oracle State

- Oracle authority and match associations
- Result submissions and validation status
- Consensus tracking and confirmation counts

## 🚀 Getting Started

### Prerequisites

- Rust 1.70+
- Solana CLI 1.16+
- Anchor CLI 0.28+
- Node.js 16+
- yarn

### Installation

1. **Clone the repository:**

   ```bash
   git clone <repository-url>
   cd trendxbet
   ```

2. **Install dependencies:**

   ```bash
   # Install Rust dependencies
   cargo build

   # Install Node.js dependencies
   yarn install
   ```

3. **Build the program:**

   ```bash
   anchor build
   ```

   The build is optimized for clean compilation with:

   - ✅ Zero Rust warnings
   - ✅ Zero Anchor warnings
   - ✅ Optimized release builds
   - ✅ Size-optimized compilation

4. **Deploy to localnet:**

   ```bash
   # Start local validator
   solana-test-validator

   # Deploy the program
   anchor deploy
   ```

5. **Run tests:**
   ```bash
   anchor test
   ```

## 📖 Usage Guide

### Platform Initialization

```typescript
// Initialize the platform with admin authority
await program.methods
  .initialize(adminPublicKey)
  .accounts({
    globalState: globalStatePda,
    treasury: treasuryPda,
    admin: adminPublicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([admin])
  .rpc();
```

### User Management

```typescript
// Create user profile
await program.methods
  .createUserProfile("username")
  .accounts({
    userState: userStatePda,
    user: userPublicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([user])
  .rpc();

// Deposit funds
await program.methods
  .depositFunds(new anchor.BN(amount))
  .accounts({
    userState: userStatePda,
    treasury: treasuryPda,
    authority: userPublicKey,
    user: userPublicKey,
    treasuryAccount: treasuryPda,
    systemProgram: SystemProgram.programId,
  })
  .signers([user])
  .rpc();
```

### Match Creation

```typescript
// Create cricket match (starts in future for live betting)
const startTime = Math.floor(Date.now() / 1000) + 300; // 5 minutes from now
const endTime = startTime + 10800; // 3 hours duration

await program.methods
  .createMatch(team1, team2, startTime, endTime, description)
  .accounts({
    matchState: matchStatePda,
    globalState: globalStatePda,
    matchId: matchIdPublicKey,
    authority: adminPublicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([admin])
  .rpc();
```

### Live Betting

```typescript
// Note: Betting only allowed after match starts but before status changes to "Live"
// Wait for match start time before placing bets

// Place bet during live betting window
await program.methods
  .placeBet(amount, predictedTeam, oddsAccepted)
  .accountsPartial({
    betState: betStatePda,
    userState: userStatePda,
    matchState: matchStatePda,
    globalState: globalStatePda,
    treasury: treasuryPda,
    matchId: matchIdPublicKey,
    bettor: userPublicKey,
    authority: userPublicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([user])
  .rpc();
```

### Oracle Operations

```typescript
// Register oracle for match
await program.methods
  .registerOracle(oracleAuthorityPublicKey)
  .accounts({
    oracleState: oracleStatePda,
    globalState: globalStatePda,
    matchState: matchStatePda,
    oracleAuthority: oracleAuthorityPublicKey,
    matchId: matchIdPublicKey,
    admin: adminPublicKey,
    authority: adminPublicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([admin])
  .rpc();

// Update match result
await program.methods
  .updateMatchResult(winningTeam, finalScore)
  .accounts({
    oracleState: oracleStatePda,
    matchState: matchStatePda,
    globalState: globalStatePda,
    oracleAuthority: oracleAuthorityPublicKey,
    matchId: matchIdPublicKey,
    authority: oraclePublicKey,
  })
  .signers([oracle])
  .rpc();
```

## 🔒 Security Features

### Multi-Layer Security

1. **Access Control**: Role-based permissions for admin, users, and oracles
2. **Input Validation**: Comprehensive validation of all user inputs
3. **Oracle Validation**: Multi-oracle consensus for match results
4. **Fund Protection**: Secure treasury management with withdrawal limits
5. **Emergency Controls**: Admin pause functionality and emergency withdrawals

### Oracle Security

- **Multiple Oracle Support**: Require consensus from multiple oracle sources
- **Time-Window Validation**: Results must be submitted within valid timeframes
- **Deviation Checks**: Validate oracle consistency and detect manipulation
- **Authority Verification**: Ensure only authorized oracles can submit results

### Treasury Security

- **Segregated Funds**: Separate user funds from platform fees
- **Withdrawal Limits**: Configurable limits on fund movements
- **Multi-Signature Ready**: Architecture supports multi-sig integration
- **Audit Trail**: Complete transaction history and event logging

## 📊 Economics Model

### Fee Structure

- **House Edge**: Configurable percentage (default 5%)
- **Platform Fees**: Collected from each bet placement
- **Fair Distribution**: Parimutuel system ensures fair odds

### Betting Mechanics

1. **Pool Formation**: Bets accumulate in team-specific pools
2. **Odds Calculation**: Dynamic odds based on pool distribution
3. **Fee Collection**: Platform fees deducted from betting pools
4. **Payout Distribution**: Winners share the losing pool proportionally

### Risk Management

- **Minimum/Maximum Bets**: Configurable bet size limits
- **Pool Balancing**: Automatic odds adjustment based on pool sizes
- **Treasury Reserves**: Maintain sufficient reserves for payouts

## 🧪 Testing

The platform includes comprehensive tests covering:

- ✅ Platform initialization and configuration
- ✅ User profile management and fund operations
- ✅ Match creation and live betting functionality
- ✅ Oracle registration and result submission
- ✅ Administrative functions and security controls
- ✅ Error handling and edge cases
- ✅ Live betting window timing validation

**Test Status**: All 11 tests passing with clean compilation

### Test Features

- **Live Betting Simulation**: Tests include proper timing delays to simulate live betting windows
- **Multiple Match Scenarios**: Separate test matches for different functionality testing
- **Comprehensive Coverage**: Full end-to-end testing of all platform features
- **Error Validation**: Thorough testing of error conditions and edge cases
- Error handling and edge cases

Run the test suite:

```bash
anchor test
```

## 🚀 Deployment

### Mainnet Deployment

1. **Update Anchor.toml:**

   ```toml
   [provider]
   cluster = "mainnet"
   wallet = "~/.config/solana/id.json"
   ```

2. **Deploy to mainnet:**

   ```bash
   anchor deploy --provider.cluster mainnet
   ```

3. **Verify deployment:**
   ```bash
   solana program show <PROGRAM_ID>
   ```

### Environment Configuration

- **Development**: Use localnet for testing
- **Staging**: Deploy to devnet for integration testing
- **Production**: Deploy to mainnet with proper security audits

## 📚 API Reference

### Instructions

#### Platform Management

- `initialize(admin: Pubkey)` - Initialize the platform
- `update_platform_config()` - Update platform settings
- `pause_platform()` / `unpause_platform()` - Emergency controls

#### User Operations

- `create_user_profile(username: String)` - Create user account
- `update_user_profile()` - Update user information
- `deposit_funds(amount: u64)` - Deposit SOL to platform
- `withdraw_funds(amount: u64)` - Withdraw SOL from platform

#### Match Management

- `create_match()` - Create new cricket match
- `update_match_status()` - Update match status
- `close_match_betting()` - Close betting for match

#### Betting Operations

- `place_bet()` - Place bet on match outcome
- `cancel_bet()` - Cancel active bet (before match starts)
- `settle_bet()` - Settle bet after match completion
- `claim_winnings()` - Claim winning payouts

#### Oracle Functions

- `register_oracle()` - Register oracle for match
- `update_match_result()` - Submit match result
- `validate_oracle_update()` - Validate oracle submissions

#### Administrative Functions

- `withdraw_platform_fees()` - Withdraw collected fees
- `emergency_withdraw()` - Emergency fund withdrawal

### Events

The platform emits comprehensive events for all major operations:

- `PlatformInitialized` - Platform setup
- `UserProfileCreated` - User registration
- `MatchCreated` - New match creation
- `BetPlaced` - Bet placement
- `MatchResultUpdated` - Oracle result submission
- `BetSettled` - Bet settlement
- `WinningsClaimed` - Payout claims

## 🤝 Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Guidelines

- Follow Rust best practices and Anchor conventions
- Add comprehensive tests for new features
- Update documentation for API changes
- Ensure security considerations are addressed

## � Recent Updates

### v0.1.0 - Current Release

- ✅ **Clean Compilation**: Removed all Rust and Anchor warnings for production-ready builds
- ✅ **Live Betting Architecture**: Implemented proper live betting during match execution
- ✅ **Comprehensive Testing**: All 11 tests passing with full functionality coverage
- ✅ **Optimized Build**: Size-optimized compilation with efficient resource usage
- ✅ **Type Safety**: Updated TypeScript integration with proper account handling
- ✅ **Documentation**: Complete API documentation and usage examples

## �📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 Support

For technical support and questions:

- Create an issue in the GitHub repository
- Join our Discord community
- Check the documentation wiki

## ⚠️ Disclaimer

This software is provided "as is" without warranty. Use at your own risk. Always conduct thorough security audits before deploying to mainnet. Gambling may be regulated in your jurisdiction - ensure compliance with local laws.

---

Built with ❤️ on Solana using Anchor Framework
