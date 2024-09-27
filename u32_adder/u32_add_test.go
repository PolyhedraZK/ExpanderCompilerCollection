package u32adder

import (
	"math/rand"
	"os"
	"testing"

	"github.com/PolyhedraZK/ExpanderCompilerCollection"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/field/gf2"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/test"
	"github.com/consensys/gnark/frontend"
)

// U32AddCircuit defines a 32-bit adder circuit
type U32AddCircuit struct {
	A        [32]frontend.Variable `gnark:",public"`
	B        [32]frontend.Variable `gnark:",public"`
	CarryIn  frontend.Variable     `gnark:",public"`
	Sum      [32]frontend.Variable `gnark:",public"`
	CarryOut frontend.Variable     `gnark:",public"`
}

func (c *U32AddCircuit) Define(api frontend.API) error {
	sum, carryOut := BrentKungAdder32Bits(api, c.A[:], c.B[:], c.CarryIn)

	for i := 0; i < 32; i++ {
		api.AssertIsEqual(c.Sum[i], sum[i])
	}
	api.AssertIsEqual(c.CarryOut, carryOut)

	return nil
}

func TestU32AddCircuit(t *testing.T) {
	var circuit U32AddCircuit
	cr, _ := ExpanderCompilerCollection.Compile(gf2.ScalarField, &circuit)
	c := cr.GetLayeredCircuit()
	os.WriteFile("circuit_u32.txt", c.Serialize(), 0o644)

	testCases := []struct {
		name     string
		a        uint32
		b        uint32
		carryIn  uint32
		sum      uint32
		carryOut uint32
	}{
		{"Simple addition", 5, 7, 0, 12, 0},
		{"Addition with carries", 0xFFFFFFFF, 1, 0, 0, 1},
		{"Max value addition", 0xFFFFFFFF, 0xFFFFFFFF, 1, 0xFFFFFFFF, 1},
		{"Zero addition", 0, 0, 0, 0, 0},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			circuit := &U32AddCircuit{
				A:        uintToBits(tc.a),
				B:        uintToBits(tc.b),
				CarryIn:  frontend.Variable(tc.carryIn),
				Sum:      uintToBits(tc.sum),
				CarryOut: frontend.Variable(tc.carryOut),
			}

			wit, err := cr.GetInputSolver().SolveInput(circuit, 8)
			if err != nil {
				t.Fatalf("Failed to solve input: %v", err)
			}
			if !test.CheckCircuit(c, wit) {
				t.Errorf("Circuit check failed for a=%d, b=%d, carryIns=%v", tc.a, tc.b, tc.carryIn)
			}
		})
	}
}

func uintToBits(n uint32) [32]frontend.Variable {
	var bits [32]frontend.Variable
	for i := 0; i < 32; i++ {
		if n&(1<<i) != 0 {
			bits[i] = 1
		} else {
			bits[i] = 0
		}
	}
	return bits
}

// func uintArrayToVars(arr [8]uint32) [8]frontend.Variable {
// 	var vars [8]frontend.Variable
// 	for i, v := range arr {
// 		vars[i] = frontend.Variable(v)
// 	}
// 	return vars
// }

func TestU32AddCircuitRandom(t *testing.T) {
	var circuit U32AddCircuit
	cr, _ := ExpanderCompilerCollection.Compile(gf2.ScalarField, &circuit)
	c := cr.GetLayeredCircuit()
	os.WriteFile("circuit_u32.txt", c.Serialize(), 0o644)

	for i := 0; i < 100; i++ {
		a := rand.Uint32()
		b := rand.Uint32()

		var sum uint64
		carryIn := rand.Uint32() % 2

		sum = uint64(a) + uint64(b) + uint64(carryIn)
		sum32 := uint32(sum)
		carryOut := uint32(sum >> 32)

		circuit := &U32AddCircuit{
			A:        uintToBits(a),
			B:        uintToBits(b),
			CarryIn:  frontend.Variable(carryIn),
			Sum:      uintToBits(sum32),
			CarryOut: frontend.Variable(carryOut),
		}

		wit, err := cr.GetInputSolver().SolveInput(circuit, 8)
		if err != nil {
			t.Fatalf("Failed to solve input: %v", err)
		}
		if !test.CheckCircuit(c, wit) {
			t.Errorf("Circuit check failed for a= %d, b= %d, carryIns=%v, sum= %d, carryOut= %d", a, b, carryIn, sum32, carryOut)
		}
	}
	t.Log("All random tests passed successfully")
}
