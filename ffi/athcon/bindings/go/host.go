package athcon

/*
#cgo CFLAGS: -I${SRCDIR}/../../include -Wall -Wextra -Wno-unused-parameter

#include <athcon/athcon.h>
#include <athcon/helpers.h>
#include <stdint.h>

// Forward declarations of exported functions that are shared with the VM.
bool accountExists(void *ctx, athcon_address *addr);
athcon_bytes32 getStorage(void *ctx, athcon_address *addr, athcon_bytes32 *key);
enum athcon_storage_status setStorage(void *ctx, athcon_address *addr, athcon_bytes32 *key, athcon_bytes32 *val);
athcon_uint256be getBalance(void *ctx, athcon_address *addr);
struct athcon_tx_context getTxContext(void *ctx);
athcon_bytes32 getBlockHash(void *ctx, long long int number);
struct athcon_result call(void *ctx, struct athcon_message *msg);
athcon_address spawn(void *ctx, uint8_t *blob, size_t len);
*/
import "C"
import (
	"runtime/cgo"
	"unsafe"
)

type CallKind int

const (
	Call CallKind = C.ATHCON_CALL
)

type StorageStatus int

const (
	StorageAssigned         StorageStatus = C.ATHCON_STORAGE_ASSIGNED
	StorageAdded            StorageStatus = C.ATHCON_STORAGE_ADDED
	StorageDeleted          StorageStatus = C.ATHCON_STORAGE_DELETED
	StorageModified         StorageStatus = C.ATHCON_STORAGE_MODIFIED
	StorageDeletedAdded     StorageStatus = C.ATHCON_STORAGE_DELETED_ADDED
	StorageModifiedDeleted  StorageStatus = C.ATHCON_STORAGE_MODIFIED_DELETED
	StorageDeletedRestored  StorageStatus = C.ATHCON_STORAGE_DELETED_RESTORED
	StorageAddedDeleted     StorageStatus = C.ATHCON_STORAGE_ADDED_DELETED
	StorageModifiedRestored StorageStatus = C.ATHCON_STORAGE_MODIFIED_RESTORED
)

type StatusCode int

const (
	AthconSuccess                   StatusCode = C.ATHCON_SUCCESS
	AthconFailure                   StatusCode = C.ATHCON_FAILURE
	AthconRevert                    StatusCode = C.ATHCON_REVERT
	AthconOutOfGas                  StatusCode = C.ATHCON_OUT_OF_GAS
	AthconInvalidInstruction        StatusCode = C.ATHCON_INVALID_INSTRUCTION
	AthconUndefinedInstruction      StatusCode = C.ATHCON_UNDEFINED_INSTRUCTION
	AthconStackOverflow             StatusCode = C.ATHCON_STACK_OVERFLOW
	AthconStackUnderflow            StatusCode = C.ATHCON_STACK_UNDERFLOW
	AthconBadJumpDestination        StatusCode = C.ATHCON_BAD_JUMP_DESTINATION
	AthconInvalidMemoryAccess       StatusCode = C.ATHCON_INVALID_MEMORY_ACCESS
	AthconCallDepthExceeded         StatusCode = C.ATHCON_CALL_DEPTH_EXCEEDED
	AthconStaticModeViolation       StatusCode = C.ATHCON_STATIC_MODE_VIOLATION
	AthconPrecompileFailure         StatusCode = C.ATHCON_PRECOMPILE_FAILURE
	AthconContractValidationFailure StatusCode = C.ATHCON_CONTRACT_VALIDATION_FAILURE
	AthconArgumentOutOfRange        StatusCode = C.ATHCON_ARGUMENT_OUT_OF_RANGE
	AthconUnreachableInstruction    StatusCode = C.ATHCON_UNREACHABLE_INSTRUCTION
	AthconTrap                      StatusCode = C.ATHCON_TRAP
	AthconInsufficientBalance       StatusCode = C.ATHCON_INSUFFICIENT_BALANCE
	AthconInsufficientInput         StatusCode = C.ATHCON_INSUFFICIENT_INPUT
	AthconInvalidSyscallArgument    StatusCode = C.ATHCON_INVALID_SYSCALL_ARGUMENT
	AthconInternalError             StatusCode = C.ATHCON_INTERNAL_ERROR
	AthconRejected                  StatusCode = C.ATHCON_REJECTED
	AthconOutOfMemory               StatusCode = C.ATHCON_OUT_OF_MEMORY
)

