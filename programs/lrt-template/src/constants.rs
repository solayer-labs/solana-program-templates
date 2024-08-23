// pub const SOLAYER_STAKE_POOL_PUBKEY: &str = "po1osKDWYF9oiVEGmzKA4eTs8eMveFRMox3bUKazGN2";
// pub const SOLAYER_STAKE_POOL_TOKEN_MINT: &str = "sSo1wxKKr6zW2hqf5hZrp2CawLibcwi1pMBqk5bg2G4";
// pub const SOLAYER_STAKE_POOL_WITHDRAW_AUTHORITY: &str =
    // "H5rmot8ejBUWzMPt6E44h27xj5obbSz3jVuK4AsJpHmv";
// pub const SOLAYER_STAKE_POOL_RESERVE_STAKE_ACCOUNT: &str =
    // "Brh9rB6npnjM1vDXyCXtzkXVGRnsh6KHqmBz26tVACg9";
// pub const SOLAYER_STAKE_POOL_FEE_ACCOUNT: &str = "ARs3HTD79nsaUdDKqfGhgbNMVJkXVdRs2EpHAm4LNEcq";
#[cfg(not(feature = "devnet-mode"))]
pub const SOLAYER_RESTAKE_PROGRAM_ID: &str = "sSo1iU21jBrU9VaJ8PJib1MtorefUV4fzC9GURa2KNn";
#[cfg(not(feature = "devnet-mode"))]
pub const SOLAYER_SOL_ACCOUNT: &str = "sSo14endRuUbvQaJS3dq36Q829a3A6BEfoeeRGJywEh";
#[cfg(not(feature = "devnet-mode"))]
pub const SOLAYER_RESTAKE_POOL: &str = "3sk58CzpitB9jsnVzZWwqeCn2zcXVherhALBh88Uw9GQ";
#[cfg(not(feature = "devnet-mode"))]
pub const SOLAYER_ENDO_AVS_PROGRAM_ID: &str = "endoLNCKTqDn8gSVnN2hDdpgACUPWHZTwoYnnMybpAT";

#[cfg(feature = "devnet-mode")]
pub const SOLAYER_RESTAKE_PROGRAM_ID: &str = "3uZbsFKoxpX8NaRWgkMRebVCofCWoTcJ3whrt4Lvoqn9";
#[cfg(feature = "devnet-mode")]
pub const SOLAYER_SOL_ACCOUNT: &str = "BQoheepVg6gprtszJFiL59pFVHPa2bu3GBZ6Un7sGGsf";
#[cfg(feature = "devnet-mode")]
pub const SOLAYER_RESTAKE_POOL: &str = "HukzvthPRkQYYon61o1ZKmwU4pxVL8ahMzTzsmWcEB5F";
#[cfg(feature = "devnet-mode")]
pub const SOLAYER_ENDO_AVS_PROGRAM_ID: &str = "";
