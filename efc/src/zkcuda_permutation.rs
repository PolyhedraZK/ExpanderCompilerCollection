use expander_compiler::frontend::M31Config;
use expander_compiler::frontend::*;
use expander_compiler::zkcuda::kernel::Kernel;
use expander_compiler::zkcuda::kernel::*;
use circuit_std_rs::logup::LogUpSingleKeyTable;
use crate::permutation::TABLE_SIZE;

fn verify_permutation_hash_inner<C: Config>(api: &mut API<C>, p: &Vec<Variable>) -> Vec<Variable> {
    let index = &p[..TABLE_SIZE];
    let value = &p[TABLE_SIZE..TABLE_SIZE*2];
    let table = &p[TABLE_SIZE*2..TABLE_SIZE*3];
    let mut lutable = LogUpSingleKeyTable::new(8);
    let mut table_key = vec![];
    for i in 0..TABLE_SIZE {
        table_key.push(api.constant(i as u32));
    }
    let mut table_values = vec![];
    for i in 0..TABLE_SIZE {
        table_values.push(vec![table[i]]);
    }
    lutable.new_table(table_key, table_values);
    let mut query_values = vec![];
    for i in 0..TABLE_SIZE {
        query_values.push(vec![value[i]]);
    }
    lutable.batch_query(index.to_vec(), query_values);
    //m31 field, repeat 3 times
    lutable.final_check(api);
    lutable.final_check(api);
    lutable.final_check(api);

    return vec![api.constant(1)];
}

#[kernel]
fn verify_permutation_hash<C: Config>(
    api: &mut API<C>,
    input: &[InputVariable; TABLE_SIZE*3],
    output: &mut OutputVariable,
) {
    let outc = api.memorized_simple_call(verify_permutation_hash_inner, input);
    *output = outc[0]
}
#[test]
fn test_zkcuda_permutation_hash() {
    let _: Kernel<M31Config> = compile_verify_permutation_hash().unwrap();
    println!("compile ok");
}