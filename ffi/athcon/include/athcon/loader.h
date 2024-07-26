/**
 * ATHCON Loader Library
 *
 * The ATHCON Loader Library supports loading VMs implemented as Dynamically Loaded Libraries
 * (DLLs, shared objects).
 *
 * @defgroup loader ATHCON Loader
 * @{
 */
#pragma once

#ifdef __cplusplus
extern "C"
{
#endif

  /** The function pointer type for ATHCON create functions. */
  typedef struct athcon_vm *(*athcon_create_fn)(void);

  /// Error codes for the ATHCON loader.
  ///
  /// Objects of this type SHOULD be initialized with ::ATHCON_LOADER_UNSPECIFIED_ERROR
  /// before passing to the ATHCON loader.
  enum athcon_loader_error_code
  {
    /** The loader succeeded. */
    ATHCON_LOADER_SUCCESS = 0,

    /** The loader cannot open the given file name. */
    ATHCON_LOADER_CANNOT_OPEN = 1,

    /** The VM create function not found. */
    ATHCON_LOADER_SYMBOL_NOT_FOUND = 2,

    /** The invalid argument value provided. */
    ATHCON_LOADER_INVALID_ARGUMENT = 3,

    /** The creation of a VM instance has failed. */
    ATHCON_LOADER_VM_CREATION_FAILURE = 4,

    /** The ABI version of the VM instance has mismatched. */
    ATHCON_LOADER_ABI_VERSION_MISMATCH = 5,

    /** The VM option is invalid. */
    ATHCON_LOADER_INVALID_OPTION_NAME = 6,

    /** The VM option value is invalid. */
    ATHCON_LOADER_INVALID_OPTION_VALUE = 7,

    /// This error value will be never returned by the ATHCON loader,
    /// but can be used by users to init athcon_loader_error_code objects.
    ATHCON_LOADER_UNSPECIFIED_ERROR = -1
  };

  /**
   * Dynamically loads the ATHCON module with a VM implementation.
   *
   * This function tries to open a dynamically loaded library (DLL) at the given `filename`.
   * On UNIX-like systems dlopen() function is used. On Windows LoadLibrary() function is used.
   *
   * If the file does not exist or is not a valid shared library the ::ATHCON_LOADER_CANNOT_OPEN error
   * code is signaled and NULL is returned.
   *
   * After the DLL is successfully loaded the function tries to find the EVM create function in the
   * library. The `filename` is used to guess the EVM name and the name of the create function.
   * The create function name is constructed by the following rules. Consider example path:
   * "/ethereum/libexample-interpreter.so.1.0".
   * - the filename is taken from the path:
   *   "libexample-interpreter.so.1.0",
   * - the "lib" prefix and all file extensions are stripped from the name:
   *   "example-interpreter"
   * - all "-" are replaced with "_" to construct _base name_:
   *   "example_interpreter",
   * - the function name "athcon_create_" + _base name_ is searched in the library:
   *   "athcon_create_example_interpreter",
   * - if the function is not found, the function name "athcon_create" is searched in the library.
   *
   * If the create function is found in the library, the pointer to the function is returned.
   * Otherwise, the ::ATHCON_LOADER_SYMBOL_NOT_FOUND error code is signaled and NULL is returned.
   *
   * It is safe to call this function with the same filename argument multiple times
   * (the DLL is not going to be loaded multiple times).
   *
   * @param filename    The null terminated path (absolute or relative) to an ATHCON module
   *                    (dynamically loaded library) containing the VM implementation.
   *                    If the value is NULL, an empty C-string or longer than the path maximum length
   *                    the ::ATHCON_LOADER_INVALID_ARGUMENT is signaled.
   * @param error_code  The pointer to the error code. If not NULL the value is set to
   *                    ::ATHCON_LOADER_SUCCESS on success or any other error code as described above.
   * @return            The pointer to the EVM create function or NULL in case of error.
   */
  athcon_create_fn athcon_load(const char *filename, enum athcon_loader_error_code *error_code);

  /**
   * Dynamically loads the ATHCON module and creates the VM instance.
   *
   * This is a macro for creating the VM instance with the function returned from athcon_load().
   * The function signals the same errors as athcon_load() and additionally:
   * - ::ATHCON_LOADER_VM_CREATION_FAILURE when the create function returns NULL,
   * - ::ATHCON_LOADER_ABI_VERSION_MISMATCH when the created VM instance has ABI version different
   *   from the ABI version of this library (::ATHCON_ABI_VERSION).
   *
   * It is safe to call this function with the same filename argument multiple times:
   * the DLL is not going to be loaded multiple times, but the function will return new VM instance
   * each time.
   *
   * @param filename    The null terminated path (absolute or relative) to an ATHCON module
   *                    (dynamically loaded library) containing the VM implementation.
   *                    If the value is NULL, an empty C-string or longer than the path maximum length
   *                    the ::ATHCON_LOADER_INVALID_ARGUMENT is signaled.
   * @param error_code  The pointer to the error code. If not NULL the value is set to
   *                    ::ATHCON_LOADER_SUCCESS on success or any other error code as described above.
   * @return            The pointer to the created VM or NULL in case of error.
   */
  struct athcon_vm *athcon_load_and_create(const char *filename, enum athcon_loader_error_code *error_code);

  /**
   * Dynamically loads the ATHCON module, then creates and configures the VM instance.
   *
   * This function performs the following actions atomically:
   * - loads the ATHCON module (as athcon_load()),
   * - creates the VM instance,
   * - configures the VM instance with options provided in the @p config parameter.
   *
   * The configuration string (@p config) has the following syntax:
   *
   *     <path> ("," <option-name> ["=" <option-value>])*
   *
   * In this syntax, an option without a value can be specified (`,option,`)
   * as a shortcut for using empty value (`,option=,`).
   *
   * Options are passed to a VM in the order they are specified in the configuration string.
   * It is up to the VM implementation how to handle duplicated options and other conflicts.
   *
   * Example configuration string:
   *
   *     ./modules/vm.so,engine=compiler,trace,verbosity=2
   *
   * The function signals the same errors as athcon_load_and_create() and additionally:
   * - ::ATHCON_LOADER_INVALID_OPTION_NAME
   *   when the provided options list contains an option unknown for the VM,
   * - ::ATHCON_LOADER_INVALID_OPTION_VALUE
   *   when there exists unsupported value for a given VM option.

   *
   * @param config      The path to the ATHCON module with additional configuration options.
   * @param error_code  The pointer to the error code. If not NULL the value is set to
   *                    ::ATHCON_LOADER_SUCCESS on success or any other error code as described above.
   * @return            The pointer to the created VM or NULL in case of error.
   */
  struct athcon_vm *athcon_load_and_configure(const char *config,
                                              enum athcon_loader_error_code *error_code);

  /**
   * Returns the human-readable message describing the most recent error
   * that occurred in ATHCON loading since the last call to this function.
   *
   * In case any loading function returned ::ATHCON_LOADER_SUCCESS this function always returns NULL.
   * In case of error code other than success returned, this function MAY return the error message.
   * Calling this function "consumes" the error message and the function will return NULL
   * from subsequent invocations.
   * This function is not thread-safe.
   *
   * @return Error message or NULL if no additional information is available.
   *         The returned pointer MUST NOT be freed by the caller.
   */
  const char *athcon_last_error_msg(void);

#ifdef __cplusplus
}
#endif

/** @} */
