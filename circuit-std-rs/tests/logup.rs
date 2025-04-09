mod common;

use circuit_std_rs::{
    logup::{query_count_hint, rangeproof_hint, LogUpRangeProofTable},
    LogUpCircuit, LogUpParams,
};
use expander_compiler::{field::Goldilocks, frontend::*};

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
