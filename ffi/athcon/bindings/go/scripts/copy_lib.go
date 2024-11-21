package main

import (
	"fmt"
	"os"
	"runtime"
)

func main() {
	var (
		srcPath string
		dstPath string
	)
	switch runtime.GOOS {
	case "linux":
		srcPath = "../../../../target/release/libathena_vmlib.so"
		dstPath = "libathenavmwrapper.so"
	case "darwin":
		srcPath = "../../../../target/release/libathena_vmlib.dylib"
		dstPath = "libathenavmwrapper.dylib"
	case "windows":
		srcPath = "../../../../target/release/athena_vmlib.dll"
		dstPath = "libathenavmwrapper.dll"
	default:
		fmt.Printf("Unsupported operating system: %s", runtime.GOOS)
		os.Exit(1)
	}

	fmt.Printf("+ Copying %s to %s\n", srcPath, dstPath)
	input, err := os.ReadFile(srcPath)
	if err != nil {
		fmt.Printf("Error reading source file: %v\n", err)
		os.Exit(1)
	}

	err = os.WriteFile(dstPath, input, 0644)
	if err != nil {
		fmt.Printf("Error writing destination file: %v\n", err)
		os.Exit(1)
	}
}
