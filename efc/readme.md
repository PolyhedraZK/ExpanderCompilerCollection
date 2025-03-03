### Introduction

EthFullConsensus is a circuit library containing a series of circuits to realize Ethereum full consensus. The library can generate a series of circuit files. Given the target assignment data, the library can generate corresponding witness files for generating proof to prove that the supermajority of validators have the same target attestation, i.e., finalizing a source epoch, justifying a target epoch, and keeping the same beacon root. The layered circuit and witness files are used by the [Expander](https://github.com/PolyhedraZK/Expander) prover to generate proofs.

### Concepts

Realizing Ethereum full consensus using SNARKs can be costly. Our design is based on the concepts on [beacon-chain-validator](./spec/beacon-chain-validator.md).

### Workflow

1. Provide the assignment data files, and run the API to generate circuit.txt and witness.txt files
```RUSTFLAGS="-C target-cpu=native" cargo run --bin efc --release -- -d <dir:assignment_data_dir>```
For example, if the assignment data files are on the "~/ExpanderCompilerCollection/efc/data", then run 
```RUSTFLAGS="-C target-cpu=native" cargo run --bin efc --release -- -d ~/ExpanderCompilerCollection/efc/data```
By default, the witness files are saved on the "~/ExpanderCompilerCollection/efc/witnesses".
2. Using Expander to provide the proofs, and verify them
