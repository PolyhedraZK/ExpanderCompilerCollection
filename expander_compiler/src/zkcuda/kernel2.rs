use crate::circuit::{config::Config, ir};
use crate::frontend::{BasicAPI, Error, Variable, API};
pub use macros::kernel;

use serdes::ExpSerde;

#[derive(Debug, Clone, Hash, PartialEq, Eq, ExpSerde)]
pub struct KernelPrimitive<C: Config> {
    pub ir_source: ir::source::RootCircuit<C>,
    pub io_specs: Vec<IOVecSpec>,
    pub io_shapes: Vec<Shape>,
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

#[derive(Debug, Clone, Hash, PartialEq, Eq, ExpSerde)]
pub struct IOVecSpec {
    pub len: usize,
    pub is_input: bool,
    pub is_output: bool,
}

pub fn compile_with_spec<C, F>(f: F, io_specs: &[IOVecSpec]) -> Result<KernelPrimitive<C>, Error>
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
            global_input_offset += spec.len;
        } else {
            expected_outputs.push(vec![]);
            expected_outputs_offsets.push(0);
        }
    }
    f(&mut root_builder, &mut io_vars);
    let mut output_offsets = vec![];
    let mut global_output_offset = 0;
    let mut output_vars = vec![];
    for (i, spec) in io_specs.iter().enumerate() {
        if spec.is_output {
            for (x, y) in io_vars[i].iter().zip(expected_outputs[i].iter()) {
                root_builder.assert_is_equal(x, y);
                output_vars.push(x.id());
            }
            output_offsets.push(global_output_offset);
            global_output_offset += spec.len;
        } else {
            output_offsets.push(0);
        }
    }
    let mut r_source = root_builder.build();
    assert_eq!(r_source.circuits[&0].outputs.len(), 0);
    r_source.circuits.get_mut(&0).unwrap().outputs = output_vars.clone();

    Ok(KernelPrimitive {
        ir_source: r_source,
        io_specs: io_specs.to_vec(),
        io_shapes: shapes.to_vec(),
    })
}
