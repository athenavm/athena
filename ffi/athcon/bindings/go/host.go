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
uint64_t getBalance(void *ctx, athcon_address *addr);
struct athcon_tx_context getTxContext(void *ctx);
athcon_bytes32 getBlockHash(void *ctx, long long int number);
struct athcon_result call(void *ctx, struct athcon_message *msg);
athcon_address spawn(void *ctx, uint8_t *blob, size_t len);
*/
import "C"
import (
	"fmt"
	"os"
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
	GasPrice    uint64
	Origin      Address
	Coinbase    Address
	BlockHeight int64
	Timestamp   int64
	GasLimit    int64
	ChainID     Bytes32
}

type HostContext interface {
	AccountExists(addr Address) bool
	GetStorage(addr Address, key Bytes32) Bytes32
	SetStorage(addr Address, key Bytes32, value Bytes32) StorageStatus
	GetBalance(addr Address) uint64
	GetTxContext() TxContext
	GetBlockHash(number int64) Bytes32
	Call(kind CallKind, recipient Address, sender Address, value uint64, input []byte, gas int64, depth int) (
		output []byte, gasLeft int64, err error)
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
	return *athconBytes32(ctx.GetStorage(goAddress(*pAddr), goHash(*pKey)))
}

//export setStorage
func setStorage(pCtx unsafe.Pointer, pAddr *C.athcon_address, pKey *C.athcon_bytes32, pVal *C.athcon_bytes32) C.enum_athcon_storage_status {
	ctx := (*cgo.Handle)(pCtx).Value().(HostContext)
	return C.enum_athcon_storage_status(ctx.SetStorage(goAddress(*pAddr), goHash(*pKey), goHash(*pVal)))
}

//export getBalance
func getBalance(pCtx unsafe.Pointer, pAddr *C.athcon_address) C.uint64_t {
	ctx := (*cgo.Handle)(pCtx).Value().(HostContext)
	return C.uint64_t(ctx.GetBalance(goAddress(*pAddr)))
}

//export getTxContext
func getTxContext(pCtx unsafe.Pointer) C.struct_athcon_tx_context {
	ctx := (*cgo.Handle)(pCtx).Value().(HostContext)
	txContext := ctx.GetTxContext()

	return C.struct_athcon_tx_context{
		C.uint64_t(txContext.GasPrice),
		*athconAddress(txContext.Origin),
		C.int64_t(txContext.BlockHeight),
		C.int64_t(txContext.Timestamp),
		C.int64_t(txContext.GasLimit),
		*athconBytes32(txContext.ChainID),
	}
}

//export getBlockHash
func getBlockHash(pCtx unsafe.Pointer, number int64) C.athcon_bytes32 {
	ctx := (*cgo.Handle)(pCtx).Value().(HostContext)
	return *athconBytes32(ctx.GetBlockHash(number))
}

//export call
func call(pCtx unsafe.Pointer, msg *C.struct_athcon_message) C.struct_athcon_result {
	ctx := (*cgo.Handle)(pCtx).Value().(HostContext)

	kind := CallKind(msg.kind)
	output, gasLeft, err := ctx.Call(kind, goAddress(msg.recipient), goAddress(msg.sender), uint64(msg.value),
		goByteSlice(msg.input_data, msg.input_size), int64(msg.gas), int(msg.depth))

	statusCode := C.enum_athcon_status_code(0)
	if err != nil {
		// Wrap unknown error types with a catch-all type
		if e, ok := err.(Error); ok {
			statusCode = C.enum_athcon_status_code(e.Code)
		} else {
			fmt.Fprintf(os.Stderr, "Caught unknown error: %v", err)
			statusCode = C.ATHCON_INTERNAL_ERROR
		}
	}

	outputData := (*C.uint8_t)(nil)
	if len(output) > 0 {
		outputData = (*C.uint8_t)(&output[0])
	}

	result := C.athcon_make_result(statusCode, C.int64_t(gasLeft), outputData, C.size_t(len(output)))
	return result
}

//export spawn
func spawn(pCtx unsafe.Pointer, pBlob *C.uint8_t, blobSize C.size_t) C.athcon_address {
	ctx := (*cgo.Handle)(pCtx).Value().(HostContext)
	blob := C.GoBytes(unsafe.Pointer(pBlob), C.int(blobSize))
	return *athconAddress(ctx.Spawn(blob))
}

//export deploy
func deploy(pCtx unsafe.Pointer, pCode *C.uint8_t, codeSize C.size_t) C.athcon_address {
	ctx := (*cgo.Handle)(pCtx).Value().(HostContext)
	code := C.GoBytes(unsafe.Pointer(pCode), C.int(codeSize))
	return *athconAddress(ctx.Deploy(code))
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
