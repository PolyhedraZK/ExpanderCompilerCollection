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
