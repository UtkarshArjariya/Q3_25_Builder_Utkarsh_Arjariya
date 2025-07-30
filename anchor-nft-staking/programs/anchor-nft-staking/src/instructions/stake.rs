use anchor_lang::prelude::*;
use anchor_spl::{
    metadata::{
        mpl_token_metadata::instructions::{
            FreezeDelegatedAccountCpi, FreezeDelegatedAccountCpiAccounts,
        },
        MasterEditionAccount, Metadata, MetadataAccount,
    },
    token::{approve, Approve, Mint, Token, TokenAccount},
};

use crate::{
    error::StakeError,
    state::{StakeAccount, StakeConfig, UserAccount},
};

//      this instruction will allow the user to stake nfts.
//       - accounts:
//         - user
//         - mint
//         - collection mint
//         - metadata
//         - edition
//         - config
//         - user_account
//         - stake_account
//         - user_mint_ata
//         - the three programs

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>, // The user who is staking the NFT & signing the transaction

    pub mint: Account<'info, Mint>, // The mint of the NFT being staked

    pub collection_mint: Account<'info, Mint>, // The mint of the collection the NFT belongs to

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = user,
    )]
    pub user_mint_ata: Account<'info, TokenAccount>, // User's Mint ATA

    //     In the Metaplex Token Metadata program,
    //     every NFT’s on‑chain metadata is stored in a PDA (Program‑Derived Address)
    //     that’s computed according to a fixed scheme. Anchor lets you mirror that derivation
    //     in your #[derive(Accounts)] by specifying the exact same seeds.
    #[account(
        seeds = [
            b"metadata",
            metadata_program.key().as_ref(),
            mint.key().as_ref(),
        ],
        seeds::program = metadata_program.key(),  // These steps let us obtain & mirror the metadata of the mint from Metaplex, just for confirmation
        bump,
        constraint = metadata.collection.as_ref().unwrap().key.as_ref() == collection_mint.key().as_ref(),
        constraint = metadata.collection.as_ref().unwrap().verified == true,
        // Above two constraints are security checks, if these checks are not met, the transaction will fail
    )]
    pub metadata: Account<'info, MetadataAccount>, // Metadata of the NFT being staked

    #[account(
        seeds = [
            b"metadata",
            metadata_program.key().as_ref(),
            mint.key().as_ref(),
            b"edition"
        ],
        seeds::program = metadata_program.key(),
        bump,
    )]
    pub edition: Account<'info, MasterEditionAccount>, // Master Edition of the NFT being staked

    #[account(
        seeds = [b"config"], 
        bump = config.bump
    )]
    pub config: Account<'info, StakeConfig>, // Stake configuration account

    #[account(
        mut,
        seeds = [b"user",user.key().as_ref()],
        bump = user_account.bump
    )]
    pub user_account: Account<'info, UserAccount>, // User's account that tracks points and stake amount

    #[account(
        init,
        payer = user,
        space = 8 + StakeAccount::INIT_SPACE,
        seeds = [b"stake", mint.key().as_ref(), config.key().as_ref()],
        bump
    )]
    pub stake_account: Account<'info, StakeAccount>, // Stake account that holds the staked NFT

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub metadata_program: Program<'info, Metadata>,
}

// Steps
// 1. Getting approved
// 2. Freezing the token in the users account
// 3. Creating a stake account for the mint/NFT
// 4. Increasing the user's stake Amount

impl<'info> Stake<'info> {
    pub fn stake(&mut self, bumps: &StakeBumps) -> Result<()> {
        require!(
            self.user_account.amount_staked < self.config.max_stake,
            StakeError::MaxStakeReached
        );

        let cpi_program = self.token_program.to_account_info();

        // Function of Approve:
        // The program makes a CPI to the SPL Token Program. This approves call doesn't move the NFT.
        // Instead, the user(Authority) gives the stake_account(PDA) the permission(delegates authority) to manage the token in this user_mint_ata.
        // This permission is what allows the stake_account to freeze the NFT in the next step.

        let cpi_accounts = Approve {
            to: self.user_mint_ata.to_account_info(), // User's Mint ATA(Token account)
            delegate: self.stake_account.to_account_info(), // Stake Account PDA(Delegation authority)
            authority: self.user.to_account_info(),         // User's Signature(Owner/Authority)
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        approve(cpi_ctx, 1);

        let delegate = &self.stake_account.to_account_info();
        let token_account = &self.user_mint_ata.to_account_info();
        let edition = &self.edition.to_account_info();
        let mint = &self.mint.to_account_info();
        let token_program = &self.token_program.to_account_info();
        let metadata_program = &self.metadata_program.to_account_info();

        // After the approve your stake_account as a delegate for the user's token account,
        // you need the Metadata program(the on-chain MPL Token Metadata program) to freeze that delegated account.
        // - The token is user_mint_ata becomes non-transferable(aka locked).
        // - The metadata program marks it as "frozen" under the authority of the delegator

        FreezeDelegatedAccountCpi::new(
            metadata_program,
            FreezeDelegatedAccountCpiAccounts {
                delegate,      // Stake Account PDA(Authority)
                token_account, // User's Mint ATA(NFT Holding)
                edition,       // NFT's master edition PDA
                mint,          // NFT's mint
                token_program, // SPL Token Program
            },
        )
        .invoke();

        self.stake_account.set_inner(StakeAccount {
            owner: self.user.key(),
            mint: self.mint.key(),
            staked_at: Clock::get()?.unix_timestamp,
            bump: bumps.stake_account,
        });

        self.user_account.amount_staked += 1;

        Ok(())
    }
}
