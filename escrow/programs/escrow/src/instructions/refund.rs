use anchor_lang::prelude::*;  
use anchor_spl::token_interface::{TokenInterface, Mint, TokenAccount};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{TransferChecked, transfer_checked, close_account, CloseAccount};

use crate::state::EscrowState;

#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    pub mint_a: InterfaceAccount<'info, Mint>,
    pub mint_b: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,  

    #[account(
        mut,
        close = maker,
        has_one = maker,
        seeds = [b"escrow", maker.key.as_ref(), escrow.seed.to_le_bytes().as_ref() ],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, EscrowState>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Refund<'info> {
    pub fn refund(&mut self) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = TransferChecked {
            to: self.maker_ata_a.to_account_info(),
            mint: self.mint_a.to_account_info(),
            from: self.vault.to_account_info(),
            authority: self.maker.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer_checked(cpi_ctx, self.vault.amount, self.mint_a.decimals)?;
        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
        let cpi_accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let seed_bytes = self.escrow.seed.to_le_bytes();
        let seeds = &[b"escrow", self.escrow.maker.as_ref(), seed_bytes.as_ref(), &[self.escrow.bump]];
        let signers_seeds = [&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(self.system_program.to_account_info(), cpi_accounts, &signers_seeds);

        close_account(cpi_ctx)?;
        Ok(())
    }
}