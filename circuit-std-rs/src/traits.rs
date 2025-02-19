use std::fmt::Debug;

use expander_compiler::frontend::{
    internal::DumpLoadTwoVariables, Config, Define, GenericDefine, Variable,
};
use rand::RngCore;

// All std circuits must implement the following trait
pub trait StdCircuit<C: Config>: Clone + Define<C> + DumpLoadTwoVariables<Variable> {
    type Params: Clone + Debug;
    type Assignment: Clone + DumpLoadTwoVariables<C::CircuitField>;

    // Create a new circuit with the given parameters
    fn new_circuit(params: &Self::Params) -> Self;

    // Create a new random assignment for the circuit
    fn new_assignment(params: &Self::Params, rng: impl RngCore) -> Self::Assignment;
}

// All std circuits must implement the following trait
pub trait StdCircuitGeneric<C: Config>:
    Clone + GenericDefine<C> + DumpLoadTwoVariables<Variable>
{
    type Params: Clone + Debug;
    type Assignment: Clone + DumpLoadTwoVariables<C::CircuitField>;

    // Create a new circuit with the given parameters
    fn new_circuit(params: &Self::Params) -> Self;

    // Create a new random assignment for the circuit
    fn new_assignment(params: &Self::Params, rng: impl RngCore) -> Self::Assignment;
}
