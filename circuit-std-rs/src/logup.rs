use arith::Field;
use expander_compiler::frontend::*;
use rand::Rng;

use crate::StdCircuit;

#[derive(Clone, Copy, Debug)]
pub struct LogUpParams {
    pub key_len: usize,
    pub value_len: usize,
    pub n_table_rows: usize,
    pub n_queries: usize,
}

declare_circuit!(_LogUpCircuit {
    table_keys: [[Variable]],
    table_values: [[Variable]],

    query_keys: [[Variable]],
    query_results: [[Variable]],

    // counting the number of occurences for each row of the table
    query_count: [Variable],
});

pub type LogUpCircuit = _LogUpCircuit<Variable>;

#[derive(Clone, Copy, Debug)]
struct Rational {
    numerator: Variable,
    denominator: Variable,
}

fn add_rational<C: Config, B: RootAPI<C>>(builder: &mut B, v1: &Rational, v2: &Rational) -> Rational {
    let p1 = builder.mul(v1.numerator, v2.denominator);
    let p2 = builder.mul(v1.denominator, v2.numerator);

    Rational {
        numerator: builder.add(p1, p2),
        denominator: builder.mul(v1.denominator, v2.denominator),
    }
}

fn assert_eq_rational<C: Config, B: RootAPI<C>>(builder: &mut B, v1: &Rational, v2: &Rational) {
    let p1 = builder.mul(v1.numerator, v2.denominator);
    let p2 = builder.mul(v1.denominator, v2.numerator);
    builder.assert_is_equal(p1, p2);
}

