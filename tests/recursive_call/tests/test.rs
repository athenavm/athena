use athena_interface::{Address, AthenaContext, MockHost};
use athena_runner::AthenaVm;
use athena_sdk::{AthenaStdin, ExecutionClient};

#[test]
fn test() {
  tracing_subscriber::fmt::init();

  let elf = include_bytes!("../elf/recursive-call-test");
  let mut stdin = AthenaStdin::new();
  let vm = AthenaVm::new();
  let mut host = MockHost::new_with_vm(&vm);
  let template_address = Address::from([0x77; 24]);
  stdin.write(&(template_address, 6u32));
  host.deploy_code(template_address, elf.to_vec());

  let context = AthenaContext::new(template_address, template_address, 0);
  let result =
    ExecutionClient::new().execute(elf, stdin, Some(&mut host), Some(100000000), Some(context));

  let (mut output, _) = result.unwrap();

  let result = output.read::<u32>();
  // fibonacci(6) is 8
  assert_eq!(result, 8);
}
