#![cfg_attr(target_os = "zkvm", no_main)]

#[cfg(target_os = "zkvm")]
athena_vm::entrypoint!(main);

use blake3::hash;

fn main() {
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
