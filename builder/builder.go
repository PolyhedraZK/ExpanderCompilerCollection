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

	"github.com/Zklib/gkr-compiler/expr"
	"github.com/Zklib/gkr-compiler/ir"
	"github.com/Zklib/gkr-compiler/utils"
)

// builder implements frontend.API and frontend.Compiler, and builds a circuit
// it can be a root circuit or a sub circuit
type builder struct {
	// This R1CS is only used to provide field operations
	field constraint.R1CS

	// builder of the root circuit
	root *Root

	// map for constraints: map[expr.Expression]constraintStatus
	// if it's known to be true (e.g. in previous gates or in sub circuits), mark it
	// if it's required to be true, assert it
	booleans  utils.Map
	zeroes    utils.Map
	nonZeroes utils.Map
	// TODO: mark constraints for internal variables

	// widely used expressions
	tOne        constraint.Element
	eZero, eOne expr.Expression

	// map from expression to idx
	internalVariables utils.Map

	// instruction list, each instruction specifies the method to calculate some variables
	instructions []ir.Instruction

	// count of variables in different types
	nbInput         int
	nbExternalInput int

	// (probably estimated) layer of each variable
	vLayer []int

	// we have to implement kvstore.Store (required by gnark/internal/circuitdefer/defer.go:30)
	db map[any]any

	// output of sub circuit
	output []expr.Expression
}

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
		nbInput:           nbExternalInput,
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

// asInternalVariable will convert the variable to a single linear term
// It first convert the input to the form a*(...)+c, and then queries the database
// It remembers previous results, and uses cached id if possible
func (builder *builder) asInternalVariable(eall expr.Expression, forceRaw bool) expr.Expression {
	if len(eall) == 1 && eall[0].VID1 == 0 {
		return eall
	}
	e, coeff, constant := builder.stripConstant(eall, forceRaw)
	if len(e) == 1 && e[0].VID1 == 0 && !forceRaw {
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

func (builder *builder) tryAsInternalVariable(eall expr.Expression) expr.Expression {
	if len(eall) == 1 && eall[0].VID1 == 0 {
		return eall
	}
	e, coeff, constant := builder.stripConstant(eall, false)
	if len(e) == 1 && e[0].VID1 == 0 {
		return eall
	}
	idx_, ok := builder.internalVariables.Find(e)
	if ok {
		return builder.unstripConstant(idx_.(int), coeff, constant)
	}
	return eall
}

func (builder *builder) newVariable(layer int) int {
	r := len(builder.vLayer)
	builder.vLayer = append(builder.vLayer, layer)
	return r
}

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

func (builder *builder) stripConstant(e_ expr.Expression, forceRaw bool) (expr.Expression, constraint.Element, constraint.Element) {
	if forceRaw {
		return e_, builder.tOne, constraint.Element{}
	}
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

func (builder *builder) Field() *big.Int {
	return builder.field.Field()
}

func (builder *builder) FieldBitLen() int {
	return builder.field.FieldBitLen()
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
	return builder.field.ToBigInt(coeff), true
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
	builder.nbInput += len(outId)

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

// TODO: add special flag if it's the output
// layeredAdd sums the given expression list by layers
func (builder *builder) layeredAdd(es_ []expr.Expression) expr.Expression {
	es := builder.newExprList(es_)
	sort.Sort(es)
	cur := []expr.Expression{builder.eZero}
	lastLayer := -1
	for i, x := range es.e {
		if es.l[i] != lastLayer && lastLayer != -1 {
			sum := builder.asInternalVariable(builder.add(cur, false, 0, nil, true), false)
			cur = []expr.Expression{sum}
		}
		cur = append(cur, x)
		lastLayer = es.l[i]
	}
	return builder.add(cur, false, 0, nil, true)
}

// TODO: revert back size limit
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
	return e
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
