package main

import (
	"fmt"
	"os"

	poseidonM31 "github.com/PolyhedraZK/ExpanderCompilerCollection/circuit-std-go/poseidon-m31"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo/field/m31"
	ecc_test "github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo/test"
	"github.com/consensys/gnark/frontend"
)

func main() {
	fmt.Println("Building M31 circuits")
	M31CircuitBuild()
}

const NumRepeat = 120

type MockPoseidonM31Circuit struct {
	State  [NumRepeat][16]frontend.Variable
	Digest [NumRepeat]frontend.Variable
}

func (c *MockPoseidonM31Circuit) Define(api frontend.API) (err error) {
	// Define the circuit
	for i := 0; i < NumRepeat; i++ {
		digest := poseidonM31.PoseidonM31x16Permutate(api, c.State[i][:])
		api.AssertIsEqual(digest[0], c.Digest[i])
	}

	return
}

func M31CircuitBuild() {

	var stateVars [NumRepeat][16]frontend.Variable
	var outputVars [NumRepeat]frontend.Variable

	for i := 0; i < NumRepeat; i++ {

		for j := 0; j < 8; j++ {
			stateVars[i][j] = frontend.Variable(0)
		}

		for j := 8; j < 16; j++ {
			stateVars[i][j] = frontend.Variable(114514)
		}

		outputVars[i] = frontend.Variable(1021105124)

	}

	assignment := &MockPoseidonM31Circuit{
		State:  stateVars,
		Digest: outputVars,
	}

	// Ecc test
	circuit, err := ecgo.Compile(m31.ScalarField, &MockPoseidonM31Circuit{
		State:  stateVars,
		Digest: outputVars,
	}, frontend.WithCompressThreshold(32))
	if err != nil {
		panic(err)
	}

	layered_circuit := circuit.GetLayeredCircuit()
	if err = os.WriteFile("poseidon_120_circuit_m31.txt", layered_circuit.Serialize(), 0o644); err != nil {
		panic(err)
	}
	inputSolver := circuit.GetInputSolver()
	witness, err := inputSolver.SolveInputAuto(assignment)
	if err != nil {
		panic(err)
	}

	if err = os.WriteFile("poseidon_120_witness_m31.txt", witness.Serialize(), 0o644); err != nil {
		panic(err)
	}
	if !ecc_test.CheckCircuit(layered_circuit, witness) {
		panic("verification failed")
	}

}
