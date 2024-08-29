use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Burn,
    token_interface::{
        burn, transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
    },
};

use crate::{errors::LRTPoolError, state::LRTPool};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    signer: Signer<'info>,
    #[account(
        mut,
        mint::token_program = token_program,
        address = pool.input_token_mint
    )]
    input_token_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        associated_token::authority = signer,
        associated_token::mint = input_token_mint,
        associated_token::token_program = token_program
    )]
    signer_input_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::authority = pool,
        associated_token::mint = input_token_mint,
        associated_token::token_program = token_program
    )]
    pool_input_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        mint::token_program = token_program,
        mint::authority = pool,
        mint::freeze_authority = pool,
        mint::decimals = input_token_mint.decimals,
        address = pool.output_token_mint
    )]
    output_token_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        associated_token::authority = signer,
        associated_token::mint = output_token_mint,
        associated_token::token_program = token_program
    )]
    signer_output_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        has_one = input_token_mint,
        has_one = output_token_mint,
        seeds = [b"lrt_pool", output_token_mint.key().as_ref()],
        bump = pool.bump
    )]
    pool: Box<Account<'info, LRTPool>>,
    associated_token_program: Program<'info, AssociatedToken>,
    token_program: Interface<'info, TokenInterface>,
    system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn burn_output_token(&mut self, amount: u64) -> Result<()> {
        let ctx = CpiContext::new(
            self.token_program.to_account_info(),
            Burn {
                mint: self.output_token_mint.to_account_info(),
                from: self.signer_output_token_vault.to_account_info(),
                authority: self.signer.to_account_info(),
            },
        );
        burn(ctx, amount)
    }

    pub fn unstake(&mut self, amount: u64) -> Result<()> {
        self.pool_input_token_vault.reload()?;
        if self.pool_input_token_vault.amount < amount {
            return Err(LRTPoolError::InsufficientStakedSOLFundsForWithdraw.into());
        }

        let bump = [self.pool.bump];
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"lrt_pool",
            self.output_token_mint.to_account_info().key.as_ref(),
            &bump,
        ][..]];

        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            TransferChecked {
                from: self.pool_input_token_vault.to_account_info(),
                to: self.signer_input_token_vault.to_account_info(),
                mint: self.input_token_mint.to_account_info(),
                authority: self.pool.to_account_info(),
            },
            &signer_seeds,
        );

        transfer_checked(ctx, amount, self.input_token_mint.decimals)
    }

    // fill this function according to your business logic
    pub fn calculate_input_token_amount(&self, amount: u64) -> u64 {
        amount
    }
}
