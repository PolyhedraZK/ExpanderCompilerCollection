package builder

import (
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

	constraints_ := builder.zeroes.FilterKeys(shouldAssert)
	constraints := make([]expr.Expression, len(constraints_))
	for i, e := range constraints_ {
		constraints[i] = e.(expr.Expression)
	}

	return &circuitir.Circuit{
		Instructions:    builder.instructions,
		Constraints:     constraints,
		Output:          builder.output,
		NbExternalInput: builder.nbExternalInput,
	}
}
