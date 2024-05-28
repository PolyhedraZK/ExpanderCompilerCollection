package ir

import "github.com/Zklib/gkr-compiler/expr"

type determinableChecker struct {
	rc  *RootCircuit
	res bool
}

// IsAllHintsSolvingTimeDeterminable checks if every input variable of hints is solving time determinable.
// It returns false if the output of GetRandomValue is used in hints, as these cannot be determined at solving time.
func IsAllHintsSolvingTimeDeterminable(rc *RootCircuit) bool {
	dc := determinableChecker{
		rc:  rc,
		res: true,
	}
	in := make([]bool, rc.Circuits[0].NbExternalInput)
	for i := range in {
		in[i] = true
	}
	dc.call(0, in)
	return dc.res
}

func (dc *determinableChecker) call(id uint64, in []bool) []bool {
	c := dc.rc.Circuits[id]
	determinable := append([]bool{true}, in...)
	isExprDeterminable := func(e expr.Expression) bool {
		for _, term := range e {
			if !determinable[term.VID0] {
				return false
			}
			if !determinable[term.VID1] {
				return false
			}
		}
		return true
	}
	for _, insn := range c.Instructions {
		if insn.Type == IGetRandom {
			determinable = append(determinable, false)
		} else {
			subIn := make([]bool, len(insn.Inputs))
			for i, e := range insn.Inputs {
				subIn[i] = isExprDeterminable(e)
			}
			if insn.Type == ISubCircuit {
				determinable = append(determinable, dc.call(insn.SubCircuitId, subIn)...)
			} else if insn.Type == IInternalVariable {
				determinable = append(determinable, subIn...)
			} else if insn.Type == IHint {
				for _, indet := range subIn {
					if !indet {
						dc.res = false
					}
				}
				for range insn.OutputIds {
					determinable = append(determinable, true)
				}
			}
		}
	}
	res := make([]bool, len(c.Output))
	for i, e := range c.Output {
		res[i] = isExprDeterminable(e)
	}
	return res
}
