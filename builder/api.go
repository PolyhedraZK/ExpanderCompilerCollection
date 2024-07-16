// Some content of this file is copied from gnark/frontend/cs/r1cs/api.go

package builder

import (
	"crypto/rand"
	"errors"
	"fmt"
	"math/big"
	"sort"

	"github.com/PolyhedraZK/ExpanderCompilerCollection/expr"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/utils"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/math/bits"
)

func init() {
	solver.RegisterHint(DivHint)
	solver.RegisterHint(IdentityHint)
}

// API defines a set of methods for interacting with the circuit builder.
type API interface {
	// ToSingleVariable converts an expression to a single base variable.
	ToSingleVariable(frontend.Variable) frontend.Variable
	// Output adds a variable to the circuit's output.
	Output(frontend.Variable)
	// LayerOf returns an approximation of the layer in which a variable will be placed after compilation.
	LayerOf(frontend.Variable) int // For debug usage.
	// ToFirstLayer uses a hint to pull a variable back to the first layer.
	ToFirstLayer(frontend.Variable) frontend.Variable
	// GetRandomValue returns a random value for use within the circuit.
	GetRandomValue() frontend.Variable
	// CustomGate registers a hint, but it compiles to a custom gate in the layered circuit.
	CustomGate(f solver.Hint, gateType uint64, inputs ...frontend.Variable) frontend.Variable
}

// ---------------------------------------------------------------------------------------------
// Arithmetic

// Add computes the sum i1+i2+...in and returns the result.
func (builder *builder) Add(i1, i2 frontend.Variable, in ...frontend.Variable) frontend.Variable {
	// extract frontend.Variables from input
	vars, s := builder.toVariables(append([]frontend.Variable{i1, i2}, in...)...)
	return builder.add(vars, false, s, nil, false)
}

func (builder *builder) MulAcc(a, b, c frontend.Variable) frontend.Variable {
	return builder.Add(builder.Mul(b, c), a).(expr.Expression)
}

// Sub computes the difference between the given variables.
func (builder *builder) Sub(i1, i2 frontend.Variable, in ...frontend.Variable) frontend.Variable {
	vars, s := builder.toVariables(append([]frontend.Variable{i1, i2}, in...)...)
	return builder.add(vars, true, s, nil, false)
}

// returns res = Σ(vars) or res = vars[0] - Σ(vars[1:]) if sub == true.
func (builder *builder) add(vars []expr.Expression, sub bool, capacity int, res *expr.Expression, noCompress bool) expr.Expression {
	// we want to merge all terms from input expressions
	// if they are duplicate, we reduce; that is, if multiple terms in different vars have the
	// same variable id.

	// the frontend/ only builds expression that are sorted.
	// we build a sorted output by iterating all the lists in order and dealing
	// with the edge cases (same variable ID, coeff == 0, etc.)

	// initialize the min-heap

	heap := minHeap{}

	for lID, v := range vars {
		heap = append(heap, linMeta{val0: v[0].VID0, val1: v[0].VID1, lID: lID})
	}
	heap.heapify()

	if res == nil {
		t := make(expr.Expression, 0, capacity)
		res = &t
	}
	curr := -1

	// process all the terms from all the inputs, in sorted order
	for len(heap) > 0 {
		lID, tID := heap[0].lID, heap[0].tID
		if tID == len(vars[lID])-1 {
			// last element, we remove it from the heap.
			heap.popHead()
		} else {
			// increment and fix the heap
			heap[0].tID++
			heap[0].val0 = vars[lID][tID+1].VID0
			heap[0].val1 = vars[lID][tID+1].VID1
			heap.fix(0)
		}
		t := &vars[lID][tID]
		if t.Coeff.IsZero() {
			continue // is this really needed?
		}
		if curr != -1 && t.VID0 == (*res)[curr].VID0 && t.VID1 == (*res)[curr].VID1 {
			// accumulate, it's the same variable ID
			if sub && lID != 0 {
				(*res)[curr].Coeff = builder.field.Sub((*res)[curr].Coeff, t.Coeff)
			} else {
				(*res)[curr].Coeff = builder.field.Add((*res)[curr].Coeff, t.Coeff)
			}
			if (*res)[curr].Coeff.IsZero() {
				// remove self.
				(*res) = (*res)[:curr]
				curr--
			}
		} else {
			// append, it's a new variable ID
			(*res) = append((*res), *t)
			curr++
			if sub && lID != 0 {
				(*res)[curr].Coeff = builder.field.Neg((*res)[curr].Coeff)
			}
		}
	}

	if len((*res)) == 0 {
		// keep the expression valid (assertIsSet)
		(*res) = append((*res), expr.NewTerm(0, 0, constraint.Element{}))
	}

	if noCompress {
		return *res
	}
	return builder.compress(*res)
}

