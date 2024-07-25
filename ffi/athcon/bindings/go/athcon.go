package athcon

/*
#cgo CFLAGS: -I${SRCDIR}/../../include -Wall -Wextra
#cgo !windows LDFLAGS: -ldl

#include <athcon/athcon.h>
#include <athcon/helpers.h>
#include <athcon/loader.h>

#include <stdlib.h>
#include <string.h>

static inline enum athcon_set_option_result set_option(struct athcon_vm* vm, char* name, char* value)
{
	enum athcon_set_option_result ret = athcon_set_option(vm, name, value);
	free(name);
	free(value);
	return ret;
}

extern const struct athcon_host_interface athcon_go_host;

static struct athcon_result execute_wrapper(struct athcon_vm* vm,
	uintptr_t context_index, enum athcon_revision rev,
	enum athcon_call_kind kind, int32_t depth, int64_t gas,
	const athcon_address* recipient, const athcon_address* sender,
	const uint8_t* input_data, size_t input_size, const athcon_uint256be* value,
	const uint8_t* code, size_t code_size)
{
	struct athcon_message msg = {
		kind,
		depth,
		gas,
		*recipient,
		*sender,
		input_data,
		input_size,
		*value,
		0,     // code
		0,     // code_size
	};

	struct athcon_host_context* context = (struct athcon_host_context*)context_index;
	return athcon_execute(vm, &athcon_go_host, context, rev, &msg, code, code_size);
}
*/
import "C"
import (
	"fmt"
	"sync"
	"unsafe"
)

// Address represents the 24 bytes address of an Athena account.
type Address [24]byte

// Bytes32 represents the 32 bytes of arbitrary data (e.g. the result of Keccak256
// hash). It occasionally is used to represent 256-bit unsigned integer values
// stored in big-endian byte order.
type Bytes32 [32]byte

// Static asserts.
const (
	// The size of athcon_bytes32 equals the size of Bytes32.
	_ = uint(len(Bytes32{}) - C.sizeof_athcon_bytes32)
	_ = uint(C.sizeof_athcon_bytes32 - len(Bytes32{}))

	// The size of athcon_address equals the size of Address.
	_ = uint(len(Address{}) - C.sizeof_athcon_address)
	_ = uint(C.sizeof_athcon_address - len(Address{}))
)

type Error int32

func (err Error) IsInternalError() bool {
	return err < 0
}

func (err Error) Error() string {
	return C.GoString(C.athcon_status_code_to_string(C.enum_athcon_status_code(err)))
}

const (
	Failure = Error(C.ATHCON_FAILURE)
	Revert  = Error(C.ATHCON_REVERT)
)

type Revision int32

const (
	Frontier             Revision = C.ATHCON_FRONTIER
	MaxRevision          Revision = C.ATHCON_MAX_REVISION
	LatestStableRevision Revision = C.ATHCON_LATEST_STABLE_REVISION
)

type VM struct {
	handle *C.struct_athcon_vm
}

func Load(filename string) (vm *VM, err error) {
	cfilename := C.CString(filename)
	loaderErr := C.enum_athcon_loader_error_code(C.ATHCON_LOADER_UNSPECIFIED_ERROR)
	handle := C.athcon_load_and_create(cfilename, &loaderErr)
	C.free(unsafe.Pointer(cfilename))

	if loaderErr == C.ATHCON_LOADER_SUCCESS {
		vm = &VM{handle}
	} else {
		errMsg := C.athcon_last_error_msg()
		if errMsg != nil {
			err = fmt.Errorf("ATHCON loading error: %s", C.GoString(errMsg))
		} else {
			err = fmt.Errorf("ATHCON loading error %d", int(loaderErr))
		}
	}

	return vm, err
}

func LoadAndConfigure(config string) (vm *VM, err error) {
	cconfig := C.CString(config)
	loaderErr := C.enum_athcon_loader_error_code(C.ATHCON_LOADER_UNSPECIFIED_ERROR)
	handle := C.athcon_load_and_configure(cconfig, &loaderErr)
	C.free(unsafe.Pointer(cconfig))

	if loaderErr == C.ATHCON_LOADER_SUCCESS {
		vm = &VM{handle}
	} else {
		errMsg := C.athcon_last_error_msg()
		if errMsg != nil {
			err = fmt.Errorf("ATHCON loading error: %s", C.GoString(errMsg))
		} else {
			err = fmt.Errorf("ATHCON loading error %d", int(loaderErr))
		}
	}

	return vm, err
}

