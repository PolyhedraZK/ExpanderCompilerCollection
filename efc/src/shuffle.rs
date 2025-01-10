use std::fs::File;
use std::io::BufRead;
use std::{io, thread};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use ark_bls12_381::g2;
use circuit_std_rs::gnark::hints::register_hint;
use circuit_std_rs::logup::LogUpRangeProofTable;
use circuit_std_rs::utils::simple_select;
use expander_compiler::circuit::ir::hint_normalized::witness_solver;
use expander_compiler::frontend::*;
use expander_config::M31ExtConfigSha2;
use num_bigint::BigInt;
use sha2::{Digest, Sha256};
use circuit_std_rs::big_int::{to_binary_hint, big_array_add};
use crate::bls::check_pubkey_key_bls;
use crate::validator::ValidatorSSZ;
use circuit_std_rs::gnark::emulated::field_bls12381::*;
use circuit_std_rs::gnark::emulated::field_bls12381::e2::*;
use circuit_std_rs::gnark::emulated::sw_bls12381::pairing::*;
use circuit_std_rs::gnark::emulated::sw_bls12381::g1::*;
use circuit_std_rs::gnark::emulated::sw_bls12381::g2::*;
use circuit_std_rs::gnark::element::*;
use expander_compiler::frontend::extra::*;
use circuit_std_rs::big_int::*;
use expander_compiler::{circuit::layered::InputType, frontend::*};
use serde::Deserialize;
use serde_json::Value;

use crate::utils::run_circuit;
/*
const (
	ShuffleRound           = 90
	PrepareChunkSize       = 2048
	ValidatorChunkSize     = 128 * 4
	TestValidatorSize      = MaxValidator
	MaxValidator           = ValidatorChunkSize * SubcircuitNumber
	TestValidatorChunkSize = ValidatorChunkSize
	// OptimalSourceSize    = MaxValidator / 256
	SubcircuitNumber            = 1 << SubcircuitExp
	SubcircuitExp               = 11
	MaxValidatorExp             = 29
	MaxValidatorShuffleRoundExp = 21 + 7
	MaxSubCircuitExp            = 12
	HalfHashOutputBitLen        = 128
)
*/
const SHUFFLE_ROUND: usize = 90;
const PREPARE_CHUNK_SIZE: usize = 2048;
const VALIDATOR_CHUNK_SIZE: usize = 128 * 2;
const TEST_VALIDATOR_SIZE: usize = MAX_VALIDATOR;
const MAX_VALIDATOR: usize = VALIDATOR_CHUNK_SIZE * SUBCIRCUIT_NUMBER;
const TEST_VALIDATOR_CHUNK_SIZE: usize = VALIDATOR_CHUNK_SIZE;
const SUBCIRCUIT_NUMBER: usize = 1 << SUBCIRCUIT_EXP;
const SUBCIRCUIT_EXP: usize = 11;
const MAX_VALIDATOR_EXP: usize = 29;
const MAX_VALIDATOR_SHUFFLE_ROUND_EXP: usize = 21 + 7;
const MAX_SUBCIRCUIT_EXP: usize = 12;
const POSEIDON_HASH_LENGTH: usize = 8;

