package u32adder

import (
	"fmt"
	"os"
	"testing"

	"github.com/PolyhedraZK/ExpanderCompilerCollection"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/field/gf2"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/test"
	"github.com/consensys/gnark/frontend"
)

type U4BKAddCircuit struct {
	A0, A1, A2, A3 frontend.Variable
	B0, B1, B2, B3 frontend.Variable
	CarryIn        frontend.Variable
	S0, S1, S2, S3 frontend.Variable
	CarryOut       frontend.Variable
}

func (c *U4BKAddCircuit) Define(api frontend.API) error {
	a := []frontend.Variable{c.A0, c.A1, c.A2, c.A3}
	b := []frontend.Variable{c.B0, c.B1, c.B2, c.B3}

	sum, carryOut := BrentKungAdder4Bits(api, a, b, c.CarryIn)

	api.AssertIsEqual(c.S0, sum[0])
	api.AssertIsEqual(c.S1, sum[1])
	api.AssertIsEqual(c.S2, sum[2])
	api.AssertIsEqual(c.S3, sum[3])
	api.AssertIsEqual(c.CarryOut, carryOut)

	return nil
}
func TestBKU4Add(t *testing.T) {
	var circuit U4BKAddCircuit

	cr, _ := ExpanderCompilerCollection.Compile(gf2.ScalarField, &circuit)
	c := cr.GetLayeredCircuit()
	os.WriteFile("circuit_u4.txt", c.Serialize(), 0o644)

	boolToUint64 := func(b bool) uint64 {
		if b {
			return 1
		}
		return 0
	}

	for a := 0; a < 16; a++ {
		for b := 0; b < 16; b++ {
			for carryIn := 0; carryIn < 2; carryIn++ {
				t.Run(fmt.Sprintf("a=%d_b=%d_carryIn=%d", a, b, carryIn), func(t *testing.T) {
					aSlice := intToBoolSlice(a)
					bSlice := intToBoolSlice(b)
					carryInBool := carryIn == 1

					sum, carryOut := PlainBrentKungAdder4Bits(aSlice, bSlice, carryInBool)

					// Calculate expected result
					expectedSum := (a + b + carryIn) & 0xF
					expectedCarryOut := (a + b + carryIn) > 0xF

					// Convert uint8 sum back to []bool
					sumBools := intToBoolSlice(int(sum))

					if boolSliceToInt(sumBools) != expectedSum || carryOut != expectedCarryOut {
						t.Errorf("Test case failed for a=%d, b=%d, carryIn=%d. Got sum %d, carryOut %v, expected sum %d, carryOut %v",
							a, b, carryIn, boolSliceToInt(sumBools), carryOut, expectedSum, expectedCarryOut)
					}

					// Test the circuit
					circuit := U4BKAddCircuit{
						A0:       frontend.Variable(boolToUint64(aSlice[0])),
						A1:       frontend.Variable(boolToUint64(aSlice[1])),
						A2:       frontend.Variable(boolToUint64(aSlice[2])),
						A3:       frontend.Variable(boolToUint64(aSlice[3])),
						B0:       frontend.Variable(boolToUint64(bSlice[0])),
						B1:       frontend.Variable(boolToUint64(bSlice[1])),
						B2:       frontend.Variable(boolToUint64(bSlice[2])),
						B3:       frontend.Variable(boolToUint64(bSlice[3])),
						CarryIn:  frontend.Variable(boolToUint64(carryInBool)),
						S0:       frontend.Variable(boolToUint64(sumBools[0])),
						S1:       frontend.Variable(boolToUint64(sumBools[1])),
						S2:       frontend.Variable(boolToUint64(sumBools[2])),
						S3:       frontend.Variable(boolToUint64(sumBools[3])),
						CarryOut: frontend.Variable(boolToUint64(carryOut)),
					}

					wit, err := cr.GetInputSolver().SolveInput(&circuit, 8)
					if err != nil {
						t.Fatalf("Failed to solve input: %v", err)
					}
					if !test.CheckCircuit(c, wit) {
						t.Errorf("Circuit check failed for a=%d, b=%d, carryIn=%d", a, b, carryIn)
					}
				})
			}
		}
	}
}

