use anyhow::Result;
use vergen_git2::{BuildBuilder, Emitter, Git2Builder};

fn main() -> Result<()> {
  let build = BuildBuilder::default().build_timestamp(true).build()?;
  let git2 = Git2Builder::default().sha(true).build()?;

  Emitter::default()
    .add_instructions(&build)?
    .add_instructions(&git2)?
    .emit()?;
  Ok(())
}
