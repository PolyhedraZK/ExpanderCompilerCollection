use std::path::PathBuf;

pub struct Input {
    pub input_program: PathBuf,
    pub out_go_folder: PathBuf,
    pub out_json_constraints: PathBuf,
    pub out_json_substitutions: PathBuf,
    pub json_constraint_flag: bool,
    pub json_substitution_flag: bool,
    pub no_rounds: usize,
    pub flag_verbose: bool,
    pub prime: String,
    pub link_libraries: Vec<PathBuf>,
}

const GO: &'static str = "go";
const JSON: &'static str = "json";

impl Input {
    pub fn new() -> Result<Input, ()> {
        use ansi_term::Colour;
        use input_processing::SimplificationStyle;
        let matches = input_processing::view();
        let input = input_processing::get_input(&matches)?;
        let mut file_name = input.file_stem().unwrap().to_str().unwrap().to_string();
        let output_path = input_processing::get_output_path(&matches)?;

        let c_flag = input_processing::get_c(&matches);

        if c_flag && (file_name == "main" || file_name == "fr" || file_name == "calcwit") {
            println!("{}", Colour::Yellow.paint(format!("The name {} is reserved in Circom when using de --c flag. The files generated for your circuit will use the name {}_c instead of {}.", file_name, file_name, file_name)));
            file_name = format!("{}_c", file_name)
        };
        let output_go_path = Input::build_folder(&output_path, &file_name, GO);
        let o_style = input_processing::get_simplification_style(&matches)?;
        let link_libraries = input_processing::get_link_libraries(&matches);
        Result::Ok(Input {
            //field: P_BN128,
            input_program: input,
            out_go_folder: output_go_path.clone(),
            out_json_constraints: Input::build_output(
                &output_path,
                &format!("{}_constraints", file_name),
                JSON,
            ),
            out_json_substitutions: Input::build_output(
                &output_path,
                &format!("{}_substitutions", file_name),
                JSON,
            ),
            json_constraint_flag: input_processing::get_json_constraints(&matches),
            json_substitution_flag: input_processing::get_json_substitutions(&matches),
            no_rounds: if let SimplificationStyle::O2(r) = o_style {
                r
            } else {
                0
            },
            flag_verbose: input_processing::get_flag_verbose(&matches),
            prime: input_processing::get_prime(&matches)?,
            link_libraries,
        })
    }

    fn build_folder(output_path: &PathBuf, filename: &str, ext: &str) -> PathBuf {
        let mut file = output_path.clone();
        let folder_name = format!("{}_{}", filename, ext);
        file.push(folder_name);
        file
    }

    fn build_output(output_path: &PathBuf, filename: &str, ext: &str) -> PathBuf {
        let mut file = output_path.clone();
        file.push(format!("{}.{}", filename, ext));
        file
    }

    pub fn get_link_libraries(&self) -> &Vec<PathBuf> {
        &self.link_libraries
    }

    pub fn input_file(&self) -> &str {
        &self.input_program.to_str().unwrap()
    }
    pub fn json_constraints_file(&self) -> &str {
        self.out_json_constraints.to_str().unwrap()
    }
    pub fn json_substitutions_file(&self) -> &str {
        self.out_json_substitutions.to_str().unwrap()
    }
    pub fn json_constraints_flag(&self) -> bool {
        self.json_constraint_flag
    }
    pub fn json_substitutions_flag(&self) -> bool {
        self.json_substitution_flag
    }
    pub fn go_folder(&self) -> &str {
        self.out_go_folder.to_str().unwrap()
    }
    pub fn flag_verbose(&self) -> bool {
        self.flag_verbose
    }
    pub fn no_rounds(&self) -> usize {
        self.no_rounds
    }
    pub fn prime(&self) -> String {
        self.prime.clone()
    }
}
mod input_processing {
    use crate::VERSION;
    use ansi_term::Colour;
    use clap::{App, Arg, ArgMatches};
    use std::path::{Path, PathBuf};

    pub fn get_input(matches: &ArgMatches) -> Result<PathBuf, ()> {
        let route = Path::new(matches.value_of("input").unwrap()).to_path_buf();
        if route.is_file() {
            Result::Ok(route)
        } else {
            let route = if route.to_str().is_some() {
                ": ".to_owned() + route.to_str().unwrap()
            } else {
                "".to_owned()
            };
            Result::Err(eprintln!(
                "{}",
                Colour::Red.paint("Input file does not exist".to_owned() + &route)
            ))
        }
    }

    pub fn get_output_path(matches: &ArgMatches) -> Result<PathBuf, ()> {
        let route = Path::new(matches.value_of("output").unwrap()).to_path_buf();
        if route.is_dir() {
            Result::Ok(route)
        } else {
            Result::Err(eprintln!("{}", Colour::Red.paint("invalid output path")))
        }
    }

