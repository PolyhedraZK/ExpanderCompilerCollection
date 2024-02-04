package builder

import (
	"crypto/rand"
	"sort"

	"github.com/Zklib/gkr-compiler/circuitir"
	"github.com/Zklib/gkr-compiler/expr"
)

func (r *Root) Finalize() *circuitir.RootCircuit {
	res := make(map[uint64]*circuitir.Circuit)
	for x, b := range r.registry {
		res[x] = b.builder.Finalize()
	}
	return &circuitir.RootCircuit{
		Field:    r.field,
		Circuits: res,
	}
}

func shouldAssert(x interface{}) bool {
	return x.(constraintStatus) == asserted
}

// Finalize will process assertBooleans and assertNonZeroes, and return a Circuit IR
func (builder *builder) Finalize() *circuitir.Circuit {
	for _, e := range builder.booleans.FilterKeys(shouldAssert) {
		v := builder.Mul(e, builder.Sub(builder.eOne, e)).(expr.Expression)
		builder.zeroes.Add(v, asserted)
	}
	builder.booleans.Clear()
	for _, e := range builder.nonZeroes.FilterKeys(shouldAssert) {
		builder.Inverse(e)
	}
	builder.nonZeroes.Clear()
	constraints := builder.zeroes.FilterKeys(shouldAssert)

	output := builder.output
	// TODO: this part is just copied from the old circuit.go, it should be removed
	if builder == builder.root.builder {
		e := builder.newExprList(constraints)
		sort.Sort(e)

		wi, _ := rand.Int(rand.Reader, builder.Field())
		w := builder.field.FromInterface(wi)

		curpow := w
		res := make([]expr.Expression, len(e.e))
		for i, x := range e.e {
			res[i] = builder.Mul(curpow, x).(expr.Expression)
			curpow = builder.field.Mul(curpow, w)
		}

		// add the results by layers
		out := builder.layeredAdd(res)
		finalOut := builder.asInternalVariable(out, true)
		output = []expr.Expression{finalOut}
	}

	return &circuitir.Circuit{
		Instructions:    builder.instructions,
		Constraints:     constraints,
		Output:          output,
		NbExternalInput: builder.nbExternalInput,
	}
}
