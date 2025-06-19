//! Cost estimation functions for the circuit.

use super::config::Config;

/// The cost of compressing an expression into a single variable.
/// It estimates the cost of a new variable with corresponding gates.
pub fn cost_of_compress<C: Config>(deg_cnt: &[usize; 3]) -> usize {
    C::COST_MUL * deg_cnt[2]
        + C::COST_ADD * deg_cnt[1]
        + C::COST_CONST * deg_cnt[0]
        + C::COST_VARIABLE
}

/// The cost of multiplying two expressions.
/// It estimates the cost of gates, but not the new variable.
pub fn cost_of_multiply<C: Config>(
    a_deg_0: usize,
    a_deg_1: usize,
    b_deg_0: usize,
    b_deg_1: usize,
) -> usize {
    C::COST_MUL * (a_deg_1 * b_deg_1)
        + C::COST_ADD * (a_deg_0 * b_deg_1 + a_deg_1 * b_deg_0)
        + C::COST_CONST * (a_deg_0 * b_deg_0)
}

/// The cost of possible references to an expression.
/// It estimates the cost of adding and multiplying references to an expression
pub fn cost_of_possible_references<C: Config>(
    deg_cnt: &[usize; 3],
    ref_add: usize,
    ref_mul: usize,
) -> usize {
    C::COST_CONST * (deg_cnt[0] * ref_add)
        + C::COST_ADD * (deg_cnt[1] * ref_add + deg_cnt[0] * ref_mul)
        + C::COST_MUL * (deg_cnt[2] * ref_add + (deg_cnt[1] + deg_cnt[2] * 2) * ref_mul)
}

/// The cost of a relay between two layers.
/// It estimates the cost of n variables, where n is the difference in layers.
pub fn cost_of_relay<C: Config>(v1_layer: usize, v2_layer: usize) -> usize {
    (v1_layer as isize - v2_layer as isize).unsigned_abs() * (C::COST_VARIABLE + C::COST_ADD)
}
