---
sidebar_position: 5
---

# Supported Fields

Currently, our compiler supports the following fields:

- **BN254**: A commonly used elliptic curve modulus, utilized in Ethereum.
- **M31**: The Mersenne prime $2^{31}-1$. Operations in this field can be efficiently implemented using 64-bit integers.
- **GF2**: The binary field, used for efficiently implementing Boolean circuits.

The Expander prover will use the extension fields of these domains, but you do not need to worry about these when using the compiler.