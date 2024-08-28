use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use solana_program::{instruction::Instruction, program::invoke_signed};

use crate::{errors::LRTPoolError, state::LRTPool, utils::sighash};

#[derive(Accounts)]
pub struct Delegate<'info> {
    #[account(mut)]
    signer: Signer<'info>,

    #[account(mut)]
    avs: AccountInfo<'info>,
    #[account(
        mut,
        mint::decimals = input_token_mint.decimals,
        mint::authority = avs,
        mint::freeze_authority = avs
    )]
    avs_token_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        associated_token::mint = input_token_mint,
        associated_token::authority = avs
    )]
    avs_input_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        mint::token_program = token_program,
        address = pool.input_token_mint
    )]
    input_token_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        associated_token::mint = input_token_mint,
        associated_token::authority = pool,
        associated_token::token_program = token_program
    )]
    pool_input_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::authority = pool,
        associated_token::mint = avs_token_mint,
        associated_token::token_program = token_program
    )]
    pool_avs_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        has_one = input_token_mint,
        seeds = [b"lrt_pool", pool.output_token_mint.key().as_ref()],
        bump = pool.bump,
        constraint = pool.delegate_authority == signer.key()
    )]
    pool: Account<'info, LRTPool>,
    avs_program: AccountInfo<'info>,
    token_program: Interface<'info, TokenInterface>,
    associated_token_program: Program<'info, AssociatedToken>,
    system_program: Program<'info, System>,
}

impl<'info> Delegate<'info> {
    // this delegate method is specific to solayer endo avs program for now
    pub fn delegate(&mut self, amount: u64) -> Result<()> {
        self.pool_input_token_vault.reload()?;
        if self.pool_input_token_vault.amount < amount {
            return Err(LRTPoolError::InsufficientSSOLFundsForDelegate.into());
        }

        let mut delegate_data = sighash("global", "delegate").to_vec();
        delegate_data.extend_from_slice(&amount.to_le_bytes());

        let accounts = vec![
            // staker
            AccountMeta::new(self.pool.key(), true),
            // avs
            AccountMeta::new(self.avs.key(), false),
            // avsTokenMint
            AccountMeta::new(self.avs_token_mint.key(), false),
            // delegatedTokenVault
            AccountMeta::new(self.avs_input_token_vault.key(), false),
            // delegatedTokenMint
            AccountMeta::new(self.input_token_mint.key(), false),
            // stakerDelegatedTokenAccount,
            AccountMeta::new(self.pool_input_token_vault.key(), false),
            // stakerAvsTokenAccount
            AccountMeta::new(self.pool_avs_token_vault.key(), false),
            // tokenProgram
            AccountMeta::new_readonly(self.token_program.key(), false),
            // associatedTokenProgram
            AccountMeta::new_readonly(self.associated_token_program.key(), false),
            // systemProgram
            AccountMeta::new_readonly(self.system_program.key(), false),
        ];

        let delegate_inst = Instruction {
            program_id: self.avs_program.key(),
            data: delegate_data,
            accounts,
        };

        let bump = [self.pool.bump];
        let output_token_mint = self.pool.output_token_mint.key();
        let signer_seeds: [&[&[u8]]; 1] = [&[b"lrt_pool", output_token_mint.as_ref(), &bump][..]];

        invoke_signed(
            &delegate_inst,
            &[
                self.pool.to_account_info(),
                self.avs.to_account_info(),
                self.avs_token_mint.to_account_info(),
                self.avs_input_token_vault.to_account_info(),
                self.input_token_mint.to_account_info(),
                self.pool_input_token_vault.to_account_info(),
                self.pool_avs_token_vault.to_account_info(),
                self.token_program.to_account_info(),
                self.associated_token_program.to_account_info(),
                self.system_program.to_account_info(),
            ],
            &signer_seeds,
        )
        .map_err(Into::into)
    }

    pub fn undelegate(&mut self, amount: u64) -> Result<()> {
        self.pool_avs_token_vault.reload()?;
        if self.pool_avs_token_vault.amount < amount {
            return Err(LRTPoolError::InsufficientAvsTokenForUndelegate.into());
        }

        let mut undelegate_data = sighash("global", "undelegate").to_vec();
        undelegate_data.extend_from_slice(&amount.to_le_bytes());

        let accounts = vec![
            // staker
            AccountMeta::new(self.pool.key(), true),
            // endoAvs
            AccountMeta::new(self.avs.key(), false),
            // avsTokenMint
            AccountMeta::new(self.avs_token_mint.key(), false),
            // delegatedTokenVault
            AccountMeta::new(self.avs_input_token_vault.key(), false),
            // delegatedTokenMint
            AccountMeta::new(self.input_token_mint.key(), false),
            // stakerDelegatedTokenAccount,
            AccountMeta::new(self.pool_input_token_vault.key(), false),
            // stakerAvsTokenAccount
            AccountMeta::new(self.pool_avs_token_vault.key(), false),
            // tokenProgram
            AccountMeta::new_readonly(self.token_program.key(), false),
            // associatedTokenProgram
            AccountMeta::new_readonly(self.associated_token_program.key(), false),
            // systemProgram
            AccountMeta::new_readonly(self.system_program.key(), false),
        ];

        let delegate_inst = Instruction {
            program_id: self.avs_program.key(),
            data: undelegate_data,
            accounts,
        };

        let bump = [self.pool.bump];
        let output_token_mint = self.pool.output_token_mint.key();
        let signer_seeds: [&[&[u8]]; 1] = [&[b"lrt_pool", output_token_mint.as_ref(), &bump][..]];

        invoke_signed(
            &delegate_inst,
            &[
                self.pool.to_account_info(),
                self.avs.to_account_info(),
                self.avs_token_mint.to_account_info(),
                self.avs_input_token_vault.to_account_info(),
                self.input_token_mint.to_account_info(),
                self.pool_input_token_vault.to_account_info(),
                self.pool_avs_token_vault.to_account_info(),
                self.token_program.to_account_info(),
                self.associated_token_program.to_account_info(),
                self.system_program.to_account_info(),
            ],
            &signer_seeds,
        )
        .map_err(Into::into)
    }
}
