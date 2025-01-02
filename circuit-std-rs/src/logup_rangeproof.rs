
declare_circuit!(_LogUpTestCircuit {
    test: Variable
});
pub type LogUpTestCircuit = _LogUpTestCircuit<Variable>;
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
    pub fn add_table_row(&mut self, key: Variable) {
        self.table_keys.push(key);
    }
    pub fn add_query(&mut self, key: Variable) {
        self.query_keys.push(key);
    }
    pub fn range_proof_onechunk<C: Config>(&mut self, builder: &mut API<C>, a: Variable, n: usize) {
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
        self.query_range(builder, a);
    }
    pub fn query_range<C: Config>(&mut self, builder: &mut API<C>, key: Variable) {
        if self.table_keys.len() != 1 {
            panic!("table should have only one column");
        }
        self.query_keys.push(key);
    }
    pub fn final_check<C: Config>(&mut self, builder: &mut API<C>) {
        let alpha = builder.get_random_value();
        let inputs = self.query_keys.clone();
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
/*
func QueryCountHintFn(field *big.Int, inputs []*big.Int, outputs []*big.Int) error {
	for i := 0; i < len(outputs); i++ {
		outputs[i] = big.NewInt(0)
	}

	for i := 0; i < len(inputs); i++ {
		query_id := inputs[i].Int64()
		outputs[query_id].Add(outputs[query_id], big.NewInt(1))
	}
	return nil
}
*/
pub fn query_count_hint_fn(inputs: &[M31], outputs: &mut [M31]) -> Result<(), Error> {
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
impl<C: Config> Define<C> for LogUpTestCircuit {
    fn define(&self, builder: &mut API<C>) {
        // let key_len = self.table_keys[0].len();
        // let value_len = self.table_values[0].len();

        // let alpha = builder.get_random_value();
        // let randomness = get_column_randomness(builder, key_len + value_len);

        // let table_combined = combine_columns(
        //     builder,
        //     &concat_d1(&self.table_keys, &self.table_values),
        //     &randomness,
        // );
        // let v_table = logup_poly_val(builder, &table_combined, &self.query_count, &alpha);

        // let query_combined = combine_columns(
        //     builder,
        //     &concat_d1(&self.query_keys, &self.query_results),
        //     &randomness,
        // );
        // let one = builder.constant(1);
        // let v_query = logup_poly_val(
        //     builder,
        //     &query_combined,
        //     &vec![one; query_combined.len()],
        //     &alpha,
        // );

        // assert_eq_rational(builder, &v_table, &v_query);
    }
}
// #[test]
// fn logup_test() {
//     let params = LogUpParams {
//         key_len: 2,
//         value_len: 2,
//         n_table_rows: 4,
//         n_queries: 2,
//     };
//     let circuit = LogUpCircuit::new_circuit(&params);
//     let assignment = LogUpCircuit::new_assignment(&params, &mut rand::thread_rng());

//     let compile_result = compile(&circuit).unwrap();
//     let _ = compile_result.eval(&assignment);
// }