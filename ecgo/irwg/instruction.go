package irwg

import "github.com/consensys/gnark/constraint"

type InstructionType = int

const (
	_ InstructionType = iota
	LinComb
	Mul
	Hint
	ConstantOrRandom
	SubCircuitCall
)

type Instruction struct {
	Type        InstructionType
	Inputs      []int
	NumOutputs  int
	ExtraId     uint64
	LinCombCoef []constraint.Element
	Const       constraint.Element
}