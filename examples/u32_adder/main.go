package main

import (
	"os"

	"github.com/PolyhedraZK/ExpanderCompilerCollection"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/field/gf2"
	u32adder "github.com/PolyhedraZK/ExpanderCompilerCollection/u32_adder"
	"github.com/consensys/gnark/frontend"
)

// U32AddCircuit defines a 32-bit adder circuit
type U32AddCircuit struct {
	A        [32]frontend.Variable `gnark:",public"`
	B        [32]frontend.Variable `gnark:",public"`
	CarryIns [8]frontend.Variable  `gnark:",public"`
}

func (c *U32AddCircuit) Define(api frontend.API) error {
	u32adder.BrentKungAdder32Bits(api, c.A[:], c.B[:], c.CarryIns[:])

	return nil
}

func main() {
	var circuit U32AddCircuit
	cr, _ := ExpanderCompilerCollection.Compile(gf2.ScalarField, &circuit)
	c := cr.GetLayeredCircuit()
	os.WriteFile("circuit_u32.txt", c.Serialize(), 0o644)
}
