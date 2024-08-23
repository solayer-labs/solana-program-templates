use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use solana_program::{instruction::Instruction, program::invoke_signed};
use std::str::FromStr;

use crate::{
    constants::SOLAYER_ENDO_AVS_PROGRAM_ID, errors::LRTPoolError, state::LRTPool, utils::sighash,
};

#[derive(Accounts)]
pub struct Delegate<'info> {
    #[account(mut)]
    signer: Signer<'info>,

    endo_avs: AccountInfo<'info>,
    #[account(
        mut,
        mint::decimals = delegated_token_mint.decimals,
        mint::authority = endo_avs,
        mint::freeze_authority = endo_avs,
        mint::token_program = token_program
    )]
    pub avs_token_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = delegated_token_mint,
        associated_token::authority = endo_avs,
        associated_token::token_program = token_program
    )]
    pub delegated_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mint::token_program = token_program,
    )]
    pub delegated_token_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = delegated_token_mint,
        associated_token::authority = pool,
        associated_token::token_program = token_program
    )]
    pub pool_delegated_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::authority = pool,
        associated_token::mint = avs_token_mint,
        associated_token::token_program = token_program
    )]
    pub pool_avs_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(seeds = [b"lrt_pool", pool.rst_mint.key().as_ref()], bump = pool.bump,
    constraint = pool.delegate_authority == signer.key())]
    pool: Account<'info, LRTPool>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Delegate<'info> {
    pub fn delegate(&mut self, amount: u64) -> Result<()> {
        self.delegated_token_vault.reload()?;
        if self.delegated_token_vault.amount < amount {
            return Err(LRTPoolError::InsufficientSSOLFundsForDelegate.into());
        }

        let mut delegate_data = sighash("global", "delegate").to_vec();
        delegate_data.extend_from_slice(&amount.to_le_bytes());

        let accounts = vec![
            // staker
            AccountMeta::new(self.pool.key(), true),
            // endoAvs
            AccountMeta::new(self.endo_avs.key(), false),
            // avsTokenMint
            AccountMeta::new(self.avs_token_mint.key(), false),
            // delegatedTokenVault
            AccountMeta::new(self.delegated_token_vault.key(), false),
            // delegatedTokenMint
            AccountMeta::new(self.delegated_token_mint.key(), false),
            // stakerDelegatedTokenAccount,
            AccountMeta::new(self.pool_delegated_token_account.key(), false),
            // stakerAvsTokenAccount
            AccountMeta::new(self.pool_avs_token_account.key(), false),
            // tokenProgram
            AccountMeta::new_readonly(self.token_program.key(), false),
            // associatedTokenProgram
            AccountMeta::new_readonly(self.associated_token_program.key(), false),
            // systemProgram
            AccountMeta::new_readonly(self.system_program.key(), false),
        ];

        let delegate_inst = Instruction {
            program_id: Pubkey::from_str(SOLAYER_ENDO_AVS_PROGRAM_ID).unwrap(),
            data: delegate_data,
            accounts,
        };

        let bump = [self.pool.bump];
        let rst_mint = self.pool.rst_mint.key();
        let signer_seeds: [&[&[u8]]; 1] = [&[b"lrt_pool", rst_mint.as_ref(), &bump][..]];

        invoke_signed(
            &delegate_inst,
            &[
                self.pool.to_account_info(),
                self.endo_avs.to_account_info(),
                self.avs_token_mint.to_account_info(),
                self.delegated_token_vault.to_account_info(),
                self.delegated_token_mint.to_account_info(),
                self.pool_delegated_token_account.to_account_info(),
                self.pool_avs_token_account.to_account_info(),
                self.token_program.to_account_info(),
                self.associated_token_program.to_account_info(),
                self.system_program.to_account_info(),
            ],
            &signer_seeds,
        )
        .map_err(Into::into)
    }

    pub fn undelegate(&mut self, amount: u64) -> Result<()> {
        self.delegated_token_vault.reload()?;
        if self.delegated_token_vault.amount < amount {
            return Err(LRTPoolError::InsufficientSSOLFundsForDelegate.into());
        }

        let mut undelegate_data = sighash("global", "undelegate").to_vec();
        undelegate_data.extend_from_slice(&amount.to_le_bytes());

        let accounts = vec![
            // staker
            AccountMeta::new(self.pool.key(), true),
            // endoAvs
            AccountMeta::new(self.endo_avs.key(), false),
            // avsTokenMint
            AccountMeta::new(self.avs_token_mint.key(), false),
            // delegatedTokenVault
            AccountMeta::new(self.delegated_token_vault.key(), false),
            // delegatedTokenMint
            AccountMeta::new(self.delegated_token_mint.key(), false),
            // stakerDelegatedTokenAccount,
            AccountMeta::new(self.pool_delegated_token_account.key(), false),
            // stakerAvsTokenAccount
            AccountMeta::new(self.pool_avs_token_account.key(), false),
            // tokenProgram
            AccountMeta::new_readonly(self.token_program.key(), false),
            // associatedTokenProgram
            AccountMeta::new_readonly(self.associated_token_program.key(), false),
            // systemProgram
            AccountMeta::new_readonly(self.system_program.key(), false),
        ];

        let delegate_inst = Instruction {
            program_id: Pubkey::from_str(SOLAYER_ENDO_AVS_PROGRAM_ID).unwrap(),
            data: undelegate_data,
            accounts,
        };

        let bump = [self.pool.bump];
        let rst_mint = self.pool.rst_mint.key();
        let signer_seeds: [&[&[u8]]; 1] = [&[b"lrt_pool", rst_mint.as_ref(), &bump][..]];

        invoke_signed(
            &delegate_inst,
            &[
                self.pool.to_account_info(),
                self.endo_avs.to_account_info(),
                self.avs_token_mint.to_account_info(),
                self.delegated_token_vault.to_account_info(),
                self.delegated_token_mint.to_account_info(),
                self.pool_delegated_token_account.to_account_info(),
                self.pool_avs_token_account.to_account_info(),
                self.token_program.to_account_info(),
                self.associated_token_program.to_account_info(),
                self.system_program.to_account_info(),
            ],
            &signer_seeds,
        )
        .map_err(Into::into)
    }
}
