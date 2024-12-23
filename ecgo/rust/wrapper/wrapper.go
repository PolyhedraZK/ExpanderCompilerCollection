package wrapper

/*
#include <stdlib.h>
#include <dlfcn.h>
#include "./wrapper.h"
*/
import "C"
import (
	"bytes"
	"errors"
	"fmt"
	"path/filepath"
	"runtime"
	"sync"
	"unsafe"
)

const ABI_VERSION = 5

var compilePtr unsafe.Pointer = nil
var proveCircuitFilePtr unsafe.Pointer = nil
var verifyCircuitFilePtr unsafe.Pointer = nil
var freeObjectPtr unsafe.Pointer = nil
var loadFieldArrayPtr unsafe.Pointer = nil
var dumpFieldArrayPtr unsafe.Pointer = nil
var loadWitnessSolverPtr unsafe.Pointer = nil
var dumpWitnessSolverPtr unsafe.Pointer = nil
var solveWitnessesPtr unsafe.Pointer = nil

var functionsPtrLock sync.Mutex

type RustObj struct {
	ptr unsafe.Pointer
}

func NewRustObj(ptr unsafe.Pointer) *RustObj {
	res := &RustObj{ptr: ptr}
	runtime.SetFinalizer(res, func(obj *RustObj) {
		C.free_object(freeObjectPtr, obj.ptr)
	})
	return res
}

func getLibName() string {
	switch runtime.GOOS {
	case "darwin":
		if runtime.GOARCH == "arm64" {
			return "libec_go_lib.dylib"
		}
	case "linux":
		if runtime.GOARCH == "amd64" {
			return "libec_go_lib.so"
		}
	}
	panic(fmt.Sprintf("unsupported platform %s %s", runtime.GOOS, runtime.GOARCH))
}

func initCompilePtr() {
	functionsPtrLock.Lock()
	defer functionsPtrLock.Unlock()
	if compilePtr != nil {
		return
	}
	cacheDir, err := getCacheDir()
	if err != nil {
		panic(fmt.Sprintf("failed to get cache dir: %v", err))
	}
	soPath := filepath.Join(cacheDir, getLibName())
	updateLib(soPath)
	handle := C.dlopen(C.CString(soPath), C.RTLD_LAZY)
	if handle == nil {
		panic("failed to load libec_go_lib, you may need to install openmpi")
	}
	abiVersionPtr := C.dlsym(handle, C.CString("abi_version"))
	if abiVersionPtr == nil {
		panic("failed to load abi_version function")
	}
	abiVersion := C.abi_version(abiVersionPtr)
	if abiVersion != ABI_VERSION {
		panic("abi_version mismatch, please consider update the go package")
	}

	// other functions
	compilePtr = C.dlsym(handle, C.CString("compile"))
	if compilePtr == nil {
		panic("failed to load compile function")
	}
	proveCircuitFilePtr = C.dlsym(handle, C.CString("prove_circuit_file"))
	if proveCircuitFilePtr == nil {
		panic("failed to load prove_circuit_file function")
	}
	verifyCircuitFilePtr = C.dlsym(handle, C.CString("verify_circuit_file"))
	if verifyCircuitFilePtr == nil {
		panic("failed to load verify_circuit_file function")
	}
	freeObjectPtr = C.dlsym(handle, C.CString("free_object"))
	if freeObjectPtr == nil {
		panic("failed to load free_object function")
	}
	loadFieldArrayPtr = C.dlsym(handle, C.CString("load_field_array"))
	if loadFieldArrayPtr == nil {
		panic("failed to load load_field_array function")
	}
	dumpFieldArrayPtr = C.dlsym(handle, C.CString("dump_field_array"))
	if dumpFieldArrayPtr == nil {
		panic("failed to load dump_field_array function")
	}
	loadWitnessSolverPtr = C.dlsym(handle, C.CString("load_witness_solver"))
	if loadWitnessSolverPtr == nil {
		panic("failed to load load_witness_solver function")
	}
	dumpWitnessSolverPtr = C.dlsym(handle, C.CString("dump_witness_solver"))
	if dumpWitnessSolverPtr == nil {
		panic("failed to load dump_witness_solver function")
	}
	solveWitnessesPtr = C.dlsym(handle, C.CString("solve_witnesses"))
	if solveWitnessesPtr == nil {
		panic("failed to load solve_witnesses function")
	}
}

// from c to go
func goBytes(data *C.uint8_t, length C.uint64_t) []byte {
	return bytes.Clone(unsafe.Slice((*byte)(data), length))
}

func CompileWithRustLib(s []byte, configId uint64) (*RustObj, []byte, error) {
	initCompilePtr()

	in := C.ByteArray{data: (*C.uint8_t)(C.CBytes(s)), length: C.uint64_t(len(s))}
	defer C.free(unsafe.Pointer(in.data))

	cr := C.compile(compilePtr, in, C.uint64_t(configId))

	defer C.free(unsafe.Pointer(cr.layered.data))
	defer C.free(unsafe.Pointer(cr.error.data))

	witnessSolver := NewRustObj(cr.witness_solver)
	layered := goBytes(cr.layered.data, cr.layered.length)
	errMsg := goBytes(cr.error.data, cr.error.length)

	if len(errMsg) > 0 {
		return nil, nil, errors.New(string(errMsg))
	}

	return witnessSolver, layered, nil
}

