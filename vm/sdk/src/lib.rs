use athena_hostfunctions;
use athena_interface::{Address, Balance};
use athena_vm::helpers::{address_to_32bit_words, balance_to_32bit_words};

pub fn call(address: Address, input: Vec<u8>, amount: Balance) {
  let address = address_to_32bit_words(address);
  let amount = balance_to_32bit_words(amount);

  // for now, require input to be word-aligned
  // we can pad the input but need to know more about the contents
  if (input.len() % 4) != 0 {
    panic!("input is not byte-aligned");
  }
  let input = input
    .chunks(4)
    .map(|chunk| {
      let mut bytes = [0u8; 4];
      bytes.copy_from_slice(chunk);
      u32::from_le_bytes(bytes)
    })
    .collect::<Vec<u32>>();

  unsafe {
    athena_hostfunctions::call(
      address.as_ptr(),
      input.as_ptr(),
      input.len(),
      amount.as_ptr(),
    );
  }
}
