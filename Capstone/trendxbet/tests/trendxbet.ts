import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Trendxbet } from "../target/types/trendxbet";
import { expect } from "chai";
import { PublicKey, Keypair, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";

describe("TrendXBet - Cricket Betting Platform", () => {
  // Configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.trendxbet as Program<Trendxbet>;
  const provider = anchor.getProvider();

  // Test accounts
  const admin = Keypair.generate();
  const user1 = Keypair.generate();
  const user2 = Keypair.generate();
  const oracle = Keypair.generate();

  // Platform constants
  const PLATFORM_SEED = "platform";
  const USER_SEED = "user";
  const MATCH_SEED = "match";
  const BET_SEED = "bet";
  const TREASURY_SEED = "treasury";
  const ORACLE_SEED = "oracle";

  // Test data
  const matchId = Keypair.generate();
  const team1 = "Mumbai Indians";
  const team2 = "Chennai Super Kings";
  const description = "IPL 2024 Final - Mumbai vs Chennai";

  let globalStatePda: PublicKey;
  let treasuryPda: PublicKey;
  let user1StatePda: PublicKey;
  let user2StatePda: PublicKey;
  let matchStatePda: PublicKey;
  let oracleStatePda: PublicKey;

  before(async () => {
    // Airdrop SOL to test accounts
    await provider.connection.requestAirdrop(admin.publicKey, 10 * LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(user1.publicKey, 5 * LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(user2.publicKey, 5 * LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(oracle.publicKey, 2 * LAMPORTS_PER_SOL);

    // Wait for airdrops to confirm
    await new Promise(resolve => setTimeout(resolve, 1000));

    // Derive PDAs
    [globalStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from(PLATFORM_SEED)],
      program.programId
    );

    [treasuryPda] = PublicKey.findProgramAddressSync(
      [Buffer.from(TREASURY_SEED)],
      program.programId
    );

    [user1StatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from(USER_SEED), user1.publicKey.toBuffer()],
      program.programId
    );

    [user2StatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from(USER_SEED), user2.publicKey.toBuffer()],
      program.programId
    );

    [matchStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from(MATCH_SEED), matchId.publicKey.toBuffer()],
      program.programId
    );

    [oracleStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from(ORACLE_SEED), oracle.publicKey.toBuffer(), matchId.publicKey.toBuffer()],
      program.programId
    );
  });

  describe("Platform Initialization", () => {
    it("Should initialize the platform successfully", async () => {
      const tx = await program.methods
        .initialize(admin.publicKey)
        .accountsPartial({
          globalState: globalStatePda,
          treasury: treasuryPda,
          admin: admin.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([admin])
        .rpc();

      console.log("Platform initialized:", tx);

      // Verify global state
      const globalState = await program.account.globalState.fetch(globalStatePda);
      expect(globalState.admin.toString()).to.equal(admin.publicKey.toString());
      expect(globalState.houseEdge).to.equal(500); // 5%
      expect(globalState.isPaused).to.be.false;
    });
  });

  describe("User Management", () => {
    it("Should create user profiles", async () => {
      // Create user1 profile
      await program.methods
        .createUserProfile("CricketFan1")
        .accountsPartial({
          userState: user1StatePda,
          user: user1.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user1])
        .rpc();

      // Create user2 profile
      await program.methods
        .createUserProfile("CricketFan2")
        .accountsPartial({
          userState: user2StatePda,
          user: user2.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user2])
        .rpc();

      // Verify user profiles
      const user1State = await program.account.userState.fetch(user1StatePda);
      const user2State = await program.account.userState.fetch(user2StatePda);

      expect(user1State.username).to.equal("CricketFan1");
      expect(user2State.username).to.equal("CricketFan2");
      expect(user1State.totalBetsPlaced.toNumber()).to.equal(0);
      expect(user2State.totalBetsPlaced.toNumber()).to.equal(0);
    });

    it("Should deposit funds to user accounts", async () => {
      const depositAmount = 2 * LAMPORTS_PER_SOL;

      // Deposit funds for user1
      await program.methods
        .depositFunds(new anchor.BN(depositAmount))
        .accountsPartial({
          userState: user1StatePda,
          treasury: treasuryPda,
          authority: user1.publicKey,
          user: user1.publicKey,
          treasuryAccount: treasuryPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([user1])
        .rpc();

      // Verify deposit
      const user1State = await program.account.userState.fetch(user1StatePda);
      expect(user1State.balance.toNumber()).to.equal(depositAmount);
    });
  });

  describe("Match Management", () => {
    it("Should create a cricket match", async () => {
      const startTime = Math.floor(Date.now() / 1000) + 60; // Starts in 1 minute
      const endTime = startTime + 10800; // 3 hours match duration

      const tx = await program.methods
        .createMatch(team1, team2, new anchor.BN(startTime), new anchor.BN(endTime), description)
        .accountsPartial({
          matchState: matchStatePda,
          globalState: globalStatePda,
          matchId: matchId.publicKey,
          authority: admin.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([admin])
        .rpc();

      console.log("Match created:", tx);

      // Verify match creation
      const matchState = await program.account.matchState.fetch(matchStatePda);
      expect(matchState.team1).to.equal(team1);
      expect(matchState.team2).to.equal(team2);
      expect(matchState.description).to.equal(description);
      expect(matchState.totalPool.toNumber()).to.equal(0);
    });
  });

  describe("Oracle Management", () => {
    it("Should register oracle for match", async () => {
      await program.methods
        .registerOracle(oracle.publicKey)
        .accountsPartial({
          oracleState: oracleStatePda,
          globalState: globalStatePda,
          matchState: matchStatePda,
          oracleAuthority: oracle.publicKey,
          matchId: matchId.publicKey,
          admin: admin.publicKey,
          authority: admin.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([admin])
        .rpc();

      // Verify oracle registration
      const oracleState = await program.account.oracleState.fetch(oracleStatePda);
      expect(oracleState.oracleAuthority.toString()).to.equal(oracle.publicKey.toString());
      expect(oracleState.matchId.toString()).to.equal(matchId.publicKey.toString());
    });
  });

  describe("Betting Functionality", () => {
    it("Should allow users to place bets", async () => {
      // Create a separate match for betting with very close timing
      const bettingMatchId = Keypair.generate();
      const bettingStartTime = Math.floor(Date.now() / 1000) + 3; // Starts in 3 seconds
      const bettingEndTime = bettingStartTime + 3600; // 1 hour duration

      const [bettingMatchStatePda] = PublicKey.findProgramAddressSync(
        [Buffer.from(MATCH_SEED), bettingMatchId.publicKey.toBuffer()],
        program.programId
      );

      // Create the betting match
      await program.methods
        .createMatch("Team A", "Team B", new anchor.BN(bettingStartTime), new anchor.BN(bettingEndTime), "Test betting match")
        .accountsPartial({
          matchState: bettingMatchStatePda,
          globalState: globalStatePda,
          matchId: bettingMatchId.publicKey,
          authority: admin.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([admin])
        .rpc();

      // Wait for match to start
      console.log("Waiting for betting match to start...");
      await new Promise(resolve => setTimeout(resolve, 4000)); // Wait 4 seconds

      const betAmount = 0.5 * LAMPORTS_PER_SOL;
      const predictedTeam = 0; // Team A
      const oddsAccepted = 10000; // 1:1 odds

      const [betStatePda] = PublicKey.findProgramAddressSync(
        [Buffer.from(BET_SEED), user1.publicKey.toBuffer(), bettingMatchId.publicKey.toBuffer()],
        program.programId
      );

      await program.methods
        .placeBet(new anchor.BN(betAmount), predictedTeam, new anchor.BN(oddsAccepted))
        .accountsPartial({
          betState: betStatePda,
          userState: user1StatePda,
          matchState: bettingMatchStatePda,
          globalState: globalStatePda,
          treasury: treasuryPda,
          matchId: bettingMatchId.publicKey,
          bettor: user1.publicKey,
          authority: user1.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user1])
        .rpc();

      // Verify bet placement
      const betState = await program.account.betState.fetch(betStatePda);
      const bettingMatchState = await program.account.matchState.fetch(bettingMatchStatePda);
      const user1State = await program.account.userState.fetch(user1StatePda);

      expect(betState.amount.toNumber()).to.equal(betAmount);
      expect(betState.predictedTeam).to.equal(predictedTeam);
      expect(bettingMatchState.totalPool.toNumber()).to.equal(betAmount);
      expect(user1State.totalBetsPlaced.toNumber()).to.equal(1);
    });
  });

  describe("Admin Functions", () => {
    it("Should allow admin to update platform configuration", async () => {
      const newHouseEdge = 300; // 3%
      const newMinBet = 1000000; // 0.001 SOL
      const newMaxBet = 50000000000; // 50 SOL

      await program.methods
        .updatePlatformConfig(newHouseEdge, new anchor.BN(newMinBet), new anchor.BN(newMaxBet))
        .accountsPartial({
          globalState: globalStatePda,
          admin: admin.publicKey,
        })
        .signers([admin])
        .rpc();

      // Verify configuration update
      const globalState = await program.account.globalState.fetch(globalStatePda);
      expect(globalState.houseEdge).to.equal(newHouseEdge);
      expect(globalState.minBetAmount.toNumber()).to.equal(newMinBet);
      expect(globalState.maxBetAmount.toNumber()).to.equal(newMaxBet);
    });

    it("Should allow admin to pause/unpause platform", async () => {
      // Pause platform
      await program.methods
        .pausePlatform()
        .accountsPartial({
          globalState: globalStatePda,
          admin: admin.publicKey,
        })
        .signers([admin])
        .rpc();

      let globalState = await program.account.globalState.fetch(globalStatePda);
      expect(globalState.isPaused).to.be.true;

      // Unpause platform
      await program.methods
        .unpausePlatform()
        .accountsPartial({
          globalState: globalStatePda,
          admin: admin.publicKey,
        })
        .signers([admin])
        .rpc();

      globalState = await program.account.globalState.fetch(globalStatePda);
      expect(globalState.isPaused).to.be.false;
    });
  });

  describe("Treasury Management", () => {
    it("Should track treasury operations correctly", async () => {
      // Use the betting match for treasury operations
      const bettingMatchId2 = Keypair.generate();
      const bettingStartTime2 = Math.floor(Date.now() / 1000) + 3; // Starts in 3 seconds
      const bettingEndTime2 = bettingStartTime2 + 3600; // 1 hour duration

      const [bettingMatchStatePda2] = PublicKey.findProgramAddressSync(
        [Buffer.from(MATCH_SEED), bettingMatchId2.publicKey.toBuffer()],
        program.programId
      );

      // Create another betting match
      await program.methods
        .createMatch("Team C", "Team D", new anchor.BN(bettingStartTime2), new anchor.BN(bettingEndTime2), "Treasury test match")
        .accountsPartial({
          matchState: bettingMatchStatePda2,
          globalState: globalStatePda,
          matchId: bettingMatchId2.publicKey,
          authority: admin.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([admin])
        .rpc();

      // Wait for match to start
      await new Promise(resolve => setTimeout(resolve, 4000)); // Wait 4 seconds

      const betAmount = 0.3 * LAMPORTS_PER_SOL;
      const predictedTeam = 1; // Team D
      const oddsAccepted = 10000; // 1:1 odds

      const [betStatePda2] = PublicKey.findProgramAddressSync(
        [Buffer.from(BET_SEED), user2.publicKey.toBuffer(), bettingMatchId2.publicKey.toBuffer()],
        program.programId
      );

      // Deposit funds for user2 first
      const depositAmount = 1 * LAMPORTS_PER_SOL;
      await program.methods
        .depositFunds(new anchor.BN(depositAmount))
        .accountsPartial({
          userState: user2StatePda,
          treasury: treasuryPda,
          authority: user2.publicKey,
          user: user2.publicKey,
          treasuryAccount: treasuryPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([user2])
        .rpc();

      // Place a bet to generate treasury activity
      await program.methods
        .placeBet(new anchor.BN(betAmount), predictedTeam, new anchor.BN(oddsAccepted))
        .accountsPartial({
          betState: betStatePda2,
          userState: user2StatePda,
          matchState: bettingMatchStatePda2,
          globalState: globalStatePda,
          treasury: treasuryPda,
          matchId: bettingMatchId2.publicKey,
          bettor: user2.publicKey,
          authority: user2.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user2])
        .rpc();

      const treasury = await program.account.treasuryState.fetch(treasuryPda);

      // Verify treasury has recorded deposits and fees
      expect(treasury.totalDeposits.toNumber()).to.be.greaterThan(0);
      expect(treasury.platformFees.toNumber()).to.be.greaterThan(0);
    });
  });

  describe("Error Handling", () => {
    it("Should reject invalid bet amounts", async () => {
      // Create another betting match for error testing
      const errorTestMatchId = Keypair.generate();
      const errorTestStartTime = Math.floor(Date.now() / 1000) + 3; // Starts in 3 seconds
      const errorTestEndTime = errorTestStartTime + 3600; // 1 hour duration

      const [errorTestMatchStatePda] = PublicKey.findProgramAddressSync(
        [Buffer.from(MATCH_SEED), errorTestMatchId.publicKey.toBuffer()],
        program.programId
      );

      // Create the error test match
      await program.methods
        .createMatch("Error Team 1", "Error Team 2", new anchor.BN(errorTestStartTime), new anchor.BN(errorTestEndTime), "Error test match")
        .accountsPartial({
          matchState: errorTestMatchStatePda,
          globalState: globalStatePda,
          matchId: errorTestMatchId.publicKey,
          authority: admin.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([admin])
        .rpc();

      // Wait for match to start
      await new Promise(resolve => setTimeout(resolve, 4000)); // Wait 4 seconds

      const invalidBetAmount = 1000; // Below minimum
      const predictedTeam = 1;
      const oddsAccepted = 10000;

      const [betStatePda] = PublicKey.findProgramAddressSync(
        [Buffer.from(BET_SEED), user2.publicKey.toBuffer(), errorTestMatchId.publicKey.toBuffer()],
        program.programId
      );

      try {
        await program.methods
          .placeBet(new anchor.BN(invalidBetAmount), predictedTeam, new anchor.BN(oddsAccepted))
          .accountsPartial({
            betState: betStatePda,
            userState: user2StatePda,
            matchState: errorTestMatchStatePda,
            globalState: globalStatePda,
            treasury: treasuryPda,
            matchId: errorTestMatchId.publicKey,
            bettor: user2.publicKey,
            authority: user2.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([user2])
          .rpc();

        // Should not reach here
        expect.fail("Expected transaction to fail");
      } catch (error: any) {
        expect(error.message).to.include("BetAmountTooLow");
      }
    });

    it("Should reject unauthorized admin actions", async () => {
      try {
        await program.methods
          .pausePlatform()
          .accountsPartial({
            globalState: globalStatePda,
            admin: user1.publicKey, // Wrong admin
          })
          .signers([user1])
          .rpc();

        expect.fail("Expected transaction to fail");
      } catch (error: any) {
        expect(error.message).to.include("Unauthorized");
      }
    });
  });
});
