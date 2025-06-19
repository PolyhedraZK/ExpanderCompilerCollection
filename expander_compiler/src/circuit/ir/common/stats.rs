//! This module provides statistics gathering functionality for the circuit IR.

use std::collections::HashMap;

use super::{Instruction, IrConfig, RootCircuit};

/// This struct contains statistics about the circuit IR.
pub struct Stats {
    /// The number of inputs in the root circuit.
    pub num_inputs: usize,
    /// The number of instructions in the root circuit.
    pub num_insns: usize,
    /// The number of terms in the root circuit.
    pub num_terms: usize,
    /// The number of variables in the root circuit.
    pub num_variables: usize,
    /// The number of constraints in the root circuit.
    pub num_constraints: usize,
}

struct CircuitStats {
    num_terms: usize,
    num_insns: usize,
    num_variables: usize,
    num_constraints: usize,
}

struct StatsContext<'a, Irc: IrConfig> {
    rc: &'a RootCircuit<Irc>,
    m: HashMap<usize, CircuitStats>,
}

impl<Irc: IrConfig> RootCircuit<Irc> {
    /// Returns the statistics of the root circuit.
    pub fn get_stats(&self) -> Stats {
        let mut sc = StatsContext {
            rc: self,
            m: HashMap::new(),
        };
        let mut r = Stats {
            num_inputs: self.input_size(),
            num_insns: 0,
            num_terms: 0,
            num_variables: 0,
            num_constraints: 0,
        };
        for id in self.circuits.keys() {
            sc.calc_circuit_stats(*id);
            r.num_terms += sc.m[id].num_terms;
            r.num_insns += sc.m[id].num_insns;
            r.num_variables += sc.m[id].num_variables;
            r.num_constraints += sc.m[id].num_constraints;
        }
        r
    }
}

impl<'a, Irc: IrConfig> StatsContext<'a, Irc> {
    fn calc_circuit_stats(&mut self, id: usize) {
        if self.m.contains_key(&id) {
            return;
        }
        let circuit = &self.rc.circuits[&id];
        let mut r = CircuitStats {
            num_terms: 0,
            num_insns: 0,
            num_variables: 0,
            num_constraints: circuit.constraints.len(),
        };
        for insn in &circuit.instructions {
            r.num_insns += 1;
            r.num_variables += insn.num_outputs();
            r.num_terms += insn.inputs().len();
            if let Some((sub_circuit_id, _, _)) = insn.as_sub_circuit_call() {
                self.calc_circuit_stats(sub_circuit_id);
                r.num_terms += self.m[&sub_circuit_id].num_terms;
                r.num_constraints += self.m[&sub_circuit_id].num_constraints;
                r.num_variables += self.m[&sub_circuit_id].num_variables;
            }
        }
        self.m.insert(id, r);
    }
}
