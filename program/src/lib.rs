#![cfg_attr(not(test), forbid(unsafe_code))]

// needs to come first for macro defns
mod account;
mod checks;
pub mod error;
pub mod fee;
pub mod instructions;
pub mod packun;
pub mod pda;
pub mod processor;
pub mod state;
pub mod types;

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

solana_program::declare_id!("69DoKc37LTpJvzSf4vk5QUTzLux3ZSnR3YvuAZoLU4Mx");
