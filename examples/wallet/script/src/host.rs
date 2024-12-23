//! Example host implementation

use std::{collections::BTreeMap, error::Error};

use athena_interface::{
  payload::ExecutionPayload, Address, AthenaMessage, Balance, Bytes32, ExecutionResult,
  HostInterface, StatusCode, StorageStatus,
};
use athena_runner::{vm::AthenaRevision, AthenaVm};

// Stores some of the context that a running host would store to keep
// track of what's going on in the VM execution
// static context is set from the transaction and doesn't change until
// the execution stack is done.
pub struct HostStaticContext {
  // the ultimate initiator of the current execution stack. also the
  // account that pays gas for the execution stack.
  _principal: Address,

  // the principal's nonce from the tx
  _nonce: u64,

  // the destination of the transaction. note that, while this is the
  // program that was initiated, it likely made additional calls.
  // this is generally the caller's wallet, and is generally the same
  // as the principal.
  _destination: Address,
  // in the future we'll probably need things here like block height,
  // block hash, etc.
}

impl HostStaticContext {
  pub fn new(principal: Address, nonce: u64, destination: Address) -> HostStaticContext {
    HostStaticContext {
      _principal: principal,
      _nonce: nonce,
      _destination: destination,
    }
  }
}

// this context is relevant only for the current execution frame
pub struct HostDynamicContext {
  // the initiator and recipient programs of the current message/call frame
  template: Address,
  _callee: Address,
}

impl HostDynamicContext {
  pub fn new(template: Address, callee: Address) -> HostDynamicContext {
    HostDynamicContext {
      template,
      _callee: callee,
    }
  }
}

// a very simple mock host implementation for testing
// also useful for filling in the missing generic type
// when running the VM in standalone mode, without a bound host interface
#[derive(Default)]
pub struct MockHost {
  // stores state keyed by address and key
  storage: BTreeMap<(Address, Bytes32), Bytes32>,

  // stores balance keyed by address
  balance: BTreeMap<Address, Balance>,

  // stores contract code
  templates: BTreeMap<Address, Vec<u8>>,

  // stores program instances
  programs: BTreeMap<Address, Vec<u8>>,

  // context information
  _static_context: Option<HostStaticContext>,
  dynamic_context: Option<HostDynamicContext>,
}

impl MockHost {
  pub fn new(static_context: HostStaticContext, dynamic_context: HostDynamicContext) -> Self {
    MockHost {
      dynamic_context: Some(dynamic_context),
      _static_context: Some(static_context),
      ..MockHost::default()
    }
  }

  /// Set balance of given address.
  /// The previous balance is discarded.
  pub fn set_balance(&mut self, address: &Address, balance: Balance) {
    self.balance.insert(*address, balance);
  }

  pub fn spawn_program(&mut self, template: &Address, blob: Vec<u8>) -> Address {
    // PRINCIPAL ADDRESS = HASH(template | blob)
    let hash = blake3::Hasher::new()
      .update(&template.0)
      .update(&blob)
      .finalize();
    let address = Address(hash.as_bytes()[..24].try_into().unwrap());

    tracing::info!(principal = %address, template = %template, "spawning program");

    self.programs.insert(address, blob);
    address
  }

  pub fn get_program(&self, address: &Address) -> Option<&Vec<u8>> {
    self.programs.get(address)
  }

  pub fn deploy_code(&mut self, address: Address, code: Vec<u8>) {
    self.templates.insert(address, code);
  }

  pub fn transfer_balance(&mut self, from: &Address, to: &Address, value: u64) -> StatusCode {
    let balance_from = self.get_balance(from);
    let balance_to = self.get_balance(to);
    if value > balance_from {
      return StatusCode::InsufficientBalance;
    }
    match balance_to.checked_add(value) {
      Some(new_balance) => {
        self.balance.insert(*from, balance_from - value);
        self.balance.insert(*to, new_balance);
        StatusCode::Success
      }
      None => StatusCode::InternalError,
    }
  }
}

impl HostInterface for MockHost {
  fn get_storage(&self, addr: &Address, key: &Bytes32) -> Bytes32 {
    self
      .storage
      .get(&(*addr, *key))
      .copied()
      .unwrap_or(Bytes32::default())
  }

  fn set_storage(&mut self, addr: &Address, key: &Bytes32, value: &Bytes32) -> StorageStatus {
    // this is a very simplistic implementation and does NOT handle all possible cases correctly
    match self.storage.insert((*addr, *key), *value) {
      None => StorageStatus::StorageAdded,
      Some(_) => StorageStatus::StorageModified,
    }
  }

  fn get_balance(&self, addr: &Address) -> u64 {
    self.balance.get(addr).copied().unwrap_or(0)
  }

  #[tracing::instrument(skip(self, msg), fields(id = self as *const Self as usize, depth = msg.depth))]
  fn call(&mut self, msg: AthenaMessage) -> ExecutionResult {
    tracing::info!(msg = ?msg);

    // don't go too deep!
    if msg.depth > 10 {
      return ExecutionResult::new(StatusCode::CallDepthExceeded, 0, None);
    }

    // take snapshots of the state in case we need to roll back
    // this is relatively expensive and we'd want to do something more sophisticated in production
    // (journaling? CoW?) but it's fine for testing.
    let backup_storage = self.storage.clone();
    let backup_balance = self.balance.clone();
    let backup_programs = self.templates.clone();

    // transfer balance
    // note: the host should have already subtracted an amount from the sender
    // equal to the maximum amount of gas that could be paid, so this should
    // not allow an out of gas error.
    match self.transfer_balance(&msg.sender, &msg.recipient, msg.value) {
      StatusCode::Success => {}
      status => {
        return ExecutionResult::new(status, 0, None);
      }
    }

    // save message for context
    let old_dynamic_context = self.dynamic_context.replace(HostDynamicContext {
      template: msg.sender,
      _callee: msg.recipient,
    });

    // check programs list first
    let res = if let Some(code) = self.templates.get(&msg.recipient).cloned() {
      // The optional msg.input_data must be enriched with optional account state
      // and then passed to the VM.
      let msg = match msg.input_data {
        Some(data) => {
          // TODO: figure out when to provide a state here
          let state = vec![];
          AthenaMessage {
            input_data: Some(ExecutionPayload::encode_with_encoded_payload(state, data)),
            ..msg
          }
        }
        None => msg,
      };

      AthenaVm::new().execute(self, AthenaRevision::AthenaFrontier, msg, &code)
    } else {
      let gas_left = msg.gas.checked_sub(1).expect("gas underflow");
      ExecutionResult::new(StatusCode::Success, gas_left, None)
    };

    self.dynamic_context = old_dynamic_context;
    if res.status_code != StatusCode::Success {
      // rollback state
      self.storage = backup_storage;
      self.balance = backup_balance;
      self.templates = backup_programs;
    }
    res
  }

  fn spawn(&mut self, blob: Vec<u8>) -> Address {
    let template = self
      .dynamic_context
      .as_ref()
      .expect("missing dynamic host context")
      .template;

    // Now call spawn_program with the extracted values
    self.spawn_program(&template, blob)
  }

  fn deploy(&mut self, code: Vec<u8>) -> Result<Address, Box<dyn Error>> {
    // template_address := HASH(template_code)
    let hash = blake3::hash(&code);
    let hash_bytes = hash.as_bytes().as_slice();
    let address = Address(hash_bytes[..24].try_into().unwrap());

    if self.templates.contains_key(&address) {
      return Err("template already exists".into());
    }
    self.deploy_code(address, code);
    Ok(address)
  }
}
