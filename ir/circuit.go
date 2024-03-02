package ir

import (
	"fmt"
	"strconv"
	"strings"

	"github.com/Zklib/gkr-compiler/expr"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/constraint/solver"
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

func checkExpr(e expr.Expression, totVID int) error {
	for _, term := range e {
		if term.VID0 < 0 || term.VID0 >= totVID {
			return fmt.Errorf("VID0 %d is out of bound", term.VID0)
		}
		if term.VID1 < 0 || term.VID1 >= totVID {
			return fmt.Errorf("VID1 %d is out of bound", term.VID1)
		}
		// linear term must have VID0 nonzero
		if term.VID0 == 0 && term.VID1 != 0 {
			return fmt.Errorf("VID0 %d is zero but VID1 %d is not", term.VID0, term.VID1)
		}
	}
	if len(e) == 0 {
		return fmt.Errorf("empty expression")
	}
	return nil
}

// Validate checks if the circuit is valid
func Validate(rc *RootCircuit) error {
	for id, c := range rc.Circuits {
		if c.NbExternalInput <= 0 {
			return fmt.Errorf("circuit %d has no external input", id)
		}
		curid := c.NbExternalInput + 1
		for insnId, insn := range c.Instructions {
			for ii, input := range insn.Inputs {
				if err := checkExpr(input, curid); err != nil {
					return fmt.Errorf("circuit %d instruction %d input %d: %v", id, insnId, ii, err)
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
			if err := checkExpr(expr, curid); err != nil {
				return fmt.Errorf("circuit %d output: %v", id, err)
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

func (ci *Circuit) Print(field constraint.R1CS) {
	varToStr := func(e expr.Expression) string {
		s := make([]string, len(e))
		for i, term := range e {
			coeff := field.ToBigInt(term.Coeff).String()
			if term.VID0 == 0 {
				s[i] = coeff
			} else if term.VID1 == 0 {
				s[i] = "v" + strconv.Itoa(term.VID0) + "*" + coeff
			} else {
				s[i] = "v" + strconv.Itoa(term.VID0) + "*v" + strconv.Itoa(term.VID1) + "*" + coeff
			}
		}
		return strings.Join(s, "+")
	}

	for _, insn := range ci.Instructions {
		if insn.Type == IInternalVariable {
			fmt.Printf("v%d = %s\n", insn.OutputIds[0], varToStr(insn.Inputs[0]))
		} else if insn.Type == IHint {
			strs := make([]string, len(insn.Inputs))
			for i, x := range insn.Inputs {
				strs[i] = varToStr(x)
			}
			fmt.Printf("v%d", insn.OutputIds[0])
			for i := 1; i < len(insn.OutputIds); i++ {
				fmt.Printf(",v%d", insn.OutputIds[i])
			}
			fmt.Printf(" = %s(%s)\n", solver.GetHintName(insn.HintFunc), strings.Join(strs, ","))
		} else if insn.Type == ISubCircuit {
			strs := make([]string, len(insn.Inputs))
			for i, x := range insn.Inputs {
				strs[i] = varToStr(x)
			}
			fmt.Printf("v%d", insn.OutputIds[0])
			for i := 1; i < len(insn.OutputIds); i++ {
				fmt.Printf(",v%d", insn.OutputIds[i])
			}
			fmt.Printf(" = sub%d(%s)\n", insn.SubCircuitId, strings.Join(strs, ","))
		}
	}

	for i, e := range ci.Output {
		fmt.Printf("out%d = %s\n", i, varToStr(e))
	}
	for i, e := range ci.Constraints {
		fmt.Printf("con%d = %s\n", i, varToStr(e))
	}
}

func (rc *RootCircuit) Print() {
	for k, v := range rc.Circuits {
		fmt.Printf("Circuit %d nbIn=%d nbOut=%d =================\n", k, v.NbExternalInput, len(v.Output))
		v.Print(rc.Field)
		fmt.Println()
	}
}
