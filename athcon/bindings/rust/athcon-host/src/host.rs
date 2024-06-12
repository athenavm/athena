 use crate::types::*;
 use athcon_sys as ffi;
 use std::mem;

 #[repr(C)]
 pub(crate) struct ExtendedContext<'a> {
     pub hctx: &'a mut dyn HostContext,
 }

 pub trait HostContext {
     fn account_exists(&mut self, addr: &Address) -> bool;
     fn get_storage(&mut self, addr: &Address, key: &Bytes32) -> Bytes32;
     fn set_storage(&mut self, addr: &Address, key: &Bytes32, value: &Bytes32) -> StorageStatus;
     fn get_balance(&mut self, addr: &Address) -> Bytes32;
     fn get_tx_context(&mut self) -> (Bytes32, Address, i64, i64, i64, Bytes32);
     fn get_block_hash(&mut self, number: i64) -> Bytes32;
     fn call(
         &mut self,
         kind: MessageKind,
         recipient: &Address,
         sender: &Address,
         value: &Bytes32,
         input: &Bytes,
         gas: i64,
         depth: i32,
     ) -> (Vec<u8>, i64, Address, StatusCode);
 }

 pub(crate) fn get_athcon_host_interface() -> ffi::athcon_host_interface {
     ffi::athcon_host_interface {
         account_exists: Some(account_exists),
         get_storage: Some(get_storage),
         set_storage: Some(set_storage),
         get_balance: Some(get_balance),
         call: Some(call),
         get_tx_context: Some(get_tx_context),
         get_block_hash: Some(get_block_hash),
     }
 }

 unsafe extern "C" fn account_exists(
     context: *mut ffi::athcon_host_context,
     address: *const ffi::athcon_address,
 ) -> bool {
     return (*(context as *mut ExtendedContext))
         .hctx
         .account_exists(&(*address).bytes);
 }

 unsafe extern "C" fn get_storage(
     context: *mut ffi::athcon_host_context,
     address: *const ffi::athcon_address,
     key: *const ffi::athcon_bytes32,
 ) -> ffi::athcon_bytes32 {
     return ffi::athcon_bytes32 {
         bytes: (*(context as *mut ExtendedContext))
             .hctx
             .get_storage(&(*address).bytes, &(*key).bytes),
     };
 }

 unsafe extern "C" fn set_storage(
     context: *mut ffi::athcon_host_context,
     address: *const ffi::athcon_address,
     key: *const ffi::athcon_bytes32,
     value: *const ffi::athcon_bytes32,
 ) -> ffi::athcon_storage_status {
     return (*(context as *mut ExtendedContext)).hctx.set_storage(
         &(*address).bytes,
         &(*key).bytes,
         &(*value).bytes,
     );
 }

 unsafe extern "C" fn get_balance(
     context: *mut ffi::athcon_host_context,
     address: *const ffi::athcon_address,
 ) -> ffi::athcon_uint256be {
     return ffi::athcon_uint256be {
         bytes: (*(context as *mut ExtendedContext))
             .hctx
             .get_balance(&(*address).bytes),
     };
 }

 unsafe extern "C" fn get_tx_context(context: *mut ffi::athcon_host_context) -> ffi::athcon_tx_context {
     let (gas_price, origin, height, timestamp, gas_limit, chain_id) =
         (*(context as *mut ExtendedContext)).hctx.get_tx_context();
     return ffi::athcon_tx_context {
         tx_gas_price: athcon_sys::athcon_bytes32 { bytes: gas_price },
         tx_origin: athcon_sys::athcon_address { bytes: origin },
         block_height: height,
         block_timestamp: timestamp,
         block_gas_limit: gas_limit,
         chain_id: athcon_sys::athcon_bytes32 { bytes: chain_id },
     };
 }

 unsafe extern "C" fn get_block_hash(
     context: *mut ffi::athcon_host_context,
     number: i64,
 ) -> ffi::athcon_bytes32 {
     return ffi::athcon_bytes32 {
         bytes: (*(context as *mut ExtendedContext))
             .hctx
             .get_block_hash(number),
     };
 }

 unsafe extern "C" fn release(result: *const ffi::athcon_result) {
  // Recreate the Vec<u8> from the raw parts to take ownership back
  // This allows Rust to properly free the allocated memory when the Vec goes out of scope
  let _output = Vec::from_raw_parts(
      (*result).output_data as *mut u8,
      (*result).output_size,
      (*result).output_size,
  );
  // No need to explicitly call drop here; it will be dropped when _output goes out of scope
}

 pub unsafe extern "C" fn call(
     context: *mut ffi::athcon_host_context,
     msg: *const ffi::athcon_message,
 ) -> ffi::athcon_result {
     let msg = *msg;
     let (output, gas_left, create_address, status_code) =
         (*(context as *mut ExtendedContext)).hctx.call(
             msg.kind,
             &msg.recipient.bytes,
             &msg.sender.bytes,
             &msg.value.bytes,
             &std::slice::from_raw_parts(msg.input_data, msg.input_size),
             msg.gas,
             msg.depth,
         );
     let ptr = output.as_ptr();
     // Prevent Rust from automatically freeing the memory
     let len = output.len();
     mem::forget(output);
     return ffi::athcon_result {
         status_code: status_code,
         gas_left: gas_left,
         output_data: ptr,
         output_size: len,
         release: Some(release),
         create_address: ffi::athcon_address {
             bytes: create_address,
         },
     };
 }
