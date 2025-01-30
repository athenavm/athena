athena_vm::entrypoint!(main);

use athena_vm::syscalls::{read_storage, write_storage};
use athena_vm::types::{StorageStatus::StorageAdded, StorageStatus::StorageModified};

fn main() {
  let key = [0x01; 8];
  let key2 = [0x02; 8];
  let value = [0xaa; 8];
  let value2: [u32; 8] = [0xbb; 8];

  // Try an empty key
  let res = read_storage(&key);
  assert_eq!([0; 8], res, "read_storage failed");

  // Add it
  let status = write_storage(&key, &value);
  assert_eq!(status, StorageAdded, "write_storage failed");
  let res = read_storage(&key);
  assert_eq!(value, res, "read_storage failed");

  // Modify it
  let status = write_storage(&key, &value2);
  assert_eq!(status, StorageModified, "write_storage failed");
  let res = read_storage(&key);
  assert_eq!(value2, res, "read_storage failed");

  // Try an empty key
  let res = read_storage(&key2);
  assert_eq!([0; 8], res, "read_storage failed");

  // Write to the new key
  let status = write_storage(&key2, &value);
  assert_eq!(status, StorageAdded, "write_storage failed");

  // Read the new value
  let res = read_storage(&key2);
  assert_eq!(value, res, "read_storage failed");
}
