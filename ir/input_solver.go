package ir

import (
	"fmt"
	"math/big"

	"github.com/Zklib/gkr-compiler/expr"
	fr_bn254 "github.com/consensys/gnark-crypto/ecc/bn254/fr"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/frontend"
)

type InputSolver struct {
	Insn            []InputSolverInstruction
	CircuitInputIds []int
	InputLen        int
}

type InputSolverInstruction struct {
	// instruction id
	// specially, for the global input, InsnId == 1 << 62
	InsnId int
	// if this is a hint instruction, InputIds[i] == j -> insn.OutputIds[i] should be put to j-th global input
	CircuitInputIds []int
	// if this is a sub circuit instruction, solve it recursively
	SubCircuit []InputSolverInstruction
}

func (rc *RootCircuit) SolveInput(assignment frontend.Circuit, solver *InputSolver) []*big.Int {
	wit, err := frontend.NewWitness(assignment, rc.Field.Field())
	if err != nil {
		panic(err)
	}
	vec := wit.Vector().(fr_bn254.Vector)

	globalInput := make([]*big.Int, solver.InputLen)

	input := make([]constraint.Element, len(vec))
	for i, x := range vec {
		var t big.Int
		x.BigInt(&t)
		input[i] = rc.Field.FromInterface(t)
		globalInput[solver.CircuitInputIds[i]] = &t
	}

	circuit := rc.Circuits[0]
	circuit.SolveInput(rc, input, solver.Insn, globalInput)

	for i, x := range globalInput {
		if x == nil {
			globalInput[i] = big.NewInt(0)
		}
	}

	return globalInput
}

func (circuit *Circuit) SolveInput(
	rc *RootCircuit, input []constraint.Element, solver []InputSolverInstruction, globalInput []*big.Int,
) []constraint.Element {
	n := 0
	for _, insn := range circuit.Instructions {
		for _, x := range insn.OutputIds {
			if x > n {
				n = x
			}
		}
	}
	n++

	if len(input) != circuit.NbExternalInput {
		panic("unexpected: variable count mismatch")
	}

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

	for i, x := range input {
		values[i+1] = x
	}
	for i := 0; i < len(input)+1; i++ {
		filled[i] = true
	}

	for i, insn := range circuit.Instructions {
		is := &InputSolverInstruction{}
		if len(solver) > 0 && solver[0].InsnId == i {
			is = &solver[0]
			solver = solver[1:]
		}

		in := make([]constraint.Element, len(insn.Inputs))
		out := make([]constraint.Element, len(insn.OutputIds))

		for i, e := range insn.Inputs {
			in[i] = calcExpr(e)
		}

		if insn.Type == IInternalVariable {
			out[0] = in[0]
		} else if insn.Type == IHint {
			inB := make([]*big.Int, len(insn.Inputs))
			outB := make([]*big.Int, len(insn.OutputIds))
			for i, e := range in {
				inB[i] = rc.Field.ToBigInt(e)
			}
			for i := 0; i < len(insn.OutputIds); i++ {
				outB[i] = big.NewInt(0)
			}
			err := insn.HintFunc(rc.Field.Field(), inB, outB)
			if err != nil {
				panic(err)
			}
			for i, x := range outB {
				out[i] = rc.Field.FromInterface(x)
				fmt.Printf("set %d %d\n", is.CircuitInputIds[i], x)
				globalInput[is.CircuitInputIds[i]] = x
			}
		} else if insn.Type == ISubCircuit {
			rc.Circuits[insn.SubCircuitId].SolveInput(rc, in, is.SubCircuit, globalInput)
		}

		for i, x := range insn.OutputIds {
			if filled[x] {
				panic("unexpected: filled twice")
			}
			filled[x] = true
			values[x] = out[i]
		}
	}

	res := make([]constraint.Element, len(circuit.Output))
	for i, e := range circuit.Output {
		res[i] = calcExpr(e)
	}
	return res
}
