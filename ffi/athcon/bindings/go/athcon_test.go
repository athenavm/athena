//go:generate cargo build --release --manifest-path ../../../vmlib/Cargo.toml
//go:generate cp ../../../../target/release/libathena_vmlib.so ./libathenavmwrapper.so

package athcon

import (
	"log"
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/require"
)

var modulePath string

func init() {
	cwd, err := os.Getwd()
	if err != nil {
		log.Fatalf("Failed to get current working directory: %v", err)
	}
	modulePath = filepath.Join(cwd, "libathenavmwrapper.so")
	log.Printf("modulePath: %s", modulePath)
}

func TestLoad(t *testing.T) {
	t.Run("invalid lib path", func(t *testing.T) {
		vm, err := Load("invalid path")
		require.Error(t, err)
		require.Nil(t, vm)
	})
	t.Run("valid lib path", func(t *testing.T) {
		vm, err := Load(modulePath)
		require.NoError(t, err)
		defer vm.Destroy()
		require.Equal(t, "Athena", vm.Name())
		require.NotEmpty(t, vm.Version())
		require.Equal(t, "0.1.0", vm.Version())
	})
}

func TestLoadConfigure(t *testing.T) {
	// TODO: would be good if the VM accepted any options to test their behavior
	vm, err := LoadAndConfigure(modulePath, nil)
	require.NoError(t, err)
	defer vm.Destroy()

	require.Equal(t, "Athena", vm.Name())
	require.NotEmpty(t, vm.Version())
	require.Equal(t, "0.1.0", vm.Version())
}

// Execute with no code is an error.
func TestExecuteEmptyCode(t *testing.T) {
	vm, err := Load(modulePath)
	require.NoError(t, err)
	defer vm.Destroy()

	addr := Address{}
	h := Bytes32{}
	result, err := vm.Execute(nil, Frontier, Call, 1, 999, addr, addr, nil, nil, h, nil)

	require.Error(t, err)
	require.Empty(t, result.Output)
	require.Zero(t, result.GasLeft)
}

func TestRevision(t *testing.T) {
	if MaxRevision != Frontier {
		t.Errorf("missing constant for revision %d", MaxRevision)
	}
	if LatestStableRevision != Frontier {
		t.Errorf("wrong latest stable revision %d", LatestStableRevision)
	}
}

func TestErrorMessage(t *testing.T) {
	check := func(err Error, expectedMsg string) {
		if err.Error() != expectedMsg {
			t.Errorf("wrong error message: '%s', expected: '%s'", err.Error(), expectedMsg)
		}
	}

	check(Failure, "failure")
	check(Revert, "revert")
	check(Error(3), "out of gas")
	check(Error(-1), "internal error")
	check(Error(1000), "<unknown>")
}

func TestLibraryEncodeTx(t *testing.T) {
	lib, err := LoadLibrary(modulePath)
	require.NoError(t, err)
	t.Run("different txs", func(t *testing.T) {
		tx1 := lib.EncodeTx(Address{1, 2, 3, 4}, &Address{5, 6, 7, 8}, 1222, []byte("send"), []byte("data"))
		tx2 := lib.EncodeTx(Address{1, 2, 3, 4}, &Address{5, 6, 7, 8}, 1222, []byte("deploy"), []byte("data"))
		require.NotEqual(t, tx1, tx2)
	})
	t.Run("all optionals nil", func(t *testing.T) {
		tx := lib.EncodeTx(Address{1, 2, 3, 4}, nil, 1222, nil, nil)
		require.NotEmpty(t, tx)
	})
}
