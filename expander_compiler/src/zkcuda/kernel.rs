use crate::circuit::{
    config::Config,
    input_mapping::{InputMapping, EMPTY},
    ir::{self, expr},
    layered::Circuit as LayeredCircuit,
};
use crate::field::FieldArith;
use crate::frontend::*;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Kernel<C: Config> {
    pub witness_solver: ir::hint_normalized::RootCircuit<C>,
    pub layered_circuit: LayeredCircuit<C>,
    pub witness_solver_io: Vec<WitnessSolverIOVec>,
    pub witness_solver_hint_input: Option<WitnessSolverIOVec>,
    pub layered_circuit_input: Vec<LayeredCircuitInputVec>,
}

#[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
pub struct WitnessSolverIOVec {
    pub len: usize,
    pub input_offset: Option<usize>,
    pub output_offset: Option<usize>,
}

#[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
pub struct LayeredCircuitInputVec {
    pub len: usize,
    pub offset: usize,
}

pub struct IOVecSpec {
    pub len: usize,
    pub is_input: bool,
    pub is_output: bool,
}

fn dup_inputs<C: Config>(api: &mut API<C>, inputs: &Vec<Variable>) -> Vec<Variable> {
    use extra::UnconstrainedAPI;
    let mut res = vec![];
    for x in inputs {
        res.push(api.unconstrained_identity(x));
    }
    res
}

