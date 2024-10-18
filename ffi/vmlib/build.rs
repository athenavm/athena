fn main() {
  // Platform-specific flags
  #[cfg(target_os = "macos")]
  {
    // Workaround for linker issue
    // See https://github.com/athenavm/athena/pull/161
    println!("cargo:rustc-link-arg=-undefined");
    println!("cargo:rustc-link-arg=dynamic_lookup");
  }
}
