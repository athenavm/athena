use athena_interface::{payload::Payload, Address, Balance, Encode, MethodSelector};
use athena_vm::helpers::{address_to_32bit_words, balance_to_32bit_words};

pub fn call(address: Address, input: Option<Vec<u8>>, method: Option<&str>, amount: Balance) {
  let address = address_to_32bit_words(address);
  let amount = balance_to_32bit_words(amount);

  let payload = Payload {
    selector: method.map(MethodSelector::from),
    input: input.unwrap_or_default(),
  };
  let input_payload = payload.encode();
  let input_len = input_payload.len();
  let input = crate::bytes_to_u32_vec(input_payload);
  athena_vm::syscalls::call(address.as_ptr(), input.as_ptr(), input_len, amount.as_ptr());
}
