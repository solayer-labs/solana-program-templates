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
    constants::{
        SOLAYER_ENDO_AVS_PROGRAM_ID, SOLAYER_RESTAKE_POOL, SOLAYER_RESTAKE_PROGRAM_ID,
        SOLAYER_SOL_ACCOUNT,
    },
    errors::LRTPoolError,
    state::LRTPool,
    utils::sighash,
};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    signer: Signer<'info>,
    #[account(
        mint::token_program = token_program,
    )]
    lst_mint: InterfaceAccount<'info, Mint>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::authority = signer,
        associated_token::mint = lst_mint,
        associated_token::token_program = token_program,
    )]
    lst_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        mint::token_program = token_program,
        mint::authority = pool,
        mint::freeze_authority = pool,
        mint::decimals = lst_mint.decimals
    )]
    rst_mint: InterfaceAccount<'info, Mint>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::authority = signer,
        associated_token::mint = rst_mint,
        associated_token::token_program = token_program,
    )]
    rst_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::authority = pool,
        associated_token::mint = lst_mint,
        associated_token::token_program = token_program
    )]
    vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        mint::token_program = token_program,
        mint::authority = Pubkey::from_str(SOLAYER_RESTAKE_POOL).unwrap(),
        mint::freeze_authority = Pubkey::from_str(SOLAYER_RESTAKE_POOL).unwrap(),
        constraint = ssol_mint.key() == Pubkey::from_str(SOLAYER_SOL_ACCOUNT).unwrap(),
    )]
    ssol_mint: InterfaceAccount<'info, Mint>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::authority = pool,
        associated_token::mint = ssol_mint,
        associated_token::token_program = token_program,
    )]
    ssol_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::authority = pool,
        associated_token::mint = ssol_mint,
        associated_token::token_program = token_program
    )]
    ssol_vault: InterfaceAccount<'info, TokenAccount>,
    #[account(
        has_one = lst_mint,
        has_one = rst_mint,
        seeds = [b"lrt_pool", pool.rst_mint.key().as_ref()],
        bump = pool.bump
    )]
    pool: Account<'info, LRTPool>,
    associated_token_program: Program<'info, AssociatedToken>,
    token_program: Interface<'info, TokenInterface>,
    system_program: Program<'info, System>,
    // all following fields are optional, only used for withdrw delegated stake
    endo_avs: Option<UncheckedAccount<'info>>,
    #[account(
        mut,
        mint::authority = endo_avs,
        mint::freeze_authority = endo_avs,
        mint::token_program = token_program
    )]
    pub avs_token_mint: Option<Box<InterfaceAccount<'info, Mint>>>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = delegated_token_mint,
        associated_token::authority = endo_avs,
        associated_token::token_program = token_program
    )]
    pub delegated_token_vault: Option<Box<InterfaceAccount<'info, TokenAccount>>>,
    #[account(
        mint::token_program = token_program,
    )]
    pub delegated_token_mint: Option<Box<InterfaceAccount<'info, Mint>>>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = delegated_token_mint,
        associated_token::authority = pool,
        associated_token::token_program = token_program
    )]
    pub pool_delegated_token_account: Option<Box<InterfaceAccount<'info, TokenAccount>>>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::authority = pool,
        associated_token::mint = avs_token_mint,
        associated_token::token_program = token_program
    )]
    pub pool_avs_token_account: Option<Box<InterfaceAccount<'info, TokenAccount>>>,
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
        self.rst_mint.to_account_info().key.as_ref(),
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
    // unstake sSOL to get back staked sol
    self.ssol_vault.reload()?;
    if self.ssol_vault.amount < amount {
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
        AccountMeta::new(self.ssol_vault.key(), false),
        // pool
        AccountMeta::new_readonly(Pubkey::from_str(SOLAYER_RESTAKE_POOL).unwrap(), false),
        // associated_token_program
        AccountMeta::new_readonly(self.associated_token_program.key(), false),
        // token_program
        AccountMeta::new_readonly(self.token_program.key(), false),
        // system_program
        AccountMeta::new_readonly(self.system_program.key(), false),
    ];

    let restake_inst = Instruction {
        program_id: Pubkey::from_str(SOLAYER_RESTAKE_PROGRAM_ID).unwrap(),
        data: unrestake_data,
        accounts,
    };

    let bump = [self.pool.bump];
    let signer_seeds: [&[&[u8]]; 1] = [&[
        b"lrt_pool",
        self.rst_mint.to_account_info().key.as_ref(),
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
            self.ssol_vault.to_account_info(),
            self.pool.to_account_info(),
            self.associated_token_program.to_account_info(),
            self.token_program.to_account_info(),
            self.system_program.to_account_info(),
        ],
        &signer_seeds,
    )
    .map_err(Into::into)
}

pub fn undelegate(&mut self, amount: u64) -> Result<()> {
    if self.endo_avs.is_none()
        || self.avs_token_mint.is_none()
        || self.delegated_token_vault.is_none()
        || self.delegated_token_mint.is_none()
        || self.pool_delegated_token_account.is_none()
        || self.pool_avs_token_account.is_none()
    {
        return Err(LRTPoolError::MissingAccounts.into());
    }

    let endo_avs = self.endo_avs.clone().unwrap();
    let avs_token_mint = self.avs_token_mint.clone().unwrap();
    let mut delegated_token_vault = self.delegated_token_vault.clone().unwrap();
    let delegated_token_mint = self.delegated_token_mint.clone().unwrap();
    let pool_delegated_token_account = self.pool_delegated_token_account.clone().unwrap();
    let pool_avs_token_account = self.pool_avs_token_account.clone().unwrap();

    delegated_token_vault.reload()?;
    if delegated_token_vault.amount < amount {
        return Err(LRTPoolError::InsufficientSSOLFundsForDelegate.into());
    }

    let mut undelegate_data = sighash("global", "undelegate").to_vec();
    undelegate_data.extend_from_slice(&amount.to_le_bytes());

    let accounts = vec![
        // staker
        AccountMeta::new(self.pool.key(), true),
        // endoAvs
        AccountMeta::new(endo_avs.key(), false),
        // avsTokenMint
        AccountMeta::new(avs_token_mint.key(), false),
        // delegatedTokenVault
        AccountMeta::new(delegated_token_vault.key(), false),
        // delegatedTokenMint
        AccountMeta::new(delegated_token_mint.key(), false),
        // stakerDelegatedTokenAccount,
        AccountMeta::new(pool_delegated_token_account.key(), false),
        // stakerAvsTokenAccount
        AccountMeta::new(pool_avs_token_account.key(), false),
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
            endo_avs.to_account_info(),
            avs_token_mint.to_account_info(),
            delegated_token_vault.to_account_info(),
            delegated_token_mint.to_account_info(),
            pool_delegated_token_account.to_account_info(),
            pool_avs_token_account.to_account_info(),
            self.token_program.to_account_info(),
            self.associated_token_program.to_account_info(),
            self.system_program.to_account_info(),
        ],
        &signer_seeds,
    )
    .map_err(Into::into)
}
}
