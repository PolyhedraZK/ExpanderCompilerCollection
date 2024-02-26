package ir

import "github.com/Zklib/gkr-compiler/expr"

func optimizeUnusedRoot(rc *RootCircuit) *RootCircuit {
	res := &RootCircuit{
		Field:    rc.Field,
		Circuits: make(map[uint64]*Circuit),
	}
	res.Circuits[0] = optimizeUnused(rc, res, rc.Circuits[0])
	return res
}

// optimizeUnused removes unused instructions and variables in a circuit
// it follows a simple strategy:
// 1. mark all output, constraint and external input as used
// 2. if an output of an instruction is used, mark all its inputs as used
// TODO: non-user constraints might be removable
func optimizeUnused(rc *RootCircuit, newrc *RootCircuit, c *Circuit) *Circuit {
	varInsnId := make([]int, c.NbExternalInput+1)
	for i := 0; i <= c.NbExternalInput; i++ {
		varInsnId[i] = -1
	}
	for i, insn := range c.Instructions {
		for _ = range insn.OutputIds {
			varInsnId = append(varInsnId, i)
		}
	}
	isUsed := make([]bool, len(c.Instructions))
	markUsed := func(e expr.Expression) {
		for _, term := range e {
			if varInsnId[term.VID0] >= 0 {
				isUsed[varInsnId[term.VID0]] = true
			}
			if varInsnId[term.VID1] >= 0 {
				isUsed[varInsnId[term.VID1]] = true
			}
		}
	}
	for _, e := range c.Output {
		markUsed(e)
	}
	for _, e := range c.Constraints {
		markUsed(e)
	}
	for i := len(c.Instructions) - 1; i >= 0; i-- {
		if isUsed[i] {
			for _, e := range c.Instructions[i].Inputs {
				markUsed(e)
			}
		}
	}

	newId := make([]int, len(varInsnId))
	for i := 0; i <= c.NbExternalInput; i++ {
		newId[i] = i
	}
	nextId := c.NbExternalInput + 1

	convertExpr := func(e expr.Expression) expr.Expression {
		res := make(expr.Expression, len(e))
		for i, term := range e {
			res[i] = expr.Term{
				VID0:  newId[term.VID0],
				VID1:  newId[term.VID1],
				Coeff: term.Coeff,
			}
		}
		return res
	}

	res := &Circuit{
		NbExternalInput: c.NbExternalInput,
		Instructions:    make([]Instruction, 0, len(c.Instructions)),
		Output:          make([]expr.Expression, len(c.Output)),
		Constraints:     make([]expr.Expression, len(c.Constraints)),
	}

	for i, insn := range c.Instructions {
		if isUsed[i] {
			if insn.Type == ISubCircuit {
				if _, ok := newrc.Circuits[insn.SubCircuitId]; !ok {
					newrc.Circuits[insn.SubCircuitId] = optimizeUnused(rc, newrc, rc.Circuits[insn.SubCircuitId])
				}
			}
			res.Instructions = append(res.Instructions, Instruction{
				Type:         insn.Type,
				HintFunc:     insn.HintFunc,
				SubCircuitId: insn.SubCircuitId,
				OutputIds:    make([]int, len(insn.OutputIds)),
				Inputs:       make([]expr.Expression, len(insn.Inputs)),
			})
			newInsn := &res.Instructions[len(res.Instructions)-1]
			for j, x := range insn.OutputIds {
				newInsn.OutputIds[j] = nextId
				newId[x] = nextId
				nextId++
			}
			for j, x := range insn.Inputs {
				newInsn.Inputs[j] = convertExpr(x)
			}
		}
	}

	for i, out := range c.Output {
		res.Output[i] = convertExpr(out)
	}
	for i, con := range c.Constraints {
		res.Constraints[i] = convertExpr(con)
	}
	return res
}

func Optimize(c *RootCircuit) *RootCircuit {

	// TODO: fix optimizeUnusedRoot
	// The current implementation is incorrect: when a subcircuit has user-defined assertions but no output, it will be removed

	return c
	//return optimizeUnusedRoot(c)
}
