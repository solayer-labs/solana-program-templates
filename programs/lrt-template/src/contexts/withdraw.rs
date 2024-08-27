use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Burn,
    token_interface::{
        burn, transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
    },
};
use solana_program::{instruction::Instruction, program::invoke_signed};
use std::str::FromStr;

use crate::{
    constants::{SOLAYER_RESTAKE_POOL, SOLAYER_RESTAKE_PROGRAM_ID, SOLAYER_SOL_ACCOUNT},
    errors::LRTPoolError,
    state::LRTPool,
    utils::sighash,
};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    signer: Signer<'info>,
    #[account(
        mut,
        mint::token_program = token_program,
    )]
    lst_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        associated_token::authority = signer,
        associated_token::mint = lst_mint,
        associated_token::token_program = token_program
    )]
    lst_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        mint::token_program = token_program,
        mint::authority = pool,
        mint::freeze_authority = pool,
        mint::decimals = lst_mint.decimals
    )]
    rst_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        associated_token::authority = signer,
        associated_token::mint = rst_mint,
        associated_token::token_program = token_program
    )]
    rst_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::authority = pool,
        associated_token::mint = lst_mint,
        associated_token::token_program = token_program
    )]
    vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        mint::token_program = token_program,
        mint::authority = Pubkey::from_str(SOLAYER_RESTAKE_POOL).unwrap(),
        mint::freeze_authority = Pubkey::from_str(SOLAYER_RESTAKE_POOL).unwrap(),
        constraint = ssol_mint.key() == Pubkey::from_str(SOLAYER_SOL_ACCOUNT).unwrap(),
    )]
    ssol_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        associated_token::authority = pool,
        associated_token::mint = ssol_mint,
        associated_token::token_program = token_program,
    )]
    ssol_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::authority = Pubkey::from_str(SOLAYER_RESTAKE_POOL).unwrap(),
        associated_token::mint = lst_mint,
        associated_token::token_program = token_program
    )]
    restaking_pool_lst_vault: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        has_one = lst_mint,
        has_one = rst_mint,
        constraint = pool.lrt_mint == ssol_mint.key(),
        seeds = [b"lrt_pool", pool.lst_mint.key().as_ref(), pool.rst_mint.key().as_ref(), pool.lrt_mint.key().as_ref()],
        bump = pool.bump
    )]
    pool: Box<Account<'info, LRTPool>>,
    #[account(
        address = Pubkey::from_str(SOLAYER_RESTAKE_POOL).unwrap()
    )]
    restaking_pool: AccountInfo<'info>,
    #[account(address = Pubkey::from_str(SOLAYER_RESTAKE_PROGRAM_ID).unwrap())]
    restaking_program: AccountInfo<'info>,
    associated_token_program: Program<'info, AssociatedToken>,
    token_program: Interface<'info, TokenInterface>,
    system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn burn_rst(&mut self, amount: u64) -> Result<()> {
        let ctx = CpiContext::new(
            self.token_program.to_account_info(),
            Burn {
                mint: self.rst_mint.to_account_info(),
                from: self.rst_ata.to_account_info(),
                authority: self.signer.to_account_info(),
            },
        );
        burn(ctx, amount)
    }

    pub fn unstake(&mut self, amount: u64) -> Result<()> {
        // transfer staked sol to user
        self.vault.reload()?;
        if self.vault.amount < amount {
            return Err(LRTPoolError::InsufficientStakedSOLFundsForWithdraw.into());
        }

        let bump = [self.pool.bump];
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"lrt_pool",
            self.lst_mint.to_account_info().key.as_ref(),
            self.rst_mint.to_account_info().key.as_ref(),
            self.ssol_mint.to_account_info().key.as_ref(),
            &bump,
        ][..]];

        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            TransferChecked {
                from: self.vault.to_account_info(),
                to: self.lst_ata.to_account_info(),
                mint: self.lst_mint.to_account_info(),
                authority: self.pool.to_account_info(),
            },
            &signer_seeds,
        );

        transfer_checked(ctx, amount, self.lst_mint.decimals)
    }

    pub fn unrestake(&mut self, amount: u64) -> Result<()> {
        // unrestake sSOL to get back staked sol
        self.ssol_ata.reload()?;
        if self.ssol_ata.amount < amount {
            return Err(LRTPoolError::InsufficientSSOLFundsForWithdraw.into());
        }
        let mut unrestake_data = sighash("global", "unrestake").to_vec();
        unrestake_data.extend_from_slice(&amount.to_le_bytes());

        let accounts = vec![
            // signer
            AccountMeta::new(self.pool.key(), true),
            // lst_mint
            AccountMeta::new(self.lst_mint.key(), false),
            // lst_ata
            AccountMeta::new(self.vault.key(), false),
            // rst_ata
            AccountMeta::new(self.ssol_ata.key(), false),
            // rst_mint
            AccountMeta::new(self.ssol_mint.key(), false),
            // vault
            AccountMeta::new(self.restaking_pool_lst_vault.key(), false),
            // pool
            AccountMeta::new_readonly(self.restaking_pool.key(), false),
            // associated_token_program
            AccountMeta::new_readonly(self.associated_token_program.key(), false),
            // token_program
            AccountMeta::new_readonly(self.token_program.key(), false),
            // system_program
            AccountMeta::new_readonly(self.system_program.key(), false),
        ];

        let restake_inst = Instruction {
            program_id: self.restaking_program.key(),
            data: unrestake_data,
            accounts,
        };

        let bump = [self.pool.bump];
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"lrt_pool",
            self.lst_mint.to_account_info().key.as_ref(),
            self.rst_mint.to_account_info().key.as_ref(),
            self.ssol_mint.to_account_info().key.as_ref(),
            &bump,
        ][..]];

        invoke_signed(
            &restake_inst,
            &[
                self.pool.to_account_info(),
                self.lst_mint.to_account_info(),
                self.vault.to_account_info(),
                self.ssol_ata.to_account_info(),
                self.ssol_mint.to_account_info(),
                self.restaking_pool_lst_vault.to_account_info(),
                self.restaking_pool.to_account_info(),
                self.associated_token_program.to_account_info(),
                self.token_program.to_account_info(),
                self.system_program.to_account_info(),
            ],
            &signer_seeds,
        )
        .map_err(Into::into)
    }
}
