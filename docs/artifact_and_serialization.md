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

At its core, `uint64` types are serialized in little-endian format, while `big.Int` types are serialized as 1/4/32-byte little-endian sequences depending on the particular field.

The serialization of an array begins with a `uint64` that denotes its length, followed by the serialized representation of its elements.

A unique identifier, the magic number 3770719418566461763 (b'CIRCUIT4'), is prefixed to the serialized RootCircuit data stream to ensure data integrity.

Refer [Go implementation](../ecgo/layered/serialize.go) and [Rust implementation](../expander_compiler/src/circuit/layered/serde.rs) for details.

## Input Solver

The input solver (`irwg.RootCircuit`) is an intermediary form of the circuit that facilitates witness generation.

Refer [Go implementation](../ecgo/irwg/serialize.go), [Rust implementation for circuit](../expander_compiler/src/circuit/ir/common/serde.rs) and [Rust implementation for instruction](../expander_compiler/src/circuit/ir/hint_normalized/serde.rs) for details.

## Witness Serialization

The witness, an array of `big.Int`, serves as the input for the layered circuit. It is also defined in `ir/input_solver.go`.

One witness file contains one or multiple witnesses, stored in a compact form.

Refer [Go implementation](../ecgo/irwg/witness_gen.go) for details.