fn sum_rational_vec<C: Config, B: RootAPI<C>>(builder: &mut B, vs: &[Rational]) -> Rational {
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

// TODO-Feature: poly randomness
fn get_column_randomness<C: Config>(builder: &mut API<C>, n_columns: usize) -> Vec<Variable> {
    let mut randomness = vec![];
    randomness.push(builder.constant(1));
    for _ in 1..n_columns {
        randomness.push(builder.get_random_value());
    }
    randomness
}

fn concat_d1(v1: &[Vec<Variable>], v2: &[Vec<Variable>]) -> Vec<Vec<Variable>> {
    v1.iter()
        .zip(v2.iter())
        .map(|(row_key, row_value)| [row_key.to_vec(), row_value.to_vec()].concat())
        .collect()
}

fn combine_columns<C: Config>(
    builder: &mut API<C>,
    vec_2d: &[Vec<Variable>],
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

fn logup_poly_val<C: Config, B: RootAPI<C>>(
    builder: &mut B,
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

impl<C: Config> Define<C> for LogUpCircuit {
    fn define(&self, builder: &mut API<C>) {
        let key_len = self.table_keys[0].len();
        let value_len = self.table_values[0].len();

        let alpha = builder.get_random_value();
        let randomness = get_column_randomness(builder, key_len + value_len);

        let table_combined = combine_columns(
            builder,
            &concat_d1(&self.table_keys, &self.table_values),
            &randomness,
        );
        let v_table = logup_poly_val(builder, &table_combined, &self.query_count, &alpha);

        let query_combined = combine_columns(
            builder,
            &concat_d1(&self.query_keys, &self.query_results),
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

impl<C: Config> StdCircuit<C> for LogUpCircuit {
    type Params = LogUpParams;
    type Assignment = _LogUpCircuit<C::CircuitField>;

    fn new_circuit(params: &Self::Params) -> Self {
        let mut circuit = Self::default();

        circuit.table_keys.resize(
            params.n_table_rows,
            vec![Variable::default(); params.key_len],
        );
        circuit.table_values.resize(
            params.n_table_rows,
            vec![Variable::default(); params.value_len],
        );
        circuit
            .query_keys
            .resize(params.n_queries, vec![Variable::default(); params.key_len]);
        circuit.query_results.resize(
            params.n_queries,
            vec![Variable::default(); params.value_len],
        );
        circuit
            .query_count
            .resize(params.n_table_rows, Variable::default());

        circuit
    }

    fn new_assignment(params: &Self::Params, mut rng: impl rand::RngCore) -> Self::Assignment {
        let mut assignment = _LogUpCircuit::<C::CircuitField>::default();
        assignment.table_keys.resize(params.n_table_rows, vec![]);
        assignment.table_values.resize(params.n_table_rows, vec![]);
        assignment.query_keys.resize(params.n_queries, vec![]);
        assignment.query_results.resize(params.n_queries, vec![]);

        for i in 0..params.n_table_rows {
            for _ in 0..params.key_len {
                assignment.table_keys[i].push(C::CircuitField::random_unsafe(&mut rng));
            }

            for _ in 0..params.value_len {
                assignment.table_values[i].push(C::CircuitField::random_unsafe(&mut rng));
            }
        }

        assignment.query_count = vec![C::CircuitField::ZERO; params.n_table_rows];
        for i in 0..params.n_queries {
            let query_id: usize = rng.gen::<usize>() % params.n_table_rows;
            assignment.query_count[query_id] += C::CircuitField::ONE;
            assignment.query_keys[i] = assignment.table_keys[query_id].clone();
            assignment.query_results[i] = assignment.table_values[query_id].clone();
        }

        assignment
    }
}



declare_circuit!(LogUpTestCircuit {
    test: Variable
});
pub struct LogUpTable {
    pub table_keys: Vec<Vec<Variable>>,
    pub table_values: Vec<Vec<Variable>>,
    pub query_keys: Vec<Vec<Variable>>,
    pub query_results: Vec<Vec<Variable>>,
}
impl LogUpTable {
    pub fn new() -> Self {
        Self {
            table_keys: vec![],
            table_values: vec![],
            query_keys: vec![],
            query_results: vec![],
        }
    }
    pub fn add_table_row(&mut self, key: Vec<Variable>, value: Vec<Variable>) {
        self.table_keys.push(key);
        self.table_values.push(value);
    }
    pub fn add_query(&mut self, key: Vec<Variable>, result: Vec<Variable>) {
        self.query_keys.push(key);
        self.query_results.push(result);
    }
}

pub struct LogUpRangeProofTable {
    pub table_keys: Vec<Variable>,
    pub query_keys: Vec<Variable>,
    pub rangeproof_bits: usize,
}
impl LogUpRangeProofTable {
    pub fn new(nb_bits: usize) -> Self {
        Self {
            table_keys: vec![],
            query_keys: vec![],
            rangeproof_bits: nb_bits,
        }
    }
    pub fn initial<C: Config, B: RootAPI<C>>(&mut self, builder: &mut B) {
        for i in 0..1<<self.rangeproof_bits {
            let key = builder.constant(i as u32);
            self.add_table_row(key);
        }
    }
    pub fn add_table_row(&mut self, key: Variable) {
        self.table_keys.push(key);
    }
    pub fn add_query(&mut self, key: Variable) {
        self.query_keys.push(key);
    }
    pub fn rangeproof<C: Config, B: RootAPI<C>>(&mut self, builder: &mut B, a: Variable, n: usize) {
        //add a shift value
        let mut n = n;
        let mut new_a = a;
        if n % self.rangeproof_bits != 0 {
            let rem = n % self.rangeproof_bits;
            let shift = self.rangeproof_bits - rem;
            let constant = (1 << shift) - 1;
            let mut mul_factor = 1;
            println!("n:{}", n);
            mul_factor = mul_factor << n;
            let a_shift = builder.mul(constant, mul_factor);
            new_a = builder.add(a, a_shift);
            n = n + shift;
        }
        let hint_input = vec![builder.constant(n as u32), builder.constant(self.rangeproof_bits as u32), new_a];
        let witnesses = builder.new_hint("myhint.rangeproofhint", &hint_input, n / self.rangeproof_bits);
        let mut sum = witnesses[0];
        for i in 1..witnesses.len() {
            let constant = 1 << (self.rangeproof_bits * i);
            let constant = builder.constant(constant);
            let mul = builder.mul(witnesses[i], constant);
            sum = builder.add(sum, mul);
        }
        builder.assert_is_equal(sum, new_a);
        for i in 0..n / self.rangeproof_bits {
            self.query_range(witnesses[i]);
        }
    }
    pub fn rangeproof_onechunk<C: Config, B: RootAPI<C>>(&mut self, builder: &mut B, a: Variable, n: usize) {
        //n must be less than self.rangeproof_bits, not need the hint 
        if n > self.rangeproof_bits {
            panic!("n must be less than self.rangeproof_bits");
        }
        //add a shift value
        let mut new_a = a;
        if n % self.rangeproof_bits != 0 {
            let rem = n % self.rangeproof_bits;
            let shift = self.rangeproof_bits - rem;
            let constant = (1 << shift) - 1;
            let mut mul_factor = 0;
            mul_factor = mul_factor << n;
            let a_shift = builder.mul(constant, mul_factor);
            new_a = builder.add(a, a_shift);
        }
        self.query_range(a);
    }
    pub fn query_range(&mut self, key: Variable) {
        self.query_keys.push(key);
    }
    pub fn final_check<C: Config, B: RootAPI<C>>(&mut self, builder: &mut B) {
        let alpha = builder.get_random_value();
        let inputs = self.query_keys.clone();
        println!("table len: {}", self.table_keys.len());
        let query_count = builder.new_hint("myhint.querycounthint", &inputs, self.table_keys.len());
        let v_table = logup_poly_val(builder, &self.table_keys, &query_count, &alpha);

        let one = builder.constant(1);
        let v_query = logup_poly_val(
            builder,
            &self.query_keys,
            &vec![one; self.query_keys.len()],
            &alpha,
        );
        assert_eq_rational(builder, &v_table, &v_query);
    } 
}
pub fn query_count_hint(inputs: &[M31], outputs: &mut [M31]) -> Result<(), Error> {
    let mut count = vec![0; outputs.len()];
    for i in 0..inputs.len() {
        let query_id = inputs[i].to_u256().as_usize();
        count[query_id] += 1;
    }
    for i in 0..outputs.len() {
        outputs[i] = M31::from(count[i] as u32);
    }
    Ok(())
}
pub fn rangeproof_hint(inputs: &[M31], outputs: &mut [M31]) -> Result<(), Error> {
    let n = inputs[0].to_u256().as_i64();
    let m = inputs[1].to_u256().as_i64();
    let mut a = inputs[2].to_u256().as_i64();
    for i in 0..n / m {
        let r = a % (1 << m);
        // println!("r: {}", r);
        a = a / (1 << m);
        outputs[i as usize] = M31::from(r as u32);
    }
    Ok(())
}
impl GenericDefine<M31Config> for LogUpTestCircuit<Variable>  {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let mut table = LogUpRangeProofTable::new(8);
        table.initial(builder);
        for i in 1..12 {
            for j in (1 << (i - 1))..(1 << i) {
                let key = builder.constant(j);
                if i > 8 {
                    table.rangeproof(builder,key, i);
                } else {
                    table.rangeproof_onechunk(builder,key, i);
                }
            }
        }
        table.final_check(builder);
    }
}

#[test]
fn logup_test() {
    let mut hint_registry = HintRegistry::<M31>::new();
	hint_registry.register("myhint.querycounthint", query_count_hint);
	hint_registry.register("myhint.rangeproofhint", rangeproof_hint);
	//compile and test
	let compile_result = compile_generic(&LogUpTestCircuit::default(),CompileOptions::default()).unwrap();
	let assignment = LogUpTestCircuit {
        test: M31::from(0),
	};
	let witness = compile_result
		.witness_solver
		.solve_witness_with_hints(&assignment, &mut hint_registry)
			.unwrap();
	let output = compile_result.layered_circuit.run(&witness);
	assert_eq!(output, vec![true]);
}