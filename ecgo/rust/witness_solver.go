package rust

import (
	"fmt"
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
	Values                    []*big.Int
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
		vars = append(vars, vecPub...)
		vars = append(vars, vecSec...)
	}
	o := utils.OutputBuf{}
	bnlen := ws.field.SerializedLen()
	o.AppendUint64(uint64(len(vars)))
	for _, v := range vars {
		o.AppendBigInt(bnlen, ws.field.ToBigInt(v))
	}
	arr, err := LoadFieldArray(o.Bytes(), ws.field)
	if err != nil {
		return nil, err
	}
	wit, err := SolveWitnessesRaw(ws, arr, len(assignments))
	if err != nil {
		return nil, err
	}
	fmt.Println(wit)
	// TODO
	return nil, nil
}

func (ws *WitnessSolver) Serialize() []byte {
	panic("not implemented")
}

func (w *Witness) Serialize() []byte {
	panic("not implemented")
}
