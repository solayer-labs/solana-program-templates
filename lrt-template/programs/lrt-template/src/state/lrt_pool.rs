use anchor_lang::prelude::*;
use anchor_lang::InitSpace;

#[account]
#[derive(InitSpace)]
pub struct LRTPool {
    pub bump: u8,
    pub input_token_mint: Pubkey,
    pub output_token_mint: Pubkey,
    pub delegate_authority: Pubkey,
}
