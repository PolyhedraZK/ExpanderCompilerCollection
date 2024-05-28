package ir

import (
	"github.com/PolyhedraZK/ExpanderCompilerCollection/expr"
)

// AdjustForLayering adjusts the circuit for layering, ensuring it meets the
// requirements of ValidateForLayering.
func AdjustForLayering(rc *RootCircuit) *RootCircuit {
	res := &RootCircuit{
		Circuits: make(map[uint64]*Circuit),
		Field:    rc.Field,
	}
	for id, c := range rc.Circuits {
		res.Circuits[id] = adjustCircuitForLayering(rc, c)
	}
	return res
}

func adjustCircuitForLayering(rc *RootCircuit, c *Circuit) *Circuit {
	res := &Circuit{
		NbExternalInput: c.NbExternalInput,
	}

	newId := make([]int, c.NbExternalInput+1)
	for i := 0; i <= c.NbExternalInput; i++ {
		newId[i] = i
	}
	nextId := c.NbExternalInput + 1
	internalVarExpr := make(map[int]expr.Expression)

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

	newInternalVariable := func(e expr.Expression) int {
		res.Instructions = append(res.Instructions, Instruction{
			Type:      IInternalVariable,
			OutputIds: []int{nextId},
			Inputs:    []expr.Expression{convertExpr(e)},
		})
		nextId++
		return nextId - 1
	}

	convertToSingleVariables := func(es []expr.Expression, allowSame bool) []expr.Expression {
		occuredId := make(map[int]bool)
		res := make([]expr.Expression, len(es))
		for i, e := range es {
			if !rc.isSingleVariable(e) {
				res[i] = expr.NewLinearExpression(newInternalVariable(e), rc.Field.One())
			} else {
				_, ok := occuredId[e[0].VID0]
				if ok && !allowSame {
					if er, ok := internalVarExpr[e[0].VID0]; ok {
						res[i] = expr.NewLinearExpression(newInternalVariable(er), rc.Field.One())
					} else {
						res[i] = expr.NewLinearExpression(newInternalVariable(e), rc.Field.One())
					}
				} else {
					res[i] = convertExpr(e)
				}
			}
			occuredId[res[i][0].VID0] = true
		}
		return res
	}

	for _, insn := range c.Instructions {
		newInsn := Instruction{
			Type:         insn.Type,
			HintFunc:     insn.HintFunc,
			SubCircuitId: insn.SubCircuitId,
		}

		if insn.Type == IInternalVariable {
			internalVarExpr[insn.OutputIds[0]] = insn.Inputs[0]
		}

		// for sub circuit insn, we need to convert the input into single variables
		if insn.Type == ISubCircuit {
			newInsn.Inputs = convertToSingleVariables(insn.Inputs, false)
		} else {
			for _, input := range insn.Inputs {
				newInsn.Inputs = append(newInsn.Inputs, convertExpr(input))
			}
		}
		for _, output := range insn.OutputIds {
			newId = append(newId, nextId)
			newInsn.OutputIds = append(newInsn.OutputIds, newId[output])
			nextId++
		}
		res.Instructions = append(res.Instructions, newInsn)
	}

	res.Output = convertToSingleVariables(c.Output, false)
	res.Constraints = convertToSingleVariables(c.Constraints, true)

	return res
}
