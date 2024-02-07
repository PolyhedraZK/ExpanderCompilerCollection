package ir

import (
	"github.com/Zklib/gkr-compiler/expr"
	"github.com/consensys/gnark/constraint/solver"
)

type InstructionType int

const (
	_                                 = 0
	IInternalVariable InstructionType = iota
	IHint
	ISubCircuit
)

// an instruction can be:
//  1. an internal variable, which compress an expression into a single variable
//  2. a hint, as in gnark
//  3. a sub circuit
type Instruction struct {
	Type         InstructionType
	HintFunc     solver.Hint
	SubCircuitId uint64
	Inputs       []expr.Expression
	OutputIds    []int
}

func NewInternalVariableInstruction(e expr.Expression, o int) Instruction {
	return Instruction{
		Type:      IInternalVariable,
		Inputs:    []expr.Expression{e},
		OutputIds: []int{o},
	}
}

func NewHintInstruction(f solver.Hint, inputs []expr.Expression, outputIds []int) Instruction {
	return Instruction{
		Type:      IHint,
		HintFunc:  f,
		Inputs:    inputs,
		OutputIds: outputIds,
	}
}

func NewSubCircuitInstruction(subId uint64, inputs []expr.Expression, outputsIds []int) Instruction {
	return Instruction{
		Type:         ISubCircuit,
		SubCircuitId: subId,
		Inputs:       inputs,
		OutputIds:    outputsIds,
	}
}
