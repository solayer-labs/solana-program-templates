use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        burn, transfer_checked, Burn, Mint, TokenAccount, TokenInterface, TransferChecked,
    },
};
use solana_program::{instruction::Instruction, program::invoke_signed};
use std::str::FromStr;

use crate::{
    constants::{SOLAYER_ENDO_AVS_PROGRAM_ID, SOLAYER_RESTAKE_POOL, SOLAYER_SOL_ACCOUNT},
    errors::LRTPoolError,
    state::LRTPool,
    utils::sighash,
};

#[derive(Accounts)]
pub struct WithdrawStake<'info> {
    #[account(mut)]
    signer: Signer<'info>,
    #[account(
        mut,
        mint::token_program = token_program,
        mint::authority = pool,
        mint::freeze_authority = pool
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
        init_if_needed,
        payer = signer,
        associated_token::authority = signer,
        associated_token::mint = ssol_mint,
        associated_token::token_program = token_program,
    )]
    signer_ssol_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        has_one = rst_mint,
        constraint = pool.lrt_mint == ssol_mint.key(),
        seeds = [b"lrt_pool", pool.lst_mint.key().as_ref(), pool.rst_mint.key().as_ref(), pool.lrt_mint.key().as_ref()],
        bump = pool.bump
    )]
    pool: Box<Account<'info, LRTPool>>,
    #[account(mut)]
    endo_avs: AccountInfo<'info>,
    #[account(
        mut,
        mint::authority = endo_avs,
        mint::freeze_authority = endo_avs,
        mint::decimals = ssol_mint.decimals,
        mint::token_program = token_program
    )]
    avs_token_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        associated_token::mint = ssol_mint,
        associated_token::authority = endo_avs,
        associated_token::token_program = token_program
    )]
    delegated_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::authority = pool,
        associated_token::mint = avs_token_mint,
        associated_token::token_program = token_program
    )]
    pool_avs_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(address = Pubkey::from_str(SOLAYER_ENDO_AVS_PROGRAM_ID).unwrap())]
    endo_avs_program: AccountInfo<'info>,
    associated_token_program: Program<'info, AssociatedToken>,
    token_program: Interface<'info, TokenInterface>,
    system_program: Program<'info, System>,
}

impl<'info> WithdrawStake<'info> {
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

    pub fn undelegate(&mut self, amount: u64) -> Result<()> {
        self.pool_avs_token_account.reload()?;
        if self.pool_avs_token_account.amount < amount {
            return Err(LRTPoolError::InsufficientAvsTokenForUndelegate.into());
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
            AccountMeta::new(self.ssol_mint.key(), false),
            // stakerDelegatedTokenAccount,
            AccountMeta::new(self.ssol_ata.key(), false),
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
            program_id: self.endo_avs_program.key(),
            data: undelegate_data,
            accounts,
        };

        let bump = [self.pool.bump];
        let lst_mint = self.pool.lst_mint.key();
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"lrt_pool",
            lst_mint.as_ref(),
            self.rst_mint.to_account_info().key.as_ref(),
            self.ssol_mint.to_account_info().key.as_ref(),
            &bump,
        ][..]];

        invoke_signed(
            &delegate_inst,
            &[
                self.pool.to_account_info(),
                self.endo_avs.to_account_info(),
                self.avs_token_mint.to_account_info(),
                self.delegated_token_vault.to_account_info(),
                self.ssol_mint.to_account_info(),
                self.ssol_ata.to_account_info(),
                self.pool_avs_token_account.to_account_info(),
                self.token_program.to_account_info(),
                self.associated_token_program.to_account_info(),
                self.system_program.to_account_info(),
            ],
            &signer_seeds,
        )
        .map_err(Into::into)
    }

    pub fn transfer_ssol(&mut self, amount: u64) -> Result<()> {
        self.ssol_ata.reload()?;
        if self.ssol_ata.amount < amount {
            return Err(LRTPoolError::InsufficientSSOLFundsForWithdraw.into());
        }

        let bump = [self.pool.bump];
        let lst_mint = self.pool.lst_mint.key();
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"lrt_pool",
            lst_mint.as_ref(),
            self.rst_mint.to_account_info().key.as_ref(),
            self.ssol_mint.to_account_info().key.as_ref(),
            &bump,
        ][..]];

        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            TransferChecked {
                from: self.ssol_ata.to_account_info(),
                to: self.signer_ssol_ata.to_account_info(),
                mint: self.ssol_mint.to_account_info(),
                authority: self.pool.to_account_info(),
            },
            &signer_seeds,
        );

        transfer_checked(ctx, amount, self.ssol_mint.decimals)
    }
}
