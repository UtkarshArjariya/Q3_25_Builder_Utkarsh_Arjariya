use crate::constants::*;
use crate::events::*;
use crate::state::*;
use crate::utils::{TimeUtils, ValidationUtils};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = admin,
        space = GlobalState::LEN,
        seeds = [PLATFORM_SEED],
        bump
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        init,
        payer = admin,
        space = TreasuryState::LEN,
        seeds = [TREASURY_SEED],
        bump
    )]
    pub treasury: Account<'info, TreasuryState>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Initialize>, admin: Pubkey) -> Result<()> {
    let global_state = &mut ctx.accounts.global_state;
    let treasury = &mut ctx.accounts.treasury;
    let current_time = TimeUtils::get_current_timestamp();

    // Validate house edge
    ValidationUtils::validate_house_edge(DEFAULT_HOUSE_EDGE)?;

    // Initialize global state
    global_state.initialize(
        admin,
        DEFAULT_HOUSE_EDGE,
        MIN_BET_AMOUNT,
        MAX_BET_AMOUNT,
        ctx.bumps.global_state,
        current_time,
    );

    // Initialize treasury
    treasury.initialize(admin, ctx.bumps.treasury, current_time);

    // Emit initialization event
    emit!(PlatformInitialized {
        admin,
        house_edge: DEFAULT_HOUSE_EDGE,
        min_bet_amount: MIN_BET_AMOUNT,
        max_bet_amount: MAX_BET_AMOUNT,
        timestamp: current_time,
    });

    msg!(
        "TrendXBet platform initialized successfully by admin: {}",
        admin
    );
    Ok(())
}
