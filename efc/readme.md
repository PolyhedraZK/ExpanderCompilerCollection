### Introduction

EthFullConsensus is a circuit library containing a series of circuits to realize Ethereum full consensus. The library can generate a series of circuit files. Given the target assignment data, the library can generate corresponding witness files for generating proof to prove that the supermajority of validators have the same target attestation, i.e., finalizing a source epoch, justifying a target epoch, and keeping the same beacon root. The layered circuit and witness files are used by the [Expander](https://github.com/PolyhedraZK/Expander) prover to generate proofs.

### Concepts

Realizing Ethereum full consensus using SNARKs can be costly. Our design is based on the concepts on [beacon-chain-validator](./spec/beacon-chain-validator.md).

### Workflow
0. Enter efc directory
1. Prepare required programs, solvers, and circuits (may take a hour)
```./prepare_solver.sh```
2. Start end-to-end prove and verify
```./efc_end2end.sh --epoch <number>```
