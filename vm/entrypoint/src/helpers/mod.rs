use athena_interface::{Address, Balance, Bytes32, Bytes32Wrapper};
use bytemuck::cast;

// Simple helper functions for smart contracts

pub fn address_to_bytes32(address: Address) -> Bytes32 {
  Bytes32Wrapper::from(address).into()
}

pub fn address_to_32bit_words(address: Address) -> [u32; 6] {
  cast::<[u8; 24], [u32; 6]>(address)
}

pub fn balance_to_32bit_words(balance: Balance) -> [u32; 2] {
  cast::<[u8; 8], [u32; 2]>(balance.to_le_bytes())
}

pub fn bytes32_to_32bit_words(bytes32: Bytes32) -> [u32; 8] {
  cast::<[u8; 32], [u32; 8]>(bytes32)
}