// Neg returns the negation of the given variable.
func (builder *builder) Neg(i frontend.Variable) frontend.Variable {
	v := builder.toVariable(i)

	if n, ok := builder.constantValue(v); ok {
		n = builder.field.Neg(n)
		return expr.NewConstantExpression(n)
	}

	return builder.negateLinExp(v)
}

// returns -e, the result is a copy
func (builder *builder) negateLinExp(e expr.Expression) expr.Expression {
	res := make(expr.Expression, len(e))
	copy(res, e)
	for i := 0; i < len(res); i++ {
		res[i].Coeff = builder.field.Neg(res[i].Coeff)
	}
	return res
}

// Mul computes the product of the given variables.
func (builder *builder) Mul(i1, i2 frontend.Variable, in ...frontend.Variable) frontend.Variable {
	vars, _ := builder.toVariables(append([]frontend.Variable{i1, i2}, in...)...)

	mul := func(v1, v2 expr.Expression, first bool) expr.Expression {

		n1, v1Constant := builder.constantValue(v1)
		n2, v2Constant := builder.constantValue(v2)

		v1Deg := v1.Degree()
		v2Deg := v2.Degree()

		// v1 and v2 are constants, we multiply big.Int values and return resulting constant
		if v1Constant && v2Constant {
			n1 = builder.field.Mul(n1, n2)
			return expr.NewConstantExpression(n1)
		}

		// either is constant, we multiply the other by it
		if v1Constant {
			return builder.mulConstant(v2, n1, false)
		}
		if v2Constant {
			return builder.mulConstant(v1, n2, !first)
		}

		// for second degree expressions, we need to compress them to linear
		if v1Deg == 2 {
			v1 = builder.asInternalVariable(v1)
		}
		if v2Deg == 2 {
			v2 = builder.asInternalVariable(v2)
		}

		v1C0, v1C1, _ := v1.CountOfDegrees()
		v2C0, v2C1, _ := v2.CountOfDegrees()
		// if directly multiply, the cost will be
		costDirect := utils.CostOfMultiply(v1C0, v1C1, v2C0, v2C1)
		// compress one of them
		costCompressV1 := utils.CostOfMultiply(v1C0, 1, v2C0, v2C1) + utils.CostOfCompress(0, v1C1, 0)
		costCompressV2 := utils.CostOfMultiply(v1C0, v1C1, v2C0, 1) + utils.CostOfCompress(0, v2C1, 0)
		// compress both
		costCompressBoth := utils.CostOfMultiply(v1C0, 1, v2C0, 1) + utils.CostOfCompress(v1C1, 0, v2C1)
		minCost := costDirect
		if costCompressV1 < minCost {
			minCost = costCompressV1
		}
		if costCompressV2 < minCost {
			minCost = costCompressV2
		}
		if costCompressBoth < minCost {
			minCost = costCompressBoth
		}
		if costCompressBoth == minCost {
			v1 = builder.asInternalVariable(v1)
			v2 = builder.asInternalVariable(v2)
		} else if costCompressV1 == minCost {
			v1 = builder.asInternalVariable(v1)
		} else if costCompressV2 == minCost {
			v2 = builder.asInternalVariable(v2)
		}

		vars := make([]expr.Expression, 0, len(v1))
		for i := 0; i < len(v1); i++ {
			exp := make(expr.Expression, 0, len(v2))
			for j := 0; j < len(v2); j++ {
				coeff := builder.field.Mul(v1[i].Coeff, v2[j].Coeff)
				exp = append(exp, expr.NewTerm(v1[i].VID0, v2[j].VID0, coeff))
			}
			sort.Sort(exp)
			vars = append(vars, exp)
		}
		return builder.add(vars, false, len(v1)*len(v2), nil, false)
	}

	e := builder.newExprList(vars)
	sort.Sort(e)

	// it might be better to implement binary tree multiplication, but
	// almost all calls to Mul have only 2 arguments, so the order might be useless
	res := mul(e.e[0], e.e[1], true)

	for i := 2; i < len(e.e); i++ {
		res = mul(res, e.e[i], false)
	}

	return res
}

