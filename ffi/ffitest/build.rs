use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
  // Define the path to the library's source code
  let lib_src_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("../vmlib");

  // Define the output directory within OUT_DIR
  let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

  // Compile the library
  Command::new("cargo")
    .args(&["build", "--release"])
    .current_dir(lib_src_path)
    .status()
    .expect("Failed to compile the library");

  // Tell cargo to link the compiled library
  println!(
    "cargo:rustc-link-search=native={}",
    out_dir.join("release").display()
  );
  println!("cargo:rustc-link-lib=dylib=athena_vmlib"); // Adjust the library name as necessary
}
