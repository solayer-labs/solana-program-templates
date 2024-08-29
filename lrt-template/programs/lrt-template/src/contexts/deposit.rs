use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    mint_to, transfer_checked, Mint, MintTo, TokenAccount, TokenInterface, TransferChecked,
};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    signer: Signer<'info>,

    #[account(
        mint::token_program = token_program,
        address = pool.input_token_mint
    )]
    input_token_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        associated_token::authority = signer,
        associated_token::mint = input_token_mint,
        associated_token::token_program = token_program
    )]
    signer_input_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::authority = pool,
        associated_token::mint = input_token_mint
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
        init_if_needed,
        payer = signer,
        associated_token::authority = signer,
        associated_token::mint = output_token_mint
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

impl<'info> Deposit<'info> {
    pub fn stake(&mut self, amount: u64) -> Result<()> {
        let ctx = CpiContext::new(
            self.token_program.to_account_info(),
            TransferChecked {
                from: self.signer_input_token_vault.to_account_info(),
                to: self.pool_input_token_vault.to_account_info(),
                mint: self.input_token_mint.to_account_info(),
                authority: self.signer.to_account_info(),
            },
        );

        transfer_checked(ctx, amount, self.input_token_mint.decimals)
    }

    pub fn mint_output_token(&mut self, amount: u64) -> Result<()> {
        let bump = [self.pool.bump];

        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"lrt_pool",
            self.output_token_mint.to_account_info().key.as_ref(),
            &bump,
        ][..]];

        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            MintTo {
                mint: self.output_token_mint.to_account_info(),
                to: self.signer_output_token_vault.to_account_info(),
                authority: self.pool.to_account_info(),
            },
            &signer_seeds[..],
        );

        mint_to(ctx, amount)
    }

    // fill this function according to your business logic
    pub fn calculate_output_token_amount(&self, amount: u64) -> u64 {
        amount
    }
}
