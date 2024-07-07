use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use super::execution_data::ExecutedProgram;

struct GoExporter {
    inputs: Vec<(Vec<String>, HashMap<String, usize>)>,
    outputs: Vec<(Vec<String>, HashMap<String, usize>)>,
    func_name: Vec<String>,
}

fn recursive_flatten(
    symbol: String,
    dim: &Vec<usize>,
    cur: usize,
    res_vec: &mut Vec<String>,
    res_map: &mut HashMap<String, usize>,
) {
    if cur == dim.len() {
        res_map.insert(symbol.clone(), res_vec.len());
        res_vec.push(symbol);
        return;
    }
    for i in 0..dim[cur] {
        recursive_flatten(format!("{}[{}]", symbol, i), dim, cur + 1, res_vec, res_map);
    }
}

fn flatten_collector(c: &Vec<(String, Vec<usize>)>) -> (Vec<String>, HashMap<String, usize>) {
    let mut res_vec = Vec::new();
    let mut res_map = HashMap::new();
    for (x, y) in c.iter() {
        recursive_flatten(x.clone(), y, 0, &mut res_vec, &mut res_map)
    }
    (res_vec, res_map)
}

fn export_go_func(exporter: &GoExporter, program: &ExecutedProgram, pointer: usize) -> String {
    use super::expression_trace::TraceItem::*;
    let template = &program.model[pointer];
    let mut res: Vec<String> = Vec::new();

    let (components, components_index) = flatten_collector(&template.components);
    let mut component_of_signal: HashMap<String, (String, String)> = HashMap::new();
    let mut used = vec![false; program.trace_registry.vec.len()];
    for component in components.iter() {
        let component_pointer = template.final_components[component];
        for out in exporter.outputs[component_pointer].0.iter() {
            component_of_signal.insert(
                format!("{}.{}", component, out),
                (component.clone(), out.clone()),
            );
        }
        for input in exporter.inputs[component_pointer].0.iter() {
            used[template.final_signal_traces[&format!("{}.{}", component, input)]] = true;
        }
    }
    for constraint in template.trace_constraints.iter() {
        used[*constraint] = true;
    }
    for symbol in exporter.outputs[pointer].0.iter() {
        //println!("{} {}", template.template_name, symbol);
        used[template.final_signal_traces[symbol]] = true;
    }

    for (i, trace_item) in program.trace_registry.vec.iter().enumerate().rev() {
        if !used[i] {
            continue;
        }
        match trace_item {
            Number { value: _ } => {}
            Signal { symbol: _ } => {}
            InfixOp { l_id, r_id, op: _ } => {
                used[*l_id] = true;
                used[*r_id] = true;
            }
            PrefixOp { id, op: _ } => {
                used[*id] = true;
            }
            InlineSwitch {
                cond,
                if_true,
                if_false,
            } => {
                used[*cond] = true;
                used[*if_true] = true;
                used[*if_false] = true;
            }
            Unknown => {
                //println!("{} {}", template.template_name, i);
                panic!(
                    "Template {} parsing failed, please open an issue.",
                    template.template_name
                );
            }
        }
    }

    let mut called: HashSet<String> = HashSet::new();
    for (i, trace_item) in program.trace_registry.vec.iter().enumerate() {
        if !used[i] {
            continue;
        }
        match trace_item {
            Number { value } => res.push(format!(
                "t{}, _ := big.NewInt(0).SetString(\"{}\", 10)",
                i, value
            )),
            Signal { symbol } => {
                if !component_of_signal.contains_key(symbol) {
                    //println!("key {}", symbol);
                    res.push(format!(
                        "t{} := inputs[{}]",
                        i, exporter.inputs[pointer].1[symbol]
                    ));
                } else {
                    let (component, csymbol) = &component_of_signal[symbol];
                    let component_index = components_index[component];
                    let component_pointer = template.final_components[component];
                    if !called.contains(component) {
                        called.insert(component.clone());
                        res.push(format!(
                            "sub_inputs{} := make([]frontend.Variable, {})",
                            component_index,
                            exporter.inputs[component_pointer].0.len()
                        ));
                        for (j, input_symbol) in
                            exporter.inputs[component_pointer].0.iter().enumerate()
                        {
                            let symbol = format!("{}.{}", component, input_symbol);
                            res.push(format!(
                                "sub_inputs{}[{}] = t{}",
                                component_index, j, template.final_signal_traces[&symbol]
                            ));
                        }
                        res.push(format!(
                            "sub_outputs{} := {}(api, sub_inputs{})",
                            component_index, exporter.func_name[component_pointer], component_index
                        ));
                    }
                    res.push(format!(
                        "t{} := sub_outputs{}[{}]",
                        i, component_index, exporter.outputs[component_pointer].1[csymbol]
                    ));
                }
            }
            InfixOp { l_id, r_id, op } => {
                use super::expression_trace::InfixOpcode::*;
                let go_op = match op {
                    Mul => "api.Mul(",
                    Div => "api.Div(",
                    Add => "api.Add(",
                    Sub => "api.Sub(",
                    _ => {
                        panic!("Infix operation \"{:?}\" on signals is not implemented yet, please open an issue.",op);
                    }
                };
                res.push(format!("t{} := {}t{}, t{})", i, go_op, l_id, r_id));
            }
            PrefixOp { id, op } => {
                use super::expression_trace::PrefixOpcode::*;
                match op {
                    Sub => {
                        res.push(format!("t{} := api.Neg(t{})", i, id));
                    }
                    _ => {
                        panic!("Prefix operation \"{:?}\" on signals is not implemented yet, please open an issue.",op);
                    }
                };
            }
            InlineSwitch {
                cond,
                if_true,
                if_false,
            } => {
                res.push(format!(
                    "t{} := api.Select(t{}, t{}, t{})",
                    i, cond, if_true, if_false
                ));
            }
            Unknown => {
                panic!("GG");
            }
        }
    }
    res.push(format!(
        "outputs := make([]frontend.Variable, {})",
        exporter.outputs[pointer].0.len()
    ));
    for (j, symbol) in exporter.outputs[pointer].0.iter().enumerate() {
        res.push(format!(
            "outputs[{}] = t{}",
            j, template.final_signal_traces[symbol]
        ));
    }

    for constraint in template.trace_constraints.iter() {
        res.push(format!("api.AssertIsEqual(t{}, 0)", constraint));
    }

    format!(
        "func {}(api frontend.API, inputs []frontend.Variable) []frontend.Variable {{\n\t{}\n\treturn outputs\n}}",
        &exporter.func_name[pointer],
        res.join("\n\t")
    )
}

