# Compilation Artifacts and Serialization

This document provides an in-depth explanation of the primary compilation artifacts - the layered circuit and the input solver, as well as their respective serialization formats.

## Layered Circuit

The layered circuit is a crucial component that represents the final structure of the compiled circuit. It is used by both the prover and verifier during the zero-knowledge proof process.

### Format Specification

The layered circuits are structured according to the definitions provided in `layered/circuit.go`.


#### Overview

The entire circuit is known as `RootCircuit`, while individual layers are referred to as `Circuit`.

A `RootCircuit` consists of multiple layers, each containing $2^n$ gates. A `Circuit` encapsulates the interconnections of gates between two successive layers.

Circuits can recursively include other circuits as sub-circuits.

The identifier for a `Circuit` corresponds to its position within the `RootCircuit.Circuits` array.

The `RootCircuit.Layers` array holds the identifiers for each layer's `Circuit`.

#### Distinct Representations

A gate is considered random if its `Coef` is equivalent to `RootCircuit.Field`.

### Serialization Process

At its core, `uint64` types are serialized in little-endian format, while `big.Int` types are serialized as 32-byte little-endian sequences.

The serialization of an array begins with a `uint64` that denotes its length, followed by the serialized representation of its elements.

In the serialized form, `Coef` values are constrained to be less than `RootCircuit.Field`. Random gates are represented using additional arrays.

The `Circuit` structure, when serialized, is represented as follows:

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
In this structure, RandomCoefIdx denotes the indices of random gates within the combined arrays of Mul, Add, and Cst.

A unique identifier, the magic number 3626604230490605891 (b'CIRCUIT2'), is prefixed to the serialized RootCircuit data stream to ensure data integrity.

## Input Solver

The input solver is an intermediary form of the circuit that facilitates witness generation. It is defined in ir/input_solver.go.

For Go-specific implementations, serialization is performed using the gob package.

## Witness Serialization

The witness, an array of big.Int, serves as the input for the layered circuit. It is also defined in ir/input_solver.go.

The serialization process for the witness is straightforward. Given the known array length, the big.Int array is serialized as a sequence of 32-byte little-endian values.