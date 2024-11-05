// Copyright 2023 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Memory addresses must be lower than BabyBear prime.
// Note: We inherit this constraint from SP1, and I don't see a reason to remove it.
const MAX_MEMORY: usize = 0x78000000;

// Platform-agnostic heap allocation starting at 1MB mark
static mut HEAP_START: usize = 1024 * 1024;
static mut HEAP_POS: usize = 0;

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn sys_alloc_aligned(bytes: usize, align: usize) -> *mut u8 {
  // SAFETY: Single threaded, so nothing else can touch this while we're working.
  let mut heap_pos = unsafe { HEAP_POS };

  if heap_pos == 0 {
    heap_pos = HEAP_START;
  }

  let offset = heap_pos & (align - 1);
  if offset != 0 {
    heap_pos += align - offset;
  }

  let ptr = heap_pos as *mut u8;
  let (heap_pos, overflowed) = heap_pos.overflowing_add(bytes);

  if overflowed || MAX_MEMORY < heap_pos {
    panic!("Memory limit exceeded (0x78000000)");
  }

  unsafe { HEAP_POS = heap_pos };
  ptr
}