type NaiveAddCircult struct {
	A0, A1, A2, A3 frontend.Variable
	B0, B1, B2, B3 frontend.Variable
	CarryIn        frontend.Variable
	S0, S1, S2, S3 frontend.Variable
	CarryOut       frontend.Variable
}

func (c *NaiveAddCircult) Define(api frontend.API) error {
	a := []frontend.Variable{c.A0, c.A1, c.A2, c.A3}
	b := []frontend.Variable{c.B0, c.B1, c.B2, c.B3}

	sum, carryOut := NaiveAdder4Bits(api, a, b, c.CarryIn)

	api.AssertIsEqual(c.S0, sum[0])
	api.AssertIsEqual(c.S1, sum[1])
	api.AssertIsEqual(c.S2, sum[2])
	api.AssertIsEqual(c.S3, sum[3])
	api.AssertIsEqual(c.CarryOut, carryOut)

	return nil
}

func TestNaiveU4Add(t *testing.T) {
	var circuit NaiveAddCircult

	cr, _ := ExpanderCompilerCollection.Compile(gf2.ScalarField, &circuit)
	c := cr.GetLayeredCircuit()
	os.WriteFile("circuit_u4.txt", c.Serialize(), 0o644)

	boolToUint64 := func(b bool) uint64 {
		if b {
			return 1
		}
		return 0
	}

	for a := 0; a < 16; a++ {
		for b := 0; b < 16; b++ {
			for carryIn := 0; carryIn < 2; carryIn++ {
				t.Run(fmt.Sprintf("a=%d_b=%d_carryIn=%d", a, b, carryIn), func(t *testing.T) {
					aSlice := intToBoolSlice(a)
					bSlice := intToBoolSlice(b)
					carryInBool := carryIn == 1

					sum, carryOut := PlainBrentKungAdder4Bits(aSlice, bSlice, carryInBool)

					// Calculate expected result
					expectedSum := (a + b + carryIn) & 0xF
					expectedCarryOut := (a + b + carryIn) > 0xF

					// Convert uint8 sum back to []bool
					sumBools := intToBoolSlice(int(sum))

					if boolSliceToInt(sumBools) != expectedSum || carryOut != expectedCarryOut {
						t.Errorf("Test case failed for a=%d, b=%d, carryIn=%d. Got sum %d, carryOut %v, expected sum %d, carryOut %v",
							a, b, carryIn, boolSliceToInt(sumBools), carryOut, expectedSum, expectedCarryOut)
					}

					// Test the circuit
					circuit := NaiveAddCircult{
						A0:       frontend.Variable(boolToUint64(aSlice[0])),
						A1:       frontend.Variable(boolToUint64(aSlice[1])),
						A2:       frontend.Variable(boolToUint64(aSlice[2])),
						A3:       frontend.Variable(boolToUint64(aSlice[3])),
						B0:       frontend.Variable(boolToUint64(bSlice[0])),
						B1:       frontend.Variable(boolToUint64(bSlice[1])),
						B2:       frontend.Variable(boolToUint64(bSlice[2])),
						B3:       frontend.Variable(boolToUint64(bSlice[3])),
						CarryIn:  frontend.Variable(boolToUint64(carryInBool)),
						S0:       frontend.Variable(boolToUint64(sumBools[0])),
						S1:       frontend.Variable(boolToUint64(sumBools[1])),
						S2:       frontend.Variable(boolToUint64(sumBools[2])),
						S3:       frontend.Variable(boolToUint64(sumBools[3])),
						CarryOut: frontend.Variable(boolToUint64(carryOut)),
					}

					wit, err := cr.GetInputSolver().SolveInput(&circuit, 8)
					if err != nil {
						t.Fatalf("Failed to solve input: %v", err)
					}
					if !test.CheckCircuit(c, wit) {
						t.Errorf("Circuit check failed for a=%d, b=%d, carryIn=%d", a, b, carryIn)
					}
				})
			}
		}
	}
}

func TestCost(t *testing.T) {
	var circuit1 U4BKAddCircuit

	cr1, _ := ExpanderCompilerCollection.Compile(gf2.ScalarField, &circuit1)
	cr1.GetLayeredCircuit()

	var circuit2 NaiveAddCircult

	cr2, _ := ExpanderCompilerCollection.Compile(gf2.ScalarField, &circuit2)
	cr2.GetLayeredCircuit()
}
