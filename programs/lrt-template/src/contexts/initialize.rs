use std::str::FromStr;

use crate::{
    constants::{SOLAYER_RESTAKE_POOL, SOLAYER_SOL_ACCOUNT},
    errors::LRTPoolError,
    state::*,
};
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
    lst_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::authority = pool,
        associated_token::mint = lst_mint,
        associated_token::token_program = token_program
    )]
    lst_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mint::decimals = lst_mint.decimals,
        mint::authority = pool,
        mint::freeze_authority = pool,
        mint::token_program = token_program,
        constraint = rst_mint.supply == 0 @ LRTPoolError::NonZeroRstMintSupply
    )]
    rst_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        mint::token_program = token_program,
        mint::authority = Pubkey::from_str(SOLAYER_RESTAKE_POOL).unwrap(),
        mint::freeze_authority = Pubkey::from_str(SOLAYER_RESTAKE_POOL).unwrap(),
        constraint = ssol_mint.key() == Pubkey::from_str(SOLAYER_SOL_ACCOUNT).unwrap(),
    )]
    ssol_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::authority = pool,
        associated_token::mint = ssol_mint,
        associated_token::token_program = token_program
    )]
    ssol_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(init, payer=signer, space = 8 + LRTPool::INIT_SPACE, seeds = [b"lrt_pool", lst_mint.key().as_ref(), rst_mint.key().as_ref(), ssol_mint.key().as_ref()], bump)]
    pool: Box<Account<'info, LRTPool>>,
    associated_token_program: Program<'info, AssociatedToken>,
    token_program: Interface<'info, TokenInterface>,
    system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, bumps: InitializeBumps) -> Result<()> {
        self.pool.set_inner(LRTPool {
            bump: bumps.pool,
            lst_mint: self.lst_mint.key(),
            rst_mint: self.rst_mint.key(),
            lrt_mint: self.ssol_mint.key(),
            delegate_authority: self.delegate_authority.key(),
        });
        Ok(())
    }
}
