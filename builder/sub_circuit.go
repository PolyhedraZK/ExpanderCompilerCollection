package builder

import (
	"crypto/sha256"
	"encoding/binary"
	"fmt"
	"hash"
	"reflect"
	"runtime"
	"strconv"

	"github.com/Zklib/gkr-compiler/expr"
	"github.com/Zklib/gkr-compiler/ir"
	"github.com/consensys/gnark/frontend"
)

// the unique identifier to a sub-circuit function, including
// 1. function name
// 2. non frontend.Variable function args
// 3. dimension of frontend.Variable function args

// SubCircuitSimpleFunc
type SubCircuitSimpleFunc func(api frontend.API, input []frontend.Variable) []frontend.Variable
type SubCircuitFunc interface{}

type SubCircuit struct {
	builder                *builder
	inputAssertedBooleans  []int
	inputAssertedZeroes    []int
	inputAssertedNonZeroes []int
	outputMarkedBooleans   []int
	outputLayers           []int
}

type SubCircuitRegistry struct {
	m               map[uint64]*SubCircuit
	outputStructure map[uint64]*sliceStructure
}

type SubCircuitAPI interface {
	MemorizedSimpleCall(SubCircuitSimpleFunc, []frontend.Variable) []frontend.Variable
	MemorizedCall(SubCircuitFunc, ...interface{}) interface{}
}

func newSubCircuitRegistry() *SubCircuitRegistry {
	return &SubCircuitRegistry{
		m:               make(map[uint64]*SubCircuit),
		outputStructure: make(map[uint64]*sliceStructure),
	}
}

func (parent *builder) callSubCircuit(
	circuitId uint64,
	input_ []frontend.Variable,
	f SubCircuitSimpleFunc,
) []frontend.Variable {
	input, _ := parent.toVariables(input_...)
	// we need to hash circuitId with the constraint status of input, to get a new circuitId
	h := sha256.New()
	h.Write(binary.LittleEndian.AppendUint64(nil, circuitId))
	subMarkBooleans := make([]int, 0, len(input))
	subMarkZeroes := make([]int, 0, len(input))
	subMarkNonZeroes := make([]int, 0, len(input))
	for i, x := range input {
		if _, ok := parent.booleans.Find(x); ok {
			h.Write([]byte("a"))
			subMarkBooleans = append(subMarkBooleans, i)
		} else {
			h.Write([]byte("_"))
		}
		if _, ok := parent.zeroes.Find(x); ok {
			h.Write([]byte("a"))
			subMarkZeroes = append(subMarkZeroes, i)
		} else {
			h.Write([]byte("_"))
		}
		if _, ok := parent.nonZeroes.Find(x); ok {
			h.Write([]byte("a"))
			subMarkNonZeroes = append(subMarkNonZeroes, i)
		} else {
			h.Write([]byte("_"))
		}
	}
	circuitId = binary.LittleEndian.Uint64(h.Sum(nil)[:8])
	if _, ok := parent.root.registry.m[circuitId]; !ok {
		n := len(input)
		subBuilder := parent.root.newBuilder(n)
		subInput := make([]frontend.Variable, n)
		for i := 0; i < n; i++ {
			subInput[i] = expr.NewLinearExpression(i+1, subBuilder.tOne)
		}
		for _, i := range subMarkBooleans {
			subBuilder.booleans.Set(subInput[i].(expr.Expression), marked)
		}
		for _, i := range subMarkZeroes {
			subBuilder.zeroes.Set(subInput[i].(expr.Expression), marked)
		}
		for _, i := range subMarkNonZeroes {
			subBuilder.nonZeroes.Set(subInput[i].(expr.Expression), marked)
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
		for i, x := range subBuilder.output {
			if _, ok := subBuilder.booleans.Find(x); ok {
				sub.outputMarkedBooleans = append(sub.outputMarkedBooleans, i)
			}
			sub.outputLayers[i] = subBuilder.layerOfExpr(x)
		}
		parent.root.registry.m[circuitId] = &sub
	}
	sub := parent.root.registry.m[circuitId]

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
		ir.NewSubCircuitInstruction(circuitId, input, outputIds),
	)

	output_ := make([]frontend.Variable, len(output))
	for i, x := range output {
		output_[i] = x
	}
	return output_
}

func (parent *builder) MemorizedSimpleCall(f SubCircuitSimpleFunc, input []frontend.Variable) []frontend.Variable {
	name := GetFuncName(f)
	h := sha256.Sum256([]byte(fmt.Sprintf("simple_%d(%s)_%d", len(name), name, len(input))))
	circuitId := binary.LittleEndian.Uint64(h[:8])
	return parent.callSubCircuit(circuitId, input, f)
}

var frontendAPIType = reflect.TypeOf((*frontend.API)(nil)).Elem()
var frontendVariableType = reflect.TypeOf((*frontend.Variable)(nil)).Elem()

