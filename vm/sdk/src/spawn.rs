use athena_interface::Address;

pub fn spawn(blob: &[u8]) -> Address {
  athena_vm::syscalls::spawn(blob)
}
