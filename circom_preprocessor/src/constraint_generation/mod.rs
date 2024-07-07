mod compute_constants;
mod environment_utils;
mod execute;
mod execution_data;
mod expression_trace;
mod go_export;

use ansi_term::Colour;
use circom_algebra::algebra::{ArithmeticError};
use compiler::hir::very_concrete_program::VCP;
use constraint_list::ConstraintList;
use constraint_writers::ConstraintExporter;
use dag::DAG;
use execution_data::executed_program::ExportResult;
use execution_data::ExecutedProgram;
use program_structure::ast::{self};
use program_structure::error_code::ReportCode;
use program_structure::error_definition::{Report, ReportCollection};
use program_structure::file_definition::FileID;
use program_structure::program_archive::ProgramArchive;
use std::rc::Rc;

pub struct BuildConfig {
    pub no_rounds: usize,
    pub flag_json_sub: bool,
    pub json_substitutions: String,
    pub flag_s: bool,
    pub flag_f: bool,
    pub flag_p: bool,
    pub flag_verbose: bool,
    pub flag_old_heuristics: bool,
    pub inspect_constraints: bool,
    pub prime: String,
    pub go_folder: String,
}

#[derive(Debug, Copy, Clone)]
pub struct FlagsExecution {
    pub verbose: bool,
    pub inspect: bool,
}

pub type ConstraintWriter = Box<dyn ConstraintExporter>;
type BuildResponse = Result<(ConstraintWriter, VCP), ()>;
pub fn build_circuit(program: ProgramArchive, config: BuildConfig) -> Result<(), ()> {
    let files = program.file_library.clone();
    let flags = FlagsExecution {
        verbose: config.flag_verbose,
        inspect: config.inspect_constraints,
    };
    let (exe, _warnings) = instantiation(&program, flags, &config.prime).map_err(|r| {
        Report::print_reports(&r, &files);
    })?;
    /*for (v, k) in exe.trace_registry.vec.iter().enumerate() {
        println!("{:?} = {:?}", v, k)
    }
    for tp in exe.model.iter() {
        //println!("{:?} {:?}", tp.report_name, tp.trace_constraints);
        println!("{:?} {:?}", tp.template_name, tp.trace_constraints);
        for (k, v) in tp.final_components.iter() {
            println!("component {:?} = {:?}", k, v);
        }
        for (k, v) in tp.final_signal_traces.iter() {
            println!("signal {:?} = {:?}", k, v);
        }
        for k in tp.trace_constraints.iter() {
            println!("constraint {:?}", k);
        }
        println!("{:?}", tp.components);
    }
    println!("sizes {:?} {:?}", exe.model.len(), exe.model_pretemplates.len());*/

    use std::path::Path;
    let go_folder_path = Path::new(&config.go_folder).to_path_buf();
    go_export::export_go(
        &exe,
        &go_folder_path,
        program.get_public_inputs_main_component(),
    )
    .map_err(|_| {})
}

type InstantiationResponse = Result<(ExecutedProgram, ReportCollection), ReportCollection>;
fn instantiation(
    program: &ProgramArchive,
    flags: FlagsExecution,
    prime: &String,
) -> InstantiationResponse {
    let execution_result = execute::constraint_execution(&program, flags, prime);
    match execution_result {
        Ok((program_exe, warnings)) => {
            let no_nodes = program_exe.number_of_nodes();
            let success = Colour::Green.paint("template instances");
            let nodes_created = format!("{}: {}", success, no_nodes);
            println!("{}", &nodes_created);
            InstantiationResponse::Ok((program_exe, warnings))
        }
        Err(reports) => InstantiationResponse::Err(reports),
    }
}

fn export(exe: ExecutedProgram, program: ProgramArchive, flags: FlagsExecution) -> ExportResult {
    let exported = exe.export(program, flags);
    exported
}

fn sync_dag_and_vcp(vcp: &mut VCP, dag: &mut DAG) {
    let witness = Rc::new(DAG::produce_witness(dag));
    VCP::add_witness_list(vcp, Rc::clone(&witness));
}

fn simplification_process(vcp: &mut VCP, dag: DAG, config: &BuildConfig) -> ConstraintList {
    use dag::SimplificationFlags;
    let flags = SimplificationFlags {
        flag_s: config.flag_s,
        parallel_flag: config.flag_p,
        port_substitution: config.flag_json_sub,
        json_substitutions: config.json_substitutions.clone(),
        no_rounds: config.no_rounds,
        flag_old_heuristics: config.flag_old_heuristics,
        prime: config.prime.clone(),
    };
    let list = DAG::map_to_list(dag, flags);
    VCP::add_witness_list(vcp, Rc::new(list.get_witness_as_vec()));
    list
}
