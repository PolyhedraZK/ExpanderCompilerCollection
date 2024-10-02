---
sidebar_position: 4
---

# Sub Circuits

A sub-circuit is similar to a component in Circom. It is a segment of a circuit that is called repeatedly and can be packaged into a sub-circuit.

The sub-circuit API in Go is introduced in [APIs](../apis).

The current Rust frontend only supports sub-circuits equivalent to `SubCircuitSimpleFunc` in Go. The function definition is as follows:

```rust
impl<C: Config> RootBuilder<C> {
    pub fn memorized_simple_call<F: Fn(&mut Self, &Vec<Variable>) -> Vec<Variable> + 'static>(
        &mut self,
        f: F,
        inputs: &[Variable],
    ) -> Vec<Variable> {
        // implementation
    }
}
```

An example of its usage in the Keccak circuit can be found at [keccak_gf2](https://github.com/PolyhedraZK/ExpanderCompilerCollection/blob/master/expander_compiler/tests/keccak_gf2.rs#L217). In this example, the sub-circuit call is equivalent to `let out = compute_keccak(&self.p[i].to_vec());`.