package layering

import (
	"github.com/PolyhedraZK/ExpanderCompilerCollection/ir"
)

func (ctx *compileContext) recordInputOrder(layoutId int) ir.InputOrder {
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
		if vi, ok := v[i]; ok {
			gi = append(gi, vi)
		} else {
			gi = append(gi, -1)
		}
	}

	return ir.InputOrder{
		Insn:            ctx.getSubCircuitHintInputOrder(l.circuitId, v),
		CircuitInputIds: gi,
		InputLen:        l.size,
	}
}

func (ctx *compileContext) getSubCircuitHintInputOrder(subId uint64, v map[int]int) []ir.InputOrderInstruction {
	ic := ctx.circuits[subId]
	res := make([]ir.InputOrderInstruction, len(ic.circuit.Instructions))
	hintInputSubIdx := ic.nbVariable
	for i, insn := range ic.circuit.Instructions {
		if insn.Type == ir.IHint {
			p := make([]int, len(insn.OutputIds))
			for j, id := range insn.OutputIds {
				if vi, ok := v[id]; ok {
					p[j] = vi
				} else {
					p[j] = -1
				}
			}
			res[i].CircuitInputIds = p
		} else if insn.Type == ir.ISubCircuit {
			subc := ctx.circuits[insn.SubCircuitId]
			sv := make(map[int]int)
			for j, x := range subc.hintInputs {
				if vi, ok := v[hintInputSubIdx+j]; ok {
					sv[x] = vi
				}
			}
			hintInputSubIdx += len(subc.hintInputs)
			res[i].SubCircuit = ctx.getSubCircuitHintInputOrder(insn.SubCircuitId, sv)
		}
	}
	return res
}
