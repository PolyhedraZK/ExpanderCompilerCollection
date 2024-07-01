package main

import (
	"os"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/frontend"
	"github.com/zkbridge-testnet/circuits/eth2/validator/plookup"

	gkr "github.com/Zklib/gkr-compiler"
	"github.com/Zklib/gkr-compiler/test"
)

func getAssignment() frontend.Circuit {
	var key [plookup.LookupTableSize]frontend.Variable
	var value [plookup.LookupTableSize]frontend.Variable
	var lookupKey [plookup.LookupSize]frontend.Variable
	var lookupValue [plookup.LookupSize]frontend.Variable
	for i := 0; i < plookup.LookupTableSize; i++ {
		key[i] = frontend.Variable(i)
		value[i] = frontend.Variable(i * 1256)
	}
	//randon := 132441223
	for i := 0; i < plookup.LookupSize; i++ {
		index := (i) % plookup.LookupTableSize
		lookupKey[i] = key[index]
		lookupValue[i] = value[index]
	}
	assignment := new(plookup.LookupCircuit)
	assignment.Key = key
	assignment.Value = value
	assignment.LookupKey = lookupKey
	assignment.LookupValue = lookupValue
	assignment.R = frontend.Variable(plookup.LookupSize)
	return assignment
}
func main() {
	circuit, err := gkr.Compile(ecc.BN254.ScalarField(), &plookup.LookupCircuit{})
	if err != nil {
		panic(err)
	}

	c := circuit.GetLayeredCircuit()
	os.WriteFile("circuit.txt", c.Serialize(), 0o644)

	assignment := getAssignment()

	inputSolver := circuit.GetInputSolver()
	witness, err := inputSolver.SolveInput(assignment, 8)
	if err != nil {
		panic(err)
	}

	if !test.CheckCircuit(c, witness) {
		panic("error")
	}

	os.WriteFile("inputsolver.txt", inputSolver.Serialize(), 0o644)
	os.WriteFile("circuit.txt", c.Serialize(), 0o644)
	os.WriteFile("witness.txt", witness.Serialize(), 0o644)
}
