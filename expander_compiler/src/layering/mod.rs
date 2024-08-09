use std::collections::HashMap;

use crate::{
    circuit::{config::Config, input_mapping::InputMapping, ir, layered},
    utils::pool::Pool,
};

mod compile;
mod input;
mod layer_layout;
mod wire;

#[cfg(test)]
mod tests;

pub fn compile<C: Config>(rc: &ir::dest::RootCircuit<C>) -> (layered::Circuit<C>, InputMapping) {
    let mut ctx = compile::CompileContext {
        rc,
        circuits: HashMap::new(),
        order: Vec::new(),
        layer_layout_pool: Pool::new(),
        layer_req_to_layout: HashMap::new(),
        compiled_circuits: Vec::new(),
        conncected_wires: HashMap::new(),
        layout_ids: Vec::new(),
        layers: Vec::new(),
        input_order: Vec::new(),
    };
    ctx.compile();
    let l0_size = ctx.compiled_circuits[ctx.layers[0]].num_inputs;
    (
        layered::Circuit {
            segments: ctx.compiled_circuits,
            layer_ids: ctx.layers,
        },
        InputMapping::new(l0_size, ctx.input_order),
    )
}
