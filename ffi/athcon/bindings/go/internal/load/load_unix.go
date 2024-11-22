//go:build (darwin || freebsd || linux) && !android && !faketime

package load

import "github.com/ebitengine/purego"

func LoadLibrary(path string) (uintptr, error) {
	return purego.Dlopen(path, purego.RTLD_NOW|purego.RTLD_GLOBAL)
}

func CloseLibrary(handle uintptr) error {
	return purego.Dlclose(handle)
}
