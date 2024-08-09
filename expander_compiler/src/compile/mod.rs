use crate::{
    builder,
    circuit::{config::Config, input_mapping::InputMapping, ir, layered},
    layering,
};

#[cfg(test)]
mod tests;

fn optimize_until_fixed_point<T, F>(x: &T, im: &mut InputMapping, f: F) -> T
where
    T: Clone + Eq,
    F: Fn(&T) -> (T, InputMapping),
{
    let (mut y, imy) = f(x);
    if *x == y {
        return y;
    }
    im.compose_in_place(&imy);
    loop {
        let (z, imz) = f(&y);
        if y == z {
            return y;
        }
        y = z;
        im.compose_in_place(&imz);
    }
}

pub fn compile<C: Config>(
    r_source: &ir::source::RootCircuit<C>,
) -> Result<(ir::hint_normalized::RootCircuit<C>, layered::Circuit<C>), String> {
    r_source.validate()?;

    let mut src_im = InputMapping::new_identity(r_source.input_size());

    let r_source_opt = optimize_until_fixed_point(r_source, &mut src_im, |r| {
        let (mut r, im) = r.remove_unreachable();
        r.reassign_duplicate_sub_circuit_outputs();
        (r, im)
    });

    let r_hint_normalized = builder::hint_normalize::process(&r_source_opt)?;
    r_hint_normalized.validate()?;

    let r_hint_normalized_opt = optimize_until_fixed_point(&r_hint_normalized, &mut src_im, |r| {
        let (mut r, im) = r.remove_unreachable();
        r.reassign_duplicate_sub_circuit_outputs();
        (r, im)
    });

    let (r_hint_less, mut r_hint_exported) = r_hint_normalized_opt.remove_and_export_hints();
    r_hint_less.validate()?;
    r_hint_exported.validate()?;

    let mut hl_im = InputMapping::new_identity(r_hint_less.input_size());

    let r_hint_less_opt = optimize_until_fixed_point(&r_hint_less, &mut hl_im, |r| {
        let (mut r, im) = r.remove_unreachable();
        r.reassign_duplicate_sub_circuit_outputs();
        (r, im)
    });

    let r_dest_relaxed = builder::final_build::process(&r_hint_less_opt)?;
    r_dest_relaxed.validate()?;

    let r_dest_relaxed_opt = optimize_until_fixed_point(&r_dest_relaxed, &mut hl_im, |r| {
        let (mut r, im) = r.remove_unreachable();
        r.reassign_duplicate_sub_circuit_outputs();
        (r, im)
    });

    let r_dest = r_dest_relaxed_opt.adjust_for_layering();
    r_dest.validate()?;

    let r_dest_opt = optimize_until_fixed_point(&r_dest, &mut hl_im, |r| {
        let (mut r, im) = r.remove_unreachable();
        r.reassign_duplicate_sub_circuit_outputs();
        (r, im)
    });

    let (lc, dest_im) = layering::compile(&r_dest_opt);
    lc.validate()?;

    // TODO: optimize lc

    hl_im.compose_in_place(&dest_im);

    let rhe_c0 = r_hint_exported.circuits.get_mut(&0).unwrap();
    rhe_c0.outputs = hl_im
        .map_inputs(&rhe_c0.outputs)
        .iter()
        .map(|&x| x.max(1))
        .collect();
    r_hint_exported.validate()?;

    let mut r_hint_exported_opt = optimize_until_fixed_point(&r_hint_exported, &mut src_im, |r| {
        let (r, im) = r.remove_unreachable();
        (r, im)
    });
    r_hint_exported_opt.add_back_removed_inputs(&src_im);

    Ok((r_hint_exported_opt, lc))
}
