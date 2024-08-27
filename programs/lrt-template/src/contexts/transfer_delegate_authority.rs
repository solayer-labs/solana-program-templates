use crate::state::LRTPool;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct TransferDelegateAuthority<'info> {
    #[account(mut)]
    authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"lrt_pool", pool.lst_mint.key().as_ref(), pool.rst_mint.key().as_ref(), pool.lrt_mint.key().as_ref()],
        bump = pool.bump,
        constraint = pool.delegate_authority == authority.key()
    )]
    pool: Account<'info, LRTPool>,

    new_authority: UncheckedAccount<'info>,
}

impl<'info> TransferDelegateAuthority<'info> {
    pub fn transfer_authority(&mut self) -> Result<()> {
        self.pool.delegate_authority = self.new_authority.key();
        msg!("Transferred authority to {}", self.new_authority.key());
        Ok(())
    }
}
