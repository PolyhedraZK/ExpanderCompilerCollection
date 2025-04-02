use std::fmt::Debug;

use expander_compiler::frontend::{internal::DumpLoadTwoVariables, CircuitField, Config, Define, Variable};
use rand::RngCore;

// All std circuits must implement the following trait
pub trait StdCircuit<C: Config>: Clone + Define<C> + DumpLoadTwoVariables<Variable> {
    type Params: Clone + Debug;
    type Assignment: Clone + DumpLoadTwoVariables<CircuitField<C>>;

    fn new_circuit(params: &Self::Params) -> Self;

    fn new_assignment(params: &Self::Params, rng: impl RngCore) -> Self::Assignment;
}
