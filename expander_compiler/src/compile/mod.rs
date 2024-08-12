use crate::{
    builder,
    circuit::{config::Config, input_mapping::InputMapping, ir, layered},
    layering,
    utils::error::Error,
};

#[cfg(test)]
mod random_circuit_tests;
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

fn print_info(info: &str) {
    print!(
        "\x1b[90m{}\x1b[0m \x1b[32mINF\x1b[0m {} ",
        chrono::Local::now().format("%H:%M:%S").to_string(),
        info
    );
}

fn print_stat(stat_name: &str, stat: usize, is_last: bool) {
    print!("\x1b[36m{}=\x1b[0m{}", stat_name, stat);
    if !is_last {
        print!(" ");
    } else {
        println!();
    }
}

pub fn compile<C: Config>(
    r_source: &ir::source::RootCircuit<C>,
) -> Result<(ir::hint_normalized::RootCircuit<C>, layered::Circuit<C>), Error> {
    r_source.validate()?;

    let mut src_im = InputMapping::new_identity(r_source.input_size());

    let r_source_opt = optimize_until_fixed_point(r_source, &mut src_im, |r| {
        let (mut r, im) = r.remove_unreachable();
        r.reassign_duplicate_sub_circuit_outputs();
        (r, im)
    });

    let r_hint_normalized = builder::hint_normalize::process(&r_source_opt)
        .map_err(|e| e.prepend("hint normalization failed"))?;
    r_hint_normalized
        .validate()
        .map_err(|e| e.prepend("hint normalized circuit invalid"))?;

    let r_hint_normalized_opt = optimize_until_fixed_point(&r_hint_normalized, &mut src_im, |r| {
        let (mut r, im) = r.remove_unreachable();
        r.reassign_duplicate_sub_circuit_outputs();
        (r, im)
    });
    let ho_stats = r_hint_normalized_opt.get_stats();
    print_info("built hint normalized ir");
    print_stat("numInputs", ho_stats.num_inputs, false);
    print_stat("numConstraints", ho_stats.num_constraints, false);
    print_stat("numInsns", ho_stats.num_insns, false);
    print_stat("numVars", ho_stats.num_variables, false);
    print_stat("numTerms", ho_stats.num_terms, true);

    let (r_hint_less, mut r_hint_exported) = r_hint_normalized_opt.remove_and_export_hints();
    r_hint_less
        .validate()
        .map_err(|e| e.prepend("hint less circuit invalid"))?;
    r_hint_exported
        .validate()
        .map_err(|e| e.prepend("hint exported circuit invalid"))?;

    let mut hl_im = InputMapping::new_identity(r_hint_less.input_size());

    let r_hint_less_opt = optimize_until_fixed_point(&r_hint_less, &mut hl_im, |r| {
        let (mut r, im) = r.remove_unreachable();
        r.reassign_duplicate_sub_circuit_outputs();
        (r, im)
    });

    let r_dest_relaxed = builder::final_build::process(&r_hint_less_opt)
        .map_err(|e| e.prepend("final build failed"))?;
    r_dest_relaxed
        .validate()
        .map_err(|e| e.prepend("final build circuit invalid"))?;

    let r_dest_relaxed_opt = optimize_until_fixed_point(&r_dest_relaxed, &mut hl_im, |r| {
        let (mut r, im) = r.remove_unreachable();
        r.reassign_duplicate_sub_circuit_outputs();
        (r, im)
    });

    let r_dest = r_dest_relaxed_opt.adjust_for_layering();
    r_dest
        .validate()
        .map_err(|e| e.prepend("layering circuit invalid"))?;

    let r_dest_opt = optimize_until_fixed_point(&r_dest, &mut hl_im, |r| {
        let (mut r, im) = r.remove_unreachable();
        r.reassign_duplicate_sub_circuit_outputs();
        (r, im)
    });

    let (lc, dest_im) = layering::compile(&r_dest_opt);
    lc.validate()
        .map_err(|e| e.prepend("layered circuit invalid"))?;

    // TODO: optimize lc

    let lc_stats = lc.get_stats();
    print_info("built layered circuit");
    print_stat("numSegment", lc_stats.num_segments, false);
    print_stat("numLayer", lc_stats.num_layers, false);
    print_stat("numUsedInputs", lc_stats.num_inputs, false);
    print_stat("numUsedVariables", lc_stats.num_used_gates, false);
    print_stat("numVariables", lc_stats.num_total_gates, false);
    print_stat("numAdd", lc_stats.num_expanded_add, false);
    print_stat("numCst", lc_stats.num_expanded_cst, false);
    print_stat("numMul", lc_stats.num_expanded_mul, true);

    hl_im.compose_in_place(&dest_im);

    let rhe_c0 = r_hint_exported.circuits.get_mut(&0).unwrap();
    rhe_c0.outputs = hl_im
        .map_inputs(&rhe_c0.outputs)
        .iter()
        .map(|&x| x.max(1))
        .collect();
    r_hint_exported
        .validate()
        .map_err(|e| e.prepend("final hint exported circuit invalid"))?;

    let mut r_hint_exported_opt = optimize_until_fixed_point(&r_hint_exported, &mut src_im, |r| {
        let (r, im) = r.remove_unreachable();
        (r, im)
    });
    r_hint_exported_opt.add_back_removed_inputs(&src_im);

    Ok((r_hint_exported_opt, lc))
}
