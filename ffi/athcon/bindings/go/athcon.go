package athcon

/*
#cgo CFLAGS: -I${SRCDIR}/../../include -Wall -Wextra
#cgo !windows LDFLAGS: -ldl

#include <athcon/athcon.h>
#include <athcon/helpers.h>

#include <stdlib.h> // for 'free'
*/
import "C"
import (
	"errors"
	"fmt"
	"runtime/cgo"
	"unsafe"

	"github.com/athenavm/athena/ffi/athcon/bindings/go/internal/load"
	"github.com/ebitengine/purego"
)

// Static asserts.
const (
	// The size of athcon_bytes32 equals the size of Bytes32.
	_ = uint(len(Bytes32{}) - C.sizeof_athcon_bytes32)
	_ = uint(C.sizeof_athcon_bytes32 - len(Bytes32{}))

	// The size of athcon_address equals the size of Address.
	_ = uint(len(Address{}) - C.sizeof_athcon_address)
	_ = uint(C.sizeof_athcon_address - len(Address{}))
)

type Error struct {
	// athcon-compatible error code
	Code int32
	// underlying Go error for additional context
	Err error
}

func (err Error) IsInternalError() bool {
	return err.Code < 0
}

// Implement the Error method to return a string representation
func (err Error) Error() string {
	if err.Err != nil {
		return fmt.Sprintf("%s: %v", C.GoString(C.athcon_status_code_to_string(C.enum_athcon_status_code(err.Code))), err.Err)
	}
	return C.GoString(C.athcon_status_code_to_string(C.enum_athcon_status_code(err.Code)))
}

var (
	Failure             = Error{Code: C.ATHCON_FAILURE}
	Revert              = Error{Code: C.ATHCON_REVERT}
	OutOfGas            = Error{Code: C.ATHCON_OUT_OF_GAS}
	CallDepthExceeded   = Error{Code: C.ATHCON_CALL_DEPTH_EXCEEDED}
	PrecompileFailure   = Error{Code: C.ATHCON_PRECOMPILE_FAILURE}
	InsufficientBalance = Error{Code: C.ATHCON_INSUFFICIENT_BALANCE}
	InternalError       = Error{Code: C.ATHCON_INTERNAL_ERROR}
)

type Revision int32

const (
	Frontier             Revision = C.ATHCON_FRONTIER
	MaxRevision          Revision = C.ATHCON_MAX_REVISION
	LatestStableRevision Revision = C.ATHCON_LATEST_STABLE_REVISION
)

type Library struct {
	// handle to the opened shared library. Must be closed with Dlclose.
	libHandle uintptr

	create    func() *C.struct_athcon_vm
	freeBytes func(*C.athcon_bytes)
}

func LoadLibrary(path string) (*Library, error) {
	libHandle, err := load.LoadLibrary(path)
	if err != nil {
		return nil, fmt.Errorf("loading library: %v", err)
	}

	lib := &Library{
		libHandle: libHandle,
	}
	purego.RegisterLibFunc(&lib.create, libHandle, "athcon_create")
	purego.RegisterLibFunc(&lib.freeBytes, libHandle, "athcon_free_bytes")

	return lib, nil
}

func (l *Library) Close() error {
	return load.CloseLibrary(l.libHandle)
}

type VM struct {
	Lib *Library
	// handle to the VM instance. Must be destroyed with athcon_destroy.
	handle *C.struct_athcon_vm
}

// Load loads the VM from the shared library and returns an instance of VM.
//
// It is the caller's responsibility to call Destroy on the VM instance when it
// is no longer needed.
func Load(path string) (*VM, error) {
	lib, err := LoadLibrary(path)
	if err != nil {
		return nil, err
	}
	vmHandle := lib.create()
	if vmHandle == nil {
		return nil, fmt.Errorf("failed to create VM")
	}
	return &VM{Lib: lib, handle: vmHandle}, nil
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

func (vm *VM) Destroy() error {
	C.athcon_destroy(vm.handle)
	return vm.Lib.Close()
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
	recipient, sender, sender_template Address,
	input []byte,
	value uint64,
	code []byte,
) (res Result, err error) {
	if len(code) == 0 {
		return res, Error{
			Code: C.ATHCON_FAILURE,
			Err:  errors.New("athcon execute: no input code"),
		}
	}
	msg := C.struct_athcon_message{
		kind:            C.enum_athcon_call_kind(kind),
		depth:           C.int32_t(depth),
		gas:             C.int64_t(gas),
		recipient:       *athconAddress(recipient),
		sender:          *athconAddress(sender),
		sender_template: *athconAddress(sender_template),
		value:           C.uint64_t(value),
	}
	if len(input) > 0 {
		// Allocate memory for input data in C.
		// Otherwise, the Go garbage collector may move the data around and
		// invalidate the pointer passed to the C code.
		// Without this, the CGO complains `cgo argument has Go pointer to unpinned Go pointer`.
		cInputData := C.CBytes(input)
		defer C.free(cInputData)
		msg.input_data = (*C.uchar)(cInputData)
		msg.input_size = C.size_t(len(input))
	}

	ctxHandle := cgo.NewHandle(ctx)

	cCode := C.CBytes(code)
	defer C.free(cCode)

	hostInterface := newHostInterface()
	result := C.athcon_execute(
		vm.handle,
		hostInterface,
		(*C.struct_athcon_host_context)(unsafe.Pointer(&ctxHandle)),
		uint32(rev),
		&msg,
		(*C.uchar)(cCode),
		C.size_t(len(code)),
	)
	ctxHandle.Delete()

	res.Output = C.GoBytes(unsafe.Pointer(result.output_data), C.int(result.output_size))
	res.GasLeft = int64(result.gas_left)
	if result.status_code != C.ATHCON_SUCCESS {
		err = Error{Code: result.status_code}
	}

	if result.release != nil {
		C.athcon_release_result(&result)
	}

	return res, err
}

func athconBytes32(in Bytes32) *C.athcon_bytes32 {
	var out C.athcon_bytes32
	for i := 0; i < len(in); i++ {
		out.bytes[i] = C.uint8_t(in[i])
	}
	return &out
}

func athconAddress(address Address) *C.athcon_address {
	var out C.athcon_address
	for i := 0; i < len(address); i++ {
		out.bytes[i] = C.uint8_t(address[i])
	}
	return &out
}
