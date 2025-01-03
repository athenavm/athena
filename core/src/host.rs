use std::error::Error;

use athena_interface::{Address, AthenaMessage, ExecutionResult, StorageStatus};

#[mockall::automock]
pub trait HostInterface {
  fn get_storage(&self, addr: &Address, key: &[u8; 32]) -> [u8; 32];
  fn set_storage(&mut self, addr: &Address, key: &[u8; 32], value: &[u8; 32]) -> StorageStatus;
  fn get_balance(&self, addr: &Address) -> u64;
  fn call(&mut self, msg: AthenaMessage) -> ExecutionResult;
  fn spawn(&mut self, blob: Vec<u8>) -> Address;
  fn deploy(&mut self, code: Vec<u8>) -> Result<Address, Box<dyn Error>>;
}
