use vergen_git2::{BuildBuilder, Emitter, Git2Builder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let build = BuildBuilder::default().build_timestamp(true).build()?;
  let git2 = Git2Builder::default().sha(true).build()?;

  Emitter::default()
    .add_instructions(&build)?
    .add_instructions(&git2)?
    .emit()
    .unwrap();

  #[cfg(feature = "unittest")]
  build_programs_for_tests();

  Ok(())
}

#[cfg(feature = "unittest")]
fn build_programs_for_tests() {
  athena_builder::build::build_program("../examples/fibonacci/program");
  athena_builder::build::build_program("../tests/panic");
}
