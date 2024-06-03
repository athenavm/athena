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

pub mod disassembler;
pub mod io;
pub mod runtime;
pub mod syscall;
pub mod utils;

#[cfg(test)]
mod tests {
  #[test]
  fn test_load_and_run_elf() {
    use crate::runtime::Runtime;
    use crate::runtime::Program;
    use crate::utils::{AthenaCoreOpts, setup_logger};

    setup_logger();

    let program = Program::from_elf("../examples/hello_world/program/target/riscv32im-succinct-zkvm-elf/release/hello_world");
    let mut runtime = Runtime::new(program.clone(), AthenaCoreOpts::default());
    let _ = runtime.run();

    // Add assertions here to verify the expected behavior
  }
}
