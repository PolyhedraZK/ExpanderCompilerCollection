package expr

// similar to gnark frontend/internal/expr/term, but we support quadratic variables

import (
	"math/big"

	"github.com/consensys/gnark/constraint"
)

type Term struct {
	// if vid1 is 0, it means linear term.
	// if both vid are 0, it means constant
	VID0  int
	VID1  int
	Coeff constraint.Element
}

func NewTerm(vID0, vID1 int, coeff constraint.Element) Term {
	if vID0 < vID1 {
		vID0, vID1 = vID1, vID0
	}
	return Term{Coeff: coeff, VID0: vID0, VID1: vID1}
}

func (t *Term) SetCoeff(c constraint.Element) {
	t.Coeff = c
}

// Slightly better (?) hash algorithm than the gnark one
func (t Term) HashCode() uint64 {
	x := t.Coeff[0] ^ t.Coeff[1] ^ t.Coeff[2] ^ t.Coeff[3] ^ t.Coeff[4] ^ t.Coeff[5]
	x ^= uint64(t.VID0) * 998244353
	x ^= uint64(t.VID1) * 1000000007
	return x
}

func (t *Term) Degree() int {
	if t.VID0 == 0 {
		return 0
	}
	if t.VID1 == 0 {
		return 1
	}
	return 2
}

// ToBigIntRegular implements gnark toBigIntInterface interface
// Actually it's impossible to convert a Term to big.Int, but sometimes it requires such evaluation (like in gnark utils.FromInterface).
// So a fake implementation is created to provide better instructions for users
func (t Term) ToBigIntRegular(*big.Int) *big.Int {
	panic("Conversion from expr.Term to big.Int triggered, please check the type of the API call here.")
}
