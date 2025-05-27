use crate::compile::{
    compile_step_1, compile_step_2, compile_step_3, compile_step_4, print_ir_stats,
    print_layered_circuit_stats,
};
use crate::frontend::{extra, BasicAPI, CompileOptions, Error, RootAPI, Variable, API};
use crate::utils::pool::Pool;
use crate::{
    circuit::{
        config::{CircuitField, Config},
        input_mapping::{InputMapping, EMPTY},
        ir::{self, common::Instruction, expr},
        layered::{Circuit as LayeredCircuit, NormalInputType},
    },
    field::FieldArith,
    utils::misc::next_power_of_two,
};
pub use macros::kernel;

use exp_serde::ExpSerde;

#[derive(Debug, Clone, Hash, PartialEq, Eq, ExpSerde)]
pub struct Kernel<C: Config> {
    pub witness_solver: ir::hint_normalized::RootCircuit<C>,
    pub layered_circuit: LayeredCircuit<C, NormalInputType>,
    pub io_shapes: Vec<Shape>,
    pub witness_solver_io: Vec<WitnessSolverIOVec>,
    pub witness_solver_hint_input: Option<WitnessSolverIOVec>,
    pub layered_circuit_input: Vec<LayeredCircuitInputVec>,
}

pub type Shape = Option<Vec<usize>>;

pub fn shape_prepend(shape: &Shape, x: usize) -> Shape {
    match shape {
        Some(shape) => {
            let mut shape = shape.clone();
            shape.insert(0, x);
            Some(shape)
        }
        None => None,
    }
}

#[derive(Default, Debug, Clone, Hash, PartialEq, Eq, ExpSerde)]
pub struct WitnessSolverIOVec {
    pub len: usize,
    pub input_offset: Option<usize>,
    pub output_offset: Option<usize>,
}

#[derive(Default, Debug, Clone, Hash, PartialEq, Eq, ExpSerde)]
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
    compile_with_spec_and_shapes(f, io_specs, &vec![None; io_specs.len()])
}

pub fn compile_with_spec_and_shapes<C, F>(
    f: F,
    io_specs: &[IOVecSpec],
    shapes: &[Option<Vec<usize>>],
) -> Result<Kernel<C>, Error>
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
    let en0 = r_source.expected_num_output_zeroes;
    let c0 = r_source.circuits.get_mut(&0).unwrap();
    let tmp = remove_duplicate(en0, &mut c0.outputs);
    let (mut r_hint_normalized_opt, src_im) = compile_step_1(&r_source)?;
    let en0 = r_hint_normalized_opt.expected_num_output_zeroes;
    let c0 = r_hint_normalized_opt.circuits.get_mut(&0).unwrap();
    add_duplicate(en0, &mut c0.outputs, &tmp);
    for (i, x) in src_im.mapping().iter().enumerate() {
        assert_eq!(i, *x);
    }
    print_ir_stats(&r_hint_normalized_opt);
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
    let hint_size = off1 - n_in - n_out;
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
    let mut add_insns = vec![];
    for i in 0..n_out {
        add_insns.push(ir::hint_less::Instruction::LinComb(expr::LinComb {
            terms: vec![expr::LinCombTerm {
                var: n_in + i + 1,
                coef: CircuitField::<C>::one(),
            }],
            constant: CircuitField::<C>::zero(),
        }));
    }
    add_insns.extend_from_slice(&rl_c0.instructions);
    rl_c0.instructions = add_insns;
    assert_eq!(rl_c0.outputs.len(), n_in + n_out * 2);
    rl_c0.outputs.truncate(n_in + n_out);
    // reorder inputs
    reorder_ir_inputs(&mut r_hint_less, &mut lc_in);
    let rhl_c0 = r_hint_less.circuits.get_mut(&0).unwrap();
    let num_inputs_with_hint_padded = rhl_c0.num_inputs;
    for i in 1..=num_inputs_with_hint_padded {
        rhl_c0.outputs.push(i);
    }
    // compile step 2
    let en0 = r_hint_less.expected_num_output_zeroes;
    let rhl_c0 = r_hint_less.circuits.get_mut(&0).unwrap();
    let tmp = remove_duplicate(en0, &mut rhl_c0.outputs);
    let (mut r_dest_opt, hl_im) =
        compile_step_2::<C, NormalInputType>(r_hint_less, CompileOptions::default())?;
    let en0 = r_dest_opt.expected_num_output_zeroes;
    let rd_c0 = r_dest_opt.circuits.get_mut(&0).unwrap();
    add_duplicate(en0, &mut rd_c0.outputs, &tmp);
    for (i, x) in hl_im.mapping().iter().enumerate() {
        assert_eq!(i, *x);
    }
    // remove outputs that used for prevent optimization
    let rd_c0 = r_dest_opt.circuits.get_mut(&0).unwrap();
    rd_c0
        .outputs
        .truncate(rd_c0.outputs.len() - n_in - n_out - num_inputs_with_hint_padded);
    // compile step 3
    let (lc, dest_im) = crate::layering::compile(
        &r_dest_opt,
        crate::layering::CompileOptions {
            allow_input_reorder: false,
        },
    );
    for (i, x) in dest_im.mapping().iter().enumerate() {
        if i < num_inputs_with_hint_padded {
            assert_eq!(i, *x);
        } else {
            assert_eq!(*x, EMPTY);
        }
    }
    let lc = compile_step_3(lc)?;
    print_layered_circuit_stats(&lc);
    // compile step 4
    let mut tmp_im = InputMapping::new_identity(r_hint_exported.input_size());
    let mut r_hint_exported_opt = compile_step_4(r_hint_exported, &mut tmp_im)?;
    for (i, x) in tmp_im.mapping().iter().enumerate() {
        assert_eq!(i, *x);
    }
    let re_c0 = r_hint_exported_opt.circuits.get_mut(&0).unwrap();
    re_c0.outputs.truncate(off1);
    //println!("{:?}", lc_in);

    Ok(Kernel {
        witness_solver: r_hint_exported_opt,
        layered_circuit: lc,
        io_shapes: shapes.to_vec(),
        witness_solver_io: io_off,
        witness_solver_hint_input: hint_io,
        layered_circuit_input: lc_in,
    })
}

