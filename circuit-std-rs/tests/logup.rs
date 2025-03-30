mod common;

use circuit_std_rs::{
    logup::{query_count_hint, rangeproof_hint, LogUpRangeProofTable},
    LogUpCircuit, LogUpParams,
};
use expander_compiler::frontend::{extra::debug_eval, *};

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
impl GenericDefine<M31Config> for LogUpRangeproofCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let mut table = LogUpRangeProofTable::new(16);
        table.initial(builder);
        for i in 12..30 {
            for j in ((1 << (i - 1))..(1 << i)).step_by(64) {
                let key = builder.constant(j);
                table.rangeproof(builder, key, i);
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
    let assignment = LogUpRangeproofCircuit { test: M31::from(0) };
    debug_eval(
        &LogUpRangeproofCircuit::default(),
        &assignment,
        hint_registry,
    );
    // //compile and test
    // let compile_result = compile_generic(
    //     &LogUpRangeproofCircuit::default(),
    //     CompileOptions::default(),
    // )
    // .unwrap();
    // let witness = compile_result
    //     .witness_solver
    //     .solve_witness_with_hints(&assignment, &mut hint_registry)
    //     .unwrap();
    // let output = compile_result.layered_circuit.run(&witness);
    // assert_eq!(output, vec![true]);
}
