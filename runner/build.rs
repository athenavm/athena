#[cfg(feature = "unittest")]
fn build_programs_for_tests() {
  use athena_helper::build_program;
  build_program("../tests/entrypoint");
  build_program("../tests/recursive_call");
}

fn main() {
  #[cfg(feature = "unittest")]
  build_programs_for_tests()
}
