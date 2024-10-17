pub use athcon_vm::{MessageKind, Revision, StatusCode, StorageStatus};

pub const ADDRESS_LENGTH: usize = 24;
pub const BYTES32_LENGTH: usize = 32;
pub const METHOD_SELECTOR_LENGTH: usize = 4;
pub type Address = [u8; ADDRESS_LENGTH];
pub type Bytes32 = [u8; BYTES32_LENGTH];
pub type Bytes = [u8];
pub type MethodSelector = [u8; METHOD_SELECTOR_LENGTH];
