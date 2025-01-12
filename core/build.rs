#[cfg(feature = "unittest")]
fn build_programs_for_tests() {
  use athena_builder::build::build_program;

  build_program("../examples/io/program");
  build_program("../tests/panic");
}

fn main() {
  #[cfg(feature = "unittest")]
  build_programs_for_tests()
}
