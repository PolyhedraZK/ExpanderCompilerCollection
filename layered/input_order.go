package layered

import "github.com/Zklib/gkr-compiler/circuitir"

type InputSolve []InputSolveInstruction

type InputSolveInstruction struct {
	// instruction id
	// specially, for the global input, InsnId == 1 << 62
	InsnId int
	// if this is a hint instruction, InputIds[i] == j -> insn.OutputIds[i] should be put to j-th global input
	CircuitInputIds []int
	// if this is a sub circuit instruction, solve it recursively
	SubCircuit InputSolve
}

func (ctx *compileContext) recordInputOrder(layoutId int) InputSolve {
	l := ctx.layerLayout[layoutId]
	if l.sparse || l.circuitId != 0 || l.layer != 0 {
		panic("unexpected situation")
	}
	lc := ctx.circuits[0].lcs[0]
	v := make(map[int]int)
	for i, x := range l.placementDense {
		if x != -1 {
			v[lc.varIdx[x]] = i
		}
	}
	gi := []int{}
	for i := 1; i <= ctx.circuits[0].circuit.NbExternalInput; i++ {
		gi = append(gi, v[i])
	}

	return append(ctx.getSubCircuitHintInputOrder(l.circuitId, v), InputSolveInstruction{
		InsnId:          1 << 62,
		CircuitInputIds: gi,
	})
}

func (ctx *compileContext) getSubCircuitHintInputOrder(subId uint64, v map[int]int) InputSolve {
	res := InputSolve{}
	ic := ctx.circuits[subId]
	hintInputSubIdx := ic.nbVariable
	for i, insn := range ic.circuit.Instructions {
		if insn.Type == circuitir.IHint {
			p := make([]int, len(insn.OutputIds))
			for j, id := range insn.OutputIds {
				p[j] = v[id]
			}
			res = append(res, InputSolveInstruction{
				InsnId:          i,
				CircuitInputIds: p,
			})
		} else if insn.Type == circuitir.ISubCircuit {
			subc := ctx.circuits[insn.SubCircuitId]
			sv := make(map[int]int)
			for j, x := range subc.hintInputs {
				sv[x] = v[hintInputSubIdx+j]
			}
			res = append(res, InputSolveInstruction{
				InsnId:     i,
				SubCircuit: ctx.getSubCircuitHintInputOrder(insn.SubCircuitId, sv),
			})
		}
	}
	return res
}
