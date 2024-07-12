extern crate num_bigint_dig as num_bigint;
extern crate num_traits;

mod constraint_generation;
mod execution_user;
mod input_user;
mod parser_user;
mod type_analysis_user;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

use ansi_term::Colour;
use input_user::Input;
fn main() {
    let result = start();
    if result.is_err() {
        eprintln!("{}", Colour::Red.paint("previous errors were found"));
        std::process::exit(1);
    } else {
        println!("{}", Colour::Green.paint("Everything went okay"));
        //std::process::exit(0);
    }
}

fn start() -> Result<(), ()> {
    use execution_user::ExecutionConfig;
    let user_input = Input::new()?;
    let mut program_archive = parser_user::parse_project(&user_input)?;
    type_analysis_user::analyse_project(&mut program_archive)?;

    let config = ExecutionConfig {
        no_rounds: user_input.no_rounds(),
        flag_p: false,
        flag_s: false,
        flag_f: true,
        flag_old_heuristics: false,
        flag_verbose: user_input.flag_verbose(),
        inspect_constraints_flag: false,
        json_constraint_flag: user_input.json_constraints_flag(),
        json_substitution_flag: user_input.json_substitutions_flag(),
        json_constraints: user_input.json_constraints_file().to_string(),
        json_substitutions: user_input.json_substitutions_file().to_string(),
        prime: user_input.prime(),
        go_folder: user_input.go_folder().to_string(),
    };
    execution_user::execute_project(program_archive, config)
}
