//go:build windows

package load

import "syscall"

func LoadLibrary(path string) (uintptr, error) {
	handle, err := syscall.LoadLibrary(path)
	return uintptr(handle), err
}

func CloseLibrary(handle uintptr) error {
	return syscall.FreeLibrary(syscall.Handle(handle))
}
