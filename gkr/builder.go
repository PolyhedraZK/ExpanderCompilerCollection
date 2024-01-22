package gkr

import (
	"errors"
	"math/big"
	"reflect"
	"sort"

	"github.com/consensys/gnark/constraint"
	bn254r1cs "github.com/consensys/gnark/constraint/bn254"
	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/debug"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/schema"

	"github.com/Zklib/gkr-compiler/gkr/expr"
)

type builder struct {
	cs     constraint.R1CS
	config frontend.CompileConfig

	// map for recording boolean constrained variables (to not constrain them twice)
	mtBooleans map[uint64][]expr.Expression

	tOne        constraint.Element
	eZero, eOne expr.Expression

	// normal internal wires expressions
	cachedInternalVariables map[uint64][]internalVariable

	hints []hint

	// (probably estimated) layer of each variable
	vLayer []int

	// each constraint is expr == 0
	// output = ai*r^i where r is the last input (r should be committed)
	constraints []expr.Expression

	// implement kvstore.Store
	db map[any]any

	// final output internal wire id
	output int

	// artifacts
	circuit          circuit
	inputVariableIdx []int
}

type internalVariable struct {
	expr expr.Expression
	idx  int
}

type hint struct {
	f         solver.Hint // if f is nil, then it's a normal internal variable
	inputs    []expr.Expression
	outputIds []int
}

func newBuilder(field *big.Int, config frontend.CompileConfig) *builder {
	builder := builder{
		config:                  config,
		mtBooleans:              make(map[uint64][]expr.Expression),
		cachedInternalVariables: make(map[uint64][]internalVariable),
		db:                      make(map[any]any),
	}

	// TODO: check different fields
	// This R1CS is only used to manage variables and get a field
	builder.cs = bn254r1cs.NewR1CS(config.Capacity)

	if field.Cmp(builder.Field()) != 0 {
		panic("currently only BN254 is supported")
	}

	builder.tOne = builder.cs.One()
	builder.cs.AddPublicVariable("1")
	builder.vLayer = append(builder.vLayer, 1)

	builder.eZero = expr.NewConstantExpression(constraint.Element{})
	builder.eOne = expr.NewConstantExpression(builder.tOne)

	return &builder
}

// PublicVariable creates a new public Variable
func (builder *builder) PublicVariable(f schema.LeafInfo) frontend.Variable {
	idx := builder.cs.AddPublicVariable(f.FullName())
	builder.vLayer = append(builder.vLayer, 1)
	return expr.NewLinearExpression(idx, builder.tOne)
}

// SecretVariable creates a new secret Variable
func (builder *builder) SecretVariable(f schema.LeafInfo) frontend.Variable {
	idx := builder.cs.AddSecretVariable(f.FullName())
	builder.vLayer = append(builder.vLayer, 1)
	return expr.NewLinearExpression(idx, builder.tOne)
}

// newInternalVariable creates a new wire, appends it on the list of wires of the circuit, sets
// the wire's id to the number of wires, and returns it
// It remembers previous results, and uses cached id if possible
// TODO: improve cache (maybe force constant=1)
func (builder *builder) asInternalVariable(e expr.Expression) expr.Expression {
	h := e.HashCode()
	s, ok := builder.cachedInternalVariables[h]
	if ok {
		for _, v := range s {
			if e.Equal(v.expr) {
				return expr.NewLinearExpression(v.idx, builder.tOne)
			}
		}
	} else {
		s = make([]internalVariable, 0, 1)
	}
	builder.vLayer = append(builder.vLayer, builder.layerOfExpr(e)+1)
	idx := builder.cs.AddInternalVariable()
	builder.cachedInternalVariables[h] = append(s, internalVariable{
		expr: e,
		idx:  idx,
	})
	builder.hints = append(builder.hints, hint{
		f:         nil,
		inputs:    []expr.Expression{e},
		outputIds: []int{idx},
	})
	return expr.NewLinearExpression(idx, builder.tOne)
}

func (builder *builder) Field() *big.Int {
	return builder.cs.Field()
}

