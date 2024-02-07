package layering

import "github.com/Zklib/gkr-compiler/ir"

func (ctx *compileContext) recordInputOrder(layoutId int) ir.InputSolver {
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

	return ir.InputSolver{
		Insn:            ctx.getSubCircuitHintInputOrder(l.circuitId, v),
		CircuitInputIds: gi,
		InputLen:        l.size,
	}
}

func (ctx *compileContext) getSubCircuitHintInputOrder(subId uint64, v map[int]int) []ir.InputSolverInstruction {
	res := []ir.InputSolverInstruction{}
	ic := ctx.circuits[subId]
	hintInputSubIdx := ic.nbVariable
	for i, insn := range ic.circuit.Instructions {
		if insn.Type == ir.IHint {
			p := make([]int, len(insn.OutputIds))
			for j, id := range insn.OutputIds {
				p[j] = v[id]
			}
			res = append(res, ir.InputSolverInstruction{
				InsnId:          i,
				CircuitInputIds: p,
			})
		} else if insn.Type == ir.ISubCircuit {
			subc := ctx.circuits[insn.SubCircuitId]
			sv := make(map[int]int)
			for j, x := range subc.hintInputs {
				sv[x] = v[hintInputSubIdx+j]
			}
			hintInputSubIdx += len(subc.hintInputs)
			res = append(res, ir.InputSolverInstruction{
				InsnId:     i,
				SubCircuit: ctx.getSubCircuitHintInputOrder(insn.SubCircuitId, sv),
			})
		}
	}
	return res
}
