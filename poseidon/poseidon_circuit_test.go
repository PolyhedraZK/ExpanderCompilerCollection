package poseidon

import (
	"fmt"
	"testing"

	"github.com/PolyhedraZK/ExpanderCompilerCollection/field/m31"
	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/r1cs"
	"github.com/consensys/gnark/test"
)

type MockPoseidonCircuit struct {
	State             []frontend.Variable      `gnark:",public"`
	InternalStateVars PoseidonInternalStateVar `gnark:",public"`
	Output            frontend.Variable        `gnark:",public"`
}

func (c *MockPoseidonCircuit) Define(api frontend.API) (err error) {
	// Define the circuit
	param := NewPoseidonParams()
	engine := m31.Field{}
	t := PoseidonCircuit(api, engine, param, c.State, c.InternalStateVars, false)
	api.AssertIsEqual(t, c.Output)

	return
}

func TestPoseidonCircuit(t *testing.T) {
	assert := test.NewAssert(t)

	param := NewPoseidonParams()

	state := make([]constraint.Element, 16)
	stateVar := make([]frontend.Variable, 16)
	var internalStateVars PoseidonInternalStateVar

	for i := 0; i < 16; i++ {
		state[i] = constraint.Element{uint64(i)}
		stateVar[i] = frontend.Variable(uint64(i))
	}
	internalState, output := PoseidonM31(param, state)
	outputVar := frontend.Variable(output[0])

	fmt.Println("internal state", internalState)

	for j := 0; j < 16; j++ {
		internalStateVars.AfterHalfFullRound[j] = frontend.Variable(internalState.AfterHalfFullRound[j][0])
		internalStateVars.AfterHalfPartialRound[j] = frontend.Variable(internalState.AfterHalfPartialRound[j][0])
		internalStateVars.AfterPartialRound[j] = frontend.Variable(internalState.AfterPartialRound[j][0])
	}

	c := MockPoseidonCircuit{
		stateVar,
		internalStateVars,
		outputVar,
	}

	w, _ := frontend.NewWitness(&c, m31.ScalarField)
	fmt.Println("witness", w)

	err := test.IsSolved(&c, &c, m31.ScalarField)
	assert.NoError(err)

	r1cs, err := frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, &c)
	assert.NoError(err)
	fmt.Println("num constraints:", r1cs.GetNbConstraints())
	fmt.Println("num coefficients:", r1cs.GetNbCoefficients())
	i, p, s := r1cs.GetNbVariables()
	fmt.Println("num variables:", i, p, s)
}
