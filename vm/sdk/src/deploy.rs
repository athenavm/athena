use athena_interface::Address;

pub fn deploy(code: Vec<u8>) -> Address {
  let blob_u32 = crate::bytes_to_u32_vec(&code);
  Address::from(athena_vm::syscalls::deploy(&blob_u32, code.len()))
}
