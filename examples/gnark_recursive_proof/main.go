package main

import (
	"fmt"
	"math/big"
	"os"
	"runtime/debug"

	"github.com/PolyhedraZK/ExpanderCompilerCollection"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/ir"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/layered"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/test"
	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend/groth16"
	"github.com/consensys/gnark/backend/witness"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/r1cs"
	"github.com/consensys/gnark/std/algebra"
	"github.com/consensys/gnark/std/algebra/emulated/sw_bn254"
	stdgroth16 "github.com/consensys/gnark/std/recursion/groth16"
)

// InnerCircuitNative is the definition of the inner circuit we want to
// recursively verify inside an outer circuit. The circuit proves the knowledge
// of a factorisation of a semiprime.
type InnerCircuitNative struct {
	P, Q frontend.Variable
	N    frontend.Variable `gnark:",public"`
}

func (c *InnerCircuitNative) Define(api frontend.API) error {
	// prove that P*Q == N
	res := api.Mul(c.P, c.Q)
	api.AssertIsEqual(res, c.N)
	// we must also enforce that P != 1 and Q != 1
	api.AssertIsDifferent(c.P, 1)
	api.AssertIsDifferent(c.Q, 1)
	return nil
}

// computeInnerProof computes the proof for the inner circuit we want to verify
// recursively. In this example the Groth16 keys are generated on the fly, but
// in practice should be genrated once and using MPC.
func computeInnerProof(field *big.Int) (constraint.ConstraintSystem, groth16.VerifyingKey, witness.Witness, groth16.Proof) {
	innerCcs, err := frontend.Compile(field, r1cs.NewBuilder, &InnerCircuitNative{})
	if err != nil {
		panic(err)
	}
	// NB! UNSAFE! Use MPC.
	innerPK, innerVK, err := groth16.Setup(innerCcs)
	if err != nil {
		panic(err)
	}

	// inner proof
	innerAssignment := &InnerCircuitNative{
		P: 3,
		Q: 5,
		N: 15,
	}
	innerWitness, err := frontend.NewWitness(innerAssignment, field)
	if err != nil {
		panic(err)
	}
	innerProof, err := groth16.Prove(innerCcs, innerPK, innerWitness)
	if err != nil {
		panic(err)
	}
	innerPubWitness, err := innerWitness.Public()
	if err != nil {
		panic(err)
	}
	err = groth16.Verify(innerProof, innerVK, innerPubWitness)
	if err != nil {
		panic(err)
	}
	return innerCcs, innerVK, innerPubWitness, innerProof
}

// OuterCircuit is the generic outer circuit which can verify Groth16 proofs
// using field emulation or 2-chains of curves.
type OuterCircuit[S algebra.ScalarT, G1El algebra.G1ElementT, G2El algebra.G2ElementT, GtEl algebra.GtElementT] struct {
	Proof        stdgroth16.Proof[G1El, G2El]
	VerifyingKey stdgroth16.VerifyingKey[G1El, G2El, GtEl]
	InnerWitness stdgroth16.Witness[S]
}

func (c *OuterCircuit[S, G1El, G2El, GtEl]) Define(api frontend.API) error {
	curve, err := algebra.GetCurve[S, G1El](api)
	if err != nil {
		return fmt.Errorf("new curve: %w", err)
	}
	pairing, err := algebra.GetPairing[G1El, G2El, GtEl](api)
	if err != nil {
		return fmt.Errorf("get pairing: %w", err)
	}
	verifier := stdgroth16.NewVerifier(curve, pairing)
	err = verifier.AssertProof(c.VerifyingKey, c.Proof, c.InnerWitness)
	return err
}

func main() {
	innerCcs, innerVK, innerWitness, innerProof := computeInnerProof(ecc.BN254.ScalarField())

	// initialize the witness elements
	circuitVk, err := stdgroth16.ValueOfVerifyingKey[sw_bn254.G1Affine, sw_bn254.G2Affine, sw_bn254.GTEl](innerVK)
	if err != nil {
		panic(err)
	}
	circuitWitness, err := stdgroth16.ValueOfWitness[sw_bn254.Scalar, sw_bn254.G1Affine](innerWitness)
	if err != nil {
		panic(err)
	}
	circuitProof, err := stdgroth16.ValueOfProof[sw_bn254.G1Affine, sw_bn254.G2Affine](innerProof)
	if err != nil {
		panic(err)
	}

	outerAssignment := &OuterCircuit[sw_bn254.Scalar, sw_bn254.G1Affine, sw_bn254.G2Affine, sw_bn254.GTEl]{
		InnerWitness: circuitWitness,
		Proof:        circuitProof,
		VerifyingKey: circuitVk,
	}

	var is *ir.InputSolver
	var lc *layered.RootCircuit
	if len(os.Args) > 1 && os.Args[1] == "--use-compiled" {
		s, _ := os.ReadFile("inputsolver.txt")
		is = ExpanderCompilerCollection.DeserializeInputSolver(s)
		s, _ = os.ReadFile("circuit.txt")
		lc = ExpanderCompilerCollection.DeserializeLayeredCircuit(s)
	} else {
		outerCircuit := &OuterCircuit[sw_bn254.Scalar, sw_bn254.G1Affine, sw_bn254.G2Affine, sw_bn254.GTEl]{
			InnerWitness: stdgroth16.PlaceholderWitness[sw_bn254.Scalar](innerCcs),
			VerifyingKey: stdgroth16.PlaceholderVerifyingKey[sw_bn254.G1Affine, sw_bn254.G2Affine, sw_bn254.GTEl](innerCcs),
		}

		cr, _ := ExpanderCompilerCollection.Compile(ecc.BN254.ScalarField(), outerCircuit)

		is = cr.GetInputSolver()
		lc = cr.GetLayeredCircuit()
		cr = nil

		debug.FreeOSMemory()
		os.WriteFile("inputsolver.txt", is.Serialize(), 0o644)
		debug.FreeOSMemory()
		os.WriteFile("circuit.txt", lc.Serialize(), 0o644)
		debug.FreeOSMemory()
	}

	witness, err := is.SolveInput(outerAssignment, 8)
	if err != nil {
		panic("witness failed: " + err.Error())
	}

	if !test.CheckCircuit(lc, witness) {
		panic("circuit verification failed")
	}
	fmt.Println("done")
}
