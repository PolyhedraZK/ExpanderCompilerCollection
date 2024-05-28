package builder

import (
	"github.com/PolyhedraZK/ExpanderCompilerCollection/expr"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/ir"
)

// Finalize processes deferred functions, converts boolean and nonzero assertions to zero assertions,
// and adds public variables to the output.
func (r *Root) Finalize() *ir.RootCircuit {
	if len(r.publicVariables) > 0 {
		r.finalizePublicVariables(r.publicVariables)
	}

	res := make(map[uint64]*ir.Circuit)
	for x, b := range r.registry.m {
		res[x] = b.builder.Finalize()
	}
	return &ir.RootCircuit{
		Field:    r.field,
		Circuits: res,
	}
}

func shouldAssert(x interface{}) bool {
	return x.(constraintStatus) == asserted
}

func (builder *builder) Finalize() *ir.Circuit {
	// defers may change during the process
	for i := 0; i < len(builder.defers); i++ {
		cb := builder.defers[i]
		err := cb(builder)
		if err != nil {
			panic(err)
		}
	}

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

	return &ir.Circuit{
		Instructions:    builder.instructions,
		Constraints:     constraints,
		Output:          builder.output,
		NbExternalInput: builder.nbExternalInput,
	}
}

// This function will only be called on the root circuit
func (builder *builder) finalizePublicVariables(pvIds []int) {
	for _, id := range pvIds {
		builder.output = append(builder.output, expr.NewLinearExpression(id, builder.tOne))
	}
}
