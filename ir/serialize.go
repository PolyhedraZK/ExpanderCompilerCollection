package ir

import (
	"bytes"
	"encoding/gob"
	"math/big"

	"github.com/PolyhedraZK/ExpanderCompilerCollection/expr"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/field"
	"github.com/consensys/gnark/constraint/solver"
)

// InputSolverForSerialization is used for serializing an InputSolver.
type InputSolverForSerialization struct {
	Field             *big.Int
	Circuits          map[uint64]*CircuitForSerialization
	InputOrder        *InputOrder
	CircuitsSolveInfo map[uint64]*CircuitSolveInfo
}

// CircuitForSerialization is used for serializing a Circuit to be stored or transmitted.
type CircuitForSerialization struct {
	Instructions    []InstructionForSerialization
	Constraints     []expr.Expression
	Output          []expr.Expression
	NbExternalInput int
}

// InstructionForSerialization is used for serializing an Instruction.
type InstructionForSerialization struct {
	Type           InstructionType
	HintID         solver.HintID
	SubCircuitId   uint64
	CustomGateType uint64
	Inputs         []expr.Expression
	OutputIds      []int
}

// Serialize converts the InputSolver into a byte slice for storage or transmission.
func (is *InputSolver) Serialize() []byte {
	isfs := &InputSolverForSerialization{
		Field:             is.RootCircuit.Field.Field(),
		Circuits:          make(map[uint64]*CircuitForSerialization),
		InputOrder:        is.InputOrder,
		CircuitsSolveInfo: is.CircuitsSolveInfo,
	}
	for id, c := range is.RootCircuit.Circuits {
		cfs := &CircuitForSerialization{
			Instructions:    make([]InstructionForSerialization, len(c.Instructions)),
			Constraints:     c.Constraints,
			Output:          c.Output,
			NbExternalInput: c.NbExternalInput,
		}
		for i, insn := range c.Instructions {
			cfs.Instructions[i] = InstructionForSerialization{
				Type:         insn.Type,
				SubCircuitId: insn.SubCircuitId,
				Inputs:       insn.Inputs,
				OutputIds:    insn.OutputIds,
			}
			if cfs.Instructions[i].Type == IHint || cfs.Instructions[i].Type == ICustomGate {
				cfs.Instructions[i].HintID = solver.GetHintID(insn.HintFunc)
			}
		}
		isfs.Circuits[id] = cfs
	}
	buf := new(bytes.Buffer)
	encoder := gob.NewEncoder(buf)
	err := encoder.Encode(isfs)
	if err != nil {
		panic(err)
	}
	return buf.Bytes()
}

// DeserializeInputSolver creates an InputSolver from serialized data.
func DeserializeInputSolver(data []byte) *InputSolver {
	buf := bytes.NewBuffer(data)
	decoder := gob.NewDecoder(buf)
	isfs := &InputSolverForSerialization{}
	err := decoder.Decode(isfs)
	if err != nil {
		panic(err)
	}
	rc := &RootCircuit{
		Field:    field.GetFieldFromOrder(isfs.Field),
		Circuits: make(map[uint64]*Circuit),
	}
	for id, cfs := range isfs.Circuits {
		c := &Circuit{
			Instructions:    make([]Instruction, len(cfs.Instructions)),
			Constraints:     cfs.Constraints,
			Output:          cfs.Output,
			NbExternalInput: cfs.NbExternalInput,
		}
		for i, insn := range cfs.Instructions {
			c.Instructions[i] = Instruction{
				Type:         insn.Type,
				SubCircuitId: insn.SubCircuitId,
				Inputs:       insn.Inputs,
				OutputIds:    insn.OutputIds,
			}
			if c.Instructions[i].Type == IHint || c.Instructions[i].Type == ICustomGate {
				c.Instructions[i].HintFunc = solver.GetRegisteredHint(insn.HintID)
				if c.Instructions[i].HintFunc == nil {
					panic("hint not registered")
				}
			}
		}
		rc.Circuits[id] = c
	}
	return &InputSolver{
		RootCircuit:       rc,
		InputOrder:        isfs.InputOrder,
		CircuitsSolveInfo: isfs.CircuitsSolveInfo,
	}
}
