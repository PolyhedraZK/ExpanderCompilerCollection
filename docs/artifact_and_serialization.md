# Compilation Artifacts and Serialization

This section details the two principal compilation artifacts, the layered circuit, and the input solver, along with their serialization formats.

## Layered Circuit

The layered circuit represents the compiled circuit's final structure, utilized by both the prover and verifier during the zero-knowledge proof process.

### Format Specification

Layered circuits are structured as per the definitions in `layered/circuit.go`.

#### Overview

The complete circuit is referred to as `RootCircuit`, with individual layers termed `Circuit`.

A `RootCircuit` comprises multiple layers, each containing $2^n$ gates. The `Circuit` encapsulates the interconnections of gates between two consecutive layers.

Circuits can recursively contain other circuits as sub-circuits.

The identifier for a `Circuit` corresponds to its position within the `RootCircuit.Circuits` array.

The `RootCircuit.Layers` array maintains the identifiers for each layer's `Circuit`.

#### Distinct Representations

A gate is deemed random if its `Coef` is equivalent to `RootCircuit.Field`.

### Serialization Process

Fundamentally, `uint64` types are serialized in little-endian format, while `big.Int` types are serialized as 32-byte little-endian sequences.

An array's serialization begins with a `uint64` denoting its length, succeeded by the serialized representation of its constituent elements.

In serialized form, `Coef` values are constrained to be less than `RootCircuit.Field`. Random gates are represented using additional arrays.

The serialized structure of a `Circuit` can be visualized as follows:

```go
type Circuit struct {
    InputLen      uint64
    OutputLen     uint64
    SubCircuits   []SubCircuit
    Mul           []GateMul
    Add           []GateAdd
    Cst           []GateCst
    RandomCoefIdx []uint64
}
```

In this structure, `RandomCoefIdx` records the indices of random gates within the combined arrays of `Mul`, `Add`, and `Cst`.

A unique identifier, the magic number 3626604230490605891 (`b'CIRCUIT2'`), is prefixed to the serialized `RootCircuit` data stream.

## Input Solver

The input solver, an intermediary form of the circuit, aids in witness generation and is defined within `ir/input_solver.go`.

For Go-specific usage, serialization is performed using the `gob` package.

## Witness Serialization

The witness, an array of `big.Int`, constitutes the input for the layered circuit and is also defined in `ir/input_solver.go`.

Serialization of the witness is straightforward. Given the known array length, the `big.Int` array is serialized as a sequence of 32-byte little-endian values.