use cargo_metadata::Metadata;
use std::path::Path;

use crate::BuildArgs;

/// Re-run the cargo command if the Cargo.toml or Cargo.lock file changes.
fn cargo_rerun_if_changed(metadata: &Metadata, program_dir: &Path) {
  // Tell cargo to rerun the script only if program/{src, bin, build.rs, Cargo.toml} changes
  // Ref: https://doc.rust-lang.org/nightly/cargo/reference/build-scripts.html#rerun-if-changed
  let dirs = vec![
    program_dir.join("src"),
    program_dir.join("bin"),
    program_dir.join("build.rs"),
    program_dir.join("Cargo.toml"),
  ];
  for dir in dirs {
    if dir.exists() {
      println!(
        "cargo::rerun-if-changed={}",
        dir.canonicalize().unwrap().display()
      );
    }
  }

  // Re-run the build script if the workspace root's Cargo.lock changes. If the program is its own
  // workspace, this will be the program's Cargo.lock.
  println!(
    "cargo:rerun-if-changed={}",
    metadata.workspace_root.join("Cargo.lock")
  );

  // Re-run if any local dependency changes.
  for package in &metadata.packages {
    for dependency in &package.dependencies {
      if let Some(path) = &dependency.path {
        println!("cargo:rerun-if-changed={}", path);
      }
    }
  }
}

/// Executes the `cargo athena build` command in the program directory. If there are any cargo athena
/// build arguments, they are added to the command.
fn execute_build_cmd(program_dir: &impl AsRef<std::path::Path>, args: Option<BuildArgs>) {
  // Check if RUSTC_WORKSPACE_WRAPPER is set to clippy-driver (i.e. if `cargo clippy` is the current
  // compiler). If so, don't execute `cargo athena build` because it breaks rust-analyzer's `cargo clippy` feature.
  let is_clippy_driver = std::env::var("RUSTC_WORKSPACE_WRAPPER")
    .map(|val| val.contains("clippy-driver"))
    .unwrap_or(false);
  if is_clippy_driver {
    println!("cargo:warning=Skipping build due to clippy invocation.");
    return;
  }

  crate::build_program(
    &args.unwrap_or_default(),
    Some(program_dir.as_ref().to_path_buf()),
  )
  .expect("building Athena program");
}

/// Builds the program if the program at the specified path, or one of its dependencies, changes.
///
/// This function monitors the program and its dependencies for changes. If any changes are detected,
/// it triggers a rebuild of the program.
///
/// # Arguments
///
/// * `path` - A string slice that holds the path to the program directory.
///
/// This function is useful for automatically rebuilding the program during development
/// when changes are made to the source code or its dependencies.
///
/// Set the `ATHENA_SKIP_PROGRAM_BUILD` environment variable to `true` to skip building the program.
pub fn build_program(path: &str) {
  build_program_internal(path, None)
}

/// Builds the program with the given arguments if the program at path, or one of its dependencies,
/// changes.
///
/// # Arguments
///
/// * `path` - A string slice that holds the path to the program directory.
/// * `args` - A [`BuildArgs`] struct that contains various build configuration options.
///
/// Set the `ATHENA_SKIP_PROGRAM_BUILD` environment variable to `true` to skip building the program.
pub fn build_program_with_args(path: &str, args: BuildArgs) {
  build_program_internal(path, Some(args))
}

/// Internal helper function to build the program with or without arguments.
fn build_program_internal(path: &str, args: Option<BuildArgs>) {
  // Get the root package name and metadata.
  let program_dir = Path::new(path);
  let metadata_file = program_dir.join("Cargo.toml");
  let mut metadata_cmd = cargo_metadata::MetadataCommand::new();
  let metadata = metadata_cmd.manifest_path(metadata_file).exec().unwrap();
  let root_package = metadata.root_package();
  let root_package_name = root_package.map(|p| p.name.as_str()).unwrap_or("Program");

  // Skip the program build if the ATHENA_SKIP_PROGRAM_BUILD environment variable is set to true.
  let skip_program_build = std::env::var("ATHENA_SKIP_PROGRAM_BUILD")
    .map(|v| v.eq_ignore_ascii_case("true"))
    .unwrap_or(false);
  if skip_program_build {
    println!(
      "cargo:warning=Skipping building {root_package_name} due to ATHENA_SKIP_PROGRAM_BUILD flag",
    );
    return;
  }

  // Activate the build command if the dependencies change.
  cargo_rerun_if_changed(&metadata, program_dir);

  execute_build_cmd(&program_dir, args);
  println!("cargo:warning={root_package_name} built successfully");
}
