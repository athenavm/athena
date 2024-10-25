use athena_interface::{
  Address, AthenaContext, HostInterface, MockHost, ADDRESS_CHARLIE, STORAGE_KEY, STORAGE_VALUE,
};
use athena_sdk::{AthenaStdin, ExecutionClient};

#[test]
fn test() {
  tracing_subscriber::fmt::init();

  let caller = ADDRESS_CHARLIE;
  // host-test binary is precompiled and kept in git
  let elf = include_bytes!("../elf/host-test");
  let stdin = AthenaStdin::new();
  let mut host = MockHost::new();
  host.set_balance(&caller, 10000);
  host.set_storage(&caller, &STORAGE_KEY, &STORAGE_VALUE);

  let context = AthenaContext::new(caller, Address::default(), 0);
  let result =
    ExecutionClient::new().execute(elf, stdin, Some(&mut host), Some(100000), Some(context));
  // result will be Err if asserts in the test failed
  let (_, gas_left) = result.unwrap();

  // don't bother checking exact gas value, that's checked in the following test
  assert!(gas_left.is_some());
}
