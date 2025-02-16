use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenInterface;

use crate::state::User;

#[derive(Accounts)]
pub struct InitializeUser<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        space = 8 + User::INIT_SPACE,
        seeds = [b"user", user.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, User>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeUser<'info> {
    pub fn initialize_user(&mut self, bumps: &InitializeUserBumps) -> Result<()> {
        self.user_account.set_inner(User {
            points: 0,
            amount_staked: 0,
            bump: bumps.user_account,
        });
        Ok(())
    }
}