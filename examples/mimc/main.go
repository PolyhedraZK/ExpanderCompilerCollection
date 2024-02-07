package main

import (
	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/hash/mimc"

	gkr "github.com/Zklib/gkr-compiler"
	"github.com/Zklib/gkr-compiler/checker"
)

type Circuit struct {
	PreImage frontend.Variable
	Hash     frontend.Variable
}

// Define declares the circuit's constraints
func (circuit *Circuit) Define(api frontend.API) error {
	// hash function
	mimc, _ := mimc.NewMiMC(api)

	// specify constraints
	// mimc(preImage) == hash
	mimc.Write(circuit.PreImage)
	api.AssertIsEqual(circuit.Hash, mimc.Sum())

	return nil
}

func main() {
	circuit, err := gkr.Compile(ecc.BN254.ScalarField(), &Circuit{}, true)
	if err != nil {
		panic(err)
	}

	c := circuit.GetLayeredCircuit()

	assignment := &Circuit{
		PreImage: "16130099170765464552823636852555369511329944820189892919423002775646948828469",
		Hash:     "12886436712380113721405259596386800092738845035233065858332878701083870690753",
	}
	witness := circuit.GetWitness(assignment)

	if !checker.CheckCircuit(c, witness) {
		panic("error")
	}
}
