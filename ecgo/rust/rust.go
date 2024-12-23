package rust

import (
	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo/field"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo/irsource"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo/layered"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo/rust/wrapper"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo/utils"
)

type WitnessSolver struct {
	r     *wrapper.RustObj
	field field.Field
}

type RustFieldArray struct {
	r     *wrapper.RustObj
	field field.Field
	n     int
}

func Compile(rc *irsource.RootCircuit) (*WitnessSolver, *layered.RootCircuit, error) {
	s := irsource.SerializeRootCircuit(rc)
	irWgSer, lcSer, err := wrapper.CompileWithRustLib(s, field.GetFieldId(rc.Field))
	if err != nil {
		return nil, nil, err
	}
	ws := &WitnessSolver{r: irWgSer, field: rc.Field}
	lc := layered.DeserializeRootCircuit(lcSer)
	return ws, lc, nil
}

func ProveFile(circuitFilename string, witnessBytes []byte) []byte {
	return wrapper.ProveCircuitFile(circuitFilename, witnessBytes, layered.DetectFieldIdFromFile(circuitFilename))
}

func VerifyFile(circuitFilename string, witnessBytes []byte, proofBytes []byte) bool {
	return wrapper.VerifyCircuitFile(circuitFilename, witnessBytes, proofBytes, layered.DetectFieldIdFromFile(circuitFilename))
}

func LoadFieldArray(data []byte, field_ field.Field) (*RustFieldArray, error) {
	n := len(data) / field_.SerializedLen()
	r, err := wrapper.LoadFieldArray(data, uint64(n), field.GetFieldId(field_))
	if err != nil {
		return nil, err
	}
	return &RustFieldArray{r: r, field: field_, n: n}, nil
}

func DumpFieldArray(rfa *RustFieldArray) ([]byte, error) {
	return wrapper.DumpFieldArray(rfa.r, field.GetFieldId(rfa.field))
}

func LoadWitnessSolver(data []byte) (*WitnessSolver, error) {
	i := utils.NewInputBuf(data)
	fieldId := i.ReadUint64()
	r, err := wrapper.LoadWitnessSolver(data, fieldId)
	if err != nil {
		return nil, err
	}
	return &WitnessSolver{r: r, field: field.GetFieldById(fieldId)}, nil
}

func DumpWitnessSolver(ws *WitnessSolver) ([]byte, error) {
	return wrapper.DumpWitnessSolver(ws.r, field.GetFieldId(ws.field))
}

func SolveWitnessesRaw(ws *WitnessSolver, raw_in *RustFieldArray, n int) (*RustFieldArray, int, int, error) {
	r, ni, npi, err := wrapper.SolveWitnesses(ws.r, raw_in.r, n, field.GetFieldId(ws.field))
	if err != nil {
		return nil, 0, 0, err
	}
	return &RustFieldArray{r: r, field: ws.field, n: n}, ni, npi, nil
}
