use athena_sdk::{AthenaStdin, ExecutionClient};

#[test]
fn test() {
  tracing_subscriber::fmt::init();

  // haser-test binary is precompiled and kept in git
  let elf = include_bytes!("../elf/hasher-test");
  let stdin = AthenaStdin::new();

  let result =
    ExecutionClient::new().execute_with_gdb(elf, stdin, None, Some(100000000), None, 9001);
  // result will be Err if asserts in the test failed
  result.unwrap();
}
