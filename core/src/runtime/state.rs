use std::collections::HashMap;

use nohash_hasher::BuildNoHashHasher;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

/// Holds data describing the current state of a program's execution.
#[serde_as]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionState {
  /// The global clock keeps track of how many instrutions have been executed.
  pub global_clk: u64,

  /// The clock increments by 4 (possibly more in syscalls) for each instruction that has been
  /// executed.
  pub clk: u32,

  /// The program counter.
  pub pc: u32,

  /// The memory which instructions operate over.
  pub memory: HashMap<u32, u32, BuildNoHashHasher<u32>>,

  /// Uninitialized memory addresses that have a specific value they should be initialized with.
  /// SyscallHintRead uses this to write hint data into uninitialized memory.
  pub uninitialized_memory: HashMap<u32, u32, BuildNoHashHasher<u32>>,

  /// A stream of input values (global to the entire program).
  pub input_stream: Vec<Vec<u8>>,

  /// A ptr to the current position in the input stream incremented by HINT_READ opcode.
  pub input_stream_ptr: usize,

  /// A stream of public values from the program (global to entire program).
  pub public_values_stream: Vec<u8>,

  /// A ptr to the current position in the public values stream, incremented when reading from public_values_stream.
  pub public_values_stream_ptr: usize,
}

impl ExecutionState {
  pub fn new(pc_start: u32) -> Self {
    Self {
      global_clk: 0,
      clk: 0,
      pc: pc_start,
      memory: HashMap::default(),
      uninitialized_memory: HashMap::default(),
      input_stream: Vec::new(),
      input_stream_ptr: 0,
      public_values_stream: Vec::new(),
      public_values_stream_ptr: 0,
    }
  }
}
