use athena_interface::Bytes32;
use cfg_if::cfg_if;
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

pub mod wallet;

cfg_if! {
  if #[cfg(feature = "vm")] {
    pub mod precompiles;
    mod call;
    pub use call::call;
  }
}

cfg_if! {
  // TODO: migrate to feature instead of target_os
  // reasoning: using a feature allows testing the code, enables LSP etc.
  if #[cfg(target_os = "zkvm")] {
    mod spawn;
    pub use spawn::spawn;
    mod deploy;
    pub use deploy::deploy;
    mod io;
    pub use io::read_storage;
    pub use io::write_storage;
  }
}

#[derive(Clone, Copy, Debug, Default, Encode, Decode, Serialize, Deserialize, PartialEq, Eq)]
pub struct Pubkey(pub Bytes32);

pub trait VerifiableTemplate {
  fn verify(&self, tx: Vec<u8>, signature: [u8; 64]) -> bool;
}
