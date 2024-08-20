package irwg

import (
	"errors"
	"math/big"
	"reflect"

	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo/field"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo/utils"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/schema"
)

// Witness represents the solved values of the circuit's inputs.
type Witness []*big.Int

var tVariable reflect.Type

func init() {
	tVariable = reflect.ValueOf(struct{ A frontend.Variable }{}).FieldByName("A").Type()
}

// GetCircuitVariables reimplements frontend.NewWitness to support fields that are not present in gnark.
func GetCircuitVariables(assignment frontend.Circuit, field field.Field) []constraint.Element {
	chValues := make(chan any)
	go func() {
		defer close(chValues)
		schema.Walk(assignment, tVariable, func(leaf schema.LeafInfo, tValue reflect.Value) error {
			if leaf.Visibility == schema.Public {
				chValues <- tValue.Interface()
			}
			return nil
		})
		schema.Walk(assignment, tVariable, func(leaf schema.LeafInfo, tValue reflect.Value) error {
			if leaf.Visibility == schema.Secret {
				chValues <- tValue.Interface()
			}
			return nil
		})
	}()
	res := []constraint.Element{}
	for v := range chValues {
		res = append(res, field.FromInterface(v))
	}
	return res
}

// SolveInput is the entry point to solve the final input of the given assignment using a specified number of threads.
func (rc *RootCircuit) SolveInput(assignment frontend.Circuit, _ int) (Witness, error) {
	vec := GetCircuitVariables(assignment, rc.Field)
	res, err := rc.eval(vec)
	if err != nil {
		return nil, err
	}
	witness := make(Witness, len(res))
	for i, x := range res {
		witness[i] = rc.Field.ToBigInt(x)
	}
	return witness, nil
}

func (rc *RootCircuit) eval(inputs []constraint.Element) ([]constraint.Element, error) {
	return rc.evalSub(0, inputs)
}

func (rc *RootCircuit) evalSub(circuitId uint64, inputs []constraint.Element) ([]constraint.Element, error) {
	values := append([]constraint.Element{{}}, inputs...)
	for _, insn := range rc.Circuits[circuitId].Instructions {
		switch insn.Type {
		case LinComb:
			res := insn.Const
			for i, x := range insn.Inputs {
				res = rc.Field.Add(res, rc.Field.Mul(values[x], insn.LinCombCoef[i]))
			}
			values = append(values, res)
		case Mul:
			res := rc.Field.One()
			for _, x := range insn.Inputs {
				res = rc.Field.Mul(res, values[x])
			}
			values = append(values, res)
		case Hint:
			hint_inputs := []*big.Int{}
			for _, x := range insn.Inputs {
				hint_inputs = append(hint_inputs, rc.Field.ToBigInt(values[x]))
			}
			hint_outputs := make([]*big.Int, insn.NumOutputs)
			err := callHint(insn.ExtraId, rc.Field.Field(), hint_inputs, hint_outputs)
			if err != nil {
				return nil, err
			}
			for _, x := range hint_outputs {
				values = append(values, rc.Field.FromInterface(x))
			}
		case ConstantLike:
			if insn.ExtraId == 0 {
				values = append(values, insn.Const)
			} else {
				return nil, errors.New("random constant not supported")
			}
		case SubCircuitCall:
			sub_inputs := []constraint.Element{}
			for _, x := range insn.Inputs {
				sub_inputs = append(sub_inputs, values[x])
			}
			sub_outputs, err := rc.evalSub(insn.ExtraId, sub_inputs)
			if err != nil {
				return nil, err
			}
			values = append(values, sub_outputs...)
		}
	}
	outputs := []constraint.Element{}
	for _, x := range rc.Circuits[circuitId].Outputs {
		outputs = append(outputs, values[x])
	}
	return outputs, nil
}

func callHint(hintId uint64, field *big.Int, inputs []*big.Int, outputs []*big.Int) error {
	// The only required builtin hint (Div)
	if hintId == 0xCCC000000001 {
		x := (&big.Int{}).Mod(inputs[0], field)
		y := (&big.Int{}).Mod(inputs[1], field)
		if y.Cmp(big.NewInt(0)) == 0 {
			outputs[0] = big.NewInt(0)
			return nil
		}
		a := (&big.Int{}).ModInverse(y, field)
		a.Mul(a, x)
		a.Mod(a, field)
		outputs[0] = a
		return nil
	}
	return solver.GetRegisteredHint(solver.HintID(hintId))(field, inputs, outputs)
}

// Serialize converts the Witness into a byte slice for storage or transmission.
func (w Witness) Serialize() []byte {
	o := utils.OutputBuf{}
	for _, x := range w {
		o.AppendBigInt(32, x)
	}
	return o.Bytes()
}
