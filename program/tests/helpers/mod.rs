#![allow(dead_code)]

mod simpledex_helpers;
mod spl_token_helpers;
mod sys_helpers;

pub use simpledex_helpers::*;
pub use spl_token_helpers::*;
pub use sys_helpers::*;

use simpledex::processor;
use solana_program_test::{processor, ProgramTest};

pub fn program_test() -> ProgramTest {
    ProgramTest::new(
        "simpledex",
        simpledex::id(),
        processor!(processor::Processor::process),
    )
}
