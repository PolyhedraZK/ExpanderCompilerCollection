package gkr

import "github.com/Zklib/gkr-compiler/gkr/expr"

// Sometimes we need to sort expressions by their layers
type exprList struct {
	e []expr.Expression
	l []int
	b *builder
}

func (builder *builder) newExprList(e []expr.Expression) *exprList {
	res := &exprList{
		e: e,
		l: make([]int, len(e)),
		b: builder,
	}
	for i, x := range e {
		res.l[i] = builder.layerOfExpr(x)
	}
	return res
}

func (e *exprList) Len() int {
	return len(e.e)
}

func (e *exprList) Less(i, j int) bool {
	return e.l[i] < e.l[j]
}
func (e *exprList) Swap(i, j int) {
	e.e[i], e.e[j] = e.e[j], e.e[i]
	e.l[i], e.l[j] = e.l[j], e.l[i]
}
