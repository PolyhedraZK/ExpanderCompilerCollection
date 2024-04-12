package main

import (
	"math/rand"
	"os"

	gkr "github.com/Zklib/gkr-compiler"
	"github.com/Zklib/gkr-compiler/builder"
	"github.com/Zklib/gkr-compiler/field/m31"
	"github.com/Zklib/gkr-compiler/test"
	"github.com/consensys/gnark/frontend"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/zkbridge-testnet/circuits/common/keccak"
)

// Due to some features of the compiler, it has the best space efficiency when n=2^k-1
const NHashes = 127

type keccak256Circuit struct {
	M    [NHashes][100]frontend.Variable
	Hash [NHashes][32]frontend.Variable
}

func checkHash(api frontend.API, input []frontend.Variable, output []frontend.Variable) {
	hash := keccak.Keccak256(api, input)
	for i := 0; i < len(hash); i++ {
		api.AssertIsEqual(hash[i], output[i])
	}
}

func (t *keccak256Circuit) Define(api frontend.API) error {
	check := builder.MemorizedVoidFunc(checkHash)

	for j := 0; j < NHashes; j++ {
		check(api, t.M[j][:], t.Hash[j][:])
	}

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

	cr, err := gkr.Compile(m31.ScalarField, &circuit)
	if err != nil {
		panic(err)
	}
	//cr.Print()
	_ = cr

	c := cr.GetLayeredCircuit()
	//c.Print()

	inputSolver := cr.GetInputSolver()
	witness, err := inputSolver.SolveInput(&assignment, 8)
	if err != nil {
		panic(err)
	}

	if !test.CheckCircuit(c, witness) {
		panic("error")
	}

	os.WriteFile("inputsolver.txt", inputSolver.Serialize(), 0o644)
	os.WriteFile("circuit.txt", c.Serialize(), 0o644)
}
