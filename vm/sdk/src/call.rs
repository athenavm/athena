use athena_interface::{Address, Balance, ExecutionPayload, MethodSelector};
use athena_vm::helpers::{address_to_32bit_words, balance_to_32bit_words};

pub fn call(address: Address, input: Option<Vec<u8>>, method: Option<&str>, amount: Balance) {
  let address = address_to_32bit_words(address);
  let amount = balance_to_32bit_words(amount);

  let payload = ExecutionPayload {
    selector: method.map(MethodSelector::from),
    input: input.unwrap_or_default(),
  };
  let input_payload = bincode::serialize(&payload).unwrap();
  let input_len = input_payload.len();
  let input = crate::bytes_to_u32_vec(input_payload);
  athena_vm::syscalls::call(address.as_ptr(), input.as_ptr(), input_len, amount.as_ptr());
}
