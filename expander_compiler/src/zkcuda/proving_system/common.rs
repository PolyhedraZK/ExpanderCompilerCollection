use crate::circuit::config::Config;
use arith::Field;

use super::super::kernel::Kernel;
use super::Commitment;

pub fn check_inputs<C: Config>(
    kernel: &Kernel<C>,
    commitments: &[&impl Commitment<C>],
    parallel_count: usize,
    is_broadcast: &[bool],
) {
    if kernel.layered_circuit_input.len() != commitments.len() {
        panic!("Input size mismatch");
    }
    if kernel.layered_circuit_input.len() != is_broadcast.len() {
        panic!("Input size mismatch");
    }
    for i in 0..kernel.layered_circuit_input.len() {
        if is_broadcast[i] {
            if kernel.layered_circuit_input[i].len != commitments[i].vals_len() {
                panic!("Input size mismatch");
            }
        } else if kernel.layered_circuit_input[i].len * parallel_count != commitments[i].vals_len()
        {
            panic!("Input size mismatch");
        }
    }
}

pub fn prepare_inputs<C: Config>(
    kernel: &Kernel<C>,
    commitments: &[&impl Commitment<C>],
    is_broadcast: &[bool],
    parallel_index: usize,
) -> Vec<C::DefaultSimdField> {
    let mut lc_input = vec![C::DefaultSimdField::zero(); kernel.layered_circuit.input_size()];
    for ((input, commitment), ib) in kernel
        .layered_circuit_input
        .iter()
        .zip(commitments.iter())
        .zip(is_broadcast)
    {
        if *ib {
            for (i, x) in commitment.vals_ref().iter().enumerate() {
                lc_input[input.offset + i] = *x;
            }
        } else {
            for (i, x) in commitment
                .vals_ref()
                .iter()
                .skip(parallel_index * input.len)
                .take(input.len)
                .enumerate()
            {
                lc_input[input.offset + i] = *x;
            }
        }
    }
    lc_input
}
