---
sidebar_position: 1
---

# ExpanderCompilerCollection

Expander is a proof generation backend for the Polyhedra Network. The ExpanderCompilerCollection is a component of the Expander proof system. It transforms circuits written in high-level languages into layered circuit. This layered circuit can later be used by the [Expander prover](https://github.com/PolyhedraZK/Expander) to generate proofs.

## Typical Workflow

In a typical workflow:

1. Implement the circuit in our circuit frontend language.
2. Use [ExpanderCompilerCollection](https://github.com/PolyhedraZK/ExpanderCompilerCollection) to compiler the circuit into layered circuit.
3. Use [Expander prover](https://github.com/PolyhedraZK/Expander) to generate and verify proofs. You may also use integrated prover inside the compiler.

## Using this Library

Currently we provide Go and Rust APIs. You can find examples and other informations in [Go Walkthrough](go/intro) and [Rust Walkthrough](rust/intro), we are also working on a novel API interface based on Rust, it's called [zkCuda](cuda/cuda_like_frontend), which provides a CUDA-like programming experience.


We also have an experimental [Circom](https://github.com/iden3/circom) preprocessor, which can be found at [circom_preprocessor](https://github.com/PolyhedraZK/ExpanderCompilerCollection/tree/v0.0.3/circom_preprocessor).

Our Go frontend language is compatible with [gnark](https://github.com/ConsenSys/gnark)'s frontend, and existing circuits could be directly used in our compiler.

## Deeper Dive in to the tech

For a more technical overview of the overall architecture, visit our [Compiler Internals](internal/intro) document.

For a detailed explanation of the primary compilation artifacts - the layered circuit and the input solver, as well as their respective serialization formats, refer to [Artifact and Serialization](internal/artifact_and_serialization).

## Acknowledgement

We extend our gratitude to the following projects, whose prior work has been crucial in bringing this project to fruition:

[gnark](https://github.com/Consensys/gnark): our frontend language is based on gnark's frontend.

## Future roadmap

As a compiler collection, we will support more circuit frontend languages in near term.

## Features in progress
* zkCuda: a CUDA-like programming experience
* On-chain verifier generation
* Extended in-circuit randomness generation
