package main

import (
	"fmt"
	"math/rand"

	gkr "github.com/Zklib/gkr-compiler"
	"github.com/Zklib/gkr-compiler/builder"
	"github.com/Zklib/gkr-compiler/checker"
	"github.com/consensys/gnark/frontend"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/liyue201/gnark-crypto/ecc"
	"github.com/zkbridge-testnet/circuits/common/keccak"
)

const NHashes = 100

type keccak256Circuit struct {
	M    [NHashes][100]frontend.Variable
	Hash [NHashes][32]frontend.Variable
}

func (t *keccak256Circuit) Define(api frontend.API) error {
	f := builder.MemorizedFunc(keccak.Keccak256)

	for j := 0; j < NHashes; j++ {
		hash := f(api, t.M[j][:])
		for i := 0; i < len(hash); i++ {
			api.AssertIsEqual(hash[i], t.Hash[j][i])
		}
	}
	//api.Println(hash...)
	return nil
}

func main() {
	var circuit keccak256Circuit
	hash := make([]common.Hash, 0)
	for j := 0; j < NHashes; j++ {
		m := make([]byte, len(circuit.M[0])-1)
		for i := 0; i < len(m); i++ {
			m[i] = byte(rand.Int() % 256)
			circuit.M[j][i] = m[i]
		}
		circuit.M[j][len(circuit.M[0])-1] = frontend.Variable(-1)
		hash = append(hash, crypto.Keccak256Hash(m))
	}
	var assignment keccak256Circuit
	for j := 0; j < NHashes; j++ {
		for i := 0; i < len(assignment.M[0]); i++ {
			assignment.M[j][i] = circuit.M[j][i]
		}
		for i := 0; i < len(assignment.Hash[0]); i++ {
			assignment.Hash[j][i] = hash[j][i]
		}
	}

	cr, _ := gkr.Compile(ecc.BN254.ScalarField(), &circuit, true)
	//cr.Print()
	_ = cr

	c := cr.GetLayeredCircuit()
	fmt.Printf("ok1\n")

	witness := cr.GetWitness(&assignment)
	fmt.Printf("ok2\n")

	if !checker.CheckCircuit(c, witness) {
		panic("error")
	}
}
