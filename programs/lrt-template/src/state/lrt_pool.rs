use anchor_lang::prelude::*;
use anchor_lang::InitSpace;

#[account]
#[derive(InitSpace)]
pub struct LRTPool {
    pub bump: u8,
    pub lst_mint: Pubkey,
    pub rst_mint: Pubkey,
    pub lrt_mint: Pubkey,
    pub delegate_authority: Pubkey,
}
