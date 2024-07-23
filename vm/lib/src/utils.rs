/// Converts a slice of words to a byte array in little endian.
pub fn words_to_bytes_le(words: &[u32]) -> Vec<u8> {
  words
    .iter()
    .flat_map(|word| word.to_le_bytes().to_vec())
    .collect::<Vec<_>>()
}

/// Converts a byte array in little endian to a slice of words.
pub fn bytes_to_words_le(bytes: &[u8]) -> Vec<u32> {
  bytes
    .chunks_exact(4)
    .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
    .collect::<Vec<_>>()
}
