use std::{default, ops::Add};

use arith::Field;
use expander_compiler::frontend::*;
use extra::Serde;
use rand::{random, thread_rng};

const N_TABLE_ROWS: uint = 16;
const N_COLUMNS: uint = 2;
const N_QUERIES: uint = 32;

declare_circuit!(Circuit {
    table: [[Variable; N_COLUMNS]; N_TABLE_ROWS],
    
    // this will not be used for now, but later if we switch to hint, we'll need these ids
    query_ids: [Variable; N_QUERIES], 
    query_results: [Variable; N_QUERIES],

    // It would be great if we can compute this using hint, but acceptable as private input
    query_count: [Variable; N_TABLE_ROWS], 
});

#[derive(Clone)]
struct Rational {
    numerator: Variable,
    denominator: Variable,
}

fn add_rational(builder: &mut API<M31Config>, v1: &Rational, v2: &Rational) -> Rational {
    return Rational {
        numerator: builder.add(builder.mul(v1.numerator, v2.denominator), builder.mul(v1.denominator, v2.numerator)),
        denominator: builder.mul(v1.denominator, v2.denominator),
    }
}

fn sum_rational_vec(builder: &mut API<M31Config>, vs: &[Rational]) -> Rational {
    assert!(vs.len().is_power_of_two());
    let mut vs = vs.clone();
    let mut cur_output_len = vs.len() >> 1;
    while cur_output_len > 0 {
        for i in 0..cur_output_len {
            vs[i] = add_rational(builder, &vs[i], &vs[i << 1]);
        }
        cur_output_len >>= 1;
    }
    return vs[0];
}

fn combine_columns(builder: &mut API<M31Config>, vec_2d: &Vec<Vec<Variable>>, randomness: &[Variable]) -> Vec<Variable> {
    if vec_2d.is_empty() {
        return vec![];
    }

    let column_size = len(vec_2d[0]);
    assert!(randomness.len() == column_size);

    vec_2d.iter()
          .map(|column| column.iter()
            .zip(randomness).fold(0, |(acc, (v, r))| builder.add(acc, builder.mul(v, r))))
          .collect()
}

fn logup_poly_val(builder: &mut API<M31Config>, vals: &[Variable], counts: &[Variable], x: &Variable) -> Rational {
    let poly_terms = vals
    .iter().zip(counts)
    .map(|(v, c)| 
        Rational {
            numerator: c,
            denominator: builder.sub(x, v),
        })
    .collect::<Vec<Rational>>();
    sum_rational_vec(builder, &poly_terms)
}

impl Define<M31Config> for Circuit<Variable> {
    fn define(&self, builder: &mut API<M31Config>) {
        let table_combined = combine_columns(builder, self.table, &self.randomness);
        let v_left = logup_poly_val(builder, &table_combined, &self.query_count, &self.challenge);        
        
        let query_combined = combine_columns(builder, self.query_results, &self.randomness);
        let v_right = logup_poly_val(builder, &query_combined, &vec![1; query_combined.len()], &self.challenge);
        builder.assert_is_equal(
            builder.mul(v_left.numerator, v_right.denominator),
            builder.mul(v_right.numerator, v_left.denominator), 
        );
    }
}

#[inline]
fn gen_assignment() -> Circuit<M31> {
    let mut circuit = Circuit<M31>::default();
    let mut rng = thread_rng();
    for i in 0..N_TABLE_ROWS {
        for j in 0..N_COLUMNS {
            circuit.table[i][j] = M31::random_unsafe(rng);
        }
    }

    for i in 0..N_QUERIES {
        circuit.query_ids
    }

    circuit
}

#[test]
fn logup() {
    let compile_result = compile(&Circuit::default()).unwrap();
    let assignment = gen_assignment();
    let witness = compile_result
        .witness_solver
        .solve_witness(&assignment)
        .unwrap();
    let output = compile_result.layered_circuit.run(&witness);
    assert_eq!(output, vec![true]);

    let file = std::fs::File::create("circuit.txt").unwrap();
    let writer = std::io::BufWriter::new(file);
    compile_result
        .layered_circuit
        .serialize_into(writer)
        .unwrap();

    let file = std::fs::File::create("witness.txt").unwrap();
    let writer = std::io::BufWriter::new(file);
    witness.serialize_into(writer).unwrap();

    let file = std::fs::File::create("witness_solver.txt").unwrap();
    let writer = std::io::BufWriter::new(file);
    compile_result
        .witness_solver
        .serialize_into(writer)
        .unwrap();
}
