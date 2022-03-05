#![cfg_attr(not(test), forbid(unsafe_code))]

pub mod error;
pub mod processor;

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

solana_program::declare_id!("69DoKc37LTpJvzSf4vk5QUTzLux3ZSnR3YvuAZoLU4Mx");