/*
type ShuffleWithHashMapAggPubkeyCircuit struct {
	StartIndex         frontend.Variable
	ChunkLength        frontend.Variable
	ShuffleIndices     [ValidatorChunkSize]frontend.Variable `gnark:",public"`
	CommitteeIndices   [ValidatorChunkSize]frontend.Variable `gnark:",public"`
	Pivots             [ShuffleRound]frontend.Variable
	IndexCount         frontend.Variable
	PositionResults    [ShuffleRound * ValidatorChunkSize]frontend.Variable           // the curIndex -> curPosition Table
	PositionBitResults [ShuffleRound * ValidatorChunkSize]frontend.Variable           `gnark:",public"` // mimic a hint: query the positionBitSortedResults table with runtime positions, get the flip bits
	FlipResults        [ShuffleRound * ValidatorChunkSize]frontend.Variable           // mimic a hint: get the flips, but we will ensure the correctness of the flipResults in the funciton (CheckPhasesAndResults)
	ValidatorHashes    [ValidatorChunkSize][hash.PoseidonHashLength]frontend.Variable `gnark:",public"`
	Slot               frontend.Variable                                              //the pre-pre beacon root
	AggregationBits    [ValidatorChunkSize]frontend.Variable                          //the aggregation bits
	AggregatedPubkey   sw_bls12381_m31.G1Affine                                       `gnark:",public"` //the aggregated pubkey of this committee, used for later signature verification circuit
	AttestationBalance [8]frontend.Variable                                           `gnark:",public"` //the attestation balance of this committee, the accBalance of each effective attestation should be supermajority, > 2/3 total balance
}

*/
// Define defines the circuit
declare_circuit!(ShuffleCircuit {
    start_index:         Variable,
    chunk_length:        Variable,
    shuffle_indices:     [Variable;VALIDATOR_CHUNK_SIZE],
    committee_indices:   [Variable;VALIDATOR_CHUNK_SIZE],
    pivots:             [Variable;SHUFFLE_ROUND],
    index_count:         Variable,
    position_results:    [Variable;SHUFFLE_ROUND * VALIDATOR_CHUNK_SIZE],
    position_bit_results: [Variable;SHUFFLE_ROUND * VALIDATOR_CHUNK_SIZE],
    flip_results:        [Variable;SHUFFLE_ROUND * VALIDATOR_CHUNK_SIZE],
    validator_hashes:    [[Variable;POSEIDON_HASH_LENGTH];VALIDATOR_CHUNK_SIZE],
    slot:               Variable,
    aggregation_bits:    [Variable;VALIDATOR_CHUNK_SIZE],
    aggregated_pubkey:   [[Variable;48];2],
    attestation_balance: [Variable;8],
    pubkeys_bls:      [[[Variable;48];2];VALIDATOR_CHUNK_SIZE],
    // // validators:      [ValidatorSSZ;VALIDATOR_CHUNK_SIZE],
    public_key: [[Variable; 48];VALIDATOR_CHUNK_SIZE],
    withdrawal_credentials: [[Variable; 32];VALIDATOR_CHUNK_SIZE],
    effective_balance: [[Variable; 8];VALIDATOR_CHUNK_SIZE],
    slashed: [[Variable; 1];VALIDATOR_CHUNK_SIZE],
    activation_eligibility_epoch: [[Variable; 8];VALIDATOR_CHUNK_SIZE],
    activation_epoch: [[Variable; 8];VALIDATOR_CHUNK_SIZE],
    exit_epoch: [[Variable; 8];VALIDATOR_CHUNK_SIZE],
    withdrawable_epoch: [[Variable; 8];VALIDATOR_CHUNK_SIZE],
});


impl GenericDefine<M31Config> for ShuffleCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let mut g1 = G1::new(builder);

        let mut indices_chunk = get_indice_chunk(builder, self.start_index, self.chunk_length, VALIDATOR_CHUNK_SIZE);

        //set padding indices to 0
        let zero_var = builder.constant(0);
        for i in 0..indices_chunk.len() {
            let tmp = builder.add(self.flip_results[i], 1);
            let ignore_flag = builder.is_zero(tmp);
            indices_chunk[i] = simple_select(builder, ignore_flag, zero_var.clone(), indices_chunk[i]);
        }
        //flip the indices based on the hashbit
        let mut cur_indices = indices_chunk.clone();
        let mut copy_cur_indices = builder.new_hint("myhint.copyvarshint",  &cur_indices, cur_indices.len());
        for i in 0..SHUFFLE_ROUND {
            cur_indices = flip_with_hash_bits(builder, &mut g1.curve_f.table, self.pivots[i], self.index_count, &copy_cur_indices, &self.position_results[i*VALIDATOR_CHUNK_SIZE..(i+1)*VALIDATOR_CHUNK_SIZE], &self.position_bit_results[i*VALIDATOR_CHUNK_SIZE..(i+1)*VALIDATOR_CHUNK_SIZE], &self.flip_results[i*VALIDATOR_CHUNK_SIZE..(i+1)*VALIDATOR_CHUNK_SIZE]);
            copy_cur_indices = builder.new_hint("myhint.copyvarshint",  &cur_indices, cur_indices.len());
        }

        //check the final curIndices, should be equal to the shuffleIndex
        for i in 0..self.shuffle_indices.len() {
            let tmp = builder.add(self.flip_results[i], 1);
            let is_minus_one = builder.is_zero(tmp);
            // println!("shuffle_indices:{:?}", builder.value_of(self.shuffle_indices[i]));
            // println!("cur_indices:{:?}", builder.value_of(cur_indices[i]));
            cur_indices[i] = simple_select(builder, is_minus_one, self.shuffle_indices[i], cur_indices[i]);
            let tmp = builder.sub(self.shuffle_indices[i], cur_indices[i]);
            let tmp_res = builder.is_zero(tmp);
            builder.assert_is_equal(tmp_res, 1);
        }

        let mut pubkey_list = vec![];
        let mut acc_balance = vec![];
        for i in 0..VALIDATOR_CHUNK_SIZE {
            pubkey_list.push(self.public_key[i].clone());
            acc_balance.push(self.effective_balance[i].clone());
        }
        let effect_balance = calculate_balance(builder, &mut acc_balance, &self.aggregation_bits);
        for i in 0..effect_balance.len() {
            builder.assert_is_equal(effect_balance[i], self.attestation_balance[i]);
        }

        let mut pubkey_list_bls = vec![];
        for i in 0..pubkey_list.len() {
            let pubkey_g1 = G1Affine::from_vars(self.pubkeys_bls[i][0].to_vec(), self.pubkeys_bls[i][1].to_vec());
            let logup_var = check_pubkey_key_bls(builder, pubkey_list[i].to_vec(), &pubkey_g1);
            g1.curve_f.table.rangeproof(builder, logup_var, 5);
            pubkey_list_bls.push(pubkey_g1);
        }

        let mut aggregated_pubkey = G1Affine::from_vars(self.aggregated_pubkey[0].to_vec(), self.aggregated_pubkey[1].to_vec());
        aggregate_attestation_public_key(builder, &mut g1, &pubkey_list_bls, &self.aggregation_bits, &mut aggregated_pubkey);
        
        for index in 0..VALIDATOR_CHUNK_SIZE{
            let mut validator = ValidatorSSZ::new();
            for i in 0..48 {
                validator.public_key[i] = self.public_key[index][i];
            }
            for i in 0..32 {
                validator.withdrawal_credentials[i] = self.withdrawal_credentials[index][i];
            }
            for i in 0..8 {
                validator.effective_balance[i] = self.effective_balance[index][i];
            }
            for i in 0..1 {
                validator.slashed[i] = self.slashed[index][i];
            }
            for i in 0..8 {
                validator.activation_eligibility_epoch[i] = self.activation_eligibility_epoch[index][i];
            }
            for i in 0..8 {
                validator.activation_epoch[i] = self.activation_epoch[index][i];
            }
            for i in 0..8 {
                validator.exit_epoch[i] = self.exit_epoch[index][i];
            }
            for i in 0..8 {
                validator.withdrawable_epoch[i] = self.withdrawable_epoch[index][i];
            }
            let hash = validator.hash(builder);
            for i in 0..8 {
                builder.assert_is_equal(&hash[i], &self.validator_hashes[index][i]);
            }
        }
        g1.curve_f.check_mul(builder);
        g1.curve_f.table.final_check(builder);
    }
}

