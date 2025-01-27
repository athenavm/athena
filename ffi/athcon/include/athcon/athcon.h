/**
 * AthCon: Athena Client-VM Connector API
 *
 * @defgroup ATHCON ATHCON
 * @{
 */
#ifndef ATHCON_H
#define ATHCON_H

#include <stdbool.h> /* Definition of bool, true and false. */
#include <stddef.h>  /* Definition of size_t. */
#include <stdint.h>  /* Definition of int64_t, uint64_t. */

#ifdef __cplusplus
extern "C"
{
#endif

  /* BEGIN CFFI declarations */

  enum
  {
    /**
     * The ATHCON ABI version number of the interface declared in this file.
     *
     * The ATHCON ABI version always equals the major version number of the ATHCON project.
     * The Host SHOULD check if the ABI versions match when dynamically loading VMs.
     */
    ATHCON_ABI_VERSION = 0
  };

  /**
   * The fixed size array of 32 bytes.
   *
   * 32 bytes of data capable of storing e.g. 256-bit hashes.
   */
  typedef struct athcon_bytes32
  {
    /** The 32 bytes. */
    uint8_t bytes[32];
  } athcon_bytes32;

  /**
   * The alias for athcon_bytes32 to represent a big-endian 256-bit integer.
   */
  typedef struct athcon_bytes32 athcon_uint256be;

  /** Big-endian 192-bit hash suitable for keeping an account address. */
  typedef struct athcon_address
  {
    /** The 24 bytes of the hash. */
    uint8_t bytes[24];
  } athcon_address;

  /** The kind of call-like instruction. */
  enum athcon_call_kind
  {
    ATHCON_CALL = 0, /**< Request CALL. */
  };

  /**
   * The message describing an Athena call, including a zero-depth calls from a transaction origin.
   *
   * Most of the fields are modelled by the section 8. Message Call of the Ethereum Yellow Paper.
   */
  struct athcon_message
  {
    /** The kind of the call. For zero-depth calls ::ATHCON_CALL SHOULD be used. */
    enum athcon_call_kind kind;

    /**
     * The present depth of the message call stack.
     */
    int32_t depth;

    /**
     * The amount of gas available to the message execution.
     */
    int64_t gas;

    /**
     * The recipient of the message.
     *
     * This is the address of the account which storage/balance/nonce is going to be modified
     * by the message execution. In case of ::ATHCON_CALL, this is also the account where the
     * message value athcon_message::value is going to be transferred.
     */
    athcon_address recipient;

    /**
     * The sender of the message.
     *
     * The address of the sender of a message call.
     * This must be the message recipient of the message at the previous (lower) depth,
     * except for the ::ATHCON_DELEGATECALL where recipient is the 2 levels above the present depth.
     * At the depth 0 this must be the transaction origin.
     */
    athcon_address sender;

    athcon_address sender_template;

    /**
     * The message input data.
     *
     * The arbitrary length byte array of the input data of the call.
     * This MAY be NULL.
     */
    const uint8_t *input_data;

    /**
     * The size of the message input data.
     *
     * If input_data is NULL this MUST be 0.
     */
    size_t input_size;

    /**
     * The number of coins transferred with the message.
     *
     * This is transferred value for ::ATHCON_CALL or apparent value for ::ATHCON_DELEGATECALL.
     */
    uint64_t value;
  };

  /** The transaction and block data for execution. */
  struct athcon_tx_context
  {
    uint64_t tx_gas_price;         /**< The transaction gas price. */
    athcon_address tx_origin;      /**< The transaction origin account. */
    int64_t block_height;          /**< The block height. */
    int64_t block_timestamp;       /**< The block timestamp. */
    int64_t block_gas_limit;       /**< The block gas limit. */
    athcon_uint256be chain_id;     /**< The blockchain's ChainID. */
  };

  /**
   * @struct athcon_host_context
   * The opaque data type representing the Host execution context.
   * @see athcon_execute_fn().
   */
  struct athcon_host_context;

  /**
   * Get transaction context callback function.
   *
   *  This callback function is used by a VM to retrieve the transaction and
   *  block context.
   *
   *  @param      context  The pointer to the Host execution context.
   *  @return              The transaction context.
   */
  typedef struct athcon_tx_context (*athcon_get_tx_context_fn)(struct athcon_host_context *context);

  /**
   * Get block hash callback function.
   *
   * This callback function is used by a VM to query the hash of the header of the given block.
   * If the information about the requested block is not available, then this is signalled by
   * returning null bytes.
   *
   * @param context  The pointer to the Host execution context.
   * @param number   The block height.
   * @return         The block hash or null bytes
   *                 if the information about the block is not available.
   */
  typedef athcon_bytes32 (*athcon_get_block_hash_fn)(struct athcon_host_context *context, int64_t number);

  /**
   * Spawn program callback function.
   *
   * This callback function is used by a VM to spawn a new program based on an existing template
   * and initialization params.
   *
   * @param context   The pointer to the Host execution context.
   * @param blob      The program's serialized state blob.
   * @param blob_size The length of the blob, in bytes.
   * @return          The newly-created program address or null bytes
   *                  if the spawn failed.
   */
  typedef athcon_address (*athcon_spawn_fn)(struct athcon_host_context *context, const uint8_t *blob, size_t blob_size);

  /**
   * Deploy program callback function.
   *
   * This callback function is used by a VM to deploy a new program template
   *
   * @param context   The pointer to the Host execution context.
   * @param blob      The program's template (bytecode).
   * @param blob_size The length of the blob, in bytes.
   * @return          The newly-created template address or null bytes
   *                  if the deploy failed.
   */
  typedef athcon_address (*athcon_deploy_fn)(struct athcon_host_context *context, const uint8_t *blob, size_t blob_size);

  /**
   * The execution status code.
   *
   * Successful execution is represented by ::ATHCON_SUCCESS having value 0.
   *
   * Positive values represent failures defined by VM specifications with generic
   * ::ATHCON_FAILURE code of value 1.
   *
   * Status codes with negative values represent VM internal errors
   * not provided by Athena specifications. These errors MUST not be passed back
   * to the caller. They MAY be handled by the Client in predefined manner
   * (see e.g. ::ATHCON_REJECTED), otherwise internal errors are not recoverable.
   * The generic representant of errors is ::ATHCON_INTERNAL_ERROR but
   * an Athena implementation MAY return negative status codes that are not defined
   * in the ATHCON documentation.
   */
  enum athcon_status_code
  {
    /** Execution finished with success. */
    ATHCON_SUCCESS = 0,

    /** Generic execution failure. */
    ATHCON_FAILURE = 1,

    /**
     * Execution terminated with REVERT opcode.
     *
     * In this case the amount of gas left MAY be non-zero and additional output
     * data MAY be provided in ::athcon_result.
     */
    ATHCON_REVERT = 2,

    /** The execution has run out of gas. */
    ATHCON_OUT_OF_GAS = 3,

    /** Execution encountered an invalid instruction. */
    ATHCON_INVALID_INSTRUCTION = 4,

    /** An undefined instruction has been encountered. */
    ATHCON_UNDEFINED_INSTRUCTION = 5,

    /** Stack over/underflow. */
    ATHCON_STACK_OVERFLOW = 6,
    ATHCON_STACK_UNDERFLOW = 7,

    /** Bad jump destination. */
    ATHCON_BAD_JUMP_DESTINATION = 8,

    /**
     * Tried to read outside memory bounds.
     *
     * An example is RETURNDATACOPY reading past the available buffer.
     */
    ATHCON_INVALID_MEMORY_ACCESS = 9,

    /** Call depth has exceeded the limit (if any) */
    ATHCON_CALL_DEPTH_EXCEEDED = 10,

    /** Static mode violation (currently unsupported) */
    ATHCON_STATIC_MODE_VIOLATION = 11,

    /**
     * A call to a precompiled or system contract has ended with a failure.
     *
     * An example: elliptic curve functions handed invalid EC points.
     */
    ATHCON_PRECOMPILE_FAILURE = 12,

    /**
     * Contract validation has failed.
     */
    ATHCON_CONTRACT_VALIDATION_FAILURE = 13,

    /**
     * An argument to a state accessing method has a value outside of the
     * accepted range of values.
     */
    ATHCON_ARGUMENT_OUT_OF_RANGE = 14,

    /** Unreachable instruction. */
    ATHCON_UNREACHABLE_INSTRUCTION = 15,

    /** Trap encountered. */
    ATHCON_TRAP = 16,

    /** The caller does not have enough funds for value transfer. */
    ATHCON_INSUFFICIENT_BALANCE = 17,

    /** A system call tried to read more from STDIN than was available. */
    ATHCON_INSUFFICIENT_INPUT = 18,

    /** A system call was called with invalid arguments. */
    ATHCON_INVALID_SYSCALL_ARGUMENT = 19,

    /** Athena implementation generic internal error. */
    ATHCON_INTERNAL_ERROR = -1,

    /**
     * The execution of the given code and/or message has been rejected
     * by the Athena implementation.
     *
     * This error SHOULD be used to signal that the Athena is not able to or
     * willing to execute the given code type or message.
     * If an Athena returns the ::ATHCON_REJECTED status code,
     * the Client MAY try to execute it in other Athena implementation.
     * For example, the Client tries running a code in the Athena 1.5. If the
     * code is not supported there, the execution falls back to the Athena 1.0.
     */
    ATHCON_REJECTED = -2,

    /** The VM failed to allocate the amount of memory needed for execution. */
    ATHCON_OUT_OF_MEMORY = -3
  };

  /* Forward declaration. */
  struct athcon_result;

  /**
   * Releases resources assigned to an execution result.
   *
   * This function releases memory (and other resources, if any) assigned to the
   * specified execution result making the result object invalid.
   *
   * @param result  The execution result which resources are to be released. The
   *                result itself it not modified by this function, but becomes
   *                invalid and user MUST discard it as well.
   *                This MUST NOT be NULL.
   *
   * @note
   * The result is passed by pointer to avoid (shallow) copy of the ::athcon_result
   * struct. Think of this as the best possible C language approximation to
   * passing objects by reference.
   */
  typedef void (*athcon_release_result_fn)(const struct athcon_result *result);

  /** The Athena code execution result. */
  struct athcon_result
  {
    /** The execution status code. */
    enum athcon_status_code status_code;

    /**
     * The amount of gas left after the execution.
     *
     * If athcon_result::status_code is neither ::ATHCON_SUCCESS nor ::ATHCON_REVERT
     * the value MUST be 0.
     */
    int64_t gas_left;

    /**
     * The reference to output data.
     *
     * The output contains data coming from RETURN opcode (iff athcon_result::code
     * field is ::ATHCON_SUCCESS) or from REVERT opcode.
     *
     * The memory containing the output data is owned by Athena and has to be
     * freed with athcon_result::release().
     *
     * This pointer MAY be NULL.
     * If athcon_result::output_size is 0 this pointer MUST NOT be dereferenced.
     */
    const uint8_t *output_data;

    /**
     * The size of the output data.
     *
     * If athcon_result::output_data is NULL this MUST be 0.
     */
    size_t output_size;

    /**
     * The method releasing all resources associated with the result object.
     *
     * This method (function pointer) is optional (MAY be NULL) and MAY be set
     * by the VM implementation. If set it MUST be called by the user once to
     * release memory and other resources associated with the result object.
     * Once the resources are released the result object MUST NOT be used again.
     *
     * The suggested code pattern for releasing execution results:
     * @code
     * struct athcon_result result = ...;
     * if (result.release)
     *     result.release(&result);
     * @endcode
     *
     * @note
     * It works similarly to C++ virtual destructor. Attaching the release
     * function to the result itself allows VM composition.
     */
    athcon_release_result_fn release;
  };

  /**
   * Check account existence callback function.
   *
   * This callback function is used by the VM to check if
   * there exists an account at given address.
   * @param context  The pointer to the Host execution context.
   * @param address  The address of the account the query is about.
   * @return         true if exists, false otherwise.
   */
  typedef bool (*athcon_account_exists_fn)(struct athcon_host_context *context,
                                           const athcon_address *address);

  /**
   * Get storage callback function.
   *
   * This callback function is used by a VM to query the given account storage entry.
   *
   * @param context  The Host execution context.
   * @param address  The address of the account.
   * @param key      The index of the account's storage entry.
   * @return         The storage value at the given storage key or null bytes
   *                 if the account does not exist.
   */
  typedef athcon_bytes32 (*athcon_get_storage_fn)(struct athcon_host_context *context,
                                                  const athcon_address *address,
                                                  const athcon_bytes32 *key);

  /**
   * The effect of an attempt to modify a contract storage item.
   *
   * See @ref storagestatus for additional information about design of this enum
   * and analysis of the specification.
   *
   * For the purpose of explaining the meaning of each element, the following
   * notation is used:
   * - 0 is zero value,
   * - X != 0 (X is any value other than 0),
   * - Y != 0, Y != X,  (Y is any value other than X and 0),
   * - Z != 0, Z != X, Z != X (Z is any value other than Y and X and 0),
   * - the "o -> c -> v" triple describes the change status in the context of:
   *   - o: original value (cold value before a transaction started),
   *   - c: current storage value,
   *   - v: new storage value to be set.
   *
   * The order of elements follows EIPs introducing net storage gas costs:
   * - EIP-2200: https://eips.ethereum.org/EIPS/eip-2200,
   * - EIP-1283: https://eips.ethereum.org/EIPS/eip-1283.
   */
  enum athcon_storage_status
  {
    /**
     * The new/same value is assigned to the storage item without affecting the cost structure.
     *
     * The storage value item is either:
     * - left unchanged (c == v) or
     * - the dirty value (o != c) is modified again (c != v).
     * This is the group of cases related to minimal gas cost of only accessing warm storage.
     * 0|X   -> 0 -> 0 (current value unchanged)
     * 0|X|Y -> Y -> Y (current value unchanged)
     * 0|X   -> Y -> Z (modified previously added/modified value)
     *
     * This is "catch all remaining" status. I.e. if all other statuses are correctly matched
     * this status should be assigned to all remaining cases.
     */
    ATHCON_STORAGE_ASSIGNED = 0,

    /**
     * A new storage item is added by changing
     * the current clean zero to a nonzero value.
     * 0 -> 0 -> Z
     */
    ATHCON_STORAGE_ADDED = 1,

    /**
     * A storage item is deleted by changing
     * the current clean nonzero to the zero value.
     * X -> X -> 0
     */
    ATHCON_STORAGE_DELETED = 2,

    /**
     * A storage item is modified by changing
     * the current clean nonzero to other nonzero value.
     * X -> X -> Z
     */
    ATHCON_STORAGE_MODIFIED = 3,

    /**
     * A storage item is added by changing
     * the current dirty zero to a nonzero value other than the original value.
     * X -> 0 -> Z
     */
    ATHCON_STORAGE_DELETED_ADDED = 4,

    /**
     * A storage item is deleted by changing
     * the current dirty nonzero to the zero value and the original value is not zero.
     * X -> Y -> 0
     */
    ATHCON_STORAGE_MODIFIED_DELETED = 5,

    /**
     * A storage item is added by changing
     * the current dirty zero to the original value.
     * X -> 0 -> X
     */
    ATHCON_STORAGE_DELETED_RESTORED = 6,

    /**
     * A storage item is deleted by changing
     * the current dirty nonzero to the original zero value.
     * 0 -> Y -> 0
     */
    ATHCON_STORAGE_ADDED_DELETED = 7,

    /**
     * A storage item is modified by changing
     * the current dirty nonzero to the original nonzero value other than the current value.
     * X -> Y -> X
     */
    ATHCON_STORAGE_MODIFIED_RESTORED = 8
  };

  /**
   * Set storage callback function.
   *
   * This callback function is used by a VM to update the given account storage entry.
   * The VM MUST make sure that the account exists. This requirement is only a formality because
   * VM implementations only modify storage of the account of the current execution context
   * (i.e. referenced by athcon_message::recipient).
   *
   * @param context  The pointer to the Host execution context.
   * @param address  The address of the account.
   * @param key      The index of the storage entry.
   * @param value    The value to be stored.
   * @return         The effect on the storage item.
   */
  typedef enum athcon_storage_status (*athcon_set_storage_fn)(struct athcon_host_context *context,
                                                              const athcon_address *address,
                                                              const athcon_bytes32 *key,
                                                              const athcon_bytes32 *value);

  /**
   * Get balance callback function.
   *
   * This callback function is used by a VM to query the balance of the given account.
   *
   * @param context  The pointer to the Host execution context.
   * @param address  The address of the account.
   * @return         The balance of the given account or 0 if the account does not exist.
   */
  typedef uint64_t (*athcon_get_balance_fn)(struct athcon_host_context *context,
                                            const athcon_address *address);

  /**
   * Pointer to the callback function supporting Athena calls.
   *
   * @param context  The pointer to the Host execution context.
   * @param msg      The call parameters.
   * @return         The result of the call.
   */
  typedef struct athcon_result (*athcon_call_fn)(struct athcon_host_context *context,
                                                 const struct athcon_message *msg);

  /**
   * The Host interface.
   *
   * The set of all callback functions expected by VM instances. This is C
   * realisation of vtable for OOP interface (only virtual methods, no data).
   * Host implementations SHOULD create constant singletons of this (similarly
   * to vtables) to lower the maintenance and memory management cost.
   */
  struct athcon_host_interface
  {
    /** Check account existence callback function. */
    athcon_account_exists_fn account_exists;

    /** Get storage callback function. */
    athcon_get_storage_fn get_storage;

    /** Set storage callback function. */
    athcon_set_storage_fn set_storage;

    /** Get balance callback function. */
    athcon_get_balance_fn get_balance;

    /** Call callback function. */
    athcon_call_fn call;

    /** Get transaction context callback function. */
    athcon_get_tx_context_fn get_tx_context;

    /** Get block hash callback function. */
    athcon_get_block_hash_fn get_block_hash;

    /** Spawn program callback function. */
    athcon_spawn_fn spawn;

    /** Deploy program template callback function. */
    athcon_deploy_fn deploy;
  };

  /* Forward declaration. */
  struct athcon_vm;

  /**
   * Destroys the VM instance.
   *
   * @param vm  The VM instance to be destroyed.
   */
  typedef void (*athcon_destroy_fn)(struct athcon_vm *vm);

  /**
   * Possible outcomes of athcon_set_option.
   */
  enum athcon_set_option_result
  {
    ATHCON_SET_OPTION_SUCCESS = 0,
    ATHCON_SET_OPTION_INVALID_NAME = 1,
    ATHCON_SET_OPTION_INVALID_VALUE = 2
  };

  /**
   * Configures the VM instance.
   *
   * Allows modifying options of the VM instance.
   * Options:
   * - code cache behavior: on, off, read-only, ...
   * - optimizations,
   *
   * @param vm     The VM instance to be configured.
   * @param name   The option name. NULL-terminated string. Cannot be NULL.
   * @param value  The new option value. NULL-terminated string. Cannot be NULL.
   * @return       The outcome of the operation.
   */
  typedef enum athcon_set_option_result (*athcon_set_option_fn)(struct athcon_vm *vm,
                                                                char const *name,
                                                                char const *value);

  /**
   * Athena revision.
   *
   * The revision of the Athena specification.
   */
  enum athcon_revision
  {
    /**
     * The Frontier revision.
     */
    ATHCON_FRONTIER = 0,

    /** The maximum Athena revision supported. */
    ATHCON_MAX_REVISION = ATHCON_FRONTIER,

    /**
     * The latest known Athena revision with finalized specification.
     *
     * This is handy for Athena tools to always use the latest revision available.
     */
    ATHCON_LATEST_STABLE_REVISION = ATHCON_FRONTIER
  };

  /**
   * Executes the given code using the input from the message.
   *
   * This function MAY be invoked multiple times for a single VM instance.
   *
   * @param vm         The VM instance. This argument MUST NOT be NULL.
   * @param host       The Host interface. This argument MUST NOT be NULL unless
   *                   the @p vm has the ::ATHCON_CAPABILITY_PRECOMPILES capability.
   * @param context    The opaque pointer to the Host execution context.
   *                   This argument MAY be NULL. The VM MUST pass the same
   *                   pointer to the methods of the @p host interface.
   *                   The VM MUST NOT dereference the pointer.
   * @param rev        The requested Athena specification revision.
   * @param msg        The call parameters. See ::athcon_message. This argument MUST NOT be NULL.
   * @param code       The reference to the code to be executed. This argument MAY be NULL.
   * @param code_size  The length of the code. If @p code is NULL this argument MUST be 0.
   * @return           The execution result.
   */
  typedef struct athcon_result (*athcon_execute_fn)(struct athcon_vm *vm,
                                                    const struct athcon_host_interface *host,
                                                    struct athcon_host_context *context,
                                                    enum athcon_revision rev,
                                                    const struct athcon_message *msg,
                                                    uint8_t const *code,
                                                    size_t code_size);

  /**
   * Possible capabilities of a VM.
   */
  enum athcon_capabilities
  {
    /**
     * The VM is capable of executing Athena1 bytecode.
     */
    ATHCON_CAPABILITY_Athena1 = (1u << 0),
  };

  /**
   * Alias for unsigned integer representing a set of bit flags of ATHCON capabilities.
   *
   * @see athcon_capabilities
   */
  typedef uint32_t athcon_capabilities_flagset;

  /**
   * Return the supported capabilities of the VM instance.
   *
   * This function MAY be invoked multiple times for a single VM instance,
   * and its value MAY be influenced by calls to athcon_vm::set_option.
   *
   * @param vm  The VM instance.
   * @return    The supported capabilities of the VM. @see athcon_capabilities.
   */
  typedef athcon_capabilities_flagset (*athcon_get_capabilities_fn)(struct athcon_vm *vm);

  /**
   * The VM instance.
   *
   * Defines the base struct of the VM implementation.
   */
  struct athcon_vm
  {
    /**
     * ATHCON ABI version implemented by the VM instance.
     *
     * Can be used to detect ABI incompatibilities.
     * The ATHCON ABI version represented by this file is in ::ATHCON_ABI_VERSION.
     */
    const int abi_version;

    /**
     * The name of the ATHCON VM implementation.
     *
     * It MUST be a NULL-terminated not empty string.
     * The content MUST be UTF-8 encoded (this implies ASCII encoding is also allowed).
     */
    const char *name;

    /**
     * The version of the ATHCON VM implementation, e.g. "1.2.3b4".
     *
     * It MUST be a NULL-terminated not empty string.
     * The content MUST be UTF-8 encoded (this implies ASCII encoding is also allowed).
     */
    const char *version;

    /**
     * Pointer to function destroying the VM instance.
     *
     * This is a mandatory method and MUST NOT be set to NULL.
     */
    athcon_destroy_fn destroy;

    /**
     * Pointer to function executing a code by the VM instance.
     *
     * This is a mandatory method and MUST NOT be set to NULL.
     */
    athcon_execute_fn execute;

    /**
     * A method returning capabilities supported by the VM instance.
     *
     * The value returned MAY change when different options are set via the set_option() method.
     *
     * A Client SHOULD only rely on the value returned if it has queried it after
     * it has called the set_option().
     *
     * This is a mandatory method and MUST NOT be set to NULL.
     */
    athcon_get_capabilities_fn get_capabilities;

    /**
     * Optional pointer to function modifying VM's options.
     *
     * If the VM does not support this feature the pointer can be NULL.
     */
    athcon_set_option_fn set_option;
  };

  /* END CFFI declarations */

#ifdef ATHCON_DOCUMENTATION
  /**
   * Example of a function creating an instance of an example Athena implementation.
   *
   * Each Athena implementation MUST provide a function returning an Athena instance.
   * The function SHOULD be named `athcon_create_<vm-name>(void)`. If the VM name contains hyphens
   * replaces them with underscores in the function names.
   *
   * @par Binaries naming convention
   * For VMs distributed as shared libraries, the name of the library SHOULD match the VM name.
   * The conventional library filename prefixes and extensions SHOULD be ignored by the Client.
   * For example, the shared library with the "beta-interpreter" implementation may be named
   * `libbeta-interpreter.so`.
   *
   * @return  The VM instance or NULL indicating instance creation failure.
   */
  struct athcon_vm *athcon_create_example_vm(void);
#endif

  typedef struct athcon_bytes_t
  {
    const uint8_t* ptr;
    size_t size;
  } athcon_bytes;

  void athcon_free_bytes(athcon_bytes* v);

  athcon_bytes* athcon_encode_tx_spawn(const athcon_bytes32* pubkey);
  athcon_bytes* athcon_encode_tx_spend(const athcon_address* recipient, uint64_t amount);

#ifdef __cplusplus
}
#endif

#endif
/** @} */