fn reorder_ir_inputs<C: Config>(
    r: &mut ir::hint_less::RootCircuit<C>,
    lc_in: &mut [LayeredCircuitInputVec],
) {
    // sort by size, pad to 2^n, then reorder
    let mut sizes: Vec<(usize, usize)> =
        lc_in.iter().enumerate().map(|(i, x)| (x.len, i)).collect();
    sizes.sort_by(|a, b| b.cmp(a));
    let r0 = r.circuits.get_mut(&0).unwrap();
    let mut var_new_id = vec![0; r0.num_inputs + 1];
    let mut var_max = 0;
    for &(size, i) in sizes.iter() {
        let n = next_power_of_two(size);
        let prev = lc_in[i].offset;
        lc_in[i].offset = var_max;
        lc_in[i].len = n;
        assert!(var_max % n == 0);
        for j in 1..=size {
            var_new_id[prev + j] = var_max + j;
        }
        var_max += n;
    }
    r0.num_inputs = var_max;
    let mut new_insns = vec![];
    for insn in r0.instructions.iter() {
        new_insns.push(insn.replace_vars(|x| var_new_id[x]));
        for _ in 0..insn.num_outputs() {
            var_max += 1;
            var_new_id.push(var_max);
        }
    }
    r0.instructions = new_insns;
    r0.constraints = r0.constraints.iter().map(|x| var_new_id[*x]).collect();
    r0.outputs = r0.outputs.iter().map(|x| var_new_id[*x]).collect();
}

// To prevent reassign_duplicate_sub_circuit_outputs being executed too many times,
// we can remove duplicate outputs from the circuit at the beginning of compile_step_X.
fn remove_duplicate(st: usize, a: &mut Vec<usize>) -> Vec<usize> {
    let mut p = Pool::new();
    let mut res = vec![];
    for i in a.iter().skip(st) {
        res.push(p.add(i));
    }
    a.resize(st + p.len(), 0);
    for i in 0..p.len() {
        a[i + st] = *p.get(i);
    }
    res
}

// Inverse of remove_duplicate
fn add_duplicate(st: usize, a: &mut Vec<usize>, b: &[usize]) {
    let mut res = vec![];
    for i in b.iter() {
        res.push(a[*i + st]);
    }
    a.resize(st + res.len(), 0);
    a[st..(res.len() + st)].copy_from_slice(&res[..]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit::config::M31Config;

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
