use crate::utils::Buffer;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// Standard input.
#[derive(Debug, Clone)]
pub struct AthenaStdin {
  buffer: Vec<u8>,
}

/// Public values for the runner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AthenaPublicValues {
  buffer: Buffer,
}

impl AthenaStdin {
  pub const fn new() -> Self {
    Self { buffer: Vec::new() }
  }

  /// Write a value to the buffer.
  pub fn write<T: Serialize>(&mut self, data: &T) {
    bincode::serialize_into(&mut self.buffer, data).expect("serialization failed");
  }

  /// Write a slice of bytes to the buffer.
  pub fn write_slice(&mut self, slice: &[u8]) {
    self.buffer.extend_from_slice(slice);
  }

  pub fn write_vec(&mut self, mut vec: Vec<u8>) {
    self.buffer.append(&mut vec);
  }

  pub fn to_vec(self) -> Vec<u8> {
    self.buffer
  }
}

impl AthenaPublicValues {
  /// Create a new `AthenaPublicValues`.
  pub const fn new() -> Self {
    Self {
      buffer: Buffer::new(),
    }
  }

  /// Create a `AthenaPublicValues` from a slice of bytes.
  pub fn from(data: &[u8]) -> Self {
    Self {
      buffer: Buffer::from(data),
    }
  }

  pub fn as_slice(&self) -> &[u8] {
    self.buffer.data.as_slice()
  }

  pub fn to_vec(&self) -> Vec<u8> {
    self.buffer.data.clone()
  }

  /// Read a value from the buffer.
  pub fn read<T: Serialize + DeserializeOwned>(&mut self) -> T {
    self.buffer.read()
  }

  /// Read a slice of bytes from the buffer.
  pub fn read_slice(&mut self, slice: &mut [u8]) {
    self.buffer.read_slice(slice);
  }

  /// Write a value to the buffer.
  pub fn write<T: Serialize>(&mut self, data: &T) {
    self.buffer.write(data);
  }

  /// Write a slice of bytes to the buffer.
  pub fn write_slice(&mut self, slice: &[u8]) {
    self.buffer.write_slice(slice);
  }
}

impl AsRef<[u8]> for AthenaPublicValues {
  fn as_ref(&self) -> &[u8] {
    &self.buffer.data
  }
}
