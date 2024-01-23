package expr

import (
	"github.com/consensys/gnark/constraint"
)

type Expression []Term

func NewConstantExpression(c constraint.Element) Expression {
	return Expression{NewTerm(0, 0, c)}
}

func NewLinearExpression(v int, c constraint.Element) Expression {
	return Expression{NewTerm(v, 0, c)}
}

func NewQuadraticExpression(v0, v1 int, c constraint.Element) Expression {
	return Expression{NewTerm(v0, v1, c)}
}

func (e Expression) Clone() Expression {
	res := make(Expression, len(e))
	copy(res, e)
	return res
}

// Len return the length of the Variable (implements Sort interface)
func (e Expression) Len() int {
	return len(e)
}

// Equals returns true if both SORTED expressions are the same
//
// pre conditions: l and o are sorted
func (e Expression) Equal(o Expression) bool {
	if len(e) != len(o) {
		return false
	}
	if (e == nil) != (o == nil) {
		return false
	}
	for i := 0; i < len(e); i++ {
		if e[i] != o[i] {
			return false
		}
	}
	return true
}

// Swap swaps terms in the Variable (implements Sort interface)
func (e Expression) Swap(i, j int) {
	e[i], e[j] = e[j], e[i]
}

// Less returns true if variableID for term at i is less than variableID for term at j (implements Sort interface)
func (e Expression) Less(i, j int) bool {
	if e[i].VID0 != e[j].VID0 {
		return e[i].VID0 < e[j].VID0
	}
	return e[i].VID1 < e[j].VID1
}

// HashCode returns a fast-to-compute but NOT collision resistant hash code identifier for the linear expression
//
// requires sorted
func (e Expression) HashCode() uint64 {
	// TODO: improve this
	h := uint64(17)
	for _, val := range e {
		h = h*23 + val.HashCode()
	}
	return h
}

func (e Expression) Degree() int {
	res := 0
	for _, val := range e {
		deg := val.Degree()
		if deg == 2 {
			return 2
		}
		if deg > res {
			res = deg
		}
	}
	return res
}
