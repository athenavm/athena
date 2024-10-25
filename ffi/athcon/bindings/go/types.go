package athcon

import (
	"encoding/hex"
	"fmt"

	"github.com/ChainSafe/gossamer/pkg/scale"
	"github.com/zeebo/blake3"
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
	var selector MethodSelector
	hasher := blake3.New()
	hasher.Write([]byte(value))
	hasher.Digest().Read(selector[:])

	return selector, nil
}

// String implements the fmt.Stringer interface for MethodSelector, similar to Rust's Display trait.
func (ms MethodSelector) String() string {
	return hex.EncodeToString(ms[:])
}

type ExecutionPayload struct {
	State   []byte
	Payload Payload
}

type Payload struct {
	Selector *MethodSelector
	Input    []byte
}

// EncodedExecutionPayload combines the program state and an already encoded payload into a single byte array
// which is equivalent to a scale-encoded `ExecutionPayload` with the same data.
func EncodedExecutionPayload(state []byte, encodedPayload []byte) []byte {
	encodedState, err := scale.Marshal(state)
	if err != nil {
		panic(fmt.Errorf("unable to encode state: %w", err))
	}

	return append(encodedState, encodedPayload...)
}
