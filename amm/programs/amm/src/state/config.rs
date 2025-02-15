use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub authority: Option<Pubkey>, // authority to lock the config account
    pub seed: u64, // to be able to create different pools
    pub fee: u16, // swap fee in bps
    pub mint_x: Pubkey,
    pub mint_y: Pubkey,
    pub locked: bool,
    pub config_bump: u8,
    pub lp_bump: u8,
}

impl Space for Config {
    // Option -> 1 byte
    // PubKey -> 32 bytes
    const INIT_SPACE: usize = 8 + 8 + (1 + 32) + 32 + 32 + 2 + 1 + 1 + 1;
}