fn capitalize_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn export_go_circuit_definition(
    exporter: &GoExporter,
    program: &ExecutedProgram,
    pointer: usize,
    public_inputs: &Vec<String>,
) -> String {
    let template = &program.model[pointer];
    let mut res: Vec<String> = Vec::new();
    res.push("type Circuit struct {".to_string());
    for (input, dim) in template.inputs.iter() {
        let mut dims = "".to_string();
        for x in dim.iter() {
            dims.push_str(&format!("[{}]", x));
        }
        let mut public = "".to_string();
        if public_inputs.contains(input) {
            public = " `gnark:\",public\"`".to_string();
        }
        res.push(format!(
            "\t{} {}frontend.Variable{}",
            capitalize_first(&input),
            dims,
            public,
        ));
    }
    res.push("}".to_string());
    res.push("func (c *Circuit) Define(api frontend.API) error {".to_string());
    res.push(format!(
        "\tinputs := make([]frontend.Variable, {})",
        exporter.inputs[pointer].0.len()
    ));
    for (i, input) in exporter.inputs[pointer].0.iter().enumerate() {
        res.push(format!("\tinputs[{}] = c.{}", i, capitalize_first(&input)));
    }
    res.push(format!("\t{}(api, inputs)", exporter.func_name[pointer]));
    res.push("\treturn nil".to_string());
    res.push("}".to_string());

    res.join("\n")
}

pub fn export_go(
    program: &ExecutedProgram,
    go_folder: &PathBuf,
    public_inputs: &Vec<String>,
) -> std::io::Result<()> {
    let mut exporter = GoExporter {
        inputs: Vec::new(),
        outputs: Vec::new(),
        func_name: Vec::new(),
    };
    for (i, template) in program.model.iter().enumerate() {
        exporter.inputs.push(flatten_collector(&template.inputs));
        exporter.outputs.push(flatten_collector(&template.outputs));

        let temp_name: String = if template.report_name.len() < 120 {
            template
                .report_name
                .chars()
                .map(|c| if c.is_alphanumeric() { c } else { '_' })
                .collect()
        } else {
            template.template_name.clone()
        };
        exporter
            .func_name
            .push(format!("component_{}_{}", i, temp_name));
    }
    let mut res: Vec<String> = Vec::new();
    res.push("package main\n\nimport (\n\t\"math/big\"\n".to_string());
    res.push("\t\"github.com/consensys/gnark-crypto/ecc\"".to_string());
    res.push("\t\"github.com/consensys/gnark/frontend\"".to_string());
    res.push("\t\"github.com/consensys/gnark/frontend/cs/r1cs\"".to_string());
    res.push(")\n".to_string());
    for i in 0..program.model.len() {
        res.push(export_go_func(&exporter, program, i));
    }
    res.push(export_go_circuit_definition(
        &exporter,
        program,
        program.model.len() - 1,
        public_inputs,
    ));
    res.push("func main() {\n\t_, err := frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, &Circuit{})\n\tif err != nil {\n\t\tpanic(err)\n\t}\n}".to_string());
    let code = res.join("\n");

    use std::fs::File;
    use std::io::{BufWriter, Write};
    use std::path::Path;
    if Path::new(go_folder).is_dir() {
        std::fs::remove_dir_all(go_folder)?;
    }
    std::fs::create_dir(go_folder)?;
    let mut file_path = go_folder.clone();
    file_path.push("main");
    file_path.set_extension("go");
    let file_name = file_path.to_str().unwrap();
    let mut go_file = BufWriter::new(File::create(file_name).unwrap());
    go_file.write_all(code.as_bytes())?;
    go_file.flush()?;
    Ok(())
}
