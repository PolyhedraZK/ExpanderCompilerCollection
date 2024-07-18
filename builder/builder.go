// Some content of this file is copied from gnark/frontend/cs/r1cs/builder.go

// Package builder provides an implementation based on the gnark frontend builder with the following modifications:
// - LinearExpression has been changed to allow for quadratic terms in the form of Expression.
// - Assert series functions are recorded first and later solidified into the IR.
// - Support for subcircuits is integrated within the builder.
package builder

import (
	"errors"
	"math/big"
	"reflect"
	"sort"

	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/debug"
	"github.com/consensys/gnark/frontend"

	"github.com/PolyhedraZK/ExpanderCompilerCollection/expr"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/field"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/ir"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/utils"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/utils/customgates"
)

// builder implements frontend.API and frontend.Compiler, and builds a circuit
// it can be a root circuit or a sub circuit
type builder struct {
	field field.Field

	// builder of the root circuit
	root *Root

	// map for constraints: map[expr.Expression]constraintStatus
	// if it's known to be true (e.g. in previous gates or in sub circuits), mark it
	// if it's required to be true, assert it
	booleans  utils.Map
	zeroes    utils.Map
	nonZeroes utils.Map

	// widely used expressions
	tOne        constraint.Element
	eZero, eOne expr.Expression

	// map from expression to idx
	internalVariables utils.Map

	// instruction list, each instruction specifies the method to calculate some variables
	instructions []ir.Instruction

	// count of variables in different types
	nbExternalInput int

	// (probably estimated) layer of each variable
	vLayer []int

	// defers (for gnark API)
	defers []func(frontend.API) error

	// we have to implement kvstore.Store (required by gnark/internal/circuitdefer/defer.go:30)
	db map[any]any

	// output of sub circuit
	output []expr.Expression
}

// newBuilder returns a builder with known number of external input
func (r *Root) newBuilder(nbExternalInput int) *builder {
	builder := builder{
		field:             r.field,
		root:              r,
		booleans:          make(utils.Map),
		internalVariables: make(utils.Map),
		zeroes:            make(utils.Map),
		nonZeroes:         make(utils.Map),
		db:                make(map[any]any),
		nbExternalInput:   nbExternalInput,
	}

	builder.tOne = builder.field.One()
	builder.vLayer = append(builder.vLayer, 1)

	builder.eZero = expr.NewConstantExpression(constraint.Element{})
	builder.eOne = expr.NewConstantExpression(builder.tOne)

	// add 1 for the constant "1"
	builder.vLayer = make([]int, nbExternalInput+1)
	for i := 1; i <= nbExternalInput; i++ {
		builder.vLayer[i] = 1
	}

	return &builder
}

type constraintStatus int

const (
	_                         = 0
	marked   constraintStatus = iota
	asserted constraintStatus = iota
)

// asInternalVariableInner will convert the variable to a single linear term
// It first convert the input to the form coeff*(...)+constant, and then queries the database
// It remembers previous results, and uses cached id if possible
func (builder *builder) asInternalVariableInner(eall expr.Expression, force bool) expr.Expression {
	e, coeff, constant := builder.stripConstant(eall)
	if force {
		e = eall
		coeff = builder.tOne
		constant = constraint.Element{}
	}
	if len(e) == 1 && e[0].VID1 == 0 {
		return eall
	}
	idx_, ok := builder.internalVariables.Find(e)
	if ok {
		return builder.unstripConstant(idx_.(int), coeff, constant)
	}
	idx := builder.newVariable(builder.layerOfExpr(e) + 1)
	builder.internalVariables.Set(e, idx)
	builder.instructions = append(builder.instructions,
		ir.NewInternalVariableInstruction(e, idx),
	)
	return builder.unstripConstant(idx, coeff, constant)
}

// asInternalVariable converts the variable to a linear term
func (builder *builder) asInternalVariable(eall expr.Expression) expr.Expression {
	res := builder.asInternalVariableInner(eall, false)
	if !eall.Equal(res) {
		builder.markConstraintsForInternalVariable(eall, res)
	}
	return res
}

