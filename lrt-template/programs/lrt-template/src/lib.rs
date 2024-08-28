use anchor_lang::prelude::*;
use contexts::*;

mod contexts;
mod errors;
mod state;
mod utils;

declare_id!("Be419vzFciNeDWrX61Wwo2pqHWeX1JQVRQrwgoK6Lur2");

#[program]
pub mod lrt_template {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.initialize(ctx.bumps)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        // transfer input token into the pool
        ctx.accounts.stake(amount)?;
        // mint RST token
        ctx.accounts.mint_rst(amount)?;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        // burn output token from user first
        ctx.accounts.burn_output_token(amount)?;
        // transfer input token back to user's vault
        ctx.accounts.unstake(amount)?;
        Ok(())
    }

    // user can always withdraw stake to get sSol back even if there is no sSol liquidity in the pool
    pub fn withdraw_delegated_stake(ctx: Context<WithdrawStake>, amount: u64) -> Result<()> {
        // burn output token from user first
        ctx.accounts.burn_output_token(amount)?;
        // undelegate avs token
        ctx.accounts.undelegate(amount)?;
        // transfer input token back to user's vault
        ctx.accounts.unstake(amount)?;
        Ok(())
    }

    pub fn transfer_delegate_authority(ctx: Context<TransferDelegateAuthority>) -> Result<()> {
        ctx.accounts.transfer_authority()?;
        Ok(())
    }

    pub fn delegate(ctx: Context<Delegate>, amount: u64) -> Result<()> {
        ctx.accounts.delegate(amount)?;
        Ok(())
    }

    pub fn undelegate(ctx: Context<Delegate>, amount: u64) -> Result<()> {
        ctx.accounts.undelegate(amount)?;
        Ok(())
    }
}