// ConvertToGoMessage converts C.struct_athcon_message to AthconMessage
func ConvertToGoMessage(cMsg *C.struct_athcon_message) AthconMessage {
	return AthconMessage{
		Kind:      CallKind(cMsg.kind),
		Depth:     uint32(cMsg.depth),
		Gas:       int64(cMsg.gas),
		Recipient: goAddress(cMsg.recipient),
		Sender:    goAddress(cMsg.sender),
		Input:     goByteSlice(cMsg.input_data, cMsg.input_size),
		Method:    goByteSlice(cMsg.method_name, cMsg.method_name_size),
		Value:     goHash(cMsg.value),
		Code:      goByteSlice(cMsg.code, cMsg.code_size),
	}
}

// ConvertToCResult converts AthconResult to C.struct_athcon_result
func ConvertToCResult(goResult AthconResult) C.struct_athcon_result {
	cResult := C.struct_athcon_result{
		status_code:    C.enum_athcon_status_code(goResult.Status),
		gas_left:       C.int64_t(goResult.GasLeft),
		create_address: cAddress(goResult.CreateAddr),
	}

	if len(goResult.Output) > 0 {
		cResult.output_data = (*C.uint8_t)(&goResult.Output[0])
		cResult.output_size = C.size_t(len(goResult.Output))
	}

	return cResult
}

func cAddress(goAddr Address) C.struct_athcon_address {
	var cAddr C.struct_athcon_address
	for i := 0; i < 20; i++ {
		cAddr.bytes[i] = C.uint8_t(goAddr[i])
	}
	return cAddr
}

func goAddress(in C.athcon_address) Address {
	out := Address{}
	for i := 0; i < len(out); i++ {
		out[i] = byte(in.bytes[i])
	}
	return out
}

func goHash(in C.athcon_bytes32) Bytes32 {
	out := Bytes32{}
	for i := 0; i < len(out); i++ {
		out[i] = byte(in.bytes[i])
	}
	return out
}

func goByteSlice(data *C.uint8_t, size C.size_t) []byte {
	if size == 0 {
		return []byte{}
	}
	return (*[1 << 30]byte)(unsafe.Pointer(data))[:size:size]
}

// TxContext contains information about current transaction and block.
type TxContext struct {
	GasPrice    Bytes32
	Origin      Address
	Coinbase    Address
	BlockHeight int64
	Timestamp   int64
	GasLimit    int64
	ChainID     Bytes32
}

type AthconResult struct {
	Status     StatusCode
	Output     []byte
	GasLeft    int64
	CreateAddr Address
}

type AthconMessage struct {
	Kind      CallKind
	Depth     uint32
	Gas       int64
	Recipient Address
	Sender    Address
	Input     []byte
	Method    []byte
	Value     Bytes32
	Code      []byte
}

type HostContext interface {
	AccountExists(addr Address) bool
	GetStorage(addr Address, key Bytes32) Bytes32
	SetStorage(addr Address, key Bytes32, value Bytes32) StorageStatus
	GetBalance(addr Address) Bytes32
	GetTxContext() TxContext
	GetBlockHash(number int64) Bytes32
	Call(msg AthconMessage) AthconResult
	Spawn(blob []byte) Address
	Deploy(code []byte) Address
}

