package builder

import (
	"github.com/Zklib/gkr-compiler/expr"
	"github.com/Zklib/gkr-compiler/ir"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/hash/sha2"
	"github.com/consensys/gnark/std/math/uints"
)

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

// Finalize will process assertBooleans and assertNonZeroes, and return a Circuit IR
func (builder *builder) Finalize() *ir.Circuit {
	for _, cb := range builder.defers {
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
	h, err := sha2.New(builder)
	if err != nil {
		panic(err)
	}
	bytes := []uints.U8{}
	for _, id := range pvIds {
		bits := builder.ToBinary(expr.NewLinearExpression(id, builder.tOne))
		for i := (len(bits) - 1) / 8 * 8; i >= 0; i -= 8 {
			var tmp []frontend.Variable
			if i+8 > len(bits) {
				tmp = bits[i:]
			} else {
				tmp = bits[i : i+8]
			}
			var curbyte frontend.Variable = 0
			for j, b := range tmp {
				curbyte = builder.Add(curbyte, builder.Mul(b, 1<<j))
			}
			bytes = append(bytes, uints.U8{Val: curbyte})
		}
	}
	h.Write(bytes)
	sum := h.Sum()

	// the hash is decomposed into 2 128-bit integers to fit the BN254 field
	for i := 0; i < 2; i++ {
		var cur frontend.Variable = 0
		var mul frontend.Variable = 1
		for j := 15; j >= 0; j-- {
			cur = builder.Add(cur, builder.Mul(sum[i*16+j].Val, mul))
			mul = builder.Mul(mul, 1<<8)
		}
		builder.output = append(builder.output, builder.toVariable(cur))
	}
}
