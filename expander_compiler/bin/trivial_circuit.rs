//! This module generate a trivial GKR layered circuit for test purpose.
//! Arguments:
//! - field: field identifier
//! - n_var: number of variables
//! - n_layer: number of layers

use ark_std::test_rng;
use clap::Parser;
use expander_compiler::field::Field;
use expander_compiler::frontend::{compile, BN254Config, CompileResult, Define, M31Config};
use expander_compiler::utils::serde::Serde;
use expander_compiler::{
    declare_circuit,
    frontend::{BasicAPI, Config, Variable, API},
};

/// Arguments for the command line
/// - field: field identifier
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Field Identifier: bn254, m31ext3, gf2ext128
    #[arg(short, long,default_value_t = String::from("bn254"))]
    field: String,
}

// this cannot be too big as we currently uses static array of size 2^LOG_NUM_VARS
const LOG_NUM_VARS: usize = 15;
const NUM_LAYERS: usize = 1;

fn main() {
    let args = Args::parse();
    print_info(&args);

    match args.field.as_str() {
        "bn254" => build::<BN254Config>(),
        "m31ext3" => build::<M31Config>(),
        _ => panic!("Unsupported field"),
    }
}

fn build<C: Config>() {
    let assignment = TrivialCircuit::<C::CircuitField>::random_witnesses();

    let compile_result = compile::<C, _>(&TrivialCircuit::default()).unwrap();

    let CompileResult {
        witness_solver,
        layered_circuit,
    } = compile_result;

    let witness = witness_solver.solve_witness(&assignment).unwrap();
    let res = layered_circuit.run(&witness);

    assert_eq!(res, vec![true]);

    let file = std::fs::File::create("circuit.txt").unwrap();
    let writer = std::io::BufWriter::new(file);
    layered_circuit.serialize_into(writer).unwrap();

    let file = std::fs::File::create("witness.txt").unwrap();
    let writer = std::io::BufWriter::new(file);
    witness.serialize_into(writer).unwrap();
}

fn print_info(args: &Args) {
    println!("===============================");
    println!("Gen circuit for {} field", args.field);
    println!("Log Num of variables: {}", LOG_NUM_VARS);
    println!("Number of layers:     {}", NUM_LAYERS);
    println!("===============================")
}

declare_circuit!(TrivialCircuit {
    input_layer: [Variable; 1 << LOG_NUM_VARS],
    output_layer: [Variable; 1 << LOG_NUM_VARS],
});

impl<C: Config> Define<C> for TrivialCircuit<Variable> {
    fn define(&self, builder: &mut API<C>) {
        let out = compute_output::<C>(builder, &self.input_layer);
        out.iter().zip(self.output_layer.iter()).for_each(|(x, y)| {
            builder.assert_is_equal(x, y);
        });
    }
}

fn compute_output<C: Config>(
    api: &mut API<C>,
    input_layer: &[Variable; 1 << LOG_NUM_VARS],
) -> [Variable; 1 << LOG_NUM_VARS] {
    let mut cur_layer = *input_layer;
    for _ in 1..NUM_LAYERS {
        let mut next_layer = [Variable::default(); 1 << LOG_NUM_VARS];
        for i in 0..(1 << (LOG_NUM_VARS - 1)) {
            next_layer[i << 1] = api.add(cur_layer[i << 1], cur_layer[(i << 1) + 1]);
            next_layer[(i << 1) + 1] = api.mul(cur_layer[i << 1], cur_layer[(i << 1) + 1]);
        }
        cur_layer = next_layer;
    }
    cur_layer
}

impl<T: Field> TrivialCircuit<T> {
    fn random_witnesses() -> Self {
        let mut rng = test_rng();

        let mut input_layer = [T::default(); 1 << LOG_NUM_VARS];
        input_layer
            .iter_mut()
            .for_each(|x| *x = T::random_unsafe(&mut rng));

        let mut cur_layer = input_layer;
        for _ in 1..NUM_LAYERS {
            let mut next_layer = [T::default(); 1 << LOG_NUM_VARS];
            for i in 0..1 << (LOG_NUM_VARS - 1) {
                next_layer[i << 1] = cur_layer[i << 1] + cur_layer[(i << 1) + 1];
                next_layer[(i << 1) + 1] = cur_layer[i << 1] * cur_layer[(i << 1) + 1];
            }
            cur_layer = next_layer;
        }
        Self {
            input_layer,
            output_layer: cur_layer,
        }
    }
}
