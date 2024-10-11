use athena_interface::{Address, Balance, MethodSelector};
use athena_vm::helpers::{address_to_32bit_words, balance_to_32bit_words};

pub fn call(
  address: Address,
  input: Option<Vec<u8>>,
  method: Option<MethodSelector>,
  amount: Balance,
) {
  let address = address_to_32bit_words(address);
  let amount = balance_to_32bit_words(amount);

  // for now, require input to be word-aligned
  // we can pad the input but need to know more about the contents
  let input32 = if let Some(input) = input {
    assert!(input.len() % 4 == 0, "input is not 4-byte-aligned");
    Some((crate::bytes_to_u32_vec(&input), input.len()))
  } else {
    None
  };

  let (input, input_len) = input32.map_or((std::ptr::null(), 0), |(v, l)| (v.as_ptr(), l));

  // we don't require method name to be word-aligned
  let method32 = if let Some(method) = method {
    Some((crate::bytes_to_u32_vec(&method), method.len()))
  } else {
    None
  };

  let (method_name, method_name_len) =
    method32.map_or((std::ptr::null(), 0), |(v, _)| (v.as_ptr(), v.len() * 4));

  athena_vm::syscalls::call(
    address.as_ptr(),
    input,
    input_len,
    method_name,
    method_name_len,
    amount.as_ptr(),
  );
}
