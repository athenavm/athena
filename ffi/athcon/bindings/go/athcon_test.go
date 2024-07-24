//go:generate cargo build --release --manifest-path ../../../vmlib/Cargo.toml

package athcon

import (
	"bytes"
	"testing"
)

var modulePath = "../../../../target/release/libathena_vmlib.so"

func TestLoad(t *testing.T) {
	i, err := Load(modulePath)
	if err != nil {
		t.Fatal(err.Error())
	}
	defer i.Destroy()
	if i.Name() != "example_vm" {
		t.Fatalf("name is %s", i.Name())
	}
	if i.Version()[0] < '0' || i.Version()[0] > '9' {
		t.Fatalf("version number is weird: %s", i.Version())
	}
}

func TestLoadConfigure(t *testing.T) {
	i, err := LoadAndConfigure(modulePath)
	if err != nil {
		t.Fatal(err.Error())
	}
	defer i.Destroy()
	if i.Name() != "example_vm" {
		t.Fatalf("name is %s", i.Name())
	}
	if i.Version()[0] < '0' || i.Version()[0] > '9' {
		t.Fatalf("version number is weird: %s", i.Version())
	}
}

func TestExecuteEmptyCode(t *testing.T) {
	vm, _ := Load(modulePath)
	defer vm.Destroy()

	addr := Address{}
	h := Bytes32{}
	result, err := vm.Execute(nil, Frontier, Call, 1, 999, addr, addr, nil, h, nil)

	if !bytes.Equal(result.Output, []byte("")) {
		t.Errorf("execution unexpected output: %x", result.Output)
	}
	if result.GasLeft != 999 {
		t.Errorf("execution gas left is incorrect: %d", result.GasLeft)
	}
	if err != nil {
		t.Errorf("execution returned unexpected error: %v", err)
	}
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
