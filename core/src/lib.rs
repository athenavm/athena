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
#![warn(unused_extern_crates)]

pub mod disassembler;
pub mod host;
mod instruction;
pub mod io;
pub mod runtime;
pub mod syscall;
pub mod utils;
