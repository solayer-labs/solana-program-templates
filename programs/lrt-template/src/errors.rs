use anchor_lang::prelude::*;

#[error_code]
pub enum LRTPoolError {
    #[msg("The RST mint supply must be zero during initialization")]
    NonZeroRstMintSupply,

    #[msg("Insufficient sSOL funds for withdraw")]
    InsufficientSSOLFundsForWithdraw,

    #[msg("Insufficient Staked SOL funds for withdraw")]
    InsufficientStakedSOLFundsForWithdraw,

    #[msg("Insufficient sSOL funds for delegate")]
    InsufficientSSOLFundsForDelegate,

    #[msg("Insufficient Staked SOL funds for delegate")]
    InsufficientAvsTokenForUndelegate,

    #[msg("Missing necessary accounts")]
    MissingAccounts
}
