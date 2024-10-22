package athcon

import (
	"bytes"
	"crypto/ed25519"
	"crypto/rand"
	_ "embed"
	"encoding/binary"
	"errors"
	"testing"

	"github.com/ChainSafe/gossamer/pkg/scale"
	"github.com/stretchr/testify/require"
)

//go:generate cp ../../../../tests/minimal/getbalance.bin .
//go:embed getbalance.bin
var MINIMAL_TEST_CODE []byte

//go:generate cp ../../../../tests/recursive_call/elf/recursive-call-test ./recursive-call-test.bin
//go:embed recursive-call-test.bin
var RECURSIVE_CALL_TEST []byte

//go:generate cp ../../../../examples/wallet/program/elf/wallet-template ./wallet-template.bin
//go:embed wallet-template.bin
var WALLET_TEST []byte

type testHostContext struct {
	balances map[Address]uint64
	programs map[Address][]byte
}

func newHost() *testHostContext {
	return &testHostContext{
		balances: make(map[Address]uint64),
		programs: make(map[Address][]byte),
	}

}

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
	return host.balances[addr]
}

func (host *testHostContext) GetTxContext() TxContext {
	txContext := TxContext{}
	txContext.BlockHeight = 42
	return txContext
}

func (host *testHostContext) GetBlockHash(number int64) Bytes32 {
	return Bytes32{}
}

func (host *testHostContext) Call(
	kind CallKind,
	recipient Address,
	sender Address,
	value uint64,
	input []byte,
	gas int64,
	depth int,
) (output []byte, gasLeft int64, createAddr Address, err error) {
	if host.balances[sender] < value {
		return nil, 0, Address{}, errors.New("insufficient balance")
	}

	host.balances[sender] -= value
	host.balances[recipient] += value
	return nil, gas, Address{}, nil
}

func (host *testHostContext) Spawn(blob []byte) Address {
	addr := randomAddress()
	host.programs[addr] = blob
	return addr
}

func (host *testHostContext) Deploy(code []byte) Address {
	return Address{}
}

func randomAddress() Address {
	var a Address
	_, err := rand.Read(a[:])
	if err != nil {
		panic(err.Error())
	}
	return a
}

// TestGetBalance tests the GetBalance() host function. It's a minimal test that
// only executes a few instructions.
func TestGetBalance(t *testing.T) {
	vm, _ := Load(modulePath)
	defer vm.Destroy()

	host := newHost()
	addr := randomAddress()
	host.balances[addr] = 1000
	result, err := vm.Execute(host, Frontier, Call, 1, 100, addr, addr, nil, 0, MINIMAL_TEST_CODE)
	require.NoError(t, err)
	output := result.Output
	require.EqualValues(t, result.GasLeft, 68)
	require.Len(t, output, 32)

	// Should return value 42 (0x2a) as defined in GetTxContext().
	var expectedOutput Bytes32
	binary.LittleEndian.PutUint32(expectedOutput[:], 1000)
	if !bytes.Equal(output, expectedOutput[:]) {
		t.Errorf("expected output: %x", expectedOutput)
		t.Errorf("unexpected output: %x", output)
	}
	if err != nil {
		t.Errorf("execution returned unexpected error: %v", err)
	}
}

func TestCall(t *testing.T) {
	vm, _ := Load(modulePath)
	defer vm.Destroy()

	host := newHost()
	addr := Address{}
	payload := Payload{Input: []byte{2, 0, 0, 0}}
	executionPayload := ExecutionPayload{Payload: payload}

	encoded, err := scale.Marshal(executionPayload)
	require.NoError(t, err)
	result, err := vm.Execute(host, Frontier, Call, 1, 100000, addr, addr, encoded, 0, RECURSIVE_CALL_TEST)
	output := result.Output

	if len(output) != 4 {
		t.Errorf("execution unexpected output length: %d", len(output))
	}
	if err != nil {
		t.Errorf("execution returned unexpected error: %v", err)
	}
}

