package main

import (
	"math/rand"
	"os"
	"time"

	"github.com/Zklib/gkr-compiler/gkr"
	"github.com/consensys/gnark/frontend"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/liyue201/gnark-crypto/ecc"
	"github.com/zkbridge-testnet/circuits/common/keccak"
)

type keccak256Circuit struct {
	M    [100]frontend.Variable `gnark:",public"`
	Hash [32]frontend.Variable  `gnark:",public"`
}

func (t *keccak256Circuit) Define(api frontend.API) error {

	hash := keccak.Keccak256(api, t.M[:])
	for i := 0; i < len(hash); i++ {
		api.AssertIsEqual(hash[i], t.Hash[i])
	}
	//api.Println(hash...)
	return nil
}

func main() {
	var circuit keccak256Circuit
	rand.Seed(time.Now().Unix())
	m := make([]byte, len(circuit.M)-1)
	for i := 0; i < len(m); i++ {
		m[i] = byte(rand.Int() % 256)
		circuit.M[i] = m[i]
	}
	circuit.M[len(circuit.M)-1] = frontend.Variable(-1)
	hash := crypto.Keccak256Hash(m)
	var assignment keccak256Circuit
	for i := 0; i < len(assignment.M); i++ {
		assignment.M[i] = circuit.M[i]
	}
	for i := 0; i < len(assignment.Hash); i++ {
		assignment.Hash[i] = hash[i]
	}

	cr, _ := gkr.Compile(ecc.BN254.ScalarField(), &circuit, true)
	//cr.Print()
	_ = cr

	c := cr.GetLayeredCircuit()
	os.WriteFile("circuit.txt", c.Serialize(), 0o644)

	witness := cr.GetWitness(&assignment)
	os.WriteFile("witness.txt", witness.Serialize(), 0o644)
}
