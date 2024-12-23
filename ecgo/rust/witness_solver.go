package rust

import (
	"math/big"
	"reflect"

	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo/field"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo/utils"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/schema"
)

type Witness struct {
	NumWitnesses              int
	NumInputsPerWitness       int
	NumPublicInputsPerWitness int
	Field                     *big.Int
	Values                    *RustFieldArray
}

var TVariable reflect.Type

func init() {
	TVariable = reflect.ValueOf(struct{ A frontend.Variable }{}).FieldByName("A").Type()
}

// GetCircuitVariables reimplements frontend.NewWitness to support fields that are not present in gnark.
func GetCircuitVariables(assignment frontend.Circuit, field field.Field) ([]constraint.Element, []constraint.Element) {
	chPubValues := make(chan any)
	chSecValues := make(chan any)
	go func() {
		schema.Walk(assignment, TVariable, func(leaf schema.LeafInfo, tValue reflect.Value) error {
			if leaf.Visibility == schema.Public {
				chPubValues <- tValue.Interface()
			}
			return nil
		})
		close(chPubValues)
		schema.Walk(assignment, TVariable, func(leaf schema.LeafInfo, tValue reflect.Value) error {
			if leaf.Visibility == schema.Secret {
				chSecValues <- tValue.Interface()
			}
			return nil
		})
		close(chSecValues)
	}()
	resPub := []constraint.Element{}
	for v := range chPubValues {
		resPub = append(resPub, field.FromInterface(v))
	}
	resSec := []constraint.Element{}
	for v := range chSecValues {
		resSec = append(resSec, field.FromInterface(v))
	}
	return resPub, resSec
}

// SolveInput is the entry point to solve the final input of the given assignment using a specified number of threads.
func (ws *WitnessSolver) SolveInput(assignment frontend.Circuit, _ int) (*Witness, error) {
	return ws.SolveInputAuto(assignment)
}

func (ws *WitnessSolver) SolveInputAuto(assignment frontend.Circuit) (*Witness, error) {
	return ws.SolveInputs([]frontend.Circuit{assignment})
}

func (ws *WitnessSolver) SolveInputs(assignments []frontend.Circuit) (*Witness, error) {
	vars := []constraint.Element{}
	for _, assignment := range assignments {
		vecPub, vecSec := GetCircuitVariables(assignment, ws.field)
		vars = append(vars, vecSec...)
		vars = append(vars, vecPub...)
	}
	o := utils.OutputBuf{}
	bnlen := ws.field.SerializedLen()
	for _, v := range vars {
		o.AppendBigInt(bnlen, ws.field.ToBigInt(v))
	}
	arr, err := LoadFieldArray(o.Bytes(), ws.field)
	if err != nil {
		return nil, err
	}
	wit, ni, npi, err := SolveWitnessesRaw(ws, arr, len(assignments))
	if err != nil {
		return nil, err
	}
	return &Witness{
		NumWitnesses:              len(assignments),
		NumInputsPerWitness:       ni,
		NumPublicInputsPerWitness: npi,
		Field:                     ws.field.Field(),
		Values:                    wit,
	}, nil
}

func (ws *WitnessSolver) Serialize() []byte {
	data, err := DumpWitnessSolver(ws)
	if err != nil {
		panic(err)
	}
	return data
}

func (w *Witness) Serialize() []byte {
	o := utils.OutputBuf{}
	o.AppendUint64(uint64(w.NumWitnesses))
	o.AppendUint64(uint64(w.NumInputsPerWitness))
	o.AppendUint64(uint64(w.NumPublicInputsPerWitness))
	o.AppendBigInt(32, w.Field)

	bv, err := DumpFieldArray(w.Values)
	if err != nil {
		panic(err)
	}
	return append(o.Bytes(), bv...)
}

func (w *Witness) ValuesSlice() []*big.Int {
	bv, err := DumpFieldArray(w.Values)
	if err != nil {
		panic(err)
	}
	bnlen := field.GetFieldFromOrder(w.Field).SerializedLen()
	i := utils.NewInputBuf(bv)
	res := make([]*big.Int, w.NumWitnesses*(w.NumInputsPerWitness+w.NumPublicInputsPerWitness))
	for j := 0; j < len(res); j++ {
		res[j] = i.ReadBigInt(bnlen)
	}
	return res
}
