use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{Transfer, transfer, Mint, Token, TokenAccount, MintTo, mint_to}};
use constant_product_curve::ConstantProduct;

use crate::{errors::AmmError, state::Config};

/// Deposit liquidity into the AMM pool
#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>, // the account providing liquidity for the market 
    
    pub mint_x: Account<'info, Mint>,
    pub mint_y: Account<'info, Mint>,
    
    /// Pool configuration account
    #[account(
        has_one = mint_x, // config.mint_x == mint_x
        has_one = mint_y, // config.mint_y == mint_y
        seeds = [
            b"config",
            mint_x.key().to_bytes().as_ref(),
            mint_y.key().to_bytes().as_ref(),
            seed.to_le_bytes().as_ref(),
        ],
        bump = config.config_bump,
    )]
    pub config: Account<'info, Config>,
    
    /// LP token mint account
    #[account(
        mut,
        seeds = [b"lp", config.key().as_ref()],
        bump = config.lp_bump,
        mint::authority=config,
        mint::decimals=6
    )]
    pub mint_lp: Account<'info, Mint>,
    
    /// Pool's vault for token X
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config,
    )]
    pub vault_x: Account<'info, TokenAccount>,
    
    /// Pool's vault for token Y
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config,
    )]
    pub vault_y: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = user,
    )]
    pub user_x_ata: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = user,
    )]
    pub user_y_ata: Account<'info, TokenAccount>,
    
    /// User's LP token account (init if it doesn't exist)
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_lp,
        associated_token::authority = user,
    )]
    pub user_lp: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Deposit<'info> {
    /// * `amount` - Amount of LP tokens the user wants to receive
    /// * `max_x` - Maximum amount of token X user is willing to deposit
    /// * `max_y` - Maximum amount of token Y user is willing to deposit
    pub fn deposit (
        &mut self,
        amount: u64,
        max_x: u64,
        max_y: u64,
    ) -> Result<()> {
        require!(self.config.locked == false, AmmError::PoolLocked);
        require!(amount != 0, AmmError::InvalidAmount);

        // Calculate deposit amounts based on pool state
        let (x, y) = match self.mint_lp.supply == 0 && self.vault_x.amount == 0 && self.vault_y.amount == 0 {
            // For first deposit, use maximum amounts directly
            true => (max_x, max_y),
            // For subsequent deposits, calculate based on pool ratio
            false => {
                let amounts = ConstantProduct::xy_deposit_amounts_from_l(
                    self.vault_x.amount, 
                    self.vault_y.amount, 
                    self.mint_lp.supply, 
                    amount, 
                    6
                ).unwrap();
                (amounts.x, amounts.y)
            }
        };

        // Verify amounts are within user's specified limits
        require!(x <= max_x && y <= max_y, AmmError::SlippageExceeded);

        // Execute the deposits and LP token minting
        self.deposit_tokens(true, x)?;  // deposit token X
        self.deposit_tokens(false, y)?; // deposit token Y
        self.mint_lp_tokens(amount)?;     // mint LP tokens to user

        Ok(())
    }

    /// Helper function to transfer tokens from user to pool vault
    pub fn deposit_tokens(&self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to) = match is_x {
            true => (self.user_x_ata.to_account_info(), self.vault_x.to_account_info()),
            false => (self.user_y_ata.to_account_info(), self.vault_y.to_account_info()),
        };

        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from,
            to,
            authority: self.user.to_account_info(),
        };
        let ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer(ctx, amount)?;

        Ok(())
    }

    pub fn mint_lp_tokens(&self, amount: u64) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = MintTo {
            mint: self.mint_lp.to_account_info(),
            to: self.user_lp.to_account_info(),
            authority: self.config.to_account_info(),
        };

        let seeds = [
            b"config",
            self.mint_x.key().to_bytes().as_ref(),
            self.mint_y.key().to_bytes().as_ref(), 
            self.config.seed.to_le_bytes().as_ref()
        ];
        let signer_seeds = &[&seeds[..]];

        let ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        mint_to(ctx, amount)?;

        Ok(())
    }
}