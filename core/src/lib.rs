#![allow(
    clippy::new_without_default,
    clippy::field_reassign_with_default,
    clippy::unnecessary_cast,
    clippy::cast_abs_to_unsigned,
    clippy::needless_range_loop,
    clippy::type_complexity,
    clippy::unnecessary_unwrap,
    clippy::default_constructed_unit_structs,
    clippy::box_default,
    deprecated,
    incomplete_features
)]
#![feature(generic_const_exprs)]
#![warn(unused_extern_crates)]

extern crate alloc;

pub mod disassembler;
pub mod io;
pub mod runtime;
pub mod syscall;
pub mod utils;

#[allow(unused_imports)]
use crate::runtime::{Program, Runtime};

#[cfg(test)]
mod tests {
  #[test]
  fn test_load_and_run_elf() {
    let program = Program::from_elf("../examples/hello_world/target/riscv32im-succinct-zkvm-elf/release/test_program");
    let mut runtime = Runtime::new(program.clone(), AthenaCoreOpts::default());
    runtime.run()?;

    // Add assertions here to verify the expected behavior
  }
}
