use anchor_lang::prelude::*;

pub mod instructions;

pub use instructions::{Initialize, List, Delist};

declare_id!("6VW4jkQzWFUQikiTaXeLtJsCp4ERtShGhgTeRTSijMV");

#[program]
pub mod marketplace {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, name: String, fee: u16) -> Result<()> {
        ctx.accounts.init(name, fee, &ctx.bumps)?;
        Ok(())
    }

    pub fn list(ctx: Context<List>, price: u64) -> Result<()> {
        ctx.accounts.create_listing(price, &ctx.bumps)?;
        ctx.accounts.deposit_nft()?;
        Ok(())
    }

    pub fn delist(ctx: Context<Delist>) -> Result<()> {
        ctx.accounts.withdraw_nft()?;
        ctx.accounts.close_listing()?;
        Ok(())
    }
}