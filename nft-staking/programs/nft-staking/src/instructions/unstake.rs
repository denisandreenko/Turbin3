use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        mpl_token_metadata::instructions::{
            ThawDelegatedAccountCpi, ThawDelegatedAccountCpiAccounts,
        },
        MasterEditionAccount, Metadata, MetadataAccount,
    },
    token_interface::{revoke, Mint, Revoke, TokenAccount, TokenInterface},
};

use crate::state::{StakeAccount, StakeConfig, User};

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub mint: Box<InterfaceAccount<'info, Mint>>,
    pub collection: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        associated_token::authority = user,
        associated_token::mint = mint
    )]
    pub mint_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        seeds = [b"metadata", metadata_program.key().as_ref(), mint.key().as_ref()],
        seeds::program = metadata_program.key(),
        bump
    )]
    pub metadata: Box<Account<'info, MetadataAccount>>,

    #[account(
        seeds = [b"metadata", metadata_program.key().as_ref(), mint.key().as_ref(), b"edition"],
        seeds::program = metadata_program.key(),
        bump
    )]
    pub edition: Box<Account<'info, MasterEditionAccount>>,

    pub config: Box<Account<'info, StakeConfig>>,

    #[account(
        mut,
        seeds = [b"stake", mint.key().as_ref(), config.key().as_ref()],
        bump
    )]
    pub stake_account: Box<Account<'info, StakeAccount>>,

    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_account.bump
    )]
    pub user_account: Box<Account<'info, User>>,

    pub metadata_program: Program<'info, Metadata>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Unstake<'info> {
    pub fn unstake(&mut self) -> Result<()> {
        let delegate = &self.stake_account.to_account_info();
        let token_account = &self.mint_ata.to_account_info();
        let edition = &self.edition.to_account_info();
        let mint = &self.mint.to_account_info();
        let token_program = &self.token_program.to_account_info();
        let metadata_program = &self.metadata_program.to_account_info();

        let seeds = &[
            b"stake",
            self.mint.to_account_info().key.as_ref(),
            self.config.to_account_info().key.as_ref(),
            &[self.stake_account.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        // unfreeze a delegated nft
        ThawDelegatedAccountCpi::new(
            metadata_program,
            ThawDelegatedAccountCpiAccounts {
                delegate,
                token_account,
                edition,
                mint,
                token_program,
            },
        )
        .invoke_signed(signer_seeds)?;

        // Revoke Auth
        let cpi_accounts = Revoke {
            source: self.mint_ata.to_account_info(),
            authority: self.user.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);

        revoke(cpi_ctx)?;

        self.user_account.amount_staked -= 1;

        Ok(())
    }
}