fn get_indice_chunk<C: Config, B: RootAPI<C>>(builder: &mut B, start: Variable, length: Variable, max_len: usize) -> Vec<Variable> {
    let mut res = vec![];
    //M31_MOD = 2147483647
    let neg_one = builder.constant(2147483647-1);
    for i in 0..max_len {
        let tmp = builder.sub(length, i as u32);
        let reach_end = builder.is_zero(tmp);
        let mut tmp = builder.add(start, i as u32);
        tmp = simple_select(builder, reach_end, neg_one.clone(), tmp);
        res.push(tmp);
    }
    res
}
fn calculate_balance<C: Config, B: RootAPI<C>>(builder: &mut B, acc_balance: &mut Vec<[Variable;8]>, aggregation_bits: &[Variable]) -> Vec<Variable> {
    if acc_balance.len() == 0 || acc_balance[0].len() == 0 {
        panic!("accBalance is empty or invalid balance");
    } else if acc_balance.len() == 1 {
        return acc_balance[0].to_vec();
    }
    //initialize the balance
    let mut cur_balance = vec![builder.constant(0); acc_balance[0].len()];
    let zero_var = builder.constant(0);

    //set the balance to 0 if aggregationBits[i] = 0
    for i in 0..aggregation_bits.len() {
        for j in 0..acc_balance[i].len() {
            acc_balance[i][j] = simple_select(builder, aggregation_bits[i], acc_balance[i][j], zero_var.clone());
        }
    }
    //since balance is [8]frontend.Variable, we need to support Array addition
    for i in 0..acc_balance.len() {
        cur_balance = big_array_add(builder, &cur_balance, &acc_balance[i], cur_balance.len());
    }
    cur_balance
}
fn flip_with_hash_bits<C: Config, B: RootAPI<C>>(builder: &mut B, table: &mut LogUpRangeProofTable, pivot: Variable, index_count: Variable, cur_indices: &[Variable], position_results: &[Variable], position_bit_results: &[Variable], flip_results: &[Variable]) -> Vec<Variable> {
    let mut res = vec![];
    for i in 0..cur_indices.len() {
        let tmp = builder.add(flip_results[i].clone(), 1);
        let ignore_flag = builder.is_zero(tmp);
        let tmp = builder.sub(pivot, cur_indices[i]);
        let tmp = builder.sub(tmp, flip_results[i].clone());
        let flip_flag1 = builder.is_zero(tmp);
        let tmp = builder.add(index_count, pivot);
        let tmp = builder.sub(tmp, cur_indices[i]);
        let tmp = builder.sub(tmp, flip_results[i].clone());
        let flip_flag2 = builder.is_zero(tmp);
        let tmp = builder.or(flip_flag1, flip_flag2);
        let flip_flag = builder.or(tmp, ignore_flag);
        builder.assert_is_equal(flip_flag, 1);

        let tmp = builder.sub(position_results[i], flip_results[i]);
        let position_flag1 = builder.is_zero(tmp);
        let tmp = builder.sub(position_results[i], cur_indices[i]);
        let position_flag2 = builder.is_zero(tmp);
        let tmp = builder.or(position_flag1, position_flag2);
        let position_flag = builder.or(tmp, ignore_flag);
        builder.assert_is_equal(position_flag, 1);

        let tmp = builder.mul(2, position_results[i]);
        let tmp = builder.sub(tmp, flip_results[i].clone());
        let position_diff = builder.sub(tmp, cur_indices[i]);
        let zero_var = builder.constant(0);
        let position_diff = simple_select(builder, ignore_flag, zero_var.clone(), position_diff);
        table.rangeproof(builder, position_diff, MAX_VALIDATOR_EXP);

        res.push(simple_select(builder, position_bit_results[i], flip_results[i], cur_indices[i]));
    }
    res
}

