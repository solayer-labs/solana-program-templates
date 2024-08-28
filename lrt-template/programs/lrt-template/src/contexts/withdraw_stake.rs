use crate::{errors::LRTPoolError, state::LRTPool, utils::sighash};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        burn, transfer_checked, Burn, Mint, TokenAccount, TokenInterface, TransferChecked,
    },
};
use solana_program::{instruction::Instruction, program::invoke_signed};

#[derive(Accounts)]
pub struct WithdrawStake<'info> {
    #[account(mut)]
    signer: Signer<'info>,
    #[account(
        mut,
        mint::token_program = token_program,
        address = pool.input_token_mint
    )]
    input_token_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::authority = signer,
        associated_token::mint = input_token_mint,
        associated_token::token_program = token_program,
    )]
    signer_input_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::authority = pool,
        associated_token::mint = input_token_mint,
        associated_token::token_program = token_program,
    )]
    pool_input_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        mint::token_program = token_program,
        mint::authority = pool,
        mint::freeze_authority = pool,
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
        has_one = output_token_mint,
        seeds = [b"lrt_pool", output_token_mint.key().as_ref()],
        bump = pool.bump
    )]
    pool: Box<Account<'info, LRTPool>>,
    #[account(mut)]
    avs: AccountInfo<'info>,
    #[account(
        mut,
        mint::authority = avs,
        mint::freeze_authority = avs,
        mint::decimals = input_token_mint.decimals,
        mint::token_program = token_program
    )]
    avs_token_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        associated_token::mint = input_token_mint,
        associated_token::authority = avs,
        associated_token::token_program = token_program
    )]
    avs_input_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::authority = pool,
        associated_token::mint = avs_token_mint,
        associated_token::token_program = token_program
    )]
    pool_avs_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    avs_program: AccountInfo<'info>,
    associated_token_program: Program<'info, AssociatedToken>,
    token_program: Interface<'info, TokenInterface>,
    system_program: Program<'info, System>,
}

impl<'info> WithdrawStake<'info> {
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

    // this undelegate method is specific to solayer endo avs program for now
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
            data: undelegate_data,
            accounts,
        };

        let bump = [self.pool.bump];
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"lrt_pool",
            self.output_token_mint.to_account_info().key.as_ref(),
            &bump,
        ][..]];

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

    pub fn unstake(&mut self, amount: u64) -> Result<()> {
        self.pool_input_token_vault.reload()?;
        if self.pool_input_token_vault.amount < amount {
            return Err(LRTPoolError::InsufficientSSOLFundsForWithdraw.into());
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
}
