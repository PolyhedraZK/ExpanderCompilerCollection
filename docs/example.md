# Example

Consider the following gnark circuit as an example:

```go
package main

import (
	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend/groth16"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/r1cs"
)

type Circuit struct {
	X frontend.Variable
	Y frontend.Variable
}

func (circuit *Circuit) Define(api frontend.API) error {
	api.AssertIsEqual(circuit.X, circuit.Y)
	return nil
}

func main() {
	assignment := &Circuit{X: 1, Y: 1}

	r1cs, _ := frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, &Circuit{})
	witness, _ := frontend.NewWitness(assignment, ecc.BN254.ScalarField())
	publicWitness, _ := witness.Public()
	pk, vk, _ := groth16.Setup(r1cs)
	proof, _ := groth16.Prove(r1cs, pk, witness)
	err := groth16.Verify(proof, vk, publicWitness)
	if err != nil {
		panic(err)
	}
}
```

To adapt this circuit for our compiler, we only need to modify the compilation and execution sections within the main function. Here's how:

```go
package main

import (
	"os"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/frontend"

	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo"
	"github.com/PolyhedraZK/ExpanderCompilerCollection/ecgo/test"
)

type Circuit struct {
	X frontend.Variable
	Y frontend.Variable
}

func (circuit *Circuit) Define(api frontend.API) error {
	api.AssertIsEqual(circuit.X, circuit.Y)
	return nil
}

func main() {
	assignment := &Circuit{X: 1, Y: 1}

	circuit, _ := ecgo.Compile(ecc.BN254.ScalarField(), &Circuit{})
	c := circuit.GetLayeredCircuit()
	os.WriteFile("circuit.txt", c.Serialize(), 0o644)
	inputSolver := circuit.GetInputSolver()
	witness, _ := inputSolver.SolveInputAuto(assignment)
	os.WriteFile("witness.txt", witness.Serialize(), 0o644)
	if !test.CheckCircuit(c, witness) {
		panic("verification failed")
	}
}
```

Please note that the Prover's implementation resides in [Expander](https://github.com/PolyhedraZK/Expander). As such, the `CheckCircuit` function in this context merely evaluates the circuit once to confirm its correctness. To generate and verify the actual proof, supply `circuit.txt` and `witness.txt` to Expander.

In contrast to gnark, where the prover handles a portion of the witness generation, our compiler employs an `InputSolver` to perform this task. The `InputSolver` also supports serialization and deserialization. For more information, please consult the examples in [MIMC](../ecgo/examples/mimc) and [MIMC Deserialize](../ecgo/examples/mimc_deserialize).

For additional features, please refer to [APIs](./apis.md).

For more examples, please visit [examples](../ecgo/examples).