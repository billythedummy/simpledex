#![cfg_attr(not(test), forbid(unsafe_code))]

// These 2 mods need to come first for macro defns
pub mod pda;

pub mod checks;

pub mod account;
pub mod error;
pub mod fee;
pub mod instructions;
pub mod packun;
pub mod processor;
pub mod state;
pub mod types;

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

solana_program::declare_id!("S1MPKsrdak3rGKeHMpbfcBm6VhJTKekBzbQH9g1b9hy");
