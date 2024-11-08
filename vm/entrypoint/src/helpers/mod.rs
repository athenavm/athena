use athena_interface::Bytes32;
use bytemuck::cast;

// Simple helper functions for smart contracts

pub fn bytes32_to_32bit_words(bytes32: Bytes32) -> [u32; 8] {
  cast::<[u8; 32], [u32; 8]>(bytes32)
}

pub fn words_to_bytes32(words: [u32; 8]) -> Bytes32 {
  cast::<[u32; 8], [u8; 32]>(words)
}
