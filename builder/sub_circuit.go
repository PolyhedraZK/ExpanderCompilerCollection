package builder

import (
	"crypto/sha256"
	"encoding/binary"
	"fmt"
	"reflect"
	"runtime"

	"github.com/Zklib/gkr-compiler/circuitir"
	"github.com/Zklib/gkr-compiler/expr"
	"github.com/consensys/gnark/frontend"
)

// the unique identifier to a sub-circuit function, including
// 1. function name
// 2. non frontend.Variable function args
// 3. dimension of frontend.Variable function args

// TODO: support various args by reflect
type SubCircuitFunc func(api frontend.API, input []frontend.Variable) []frontend.Variable

type SubCircuit struct {
	builder                *builder
	inputAssertedBooleans  []int
	inputAssertedZeroes    []int
	inputAssertedNonZeroes []int
	outputMarkedBooleans   []int
	outputLayers           []int
}

type SubCircuitRegistry map[uint64]*SubCircuit

type SubCircuitAPI interface {
	MemorizedCall(SubCircuitFunc, []frontend.Variable) []frontend.Variable
}

func (parent *builder) MemorizedCall(f SubCircuitFunc, input_ []frontend.Variable) []frontend.Variable {
	name := GetFuncName(f)
	h := sha256.Sum256([]byte(fmt.Sprintf("%d(%s)_%d", len(name), name, len(input_))))
	circuitId := binary.LittleEndian.Uint64(h[:8])

	input, _ := parent.toVariables(input_...)

	if _, ok := parent.root.registry[circuitId]; !ok {
		n := len(input)
		subBuilder := parent.root.newBuilder(n)
		subInput := make([]frontend.Variable, n)
		for i := 0; i < n; i++ {
			subInput[i] = expr.NewLinearExpression(i+1, subBuilder.tOne)
		}
		subOutput := f(subBuilder, subInput)
		subBuilder.output = make([]expr.Expression, len(subOutput))
		for i, v := range subOutput {
			subBuilder.output[i] = v.(expr.Expression)
		}
		sub := SubCircuit{
			builder:      subBuilder,
			outputLayers: make([]int, len(subOutput)),
		}
		for i, x := range subInput {
			if v, ok := subBuilder.booleans.Find(x.(expr.Expression)); ok && v.(constraintStatus) == asserted {
				sub.inputAssertedBooleans = append(sub.inputAssertedBooleans, i)
			}
			if v, ok := subBuilder.zeroes.Find(x.(expr.Expression)); ok && v.(constraintStatus) == asserted {
				sub.inputAssertedZeroes = append(sub.inputAssertedZeroes, i)
			}
			if v, ok := subBuilder.nonZeroes.Find(x.(expr.Expression)); ok && v.(constraintStatus) == asserted {
				sub.inputAssertedNonZeroes = append(sub.inputAssertedNonZeroes, i)
			}
		}
		for i, x := range subOutput {
			if _, ok := subBuilder.booleans.Find(x.(expr.Expression)); ok {
				sub.outputMarkedBooleans = append(sub.outputMarkedBooleans, i)
			}
			sub.outputLayers[i] = subBuilder.layerOfExpr(x.(expr.Expression))
		}
		parent.root.registry[circuitId] = &sub
	}
	sub := parent.root.registry[circuitId]

	maxInputLayer := 0
	for _, x := range input {
		if l := parent.layerOfExpr(x); l > maxInputLayer {
			maxInputLayer = l
		}
	}

	outputIds := make([]int, len(sub.outputLayers))
	output := make([]expr.Expression, len(sub.outputLayers))
	for i, x := range sub.outputLayers {
		outputIds[i] = parent.newVariable(x - 1 + maxInputLayer)
		output[i] = expr.NewLinearExpression(outputIds[i], parent.tOne)
	}

	// 1. for assertions done in the sub circuit, remove them from the parent one
	// 2. for marked boolean variables, mark them in the parent one
	for _, i := range sub.inputAssertedBooleans {
		parent.booleans.Set(input[i], marked)
	}
	for _, i := range sub.inputAssertedZeroes {
		parent.zeroes.Set(input[i], marked)
	}
	for _, i := range sub.inputAssertedNonZeroes {
		parent.nonZeroes.Set(input[i], marked)
	}
	for _, i := range sub.outputMarkedBooleans {
		parent.booleans.Set(output[i], marked)
	}

	parent.instructions = append(parent.instructions,
		circuitir.NewSubCircuitInstruction(circuitId, input, outputIds),
	)

	output_ := make([]frontend.Variable, len(output))
	for i, x := range output {
		output_[i] = x
	}
	return output_
}

func MemorizedFunc(f SubCircuitFunc) SubCircuitFunc {
	return func(api frontend.API, input []frontend.Variable) []frontend.Variable {
		return api.(SubCircuitAPI).MemorizedCall(f, input)
	}
}

func GetFuncName(fn SubCircuitFunc) string {
	fnptr := reflect.ValueOf(fn).Pointer()
	return runtime.FuncForPC(fnptr).Name()
}
