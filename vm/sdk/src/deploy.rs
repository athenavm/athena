use athena_interface::Address;

pub fn deploy(blob: &[u8]) -> Address {
  athena_vm::syscalls::deploy(blob)
}
