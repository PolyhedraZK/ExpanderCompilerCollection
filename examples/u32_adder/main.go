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
	A       [32]frontend.Variable `gnark:",public"`
	B       [32]frontend.Variable `gnark:",public"`
	CarryIn frontend.Variable     `gnark:",public"`
	Sum     [32]frontend.Variable `gnark:",public"`
	// CarryOut frontend.Variable     `gnark:",public"`
}

func (c *U32AddCircuit) Define(api frontend.API) error {
	sum, _ := u32adder.BrentKungAdder32Bits(api, c.A[:], c.B[:], c.CarryIn)
	for i := 0; i < 32; i++ {
		api.AssertIsEqual(c.Sum[i], sum[i])
	}
	// api.AssertIsEqual(c.CarryOut, carryOut)
	return nil
}

func main() {
	var circuit U32AddCircuit
	cr, _ := ExpanderCompilerCollection.Compile(gf2.ScalarField, &circuit)
	c := cr.GetLayeredCircuit()
	os.WriteFile("circuit_u32.txt", c.Serialize(), 0o644)
}
