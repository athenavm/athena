use athena_interface::{Address, Balance, MethodSelector};
use athena_vm::helpers::{address_to_32bit_words, balance_to_32bit_words};

pub fn call(
  address: Address,
  input: Option<Vec<u8>>,
  mut method: Option<MethodSelector>,
  amount: Balance,
) {
  let address = address_to_32bit_words(address);
  let amount = balance_to_32bit_words(amount);

  // add the method selector, if present, to the input vector
  // for now, require input to be word-aligned
  // we can pad the input but need to know more about the contents
  let input32 = if let Some(input) = input {
    assert!(input.len() % 4 == 0, "input is not 4-byte-aligned");
    // concatenate the method selector to the input
    let input: Vec<u8> = method
      .unwrap_or_default()
      .into_iter()
      .chain(input.into_iter())
      .collect();
    Some((crate::bytes_to_u32_vec(&input), input.len()))
  } else {
    method.take().map(|m| (crate::bytes_to_u32_vec(&m), 4))
  };

  let (input, input_len) = input32.map_or((std::ptr::null(), 0), |(v, l)| (v.as_ptr(), l));

  athena_vm::syscalls::call(address.as_ptr(), input, input_len, amount.as_ptr());
}