//export accountExists
func accountExists(pCtx unsafe.Pointer, pAddr *C.athcon_address) C.bool {
	ctx := (*cgo.Handle)(pCtx).Value().(HostContext)
	return C.bool(ctx.AccountExists(goAddress(*pAddr)))
}

//export getStorage
func getStorage(pCtx unsafe.Pointer, pAddr *C.athcon_address, pKey *C.athcon_bytes32) C.athcon_bytes32 {
	ctx := (*cgo.Handle)(pCtx).Value().(HostContext)
	return athconBytes32(ctx.GetStorage(goAddress(*pAddr), goHash(*pKey)))
}

//export setStorage
func setStorage(pCtx unsafe.Pointer, pAddr *C.athcon_address, pKey *C.athcon_bytes32, pVal *C.athcon_bytes32) C.enum_athcon_storage_status {
	ctx := (*cgo.Handle)(pCtx).Value().(HostContext)
	return C.enum_athcon_storage_status(ctx.SetStorage(goAddress(*pAddr), goHash(*pKey), goHash(*pVal)))
}

//export getBalance
func getBalance(pCtx unsafe.Pointer, pAddr *C.athcon_address) C.athcon_uint256be {
	ctx := (*cgo.Handle)(pCtx).Value().(HostContext)
	return athconBytes32(ctx.GetBalance(goAddress(*pAddr)))
}

//export getTxContext
func getTxContext(pCtx unsafe.Pointer) C.struct_athcon_tx_context {
	ctx := (*cgo.Handle)(pCtx).Value().(HostContext)
	txContext := ctx.GetTxContext()

	return C.struct_athcon_tx_context{
		athconBytes32(txContext.GasPrice),
		athconAddress(txContext.Origin),
		C.int64_t(txContext.BlockHeight),
		C.int64_t(txContext.Timestamp),
		C.int64_t(txContext.GasLimit),
		athconBytes32(txContext.ChainID),
	}
}

//export getBlockHash
func getBlockHash(pCtx unsafe.Pointer, number int64) C.athcon_bytes32 {
	ctx := (*cgo.Handle)(pCtx).Value().(HostContext)
	return athconBytes32(ctx.GetBlockHash(number))
}

//export call
func call(pCtx unsafe.Pointer, msg *C.struct_athcon_message) C.struct_athcon_result {
	ctx := (*cgo.Handle)(pCtx).Value().(HostContext)
	// Convert C message to Go message
	goMsg := ConvertToGoMessage(msg)

	// Call the Go function
	goResult := ctx.Call(goMsg)

	// Convert Go result to C result
	return ConvertToCResult(goResult)
}

//export spawn
func spawn(pCtx unsafe.Pointer, pBlob *C.uint8_t, blobSize C.size_t) C.athcon_address {
	ctx := cgo.Handle(pCtx).Value().(HostContext)
	blob := goByteSlice(pBlob, blobSize)
	return athconAddress(ctx.Spawn(blob))
}

//export deploy
func deploy(pCtx unsafe.Pointer, pCode *C.uint8_t, codeSize C.size_t) C.athcon_address {
	ctx := cgo.Handle(pCtx).Value().(HostContext)
	code := goByteSlice(pCode, codeSize)
	return athconAddress(ctx.Deploy(code))
}

func newHostInterface() *C.struct_athcon_host_interface {
	return &C.struct_athcon_host_interface{
		account_exists: (C.athcon_account_exists_fn)(C.accountExists),
		get_storage:    (C.athcon_get_storage_fn)(C.getStorage),
		set_storage:    (C.athcon_set_storage_fn)(C.setStorage),
		get_balance:    (C.athcon_get_balance_fn)(C.getBalance),
		get_tx_context: (C.athcon_get_tx_context_fn)(C.getTxContext),
		get_block_hash: (C.athcon_get_block_hash_fn)(C.getBlockHash),
		call:           (C.athcon_call_fn)(C.call),
		spawn:          (C.athcon_spawn_fn)(C.spawn),
	}
}
