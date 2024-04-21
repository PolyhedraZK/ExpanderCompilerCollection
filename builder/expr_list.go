package builder

import (
	"bytes"
	"sort"

	"github.com/Zklib/gkr-compiler/expr"
)

// Sometimes we need to sort expressions by their layers
type exprList struct {
	e []expr.Expression
	l []int
	b *builder
}

func (builder *builder) newExprList(e []expr.Expression) *exprList {
	ec := make([]expr.Expression, len(e))
	res := &exprList{
		e: ec,
		l: make([]int, len(e)),
		b: builder,
	}
	// In the both use cases of this function, it would be better if the variables are smaller
	// So we try to use the single variable form when possible
	maxl := 1
	for i, x := range e {
		x = builder.tryAsInternalVariable(x)
		res.l[i] = builder.layerOfExpr(x)
		if res.l[i] > maxl {
			maxl = res.l[i]
		}
		res.e[i] = make(expr.Expression, len(x))
		copy(res.e[i], x)
		sort.Sort(res.e[i])
	}
	// for constant, set layer=maxlayer
	for i, x := range e {
		if x.IsConstant() {
			res.l[i] = maxl
		}
	}
	return res
}

func (e *exprList) Len() int {
	return len(e.e)
}

func (e *exprList) Less(i, j int) bool {
	if e.l[i] != e.l[j] {
		return e.l[i] < e.l[j]
	}
	// this should be stable in different runs
	if len(e.e[i]) != len(e.e[j]) {
		return len(e.e[i]) < len(e.e[j])
	}
	for k := 0; k < len(e.e[i]); k++ {
		a := e.e[i][k]
		b := e.e[j][k]
		if a.VID0 != b.VID0 {
			return a.VID0 < b.VID0
		}
		if a.VID1 != b.VID1 {
			return a.VID1 < b.VID1
		}
		ac := a.Coeff.Bytes()
		bc := b.Coeff.Bytes()
		r := bytes.Compare(ac[:], bc[:])
		if r != 0 {
			return r == -1
		}
	}
	return false
}

func (e *exprList) Swap(i, j int) {
	e.e[i], e.e[j] = e.e[j], e.e[i]
	e.l[i], e.l[j] = e.l[j], e.l[i]
}