func (vm *VM) Destroy() {
	C.athcon_destroy(vm.handle)
}

func (vm *VM) Name() string {
	// TODO: consider using C.athcon_vm_name(vm.handle)
	return C.GoString(vm.handle.name)
}

func (vm *VM) Version() string {
	// TODO: consider using C.athcon_vm_version(vm.handle)
	return C.GoString(vm.handle.version)
}

type Capability uint32

func (vm *VM) HasCapability(capability Capability) bool {
	return bool(C.athcon_vm_has_capability(vm.handle, uint32(capability)))
}

func (vm *VM) SetOption(name string, value string) (err error) {

	r := C.set_option(vm.handle, C.CString(name), C.CString(value))
	switch r {
	case C.ATHCON_SET_OPTION_INVALID_NAME:
		err = fmt.Errorf("athcon: option '%s' not accepted", name)
	case C.ATHCON_SET_OPTION_INVALID_VALUE:
		err = fmt.Errorf("athcon: option '%s' has invalid value", name)
	case C.ATHCON_SET_OPTION_SUCCESS:
	}
	return err
}

type Result struct {
	Output    []byte
	GasLeft   int64
	GasRefund int64
}

func (vm *VM) Execute(ctx HostContext, rev Revision,
	kind CallKind, depth int, gas int64,
	recipient Address, sender Address, input []byte, value Bytes32,
	code []byte) (res Result, err error) {

	ctxId := addHostContext(ctx)
	// FIXME: Clarify passing by pointer vs passing by value.
	athconRecipient := athconAddress(recipient)
	athconSender := athconAddress(sender)
	athconValue := athconBytes32(value)
	result := C.execute_wrapper(vm.handle, C.uintptr_t(ctxId), uint32(rev),
		C.enum_athcon_call_kind(kind), C.int32_t(depth), C.int64_t(gas),
		&athconRecipient, &athconSender, bytesPtr(input), C.size_t(len(input)), &athconValue,
		bytesPtr(code), C.size_t(len(code)))
	removeHostContext(ctxId)

	res.Output = C.GoBytes(unsafe.Pointer(result.output_data), C.int(result.output_size))
	res.GasLeft = int64(result.gas_left)
	if result.status_code != C.ATHCON_SUCCESS {
		err = Error(result.status_code)
	}

	if result.release != nil {
		C.athcon_release_result(&result)
	}

	return res, err
}

var (
	hostContextCounter uintptr
	hostContextMap     = map[uintptr]HostContext{}
	hostContextMapMu   sync.Mutex
)

func addHostContext(ctx HostContext) uintptr {
	hostContextMapMu.Lock()
	id := hostContextCounter
	hostContextCounter++
	hostContextMap[id] = ctx
	hostContextMapMu.Unlock()
	return id
}

func removeHostContext(id uintptr) {
	hostContextMapMu.Lock()
	delete(hostContextMap, id)
	hostContextMapMu.Unlock()
}

func getHostContext(idx uintptr) HostContext {
	hostContextMapMu.Lock()
	ctx := hostContextMap[idx]
	hostContextMapMu.Unlock()
	return ctx
}

func athconBytes32(in Bytes32) C.athcon_bytes32 {
	out := C.athcon_bytes32{}
	for i := 0; i < len(in); i++ {
		out.bytes[i] = C.uint8_t(in[i])
	}
	return out
}

func athconAddress(address Address) C.athcon_address {
	r := C.athcon_address{}
	for i := 0; i < len(address); i++ {
		r.bytes[i] = C.uint8_t(address[i])
	}
	return r
}

func bytesPtr(bytes []byte) *C.uint8_t {
	if len(bytes) == 0 {
		return nil
	}
	return (*C.uint8_t)(unsafe.Pointer(&bytes[0]))
}
