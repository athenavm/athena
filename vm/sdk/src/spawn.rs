use athena_interface::Address;

pub fn spawn(blob: Vec<u8>) -> Address {
  let blob_u32 = crate::bytes_to_u32_vec(&blob);
  Address::from(athena_vm::syscalls::spawn(&blob_u32, blob.len()))
}
