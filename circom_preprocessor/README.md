# Circom Preprocessor

The Circom Preprocessor converts circuits described in the Circom language into a format compatible with ECC and Gnark. This enables subsequent development using ECC and Gnark.

## Installation and Usage

Ensure you have the Rust development environment installed. Then, you can compile the project using cargo:

```shell
cargo build --release
```

Next, you can view the usage help:

```shell
target/release/circom_preprocessor --help
```

The parameters are similar to those of the original Circom compiler.

## Progress

Currently, only the Poseidon and Pedersen circuits from circomlib have been tested and verified to convert correctly.

Some operators (such as BitAnd) have not yet been implemented. The corresponding interfaces need to be developed in ECC first. Please stay tuned for future updates.