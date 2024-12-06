use athena_interface::Address;

pub fn deploy(blob: &[u8]) -> Address {
  Address::from(athena_vm::syscalls::deploy(blob))
}
