// Template address is read from context
pub fn spawn(blob: Vec<u8>) {
  let blob_u32 = bytes_to_u32_vec(&blob);
  athena_vm::syscalls::spawn(blob_u32.as_ptr(), blob.len());
}

/// Convert a slice of bytes to a vector of u32 little-endian values.
/// In case the length of the input slice is not a multiple of 4, the remaining bytes are
/// zero-padded and appended as the last u32 value.
fn bytes_to_u32_vec<T: AsRef<[u8]>>(bytes: T) -> Vec<u32> {
  let mut chunks = bytes.as_ref().chunks_exact(4);
  let mut result = chunks
    .by_ref()
    .map(|chunk| {
      let mut bytes = [0u8; 4];
      bytes.copy_from_slice(chunk);
      u32::from_le_bytes(bytes)
    })
    .collect::<Vec<u32>>();

  let remainder = chunks.remainder();
  if !remainder.is_empty() {
    let mut bytes = [0u8; 4];
    bytes[..remainder.len()].copy_from_slice(remainder);
    result.push(u32::from_le_bytes(bytes));
  }

  result
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn convert_empty_slice() {
    let result = bytes_to_u32_vec([]);
    assert_eq!(result, Vec::new());
  }

  #[test]
  fn convert_exact_multiple_of_4() {
    let result = bytes_to_u32_vec([1, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!(result, vec![0x04030201, 0x08070605]);
  }

  #[test]
  fn convert_not_a_multiple_of_4() {
    let result = bytes_to_u32_vec([1, 2, 3, 4, 5, 6, 7]);
    assert_eq!(result, vec![0x04030201, 0x00070605]);
  }

  #[test]
  fn convert_single_byte() {
    let result = bytes_to_u32_vec([1]);
    assert_eq!(result, vec![0x00_00_00_01]);
  }
}
