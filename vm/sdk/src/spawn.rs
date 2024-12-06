use athena_interface::Address;

pub fn spawn(blob: &[u8]) -> Address {
  Address::from(athena_vm::syscalls::spawn(blob))
}
