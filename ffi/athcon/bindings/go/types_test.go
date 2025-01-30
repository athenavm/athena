package athcon

import (
	"testing"

	"github.com/ChainSafe/gossamer/pkg/scale"
	"github.com/stretchr/testify/require"
)

func TestEncoding(t *testing.T) {
	selector := FromString("example_method")

	payload := Payload{
		Selector: &selector,
		Input:    []byte("example_input_data"),
	}

	encoded, err := scale.Marshal(payload)
	require.NoError(t, err)

	var unmarshaled Payload
	err = scale.Unmarshal(encoded, &unmarshaled)
	require.NoError(t, err)
	require.Equal(t, payload, unmarshaled)

}
