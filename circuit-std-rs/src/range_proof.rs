//! use log up for range proofs

use expander_compiler::{
    declare_circuit,
    frontend::{Config, Define, Variable, API},
};

use crate::LogUpParams;

#[derive(Clone, Copy, Debug)]
pub struct RangeProofParams {
    pub number_of_bits: usize,
}

declare_circuit!(_RangeCircuit {
    // This circuit range checks len(values) number of value
    values: [Variable]
});

pub type RangeProofCircuit = _RangeCircuit<Variable>;

impl<C: Config> Define<C> for RangeProofCircuit {
    fn define(&self, builder: &mut API<C>) {
        let log_up_param = LogUpParams {
            key_len: 1 << 8,
            value_len: 1 << 8,
            n_table_rows: 1 << 8,
            n_queries: self.values.len(),
        };
    }
}
