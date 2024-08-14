package compile

import (
	"github.com/PolyhedraZK/ExpanderCompilerCollection/go_libs"

	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo/field"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo/irsource"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo/irwg"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo/layered"
)

func Compile(rc *irsource.RootCircuit) (*irwg.RootCircuit, *layered.RootCircuit, error) {
	s := irsource.SerializeRootCircuit(rc)
	irWgSer, lcSer, err := go_libs.CompileWithRustLib(s, field.GetFieldId(rc.Field))
	if err != nil {
		return nil, nil, err
	}
	irWg := irwg.DeserializeRootCircuit(irWgSer)
	lc := layered.DeserializeRootCircuit(lcSer)
	return irWg, lc, nil
}
