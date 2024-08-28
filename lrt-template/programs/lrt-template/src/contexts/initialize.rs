use crate::{errors::LRTPoolError, state::*};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    signer: Signer<'info>,
    delegate_authority: Signer<'info>,
    #[account(
        mint::token_program = token_program,
    )]
    input_token_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::authority = pool,
        associated_token::mint = input_token_mint,
        associated_token::token_program = token_program
    )]
    pool_input_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mint::decimals = input_token_mint.decimals,
        mint::authority = pool,
        mint::freeze_authority = pool,
        mint::token_program = token_program,
        constraint = output_token_mint.supply == 0 @ LRTPoolError::NonZeroRstMintSupply
    )]
    output_token_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init,
        payer=signer,
        space = 8 + LRTPool::INIT_SPACE,
        seeds = [b"lrt_pool", output_token_mint.key().as_ref()],
        bump
    )]
    pool: Box<Account<'info, LRTPool>>,
    associated_token_program: Program<'info, AssociatedToken>,
    token_program: Interface<'info, TokenInterface>,
    system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, bumps: InitializeBumps) -> Result<()> {
        self.pool.set_inner(LRTPool {
            bump: bumps.pool,
            input_token_mint: self.input_token_mint.key(),
            output_token_mint: self.output_token_mint.key(),
            delegate_authority: self.delegate_authority.key(),
        });
        Ok(())
    }
}
