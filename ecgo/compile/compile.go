package compile

/*
#cgo LDFLAGS: ${SRCDIR}/lib/libec_go_lib.a -ldl
#include <stdlib.h>
#include "./lib/ec_go.h"
*/
import "C"
import (
	"errors"
	"unsafe"

	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo/field"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo/irsource"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo/irwg"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo/layered"
)

func compileInner(s []byte, configId uint64) ([]byte, []byte, error) {
	in := C.ByteArray{data: (*C.uint8_t)(C.CBytes(s)), length: C.uint64_t(len(s))}
	defer C.free(unsafe.Pointer(in.data))

	cr := C.compile(in, C.uint64_t(configId))

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

func Compile(rc *irsource.RootCircuit) (*irwg.RootCircuit, *layered.RootCircuit, error) {
	s := irsource.SerializeRootCircuit(rc)
	irWgSer, lcSer, err := compileInner(s, field.GetFieldId(rc.Field))
	if err != nil {
		return nil, nil, err
	}
	irWg := irwg.DeserializeRootCircuit(irWgSer)
	lc := layered.DeserializeRootCircuit(lcSer)
	return irWg, lc, nil
}
