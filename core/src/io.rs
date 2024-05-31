use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// Standard input for the prover.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AthenaStdin {
    /// Input stored as a vec of vec of bytes. It's stored this way because the read syscall reads
    /// a vec of bytes at a time.
    pub buffer: Vec<Vec<u8>>,
    pub ptr: usize,
}

impl AthenaStdin {
    /// Create a new `AthenaStdin`.
    pub const fn new() -> Self {
        Self {
            buffer: Vec::new(),
            ptr: 0,
        }
    }

    /// Create a `AthenaStdin` from a slice of bytes.
    pub fn from(data: &[u8]) -> Self {
        Self {
            buffer: vec![data.to_vec()],
            ptr: 0,
        }
    }

    /// Read a value from the buffer.
    pub fn read<T: Serialize + DeserializeOwned>(&mut self) -> T {
        let result: T =
            bincode::deserialize(&self.buffer[self.ptr]).expect("failed to deserialize");
        self.ptr += 1;
        result
    }

    /// Read a slice of bytes from the buffer.
    pub fn read_slice(&mut self, slice: &mut [u8]) {
        slice.copy_from_slice(&self.buffer[self.ptr]);
        self.ptr += 1;
    }

    /// Write a value to the buffer.
    pub fn write<T: Serialize>(&mut self, data: &T) {
        let mut tmp = Vec::new();
        bincode::serialize_into(&mut tmp, data).expect("serialization failed");
        self.buffer.push(tmp);
    }

    /// Write a slice of bytes to the buffer.
    pub fn write_slice(&mut self, slice: &[u8]) {
        self.buffer.push(slice.to_vec());
    }

    pub fn write_vec(&mut self, vec: Vec<u8>) {
        self.buffer.push(vec);
    }
}