func (builder *builder) FieldBitLen() int {
	return builder.cs.FieldBitLen()
}

func (builder *builder) layerOfExpr(e expr.Expression) int {
	layer := 1
	for _, term := range e {
		if builder.vLayer[term.VID0] > layer {
			layer = builder.vLayer[term.VID0]
		}
		if builder.vLayer[term.VID1] > layer {
			layer = builder.vLayer[term.VID1]
		}
	}
	return layer
}

// MarkBoolean sets (but do not **constraint**!) v to be boolean
// This is useful in scenarios where a variable is known to be boolean through a constraint
// that is not api.AssertIsBoolean. If v is a constant, this is a no-op.
func (builder *builder) MarkBoolean(v frontend.Variable) {
	if b, ok := builder.constantValue(v); ok {
		if !(b.IsZero() || builder.cs.IsOne(b)) {
			panic("MarkBoolean called a non-boolean constant")
		}
		return
	}
	// v is a linear expression
	l := v.(expr.Expression)
	sort.Sort(l)

	key := l.HashCode()
	list := builder.mtBooleans[key]
	list = append(list, l)
	builder.mtBooleans[key] = list
}

// IsBoolean returns true if given variable was marked as boolean in the compiler (see MarkBoolean)
// Use with care; variable may not have been **constrained** to be boolean
// This returns true if the v is a constant and v == 0 || v == 1.
func (builder *builder) IsBoolean(v frontend.Variable) bool {
	if b, ok := builder.constantValue(v); ok {
		return (b.IsZero() || builder.cs.IsOne(b))
	}
	// v is a linear expression
	l := v.(expr.Expression)
	sort.Sort(l)

	key := l.HashCode()
	list, ok := builder.mtBooleans[key]
	if !ok {
		return false
	}

	for _, v := range list {
		if v.Equal(l) {
			return true
		}
	}
	return false
}

// Compile constructs a rank-1 constraint sytem
func (builder *builder) Compile() (constraint.ConstraintSystem, error) {
	return nil, nil
}

// ConstantValue returns the big.Int value of v.
// Will panic if v.IsConstant() == false
func (builder *builder) ConstantValue(v frontend.Variable) (*big.Int, bool) {
	coeff, ok := builder.constantValue(v)
	if !ok {
		return nil, false
	}
	return builder.cs.ToBigInt(coeff), true
}

func (builder *builder) constantValue(v frontend.Variable) (constraint.Element, bool) {
	if _v, ok := v.(expr.Expression); ok {
		assertIsSet(_v)

		if len(_v) != 1 {
			// TODO @gbotrel this assumes linear expressions of coeff are not possible
			// and are always reduced to one element. may not always be true?
			return constraint.Element{}, false
		}
		if !(_v[0].VID0 == 0 && _v[0].VID1 == 0) { // public ONE WIRE
			return constraint.Element{}, false
		}
		return _v[0].Coeff, true
	}
	return builder.cs.FromInterface(v), true
}

// toVariable will return (and allocate if neccesary) an Expression from given value
//
// if input is already an Expression, does nothing
// else, attempts to convert input to a big.Int (see utils.FromInterface) and returns a toVariable Expression
func (builder *builder) toVariable(input interface{}) expr.Expression {

	switch t := input.(type) {
	case expr.Expression:
		// this is already a "kwown" variable
		assertIsSet(t)
		return t
	case *expr.Expression:
		assertIsSet(*t)
		return *t
	case constraint.Element:
		return expr.NewLinearExpression(0, t)
	case *constraint.Element:
		return expr.NewLinearExpression(0, *t)
	default:
		// try to make it into a constant
		c := builder.cs.FromInterface(t)
		return expr.NewLinearExpression(0, c)
	}
}

// toVariables return frontend.Variable corresponding to inputs and the total size of the linear expressions
func (builder *builder) toVariables(in ...frontend.Variable) ([]expr.Expression, int) {
	r := make([]expr.Expression, 0, len(in))
	s := 0
	e := func(i frontend.Variable) {
		v := builder.toVariable(i)
		r = append(r, v)
		s += len(v)
	}
	// e(i1)
	// e(i2)
	for i := 0; i < len(in); i++ {
		e(in[i])
	}
	return r, s
}

