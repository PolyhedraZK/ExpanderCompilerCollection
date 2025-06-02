use crate::circuit::{config::Config, ir};
use crate::compile::compile_step_1;
use crate::frontend::{BasicAPI, Error, Variable, API};
pub use macros::kernel;

use serdes::ExpSerde;

#[derive(Debug, Clone, Hash, PartialEq, Eq, ExpSerde)]
pub struct KernelPrimitive<C: Config> {
    // The circuit IR for output computation and later compilation
    pub ir: ir::hint_normalized::RootCircuit<C>,
    pub ir_input_offsets: Vec<usize>,
    pub ir_expected_output_offsets: Vec<usize>,

    pub io_specs: Vec<IOVecSpec>,
    pub io_shapes: Vec<Shape>,
}

pub type Shape = Vec<usize>;

pub fn shape_prepend(shape: &Shape, x: usize) -> Shape {
    let mut shape = shape.clone();
    shape.insert(0, x);
    shape
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, ExpSerde)]
pub struct IOVecSpec {
    pub len: usize,
    pub is_input: bool,
    pub is_output: bool,
}

pub fn compile_with_spec_and_shapes<C, F>(
    f: F,
    io_specs: &[IOVecSpec],
    shapes: &[Vec<usize>],
) -> Result<KernelPrimitive<C>, Error>
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
    for spec in io_specs {
        let mut cur_inputs = vec![];
        if spec.is_input {
            for i in 0..spec.len {
                cur_inputs.push(input_variables[global_input_offset + i]);
            }
            inputs_offsets.push(global_input_offset);
            global_input_offset += spec.len;
        } else {
            for _ in 0..spec.len {
                cur_inputs.push(root_builder.constant(0));
            }
            inputs_offsets.push(global_input_offset);
        }
        io_vars.push(cur_inputs);
    }
    inputs_offsets.push(global_input_offset);
    let n_in = global_input_offset;
    for spec in io_specs {
        if spec.is_output {
            let mut cur_outputs = vec![];
            for i in 0..spec.len {
                cur_outputs.push(input_variables[global_input_offset + i]);
            }
            expected_outputs.push(cur_outputs);
            expected_outputs_offsets.push(global_input_offset);
            global_input_offset += spec.len;
        } else {
            expected_outputs.push(vec![]);
            expected_outputs_offsets.push(global_input_offset);
        }
    }
    expected_outputs_offsets.push(global_input_offset);
    f(&mut root_builder, &mut io_vars);
    let mut output_vars = vec![];
    for i in 1..=n_in {
        output_vars.push(i);
    }
    for (i, spec) in io_specs.iter().enumerate() {
        if spec.is_output {
            for (x, y) in io_vars[i].iter().zip(expected_outputs[i].iter()) {
                root_builder.assert_is_equal(x, y);
                output_vars.push(x.id());
            }
        }
    }
    let mut r_source = root_builder.build();
    assert_eq!(r_source.circuits[&0].outputs.len(), 0);
    r_source.circuits.get_mut(&0).unwrap().outputs = output_vars.clone();
    let (r, src_im) = compile_step_1(&r_source)?;
    for (i, x) in src_im.mapping().iter().enumerate() {
        assert_eq!(*x, i);
    }

    Ok(KernelPrimitive {
        ir: r,
        ir_input_offsets: inputs_offsets,
        ir_expected_output_offsets: expected_outputs_offsets,
        io_specs: io_specs.to_vec(),
        io_shapes: shapes.to_vec(),
    })
}
