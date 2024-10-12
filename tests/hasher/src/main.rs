#![no_main]
athena_vm::entrypoint!(main);

use blake3::hash;
use hex;

#[cfg(all(
  any(target_arch = "riscv32", target_arch = "riscv64"),
  target_feature = "e"
))]
pub fn main() {
  let value = "athexp_test2";
  let res = hash(value.as_bytes());
  println!(
    "value: {}; as_bytes: {}; hash: {}",
    value,
    hex::encode(value.as_bytes()),
    hex::encode(res.as_bytes())
  );
  let res = hash(value.as_bytes());
  println!(
    "value: {}; as_bytes: {}; hash: {}",
    value,
    hex::encode(value.as_bytes()),
    hex::encode(res.as_bytes())
  );
  let res = hash(value.as_bytes());
  println!(
    "value: {}; as_bytes: {}; hash: {}",
    value,
    hex::encode(value.as_bytes()),
    hex::encode(res.as_bytes())
  );
}
