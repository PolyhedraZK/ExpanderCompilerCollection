package logup

import (
	"os"
	"testing"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/frontend"

	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo/test"
)

func TestLogupCircuit(t *testing.T) {
	N_TABLE_ROWS := uint(1024 * 4)
	N_QUERIES := uint(1024 * 4)
	COLUMN_SIZE := uint(2)

	circuit, err := ecgo.Compile(ecc.BN254.ScalarField(), NewRandomCircuit(N_TABLE_ROWS, N_QUERIES, COLUMN_SIZE, false))
	if err != nil {
		panic(err.Error())
	}

	c := circuit.GetLayeredCircuit()
	os.WriteFile("circuit.txt", c.Serialize(), 0o644)

	assignment := NewRandomCircuit(N_TABLE_ROWS, N_QUERIES, COLUMN_SIZE, true)
	inputSolver := circuit.GetInputSolver()
	witness, err := inputSolver.SolveInput(assignment, 0)
	if err != nil {
		panic(err.Error())
	}

	if !test.CheckCircuit(c, witness) {
		panic("Circuit not satisfied")
	}

	// os.WriteFile("inputsolver.txt", inputSolver.Serialize(), 0o644)
	os.WriteFile("witness.txt", witness.Serialize(), 0o644)
}

type LogUpRuntimeCircuit struct {
	Test [1]frontend.Variable
}

func (circuit *LogUpRuntimeCircuit) Define(api frontend.API) error {
	// range proof
	Reset()
	keys := make([]frontend.Variable, 64)
	values := make([][]frontend.Variable, 64)
	for i := 0; i < 64; i++ {
		keys[i] = frontend.Variable(i)
		values[i] = []frontend.Variable{i + 1888}
	}
	NewTable(keys, values)
	for i := 0; i < 64; i++ {
		newI := (i + 23) % 64
		Query(frontend.Variable(newI), []frontend.Variable{newI + 1888})
	}
	FinalCheck(api, ColumnCombineOption)
	api.AssertIsEqual(frontend.Variable(1), circuit.Test[0])
	return nil
}
func TestLogupRuntimeCircuit(t *testing.T) {
	circuit, err := ecgo.Compile(ecc.BN254.ScalarField(), &LogUpRuntimeCircuit{})
	if err != nil {
		panic(err.Error())
	}

	c := circuit.GetLayeredCircuit()
	os.WriteFile("circuit.txt", c.Serialize(), 0o644)

	assignment := &LogUpRuntimeCircuit{Test: [1]frontend.Variable{1}}
	inputSolver := circuit.GetInputSolver()
	witness, err := inputSolver.SolveInput(assignment, 0)
	if err != nil {
		panic(err.Error())
	}

	if !test.CheckCircuit(c, witness) {
		panic("Circuit not satisfied")
	}

	// os.WriteFile("inputsolver.txt", inputSolver.Serialize(), 0o644)
	os.WriteFile("witness.txt", witness.Serialize(), 0o644)
}

type LogUpRangeProofCircuit struct {
	Test [1]frontend.Variable
}

func (circuit *LogUpRangeProofCircuit) Define(api frontend.API) error {
	// range proof
	Reset()
	NewRangeProof(8)
	for i := 1; i < 12; i++ {
		for j := (1 << (i - 1)); j < (1 << i); j++ {
			RangeProof(api, frontend.Variable(j), i)
		}
	}
	FinalCheck(api, ColumnCombineOption)
	return nil
}
func TestLogupRangeProofCircuit(t *testing.T) {
	circuit, err := ecgo.Compile(ecc.BN254.ScalarField(), &LogUpRangeProofCircuit{})
	if err != nil {
		panic(err.Error())
	}

	c := circuit.GetLayeredCircuit()
	os.WriteFile("circuit.txt", c.Serialize(), 0o644)

	assignment := &LogUpRangeProofCircuit{Test: [1]frontend.Variable{1}}
	inputSolver := circuit.GetInputSolver()
	witness, err := inputSolver.SolveInput(assignment, 0)
	if err != nil {
		panic(err.Error())
	}

	if !test.CheckCircuit(c, witness) {
		panic("Circuit not satisfied")
	}

	// os.WriteFile("inputsolver.txt", inputSolver.Serialize(), 0o644)
	os.WriteFile("witness.txt", witness.Serialize(), 0o644)
}
