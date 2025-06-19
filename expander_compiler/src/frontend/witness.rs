//! This module provides the `WitnessSolver` struct and its methods for solving circuit witness assignments.

pub use crate::circuit::ir::hint_normalized::witness_solver::WitnessSolver;
use crate::{
    circuit::layered::witness::Witness,
    hints::registry::{EmptyHintCaller, HintCaller},
};

use super::{internal, CircuitField, Config, Error};

impl<C: Config> WitnessSolver<C> {
    /// Solves the witness for a given set of raw inputs.
    pub fn solve_witness<Cir: internal::DumpLoadTwoVariables<CircuitField<C>>>(
        &self,
        assignment: &Cir,
    ) -> Result<Witness<C>, Error> {
        self.solve_witness_with_hints(assignment, &EmptyHintCaller)
    }

    /// Solves the witness for a given set of raw inputs with hints.
    pub fn solve_witness_with_hints<Cir: internal::DumpLoadTwoVariables<CircuitField<C>>>(
        &self,
        assignment: &Cir,
        hint_caller: &impl HintCaller<CircuitField<C>>,
    ) -> Result<Witness<C>, Error> {
        let mut vars = Vec::new();
        let mut public_vars = Vec::new();
        assignment.dump_into(&mut vars, &mut public_vars);
        self.solve_witness_from_raw_inputs(vars, public_vars, hint_caller)
    }

    /// Solves the witness for a set of assignments, where each assignment is a circuit struct.
    pub fn solve_witnesses<Cir: internal::DumpLoadTwoVariables<CircuitField<C>>>(
        &self,
        assignments: &[Cir],
    ) -> Result<Witness<C>, Error> {
        self.solve_witnesses_with_hints(assignments, &EmptyHintCaller)
    }

    /// Solves the witness for a set of assignments, where each assignment is a circuit struct,
    /// using a hint caller to provide additional hints.
    pub fn solve_witnesses_with_hints<Cir: internal::DumpLoadTwoVariables<CircuitField<C>>>(
        &self,
        assignments: &[Cir],
        hint_caller: &impl HintCaller<CircuitField<C>>,
    ) -> Result<Witness<C>, Error> {
        self.solve_witnesses_from_raw_inputs(
            assignments.len(),
            |i| {
                let mut vars = Vec::new();
                let mut public_vars = Vec::new();
                assignments[i].dump_into(&mut vars, &mut public_vars);
                (vars, public_vars)
            },
            hint_caller,
        )
    }
}
