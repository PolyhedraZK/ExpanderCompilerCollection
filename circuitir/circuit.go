package circuitir

import (
	"github.com/Zklib/gkr-compiler/expr"
	"github.com/consensys/gnark/constraint"
)

type Circuit struct {
	// each instruction specifies the method to calculate some variables
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