func ProveCircuitFile(circuitFilename string, witness []byte, configId uint64) []byte {
	initCompilePtr()
	bytesFn := []byte(circuitFilename)
	cf := C.ByteArray{data: (*C.uint8_t)(C.CBytes(bytesFn)), length: C.uint64_t(len(bytesFn))}
	defer C.free(unsafe.Pointer(cf.data))
	wi := C.ByteArray{data: (*C.uint8_t)(C.CBytes(witness)), length: C.uint64_t(len(witness))}
	defer C.free(unsafe.Pointer(wi.data))
	proof := C.prove_circuit_file(proveCircuitFilePtr, cf, wi, C.uint64_t(configId))
	defer C.free(unsafe.Pointer(proof.data))
	return goBytes(proof.data, proof.length)
}

func VerifyCircuitFile(circuitFilename string, witness []byte, proof []byte, configId uint64) bool {
	initCompilePtr()
	bytesFn := []byte(circuitFilename)
	cf := C.ByteArray{data: (*C.uint8_t)(C.CBytes(bytesFn)), length: C.uint64_t(len(bytesFn))}
	defer C.free(unsafe.Pointer(cf.data))
	wi := C.ByteArray{data: (*C.uint8_t)(C.CBytes(witness)), length: C.uint64_t(len(witness))}
	defer C.free(unsafe.Pointer(wi.data))
	pr := C.ByteArray{data: (*C.uint8_t)(C.CBytes(proof)), length: C.uint64_t(len(proof))}
	defer C.free(unsafe.Pointer(pr.data))
	return C.verify_circuit_file(verifyCircuitFilePtr, cf, wi, pr, C.uint64_t(configId)) != 0
}

func LoadFieldArray(data []byte, length uint64, configId uint64) (*RustObj, error) {
	initCompilePtr()
	in := C.ByteArray{data: (*C.uint8_t)(C.CBytes(data)), length: C.uint64_t(len(data))}
	defer C.free(unsafe.Pointer(in.data))
	ptrRes := C.load_field_array(loadFieldArrayPtr, in, C.uint64_t(length), C.uint64_t(configId))
	defer C.free(unsafe.Pointer(ptrRes.error.data))
	if ptrRes.error.length > 0 {
		return nil, errors.New(string(goBytes(ptrRes.error.data, ptrRes.error.length)))
	}
	return NewRustObj(ptrRes.pointer), nil
}

func DumpFieldArray(obj *RustObj, configId uint64) ([]byte, error) {
	initCompilePtr()
	res := C.dump_field_array(dumpFieldArrayPtr, obj.ptr, C.uint64_t(configId))
	defer C.free(unsafe.Pointer(res.result.data))
	defer C.free(unsafe.Pointer(res.error.data))
	if res.error.length > 0 {
		return nil, errors.New(string(goBytes(res.error.data, res.error.length)))
	}
	return goBytes(res.result.data, res.result.length), nil
}

func LoadWitnessSolver(data []byte, configId uint64) (*RustObj, error) {
	initCompilePtr()
	in := C.ByteArray{data: (*C.uint8_t)(C.CBytes(data)), length: C.uint64_t(len(data))}
	defer C.free(unsafe.Pointer(in.data))
	ptrRes := C.load_witness_solver(loadWitnessSolverPtr, in, C.uint64_t(configId))
	defer C.free(unsafe.Pointer(ptrRes.error.data))
	if ptrRes.error.length > 0 {
		return nil, errors.New(string(goBytes(ptrRes.error.data, ptrRes.error.length)))
	}
	return NewRustObj(ptrRes.pointer), nil
}

func DumpWitnessSolver(obj *RustObj, configId uint64) ([]byte, error) {
	initCompilePtr()
	res := C.dump_witness_solver(dumpWitnessSolverPtr, obj.ptr, C.uint64_t(configId))
	defer C.free(unsafe.Pointer(res.result.data))
	defer C.free(unsafe.Pointer(res.error.data))
	if res.error.length > 0 {
		return nil, errors.New(string(goBytes(res.error.data, res.error.length)))
	}
	return goBytes(res.result.data, res.result.length), nil
}

func SolveWitnesses(ws *RustObj, raw_in *RustObj, n int, configId uint64) (*RustObj, int, int, error) {
	initCompilePtr()
	ptrRes := C.solve_witnesses(solveWitnessesPtr, ws.ptr, raw_in.ptr, C.uint64_t(n), C.hintCallBack, C.uint64_t(configId))
	defer C.free(unsafe.Pointer(ptrRes.error.data))
	if ptrRes.error.length > 0 {
		return nil, 0, 0, errors.New(string(goBytes(ptrRes.error.data, ptrRes.error.length)))
	}
	return NewRustObj(ptrRes.witness_vec), int(ptrRes.num_inputs_per_witness), int(ptrRes.num_public_inputs_per_witness), nil
}
