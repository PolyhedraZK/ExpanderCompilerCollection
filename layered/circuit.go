package layered

import (
	"fmt"
	"math/big"
)

type RootCircuit struct {
	Circuits []*Circuit
	Layers   []uint64
	Field    *big.Int
}

type Circuit struct {
	InputLen    uint64
	OutputLen   uint64
	Flatten     bool // TODO: is this useful?
	SubCircuits []SubCircuit
	Mul         []GateMul
	Add         []GateAdd
	Cst         []GateCst
}

type SubCircuit struct {
	Id          uint64
	Allocations []Allocation
}

type Allocation struct {
	InputOffset  uint64
	OutputOffset uint64
}

type GateMul struct {
	In0  uint64
	In1  uint64
	Out  uint64
	Coef *big.Int
}

type GateAdd struct {
	In   uint64
	Out  uint64
	Coef *big.Int
}

type GateCst struct {
	Out  uint64
	Coef *big.Int
}

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
}

func (rc *RootCircuit) Print() {
	for i, c := range rc.Circuits {
		fmt.Printf("Circuit %d: ", i)
		c.Print()
		fmt.Printf("================================\n")
	}
	fmt.Printf("Layers: %v\n", rc.Layers)
}

// Validate checks if the circuit is valid
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
	return nil
}
