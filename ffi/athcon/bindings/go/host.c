#include "_cgo_export.h"

#include <stdlib.h>

/* Go does not support exporting functions with parameters with const modifiers,
 * so we have to cast function pointers to the function types defined in ATHCON.
 * This disables any type checking of exported Go functions. To mitigate this
 * problem the go_exported_functions_type_checks() function simulates usage
 * of Go exported functions with expected types to check them during compilation.
 */
const struct athcon_host_interface athcon_go_host = {
    (athcon_account_exists_fn)accountExists,
    (athcon_get_storage_fn)getStorage,
    (athcon_set_storage_fn)setStorage,
    (athcon_get_balance_fn)getBalance,
    (athcon_call_fn)call,
    (athcon_get_tx_context_fn)getTxContext,
    (athcon_get_block_hash_fn)getBlockHash,
};

#pragma GCC diagnostic error "-Wconversion"
static inline void go_exported_functions_type_checks()
{
  struct athcon_host_context *context = NULL;
  athcon_address *address = NULL;
  athcon_bytes32 bytes32;
  int64_t number = 0;
  struct athcon_message *message = NULL;

  athcon_uint256be uint256be;
  (void)uint256be;
  struct athcon_tx_context tx_context;
  (void)tx_context;
  struct athcon_result result;
  (void)result;
  enum athcon_storage_status storage_status;
  (void)storage_status;
  bool bool_flag;
  (void)bool_flag;

  athcon_account_exists_fn account_exists_fn = NULL;
  bool_flag = account_exists_fn(context, address);
  bool_flag = accountExists(context, address);

  athcon_get_storage_fn get_storage_fn = NULL;
  bytes32 = get_storage_fn(context, address, &bytes32);
  bytes32 = getStorage(context, address, &bytes32);

  athcon_set_storage_fn set_storage_fn = NULL;
  storage_status = set_storage_fn(context, address, &bytes32, &bytes32);
  storage_status = setStorage(context, address, &bytes32, &bytes32);

  athcon_get_balance_fn get_balance_fn = NULL;
  uint256be = get_balance_fn(context, address);
  uint256be = getBalance(context, address);

  athcon_call_fn call_fn = NULL;
  result = call_fn(context, message);
  result = call(context, message);

  athcon_get_tx_context_fn get_tx_context_fn = NULL;
  tx_context = get_tx_context_fn(context);
  tx_context = getTxContext(context);

  athcon_get_block_hash_fn get_block_hash_fn = NULL;
  bytes32 = get_block_hash_fn(context, number);
  bytes32 = getBlockHash(context, number);
}
