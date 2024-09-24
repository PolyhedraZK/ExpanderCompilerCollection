---
sidebar_position: 4
---

# How to Implement Hints

Sometimes, it is difficult to compute a value within a circuit, and we can only compute it externally and then verify its correctness within the circuit. Common scenarios include calculating division or breaking a number down into the sum of its bits.

In gnark, this is achieved through hints, as detailed in [Hints](https://docs.gnark.consensys.io/HowTo/write/hints). Although our Rust API strives to simulate gnark's Go API, due to certain limitations of Rust, we currently do not have a function similar to gnark's `api.NewHint`. We plan to implement this in the future, so stay tuned.

Of course, there are currently some ways to achieve this external computation. We have implemented a method similar to that in circom, where you can perform arbitrary operations through a series of functions called `UnconstrainedAPI`, without generating constraints within the circuit. Its definition is as follows and can be called via `expander_compiler::frontend::extra::UnconstrainedAPI`.

```rust
pub trait UnconstrainedAPI<C: Config> {
    fn unconstrained_identity(&mut self, x: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn unconstrained_add(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn unconstrained_mul(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn unconstrained_div(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn unconstrained_pow(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn unconstrained_int_div(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn unconstrained_mod(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn unconstrained_shift_l(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn unconstrained_shift_r(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn unconstrained_lesser_eq(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn unconstrained_greater_eq(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn unconstrained_lesser(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn unconstrained_greater(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn unconstrained_eq(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn unconstrained_not_eq(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn unconstrained_bool_or(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn unconstrained_bool_and(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn unconstrained_bit_or(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn unconstrained_bit_and(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn unconstrained_bit_xor(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>) -> Variable;
}
```

The semantics of these APIs are actually consistent with the operators in circom ([Basic Operators](https://docs.circom.io/circom-language/basic-operators/)). In circom, only addition and multiplication can generate constraints within the circuit (via `<==`), while other operators can only perform non-constraining assignments (via `<--`). These assignments have the same effect as the above APIs.

You can also find an example of using this API to decompose a number into bits [here](https://github.com/PolyhedraZK/ExpanderCompilerCollection/blob/master/expander_compiler/tests/to_binary_unconstrained_api.rs).