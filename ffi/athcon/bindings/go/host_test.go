package athcon

import (
	"bytes"
	_ "embed"
	"encoding/binary"
	"testing"
)

//go:generate cp ../../../../tests/minimal/getbalance.bin .
//go:embed getbalance.bin
var MINIMAL_TEST_CODE []byte

//go:generate cp ../../../../tests/recursive_call/elf/recursive-call-test ./recursive-call-test.bin
//go:embed recursive-call-test.bin
var RECURSIVE_CALL_TEST []byte

type testHostContext struct{}

func (host *testHostContext) AccountExists(addr Address) bool {
	return false
}

func (host *testHostContext) GetStorage(addr Address, key Bytes32) Bytes32 {
	return Bytes32{}
}

func (host *testHostContext) SetStorage(addr Address, key Bytes32, value Bytes32) (status StorageStatus) {
	return StorageAdded
}

func (host *testHostContext) GetBalance(addr Address) uint64 {
	return 42
}

func (host *testHostContext) GetTxContext() TxContext {
	txContext := TxContext{}
	txContext.BlockHeight = 42
	return txContext
}

func (host *testHostContext) GetBlockHash(number int64) Bytes32 {
	return Bytes32{}
}

func (host *testHostContext) Call(kind CallKind,
	recipient Address, sender Address, value uint64, input []byte, gas int64, depth int) (
	output []byte, gasLeft int64, createAddr Address, err error) {
	return nil, gas, Address{}, nil
}

func (host *testHostContext) Spawn(blob []byte) Address {
	return Address{}
}

func (host *testHostContext) Deploy(code []byte) Address {
	return Address{}
}

// TestGetBalance tests the GetBalance() host function. It's a minimal test that
// only executes a few instructions.
func TestGetBalance(t *testing.T) {
	vm, _ := Load(modulePath)
	defer vm.Destroy()

	host := &testHostContext{}
	addr := Address{}
	result, err := vm.Execute(host, Frontier, Call, 1, 100, addr, addr, nil, 0, MINIMAL_TEST_CODE)
	output := result.Output
	gasLeft := result.GasLeft

	if len(output) != 32 {
		t.Errorf("unexpected output size: %d", len(output))
	}

	// Should return value 42 (0x2a) as defined in GetTxContext().
	var expectedOutput Bytes32
	binary.LittleEndian.PutUint32(expectedOutput[:], 42)
	if !bytes.Equal(output, expectedOutput[:]) {
		t.Errorf("expected output: %x", expectedOutput)
		t.Errorf("unexpected output: %x", output)
	}
	if gasLeft != 68 {
		t.Errorf("execution gas left is incorrect: %d", gasLeft)
	}
	if err != nil {
		t.Errorf("execution returned unexpected error: %v", err)
	}
}

func TestCall(t *testing.T) {
	vm, _ := Load(modulePath)
	defer vm.Destroy()

	host := &testHostContext{}
	addr := Address{}
	result, err := vm.Execute(host, Frontier, Call, 1, 10000, addr, addr, []byte{0, 0, 0, 0, 2, 0, 0, 0}, 0, RECURSIVE_CALL_TEST)
	output := result.Output

	if len(output) != 4 {
		t.Errorf("execution unexpected output length: %d", len(output))
	}
	if err != nil {
		t.Errorf("execution returned unexpected error: %v", err)
	}
}