func TestSpawn(t *testing.T) {
	vm, _ := Load(modulePath)
	defer vm.Destroy()

	host := newHost()
	principal := randomAddress()
	pubkey := Bytes32([32]byte{1, 1, 2, 2, 3, 3, 4, 4})

	payload := vm.Lib.EncodeTxSpawn(pubkey)
	executionPayload := EncodedExecutionPayload(nil, payload)

	result, err := vm.Execute(host, Frontier, Call, 1, 1000000, principal, principal, executionPayload, 0, WALLET_TEST)
	require.NoError(t, err)
	require.Len(t, result.Output, 24)

	require.Contains(t, host.programs, Address(result.Output))
}

func TestSpend(t *testing.T) {
	vm, _ := Load(modulePath)
	defer vm.Destroy()

	host := newHost()
	// Step 1: Spawn wallet
	principal := Address{1, 2, 3, 4}
	var walletAddress Address
	{
		pubkey := Bytes32([32]byte{1, 1, 2, 2, 3, 3, 4, 4})
		executionPayload := EncodedExecutionPayload(nil, vm.Lib.EncodeTxSpawn(pubkey))

		result, err := vm.Execute(host, Frontier, Call, 1, 10000, principal, principal, executionPayload, 0, WALLET_TEST)
		require.NoError(t, err)
		require.Len(t, result.Output, 24)

		walletAddress = Address(result.Output)
	}
	// Step 2: Send coins to another account
	// Give some coins to `principal`
	host.balances[principal] = 1000
	recipient := randomAddress()

	executionPayload := EncodedExecutionPayload(
		host.programs[walletAddress],
		vm.Lib.EncodeTxSpend(recipient, 100),
	)
	_, err := vm.Execute(host, Frontier, Call, 1, 10000, principal, principal, executionPayload, 0, WALLET_TEST)
	require.NoError(t, err)

	// Step 3: Check balance
	require.Equal(t, host.balances[recipient], uint64(100))
	require.Equal(t, host.balances[principal], uint64(900))
}

func TestVerify(t *testing.T) {
	vm, _ := Load(modulePath)
	defer vm.Destroy()

	host := newHost()
	// Step 1: Spawn wallet
	principal := randomAddress()
	pubkey, privkey, err := ed25519.GenerateKey(nil)
	var walletAddress Address
	{
		pubkey := Bytes32(pubkey)
		executionPayload := EncodedExecutionPayload(nil, vm.Lib.EncodeTxSpawn(pubkey))

		result, err := vm.Execute(host, Frontier, Call, 1, 10000000, principal, principal, executionPayload, 0, WALLET_TEST)
		require.NoError(t, err)
		require.Len(t, result.Output, 24)

		walletAddress = Address(result.Output)
	}
	// Step 2: Try verify TX with invalid signature
	tx := make([]byte, 100)
	_, err = rand.Read(tx)
	require.NoError(t, err)
	txEncoded, err := scale.Marshal(tx)
	require.NoError(t, err)

	var signature [64]byte
	signatureEncoded, err := scale.Marshal(signature)
	require.NoError(t, err)

	selector, err := FromString("athexp_verify")
	require.NoError(t, err)
	payload := Payload{
		Selector: &selector,
		Input:    append(txEncoded, signatureEncoded...),
	}
	payloadEncoded, err := scale.Marshal(payload)
	require.NoError(t, err)

	executionPayload := EncodedExecutionPayload(host.programs[walletAddress], payloadEncoded)

	result, err := vm.Execute(host, Frontier, Call, 1, 1000000000, principal, principal, executionPayload, 0, WALLET_TEST)
	require.NoError(t, err)
	require.Zero(t, result.Output[0])

	// Step 3: Try verify TX with valid signature
	copy(signature[:], ed25519.Sign(privkey, tx))
	signatureEncoded, err = scale.Marshal(signature)
	require.NoError(t, err)

	payload = Payload{
		Selector: &selector,
		Input:    append(txEncoded, signatureEncoded...),
	}
	payloadEncoded, err = scale.Marshal(payload)
	require.NoError(t, err)

	executionPayload = EncodedExecutionPayload(host.programs[walletAddress], payloadEncoded)

	result, err = vm.Execute(host, Frontier, Call, 1, 1000000000, principal, principal, executionPayload, 0, WALLET_TEST)
	require.NoError(t, err)
	require.Equal(t, uint8(1), result.Output[0])
}
