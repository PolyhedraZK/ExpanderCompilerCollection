# ExpanderCompilerCollection

Expander is a proof generation backend for the Polyhedra Network. The ExpanderCompilerCollection is a component of the Expander proof system. It transforms circuits written in [gnark](https://github.com/Consensys/gnark) into an intermediate representation (IR) of a layered circuit. This IR can later be used by the [Expander prover](https://github.com/PolyhedraZK/Expander) to generate proofs.

## Using this Library

To incorporate the compiler into your Go project, include the following import statement in your code:

```go
import "github.com/PolyhedraZK/ExpanderCompilerCollection"
```

The APIs for this library are detailed in [APIs](./docs/apis.md).

## Example 

Refer to [this example](./docs/example.md) for a practical demonstration of our compiler. In this example, we illustrate how a gnark circuit can be compiled using `ExpanderCompilerCollection`. The output of this example includes a circuit description file `"circuit.txt"` and a corresponding witnesses file `"witness.txt"`. Our prover, [Expander](https://github.com/PolyhedraZK/Expander), utilizes these IRs to generate the actual proof.

Additional examples include:
- Hash functions like [sha2](./examples/gnark_std_sha2/main.go), [keccak](./examples/keccak/main.go), and [MIMC](./examples/mimc/main.go)
- A [recursive circuit](./examples/gnark_recursive_proof/main.go)
- A [mersenne field](./examples/m31_field/main.go)

## Deeper Dive in to the tech

For a more technical overview of the overall architecture, visit our [Compilation Process](./docs/compilation_process.md) document.

For a detailed explanation of the primary compilation artifacts - the layered circuit and the input solver, as well as their respective serialization formats, refer to [Artifact and Serialization](./docs/artifact_and_serialization.md).