pub fn compile_with_spec<C, F>(f: F, io_specs: &[IOVecSpec]) -> Result<Kernel<C>, Error>
where
    C: Config,
    F: Fn(&mut API<C>, &mut Vec<Vec<Variable>>),
{
    let total_inputs = io_specs
        .iter()
        .map(|spec| spec.len * (spec.is_input as usize + spec.is_output as usize))
        .sum();
    let (mut root_builder, input_variables, _) = API::<C>::new(total_inputs, 0);
    let mut io_vars = vec![];
    let mut expected_outputs = vec![];
    let mut inputs_offsets = vec![];
    let mut expected_outputs_offsets = vec![];
    let mut global_input_offset = 0;
    let mut lc_in = vec![];
    for spec in io_specs {
        let mut cur_inputs = vec![];
        if spec.is_input {
            for i in 0..spec.len {
                cur_inputs.push(input_variables[global_input_offset + i]);
            }
            inputs_offsets.push(global_input_offset);
            lc_in.push(LayeredCircuitInputVec {
                len: spec.len,
                offset: global_input_offset,
            });
            global_input_offset += spec.len;
        } else {
            for _ in 0..spec.len {
                cur_inputs.push(root_builder.constant(0));
            }
            inputs_offsets.push(0);
        }
        io_vars.push(cur_inputs);
    }
    let n_in = global_input_offset;
    for spec in io_specs {
        if spec.is_output {
            let mut cur_outputs = vec![];
            for i in 0..spec.len {
                cur_outputs.push(input_variables[global_input_offset + i]);
            }
            expected_outputs.push(cur_outputs);
            expected_outputs_offsets.push(global_input_offset);
            lc_in.push(LayeredCircuitInputVec {
                len: spec.len,
                offset: global_input_offset,
            });
            global_input_offset += spec.len;
        } else {
            expected_outputs.push(vec![]);
            expected_outputs_offsets.push(0);
        }
    }
    let mut io_off = vec![];
    for i in 0..io_specs.len() {
        io_off.push(WitnessSolverIOVec {
            len: io_specs[i].len,
            input_offset: if io_specs[i].is_input {
                Some(inputs_offsets[i])
            } else {
                None
            },
            output_offset: if io_specs[i].is_output {
                Some(expected_outputs_offsets[i])
            } else {
                None
            },
        });
    }
    f(&mut root_builder, &mut io_vars);
    let mut output_offsets = vec![];
    let mut global_output_offset = 0;
    let mut output_vars = vec![];
    for (i, spec) in io_specs.iter().enumerate() {
        if spec.is_output {
            for (x, y) in io_vars[i].iter().zip(expected_outputs[i].iter()) {
                root_builder.assert_is_equal(x, y);
                output_vars.push(*x);
            }
            output_offsets.push(global_output_offset);
            global_output_offset += spec.len;
        } else {
            output_offsets.push(0);
        }
    }
    let dup_out = root_builder.memorized_simple_call(dup_inputs, &output_vars);
    let output_vars_ids: Vec<usize> = dup_out.iter().map(|x| x.id()).collect();

    // prevent optimization
    let mut r_source = root_builder.build();
    let c0 = r_source.circuits.get_mut(&0).unwrap();
    for i in 1..=total_inputs {
        c0.outputs.push(i);
    }
    c0.outputs.extend_from_slice(&output_vars_ids);
    // compile step 1
    let (r_hint_normalized_opt, src_im) = crate::compile::compile_step_1(&r_source)?;
    for (i, x) in src_im.mapping().iter().enumerate() {
        assert_eq!(i, *x);
    }
    // export hints
    let (mut r_hint_less, mut r_hint_exported) = r_hint_normalized_opt.remove_and_export_hints();
    // remove additional hints, move them to user outputs
    let rl_c0 = r_hint_less.circuits.get_mut(&0).unwrap();
    let re_c0 = r_hint_exported.circuits.get_mut(&0).unwrap();
    let n_out = output_vars_ids.len();
    let off1 = re_c0.outputs.len() - n_out;
    let off2 = n_in;
    for i in 0..n_out {
        re_c0.outputs.swap(off1 + i, off2 + i);
    }
    rl_c0.num_inputs -= n_out;
    let mut add_insns = vec![];
    for i in 0..n_out {
        add_insns.push(ir::hint_less::Instruction::LinComb(expr::LinComb {
            terms: vec![expr::LinCombTerm {
                var: n_in + i + 1,
                coef: C::CircuitField::one(),
            }],
            constant: C::CircuitField::zero(),
        }));
    }
    add_insns.extend_from_slice(&rl_c0.instructions);
    rl_c0.instructions = add_insns;
    assert_eq!(rl_c0.outputs.len(), n_in + n_out * 2);
    rl_c0.outputs.truncate(n_in + n_out);
    let num_inputs_with_hint = rl_c0.num_inputs;
    // compile step 2
    let (mut r_dest_opt, hl_im) = crate::compile::compile_step_2(r_hint_less)?;
    for (i, x) in hl_im.mapping().iter().enumerate() {
        assert_eq!(i, *x);
    }
    // remove outputs that used for prevent optimization
    let rd_c0 = r_dest_opt.circuits.get_mut(&0).unwrap();
    rd_c0.outputs.truncate(rd_c0.outputs.len() - n_in - n_out);
    // compile step 3
    let (lc, dest_im) = crate::layering::compile(
        &r_dest_opt,
        crate::layering::CompileOptions { is_zkcuda: true },
    );
    for (i, x) in dest_im.mapping().iter().enumerate() {
        if i < num_inputs_with_hint {
            assert_eq!(i, *x);
        } else {
            assert_eq!(*x, EMPTY);
        }
    }
    let lc = crate::compile::compile_step_3(lc)?;
    // compile step 4
    let mut tmp_im = InputMapping::new_identity(r_hint_exported.input_size());
    let mut r_hint_exported_opt = crate::compile::compile_step_4(r_hint_exported, &mut tmp_im)?;
    for (i, x) in tmp_im.mapping().iter().enumerate() {
        assert_eq!(i, *x);
    }
    let re_c0 = r_hint_exported_opt.circuits.get_mut(&0).unwrap();
    re_c0.outputs.truncate(off1);
    let hint_size = re_c0.outputs.len() - n_in - n_out;
    let hint_io = if hint_size > 0 {
        lc_in.push(LayeredCircuitInputVec {
            len: hint_size,
            offset: n_in + n_out,
        });
        Some(WitnessSolverIOVec {
            len: hint_size,
            input_offset: None,
            output_offset: Some(n_in + n_out),
        })
    } else {
        None
    };

    Ok(Kernel {
        witness_solver: r_hint_exported_opt,
        layered_circuit: lc,
        witness_solver_io: io_off,
        witness_solver_hint_input: hint_io,
        layered_circuit_input: lc_in,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example_kernel_1<C: Config>(api: &mut API<C>, a: &mut Vec<Vec<Variable>>) {
        let x = a[1][1];
        a[0][0] = x;
        a[1][2] = api.add(x, 1);
    }

    #[test]
    fn test_1() {
        let kernel: Kernel<M31Config> = compile_with_spec(
            example_kernel_1,
            &[
                IOVecSpec {
                    len: 1,
                    is_input: true,
                    is_output: true,
                },
                IOVecSpec {
                    len: 3,
                    is_input: true,
                    is_output: true,
                },
            ],
        )
        .unwrap();
        println!(
            "{} {} {}",
            kernel.layered_circuit.num_public_inputs,
            kernel.layered_circuit.num_actual_outputs,
            kernel.layered_circuit.expected_num_output_zeroes
        );
    }
}
