package athcon

/*
#cgo CFLAGS: -I${SRCDIR}/../../include -Wall -Wextra
#cgo !windows LDFLAGS: -ldl

#include <athcon/athcon.h>
#include <athcon/helpers.h>

#include <stdlib.h> // for 'free'

extern const struct athcon_host_interface athcon_go_host;
*/
import "C"
import (
	"fmt"
	"path/filepath"
	"runtime/cgo"
	"strings"
	"unsafe"

	"github.com/ebitengine/purego"
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
	// handle to the opened shared library. Must be closed with Dlclose.
	libHandle uintptr
	// handle to the VM instance. Must be destroyed with athcon_destroy.
	handle *C.struct_athcon_vm
}

// Load loads the VM from the shared library and returns an instance of VM.
//
// It is the caller's responsibility to call Destroy on the VM instance when it
// is no longer needed.
func Load(path string) (*VM, error) {
	libHandle, err := purego.Dlopen(path, purego.RTLD_NOW|purego.RTLD_GLOBAL)
	if err != nil {
		return nil, fmt.Errorf("loading library: %v", err)
	}

	filename := filepath.Base(path)
	filename = strings.TrimSuffix(filename, filepath.Ext(filename))
	vmName := strings.TrimPrefix(filename, "lib")

	var athcon_create func() *C.struct_athcon_vm
	purego.RegisterLibFunc(&athcon_create, libHandle, "athcon_create_"+vmName)
	vmHandle := athcon_create()

	if vmHandle == nil {
		return nil, fmt.Errorf("failed to create VM")
	}
	return &VM{libHandle: libHandle, handle: vmHandle}, nil
}

// LoadAndConfigure loads the VM from the shared library and configures it with
// the provided options.
//
// It is the caller's responsibility to call Destroy on the VM instance when it
// is no longer needed.
func LoadAndConfigure(filename string, config map[string]string) (vm *VM, err error) {
	vm, err = Load(filename)
	if err != nil {
		return nil, err
	}
	for name, value := range config {
		err = vm.SetOption(name, value)
		if err != nil {
			vm.Destroy()
			return nil, err
		}
	}

	return vm, err
}

func (vm *VM) Destroy() {
	C.athcon_destroy(vm.handle)
	purego.Dlclose(vm.libHandle)
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
	cName := C.CString(name)
	defer C.free(unsafe.Pointer(cName))
	cValue := C.CString(value)
	defer C.free(unsafe.Pointer(cValue))

	r := C.athcon_set_option(vm.handle, cName, cValue)
	switch r {
	case C.ATHCON_SET_OPTION_SUCCESS:
		return nil
	case C.ATHCON_SET_OPTION_INVALID_NAME:
		return fmt.Errorf("athcon: option '%s' not accepted", name)
	case C.ATHCON_SET_OPTION_INVALID_VALUE:
		return fmt.Errorf("athcon: option '%s' has invalid value", name)
	default:
		return fmt.Errorf("athcon: unknown error %d setting option '%s'", r, name)
	}
}

type Result struct {
	Output    []byte
	GasLeft   int64
	GasRefund int64
}

func (vm *VM) Execute(
	ctx HostContext,
	rev Revision,
	kind CallKind,
	depth int,
	gas int64,
	recipient, sender Address,
	input []byte,
	method []byte,
	value Bytes32,
	code []byte,
) (res Result, err error) {
	if len(code) == 0 {
		return res, fmt.Errorf("code is empty")
	}
	msg := C.struct_athcon_message{
		kind:      C.enum_athcon_call_kind(kind),
		depth:     C.int32_t(depth),
		gas:       C.int64_t(gas),
		recipient: athconAddress(recipient),
		sender:    athconAddress(sender),
		value:     athconBytes32(value),
	}
	if len(input) > 0 {
		// Allocate memory for input data in C.
		// Otherwise, the Go garbage collector may move the data around and
		// invalidate the pointer passed to the C code.
		// Without this, the CGO complains `cgo argument has Go pointer to unpinned Go pointer`.
		cInputData := C.malloc(C.size_t(len(input)))
		if cInputData == nil {
			return res, fmt.Errorf("failed to allocate memory for input data")
		}
		defer C.free(cInputData)

		cSlice := unsafe.Slice((*byte)(cInputData), len(input))
		copy(cSlice, input)
		msg.input_data = (*C.uchar)(unsafe.Pointer(&cSlice[0]))
		msg.input_size = C.size_t(len(input))
	}
	if len(method) > 0 {
		// Allocate memory for method name in C.
		cMethodName := C.malloc(C.size_t(len(input)))
		if cMethodName == nil {
			return res, fmt.Errorf("failed to allocate memory for method name")
		}
		defer C.free(cMethodName)

		cSlice := unsafe.Slice((*byte)(cMethodName), len(method))
		copy(cSlice, method)
		msg.method_name = (*C.uchar)(unsafe.Pointer(&cSlice[0]))
		msg.method_name_size = C.size_t(len(method))
	}

	ctxHandle := cgo.NewHandle(ctx)

	hostInterface := newHostInterface()
	result := C.athcon_execute(
		vm.handle,
		hostInterface,
		(*C.struct_athcon_host_context)(unsafe.Pointer(uintptr(ctxHandle))),
		uint32(rev),
		&msg,
		(*C.uint8_t)(unsafe.Pointer(&code[0])),
		C.size_t(len(code)),
	)
	ctxHandle.Delete()

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
