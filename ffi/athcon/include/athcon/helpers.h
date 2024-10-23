/**
 * ATHCON Helpers
 *
 * A collection of C helper functions for invoking a VM instance methods.
 * These are convenient for languages where invoking function pointers
 * is "ugly" or impossible (such as Go).
 *
 * @defgroup helpers ATHCON Helpers
 * @{
 */
#pragma once

#include <athcon/athcon.h>
#include <stdlib.h>
#include <string.h>

#ifdef __cplusplus
extern "C"
{
#ifdef __GNUC__
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wold-style-cast"
#endif
#endif

  /**
   * Returns true if the VM has a compatible ABI version.
   */
  static inline bool athcon_is_abi_compatible(struct athcon_vm *vm)
  {
    return vm->abi_version == ATHCON_ABI_VERSION;
  }

  /**
   * Returns the name of the VM.
   */
  static inline const char *athcon_vm_name(struct athcon_vm *vm)
  {
    return vm->name;
  }

  /**
   * Returns the version of the VM.
   */
  static inline const char *athcon_vm_version(struct athcon_vm *vm)
  {
    return vm->version;
  }

  /**
   * Checks if the VM has the given capability.
   *
   * @see athcon_get_capabilities_fn
   */
  static inline bool athcon_vm_has_capability(struct athcon_vm *vm, enum athcon_capabilities capability)
  {
    return (vm->get_capabilities(vm) & (athcon_capabilities_flagset)capability) != 0;
  }

  /**
   * Destroys the VM instance.
   *
   * @see athcon_destroy_fn
   */
  static inline void athcon_destroy(struct athcon_vm *vm)
  {
    vm->destroy(vm);
  }

  /**
   * Sets the option for the VM, if the feature is supported by the VM.
   *
   * @see athcon_set_option_fn
   */
  static inline enum athcon_set_option_result athcon_set_option(struct athcon_vm *vm,
                                                                char const *name,
                                                                char const *value)
  {
    if (vm->set_option)
      return vm->set_option(vm, name, value);
    return ATHCON_SET_OPTION_INVALID_NAME;
  }

  /**
   * Executes code in the VM instance.
   *
   * @see athcon_execute_fn.
   */
  static inline struct athcon_result athcon_execute(struct athcon_vm *vm,
                                                    const struct athcon_host_interface *host,
                                                    struct athcon_host_context *context,
                                                    enum athcon_revision rev,
                                                    const struct athcon_message *msg,
                                                    uint8_t const *code,
                                                    size_t code_size)
  {
    return vm->execute(vm, host, context, rev, msg, code, code_size);
  }

  /// The athcon_result release function using free() for releasing the memory.
  ///
  /// This function is used in the athcon_make_result(),
  /// but may be also used in other case if convenient.
  ///
  /// @param result The result object.
  static void athcon_free_result_memory(const struct athcon_result *result)
  {
    free((uint8_t *)result->output_data);
  }

  /// Creates the result from the provided arguments.
  ///
  /// The provided output is copied to memory allocated with malloc()
  /// and the athcon_result::release function is set to one invoking free().
  ///
  /// In case of memory allocation failure, the result has all fields zeroed
  /// and only athcon_result::status_code is set to ::ATHCON_OUT_OF_MEMORY internal error.
  ///
  /// @param status_code  The status code.
  /// @param gas_left     The amount of gas left.
  /// @param output_data  The pointer to the output.
  /// @param output_size  The output size.
  static inline struct athcon_result athcon_make_result(enum athcon_status_code status_code,
                                                        int64_t gas_left,
                                                        const uint8_t *output_data,
                                                        size_t output_size)
  {
    struct athcon_result result;
    memset(&result, 0, sizeof(result));

    if (output_size != 0)
    {
      uint8_t *buffer = (uint8_t *)malloc(output_size);

      if (!buffer)
      {
        result.status_code = ATHCON_OUT_OF_MEMORY;
        return result;
      }

      memcpy(buffer, output_data, output_size);
      result.output_data = buffer;
      result.output_size = output_size;
      result.release = athcon_free_result_memory;
    }

    result.status_code = status_code;
    result.gas_left = gas_left;
    return result;
  }

  /**
   * Releases the resources allocated to the execution result.
   *
   * @param result  The result object to be released. MUST NOT be NULL.
   *
   * @see athcon_result::release() athcon_release_result_fn
   */
  static inline void athcon_release_result(struct athcon_result *result)
  {
    if (result->release)
      result->release(result);
  }

  /** Returns text representation of the ::athcon_status_code. */
  static inline const char *athcon_status_code_to_string(enum athcon_status_code status_code)
  {
    switch (status_code)
    {
    case ATHCON_SUCCESS:
      return "success";
    case ATHCON_FAILURE:
      return "failure";
    case ATHCON_REVERT:
      return "revert";
    case ATHCON_OUT_OF_GAS:
      return "out of gas";
    case ATHCON_INVALID_INSTRUCTION:
      return "invalid instruction";
    case ATHCON_UNDEFINED_INSTRUCTION:
      return "undefined instruction";
    case ATHCON_STACK_OVERFLOW:
      return "stack overflow";
    case ATHCON_STACK_UNDERFLOW:
      return "stack underflow";
    case ATHCON_BAD_JUMP_DESTINATION:
      return "bad jump destination";
    case ATHCON_INVALID_MEMORY_ACCESS:
      return "invalid memory access";
    case ATHCON_CALL_DEPTH_EXCEEDED:
      return "call depth exceeded";
    case ATHCON_STATIC_MODE_VIOLATION:
      return "static mode violation";
    case ATHCON_PRECOMPILE_FAILURE:
      return "precompile failure";
    case ATHCON_CONTRACT_VALIDATION_FAILURE:
      return "contract validation failure";
    case ATHCON_ARGUMENT_OUT_OF_RANGE:
      return "argument out of range";
    case ATHCON_UNREACHABLE_INSTRUCTION:
      return "unreachable instruction";
    case ATHCON_TRAP:
      return "trap";
    case ATHCON_INSUFFICIENT_BALANCE:
      return "insufficient balance";
    case ATHCON_INTERNAL_ERROR:
      return "internal error";
    case ATHCON_REJECTED:
      return "rejected";
    case ATHCON_OUT_OF_MEMORY:
      return "out of memory";
    case ATHCON_INSUFFICIENT_INPUT:
      return "insufficient input";
    case ATHCON_INVALID_SYSCALL_ARGUMENT:
      return "invalid syscall argument";
    }
    return "<unknown>";
  }

  /** Returns the name of the ::athcon_revision. */
  static inline const char *athcon_revision_to_string(enum athcon_revision rev)
  {
    switch (rev)
    {
    case ATHCON_FRONTIER:
      return "Frontier";
    }
    return "<unknown>";
  }

  /** @} */

#ifdef __cplusplus
#ifdef __GNUC__
#pragma GCC diagnostic pop
#endif
} // extern "C"
#endif
