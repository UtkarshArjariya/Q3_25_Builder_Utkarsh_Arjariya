#![allow(unexpected_cfgs)]
#![allow(deprecated)]
#![allow(unused)]

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("AKg34g8K4mouyyB1Fx53wnJoBTiErHmtsstztXfmCWZ3");

#[program]
pub mod anchor_escrow {
    use super::*;
}
