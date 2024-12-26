use arith::Field;
use expander_compiler::frontend::*;


#[derive(Clone, Copy, Debug)]
pub struct HashTableParams {
    pub table_size: usize,
    pub hash_len: usize,
}
declare_circuit!(_HashTableCircuit {
    shuffle_round: Variable,
    start_index: [Variable;4],
    seed: [Variable;32],
    hash_outputs: [[Variable;32];64],
});

pub type HashTableCircuit = _HashTableCircuit<Variable>;


impl<C: Config> Define<C> for HashTableCircuit {
    fn define(&self, builder: &mut API<C>) {

    let  mut  hash_inputs:Vec<Vec<Variable>> = Vec::new();
    let mut cur_index = self.start_index.clone();
    for i in 0..64 {
        let mut cur_input:Vec<Variable> = Vec::new();
        for j in 0..32 {
            cur_input.push(self.seed[j]);
        }
        cur_input.push(self.shuffle_round);
        for j in 0..4 {
            cur_input.push(cur_index[j]);
        }
        hash_inputs.push(cur_input);
        cur_index = common::ArrayBoundedAdd(builder, cur_index, [1, 0, 0, 0], 8);
    }
    let hash_res_array = hash::CalSpecialHashArray(builder, hash_inputs, opt);
    for i in 0..64 {
        for j in 0..32 {
                builder.assert_is_equal(hash_res_array[i][j], self.hash_outputs[i][j]);
        }
    }
    }
}
