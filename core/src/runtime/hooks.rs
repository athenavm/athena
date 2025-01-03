use std::collections::HashMap;

use super::Runtime;

#[mockall::automock]
#[allow(clippy::needless_lifetimes)] // the lifetimes are needed by automock
pub trait Hook {
  fn execute<'r, 'h>(&self, env: HookEnv<'r, 'h>, data: &[u8]) -> anyhow::Result<Vec<u8>>;
}

/// A registry of hooks to call, indexed by the file descriptors through which they are accessed.
#[derive(Default)]
pub struct HookRegistry {
  /// Table of registered hooks.
  table: HashMap<u32, Box<dyn Hook>>,
}

impl HookRegistry {
  /// Create a registry with the default hooks.
  pub fn new() -> Self {
    Default::default()
  }

  /// Register a hook under a given FD.
  /// Will fail if a hook is already registered on a given FD
  /// or a FD is <= 4.
  pub fn register(&mut self, fd: u32, hook: Box<dyn Hook>) -> anyhow::Result<()> {
    anyhow::ensure!(fd > 4, "FDs 0-4 are reserved for internal usage");
    anyhow::ensure!(
      !self.table.contains_key(&fd),
      "there is already a hook for FD {fd} registered"
    );
    self.table.insert(fd, hook);
    Ok(())
  }

  pub(crate) fn get(&self, fd: u32) -> Option<&dyn Hook> {
    self.table.get(&fd).map(AsRef::as_ref)
  }
}

/// Environment that a hook may read from.
pub struct HookEnv<'r, 'h> {
  pub runtime: &'r Runtime<'h>,
}

#[cfg(test)]
pub mod tests {
  use super::*;

  #[test]
  pub fn registry_new_is_empty() {
    assert!(HookRegistry::new().table.is_empty());
  }
}
