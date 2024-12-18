use std::{cmp::min, collections::VecDeque, io::Write};

use crate::utils::Buffer;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// Standard input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AthenaStdin {
  pub buffer: VecDeque<u8>,
}

/// Public values for the runner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AthenaPublicValues {
  buffer: Buffer,
}

impl AthenaStdin {
  pub const fn new() -> Self {
    Self {
      buffer: VecDeque::new(),
    }
  }

  /// Create a `AthenaStdin` from a slice of bytes.
  pub fn from(data: &[u8]) -> Self {
    let mut stdin = Self::new();
    stdin.write_slice(data);
    stdin
  }

  /// Read a slice of bytes from the buffer.
  /// Reads up to slice.len() bytes from the beginning of the buffer into the provided slice.
  pub fn read_slice(&mut self, slice: &mut [u8]) {
    let bytes_to_read = min(slice.len(), self.buffer.len());
    if bytes_to_read == 0 {
      return;
    }

    // Get the two contiguous slices from the VecDeque
    let (first, second) = self.buffer.as_slices();

    // Copy from the first slice
    let first_copy = min(first.len(), bytes_to_read);
    slice[..first_copy].copy_from_slice(&first[..first_copy]);

    // If we need more bytes and there's a second slice, copy from it
    if first_copy < bytes_to_read {
      let second_copy = bytes_to_read - first_copy;
      slice[first_copy..bytes_to_read].copy_from_slice(&second[..second_copy]);
    }

    self.buffer.drain(..bytes_to_read);
  }

  /// Write a value to the buffer.
  pub fn write<T: Serialize>(&mut self, data: &T) {
    bincode::serialize_into(&mut self.buffer, data).expect("serialization failed");
  }

  /// Write a slice of bytes to the buffer.
  pub fn write_slice(&mut self, slice: &[u8]) {
    self.buffer.write_all(slice).expect("pushing to buffer");
  }

  pub fn write_vec(&mut self, vec: Vec<u8>) {
    self.write_slice(&vec);
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
