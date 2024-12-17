use std::collections::BTreeMap;

use anyhow::Context;
use elf::abi::{EM_RISCV, ET_EXEC, PF_X, PT_LOAD};
use elf::endian::LittleEndian;
use elf::file::Class;
use elf::ElfBytes;

/// The maximum size of the memory in bytes.
pub const MAXIMUM_MEMORY_SIZE: u32 = u32::MAX;

/// The size of a word in bytes.
pub const WORD_SIZE: usize = 4;

/// A RV32IM ELF file.
#[derive(Debug, Clone)]
pub struct Elf {
  /// The instructions of the program encoded as 32-bits.
  pub instructions: Vec<u32>,

  /// The start address of the program.
  pub pc_start: u32,

  /// The base address of the program.
  pub pc_base: u32,

  /// The initial memory image, useful for global constants.
  pub memory_image: BTreeMap<u32, u32>,

  /// Symbol table, useful for looking up function addresses.
  pub symbol_table: BTreeMap<String, u32>,
}

impl Elf {
  /// Create a new ELF file.
  pub const fn new(
    instructions: Vec<u32>,
    pc_start: u32,
    pc_base: u32,
    memory_image: BTreeMap<u32, u32>,
    symbol_table: BTreeMap<String, u32>,
  ) -> Self {
    Self {
      instructions,
      pc_start,
      pc_base,
      memory_image,
      symbol_table,
    }
  }

  /// Parse the ELF file into a vector of 32-bit encoded instructions and the first memory address.
  ///
  /// Reference: https://en.wikipedia.org/wiki/Executable_and_Linkable_Format
  pub fn decode(input: &[u8]) -> anyhow::Result<Self> {
    let mut image: BTreeMap<u32, u32> = BTreeMap::new();

    let elf = ElfBytes::<LittleEndian>::minimal_parse(input).context("failed to parse elf")?;

    // Some sanity checks to make sure that the ELF file is valid.
    anyhow::ensure!(elf.ehdr.class == Class::ELF32, "must be a 32-bit elf");
    anyhow::ensure!(elf.ehdr.e_machine == EM_RISCV, "must be a riscv machine");
    anyhow::ensure!(elf.ehdr.e_type == ET_EXEC, "must be executable");

    let entry: u32 = elf
      .ehdr
      .e_entry
      .try_into()
      .context("e_entry was larger than 32 bits")?;
    anyhow::ensure!(entry % 4 == 0);

    // Get the segments of the ELF file.
    let segments = elf.segments().context("elf should have segments")?;
    anyhow::ensure!(segments.len() <= 256, "too many program headers");

    let mut instructions = Vec::new();
    let mut base_address = u32::MAX;

    // Only read segments that are executable instructions that are also PT_LOAD.
    for segment in segments.iter().filter(|x| x.p_type == PT_LOAD) {
      let vaddr: u32 = segment
        .p_vaddr
        .try_into()
        .context("vaddr must fit in 32bit")?;
      anyhow::ensure!(vaddr % 4 == 0, "vaddr {vaddr:08x} is unaligned");

      // If the virtual address is less than the first memory address, then update the first
      // memory address.
      if (segment.p_flags & PF_X) != 0 {
        base_address = std::cmp::min(base_address, vaddr);
      }

      // Read the segment and decode each word as an instruction.
      let mut address = vaddr;
      let data = elf.segment_data(&segment)?;
      let mut word_chunks = data.chunks_exact(4);
      for chunk in &mut word_chunks {
        let word = u32::from_le_bytes(chunk.try_into().unwrap());
        image.insert(address, word);
        if (segment.p_flags & PF_X) != 0 {
          instructions.push(word);
        }
        address = address.checked_add(4).context("address out of bounds")?;
      }

      // Handle remaining bytes by padding with zeros to create a full word
      if !word_chunks.remainder().is_empty() {
        let remainder = word_chunks.remainder();
        let mut final_word = [0u8; 4];
        final_word[..remainder.len()].copy_from_slice(remainder);
        let word = u32::from_le_bytes(final_word);
        image.insert(address, word);
        if (segment.p_flags & PF_X) != 0 {
          instructions.push(word);
        }
      }
    }

    let exported_symbols = harvest_exported_symbols(&elf)?;

    Ok(Elf::new(
      instructions,
      entry,
      base_address,
      image,
      exported_symbols,
    ))
  }
}

fn harvest_exported_symbols(elf: &ElfBytes<LittleEndian>) -> anyhow::Result<BTreeMap<String, u32>> {
  let exported_section_header = elf
    .section_header_by_name(".note.athena_export")
    .context("section table should be parseable")?;

  let mut section_data = if let Some(header) = exported_section_header {
    let (s, _) = elf
      .section_data(&header)
      .context("section table should be parseable")?;
    s
  } else {
    return Ok(BTreeMap::default());
  };

  let mut exported_symbols = BTreeMap::new();

  let segments = elf.segments().context("elf should have segments")?;
  loop {
    if section_data.is_empty() {
      break;
    }
    let header = unsafe { &*(section_data.as_ptr() as *const ExportMetadata) };
    tracing::debug!(?header, "parsed export metadata header");

    anyhow::ensure!(header.version == 0);
    section_data = &section_data[std::mem::size_of::<ExportMetadata>()..];

    let address = header.address;
    let sym_ptr = header.sym_ptr;

    // Find the segment containing sym_ptr
    for segment in segments.iter().filter(|x| x.p_type == PT_LOAD) {
      let vaddr: u32 = segment.p_vaddr.try_into()?;
      let size: u32 = segment.p_memsz.try_into()?;

      // Check if sym_ptr is in this segment
      if sym_ptr >= vaddr && sym_ptr < vaddr + size {
        let str_offset = (sym_ptr - vaddr) as usize;
        let segment_data = elf.segment_data(&segment)?;
        // Read until null byte
        let str_bytes = segment_data[str_offset..]
          .iter()
          .copied()
          .take_while(|&b| b != 0)
          .collect::<Vec<u8>>();

        let symbol = String::from_utf8(str_bytes)?;
        tracing::debug!("read exported symbol: {symbol} -> 0x{address:x}");
        exported_symbols.insert(symbol, address);
        break;
      }
    }
  }
  Ok(exported_symbols)
}

#[repr(C, packed(1))]
#[derive(Debug)]
struct ExportMetadata {
  version: u8,
  address: u32,
  sym_ptr: u32,
}
