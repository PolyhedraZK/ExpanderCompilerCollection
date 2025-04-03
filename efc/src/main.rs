use std::env;

use efc::end2end::{
    debug_eval_all_assignments, end2end_prepare_solver, end2end_witnesses_from_beacon_data,
};

fn main() {
    let args: Vec<String> = env::args().collect();
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug) // set global log level to debug
        .init();
    if let Some(p_index) = args.iter().position(|x| x == "-p") {
        if let Some(p_arg) = args.get(p_index + 1) {
            println!("Preparing solver for: {}", p_arg);
            end2end_prepare_solver(p_arg);
            return;
        } else {
            println!("Missing value for -p argument");
            return;
        }
    }
    if let Some(d_index) = args.iter().position(|x| x == "-d") {
        if let Some(d_arg) = args.get(d_index + 1) {
            match d_arg.parse::<u64>() {
                Ok(epoch) => {
                    println!("Debug all circuits on epoch: {}", epoch);
                    debug_eval_all_assignments(epoch);
                    return;
                }
                Err(_) => {
                    println!("Invalid number provided for -d argument");
                    return;
                }
            }
        } else {
            println!("Missing value for -p argument");
            return;
        }
    }
    if let (Some(s_index), Some(e_index), Some(m_index)) = (
        args.iter().position(|x| x == "-s"),
        args.iter().position(|x| x == "-e"),
        args.iter().position(|x| x == "-m"),
    ) {
        match (
            args.get(s_index + 1),
            args.get(e_index + 1),
            args.get(m_index + 1),
        ) {
            (Some(stage), Some(epoch_str), Some(mpi_config_str)) => {
                match epoch_str.parse::<u64>() {
                    Ok(epoch) => {
                        println!(
                            "Running stage: {} at epoch: {}, with mpi_config: {}",
                            stage, epoch, mpi_config_str
                        );
                        let mpi_config: Vec<usize> = mpi_config_str
                            .split(',')
                            .filter_map(|x| x.parse().ok())
                            .collect();

                        end2end_witnesses_from_beacon_data(epoch, stage, &mpi_config);
                    }
                    Err(_) => println!("Epoch must be a valid number"),
                }
            }
            _ => println!("Missing value for -s or -e or -m"),
        }
    } else {
        println!("Missing -s or -e argument");
    }
}
