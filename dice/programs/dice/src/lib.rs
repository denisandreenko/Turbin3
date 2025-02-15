use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;

pub use instructions::*;
pub use state::*;
pub use errors::*;

declare_id!("DZRpyoX2AgupKALDv89pqyMRuYirDws2feJEe5o7QQUu");

#[program]
pub mod dice {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, amount: u64) -> Result<()> {
        ctx.accounts.init(amount)?;
        Ok(())
    }

    /// * `seed` - Random seed for the bet
    /// * `roll` - Number that player needs to roll under to win
    /// * `amount` - Amount of lamports being bet
    pub fn place_bet(ctx: Context<PlaceBet>, seed: u128, amount: u64, roll: u8) -> Result<()> {
        ctx.accounts.create_bet(seed, &ctx.bumps, amount, roll)?;
        ctx.accounts.deposit(amount)?;
        Ok(())
    }

    /// * `sig` - Ed25519 signature from house to verify the bet outcome
    pub fn resolve_bet(ctx: Context<ResolveBet>, sig: Vec<u8>) -> Result<()> {
        ctx.accounts.verify_ed25519_signature(&sig)?;
        ctx.accounts.resolve_bet(&ctx.bumps, &sig)
    }

    /// Refund a bet that wasn't resolved within the time limit
    pub fn refund(ctx: Context<RefundBet>) -> Result<()> {
        ctx.accounts.refund_bet(&ctx.bumps)
    }
}
