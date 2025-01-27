//go:generate cargo build --release --manifest-path ../../../vmlib/Cargo.toml
//go:generate go run ./scripts/copy_lib.go

package athcon

import (
	"fmt"
	"log"
	"os"
	"path/filepath"
	"runtime"
	"testing"

	"github.com/stretchr/testify/require"
)

func libPath(tb testing.TB) string {
	cwd, err := os.Getwd()
	if err != nil {
		log.Fatalf("Failed to get current working directory: %v", err)
	}
	var ext string
	switch runtime.GOOS {
	case "linux":
		ext = "so"
	case "darwin":
		ext = "dylib"
	case "windows":
		ext = "dll"
	}
	require.NotEmpty(tb, ext, "OS not supported")

	return filepath.Join(cwd, fmt.Sprintf("libathena_vmlib.%s", ext))
}

func TestLoad(t *testing.T) {
	t.Run("invalid lib path", func(t *testing.T) {
		vm, err := Load("invalid path")
		require.Error(t, err)
		require.Nil(t, vm)
	})
	t.Run("valid lib path", func(t *testing.T) {
		vm, err := Load(libPath(t))
		require.NoError(t, err)
		defer vm.Destroy()
		require.Equal(t, "Athena", vm.Name())
		require.NotEmpty(t, vm.Version())
		require.Equal(t, "0.1.0", vm.Version())
	})
}

func TestLoadConfigure(t *testing.T) {
	// TODO: would be good if the VM accepted any options to test their behavior
	vm, err := LoadAndConfigure(libPath(t), nil)
	require.NoError(t, err)
	defer vm.Destroy()

	require.Equal(t, "Athena", vm.Name())
	require.NotEmpty(t, vm.Version())
	require.Equal(t, "0.1.0", vm.Version())
}

// Execute with no code is an error.
func TestExecuteEmptyCode(t *testing.T) {
	vm, err := Load(libPath(t))
	require.NoError(t, err)
	defer vm.Destroy()

	addr := Address{}
	result, err := vm.Execute(nil, Frontier, Call, 1, 999, addr, addr, Address{}, nil, 0, nil)

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
	check(Error{}, "success")
	check(Error{Code: 3}, "out of gas")
	check(Error{Code: -1}, "internal error")
	check(Error{Code: 1000}, "<unknown>")
	check(Error{Err: fmt.Errorf("custom error")}, "success: custom error")
	check(Error{Code: 4, Err: fmt.Errorf("custom error")}, "invalid instruction: custom error")
	check(Error{Code: 1000, Err: fmt.Errorf("custom error")}, "<unknown>: custom error")
}

func TestLibraryEncodeTx(t *testing.T) {
	lib, err := LoadLibrary(libPath(t))
	require.NoError(t, err)
	t.Run("spawn", func(t *testing.T) {
		tx := lib.EncodeTxSpawn(Bytes32{9, 8, 7, 6})
		require.NotEmpty(t, tx)

		tx2 := lib.EncodeTxSpawn(Bytes32{1, 2, 3, 4})
		require.NotEqual(t, tx, tx2)
	})
	t.Run("spend", func(t *testing.T) {
		tx := lib.EncodeTxSpend(Address{1, 2, 3, 4}, 191239)
		require.NotEmpty(t, tx)

		tx2 := lib.EncodeTxSpend(Address{1, 2, 3, 4}, 80972)
		require.NotEqual(t, tx, tx2)
	})
}
