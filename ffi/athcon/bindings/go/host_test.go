package athcon

import (
	"crypto/ed25519"
	"crypto/rand"
	_ "embed"
	"encoding/binary"
	"errors"
	"fmt"
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
	vm       *VM
	balances map[Address]uint64
	programs map[Address][]byte
}

func newHost(vm *VM) *testHostContext {
	return &testHostContext{
		vm:       vm,
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
) (output []byte, gasLeft int64, err error) {
	if host.balances[sender] < value {
		return nil, 0, errors.New("insufficient balance")
	}

	host.balances[sender] -= value
	host.balances[recipient] += value

	p, ok := host.programs[recipient]
	if !ok {
		// nobody to call
		return nil, gas, nil
	}

	encoded := EncodedExecutionPayload(nil, input)
	result, err := host.vm.Execute(host, Frontier, Call, depth+1, gas, recipient, sender, encoded, 0, p)
	if err != nil {
		return nil, gas, fmt.Errorf("executing call: %w", err)
	}

	return result.Output, result.GasLeft, nil
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
	vm, _ := Load(libPath(t))
	defer vm.Destroy()

	host := newHost(vm)
	addr := randomAddress()
	host.balances[addr] = 1000
	result, err := vm.Execute(host, Frontier, Call, 1, 100, addr, addr, nil, 0, MINIMAL_TEST_CODE)
	require.NoError(t, err)
	require.EqualValues(t, result.GasLeft, 68)
	require.Len(t, result.Output, 32)

	balance := binary.LittleEndian.Uint64(result.Output)
	require.Equal(t, host.balances[addr], balance)
}

func TestCall(t *testing.T) {
	vm, _ := Load(libPath(t))
	defer vm.Destroy()

	host := newHost(vm)
	addr := randomAddress()
	host.programs[addr] = RECURSIVE_CALL_TEST
	type InputArgs struct {
		Call Address
		N    uint32
	}
	input, err := scale.Marshal(InputArgs{Call: addr, N: 3})
	require.NoError(t, err)

	executionPayload := ExecutionPayload{Payload: Payload{Input: input}}

	encoded, err := scale.Marshal(executionPayload)
	require.NoError(t, err)
	result, err := vm.Execute(host, Frontier, Call, 0, 100000, addr, addr, encoded, 0, RECURSIVE_CALL_TEST)
	require.NoError(t, err)
	require.Len(t, result.Output, 4)
	value := binary.LittleEndian.Uint32(result.Output)
	require.Equal(t, uint32(2), value)
}

func TestSpawn(t *testing.T) {
	vm, _ := Load(libPath(t))
	defer vm.Destroy()

	host := newHost(vm)
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
	vm, _ := Load(libPath(t))
	defer vm.Destroy()

	host := newHost(vm)
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
	vm, _ := Load(libPath(t))
	defer vm.Destroy()

	host := newHost(vm)
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

	result, err := vm.Execute(host, Frontier, Call, 1, 100000, principal, principal, executionPayload, 0, WALLET_TEST)
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

	result, err = vm.Execute(host, Frontier, Call, 1, 100000, principal, principal, executionPayload, 0, WALLET_TEST)
	require.NoError(t, err)
	require.Equal(t, uint8(1), result.Output[0])
}
