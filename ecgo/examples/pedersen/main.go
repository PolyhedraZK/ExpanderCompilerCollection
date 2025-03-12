package circuit

import (
	"fmt"
	"os"

	"github.com/consensys/gnark/frontend"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark-crypto/ecc/bn254/fr"
	"github.com/consensys/gnark-crypto/ecc/bn254/fr/pedersen"

	"github.com/hblocks/keyless/pkg/commitment"

	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo/test"
)

type PCommitment struct {
	VK               bn254.G2Affine
	Commit           bn254.G1Affine
	Proof            bn254.G1Affine
	CombinationCoeff fr.Element
}

func PedersenProver() (*PCommitment, error) {
	_, _, g1Gen, g2Gen := commitment.Generate()

	basis := [][]bn254.G1Affine{{g1Gen}}
	pk, vk, err := pedersen.Setup(basis, pedersen.WithG2Point(g2Gen))
	if err != nil {
		return nil, fmt.Errorf("pedersen setup failed: %w", err)
	}

	values := []fr.Element{fr.NewElement(123)} // one secret
	commit, err := pk[0].Commit(values)
	if err != nil {
		return nil, fmt.Errorf("commit failed: %w", err)
	}

	proof, err := pk[0].ProveKnowledge(values)
	if err != nil {
		return nil, fmt.Errorf("prove knowledge failed: %w", err)
	}

	if err := pedersen.BatchVerifyMultiVk(
		[]pedersen.VerifyingKey{vk},
		[]bn254.G1Affine{commit},
		[]bn254.G1Affine{proof},
		fr.NewElement(42),
	); err != nil {
		return nil, fmt.Errorf("batch verify failed: %w", err)
	}

	pc := &PCommitment{
		VK:               vk.G,
		Commit:           commit,
		Proof:            proof,
		CombinationCoeff: fr.NewElement(42), // example
	}
	return pc, nil
}

type Circuit struct {
	VK     bn254.G2Affine    `gnark:"public"`
	Commit bn254.G1Affine    `gnark:"secret"`
	Proof  bn254.G1Affine    `gnark:"secret"`
	Combo  frontend.Variable `gnark:"secret"`
}

// Define: minimal constraints
func (c *Circuit) Define(api frontend.API) error {
	commitX := c.Commit.X
	commitY := c.Commit.Y
	proofX := c.Proof.X
	proofY := c.Proof.Y

	computedProofX := api.Add(commitX, c.Combo)
	computedProofY := api.Add(commitY, c.Combo)

	api.AssertIsEqual(computedProofX, proofX)
	api.AssertIsEqual(computedProofY, proofY)

	return nil
}

func Prover(assignment *Circuit) error {
	circ, err := ecgo.Compile(ecc.BN254.ScalarField(), assignment)
	if err != nil {
		return fmt.Errorf("failed to compile circuit: %w", err)
	}

	layered := circ.GetLayeredCircuit()
	if err = os.WriteFile("circuit.txt", layered.Serialize(), 0o644); err != nil {
		return fmt.Errorf("failed to write circuit: %w", err)
	}

	inputSolver := circ.GetInputSolver()
	witness, err := inputSolver.SolveInputAuto(assignment)
	if err != nil {
		return fmt.Errorf("failed to solve input: %w", err)
	}
	if err = os.WriteFile("witness.txt", witness.Serialize(), 0o644); err != nil {
		return fmt.Errorf("failed to write witness: %w", err)
	}

	if ok := test.CheckCircuit(layered, witness); !ok {
		return fmt.Errorf("self-check circuit verification failed")
	}
	fmt.Println("Circuit compiled, witness solved, and self-checked successfully!")
	return nil
}

func RunPedersenCircuitDemo() {
	pCommitment, err := PedersenProver()
	if err != nil {
		panic(err)
	}

	assignment := &Circuit{
		VK:     pCommitment.VK,
		Commit: pCommitment.Commit,
		Proof:  pCommitment.Proof,
		Combo:  pCommitment.CombinationCoeff,
	}

	if err = Prover(assignment); err != nil {
		panic(err)
	}
}
