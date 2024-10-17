use athena_helper::build_program;

fn build_programs_for_tests() {
  build_program("../tests/entrypoint");
  build_program("../tests/recursive_call");
}

fn main() {
  #[cfg(feature = "unittest")]
  build_programs_for_tests()
}
