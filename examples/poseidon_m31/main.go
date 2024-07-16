package main

import (
	"fmt"
	"os"

	"github.com/PolyhedraZK/ExpanderCompilerCollection"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/field/m31"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/poseidon"
	ecc_test "github.com/PolyhedraZK/ExpanderCompilerCollection/test"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/frontend"
)

func main() {
	fmt.Println("Building M31 circuits")
	M31CircuitBuild()
}

const NumRepeat = 100

type MockPoseidonM31Circuit struct {
	State         [NumRepeat][16]frontend.Variable
	InternalState [NumRepeat]poseidon.PoseidonInternalStateVar
	Digest        [NumRepeat]frontend.Variable `gnark:",public"`
}

func (c *MockPoseidonM31Circuit) Define(api frontend.API) (err error) {
	// Define the circuit
	param := poseidon.NewPoseidonParams()
	engine := m31.Field{}
	for i := 0; i < NumRepeat; i++ {
		digest := poseidon.PoseidonCircuit(api, engine, param, c.State[i][:], c.InternalState[i], true)
		api.AssertIsEqual(digest, c.Digest[i])
	}

	return
}

func M31CircuitBuild() {

	param := poseidon.NewPoseidonParams()

	var states [NumRepeat][16]constraint.Element
	var stateVars [NumRepeat][16]frontend.Variable
	var outputVars [NumRepeat]frontend.Variable
	var internalStateVars [NumRepeat]poseidon.PoseidonInternalStateVar

	for i := 0; i < NumRepeat; i++ {
		for j := 0; j < 16; j++ {
			states[i][j] = constraint.Element{uint64(i)}
			stateVars[i][j] = frontend.Variable(uint64(i))
		}
		internalState, output := poseidon.PoseidonM31(param, states[i][:])
		outputVars[i] = frontend.Variable(output[0])
		for j := 0; j < 16; j++ {
			internalStateVars[i].AfterHalfFullRound[j] = frontend.Variable(internalState.AfterHalfFullRound[j][0])
			internalStateVars[i].AfterHalfPartialRound[j] = frontend.Variable(internalState.AfterHalfPartialRound[j][0])
			internalStateVars[i].AfterPartialRound[j] = frontend.Variable(internalState.AfterPartialRound[j][0])
		}
	}

	assignment := &MockPoseidonM31Circuit{
		State:         stateVars,
		InternalState: internalStateVars,
		Digest:        outputVars,
	}

	// Gnark test disabled as it does not support randomness
	// err := test.IsSolved(&MockPoseidonCircuit{}, assignment, m31.ScalarField)
	// if err != nil {
	// 	panic(err)
	// }
	// fmt.Println("Gnark test passed")

	// Ecc test
	circuit, err := ExpanderCompilerCollection.Compile(m31.ScalarField, &MockPoseidonM31Circuit{}, frontend.WithCompressThreshold(4))
	if err != nil {
		panic(err)
	}

	layered_circuit := circuit.GetLayeredCircuit()
	// circuit.GetCircuitIr().Print()
	err = os.WriteFile("circuit.txt", layered_circuit.Serialize(), 0o644)
	if err != nil {
		panic(err)
	}
	inputSolver := circuit.GetInputSolver()
	witness, err := inputSolver.SolveInputAuto(assignment)
	if err != nil {
		panic(err)
	}
	err = os.WriteFile("witness.txt", witness.Serialize(), 0o644)
	if err != nil {
		panic(err)
	}
	if !ecc_test.CheckCircuit(layered_circuit, witness) {
		panic("verification failed")
	}

}
