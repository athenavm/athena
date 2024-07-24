package athcon

/*
#include "../../athcon.h"
*/
import "C"

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

type Revision int32

const (
	Frontier             Revision = C.ATHCON_FRONTIER
	MaxRevision          Revision = C.ATHCON_MAX_REVISION
	LatestStableRevision Revision = C.ATHCON_LATEST_STABLE_REVISION
)

type VM struct {
	handle *C.struct_athcon_vm
}
