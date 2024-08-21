package wrapper

/*
#include <stdlib.h>
#include <dlfcn.h>
#include "./wrapper.h"
*/
import "C"
import (
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"runtime"
	"sync"
	"time"
	"unsafe"

	"github.com/consensys/gnark/logger"
)

const ABI_VERSION = 2

func currentFileDirectory() string {
	_, fileName, _, ok := runtime.Caller(1)
	if !ok {
		panic("can't get current file directory")
	}
	dir, _ := filepath.Split(fileName)
	return dir
}

var compilePtr unsafe.Pointer = nil
var compilePtrLock sync.Mutex

func downloadFile(url string, filepath string) error {
	out, err := os.Create(filepath)
	if err != nil {
		return err
	}
	defer out.Close()

	resp, err := http.Get(url)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("bad status: %s", resp.Status)
	}

	_, err = io.Copy(out, resp.Body)
	if err != nil {
		return err
	}

	return nil
}

func getUrl(url string) ([]byte, error) {
	resp, err := http.Get(url)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("bad status: %s", resp.Status)
	}

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}
	return body, nil
}

func downloadLib(path string) {
	log := logger.Logger()
	log.Info().Msg("Downloading rust libs ...")
	err := downloadFile("https://github.com/PolyhedraZK/ExpanderCompilerCollection/raw/rust-built-libs/libec_go_lib.so", path)
	if err != nil {
		os.Remove(path)
		panic(err)
	}
}

type repoInfo struct {
	Commit struct {
		Commit struct {
			Committer struct {
				Date string `json:"date"`
			} `json:"committer"`
		} `json:"commit"`
	} `json:"commit"`
}

func updateLib(path string) {
	stat, err := os.Stat(path)
	fileExists := !os.IsNotExist(err)
	if err != nil && fileExists {
		panic(err)
	}
	data, err := getUrl("https://api.github.com/repos/PolyhedraZK/ExpanderCompilerCollection/branches/rust-built-libs")
	if err != nil {
		if fileExists {
			return
		}
		panic(err)
	}
	var repoInfo repoInfo
	err = json.Unmarshal(data, &repoInfo)
	if err != nil {
		if fileExists {
			return
		}
		panic(err)
	}
	remoteTime, err := time.Parse(time.RFC3339, repoInfo.Commit.Commit.Committer.Date)
	if err != nil {
		if fileExists {
			return
		}
		panic(err)
	}
	localTime := stat.ModTime()
	if localTime.After(remoteTime) {
		return
	}
	downloadLib(path)
}

func initCompilePtr() {
	compilePtrLock.Lock()
	defer compilePtrLock.Unlock()
	if compilePtr != nil {
		return
	}
	curDir := currentFileDirectory()
	soPath := filepath.Join(curDir, "libec_go_lib.so")
	updateLib(soPath)
	handle := C.dlopen(C.CString(soPath), C.RTLD_LAZY)
	if handle == nil {
		panic("failed to load libec_go_lib.so")
	}
	compilePtr = C.dlsym(handle, C.CString("compile"))
	if compilePtr == nil {
		panic("failed to load compile function")
	}
	abiVersionPtr := C.dlsym(handle, C.CString("abi_version"))
	if abiVersionPtr == nil {
		panic("failed to load abi_version function")
	}
	abiVersion := C.abi_version(abiVersionPtr)
	if abiVersion != ABI_VERSION {
		panic("abi_version mismatch, please consider update the go package")
	}
}

func CompileWithRustLib(s []byte, configId uint64) ([]byte, []byte, error) {
	initCompilePtr()

	in := C.ByteArray{data: (*C.uint8_t)(C.CBytes(s)), length: C.uint64_t(len(s))}
	defer C.free(unsafe.Pointer(in.data))

	cr := C.compile(compilePtr, in, C.uint64_t(configId))

	defer C.free(unsafe.Pointer(cr.ir_witness_gen.data))
	defer C.free(unsafe.Pointer(cr.layered.data))
	defer C.free(unsafe.Pointer(cr.error.data))

	irWitnessGen := C.GoBytes(unsafe.Pointer(cr.ir_witness_gen.data), C.int(cr.ir_witness_gen.length))
	layered := C.GoBytes(unsafe.Pointer(cr.layered.data), C.int(cr.layered.length))
	errMsg := C.GoBytes(unsafe.Pointer(cr.error.data), C.int(cr.error.length))

	if len(errMsg) > 0 {
		return nil, nil, errors.New(string(errMsg))
	}

	return irWitnessGen, layered, nil
}
