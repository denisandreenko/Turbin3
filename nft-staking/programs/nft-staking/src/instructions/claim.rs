use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{ mint_to, Mint, MintTo, TokenAccount, TokenInterface },
};

use crate::state::{ StakeConfig, User };

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"rewards_mint", config.key().as_ref()],
        bump = config.reward_bump
    )]
    pub rewards_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = rewards_mint,
        associated_token::authority = user
    )]
    pub rewards_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_account.bump
    )]
    pub user_account: Account<'info, User>,

    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, StakeConfig>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Claim<'info> {
    pub fn claim(&mut self) -> Result<()> {
        let cpi_accounts = MintTo {
            mint: self.rewards_mint.to_account_info(),
            to: self.rewards_ata.to_account_info(),
            authority: self.config.to_account_info(),
        };

        let seeds = &[b"config".as_ref(), &[self.config.bump]];
        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds
        );

        // convert points to token units (10^6 decimals for this case)
        let amount = (self.user_account.points as u64) * (10u64).pow(self.rewards_mint.decimals as u32);
        mint_to(cpi_ctx, amount)?;

        self.user_account.points = 0;

        Ok(())
    }
}