use std::mem::{ManuallyDrop, MaybeUninit};

use athena_interface::{payload::Payload, Address, Balance, Encode, MethodSelector};

/// Execute CALL syscall
///
/// Support returning up to 1024B of output data from the called program.
pub fn call(
  address: Address,
  input: Option<Vec<u8>>,
  method: Option<&str>,
  amount: Balance,
) -> Vec<u8> {
  let payload = Payload {
    selector: method.map(MethodSelector::from),
    input: input.unwrap_or_default(),
  };
  let input_payload = payload.encode();

  // Allocate output buffer on the heap, aligned to 4B.
  // Its capacity might be configurable and changed in the future.
  // See https://github.com/athenavm/athena/issues/177.
  let output_cap = 1024;
  // Prevent `output_buf` from being dropped, as we're transferring ownership.
  let mut output_buf = ManuallyDrop::new(Vec::<MaybeUninit<u32>>::with_capacity(output_cap / 4));
  let bytes_written = athena_vm::syscalls::call(
    &address,
    &input_payload,
    output_buf.as_mut_ptr() as *mut u32,
    output_cap,
    amount,
  );
  assert!(
    bytes_written <= output_cap,
    "wrote too many bytes, possible buffer overflow"
  );
  // Convert the Vec<MaybeUninit<u32>> into a Vec<u8> without freeing buffer
  // accounting for the number of bytes written.
  let raw_ptr = output_buf.as_mut_ptr() as *mut u8;
  unsafe { Vec::from_raw_parts(raw_ptr, bytes_written, output_cap) }
}
