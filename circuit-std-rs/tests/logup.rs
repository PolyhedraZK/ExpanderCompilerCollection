mod common;

use circuit_std_rs::{
    logup::{
        query_count_by_key_hint, query_count_hint, rangeproof_hint, LogUpRangeProofTable,
        LogUpSingleKeyTable,
    },
    LogUpCircuit, LogUpParams,
};
use expander_compiler::{field::BN254Fr, field::Goldilocks, frontend::*};

#[test]
fn logup_test() {
    let logup_params = LogUpParams {
        key_len: 7,
        value_len: 7,
        n_table_rows: 123,
        n_queries: 456,
    };

    common::circuit_test_helper::<BN254Config, LogUpCircuit>(&logup_params);
    common::circuit_test_helper::<M31Config, LogUpCircuit>(&logup_params);
    common::circuit_test_helper::<GF2Config, LogUpCircuit>(&logup_params);
}

declare_circuit!(LogUpRangeproofCircuit { test: Variable });
impl Define<M31Config> for LogUpRangeproofCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let mut table = LogUpRangeProofTable::new(8);
        table.initial(builder);
        for i in 1..12 {
            for j in (1 << (i - 1))..(1 << i) {
                let key = builder.constant(j);
                if i > 8 {
                    table.rangeproof(builder, key, i);
                } else {
                    table.rangeproof_onechunk(builder, key, i);
                }
            }
        }
        table.final_check(builder);
    }
}

#[test]
fn rangeproof_logup_test() {
    let mut hint_registry = HintRegistry::<M31>::new();
    hint_registry.register("myhint.querycounthint", query_count_hint);
    hint_registry.register("myhint.rangeproofhint", rangeproof_hint);
    //compile and test
    let compile_result = compile(
        &LogUpRangeproofCircuit::default(),
        CompileOptions::default(),
    )
    .unwrap();
    let assignment = LogUpRangeproofCircuit { test: M31::from(0) };
    let witness = compile_result
        .witness_solver
        .solve_witness_with_hints(&assignment, &mut hint_registry)
        .unwrap();
    let output = compile_result.layered_circuit.run(&witness);
    assert_eq!(output, vec![true]);
}
const QUERY_TABLE_SIZE:usize = 1024;
declare_circuit!(LogUpSingleKeyCircuit {
    index: [Variable; QUERY_TABLE_SIZE],
    value: [Variable; QUERY_TABLE_SIZE],
    table: [Variable; QUERY_TABLE_SIZE],
});
impl Define<M31Config> for LogUpSingleKeyCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let mut table = LogUpSingleKeyTable::new(8);
        let mut table_key = vec![];
        for i in 0..QUERY_TABLE_SIZE {
            table_key.push(builder.constant(i as u32));
        }
        let mut table_values = vec![];
        for i in 0..QUERY_TABLE_SIZE {
            table_values.push(vec![self.table[i]]);
        }
        table.new_table(table_key, table_values);
        let mut query_values = vec![];
        for i in 0..QUERY_TABLE_SIZE {
            query_values.push(vec![self.value[i]]);
        }
        table.batch_query(self.index.to_vec(), query_values);
        //m31 field, repeat 3 times to achieve 90-bit security (or 4 times for 120-bit security)
        table.final_check(builder);
        table.final_check(builder);
        table.final_check(builder);
    }
}

#[test]
fn rangeproof_logup_singlekey_test() {
    use rand::SeedableRng;
    use rand::Rng;
    let mut hint_registry = HintRegistry::<M31>::new();
    hint_registry.register("myhint.querycounthint", query_count_hint);
    hint_registry.register("myhint.rangeproofhint", rangeproof_hint);
    hint_registry.register("myhint.querycountbykeyhint", query_count_by_key_hint);
    //compile and test
    let compile_result =
        compile(&LogUpSingleKeyCircuit::default(), CompileOptions::default()).unwrap();
    let mut assignment = LogUpSingleKeyCircuit::default();
    let mut rng = rand::rngs::StdRng::seed_from_u64(1235);
    let mut index = [0; QUERY_TABLE_SIZE];
    let mut value = [0; QUERY_TABLE_SIZE];
    let mut table = [0; QUERY_TABLE_SIZE];
    for i in 0..QUERY_TABLE_SIZE {
        index[i] = rng.gen_range(0..QUERY_TABLE_SIZE) as u32;
        table[i] = rng.gen_range(0..QUERY_TABLE_SIZE) as u32;
    }
    for i in 0..QUERY_TABLE_SIZE {
        value[i] = table[index[i] as usize];
    }
    for i in 0..QUERY_TABLE_SIZE {
        assignment.index[i] = M31::from(index[i]);
        assignment.value[i] = M31::from(value[i]);
        assignment.table[i] = M31::from(table[i]);
    }
    let witness = compile_result
        .witness_solver
        .solve_witness_with_hints(&assignment, &mut hint_registry)
        .unwrap();
    let output = compile_result.layered_circuit.run(&witness);
    assert_eq!(output, vec![true]);
}

declare_circuit!(RangeproofCircuit {
    place_holder: Variable
});
impl Define<GoldilocksConfig> for RangeproofCircuit<Variable> {
    fn define<Builder: RootAPI<GoldilocksConfig>>(&self, builder: &mut Builder) {
        let mut table = LogUpRangeProofTable::new(16);
        table.initial(builder);

        // < 2^24 value
        let key = builder.constant(16777215);
        table.rangeproof(builder, key, 24);

        table.final_check(builder);
    }
}

#[test]
fn rangeproof_goldilocks_test() {
    let mut hint_registry = HintRegistry::<Goldilocks>::new();
    hint_registry.register("myhint.querycounthint", query_count_hint);
    hint_registry.register("myhint.rangeproofhint", rangeproof_hint);
    //compile and test
    let compile_result = compile(&RangeproofCircuit::default(), CompileOptions::default()).unwrap();
    let assignment = RangeproofCircuit {
        place_holder: Goldilocks::one(),
    };
    let witness = compile_result
        .witness_solver
        .solve_witness_with_hints(&assignment, &mut hint_registry)
        .unwrap();
    let output = compile_result.layered_circuit.run(&witness);
    assert_eq!(output, vec![true]);
}

declare_circuit!(RangeproofLogupCircuit {
    _placeholder: Variable
});
impl Define<BN254Config> for RangeproofLogupCircuit<Variable> {
    fn define<Builder: RootAPI<BN254Config>>(&self, builder: &mut Builder) {
        let mut table = LogUpRangeProofTable::new(16);
        table.initial(builder);
        let key1 = builder.constant(18888888);
        table.rangeproof(builder, key1, 37);

        let key2 = builder.constant(58888888);
        table.rangeproof(builder, key2, 49);

        table.final_check(builder);
    }
}

#[test]
fn rangeproof_bn254_test() {
    let mut hint_registry = HintRegistry::<BN254Fr>::new();
    hint_registry.register("myhint.querycounthint", query_count_hint);
    hint_registry.register("myhint.rangeproofhint", rangeproof_hint);
    //compile and test
    let compile_result = compile(
        &RangeproofLogupCircuit::default(),
        CompileOptions::default(),
    )
    .unwrap();
    let assignment = RangeproofLogupCircuit {
        _placeholder: BN254Fr::zero(),
    };
    let witness = compile_result
        .witness_solver
        .solve_witness_with_hints(&assignment, &mut hint_registry)
        .unwrap();
    let output = compile_result.layered_circuit.run(&witness);
    assert_eq!(output, vec![true]);
}
