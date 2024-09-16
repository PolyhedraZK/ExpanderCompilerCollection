# Rust API Documentation

For circuit developers, most of the necessary APIs are provided in the `expander_compiler::frontend` module. If you need to perform more advanced development on top of the compiler, you may need to use other components. This document primarily discusses the contents of the `frontend` module.

Introduction of other modules can be found in [rust_internal.md](./rust_internal.md).

The following items are defined:

```rust
pub use circuit::declare_circuit;
pub type API<C> = builder::RootBuilder<C>;
pub use crate::circuit::config::*;
pub use crate::field::{Field, BN254, GF2, M31};
pub use crate::utils::error::Error;
pub use api::BasicAPI;
pub use builder::Variable;
pub use circuit::Define;
pub use witness::WitnessSolver;

pub fn compile<C: Config, Cir: internal::DumpLoadTwoVariables<Variable> + Define<C> + Clone>(
    circuit: &Cir,
) -> Result<CompileResult<C>, Error> {
    // implementation
}

pub struct CompileResult<C: Config> {
    pub witness_solver: WitnessSolver<C>,
    pub layered_circuit: layered::Circuit<C>,
}

pub mod internal {
    pub use super::circuit::{
        declare_circuit_default, declare_circuit_dump_into, declare_circuit_field_type,
        declare_circuit_load_from, declare_circuit_num_vars,
    };
    pub use super::variables::{DumpLoadTwoVariables, DumpLoadVariables};
    pub use crate::utils::serde::Serde;
}

pub mod extra {
    pub use super::api::UnconstrainedAPI;
    pub use crate::utils::serde::Serde;
}
```

## Declaring a Circuit

The `declare_circuit` macro helps define the structure of a circuit. For example:

```rust
declare_circuit!(Circuit {
    x: Variable,
    y: Variable,
});
```

You can use more complex structures, such as `[[Variable; 256]; N_HASHES]`. The defined struct will look like this:

```rust
pub struct Circuit<T> {
    pub x: T,
    pub y: T,
}
```

## API Overview

The API is similar to `gnark`'s frontend API. `C` represents the configuration for the specified field.

Currently, the `Config` and `Field` types are one-to-one:
- Fields: `BN254`, `GF2`, `M31`
- Configs: `BN254Config`, `GF2Config`, `M31Config`

Many functions and structs use `Config` as a template parameter.

## Error Handling

The `Error` type is returned by many functions and includes `UserError` and `InternalError`. `UserError` typically indicates an issue with your circuit definition, while `InternalError` suggests a problem within the compiler itself. Please contact us if you encounter an `InternalError`.

## Basic API

The `BasicAPI` trait provides a set of operations similar to those in `gnark`. The semantics of `xor`, `or`, and `and` are consistent with `gnark`.

```rust
pub trait BasicAPI<C: Config> {
    fn add(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn sub(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn mul(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn div(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>, checked: bool) -> Variable;
    fn neg(&mut self, x: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn inverse(&mut self, x: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn is_zero(&mut self, x: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn xor(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn or(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn and(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>) -> Variable;
    fn assert_is_zero(&mut self, x: impl ToVariableOrValue<C::CircuitField>);
    fn assert_is_non_zero(&mut self, x: impl ToVariableOrValue<C::CircuitField>);
    fn assert_is_bool(&mut self, x: impl ToVariableOrValue<C::CircuitField>);
    fn assert_is_equal(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>);
    fn assert_is_different(&mut self, x: impl ToVariableOrValue<C::CircuitField>, y: impl ToVariableOrValue<C::CircuitField>);
}
```

## Variables

The `Variable` type is similar to `gnark`'s frontend `Variable`.

## Define Trait

The `Define` trait, similar to `gnark`'s `define`, needs to be implemented for your circuit structure to call the `compile` function.

```rust
pub trait Define<C: Config> {
    fn define(&self, api: &mut RootBuilder<C>);
}
```

## WitnessSolver

The `WitnessSolver` is similar to the `InputSolver` in the Go version of ExpanderCompilerCollection.

## Compile Function

The `compile` function is similar to `gnark`'s `frontend.compile`. The compilation result, `CompileResult`, is similar to the Go version of ExpanderCompilerCollection.

## Internal Module

The `internal` module contains items for internal use, such as macros for proper expansion. Circuit developers typically do not need to handle these.

## Extra Module

The `extra` module includes additional items:

- `UnconstrainedAPI`: Provides operations with semantics consistent with Circom's operators. These operations do not generate constraints and are only called during the witness solving stage. Circuit developers need to manually constrain the results of these operations.
- `Serde`: Defines `serialize_into()` and `deserialize_from()`. These functions can be used to dump compilation results to a file.