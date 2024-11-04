use arith::Field;
use expander_compiler::frontend::*;
use extra::Serde;
use rand::{thread_rng, Rng};

const KEY_LEN: usize = 3;
const N_TABLE_ROWS: usize = 17;
const N_COLUMNS: usize = 5;
const N_QUERIES: usize = 33;

declare_circuit!(Circuit {
    table_keys: [[Variable; KEY_LEN]; N_TABLE_ROWS],
    table_values: [[Variable; N_COLUMNS]; N_TABLE_ROWS],

    query_keys: [[Variable; KEY_LEN]; N_QUERIES],
    query_results: [[Variable; N_COLUMNS]; N_QUERIES],

    // counting the number of occurences for each row of the table
    query_count: [Variable; N_TABLE_ROWS],
});

#[derive(Clone, Copy)]
struct Rational {
    numerator: Variable,
    denominator: Variable,
}

fn add_rational<C: Config>(builder: &mut API<C>, v1: &Rational, v2: &Rational) -> Rational {
    let p1 = builder.mul(v1.numerator, v2.denominator);
    let p2 = builder.mul(v1.denominator, v2.numerator);

    Rational {
        numerator: builder.add(p1, p2),
        denominator: builder.mul(v1.denominator, v2.denominator),
    }
}

fn assert_eq_rational<C: Config>(builder: &mut API<C>, v1: &Rational, v2: &Rational) {
    let p1 = builder.mul(v1.numerator, v2.denominator);
    let p2 = builder.mul(v1.denominator, v2.numerator);
    builder.assert_is_equal(p1, p2);
}

fn sum_rational_vec<C: Config>(builder: &mut API<C>, vs: &[Rational]) -> Rational {
    if vs.is_empty() {
        return Rational {
            numerator: builder.constant(0),
            denominator: builder.constant(1),
        };
    }

    // Basic version:
    // let mut sum = Rational {
    //     numerator: builder.constant(0),
    //     denominator: builder.constant(1),
    // };
    // for i in 0..vs.len() {
    //     sum = add_rational(builder, &sum, &vs[i]);
    // }
    // sum

    // Fewer-layers version:
    let mut vvs = vs.to_owned();
    let mut n_values_to_sum = vvs.len();
    while n_values_to_sum > 1 {
        let half_size_floor = n_values_to_sum / 2;
        for i in 0..half_size_floor {
            vvs[i] = add_rational(builder, &vvs[i], &vvs[i + half_size_floor])
        }

        if n_values_to_sum & 1 != 0 {
            vvs[half_size_floor] = vvs[n_values_to_sum - 1];
        }

        n_values_to_sum = (n_values_to_sum + 1) / 2;
    }
    vvs[0]
}

// TODO: Add poly randomness
fn get_column_randomness<C: Config>(builder: &mut API<C>, n_columns: usize) -> Vec<Variable> {
    let mut randomness = vec![];
    randomness.push(builder.constant(1));
    for _ in 1..n_columns {
        randomness.push(builder.get_random_value());
    }
    randomness
}

fn combine_columns<C: Config>(
    builder: &mut API<C>,
    vec_2d: &Vec<Vec<Variable>>,
    randomness: &[Variable],
) -> Vec<Variable> {
    if vec_2d.is_empty() {
        return vec![];
    }

    let column_size = vec_2d[0].len();
    assert!(randomness.len() == column_size);
    vec_2d
        .iter()
        .map(|row| {
            row.iter()
                .zip(randomness)
                .fold(builder.constant(0), |acc, (v, r)| {
                    let prod = builder.mul(v, r);
                    builder.add(acc, prod)
                })
        })
        .collect()
}

fn logup_poly_val<C: Config>(
    builder: &mut API<C>,
    vals: &[Variable],
    counts: &[Variable],
    x: &Variable,
) -> Rational {
    let poly_terms = vals
        .iter()
        .zip(counts)
        .map(|(v, c)| Rational {
            numerator: *c,
            denominator: builder.sub(x, v),
        })
        .collect::<Vec<Rational>>();
    sum_rational_vec(builder, &poly_terms)
}

impl<C: Config> Define<C> for Circuit<Variable> {
    fn define(&self, builder: &mut API<C>) {
        let alpha = builder.get_random_value();
        let randomness = get_column_randomness(builder, KEY_LEN + N_COLUMNS);

        let table_combined = combine_columns(
            builder,
            &self
                .table_keys
                .iter()
                .zip(self.table_values)
                .map(|(row_key, row_value)| [row_key.to_vec(), row_value.to_vec()].concat())
                .collect(),
            &randomness,
        );
        let v_table = logup_poly_val(builder, &table_combined, &self.query_count, &alpha);

        let query_combined = combine_columns(
            builder,
            &self
                .query_keys
                .iter()
                .zip(self.query_results)
                .map(|(row_key, row_value)| [row_key.to_vec(), row_value.to_vec()].concat())
                .collect(),
            &randomness,
        );
        let one = builder.constant(1);
        let v_query = logup_poly_val(
            builder,
            &query_combined,
            &vec![one; query_combined.len()],
            &alpha,
        );

        assert_eq_rational(builder, &v_table, &v_query);
    }
}

#[inline]
fn gen_assignment<C: Config>() -> Circuit<C::CircuitField> {
    let mut circuit = Circuit::<C::CircuitField>::default();
    let mut rng = thread_rng();
    for i in 0..N_TABLE_ROWS {
        for j in 0..KEY_LEN {
            circuit.table_keys[i][j] = C::CircuitField::random_unsafe(&mut rng);
        }

        for j in 0..N_COLUMNS {
            circuit.table_values[i][j] = C::CircuitField::random_unsafe(&mut rng);
        }
    }

    circuit.query_count = [C::CircuitField::ZERO; N_TABLE_ROWS];
    for i in 0..N_QUERIES {
        let query_id: usize = rng.gen::<usize>() % N_TABLE_ROWS;
        circuit.query_count[query_id] += C::CircuitField::ONE;
        circuit.query_keys[i] = circuit.table_keys[query_id];
        circuit.query_results[i] = circuit.table_values[query_id];
    }

    circuit
}

fn logup_test_helper<C: Config>() {
    let compile_result: CompileResult<C> = compile(&Circuit::default()).unwrap();
    let assignment = gen_assignment::<C>();
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

#[test]
fn logup_test() {
    logup_test_helper::<GF2Config>();
    logup_test_helper::<M31Config>();
    logup_test_helper::<BN254Config>();
}
