use parity_scale_codec::{Decode, Encode};

mod call;
pub mod precompiles;
pub mod wallet;
pub use call::call;
mod spawn;
pub use spawn::spawn;
mod deploy;
pub use deploy::deploy;
mod io;
pub use io::read_storage;
pub use io::write_storage;

#[derive(Clone, Copy, Debug, Default, Encode, Decode, PartialEq, Eq)]
pub struct Pubkey(pub [u8; 32]);