// isTypeFrontendAPI returns true if t is a frontend.API
func isTypeFrontendAPI(t reflect.Type) bool {
	//fmt.Printf("%v %v\n", t, frontendAPIType)
	return t == frontendAPIType
}

// isTypeSlicesOfVariables returns true if t is any number of slice of frontend.Variable
func getTypeSlicesOfVariables(t reflect.Type) (int, bool) {
	level := 0
	for {
		if t == frontendVariableType {
			return level, true
		}
		if t.Kind() != reflect.Slice {
			return 0, false
		}
		t = t.Elem()
		level++
	}
}

func layerOfSliceOfVariables(t reflect.Type) int {
	for i := 0; ; i++ {
		if t == frontendVariableType {
			return i
		}
		if t.Kind() != reflect.Slice {
			panic("not a slice of frontend.Variable")
		}
		t = t.Elem()
	}
}

type sliceStructure struct {
	level       int
	totVariable int
	children    []*sliceStructure
}

func joinSliceVariables(res *[]frontend.Variable, h hash.Hash, slice reflect.Value, level int) *sliceStructure {
	val := slice.Interface()
	if level == 0 {
		*res = append(*res, val.(frontend.Variable))
		if h != nil {
			h.Write([]byte("a."))
		}
		return &sliceStructure{level: 0, totVariable: 1}
	}
	if x, ok := val.([]frontend.Variable); ok {
		*res = append(*res, x...)
		if h != nil {
			h.Write([]byte(strconv.Itoa(len(x)) + "."))
		}
		return &sliceStructure{level: 1, totVariable: len(x)}
	}
	r := &sliceStructure{level: 0}
	for i := 0; i < slice.Len(); i++ {
		if h != nil {
			h.Write([]byte("("))
		}
		sub := joinSliceVariables(res, h, slice.Index(i), level-1)
		if h != nil {
			h.Write([]byte(")"))
		}
		r.children = append(r.children, sub)
		r.level = sub.level + 1
		r.totVariable += sub.totVariable
	}
	if r.level == 0 {
		r.level = layerOfSliceOfVariables(slice.Type())
	}
	return r
}

func typeNLayersOfSliceOfVariables(n int) reflect.Type {
	if n == 0 {
		return frontendVariableType
	}
	return reflect.SliceOf(typeNLayersOfSliceOfVariables(n - 1))
}

func rebuildSliceVariables(vars []frontend.Variable, s *sliceStructure) reflect.Value {
	if s.level == 0 {
		return reflect.ValueOf(vars[0])
	}
	if s.level == 1 {
		return reflect.ValueOf(vars)
	}
	cur := 0
	res := reflect.MakeSlice(typeNLayersOfSliceOfVariables(s.level), len(s.children), len(s.children))
	for i, x := range s.children {
		res.Index(i).Set(rebuildSliceVariables(vars[cur:cur+x.totVariable], x))
		cur += x.totVariable
	}
	return res
}

// check for some simple types
func isTypeSimple(t reflect.Type) bool {
	k := t.Kind()
	switch k {
	case reflect.Bool:
		return true
	case reflect.Int, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64:
		return true
	case reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
		return true
	case reflect.String:
		return true
	default:
		return false
	}
}

