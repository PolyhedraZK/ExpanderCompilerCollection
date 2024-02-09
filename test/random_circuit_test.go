package test

import (
	"testing"

	"github.com/Zklib/gkr-compiler"
	"github.com/Zklib/gkr-compiler/checker"
	"github.com/consensys/gnark-crypto/ecc"
)

func testRandomCircuit(t *testing.T, conf *randomCircuitConfig, seedL int, seedR int) {
	for seed := seedL; seed <= seedR; seed++ {
		conf.seed = seed
		rcg := newRandomCircuitGenerator(conf)
		circuit := rcg.circuit()
		c, err := gkr.Compile(ecc.BN254.ScalarField(), circuit, true)
		if err != nil {
			t.Fatal(err)
		}
		lc := c.GetLayeredCircuit()
		assignment := rcg.randomAssignment(1)
		witness := c.GetWitness(assignment)
		if !checker.CheckCircuit(lc, witness) {
			t.Fatal("should accept")
		}
	}
}

func TestRandomCircuit1(t *testing.T) {
	testRandomCircuit(t, &randomCircuitConfig{
		seed:     11,
		scNum:    randRange{5, 20},
		scInput:  randRange{5, 50},
		scOutput: randRange{5, 30},
		scInsn:   randRange{20, 50},
		rootInsn: randRange{30, 200},
		field:    ecc.BN254.ScalarField(),
	}, 11, 20)
}
