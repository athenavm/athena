use athena_interface::Bytes32;

pub struct Bytes32AsU64(Bytes32);

impl Bytes32AsU64 {
  pub fn new(bytes: Bytes32) -> Self {
    Bytes32AsU64(bytes)
  }
}

impl From<Bytes32AsU64> for u64 {
  fn from(bytes: Bytes32AsU64) -> Self {
    // take most significant 8 bytes, assume little-endian
    let slice = &bytes.0[..8];
    u64::from_le_bytes(slice.try_into().expect("slice with incorrect length"))
  }
}

impl From<Bytes32AsU64> for Bytes32 {
  fn from(bytes: Bytes32AsU64) -> Self {
    bytes.0
  }
}

impl From<u64> for Bytes32AsU64 {
  fn from(value: u64) -> Self {
    let mut bytes = [0u8; 32];
    let value_bytes = value.to_le_bytes();
    bytes[..8].copy_from_slice(&value_bytes);
    Bytes32AsU64(bytes)
  }
}
