package builder

import (
	"fmt"
	"math/big"

	"github.com/Zklib/gkr-compiler/expr"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/math/bits"
)

// AssertIsEqual adds an assertion in the constraint builder (i1 == i2)
func (builder *builder) AssertIsEqual(i1, i2 frontend.Variable) {
	x := builder.Sub(i1, i2).(expr.Expression)
	v, xConstant := builder.constantValue(x)
	if xConstant {
		if !v.IsZero() {
			panic("AssertIsEqual will never be satisfied on nonzero constant")
		}
		return
	}

	builder.zeroes.Add(x, asserted)
}

// AssertIsDifferent constrain i1 and i2 to be different
func (builder *builder) AssertIsDifferent(i1, i2 frontend.Variable) {
	s := builder.Sub(i1, i2).(expr.Expression)
	if len(s) == 1 && s[0].Coeff.IsZero() {
		panic("AssertIsDifferent(x,x) will never be satisfied")
	}

	builder.nonZeroes.Add(s, asserted)
}

// AssertIsBoolean adds an assertion in the constraint builder (v == 0 ∥ v == 1)
func (builder *builder) AssertIsBoolean(i1 frontend.Variable) {
	v := builder.toVariable(i1)

	if b, ok := builder.constantValue(v); ok {
		if !(b.IsZero() || builder.field.IsOne(b)) {
			panic("assertIsBoolean failed: constant is not 0 or 1")
		}
		return
	}

	builder.booleans.Add(v, asserted)
}

// new API added in gnark 0.9.2
func (builder *builder) AssertIsCrumb(i1 frontend.Variable) {
	i1 = builder.MulAcc(builder.Mul(-3, i1), i1, i1)
	i1 = builder.MulAcc(builder.Mul(2, i1), i1, i1)
	builder.AssertIsEqual(i1, 0)
}

// AssertIsLessOrEqual adds assertion in constraint builder  (v ⩽ bound)
//
// bound can be a constant or a Variable
//
// derived from:
// https://github.com/zcash/zips/blob/main/protocol/protocol.pdf
func (builder *builder) AssertIsLessOrEqual(v frontend.Variable, bound frontend.Variable) {
	cv, vConst := builder.constantValue(v)
	cb, bConst := builder.constantValue(bound)

	// both inputs are constants
	if vConst && bConst {
		bv, bb := builder.field.ToBigInt(cv), builder.field.ToBigInt(cb)
		if bv.Cmp(bb) == 1 {
			panic(fmt.Sprintf("AssertIsLessOrEqual: %s > %s", bv.String(), bb.String()))
		}
	}

	nbBits := builder.field.FieldBitLen()
	vBits := bits.ToBinary(builder, v, bits.WithNbDigits(nbBits), bits.WithUnconstrainedOutputs())

	// bound is constant
	if bConst {
		builder.MustBeLessOrEqCst(vBits, builder.field.ToBigInt(cb), v)
		return
	}

	builder.mustBeLessOrEqVar(v, bound)
}

func (builder *builder) mustBeLessOrEqVar(a, bound frontend.Variable) {
	// here bound is NOT a constant,
	// but a can be either constant or a wire.

	nbBits := builder.field.FieldBitLen()

	aBits := bits.ToBinary(builder, a, bits.WithNbDigits(nbBits), bits.WithUnconstrainedOutputs(), bits.OmitModulusCheck())
	boundBits := bits.ToBinary(builder, bound, bits.WithNbDigits(nbBits))

	p := make([]frontend.Variable, nbBits+1)
	p[nbBits] = builder.eOne

	zero := builder.eZero

	for i := nbBits - 1; i >= 0; i-- {

		// if bound[i] == 0
		// 		p[i] = p[i+1]
		//		t = p[i+1]
		// else
		// 		p[i] = p[i+1] * a[i]
		//		t = 0
		v := builder.Mul(p[i+1], aBits[i])
		p[i] = builder.Select(boundBits[i], v, p[i+1])

		t := builder.Select(boundBits[i], zero, p[i+1])

		// (1 - t - ai) * ai == 0
		var l frontend.Variable
		l = builder.eOne
		l = builder.Sub(l, t, aBits[i])

		// note if bound[i] == 1, this constraint is (1 - ai) * ai == 0
		// → this is a boolean constraint
		// if bound[i] == 0, t must be 0 or 1, thus ai must be 0 or 1 too

		builder.AssertIsEqual(builder.Mul(l, aBits[i]), zero)
	}
}

// MustBeLessOrEqCst asserts that value represented using its bit decomposition
// aBits is less or equal than constant bound. The method boolean constraints
// the bits in aBits, so the caller can provide unconstrained bits.
func (builder *builder) MustBeLessOrEqCst(aBits []frontend.Variable, bound *big.Int, aForDebug frontend.Variable) {
	nbBits := builder.field.FieldBitLen()
	if len(aBits) > nbBits {
		panic("more input bits than field bit length")
	}
	for i := len(aBits); i < nbBits; i++ {
		aBits = append(aBits, 0)
	}

	// ensure the bound is positive, it's bit-len doesn't matter
	if bound.Sign() == -1 {
		panic("AssertIsLessOrEqual: bound must be positive")
	}
	if bound.BitLen() > nbBits {
		panic("AssertIsLessOrEqual: bound is too large, constraint will never be satisfied")
	}

	// t trailing bits in the bound
	t := 0
	for i := 0; i < nbBits; i++ {
		if bound.Bit(i) == 0 {
			break
		}
		t++
	}

	p := make([]frontend.Variable, nbBits+1)
	// p[i] == 1 → a[j] == c[j] for all j ⩾ i
	p[nbBits] = builder.eOne

	for i := nbBits - 1; i >= t; i-- {
		if bound.Bit(i) == 0 {
			p[i] = p[i+1]
		} else {
			p[i] = builder.Mul(p[i+1], aBits[i])
		}
	}

	for i := nbBits - 1; i >= 0; i-- {
		if bound.Bit(i) == 0 {
			// (1 - p(i+1) - ai) * ai == 0
			l := builder.Sub(1, p[i+1])
			l = builder.Sub(l, aBits[i])

			builder.AssertIsEqual(builder.Mul(l, aBits[i]), builder.eZero)
		} else {
			builder.AssertIsBoolean(aBits[i])
		}
	}
}