pub fn aggregate_attestation_public_key<C: Config, B: RootAPI<C>>(builder: &mut B, g1: &mut G1, pub_key: &[G1Affine], validator_agg_bits: &[Variable], agg_pubkey: &mut G1Affine) {
    let mut validator_bits_vec = validator_agg_bits.to_vec();
    let mut aggregated_pubkey = pub_key[0].clone();
    for i in 1..validator_agg_bits.len() {
        let tmp_agg_pubkey = g1.add(builder, &aggregated_pubkey, &pub_key[i]);
        aggregated_pubkey.x = g1.curve_f.select(builder, validator_agg_bits[i], &tmp_agg_pubkey.x, &aggregated_pubkey.x);
        aggregated_pubkey.y = g1.curve_f.select(builder, validator_agg_bits[i], &tmp_agg_pubkey.y, &aggregated_pubkey.y);
    }
    g1.curve_f.assert_isequal(builder, &aggregated_pubkey.x, &agg_pubkey.x);
    g1.curve_f.assert_isequal(builder, &aggregated_pubkey.y, &agg_pubkey.y);
}
#[test]
fn run_multi_shuffle() {
    let mut rng = ark_std::test_rng();
    let mut builder = M31Config::default();
    let mut w_s: witness_solver::WitnessSolver::<M31Config>;
    if std::fs::metadata("shuffle.witness1").is_ok() {
        println!("The file exists!");
        w_s = witness_solver::WitnessSolver::deserialize_from(std::fs::File::open("shuffle.witness").unwrap()).unwrap();
    } else {
        println!("The file does not exist.");
        let compile_result = compile_generic(&ShuffleCircuit::default(), CompileOptions::default()).unwrap();
        compile_result.witness_solver.serialize_into(std::fs::File::create("shuffle.witness").unwrap()).unwrap();
        w_s = compile_result.witness_solver;
    }
}
#[test]
fn test_shuffle(){
    let mut hint_registry = HintRegistry::<M31>::new();
    register_hint(&mut hint_registry);
    let my_strcut = read_assignment();
    let mut assignment = ShuffleCircuit::<M31>::default();
    assignment.start_index = M31::from(my_strcut.start_index);
    assignment.chunk_length = M31::from(my_strcut.chunk_length);
    for i in 0..VALIDATOR_CHUNK_SIZE {
        assignment.shuffle_indices[i] = M31::from(my_strcut.shuffle_indices[i]);
        assignment.committee_indices[i] = M31::from(my_strcut.committee_indices[i]);
        assignment.aggregation_bits[i] = M31::from(my_strcut.aggregation_bits[i]);
    }
    assignment.aggregation_bits[255] = M31::from(1);
    for i in 0..SHUFFLE_ROUND {
        assignment.pivots[i] = M31::from(my_strcut.pivots[i]);
    }
    assignment.index_count = M31::from(my_strcut.index_count);
    for i in 0..SHUFFLE_ROUND * VALIDATOR_CHUNK_SIZE {
        assignment.position_results[i] = M31::from(my_strcut.position_results[i]);
        assignment.position_bit_results[i] = M31::from(my_strcut.position_bit_results[i]);
        assignment.flip_results[i] = M31::from(my_strcut.flip_results[i]);
    }
    assignment.slot = M31::from(my_strcut.slot);
    let balance_unit:u64 = 32000000000;
    let attestations_balance = balance_unit * 189;
    //0xb3e763f6f0153e49d0b1a43805ec1382bc82922000fd51e8e48ea6f5e9bb73a7f34f46ade6efa052dd1651b5bf94cd9d
    //[128 21 157 43 248 40 67 201 95 171 162 199 236 221 59 139 233 5 150 32 146 0 59 75 250 196 228 94 30 176 107 152 190 231 84 71 27 123 0 250 114 162 71 172 155 221 36 210] 
    let first_public_key = [128, 21, 157, 43, 248, 40, 67, 201, 95, 171, 162, 199, 236, 221, 59, 139, 233, 5, 150, 32, 146, 0, 59, 75, 250, 196, 228, 94, 30, 176, 107, 152, 190, 231, 84, 71, 27, 123, 0, 250, 114, 162, 71, 172, 155, 221, 36, 210] ;

    /*
    publicKeyX [210 36 221 155 172 71 162 114 250 0 123 27 71 84 231 190 152 107 176 30 94 228 196 250 75 59 0 146 32 150 5 233 139 59 221 236 199 162 171 95 201 67 40 248 43 157 21 0]
    publicKeyY [177 15 211 39 148 8 59 146 166 120 226 197 201 127 95 106 179 227 170 242 205 58 37 197 231 171 91 166 106 40 65 74 209 237 153 106 52 101 248 140 121 80 198 145 186 99 28 7]
     */
    let first_public_key_x = [210, 36, 221, 155, 172, 71, 162, 114, 250, 0, 123, 27, 71, 84, 231, 190, 152, 107, 176, 30, 94, 228, 196, 250, 75, 59, 0, 146, 32, 150, 5, 233, 139, 59, 221, 236, 199, 162, 171, 95, 201, 67, 40, 248, 43, 157, 21, 0];
    let first_public_key_y = [177, 15, 211, 39, 148, 8, 59, 146, 166, 120, 226, 197, 201, 127, 95, 106, 179, 227, 170, 242, 205, 58, 37, 197, 231, 171, 91, 166, 106, 40, 65, 74, 209, 237, 153, 106, 52, 101, 248, 140, 121, 80, 198, 145, 186, 99, 28, 7];
    //0xa4faba7e21a2ac4a29d9eef12433fd42f9baf0cafb2a046ff88c9ccdde20d88b0806673ab842664e15cdb7ad15f35d7f
    //164 250 186 126 33 162 172 74 41 217 238 241 36 51 253 66 249 186 240 202 251 42 4 111 248 140 156 205 222 32 216 139 8 6 103 58 184 66 102 78 21 205 183 173 21 243 93 127
    let test_public_key = [164, 250, 186, 126, 33, 162, 172, 74, 41, 217, 238, 241, 36, 51, 253, 66, 249, 186, 240, 202, 251, 42, 4, 111, 248, 140, 156, 205, 222, 32, 216, 139, 8, 6, 103, 58, 184, 66, 102, 78, 21, 205, 183, 173, 21, 243, 93, 127];

    /*    
    publicKeyX [127 93 243 21 173 183 205 21 78 102 66 184 58 103 6 8 139 216 32 222 205 156 140 248 111 4 42 251 202 240 186 249 66 253 51 36 241 238 217 41 74 172 162 33 126 186 250 4]
    publicKeyY [51 13 26 217 22 45 18 90 186 52 62 107 99 217 229 41 130 56 203 244 17 6 76 101 51 175 182 141 180 51 62 48 131 181 119 251 168 6 32 100 119 93 209 34 47 103 169 22]
     */
    let test_public_key_x = [127, 93, 243, 21, 173, 183, 205, 21, 78, 102, 66, 184, 58, 103, 6, 8, 139, 216, 32, 222, 205, 156, 140, 248, 111, 4, 42, 251, 202, 240, 186, 249, 66, 253, 51, 36, 241, 238, 217, 41, 74, 172, 162, 33, 126, 186, 250, 4];
    let test_public_key_y = [51, 13, 26, 217, 22, 45, 18, 90, 186, 52, 62, 107, 99, 217, 229, 41, 130, 56, 203, 244, 17, 6, 76, 101, 51, 175, 182, 141, 180, 51, 62, 48, 131, 181, 119, 251, 168, 6, 32, 100, 119, 93, 209, 34, 47, 103, 169, 22];
    //af0d108afcac75a19408b2cad29aa2191fe50d78c5c53169b01a06782f915b2b3b8f1ec522e56bc4a8f534e2ac4665cb
    //g1X:  [15 13 16 138 252 172 117 161 148 8 178 202 210 154 162 25 31 229 13 120 197 197 49 105 176 26 6 120 47 145 91 43 59 143 30 197 34 229 107 196 168 245 52 226 172 70 101 203]
    // g1Y:  [15 241 33 173 55 244 176 235 253 79 17 193 221 109 21 30 240 157 245 192 50 206 229 223 215 207 75 100 113 159 204 21 227 73 211 33 57 229 110 153 15 206 136 196 190 200 107 129]
    let aggregated_pubkey = [
        [206,22,64,219,11,55,22,57,57,232,188,112,205,116,244,1,11,33,145,200,247,86,166,219,248,30,102,125,248,89,217,166,164,113,3,244,248,53,58,162,173,25,1,36,123,1,223,22,],
        [114,93,129,62,49,1,167,235,229,203,35,20,88,219,86,119,129,178,63,173,207,204,36,252,39,184,165,77,235,165,150,163,194,112,93,194,123,40,249,143,70,190,21,68,140,138,18,14,]
    ];
    let balance_unit_byte = balance_unit.to_le_bytes();
    let attestation_balance_byte = attestations_balance.to_le_bytes();
    for i in 0..VALIDATOR_CHUNK_SIZE {
        for j in 0..48 {
            assignment.public_key[i][j] = M31::from(test_public_key[j] as u32);
        }
        for j in 0..8 {
            assignment.effective_balance[i][j] = M31::from(balance_unit_byte[j] as u32);
        }
    }
    for j in 0..48 {
        assignment.public_key[0][j] = M31::from(first_public_key[j] as u32);
    }
    for i in 0..8 {
        assignment.attestation_balance[i] = M31::from(attestation_balance_byte[i] as u32);
    }
    for i in 0..2 {
        for j in 0..48 {
            assignment.aggregated_pubkey[i][j] = M31::from(aggregated_pubkey[i][j] as u32);
        }
    }
    for i in 0..VALIDATOR_CHUNK_SIZE {
        for j in 0..48 {
            assignment.pubkeys_bls[i][0][j] = M31::from(test_public_key_x[j] as u32);
            assignment.pubkeys_bls[i][1][j] = M31::from(test_public_key_y[j] as u32);
        }
    }
    for j in 0..48 {
        assignment.pubkeys_bls[0][0][j] = M31::from(first_public_key_x[j] as u32);
        assignment.pubkeys_bls[0][1][j] = M31::from(first_public_key_y[j] as u32);
    }
    /*
    164 250 186 126 33 162 172 74 41 217 238 241 36 51 253 66 249 186 240 202 251 42 4 111 248 140 156 205 222 32 216 139 8 6 103 58 184 66 102 78 21 205 183 173 21 243 93 127
    publicKeyX [127 93 243 21 173 183 205 21 78 102 66 184 58 103 6 8 139 216 32 222 205 156 140 248 111 4 42 251 202 240 186 249 66 253 51 36 241 238 217 41 74 172 162 33 126 186 250 4]
    publicKeyY [51 13 26 217 22 45 18 90 186 52 62 107 99 217 229 41 130 56 203 244 17 6 76 101 51 175 182 141 180 51 62 48 131 181 119 251 168 6 32 100 119 93 209 34 47 103 169 22]
     */
    /*
    validator {[164 250 186 126 33 162 172 74 41 217 238 241 36 51 253 66 249 186 240 202 251 42 4 111 248 140 156 205 222 32 216 139 8 6 103 58 184 66 102 78 21 205 183 173 21 243 93 127] [0 13 174 238 43 225 74 172 101 0 121 181 152 5 116 129 23 55 205 156 170 184 177 81 36 38 133 233 173 23 88 65] 32000000000 0 65505 65717 18446744073709551615 18446744073709551615}
    validator {[128 21 157 43 248 40 67 201 95 171 162 199 236 221 59 139 233 5 150 32 146 0 59 75 250 196 228 94 30 176 107 152 190 231 84 71 27 123 0 250 114 162 71 172 155 221 36 210] [0 51 24 57 79 41 225 74 184 212 15 90 156 155 108 77 224 38 61 59 220 127 85 180 206 46 178 45 122 226 166 184] 32000000000 0 5028 9992 18446744073709551615 18446744073709551615}
     */
    let withdrawal_credentials = [0,13,174,238,43,225,74,172,101,0,121,181,152,5,116,129,23,55,205,156,170,184,177,81,36,38,133,233,173,23,88,65];
    let effective_balance:u64 = 32000000000;
    let effective_balance = effective_balance.to_le_bytes();
    let slashed = 0;
    let activation_eligibility_epoch:u64 = 65505;
    let activation_eligibility_epoch = activation_eligibility_epoch.to_le_bytes();
    let activation_epoch:u64 = 65717;
    let activation_epoch = activation_epoch.to_le_bytes();
    let exit_epoch:u64 = 18446744073709551615;
    let exit_epoch = exit_epoch.to_le_bytes();
    let withdrawable_epoch:u64 = 18446744073709551615;
    let withdrawable_epoch = withdrawable_epoch.to_le_bytes();
    for i in 0..VALIDATOR_CHUNK_SIZE {
        for j in 0..32 {
            assignment.withdrawal_credentials[i][j] = M31::from(withdrawal_credentials[j] as u32);
        }
        assignment.slashed[i][0] = M31::from(slashed as u32);
        for j in 0..8 {
            assignment.effective_balance[i][j] = M31::from(effective_balance[j] as u32);
            assignment.activation_eligibility_epoch[i][j] = M31::from(activation_eligibility_epoch[j] as u32);
            assignment.activation_epoch[i][j] = M31::from(activation_epoch[j] as u32);
            assignment.exit_epoch[i][j] = M31::from(exit_epoch[j] as u32);
            assignment.withdrawable_epoch[i][j] = M31::from(withdrawable_epoch[j] as u32);
        }
    }
    let hash = [1420980358,366442127,1729325529,1809151733,1503635331,1698111119,932538623,570007530];
    for i in 0..VALIDATOR_CHUNK_SIZE {
        for j in 0..POSEIDON_HASH_LENGTH {
            assignment.validator_hashes[i][j] = M31::from(hash[j] as u32);
        }
    }

    let withdrawal_credentials = [0,51,24,57,79,41,225,74,184,212,15,90,156,155,108,77,224,38,61,59,220,127,85,180,206,46,178,45,122,226,166,184];
    let effective_balance:u64 = 32000000000;
    let effective_balance = effective_balance.to_le_bytes();
    let slashed = 0;
    let activation_eligibility_epoch:u64 = 5028;
    let activation_eligibility_epoch = activation_eligibility_epoch.to_le_bytes();
    let activation_epoch:u64 = 9992;
    let activation_epoch = activation_epoch.to_le_bytes();
    let exit_epoch:u64 = 18446744073709551615;
    let exit_epoch = exit_epoch.to_le_bytes();
    let withdrawable_epoch:u64 = 18446744073709551615;
    let withdrawable_epoch = withdrawable_epoch.to_le_bytes();
    for i in 0..1 {
        for j in 0..32 {
            assignment.withdrawal_credentials[i][j] = M31::from(withdrawal_credentials[j] as u32);
        }
        assignment.slashed[i][0] = M31::from(slashed as u32);
        for j in 0..8 {
            assignment.effective_balance[i][j] = M31::from(effective_balance[j] as u32);
            assignment.activation_eligibility_epoch[i][j] = M31::from(activation_eligibility_epoch[j] as u32);
            assignment.activation_epoch[i][j] = M31::from(activation_epoch[j] as u32);
            assignment.exit_epoch[i][j] = M31::from(exit_epoch[j] as u32);
            assignment.withdrawable_epoch[i][j] = M31::from(withdrawable_epoch[j] as u32);
        }
    }
    let hash = [2114613924,997667299,213711641,1143404300,219133765,833923639,1195107857,116069398];
    for i in 0..1 {
        for j in 0..POSEIDON_HASH_LENGTH {
            assignment.validator_hashes[i][j] = M31::from(hash[j] as u32);
        }
    }
    stacker::grow(32 * 1024 * 1024 * 1024, ||    {debug_eval(&ShuffleCircuit::default(), &assignment, hint_registry)});
}
pub struct MyStruct { 
    start_index:         u32,
    chunk_length:        u32,
    shuffle_indices:     [u32;VALIDATOR_CHUNK_SIZE],
    committee_indices:   [u32;VALIDATOR_CHUNK_SIZE],
    pivots:             [u32;SHUFFLE_ROUND],
    index_count:         u32,
    position_results:    [u32;SHUFFLE_ROUND * VALIDATOR_CHUNK_SIZE],
    position_bit_results: [u32;SHUFFLE_ROUND * VALIDATOR_CHUNK_SIZE],
    flip_results:        [u32;SHUFFLE_ROUND * VALIDATOR_CHUNK_SIZE],
    slot:               u32,
    aggregation_bits:    [u32;VALIDATOR_CHUNK_SIZE],
}
fn read_assignment() -> MyStruct {
    let file_path = "test.json";

    let file = File::open(file_path).unwrap();
    let reader = io::BufReader::new(file);

    let mut my_struct = MyStruct {
        start_index: 0,
        chunk_length: 0,
        shuffle_indices: [0;VALIDATOR_CHUNK_SIZE],
        committee_indices: [0;VALIDATOR_CHUNK_SIZE],
        pivots: [0;SHUFFLE_ROUND],
        index_count: 0,
        position_results: [0;SHUFFLE_ROUND * VALIDATOR_CHUNK_SIZE],
        position_bit_results: [0;SHUFFLE_ROUND * VALIDATOR_CHUNK_SIZE],
        flip_results: [0;SHUFFLE_ROUND * VALIDATOR_CHUNK_SIZE],
        slot: 0,
        aggregation_bits: [0;VALIDATOR_CHUNK_SIZE],
    };

    for line in reader.lines() {
        let line = line.unwrap();
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() == 2 {
            let key = parts[0].trim();
            let value = parts[1].trim();

            match key {
                "StartIndex" => {
                    my_struct.start_index = value.parse::<u32>().unwrap_or_default();
                    println!("Parsed StartIndex");
                }
                "ChunkLength" => {
                    my_struct.chunk_length = value.parse::<u32>().unwrap_or_default();
                    println!("Parsed ChunkLength");
                }
                "ShuffleIndices" => {
                    if let Some(start) = value.find('[') {
                        if let Some(end) = value.find(']') {
                            let numbers = &value[start + 1..end];
                            let result: Vec<u32> = numbers
                                .split_whitespace()
                                .filter_map(|num| num.parse::<u32>().ok())
                                .collect();
                
                            println!("Parsed ShuffleIndices");
                            my_struct.shuffle_indices = result.try_into().unwrap();
                        }
                    }
                }
                "CommitteeIndices" => {
                    if let Some(start) = value.find('[') {
                        if let Some(end) = value.find(']') {
                            let numbers = &value[start + 1..end];
                            let result: Vec<u32> = numbers
                                .split_whitespace()
                                .filter_map(|num| num.parse::<u32>().ok())
                                .collect();
                
                            //println!("Parsed array: {:?}", result);
                            println!("Parsed CommitteeIndices");
                            my_struct.committee_indices = result.try_into().unwrap();
                        }
                    }
                }
                "Pivots" => {
                    if let Some(start) = value.find('[') {
                        if let Some(end) = value.find(']') {
                            let numbers = &value[start + 1..end];
                            let result: Vec<u32> = numbers
                                .split_whitespace()
                                .filter_map(|num| num.parse::<u32>().ok())
                                .collect();
                
                            //println!("Parsed array: {:?}", result);
                            println!("Parsed Pivots");
                            my_struct.pivots = result.try_into().unwrap();
                        }
                    }
                }
                "IndexCount" => {
                    my_struct.index_count = value.parse::<u32>().unwrap_or_default();
                    println!("Parsed IndexCount");
                }
                "PositionResults" => {
                    if let Some(start) = value.find('[') {
                        if let Some(end) = value.find(']') {
                            let numbers = &value[start + 1..end];
                            let result: Vec<u32> = numbers
                                .split_whitespace()
                                .filter_map(|num| num.parse::<u32>().ok())
                                .collect();
                
                            // println!("PositionResults: {:?}", result);
                            println!("PositionResults");
                            my_struct.position_results = result.try_into().unwrap();
                        }
                    }
                }
                "PositionBitResults" => {
                    if let Some(start) = value.find('[') {
                        if let Some(end) = value.find(']') {
                            let numbers = &value[start + 1..end];
                            let result: Vec<u32> = numbers
                                .split_whitespace()
                                .filter_map(|num| num.parse::<u32>().ok())
                                .collect();
                
                            //println!("Parsed array: {:?}", result);
                            println!("PositionBitResults");
                            my_struct.position_bit_results = result.try_into().unwrap();
                        }
                    }
                }
                "FlipResults" => {
                    if let Some(start) = value.find('[') {
                        if let Some(end) = value.find(']') {
                            let numbers = &value[start + 1..end];
                            let result: Vec<u32> = numbers
                                .split_whitespace()
                                .filter_map(|num| num.parse::<u32>().ok())
                                .collect();
                
                            //println!("Parsed array: {:?}", result);
                            println!("FlipResults");
                            my_struct.flip_results = result.try_into().unwrap();
                        }
                    }
                }
                "Slot" => {
                    my_struct.slot = value.parse::<u32>().unwrap_or_default();
                    println!("Slot");
                }
                "AggregationBits" => {
                    if let Some(start) = value.find('[') {
                        if let Some(end) = value.find(']') {
                            let numbers = &value[start + 1..end];
                            let result: Vec<u32> = numbers
                                .split_whitespace()
                                .filter_map(|num| num.parse::<u32>().ok())
                                .collect();
                
                            //println!("Parsed array: {:?}", result);
                            println!("AggregationBits");
                            my_struct.aggregation_bits = result.try_into().unwrap();
                        }
                    }

                }
                _ => {
                    eprintln!("Unknown key: {}", key);
                }
            }
        }
    }
    println!("my_struct: {:?}", my_struct.shuffle_indices);
    my_struct
}