// ToSingleVariable converts an expression to a single base variable without a constant term.
func (builder *builder) ToSingleVariable(ein frontend.Variable) frontend.Variable {
	eall := builder.toVariable(ein)
	res := builder.asInternalVariableInner(eall, true)
	if !eall.Equal(res) {
		builder.markConstraintsForInternalVariable(eall, res)
	}
	return res
}

// tryAsInternalVariableInner is similar to asInternalVariableInner, but it only trys to look up the table
// If there is no result, it keeps the original variable
func (builder *builder) tryAsInternalVariableInner(eall expr.Expression) expr.Expression {
	if len(eall) == 1 && eall[0].VID1 == 0 {
		return eall
	}
	e, coeff, constant := builder.stripConstant(eall)
	if len(e) == 1 && e[0].VID1 == 0 {
		return eall
	}
	idx_, ok := builder.internalVariables.Find(e)
	if ok {
		return builder.unstripConstant(idx_.(int), coeff, constant)
	}
	return eall
}

func (builder *builder) tryAsInternalVariable(eall expr.Expression) expr.Expression {
	res := builder.tryAsInternalVariableInner(eall)
	if !eall.Equal(res) {
		builder.markConstraintsForInternalVariable(eall, res)
	}
	return res
}

// markConstraintsForInternalVariable checks exists assertions and markings on the original variable
// Then it marks them on the new variable
func (builder *builder) markConstraintsForInternalVariable(eall expr.Expression, iv expr.Expression) {
	markConstraintForInternalVariable(&builder.booleans, eall, iv)
	markConstraintForInternalVariable(&builder.zeroes, eall, iv)
	markConstraintForInternalVariable(&builder.nonZeroes, eall, iv)
}

func markConstraintForInternalVariable(m *utils.Map, eall expr.Expression, iv expr.Expression) {
	x, ok := m.Find(eall)
	if !ok {
	} else if x.(constraintStatus) == marked {
		m.Set(iv, marked)
	} else if x.(constraintStatus) == asserted {
		m.Set(iv, asserted)
		m.Set(eall, marked)
	}
}

func (builder *builder) newVariable(layer int) int {
	r := len(builder.vLayer)
	builder.vLayer = append(builder.vLayer, layer)
	return r
}

// unstripConstant returns x*coeff+constant
func (builder *builder) unstripConstant(x int, coeff constraint.Element, constant constraint.Element) expr.Expression {
	if x == 0 {
		panic("can't unstrip 0")
	}
	e := expr.NewLinearExpression(x, coeff)
	if !constant.IsZero() {
		e = append(e, expr.NewConstantExpression(constant)...)
	}
	sort.Sort(e)
	return e
}

// stripConstant tries to find a coeff and constant for the given variable
func (builder *builder) stripConstant(e_ expr.Expression) (expr.Expression, constraint.Element, constraint.Element) {
	cst := constraint.Element{}
	e := make(expr.Expression, 0, len(e_))
	for _, term := range e_ {
		if term.VID0 == 0 {
			cst = term.Coeff
		} else {
			e = append(e, term)
		}
	}
	if len(e) == 0 {
		e = builder.eZero
	}
	sort.Sort(e)
	v := e[0].Coeff
	vi, ok := builder.field.Inverse(v)
	if !ok {
		vi = constraint.Element{}
		if len(e) != 1 {
			panic("malformed expression")
		}
	}
	for i := 0; i < len(e); i++ {
		e[i].Coeff = builder.field.Mul(e[i].Coeff, vi)
	}
	return e, v, cst
}

// Field returns the value of the current field being used.
func (builder *builder) Field() *big.Int {
	return builder.field.Field()
}

// FieldBitLen returns the bit length of the current field being used.
func (builder *builder) FieldBitLen() int {
	return builder.field.FieldBitLen()
}

// LayerOf returns the expected layer of the variable, though the actual layer may vary.
func (builder *builder) LayerOf(e frontend.Variable) int {
	return builder.layerOfExpr(builder.toVariable(e))
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
		if !(b.IsZero() || builder.field.IsOne(b)) {
			panic("MarkBoolean called a non-boolean constant")
		}
		return
	}
	// v is a linear expression
	l := v.(expr.Expression)
	sort.Sort(l)

	builder.booleans.Set(l, marked)
}

