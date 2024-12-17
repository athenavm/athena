#[repr(C, packed(1))]
pub struct ExportMetadata {
  pub version: u8,
  pub export_ptr: *const u32,
  pub symbol_ptr: *const u8,
}

unsafe impl Sync for ExportMetadata {}
