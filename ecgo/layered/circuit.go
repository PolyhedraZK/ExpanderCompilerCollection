// Package layered defines the structures and functions necessary for
// creating and manipulating layered circuits within the ExpanderCompilerCollection compiler.
// A layered circuit is a representation of a computation that is divided
// into a sequence of discrete layers, facilitating certain types of
// optimizations and parallel computations.
package layered

import (
	"fmt"
	"math/big"
)

// RootCircuit defines a multi-layered circuit.
// The Layers field specifies the indices of each layer, which are referenced
// through the Circuits array. Field denotes the mathematical field over which
// the circuit operations are carried out.
type RootCircuit struct {
	NumPublicInputs         int
	NumActualOutputs        int
	ExpectedNumOutputZeroes int
	Circuits                []*Circuit
	Layers                  []uint64
	Field                   *big.Int
}

// Circuit represents a single segment within a layered circuit.
// It contains the length of inputs and outputs, a list of subcircuits
// that can be called within this segment, and the gates that perform
// various arithmetic operations within the segment.
type Circuit struct {
	InputLen    uint64
	OutputLen   uint64
	SubCircuits []SubCircuit
	Mul         []GateMul
	Add         []GateAdd
	Cst         []GateCst
	Custom      []GateCustom
}

// SubCircuit represents a subcircuit that is used within a Circuit.
// It has the identifier of the subcircuit (indexed in RootCircuit.Circuits)
// and a list of allocations that define the input and output connections
// to the subcircuit.
type SubCircuit struct {
	Id          uint64
	Allocations []Allocation
}

// Allocation defines the input and output offsets for a subcircuit call.
// These offsets determine where the subcircuit's inputs and outputs
// are positioned within the larger circuit context.
type Allocation struct {
	InputOffset  uint64
	OutputOffset uint64
}

// GateMul represents a multiplication gate within a circuit layer.
// It specifies two input wire indices and an output wire index,
// along with a coefficient. The product of the inputs and the coefficient
// is added to the output.
type GateMul struct {
	In0           uint64
	In1           uint64
	Out           uint64
	Coef          *big.Int
	CoefType      uint8
	PublicInputId uint64
}

// GateAdd represents an addition gate within a circuit layer.
// It specifies the input and output wire indices, and the coefficient
// to be multiplied with the input before being added to the output.
type GateAdd struct {
	In            uint64
	Out           uint64
	Coef          *big.Int
	CoefType      uint8
	PublicInputId uint64
}

// GateCst represents a constant gate within a circuit layer.
// It directly adds a constant value, defined by Coef, to the output wire.
type GateCst struct {
	Out           uint64
	Coef          *big.Int
	CoefType      uint8
	PublicInputId uint64
}

// GateCustom represents a custom gate within a circuit layer.
// It takes several inputs, and produces an output value.
// The output wire must be dedicated to this gate.
type GateCustom struct {
	GateType      uint64
	In            []uint64
	Out           uint64
	Coef          *big.Int
	CoefType      uint8
	PublicInputId uint64
}

// Print outputs the entire circuit structure to the console for debugging purposes.
// It provides a human-readable representation of the circuit's layers, gates, and
// subcircuits, along with their interconnections.
func (c *Circuit) Print() {
	fmt.Printf("Input=%d Output=%d\n", c.InputLen, c.OutputLen)
	for _, sub := range c.SubCircuits {
		fmt.Printf("Apply circuit %d at:\n", sub.Id)
		for _, a := range sub.Allocations {
			fmt.Printf("    InputOffset=%d OutputOffset=%d\n", a.InputOffset, a.OutputOffset)
		}
	}
	for _, m := range c.Mul {
		fmt.Printf("out%d += in%d * in%d * %s\n", m.Out, m.In0, m.In1, m.Coef.String())
	}
	for _, a := range c.Add {
		fmt.Printf("out%d += in%d * %s\n", a.Out, a.In, a.Coef.String())
	}
	for _, c := range c.Cst {
		fmt.Printf("out%d += %s\n", c.Out, c.Coef.String())
	}
	for _, c := range c.Custom {
		fmt.Printf("out%d += custom_gate_%d(", c.Out, c.GateType)
		for i, in := range c.In {
			if i > 0 {
				fmt.Printf(",")
			}
			fmt.Printf("in%d", in)
		}
		fmt.Printf(") * %s\n", c.Coef.String())
	}
}

// Print outputs the entire multi-layered circuit structure to the console for debugging purposes.
// It provides a detailed view of the circuit's construction, including all gates and
// their connections across layers.
func (rc *RootCircuit) Print() {
	for i, c := range rc.Circuits {
		fmt.Printf("Circuit %d: ", i)
		c.Print()
		fmt.Printf("================================\n")
	}
	fmt.Printf("Layers: %v\n", rc.Layers)
}