// IsBoolean returns true if given variable was marked as boolean in the compiler (see MarkBoolean)
// Use with care; variable may not have been **constrained** to be boolean
// This returns true if the v is a constant and v == 0 || v == 1.
func (builder *builder) IsBoolean(v frontend.Variable) bool {
	if b, ok := builder.constantValue(v); ok {
		return (b.IsZero() || builder.field.IsOne(b))
	}
	// v is a linear expression
	l := v.(expr.Expression)
	sort.Sort(l)

	_, ok := builder.booleans.Find(l)
	return ok
}

// Compile is a placeholder for gnark API compatibility; it does nothing.
func (builder *builder) Compile() (constraint.ConstraintSystem, error) {
	return nil, nil
}

// ConstantValue returns the big.Int value of v and panics if v is not a constant.
func (builder *builder) ConstantValue(v frontend.Variable) (*big.Int, bool) {
	coeff, ok := builder.constantValue(v)
	if !ok {
		return nil, false
	}
	return builder.field.ToBigInt(coeff), true
}

func (builder *builder) constantValue(v frontend.Variable) (constraint.Element, bool) {
	if _v, ok := v.(expr.Expression); ok {
		assertIsSet(_v)

		if len(_v) != 1 {
			return constraint.Element{}, false
		}
		if !(_v[0].VID0 == 0 && _v[0].VID1 == 0) {
			return constraint.Element{}, false
		}
		return _v[0].Coeff, true
	}
	return builder.field.FromInterface(v), true
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
		c := builder.field.FromInterface(t)
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
// The hint function is provided at the input solving time and is not embedded
// into the circuit. From the prover point of view, the variable returned by
// the hint function is equivalent to the user-supplied witness, but its actual
// value is assigned by the InputSolver, not the caller.
//
// No new constraints are added to the newly created wire and must be added
// manually in the circuit. Failing to do so leads to solver failure.
func (builder *builder) NewHint(f solver.Hint, nbOutputs int, inputs ...frontend.Variable) ([]frontend.Variable, error) {
	return builder.newHint(f, nbOutputs, inputs)
}

// NewHintForId is not implemented and will panic if called.
func (builder *builder) NewHintForId(id solver.HintID, nbOutputs int, inputs ...frontend.Variable) ([]frontend.Variable, error) {
	panic("unimplemented")
}

func (builder *builder) newHint(f solver.Hint, nbOutputs int, inputs []frontend.Variable) ([]frontend.Variable, error) {
	hintInputs := make([]expr.Expression, len(inputs))

	for i, in := range inputs {
		if t, ok := in.(expr.Expression); ok {
			assertIsSet(t)
			hintInputs[i] = t
		} else {
			c := builder.field.FromInterface(in)
			hintInputs[i] = expr.NewConstantExpression(c)
		}
	}

	outId := make([]int, nbOutputs)
	for i := 0; i < nbOutputs; i++ {
		outId[i] = builder.newVariable(1)
	}

	builder.instructions = append(builder.instructions,
		ir.NewHintInstruction(f, hintInputs, outId),
	)

	// make the variables
	res := make([]frontend.Variable, nbOutputs)
	for i, idx := range outId {
		res[i] = expr.NewLinearExpression(idx, builder.tOne)
	}
	return res, nil
}

func (builder *builder) CustomGate(gateType uint64, inputs ...frontend.Variable) frontend.Variable {
	f := customgates.GetFunc(gateType)
	hintInputs := make([]expr.Expression, len(inputs))

	for i, in := range inputs {
		if t, ok := in.(expr.Expression); ok {
			assertIsSet(t)
			hintInputs[i] = t
		} else {
			c := builder.field.FromInterface(in)
			hintInputs[i] = expr.NewConstantExpression(c)
		}
	}

	outId := builder.newVariable(1)

	builder.instructions = append(builder.instructions,
		ir.NewCustomGateInstruction(f, gateType, hintInputs, outId),
	)

	return expr.NewLinearExpression(outId, builder.tOne)
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

// layeredAdd sums the given expression list by layers
func (builder *builder) layeredAdd(es_ []expr.Expression) expr.Expression {
	es := builder.newExprList(es_)
	sort.Sort(es)
	cur := []expr.Expression{builder.eZero}
	lastLayer := -1
	for i, x := range es.e {
		if es.l[i] != lastLayer && lastLayer != -1 {
			sum := builder.asInternalVariable(builder.add(cur, false, 0, nil, true))
			cur = []expr.Expression{sum}
		}
		cur = append(cur, x)
		lastLayer = es.l[i]
	}
	return builder.add(cur, false, 0, nil, true)
}

func (builder *builder) compress(e expr.Expression) expr.Expression {
	minL := 1 << 60
	maxL := -1 << 60
	for _, term := range e {
		if term.VID0 == 0 {
			// nop
		} else if term.VID1 == 0 {
			if builder.vLayer[term.VID0] < minL {
				minL = builder.vLayer[term.VID0]
			}
			if builder.vLayer[term.VID0] > maxL {
				maxL = builder.vLayer[term.VID0]
			}
		} else {
			if builder.vLayer[term.VID1] < minL {
				minL = builder.vLayer[term.VID1]
			}
			if builder.vLayer[term.VID0] > maxL {
				maxL = builder.vLayer[term.VID0]
			}
		}
	}
	if maxL-minL >= 1 {
		es := make([]expr.Expression, 0, len(e))
		for _, term := range e {
			t := make(expr.Expression, 1)
			t[0] = term
			es = append(es, t)
		}
		e = builder.layeredAdd(es)
	}
	if builder.root.config.CompressThreshold <= 0 || len(e) < builder.root.config.CompressThreshold {
		return e
	}
	return builder.asInternalVariable(e)
}

// IdentityHint sets output[0] to input[0] and is used to implement ToFirstLayer.
func IdentityHint(field *big.Int, inputs []*big.Int, outputs []*big.Int) error {
	a := big.NewInt(0)
	a.Set(inputs[0])
	outputs[0] = a
	return nil
}

// ToFirstLayer adds a hint to the target variable to bring it to the first layer.
func (builder *builder) ToFirstLayer(v frontend.Variable) frontend.Variable {
	x, _ := builder.NewHint(IdentityHint, 1, v)
	builder.AssertIsEqual(x[0], v)
	builder.markConstraintsForInternalVariable(builder.toVariable(v), builder.toVariable(x[0]))
	return x[0]
}

// Defer adds a callback function to the defer list to be processed later.
func (builder *builder) Defer(cb func(frontend.API) error) {
	builder.defers = append(builder.defers, cb)
}

// AddInstruction is not implemented and will panic if called.
func (builder *builder) AddInstruction(bID constraint.BlueprintID, calldata []uint32) []uint32 {
	panic("unimplemented")
}

// AddBlueprint is not implemented and will panic if called.
func (builder *builder) AddBlueprint(b constraint.Blueprint) constraint.BlueprintID {
	panic("unimplemented")
}

// InternalVariable is not implemented and will panic if called.
func (builder *builder) InternalVariable(wireID uint32) frontend.Variable {
	panic("unimplemented")
}

// ToCanonicalVariable is not implemented and will panic if called.
func (builder *builder) ToCanonicalVariable(in frontend.Variable) frontend.CanonicalVariable {
	panic("unimplemented")
}

// SetKeyValue implements kvstore for the gnark frontend.
func (builder *builder) SetKeyValue(key, value any) {
	if !reflect.TypeOf(key).Comparable() {
		panic("key type not comparable")
	}
	builder.db[key] = value
}

// GetKeyValue implements kvstore for the gnark frontend.
func (builder *builder) GetKeyValue(key any) any {
	if !reflect.TypeOf(key).Comparable() {
		panic("key type not comparable")
	}
	return builder.db[key]
}

// GetRandomValue returns a random value determined during the proving time.
// The return value cannot be used in hints, since it's unknown at the input solving phase
func (builder *builder) GetRandomValue() frontend.Variable {
	idx := builder.newVariable(2)
	builder.instructions = append(builder.instructions,
		ir.NewGetRandomInstruction(idx),
	)
	return expr.NewLinearExpression(idx, builder.tOne)
}
