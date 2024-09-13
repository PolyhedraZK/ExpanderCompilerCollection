package babybear

import (
	"math/big"
	"strconv"

	"github.com/PolyhedraZK/ExpanderCompilerCollection/utils"
	"github.com/consensys/gnark/constraint"
)

const P = 2013265921

var ScalarField = big.NewInt(P)

type Field struct{}

func (engine *Field) FromInterface(i interface{}) constraint.Element {
	b := utils.FromInterface(i)
	b.Mod(&b, ScalarField)
	return constraint.Element{b.Uint64()}
}

func (engine *Field) ToBigInt(c constraint.Element) *big.Int {
	// e := ([6]uint64)(c[:])
	// r := new(big.Int)
	// e.BigInt(r)
	// return r
	return big.NewInt(int64(c[0]))
}

func (engine *Field) Mul(a, b constraint.Element) constraint.Element {
	// _a := engine.ToBigInt(a)
	// _b := engine.ToBigInt(b)
	// _a_b := _a.Mul(_a, _b)
	// ab := _a_b.Mod(_a_b, ScalarField)
	// return constraint.Element{ab.Uint64()}

	// TODO: Mul that doesn't assume a,b reduced (i.e. a[1] = 0, a[2] = 0, etc)
	// (Although note that the below Add always reduces mod P
	// so it may be fine to assume we're always multiplying reduced elements)
	a_b := (a[0] * b[0]) % P
	return constraint.Element{a_b}
}

func (engine *Field) Add(a, b constraint.Element) constraint.Element {
	res := a[0] + b[0]
	if res >= P {
		res -= P
	}
	return constraint.Element{res}
}

func (engine *Field) Sub(a, b constraint.Element) constraint.Element {
	res := int64(a[0]) - int64(b[0])
	if res < 0 {
		res += P
	}
	return constraint.Element{uint64(res)}
}

func (engine *Field) Neg(a constraint.Element) constraint.Element {
	return constraint.Element{(P - a[0]) % P}
}

func (engine *Field) Inverse(a constraint.Element) (constraint.Element, bool) {
	if a[0] == 0 {
		return a, false
	}
	var res uint64 = 1
	b := a[0]
	// Exponentiation to power P-2
	for i := P - 2; i > 0; i >>= 1 {
		if (i & 1) != 0 {
			res = (res * b) % P
		}
		b = (b * b) % P
	}
	return constraint.Element{res}, true
}

func (engine *Field) IsOne(a constraint.Element) bool {
	return a[0] == 1
}

func (engine *Field) One() constraint.Element {
	return constraint.Element{1}
}

func (engine *Field) String(a constraint.Element) string {
	return strconv.Itoa(int(a[0]))
}

func (engine *Field) Uint64(a constraint.Element) (uint64, bool) {
	return a[0], true
}

func (engine *Field) Field() *big.Int {
	return ScalarField
}

func (engine *Field) FieldBitLen() int {
	return 31
}
