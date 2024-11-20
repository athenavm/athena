package main

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"
)

func main() {
	var ext string
	switch runtime.GOOS {
	case "linux":
		ext = "so"
	case "darwin":
		ext = "dylib"
	case "windows":
		ext = "dll"
	default:
		fmt.Printf("Unsupported operating system: %s", runtime.GOOS)
		os.Exit(1)
	}

	srcPath := filepath.Join("../../../../target/release", fmt.Sprintf("libathena_vmlib.%s", ext))
	dstPath := filepath.Join(".", fmt.Sprintf("libathenavmwrapper.%s", ext))

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
