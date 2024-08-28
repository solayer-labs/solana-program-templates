import { PublicKey } from "@solana/web3.js";

// #[cfg(not(feature = "devnet-mode"))]
// pub const SOLAYER_ENDO_AVS_PROGRAM_ID: &str = "endoLNCKTqDn8gSVnN2hDdpgACUPWHZTwoYnnMybpAT";

// #[cfg(feature = "devnet-mode")]
// pub const SOLAYER_ENDO_AVS_PROGRAM_ID: &str = "DM2ReCHeTsV4fAvHsBehZBTps3DVLiK2UW2dHAYrDZrM";

// devnet account
export const SOLAYER_RESTAKE_PROGRAM_ID_DEVNET = new PublicKey(
  "3uZbsFKoxpX8NaRWgkMRebVCofCWoTcJ3whrt4Lvoqn9"
);
export const SOLAYER_SOL_MINT_PUB_KEY_DEVNET = new PublicKey(
  "BQoheepVg6gprtszJFiL59pFVHPa2bu3GBZ6Un7sGGsf"
);
export const LRT_TEMPLATE_PROGRAM_ID_DEVNET = new PublicKey(
  "Be419vzFciNeDWrX61Wwo2pqHWeX1JQVRQrwgoK6Lur2"
);
export const STAKED_SOL_MINT_PUB_KEY_DEVNET = new PublicKey(
  "DaERMQKb2z7FyekFBnSYgLG9YF98AyDNVQS6VCFw8mfE"
);
export const SOLAYER_RESTAKE_POOL_DEVNET = new PublicKey(
  "HukzvthPRkQYYon61o1ZKmwU4pxVL8ahMzTzsmWcEB5F"
);
export const ENDO_AVS_PROGRAM_ID_DEVNET = new PublicKey(
  "DM2ReCHeTsV4fAvHsBehZBTps3DVLiK2UW2dHAYrDZrM"
);
export const ENDO_AVS_DEVNET = new PublicKey(
  "GQouxK6v51z191VRdqAuudhVma7AWiqkGQ5yBWWPysqa"
);
export const ENDO_AVS_TOKEN_MINT_DEVNET = new PublicKey(
  "5RA2wjzePPnk8z9Zy3whTDk4jTbMXgXqWxvCoeh8Fgck"
);

// mainnet account
export const SOLAYER_RESTAKE_PROGRAM_ID_MAINNET = new PublicKey(
  "sSo1iU21jBrU9VaJ8PJib1MtorefUV4fzC9GURa2KNn"
);
export const SOLAYER_SOL_MINT_PUB_KEY_MAINNET = new PublicKey(
  "sSo14endRuUbvQaJS3dq36Q829a3A6BEfoeeRGJywEh"
);
// export const LRT_TEMPLATE_PROGRAM_ID_MAINNET = new PublicKey("");
export const STAKED_SOL_MINT_PUB_KEY_MAINNET = new PublicKey(
  "sSo1wxKKr6zW2hqf5hZrp2CawLibcwi1pMBqk5bg2G4"
);
export const SOLAYER_RESTAKE_POOL_MAINNET = new PublicKey(
  "3sk58CzpitB9jsnVzZWwqeCn2zcXVherhALBh88Uw9GQ"
);
export const ENDO_AVS_PROGRAM_ID_MAINNET = new PublicKey(
  "endoLNCKTqDn8gSVnN2hDdpgACUPWHZTwoYnnMybpAT"
);
