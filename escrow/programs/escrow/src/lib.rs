use anchor_lang::prelude::*;

declare_id!("DYEKUsqpvYuATxv9aH1AGVc44EgryEi9mdnq5ypQd3Q6");

#[program]
pub mod escrow {
    use super::*;

    pub fn make(ctx: Context<Make>, seed: u64, receive_amount: u64) -> Result<()> {
        ctx.accounts.make(seed, receive_amount, &ctx.bumps)?;
        ctx.accounts.deposit(receive_amount)? ;
        Ok(())
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.refund()?;
        ctx.accounts.close()?;
        Ok(())
    }

    // swap token a to token b withouth storing it in the escrow/vault
    pub fn take(ctx: Context<Take>) -> Result<()> {
        ctx.accounts.deposit()?;
        ctx.accounts.release()?;
        ctx.accounts.close()?;
        Ok(())
    }
}