func (builder *builder) mulConstant(v1 expr.Expression, lambda constraint.Element, inPlace bool) expr.Expression {
	if lambda.IsZero() {
		return expr.NewConstantExpression(constraint.Element{})
	}

	// multiplying a frontend.Variable by a constant -> we updated the coefficients in the linear expression
	// leading to that frontend.Variable
	var res expr.Expression
	if inPlace {
		res = v1
	} else {
		res = v1.Clone()
	}

	for i := 0; i < len(res); i++ {
		res[i].Coeff = builder.field.Mul(res[i].Coeff, lambda)
	}
	return res
}

// DivHint calculates the division a/b and returns 0 when both a and b are zero.
func DivHint(field *big.Int, inputs []*big.Int, outputs []*big.Int) error {
	x := (&big.Int{}).Mod(inputs[0], field)
	y := (&big.Int{}).Mod(inputs[1], field)
	if y.Cmp(big.NewInt(0)) == 0 {
		if x.Cmp(big.NewInt(0)) == 0 {
			outputs[0] = big.NewInt(0)
			return nil
		}
		return errors.New("divide by zero in DivHint")
	}
	a := (&big.Int{}).ModInverse(y, field)
	a.Mul(a, x)
	a.Mod(a, field)
	outputs[0] = a
	return nil
}

// DivUnchecked returns i1 divided by i2 and returns 0 if both i1 and i2 are zero.
func (builder *builder) DivUnchecked(i1, i2 frontend.Variable) frontend.Variable {
	vars, _ := builder.toVariables(i1, i2)

	v1 := vars[0]
	v2 := vars[1]

	n1, v1Constant := builder.constantValue(v1)
	n2, v2Constant := builder.constantValue(v2)

	if !v2Constant {
		s, _ := builder.NewHint(DivHint, 1, v1, v2)
		builder.AssertIsEqual(builder.Mul(s[0], v2), v1)
		return s[0]
	}

	// v2 is constant
	if n2.IsZero() {
		panic("div by constant(0)")
	}
	n2, _ = builder.field.Inverse(n2)

	if v1Constant {
		n2 = builder.field.Mul(n2, n1)
		return expr.NewLinearExpression(0, n2)
	}

	// v1 is not constant
	return builder.mulConstant(v1, n2, false)
}

// Div returns the result of i1 divided by i2.
func (builder *builder) Div(i1, i2 frontend.Variable) frontend.Variable {
	vars, _ := builder.toVariables(i1, i2)

	v1 := vars[0]
	v2 := vars[1]

	n1, v1Constant := builder.constantValue(v1)
	n2, v2Constant := builder.constantValue(v2)

	if !v2Constant {
		s, _ := builder.NewHint(DivHint, 1, builder.eOne, v2)
		builder.AssertIsEqual(builder.Mul(s[0], v2), builder.eOne)
		return builder.Mul(s[0], v1)
	}

	// v2 is constant
	if n2.IsZero() {
		panic("div by constant(0)")
	}
	n2, _ = builder.field.Inverse(n2)

	if v1Constant {
		n2 = builder.field.Mul(n2, n1)
		return expr.NewLinearExpression(0, n2)
	}

	// v1 is not constant
	return builder.mulConstant(v1, n2, false)
}

// Inverse returns the multiplicative inverse of the given variable.
func (builder *builder) Inverse(i1 frontend.Variable) frontend.Variable {
	vars, _ := builder.toVariables(i1)

	if c, ok := builder.constantValue(vars[0]); ok {
		if c.IsZero() {
			panic("inverse by constant(0)")
		}

		c, _ = builder.field.Inverse(c)
		return expr.NewLinearExpression(0, c)
	}

	s, _ := builder.NewHint(DivHint, 1, builder.eOne, vars[0])
	builder.AssertIsEqual(builder.Mul(s[0], vars[0]), builder.eOne)
	return s[0]
}

// ---------------------------------------------------------------------------------------------
// Bit operations

// ToBinary unpacks a frontend.Variable in binary,
// n is the number of bits to select (starting from lsb)
// n default value is fr.Bits the number of bits needed to represent a field element
//
// The result in in little endian (first bit= lsb)
func (builder *builder) ToBinary(i1 frontend.Variable, n ...int) []frontend.Variable {
	// nbBits
	nbBits := builder.field.FieldBitLen()
	if len(n) == 1 {
		nbBits = n[0]
		if nbBits < 0 {
			panic("invalid n")
		}
	}

	return bits.ToBinary(builder, i1, bits.WithNbDigits(nbBits))
}

