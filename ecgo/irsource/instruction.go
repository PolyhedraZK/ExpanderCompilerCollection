package irsource

import "github.com/consensys/gnark/constraint"

type InstructionType = int

const (
	_ InstructionType = iota
	LinComb
	Mul
	Div
	BoolBinOp
	IsZero
	Commit
	Hint
	ConstantOrRandom
	SubCircuitCall
	UnconstrainedBinOp
	UnconstrainedSelect
)

type Instruction struct {
	Type        InstructionType
	X           int
	Y           int
	Inputs      []int
	NumOutputs  int
	ExtraId     uint64
	LinCombCoef []constraint.Element
	Const       constraint.Element
}
