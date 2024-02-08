package ir

import (
	"fmt"

	"github.com/Zklib/gkr-compiler/expr"
	"github.com/consensys/gnark/constraint"
)

type Circuit struct {
	// each instruction specifies the method to calculate some variables
	// the output id must be sequential
	Instructions []Instruction
	// each constraint constrains some expression to be zero
	Constraints []expr.Expression
	// each output gate of the circuit
	Output []expr.Expression
	// number of input gates
	// TODO: public input
	NbExternalInput int
}

type RootCircuit struct {
	Field constraint.R1CS
	// circuit list, we assume idx 0 is the root circuit
	Circuits map[uint64]*Circuit
}

// Validate checks if the circuit is valid
func Validate(rc *RootCircuit) error {
	for id, c := range rc.Circuits {
		if c.NbExternalInput <= 0 {
			return fmt.Errorf("circuit %d has no external input", id)
		}
		if id == 0 {
			if len(c.Output) != 0 {
				return fmt.Errorf("root circuit should not have output")
			}
		} else {
			if len(c.Output) == 0 {
				return fmt.Errorf("circuit %d has no output", id)
			}
		}
		curid := c.NbExternalInput + 1
		for insnId, insn := range c.Instructions {
			for _, input := range insn.Inputs {
				for _, term := range input {
					if term.VID0 < 0 || term.VID0 >= curid {
						return fmt.Errorf("circuit %d instruction %d input VID0 %d is out of bound", id, insnId, term.VID0)
					}
					if term.VID1 < 0 || term.VID1 >= curid {
						return fmt.Errorf("circuit %d instruction %d input VID1 %d is out of bound", id, insnId, term.VID1)
					}
					// linear term must have VID0 nonzero
					if term.VID0 == 0 && term.VID1 != 0 {
						return fmt.Errorf("circuit %d instruction %d input VID0 %d is zero but VID1 %d is not", id, insnId, term.VID0, term.VID1)
					}
				}
			}
			for _, output := range insn.OutputIds {
				if output != curid {
					return fmt.Errorf("circuit %d instruction %d output id %d is not sequential", id, insnId, output)
				}
				curid++
			}
			if insn.Type == ISubCircuit {
				if _, ok := rc.Circuits[insn.SubCircuitId]; !ok {
					return fmt.Errorf("circuit %d instruction %d subcircuit %d is not found", id, insnId, insn.SubCircuitId)
				}
				if insn.SubCircuitId == id {
					return fmt.Errorf("circuit %d instruction %d subcircuit %d is self", id, insnId, insn.SubCircuitId)
				}
			}
		}
		for _, expr := range append(c.Output, c.Constraints...) {
			for _, term := range expr {
				if term.VID0 < 0 || term.VID0 >= curid {
					return fmt.Errorf("circuit %d output VID0 %d is out of bound", id, term.VID0)
				}
				if term.VID1 < 0 || term.VID1 >= curid {
					return fmt.Errorf("circuit %d output VID1 %d is out of bound", id, term.VID1)
				}
				// linear term must have VID0 nonzero
				if term.VID0 == 0 && term.VID1 != 0 {
					return fmt.Errorf("circuit %d output VID0 %d is zero but VID1 %d is not", id, term.VID0, term.VID1)
				}
			}
		}
	}
	if _, ok := rc.Circuits[0]; !ok {
		return fmt.Errorf("root circuit is not found")
	}

	return nil
}

func (rc *RootCircuit) isSingleVariable(e expr.Expression) bool {
	return len(e) == 1 && e[0].VID1 == 0 && e[0].VID0 != 0 && rc.Field.IsOne(e[0].Coeff)
}

// ValidateForLayering checks if the circuit is valid for layering
// It requires that all outputs, constraints, and sub circuit inputs are single variable
func ValidateForLayering(rc *RootCircuit) error {
	err := Validate(rc)
	if err != nil {
		return err
	}
	for id, c := range rc.Circuits {
		for insnId, insn := range c.Instructions {
			if insn.Type == ISubCircuit {
				for i, input := range insn.Inputs {
					if !rc.isSingleVariable(input) {
						return fmt.Errorf("circuit %d instruction %d input %d is not single variable", id, insnId, i)
					}
					for j := 0; j < i; j++ {
						if input[0].VID0 == insn.Inputs[j][0].VID0 {
							return fmt.Errorf("circuit %d instruction %d input %d is the same as input %d", id, insnId, i, j)
						}
					}
				}
			}
		}
		for _, expr := range append(c.Output, c.Constraints...) {
			if !rc.isSingleVariable(expr) {
				return fmt.Errorf("circuit %d output is not single variable", id)
			}
		}
		for i, expr := range c.Output {
			for j := 0; j < i; j++ {
				if expr[0].VID0 == c.Output[j][0].VID0 {
					return fmt.Errorf("circuit %d output %d is the same as output %d", id, i, j)
				}
			}
		}
	}
	return nil
}
