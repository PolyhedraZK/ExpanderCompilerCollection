// Package layered defines the structures and functions necessary for
// creating and manipulating layered circuits within the ExpanderCompilerCollection compiler.
// A layered circuit is a representation of a computation that is divided
// into a sequence of discrete layers, facilitating certain types of
// optimizations and parallel computations.
package layered

import (
	"fmt"
	"math/big"
	"sort"
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

// A generic gate interface
type Gate interface {
	InWires() []uint64
	OutWire() uint64
	CoefValue() *big.Int
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

// Input wires of mul gate
func (g GateMul) InWires() []uint64 {
	return []uint64{g.In0, g.In1}
}

// Output wire of mul gate
func (g GateMul) OutWire() uint64 {
	return g.Out
}

// Coef of mul gate
func (g GateMul) CoefValue() *big.Int {
	return g.Coef
}

// GateAdd represents an addition gate within a circuit layer.
// It specifies the input and output wire indices, and the coefficient
// to be multiplied with the input before being added to the output.
type GateAdd struct {
	In   uint64
	Out  uint64
	Coef *big.Int
}

// Input wire of mul gate
func (g GateAdd) InWires() []uint64 {
	return []uint64{g.In}
}

// Output wire of add gate
func (g GateAdd) OutWire() uint64 {
	return g.Out
}

// Coef value of add gate
func (g GateAdd) CoefValue() *big.Int {
	return g.Coef
}

// GateCst represents a constant gate within a circuit layer.
// It directly adds a constant value, defined by Coef, to the output wire.
type GateCst struct {
	Out  uint64
	Coef *big.Int
}

// Input wires of const gate
func (g GateCst) InWires() []uint64 {
	return []uint64{}
}

// Output wire of const gate
func (g GateCst) OutWire() uint64 {
	return g.Out
}

// coefficient value of const gate
func (g GateCst) CoefValue() *big.Int {
	return g.Coef
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

// Input wires of customized gate
func (g GateCustom) InWires() []uint64 {
	return g.In
}

// Output wires of customized gate
func (g GateCustom) OutWire() uint64 {
	return g.Out
}

// Coefficient value of customized gate
func (g GateCustom) CoefValue() *big.Int {
	return g.Coef
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

// generate graphviz compatible DOT file for visualizing the circuit layout
func (rc *RootCircuit) Graphviz() string {
	var ret string
	// sanity check
	if len(rc.Circuits) != len(rc.Layers) {
		panic("Invalid Circuit: length of Circuits and Layers don't match")
	}

	// sort Circuit
	type Pair struct {
		Circuit *Circuit
		Layer   uint64
	}

	// Edges
	type Edge struct {
		source      string
		destination string
		coef        string
	}

	// zip circuits and their lables, then sort
	pairs := make([]Pair, len(rc.Circuits))
	for i := range rc.Circuits {
		pairs[i] = Pair{rc.Circuits[i], rc.Layers[i]}
	}
	sort.Slice(pairs, func(i, j int) bool {
		return pairs[i].Layer < pairs[j].Layer
	})

	// store edges and output them all in the end
	// key = source
	// value = destination
	var edges []Edge

	ret += fmt.Sprintln("digraph G{")
	ret += fmt.Sprintln("	rankdir=BT;")
	ret += fmt.Sprintln("	splines=false;")
	ret += fmt.Sprintln("	E_0[label=\"mul\" style=filled fillcolor=gold];")
	ret += fmt.Sprintln("	E_1[label=\"add\" style=filled fillcolor=lightskyblue];")
	ret += fmt.Sprintln("	E_2[label=\"const\" style=filled fillcolor=cornsilk];")
	ret += fmt.Sprintln("	E_4[label=\"custom\" style=filled fillcolor=plum];")

	for i, p := range pairs {
		circuit := p.Circuit
		layer := p.Layer

		// create input layer when i == 0
		if i == 0 {
			ret += fmt.Sprintln("	// input layer")
			ret += fmt.Sprintln("	subgraph cluster_0 {")
			ret += fmt.Sprintln("		label=\"Input Layer\";")

			// a dictionary of all nodes at the input layer
			// this doesn't include constant nodes
			nodes := make(map[uint64]bool)

			// We don't color the nodes for the input layer
			for _, gate := range circuit.Add {
				nodes[gate.In] = true
			}
			for _, gate := range circuit.Mul {
				nodes[gate.In0] = true
				nodes[gate.In1] = true
			}
			for _, gate := range circuit.Custom {
				for _, in := range gate.In {
					nodes[in] = true
				}
			}

			var nodeSlice []uint64
			for key := range nodes {
				nodeSlice = append(nodeSlice, key)
			}
			sort.Slice(nodeSlice, func(i, j int) bool {
				return nodeSlice[i] < nodeSlice[j]
			})

			for _, id := range nodeSlice {
				ret += fmt.Sprintf("		I_%d;\n", id)
			}
			// closing input layer
			ret += fmt.Sprintln("	}")
		}

		// draw nodes of the current layer
		ret += fmt.Sprintln("")
		ret += fmt.Sprintf("	//Layer %d \n", layer)
		ret += fmt.Sprintf("	subgraph cluster_%d {\n", layer+1)
		ret += fmt.Sprintf("		label=\"Layer %d\";\n", layer)

		// nodes stores mapping between out wire to a list of gates
		nodes := make(map[uint64][]Gate)

		// add add edges
		for j, gate := range circuit.Add {
			nodes[gate.Out] = append(nodes[gate.Out], gate)
			var src string
			if i == 0 {
				src = fmt.Sprintf("I_%d", gate.In)
			} else {
				src = fmt.Sprintf("S_%d_%d", pairs[i-1].Layer, gate.In)
			}
			// nodeId of current gate
			nodeId := fmt.Sprintf("Add_%d_%d", layer, j)
			// output node id here
			ret += fmt.Sprintf("		%s[label=\"* %v\" style=filled fillcolor=lightskyblue]; \n",
				nodeId, gate.Coef)
			// add two edges, one from input to this add node,
			// one from this node to the out
			edges = append(edges,
				Edge{src, nodeId, ""},
				Edge{nodeId, fmt.Sprintf("S_%d_%d", layer, gate.Out), ""})
		}
		// adding mul edges
		for j, gate := range circuit.Mul {
			nodes[gate.Out] = append(nodes[gate.Out], gate)
			var src0, src1 string
			if i == 0 {
				src0 = fmt.Sprintf("I_%d", gate.In0)
				src1 = fmt.Sprintf("I_%d", gate.In1)
			} else {
				src0 = fmt.Sprintf("S_%d_%d", pairs[i-1].Layer, gate.In0)
				src1 = fmt.Sprintf("S_%d_%d", pairs[i-1].Layer, gate.In1)
			}
			// node id of current gate
			nodeId := fmt.Sprintf("Mul_%d_%d", layer, j)
			// output current node
			ret += fmt.Sprintf("		%s[label=\"* %d\" style=filled fillcolor=gold]; \n",
				nodeId, gate.Coef)
			// add 3 edges, the edges to mul node, and mul node to output
			edges = append(
				edges,
				Edge{src0, nodeId, ""},
				Edge{src1, nodeId, ""},
				Edge{nodeId, fmt.Sprintf("S_%d_%d", layer, gate.Out), ""})
		}
		// add custom edges
		for j, gate := range circuit.Custom {
			nodes[gate.Out] = append(nodes[gate.Out], gate)

			var srcs []string
			if i == 0 {
				for _, in := range gate.In {
					srcs = append(srcs, fmt.Sprintf("I_%d", in))
				}
			} else {
				for _, in := range gate.In {
					srcs = append(srcs, fmt.Sprintf("S_%d_%d", pairs[i-1].Layer, in))
				}
			}
			// node id of current gate
			nodeId := fmt.Sprintf("Cus_%d_%d", layer, j)
			// output the current node
			ret += fmt.Sprintf("		%s[label=\"%v:*%d\" style=filled fillcolor=plum]", nodeId, gate.GateType, gate.Coef)
			// add edges from inputs to this node
			for _, src := range srcs {
				edges = append(
					edges,
					Edge{src, nodeId, ""})
			}
			// add edge from this node to output
			edges = append(edges,
				Edge{nodeId, fmt.Sprintf("S_%d_%d", layer, gate.Out), ""})

		}
		// add const edges
		for j, gate := range circuit.Cst {
			nodes[gate.Out] = append(nodes[gate.Out], gate)
			src := fmt.Sprintf("C_%d_%d", layer, j)
			ret += fmt.Sprintf("		%s[label=\"%v\" style=filled fillcolor=cornsilk];\n", src, gate.Coef)
			edges = append(edges, Edge{src, fmt.Sprintf("S_%d_%d", layer, gate.Out), ""})
		}

		var keys []uint64
		for key := range nodes {
			keys = append(keys, key)
		}
		sort.Slice(keys, func(i, j int) bool {
			return keys[i] < keys[j]
		})

		// output S nodes (output wires)
		for _, key := range keys {
			ret += fmt.Sprintf("		S_%d_%d;\n", layer, key)
		}

		// closing the node for current layer
		ret += fmt.Sprintln("	}")
	}

	// ploting all the edges
	for _, e := range edges {
		ret += fmt.Sprintf("	%s -> %s [label=\"%s\"];\n", e.source, e.destination, e.coef)
	}
	ret += fmt.Sprintln("}")
	return ret
}