// NewHint initializes internal variables whose value will be evaluated using
// the provided hint function at run time from the inputs. Inputs must be either
// variables or convertible to *big.Int. The function returns an error if the
// number of inputs is not compatible with f.
//
// The hint function is provided at the proof creation time and is not embedded
// into the circuit. From the backend point of view, the variable returned by
// the hint function is equivalent to the user-supplied witness, but its actual
// value is assigned by the solver, not the caller.
//
// No new constraints are added to the newly created wire and must be added
// manually in the circuit. Failing to do so leads to solver failure.
func (builder *builder) NewHint(f solver.Hint, nbOutputs int, inputs ...frontend.Variable) ([]frontend.Variable, error) {
	// TODO: memorize hints?
	return builder.newHint(f, nbOutputs, inputs)
}

func (builder *builder) NewHintForId(id solver.HintID, nbOutputs int, inputs ...frontend.Variable) ([]frontend.Variable, error) {
	panic("unimplemented")
}

func (builder *builder) newHint(f solver.Hint, nbOutputs int, inputs []frontend.Variable) ([]frontend.Variable, error) {
	hintInputs := make([]expr.Expression, len(inputs))

	// TODO @gbotrel hint input pass
	// ensure inputs are set and pack them in a []uint64
	for i, in := range inputs {
		if t, ok := in.(expr.Expression); ok {
			assertIsSet(t)
			hintInputs[i] = t
		} else {
			c := builder.cs.FromInterface(in)
			hintInputs[i] = expr.NewConstantExpression(c)
		}
	}

	outId := make([]int, nbOutputs)
	for i := 0; i < nbOutputs; i++ {
		outId[i] = builder.cs.AddInternalVariable()
		builder.vLayer = append(builder.vLayer, 1)
	}

	builder.hints = append(builder.hints, hint{
		f:         f,
		inputs:    hintInputs,
		outputIds: outId,
	})

	// make the variables
	res := make([]frontend.Variable, nbOutputs)
	for i, idx := range outId {
		res[i] = expr.NewLinearExpression(idx, builder.tOne)
	}
	return res, nil
}

// assertIsSet panics if the variable is unset
// this may happen if inside a Define we have
// var a variable
// cs.Mul(a, 1)
// since a was not in the circuit struct it is not a secret variable
func assertIsSet(e expr.Expression) {
	if len(e) == 0 {
		// errNoValue triggered when trying to access a variable that was not allocated
		errNoValue := errors.New("can't determine API input value")
		panic(errNoValue)
	}

	if debug.Debug {
		// frontend/ package must build linear expressions that are sorted.
		if !sort.IsSorted(e) {
			panic("unsorted linear expression")
		}
	}
}

func (builder *builder) Defer(cb func(frontend.API) error) {
	panic("unimplemented")
}

// AddInstruction is used to add custom instructions to the constraint system.
func (builder *builder) AddInstruction(bID constraint.BlueprintID, calldata []uint32) []uint32 {
	panic("unimplemented")
}

// AddBlueprint adds a custom blueprint to the constraint system.
func (builder *builder) AddBlueprint(b constraint.Blueprint) constraint.BlueprintID {
	panic("unimplemented")
}

func (builder *builder) InternalVariable(wireID uint32) frontend.Variable {
	panic("unimplemented")
}

// ToCanonicalVariable converts a frontend.Variable to a constraint system specific Variable
// ! Experimental: use in conjunction with constraint.CustomizableSystem
func (builder *builder) ToCanonicalVariable(in frontend.Variable) frontend.CanonicalVariable {
	panic("unimplemented")
}

// implement kvstore
func (builder *builder) SetKeyValue(key, value any) {
	if !reflect.TypeOf(key).Comparable() {
		panic("key type not comparable")
	}
	builder.db[key] = value
}

func (builder *builder) GetKeyValue(key any) any {
	if !reflect.TypeOf(key).Comparable() {
		panic("key type not comparable")
	}
	return builder.db[key]
}