    #[derive(Copy, Clone, Eq, PartialEq)]
    pub enum SimplificationStyle {
        O0,
        O1,
        O2(usize),
    }
    pub fn get_simplification_style(matches: &ArgMatches) -> Result<SimplificationStyle, ()> {
        let o_0 = matches.is_present("no_simplification");
        let o_1 = matches.is_present("reduced_simplification");
        let o_2 = matches.is_present("full_simplification");
        let o_2round = matches.is_present("simplification_rounds");
        match (o_0, o_1, o_2round, o_2) {
            (true, _, _, _) => Ok(SimplificationStyle::O0),
            (_, true, _, _) => Ok(SimplificationStyle::O1),
            (_, _, true, _) => {
                let o_2_argument = matches.value_of("simplification_rounds").unwrap();
                let rounds_r = usize::from_str_radix(o_2_argument, 10);
                if let Result::Ok(no_rounds) = rounds_r {
                    if no_rounds == 0 {
                        Ok(SimplificationStyle::O1)
                    } else {
                        Ok(SimplificationStyle::O2(no_rounds))
                    }
                } else {
                    Result::Err(eprintln!(
                        "{}",
                        Colour::Red.paint("invalid number of rounds")
                    ))
                }
            }

            (false, false, false, true) => Ok(SimplificationStyle::O2(usize::MAX)),
            (false, false, false, false) => Ok(SimplificationStyle::O2(usize::MAX)),
        }
    }

    pub fn get_json_constraints(matches: &ArgMatches) -> bool {
        matches.is_present("print_json_c")
    }

    pub fn get_json_substitutions(matches: &ArgMatches) -> bool {
        matches.is_present("print_json_sub")
    }

    pub fn get_c(matches: &ArgMatches) -> bool {
        matches.is_present("print_c")
    }

    pub fn get_flag_verbose(matches: &ArgMatches) -> bool {
        matches.is_present("flag_verbose")
    }

    pub fn get_prime(matches: &ArgMatches) -> Result<String, ()> {
        match matches.is_present("prime") {
            true => {
                let prime_value = matches.value_of("prime").unwrap();
                if prime_value == "bn128"
                    || prime_value == "bls12381"
                    || prime_value == "goldilocks"
                    || prime_value == "grumpkin"
                    || prime_value == "pallas"
                    || prime_value == "vesta"
                    || prime_value == "secq256r1"
                {
                    Ok(String::from(matches.value_of("prime").unwrap()))
                } else {
                    Result::Err(eprintln!("{}", Colour::Red.paint("invalid prime number")))
                }
            }

            false => Ok(String::from("bn128")),
        }
    }

    pub fn view() -> ArgMatches<'static> {
        App::new("circom to ECC preprocessor")
            .version(VERSION)
            .author("IDEN3 and Polyhedra")
            .about("Compiler for the circom programming language")
            .arg(
                Arg::with_name("input")
                    .multiple(false)
                    .default_value("./circuit.circom")
                    .help("Path to a circuit with a main component"),
            )
            .arg(
                Arg::with_name("no_simplification")
                    .long("O0")
                    .hidden(false)
                    .takes_value(false)
                    .help("No simplification is applied")
                    .display_order(420)
            )
            .arg(
                Arg::with_name("reduced_simplification")
                    .long("O1")
                    .hidden(false)
                    .takes_value(false)
                    .help("Only applies signal to signal and signal to constant simplification")
                    .display_order(460)
            )
            .arg(
                Arg::with_name("full_simplification")
                    .long("O2")
                    .takes_value(false)
                    .hidden(false)
                    .help("Full constraint simplification")
                    .display_order(480)
            )
            .arg(
                Arg::with_name("simplification_rounds")
                    .long("O2round")
                    .takes_value(true)
                    .hidden(false)
                    .help("Maximum number of rounds of the simplification process")
                    .display_order(500)
            )
            .arg(
                Arg::with_name("output")
                    .short("o")
                    .long("output")
                    .takes_value(true)
                    .default_value(".")
                    .display_order(1)
                    .help("Path to the directory where the output will be written"),
            )
            .arg(
                Arg::with_name("inspect_constraints")
                    .long("inspect")
                    .takes_value(false)
                    .display_order(801)
                    .help("Does an additional check over the constraints produced"),
            )
            .arg(
                Arg::with_name("link_libraries")
                .short("l")
                .takes_value(true)
                .multiple(true)
                .number_of_values(1)
                .display_order(330)
                .help("Adds directory to library search path"),
            )
            .arg(
                Arg::with_name("parallel_simplification")
                    .long("parallel")
                    .takes_value(false)
                    .hidden(true)
                    .display_order(180)
                    .help("Runs non-linear simplification in parallel"),
            )
            .arg(
                Arg::with_name("main_inputs_log")
                    .long("inputs")
                    .takes_value(false)
                    .hidden(true)
                    .display_order(210)
                    .help("Produces a log_inputs.txt file"),
            )
            .arg(
                Arg::with_name("flag_verbose")
                    .long("verbose")
                    .takes_value(false)
                    .display_order(800)
                    .help("Shows logs during compilation"),
            )
            .arg(
                Arg::with_name("flag_old_heuristics")
                    .long("use_old_simplification_heuristics")
                    .takes_value(false)
                    .display_order(980)
                    .help("Applies the old version of the heuristics when performing linear simplification"),
            )
            .arg (
                Arg::with_name("prime")
                    .short("prime")
                    .long("prime")
                    .takes_value(true)
                    .default_value("bn128")
                    .display_order(300)
                    .help("To choose the prime number to use to generate the circuit. Receives the name of the curve (bn128, bls12381, goldilocks, grumpkin, pallas, vesta, secq256r1)"),
            )
            .get_matches()
    }

    pub fn get_link_libraries(matches: &ArgMatches) -> Vec<PathBuf> {
        let mut link_libraries = Vec::new();
        let m = matches.values_of("link_libraries");
        if let Some(paths) = m {
            for path in paths.into_iter() {
                link_libraries.push(Path::new(path).to_path_buf());
            }
        }
        link_libraries
    }
}
