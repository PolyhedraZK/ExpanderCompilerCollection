package ir

import (
	"math/big"

	"github.com/Zklib/gkr-compiler/expr"
	fr_bn254 "github.com/consensys/gnark-crypto/ecc/bn254/fr"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/frontend"
)

// TODO: support multiple
func (rc *RootCircuit) Eval(assignment frontend.Circuit) []constraint.Element {
	wit, err := frontend.NewWitness(assignment, rc.Field.Field())
	if err != nil {
		panic(err)
	}
	vec := wit.Vector().(fr_bn254.Vector)

	ci := rc.Circuits[0]
	n := ci.Output[0][0].VID0 + 1
	values := make([]constraint.Element, n)
	filled := make([]bool, n)
	values[0] = rc.Field.One()

	calcExpr := func(e expr.Expression) constraint.Element {
		res := constraint.Element{}
		for _, term := range e {
			if !filled[term.VID0] || !filled[term.VID1] {
				panic("unexpected: unfilled values")
			}
			x := rc.Field.Mul(values[term.VID0], values[term.VID1])
			x = rc.Field.Mul(x, term.Coeff)
			res = rc.Field.Add(res, x)
		}
		return res
	}

	for i, x := range vec {
		var t big.Int
		x.BigInt(&t)
		values[i+1] = rc.Field.FromInterface(t)
	}

	if len(vec) != ci.NbExternalInput {
		panic("unexpected: variable count mismatch")
	}
	for i := 0; i < ci.NbExternalInput+1; i++ {
		filled[i] = true
	}

	for _, insn := range ci.Instructions {
		in := make([]*big.Int, len(insn.Inputs))
		out := make([]*big.Int, len(insn.OutputIds))

		for i, e := range insn.Inputs {
			in[i] = rc.Field.ToBigInt(calcExpr(e))
		}
		for i := 0; i < len(insn.OutputIds); i++ {
			out[i] = big.NewInt(0)
		}

		if insn.Type == IInternalVariable {
			out[0].Set(in[0])
		} else {
			err := insn.HintFunc(rc.Field.Field(), in, out)
			if err != nil {
				panic(err)
			}
		}

		for i, x := range insn.OutputIds {
			if filled[x] {
				panic("unexpected: filled twice")
			}
			filled[x] = true
			values[x] = rc.Field.FromInterface(out[i])
		}
	}

	return values
}
