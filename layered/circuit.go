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
	Circuits []*Circuit
	Layers   []uint64
	Field    *big.Int
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
	In0  uint64
	In1  uint64
	Out  uint64
	Coef *big.Int
}

// GateAdd represents an addition gate within a circuit layer.
// It specifies the input and output wire indices, and the coefficient
// to be multiplied with the input before being added to the output.
type GateAdd struct {
	In   uint64
	Out  uint64
	Coef *big.Int
}

// GateCst represents a constant gate within a circuit layer.
// It directly adds a constant value, defined by Coef, to the output wire.
type GateCst struct {
	Out  uint64
	Coef *big.Int
}

// GateCustom represents a custom gate within a circuit layer.
// It takes several inputs, and produces an output value.
// The output wire must be dedicated to this gate.
type GateCustom struct {
	GateType uint64
	In       []uint64
	Out      uint64
	Coef     *big.Int
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

// Validate checks the structural integrity of a RootCircuit.
// It ensures that all components and connections within the circuit
// adhere to the expected format and constraints of a layered circuit.
func Validate(rc *RootCircuit) error {
	for i, c := range rc.Circuits {
		if c.InputLen == 0 || (c.InputLen&(c.InputLen-1)) != 0 {
			return fmt.Errorf("circuit %d inputlen %d not power of 2", i, c.InputLen)
		}
		if c.OutputLen == 0 || (c.OutputLen&(c.OutputLen-1)) != 0 {
			return fmt.Errorf("circuit %d outputlen %d not power of 2", i, c.OutputLen)
		}
		for _, m := range c.Mul {
			if m.In0 >= c.InputLen || m.In1 >= c.InputLen || m.Out >= c.OutputLen {
				return fmt.Errorf("circuit %d mul gate (%d, %d, %d) out of range", i, m.In0, m.In1, m.Out)
			}
		}
		for _, a := range c.Add {
			if a.In >= c.InputLen || a.Out >= c.OutputLen {
				return fmt.Errorf("circuit %d add gate (%d, %d) out of range", i, a.In, a.Out)
			}
		}
		for _, cs := range c.Cst {
			if cs.Out >= c.OutputLen {
				return fmt.Errorf("circuit %d const gate %d out of range", i, cs.Out)
			}
		}
		for _, ct := range c.Custom {
			if ct.Out >= c.OutputLen {
				return fmt.Errorf("circuit %d custom gate %d out of range", i, ct.Out)
			}
			for _, in := range ct.In {
				if in >= c.InputLen {
					return fmt.Errorf("circuit %d custom gate input %d out of range", i, in)
				}
			}
		}
		for _, s := range c.SubCircuits {
			if s.Id >= uint64(i) {
				return fmt.Errorf("circuit %d subcircuit %d out of range", i, s.Id)
			}
			sc := rc.Circuits[s.Id]
			for _, a := range s.Allocations {
				if a.InputOffset%sc.InputLen != 0 {
					return fmt.Errorf("circuit %d subcircuit %d input offset %d not aligned to %d", i, s.Id, a.InputOffset, sc.InputLen)
				}
				if a.OutputOffset%sc.OutputLen != 0 {
					return fmt.Errorf("circuit %d subcircuit %d output offset %d not aligned to %d", i, s.Id, a.OutputOffset, sc.OutputLen)
				}
			}
		}
	}
	for i := 1; i < len(rc.Layers); i++ {
		if rc.Circuits[rc.Layers[i]].InputLen != rc.Circuits[rc.Layers[i-1]].OutputLen {
			return fmt.Errorf("circuit %d inputlen %d not equal to circuit %d outputlen %d",
				rc.Layers[i], rc.Circuits[i].InputLen, rc.Layers[i-1], rc.Circuits[i-1].OutputLen,
			)
		}
	}
	return nil
}

// computeMasks computes whether each input/output occurs in each circuit
func computeMasks(rc *RootCircuit) ([][]bool, [][]bool) {
	inputMask := make([][]bool, len(rc.Circuits))
	outputMask := make([][]bool, len(rc.Circuits))
	for i, c := range rc.Circuits {
		inputMask[i] = make([]bool, c.InputLen)
		outputMask[i] = make([]bool, c.OutputLen)
		for _, m := range c.Mul {
			inputMask[i][m.In0] = true
			inputMask[i][m.In1] = true
			outputMask[i][m.Out] = true
		}
		for _, a := range c.Add {
			inputMask[i][a.In] = true
			outputMask[i][a.Out] = true
		}
		for _, cs := range c.Cst {
			outputMask[i][cs.Out] = true
		}
		for _, ct := range c.Custom {
			for _, in := range ct.In {
				inputMask[i][in] = true
			}
			outputMask[i][ct.Out] = true
		}
		for _, s := range c.SubCircuits {
			sc := rc.Circuits[s.Id]
			for _, a := range s.Allocations {
				for j := uint64(0); j < sc.InputLen; j++ {
					inputMask[i][a.InputOffset+j] = inputMask[i][a.InputOffset+j] || inputMask[s.Id][j]
				}
				for j := uint64(0); j < sc.OutputLen; j++ {
					outputMask[i][a.OutputOffset+j] = outputMask[i][a.OutputOffset+j] || outputMask[s.Id][j]
				}
			}
		}
	}
	return inputMask, outputMask
}

// ValidateInitialized verifies that all wire inputs in a RootCircuit
// have been properly initialized. An uninitialized wire input would
// indicate an incomplete or improperly constructed circuit.
func ValidateInitialized(rc *RootCircuit) error {
	inputMask, outputMask := computeMasks(rc)
	for i := 1; i < len(rc.Layers); i++ {
		for j := uint64(0); j < rc.Circuits[rc.Layers[i]].InputLen; j++ {
			if inputMask[rc.Layers[i]][j] && !outputMask[rc.Layers[i-1]][j] {
				return fmt.Errorf("circuit %d input %d not initialized by circuit %d output", rc.Layers[i], j, rc.Layers[i-1])
			}
		}
	}
	return nil
}
