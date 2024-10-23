use athena_interface::{payload::Payload, Address, Balance, Encode, MethodSelector};
use athena_vm::helpers::{address_to_32bit_words, balance_to_32bit_words};

/// Execute CALL syscall
///
/// Support returning up to 1024B of output data from the called program.
pub fn call(
  address: Address,
  input: Option<Vec<u8>>,
  method: Option<&str>,
  amount: Balance,
) -> Vec<u8> {
  let address = address_to_32bit_words(address);
  let amount = balance_to_32bit_words(amount);

  let payload = Payload {
    selector: method.map(MethodSelector::from),
    input: input.unwrap_or_default(),
  };
  let input_payload = payload.encode();
  let input_len = input_payload.len();
  let input = crate::bytes_to_u32_vec(input_payload);

  // Allocate 1024 bytes on the heap, using u32 for proper alignment
  let mut output_buf = Vec::<std::mem::MaybeUninit<u32>>::with_capacity(1024 / 4);

  let output_size = athena_vm::syscalls::call(
    address.as_ptr(),
    input.as_ptr(),
    input_len,
    output_buf.as_mut_ptr() as *mut u32,
    1024,
    amount.as_ptr(),
  );

  // Convert the Vec<MaybeUninit<u32>> into a Vec<u8> without freeing buffer
  let raw_ptr = output_buf.as_mut_ptr() as *mut u8;
  let capacity = output_buf.capacity();
  // Prevent `buffer` from being dropped, as we're transferring ownership
  std::mem::forget(output_buf);

  // Convert the buffer into Vec<u8>, accounting for the number of bytes written
  unsafe { Vec::from_raw_parts(raw_ptr, output_size, capacity) }
}
