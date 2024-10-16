package athcon

import (
	"encoding/hex"
	"fmt"
	"github.com/ChainSafe/gossamer/pkg/scale"
	"github.com/stretchr/testify/require"
	"testing"
)

func TestEncoding(t *testing.T) {
	// Example usage
	selector, err := FromString("example_method")
	fmt.Printf("Selector: %s\n", selector)
	require.NoError(t, err)

	payload := ExecutionPayload{
		Selector: &selector,
		Input:    []byte("example_input_data"),
	}

	encoded, err := scale.Marshal(payload)
	require.NoError(t, err)

	fmt.Println("Encoded Payload:", hex.EncodeToString(encoded))

	var unmarshaled ExecutionPayload
	err = scale.Unmarshal(encoded, &unmarshaled)
	require.NoError(t, err)

	fmt.Printf("Decoded Payload: Selector=%v, Input=%s\n", unmarshaled.Selector, string(unmarshaled.Input))
}