// FromBinary packs the given variables, seen as a fr.Element in little endian, into a single variable.
func (builder *builder) FromBinary(_b ...frontend.Variable) frontend.Variable {
	return bits.FromBinary(builder, _b)
}

// Xor computes the logical XOR between two frontend.Variables.
func (builder *builder) Xor(_a, _b frontend.Variable) frontend.Variable {

	vars, _ := builder.toVariables(_a, _b)

	a := vars[0]
	b := vars[1]

	builder.AssertIsBoolean(a)
	builder.AssertIsBoolean(b)

	if len(b) > len(a) {
		a, b = b, a
	}
	t := builder.Sub(builder.eOne, builder.Mul(b, 2))
	t = builder.Add(builder.Mul(a, t), b)

	builder.MarkBoolean(t)

	return builder.ToSingleVariable(t)
}

// Or computes the logical OR between two frontend.Variables.
func (builder *builder) Or(_a, _b frontend.Variable) frontend.Variable {
	vars, _ := builder.toVariables(_a, _b)

	a := vars[0]
	b := vars[1]

	builder.AssertIsBoolean(a)
	builder.AssertIsBoolean(b)

	res := builder.add(
		[]expr.Expression{
			a, b,
			builder.negateLinExp(builder.Mul(a, b).(expr.Expression)),
		},
		false, 0, nil, false,
	)

	builder.MarkBoolean(res)

	return builder.ToSingleVariable(res)
}

// And computes the logical AND between two frontend.Variables.
func (builder *builder) And(_a, _b frontend.Variable) frontend.Variable {
	vars, _ := builder.toVariables(_a, _b)

	a := vars[0]
	b := vars[1]

	builder.AssertIsBoolean(a)
	builder.AssertIsBoolean(b)

	res := builder.Mul(a, b)
	builder.MarkBoolean(res)

	return builder.ToSingleVariable(res)
}

// ---------------------------------------------------------------------------------------------
// Conditionals

// Select yields the second variable if the first is true, otherwise yields the third variable.
func (builder *builder) Select(i0, i1, i2 frontend.Variable) frontend.Variable {
	vars, _ := builder.toVariables(i0, i1, i2)
	cond := vars[0]

	// ensures that cond is boolean
	builder.AssertIsBoolean(cond)

	if c, ok := builder.constantValue(cond); ok {
		// condition is a constant return i1 if true, i2 if false
		if builder.field.IsOne(c) {
			return vars[1]
		}
		return vars[2]
	}

	n1, ok1 := builder.constantValue(vars[1])
	n2, ok2 := builder.constantValue(vars[2])

	if ok1 && ok2 {
		n1 = builder.field.Sub(n1, n2)
		res := builder.Mul(cond, n1)    // no constraint is recorded
		res = builder.Add(res, vars[2]) // no constraint is recorded
		return res
	}

	// special case appearing in AssertIsLessOrEq
	if ok1 {
		if n1.IsZero() {
			v := builder.Sub(builder.eOne, vars[0])
			return builder.Mul(v, vars[2])
		}
	}

	v := builder.Sub(vars[1], vars[2]) // no constraint is recorded
	w := builder.Mul(cond, v)
	return builder.Add(w, vars[2])
}

