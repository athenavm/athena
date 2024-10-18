package athcon

import (
	"encoding/hex"

	"golang.org/x/crypto/blake2b"
)

// Address represents the 24 bytes address of an Athena account.
type Address [24]byte

// Bytes32 represents the 32 bytes of arbitrary data (e.g. the result of Keccak256
// hash). It occasionally is used to represent 256-bit unsigned integer values
// stored in big-endian byte order.
type Bytes32 [32]byte

const MethodSelectorLength = 4

type MethodSelector [MethodSelectorLength]byte

// FromString converts a string to a MethodSelector, similar to the Rust From<&str> implementation.
func FromString(value string) (MethodSelector, error) {
	hash, err := blake2b.New256(nil)
	if err != nil {
		return MethodSelector{}, err
	}
	hash.Write([]byte(value))
	hashBytes := hash.Sum(nil)
	var selector MethodSelector
	copy(selector[:], hashBytes[:MethodSelectorLength])
	return selector, nil
}

// String implements the fmt.Stringer interface for MethodSelector, similar to Rust's Display trait.
func (ms MethodSelector) String() string {
	return hex.EncodeToString(ms[:])
}

type ExecutionPayload struct {
	State   []byte
	Payload []byte
}

type Payload struct {
	Selector *MethodSelector
	Input    []byte
}
