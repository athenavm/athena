package athcon

import (
	"bytes"
	"testing"
)

type testHostContext struct{}

func (host *testHostContext) AccountExists(addr Address) bool {
	return false
}

func (host *testHostContext) GetStorage(addr Address, key Bytes32) Bytes32 {
	return Bytes32{}
}

func (host *testHostContext) SetStorage(addr Address, key Bytes32, value Bytes32) (status StorageStatus) {
	return StorageAssigned
}

func (host *testHostContext) GetBalance(addr Address) Bytes32 {
	return Bytes32{}
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
	recipient Address, sender Address, value Bytes32, input []byte, gas int64, depth int) (
	output []byte, gasLeft int64, createAddr Address, err error) {
	output = []byte("output from testHostContext.Call()")
	return output, gas, Address{}, nil
}

func TestGetBalance(t *testing.T) {
	code := []byte{
		0x7f, 0x41, 0x54, 0x48, // "\x7fATH" magic number
		0x13, 0x05, 0x00, 0x10, // 10000513 (ADDI x10, x0, 0x100) // load address to write result
		0xa3, 0x00, 0x22, 0x93, // 04000293 (ADDI x5, x0, 0xa3)   // load syscall number
		0x73, 0x00, 0x00, 0x00, // 00000073 (ECALL)
	}

	vm, _ := Load(modulePath)
	defer vm.Destroy()

	host := &testHostContext{}
	addr := Address{}
	h := Bytes32{}
	result, err := vm.Execute(host, Frontier, Call, 1, 100, addr, addr, nil, h, code)
	output := result.Output
	gasLeft := result.GasLeft

	if len(output) != 32 {
		t.Errorf("unexpected output size: %d", len(output))
	}

	// Should return value 42 (0x2a) as defined in GetTxContext().
	expectedOutput := []byte("\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x2a")
	if !bytes.Equal(output, expectedOutput) {
		t.Errorf("execution unexpected output: %x", output)
	}
	if gasLeft != 94 {
		t.Errorf("execution gas left is incorrect: %d", gasLeft)
	}
	if err != nil {
		t.Errorf("execution returned unexpected error: %v", err)
	}
}

func TestGetBlockHeightFromTxContext(t *testing.T) {
	// Yul: mstore(0, number()) return(0, msize())
	code := []byte("\x43\x60\x00\x52\x59\x60\x00\xf3")

	vm, _ := Load(modulePath)
	defer vm.Destroy()

	host := &testHostContext{}
	addr := Address{}
	h := Bytes32{}
	result, err := vm.Execute(host, Frontier, Call, 1, 100, addr, addr, nil, h, code)
	output := result.Output
	gasLeft := result.GasLeft

	if len(output) != 32 {
		t.Errorf("unexpected output size: %d", len(output))
	}

	// Should return value 42 (0x2a) as defined in GetTxContext().
	expectedOutput := []byte("\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x2a")
	if !bytes.Equal(output, expectedOutput) {
		t.Errorf("execution unexpected output: %x", output)
	}
	if gasLeft != 94 {
		t.Errorf("execution gas left is incorrect: %d", gasLeft)
	}
	if err != nil {
		t.Errorf("execution returned unexpected error: %v", err)
	}
}

func TestCall(t *testing.T) {
	// pseudo-Yul: call(0, 0, 0, 0, 0, 0, 34) return(0, msize())
	code := []byte("\x60\x22\x60\x00\x80\x80\x80\x80\x80\xf1\x59\x60\x00\xf3")

	vm, _ := Load(modulePath)
	defer vm.Destroy()

	host := &testHostContext{}
	addr := Address{}
	h := Bytes32{}
	result, err := vm.Execute(host, Frontier, Call, 1, 100, addr, addr, nil, h, code)
	output := result.Output
	gasLeft := result.GasLeft

	if len(output) != 34 {
		t.Errorf("execution unexpected output length: %d", len(output))
	}
	if !bytes.Equal(output, []byte("output from testHostContext.Call()")) {
		t.Errorf("execution unexpected output: %s", output)
	}
	if gasLeft != 89 {
		t.Errorf("execution gas left is incorrect: %d", gasLeft)
	}
	if err != nil {
		t.Errorf("execution returned unexpected error: %v", err)
	}
}