func (parent *builder) MemorizedCall(fn SubCircuitFunc, inputs ...interface{}) interface{} {
	fnVal := reflect.ValueOf(fn)
	if fnVal.Kind() != reflect.Func {
		panic("f is not a function")
	}
	fnType := fnVal.Type()

	// check function signature
	numIn := fnType.NumIn()
	if numIn == 0 {
		panic("fn should have at least 1 argument")
	}
	if !isTypeFrontendAPI(fnType.In(0)) {
		panic("first argument should be a frontend.API")
	}
	vars := []int{}
	others := []int{}
	varLevel := make([]int, numIn)
	for i := 1; i < numIn; i++ {
		argType := fnType.In(i)
		if level, ok := getTypeSlicesOfVariables(argType); ok {
			vars = append(vars, i)
			varLevel[i] = level
		} else if isTypeSimple(argType) {
			others = append(others, i)
		} else {
			panic(fmt.Sprintf("input %d (%v) is not a slice of frontend.Variable or a simple type", i, argType))
		}
	}
	numOut := fnType.NumOut()
	var outLevel int
	if numOut > 1 {
		panic(fmt.Sprintf("fn should return at most 1 value, got %d", numOut))
	} else if numOut == 1 {
		outType := fnType.Out(0)
		level, ok := getTypeSlicesOfVariables(outType)
		if !ok {
			panic("output is not a slice of frontend.Variable")
		}
		outLevel = level
	}

	// check if inputs match the function signature
	variadic := fnType.IsVariadic()
	if (!variadic && numIn != len(inputs)+1) || (variadic && len(inputs)+1 < numIn-1) {
		panic(fmt.Sprintf("expected %d args, got %d", numIn, len(inputs)))
	}
	var variadicElemType reflect.Type
	if variadic {
		variadicElemType = fnType.In(numIn - 1).Elem()
	}
	inputVals := make([]reflect.Value, len(inputs)+1)
	for i, input := range inputs {
		inputVal := reflect.ValueOf(input)
		inputType := inputVal.Type()
		if i+1 < numIn-1 || !variadic {
			if !inputType.AssignableTo(fnType.In(i + 1)) {
				panic(fmt.Sprintf("input %d (%v) is not assignable to %v", i, inputType, fnType.In(i+1)))
			}
		} else {
			if !inputType.AssignableTo(variadicElemType) {
				panic(fmt.Sprintf("input %d (%v) is not assignable to %v", i, inputType, variadicElemType))
			}
		}
		inputVals[i+1] = inputVal
	}
	if variadic {
		if numIn-1 > len(inputs) {
			vars = vars[:len(vars)-1]
		} else {
			for i := numIn; i <= len(inputs); i++ {
				vars = append(vars, i)
			}
		}
	}

	// join all frontend.Variable together and calculate circuit id
	joinedVars := []frontend.Variable{}
	varSliceStructures := make([]*sliceStructure, len(inputs)+1)
	var outStructure *sliceStructure
	name := GetFuncName(fn)
	h := sha256.New()
	h.Write([]byte(fmt.Sprintf("normal_%d(%s)_%d_", len(name), name, len(inputs))))
	for _, i := range vars {
		varSliceStructures[i] = joinSliceVariables(&joinedVars, h, inputVals[i], varLevel[i])
		h.Write([]byte("|"))
	}
	for _, i := range others {
		vs := inputVals[i].String()
		h.Write([]byte(strconv.Itoa(len(vs)) + vs))
	}
	circuitId := binary.LittleEndian.Uint64(h.Sum(nil)[:8])

	// sub-circuit caller
	fnInner := func(api frontend.API, input []frontend.Variable) []frontend.Variable {
		subInputs := make([]reflect.Value, len(inputVals))
		subInputs[0] = reflect.ValueOf(api)
		cur := 0
		for _, i := range vars {
			s := varSliceStructures[i]
			subInputs[i] = rebuildSliceVariables(input[cur:cur+s.totVariable], s)
			cur += s.totVariable
		}
		for _, i := range others {
			subInputs[i] = inputVals[i]
		}
		outputs := fnVal.Call(subInputs)
		if numOut == 0 {
			outStructure = &sliceStructure{level: -1}
			return nil
		}
		res := []frontend.Variable{}
		outStructure = joinSliceVariables(&res, nil, outputs[0], outLevel)
		return res
	}

	// call sub-circuit
	joinedOut := parent.callSubCircuit(circuitId, joinedVars, fnInner)
	if outStructure == nil {
		outStructure = parent.root.registry.outputStructure[circuitId]
	} else {
		parent.root.registry.outputStructure[circuitId] = outStructure
	}
	if outStructure.level == -1 {
		return nil
	}
	return rebuildSliceVariables(joinedOut, outStructure).Interface()
}

func MemorizedSimpleFunc(f SubCircuitSimpleFunc) SubCircuitSimpleFunc {
	return func(api frontend.API, input []frontend.Variable) []frontend.Variable {
		return api.(SubCircuitAPI).MemorizedSimpleCall(f, input)
	}
}

func MemorizedVoidFunc(f SubCircuitFunc) func(frontend.API, ...interface{}) {
	return func(api frontend.API, inputs ...interface{}) {
		api.(SubCircuitAPI).MemorizedCall(f, inputs...)
	}
}

func Memorized0DFunc(f SubCircuitFunc) func(frontend.API, ...interface{}) frontend.Variable {
	return func(api frontend.API, inputs ...interface{}) frontend.Variable {
		return api.(SubCircuitAPI).MemorizedCall(f, inputs...).(frontend.Variable)
	}
}

func Memorized1DFunc(f SubCircuitFunc) func(frontend.API, ...interface{}) []frontend.Variable {
	return func(api frontend.API, inputs ...interface{}) []frontend.Variable {
		return api.(SubCircuitAPI).MemorizedCall(f, inputs...).([]frontend.Variable)
	}
}

func Memorized2DFunc(f SubCircuitFunc) func(frontend.API, ...interface{}) [][]frontend.Variable {
	return func(api frontend.API, inputs ...interface{}) [][]frontend.Variable {
		return api.(SubCircuitAPI).MemorizedCall(f, inputs...).([][]frontend.Variable)
	}
}

func Memorized3DFunc(f SubCircuitFunc) func(frontend.API, ...interface{}) [][][]frontend.Variable {
	return func(api frontend.API, inputs ...interface{}) [][][]frontend.Variable {
		return api.(SubCircuitAPI).MemorizedCall(f, inputs...).([][][]frontend.Variable)
	}
}

func GetFuncName(fn interface{}) string {
	fnptr := reflect.ValueOf(fn).Pointer()
	return runtime.FuncForPC(fnptr).Name()
}
