
use constraint_writers::debug_writer::DebugWriter;
use program_structure::program_archive::ProgramArchive;

use super::constraint_generation;

pub struct ExecutionConfig {
    pub json_constraints: String,
    pub json_substitutions: String,
    pub no_rounds: usize,
    pub flag_s: bool,
    pub flag_f: bool,
    pub flag_p: bool,
    pub flag_old_heuristics: bool,
    pub flag_verbose: bool,
    pub inspect_constraints_flag: bool,
    pub json_substitution_flag: bool,
    pub json_constraint_flag: bool,
    pub prime: String,
    pub go_folder: String,
}

pub fn execute_project(program_archive: ProgramArchive, config: ExecutionConfig) -> Result<(), ()> {
    use constraint_generation::{build_circuit, BuildConfig};
    let _debug = DebugWriter::new(config.json_constraints).unwrap();
    let build_config = BuildConfig {
        no_rounds: config.no_rounds,
        flag_json_sub: config.json_substitution_flag,
        json_substitutions: config.json_substitutions,
        flag_s: config.flag_s,
        flag_f: config.flag_f,
        flag_p: config.flag_p,
        flag_verbose: config.flag_verbose,
        inspect_constraints: config.inspect_constraints_flag,
        flag_old_heuristics: config.flag_old_heuristics,
        prime: config.prime,
        go_folder: config.go_folder,
    };
    build_circuit(program_archive, build_config)
}
