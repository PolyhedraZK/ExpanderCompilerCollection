---
sidebar_position: 1
---

# Introduction

## Using this Library

To incorporate the compiler into your Go project, include the following import statement in your code:

```go
import "github.com/PolyhedraZK/ExpanderCompilerCollection"
```

The APIs for this library are detailed in [Compiler APIs](./apis).

## Example 

Refer to [this example](./example) for a practical demonstration of our compiler. In this example, we illustrate how a gnark circuit can be compiled using `ExpanderCompilerCollection`. The output of this example includes a circuit description file `"circuit.txt"` and a corresponding witnesses file `"witness.txt"`. Our prover, [Expander](https://github.com/PolyhedraZK/Expander), utilizes these IRs to generate the actual proof.

Additional examples include:
- Hash functions like [keccak](https://github.com/PolyhedraZK/ExpanderCompilerCollection/blob/master/ecgo/examples/keccak/main.go) and [MIMC](https://github.com/PolyhedraZK/ExpanderCompilerCollection/blob/master/ecgo/examples/mimc/main.go)
- A [mersenne field](https://github.com/PolyhedraZK/ExpanderCompilerCollection/blob/master/ecgo/examples/m31_field/main.go)