// Lookup2 performs a 2-bit lookup based on the given bits and values.
func (builder *builder) Lookup2(b0, b1 frontend.Variable, i0, i1, i2, i3 frontend.Variable) frontend.Variable {
	vars, _ := builder.toVariables(b0, b1, i0, i1, i2, i3)
	s0, s1 := vars[0], vars[1]
	in0, in1, in2, in3 := vars[2], vars[3], vars[4], vars[5]

	// ensure that bits are actually bits. Adds no constraints if the variables
	// are already constrained.
	builder.AssertIsBoolean(s0)
	builder.AssertIsBoolean(s1)

	c0, b0IsConstant := builder.constantValue(s0)
	c1, b1IsConstant := builder.constantValue(s1)

	if b0IsConstant && b1IsConstant {
		b0 := builder.field.IsOne(c0)
		b1 := builder.field.IsOne(c1)

		if !b0 && !b1 {
			return in0
		}
		if b0 && !b1 {
			return in1
		}
		if b0 && b1 {
			return in3
		}
		return in2
	}

	// two-bit lookup for the general case can be done with three constraints as
	// following:
	//    (1) (in3 - in2 - in1 + in0) * s1 = tmp1 - in1 + in0
	//    (2) tmp1 * s0 = tmp2
	//    (3) (in2 - in0) * s1 = RES - tmp2 - in0
	// the variables tmp1 and tmp2 are new internal variables and the variables
	// RES will be the returned result

	tmp1 := builder.Add(in3, in0)
	tmp1 = builder.Sub(tmp1, in2, in1)
	tmp1 = builder.Mul(tmp1, s1)
	tmp1 = builder.Add(tmp1, in1)
	tmp1 = builder.Sub(tmp1, in0) // (1) tmp1 = s1 * (in3 - in2 - in1 + in0) + in1 - in0
	tmp2 := builder.Mul(tmp1, s0) // (2) tmp2 = tmp1 * s0
	res := builder.Sub(in2, in0)
	res = builder.Mul(res, s1)
	res = builder.Add(res, tmp2, in0) // (3) res = (v2 - v0) * s1 + tmp2 + in0
	return res
}

// IsZero returns 1 if the given variable is zero, otherwise returns 0.
func (builder *builder) IsZero(i1 frontend.Variable) frontend.Variable {
	vars, _ := builder.toVariables(i1)
	a := vars[0]
	if c, ok := builder.constantValue(a); ok {
		if c.IsZero() {
			return builder.eOne
		}
		return builder.eZero
	}

	// x = 1/a 				// in a hint (x == 0 if a == 0)
	x, err := builder.NewHint(solver.InvZeroHint, 1, a)
	if err != nil {
		// the function errs only if the number of inputs is invalid.
		panic(err)
	}

	// m = -a*x + 1         // constrain m to be 1 if a == 0
	m := builder.Sub(1, builder.Mul(a, x[0]))

	// a * m = 0            // constrain m to be 0 if a != 0
	builder.AssertIsEqual(builder.Mul(a, m), builder.eZero)

	builder.MarkBoolean(m)

	return m
}

// Cmp compares i1 and i2 and returns 1 if i1>i2, 0 if i1=i2, -1 if i1<i2.
func (builder *builder) Cmp(i1, i2 frontend.Variable) frontend.Variable {
	nbBits := builder.field.FieldBitLen()
	// in AssertIsLessOrEq we omitted comparison against modulus for the left
	// side as if `a+r<b` implies `a<b`, then here we compute the inequality
	// directly.
	bi1 := bits.ToBinary(builder, i1, bits.WithNbDigits(nbBits))
	bi2 := bits.ToBinary(builder, i2, bits.WithNbDigits(nbBits))

	res := builder.eZero

	for i := builder.field.FieldBitLen() - 1; i >= 0; i-- {

		iszeroi1 := builder.IsZero(bi1[i])
		iszeroi2 := builder.IsZero(bi2[i])

		i1i2 := builder.And(bi1[i], iszeroi2)
		i2i1 := builder.And(bi2[i], iszeroi1)

		n := builder.Select(i2i1, -1, 0)
		m := builder.Select(i1i2, 1, n)

		res = builder.Select(builder.IsZero(res), m, res).(expr.Expression)

	}
	return res
}

// Println is not implemented and will panic if called.
func (builder *builder) Println(a ...frontend.Variable) {
	panic("unimplemented")
}

// Compiler returns itself as it implements the frontend.Compiler interface.
func (builder *builder) Compiler() frontend.Compiler {
	return builder
}

// Commit is faulty in its current implementation as it merely returns a compile-time random number.
func (builder *builder) Commit(v ...frontend.Variable) (frontend.Variable, error) {
	if !builder.root.commitWarned {
		builder.root.commitWarned = true
		fmt.Println("Warning: Commit uses a compile-time random number, which is not secure!")
	}
	return rand.Int(rand.Reader, builder.Field())
}

// SetGkrInfo is not implemented and will panic if called.
func (builder *builder) SetGkrInfo(info constraint.GkrInfo) error {
	panic("unimplemented")
}

// Output adds the given variable to the circuit's output.
func (builder *builder) Output(x_ frontend.Variable) {
	if builder.root.builder != builder {
		panic("Output can only be called on root circuit")
	}
	x := builder.toVariable(x_)
	builder.output = append(builder.output, x)
}
