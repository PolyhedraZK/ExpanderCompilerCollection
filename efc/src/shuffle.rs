
use std::sync::Arc;
use std::thread;
use serde::de::{Deserializer, SeqAccess, Visitor};
use serde::Deserialize;
use std::fmt;
use circuit_std_rs::gnark::hints::register_hint;
use circuit_std_rs::logup::LogUpRangeProofTable;
use circuit_std_rs::utils::simple_select;
use expander_compiler::circuit::ir::hint_normalized::witness_solver;
use expander_compiler::frontend::*;
use expander_config::M31ExtConfigSha2;
use circuit_std_rs::big_int::{to_binary_hint, big_array_add};
use crate::bls::check_pubkey_key_bls;
use crate::bls_verifier::G1Json;
use crate::validator::{read_validators, ValidatorPlain, ValidatorSSZ};
use circuit_std_rs::gnark::emulated::sw_bls12381::g1::*;
use expander_compiler::frontend::extra::*;
use base64;
use std::sync::Mutex;
use crate::utils::{ensure_directory_exists, read_from_json_file, run_circuit};
const SHUFFLE_ROUND: usize = 90;
const VALIDATOR_CHUNK_SIZE: usize = 128 * 4;
const MAX_VALIDATOR_EXP: usize = 29;
const POSEIDON_HASH_LENGTH: usize = 8;

#[derive(Debug, Deserialize, Clone)]
pub struct ShuffleJson { 
    StartIndex:         u32,
    ChunkLength:        u32,
    #[serde(deserialize_with = "deserialize_1d_u32_m31")]
    ShuffleIndices:     Vec<u32>,
    #[serde(deserialize_with = "deserialize_1d_u32_m31")]
    CommitteeIndices:   Vec<u32>,
    #[serde(deserialize_with = "deserialize_1d_u32_m31")]
    Pivots:             Vec<u32>,
    IndexCount:         u32,
    #[serde(deserialize_with = "deserialize_1d_u32_m31")]
    PositionResults:    Vec<u32>,
    #[serde(deserialize_with = "deserialize_1d_u32_m31")]
    PositionBitResults: Vec<u32>,
    #[serde(deserialize_with = "deserialize_1d_u32_m31")]
    FlipResults:        Vec<u32>,
    Slot:               u32,
    #[serde(deserialize_with = "deserialize_2d_u32_m31")]
    ValidatorHashes:    Vec<Vec<u32>>,
    #[serde(deserialize_with = "deserialize_1d_u32_m31")]
    AggregationBits:   Vec<u32>,
    AggregatedPubkey:   G1Json,
    AttestationBalance: Vec<u32>,

}
fn process_i64_value(value: i64) -> u32 {
    if value == -1 {
        (1u32 << 31) - 2    // p - 1
    } else if value >= 0 {
        value as u32
    } else {
        panic!("Unexpected negative value other than -1");
    }
}
fn deserialize_1d_u32_m31<'de, D>(deserializer: D) -> Result<Vec<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    let bits: Vec<i64> = Deserialize::deserialize(deserializer)?;
    Ok(bits.into_iter().map(process_i64_value).collect())
}

fn deserialize_2d_u32_m31<'de, D>(deserializer: D) -> Result<Vec<Vec<u32>>, D::Error>
where
    D: Deserializer<'de>,
{

    struct ValidatorHashesVisitor;

    impl<'de> Visitor<'de> for ValidatorHashesVisitor {
        type Value = Vec<Vec<u32>>;
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a nested array of integers")
        }
        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut outer = Vec::new();
            while let Some(inner) = seq.next_element::<Vec<i64>>()? {
                let processed_inner = inner.into_iter().map(process_i64_value).collect();
                outer.push(processed_inner);
            }
            Ok(outer)
        }
    }

    deserializer.deserialize_seq(ValidatorHashesVisitor)
}

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
    slot:               Variable,
    aggregation_bits:    [Variable;VALIDATOR_CHUNK_SIZE],
    validator_hashes:    [[Variable;POSEIDON_HASH_LENGTH];VALIDATOR_CHUNK_SIZE],
    aggregated_pubkey:   [[Variable;48];2],
    attestation_balance: [Variable;8],
    pubkeys_bls:      [[[Variable;48];2];VALIDATOR_CHUNK_SIZE],
    // validators:      [ValidatorSSZ;VALIDATOR_CHUNK_SIZE],
    pubkey: [[Variable; 48];VALIDATOR_CHUNK_SIZE],
    withdrawal_credentials: [[Variable; 32];VALIDATOR_CHUNK_SIZE],
    effective_balance: [[Variable; 8];VALIDATOR_CHUNK_SIZE],
    slashed: [[Variable; 1];VALIDATOR_CHUNK_SIZE],
    activation_eligibility_epoch: [[Variable; 8];VALIDATOR_CHUNK_SIZE],
    activation_epoch: [[Variable; 8];VALIDATOR_CHUNK_SIZE],
    exit_epoch: [[Variable; 8];VALIDATOR_CHUNK_SIZE],
    withdrawable_epoch: [[Variable; 8];VALIDATOR_CHUNK_SIZE],
});


impl ShuffleCircuit<M31> {
    pub fn from_plains(&mut self, shuffle_json: ShuffleJson, plain_validators: &Vec<ValidatorPlain>, pubkey_bls: &Vec<Vec<String>>) {
        if shuffle_json.CommitteeIndices.len() != VALIDATOR_CHUNK_SIZE {
            panic!("committee_indices length is not equal to VALIDATOR_CHUNK_SIZE");
        }
        //assign shuffle_json
        self.start_index = M31::from(shuffle_json.StartIndex);
        self.chunk_length = M31::from(shuffle_json.ChunkLength);
        for i in 0..VALIDATOR_CHUNK_SIZE {
            self.shuffle_indices[i] = M31::from(shuffle_json.ShuffleIndices[i]);
            self.committee_indices[i] = M31::from(shuffle_json.CommitteeIndices[i]);
            self.aggregation_bits[i] = M31::from(shuffle_json.AggregationBits[i]);
        }
        for i in 0..SHUFFLE_ROUND {
            self.pivots[i] = M31::from(shuffle_json.Pivots[i]);
        }
        self.index_count = M31::from(shuffle_json.IndexCount);
        for i in 0..SHUFFLE_ROUND * VALIDATOR_CHUNK_SIZE {
            self.position_results[i] = M31::from(shuffle_json.PositionResults[i]);
            self.position_bit_results[i] = M31::from(shuffle_json.PositionBitResults[i]);
            self.flip_results[i] = M31::from(shuffle_json.FlipResults[i]);
        }
        self.slot = M31::from(shuffle_json.Slot);

        //assign validator_hashes
        for i in 0..VALIDATOR_CHUNK_SIZE{
            for j in 0..POSEIDON_HASH_LENGTH {
                self.validator_hashes[i][j] = M31::from(shuffle_json.ValidatorHashes[i][j]);
            }
        }

        //assign aggregated_pubkey
        let pubkey = &shuffle_json.AggregatedPubkey;
        for i in 0..48 {
            self.aggregated_pubkey[0][i] = M31::from(pubkey.X.Limbs[i] as u32);
            self.aggregated_pubkey[1][i] = M31::from(pubkey.Y.Limbs[i] as u32);
        }

        //assign attestation_balance
        for i in 0..8 {
            self.attestation_balance[i] = M31::from(shuffle_json.AttestationBalance[i]);
        }

        for i in 0..VALIDATOR_CHUNK_SIZE{
            //assign pubkey_bls
            let raw_pubkey_bls = &pubkey_bls[shuffle_json.CommitteeIndices[i] as usize];
            let pubkey_bls_x = base64::decode(&raw_pubkey_bls[0]).unwrap();
            let pubkey_bls_y = base64::decode(&raw_pubkey_bls[1]).unwrap();
            for k in 0..48 {
                self.pubkeys_bls[i][0][k] = M31::from(pubkey_bls_x[47-k] as u32);
                self.pubkeys_bls[i][1][k] = M31::from(pubkey_bls_y[47-k] as u32);
            }

            //assign validator
            let validator = plain_validators[shuffle_json.CommitteeIndices[i] as usize].clone();

            //assign pubkey
            let raw_pubkey = validator.public_key.clone();
            let pubkey = base64::decode(raw_pubkey).unwrap();
            for j in 0..48 {
                self.pubkey[i][j] = M31::from(pubkey[j] as u32);
            }
            //assign withdrawal_credentials
            let raw_withdrawal_credentials = validator.withdrawal_credentials.clone();
            let withdrawal_credentials = base64::decode(raw_withdrawal_credentials).unwrap();
            for j in 0..32 {
                self.withdrawal_credentials[i][j] = M31::from(withdrawal_credentials[j] as u32);
            }
            //assign effective_balance
            let effective_balance = validator.effective_balance.to_le_bytes();
            for j in 0..8 {
                self.effective_balance[i][j] = M31::from(effective_balance[j] as u32);
            }
            //assign slashed
            let slashed = if validator.slashed { 1 } else { 0 };
            self.slashed[i][0] = M31::from(slashed);
            //assign activation_eligibility_epoch
            let activation_eligibility_epoch = validator.activation_eligibility_epoch.to_le_bytes();
            for j in 0..8 {
                self.activation_eligibility_epoch[i][j] = M31::from(activation_eligibility_epoch[j] as u32);
            }
            //assign activation_epoch
            let activation_epoch = validator.activation_epoch.to_le_bytes();
            for j in 0..8 {
                self.activation_epoch[i][j] = M31::from(activation_epoch[j] as u32);
            }
            //assign exit_epoch
            let exit_epoch = validator.exit_epoch.to_le_bytes();
            for j in 0..8 {
                self.exit_epoch[i][j] = M31::from(exit_epoch[j] as u32);
            }
            //assign withdrawable_epoch
            let withdrawable_epoch = validator.withdrawable_epoch.to_le_bytes();
            for j in 0..8 {
                self.withdrawable_epoch[i][j] = M31::from(withdrawable_epoch[j] as u32);
            }
        }
    }
    pub fn from_pubkey_bls(&mut self, committee_indices:Vec<u32>, pubkey_bls: Vec<Vec<String>>) {
        for i in 0..VALIDATOR_CHUNK_SIZE {
            let pubkey = &pubkey_bls[committee_indices[i] as usize];
            let pubkey_x = base64::decode(&pubkey[0]).unwrap();
            let pubkey_y = base64::decode(&pubkey[1]).unwrap();
            for k in 0..48 {
                self.pubkeys_bls[i][0][k] = M31::from(pubkey_x[k] as u32);
                self.pubkeys_bls[i][1][k] = M31::from(pubkey_y[k] as u32);
            }
        }
    }
}
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
            cur_indices[i] = simple_select(builder, is_minus_one, self.shuffle_indices[i], cur_indices[i]);
            let tmp = builder.sub(self.shuffle_indices[i], cur_indices[i]);
            let tmp_res = builder.is_zero(tmp);
            builder.assert_is_equal(tmp_res, 1);
        }

        let mut pubkey_list = vec![];
        let mut acc_balance = vec![];
        for i in 0..VALIDATOR_CHUNK_SIZE {
            pubkey_list.push(self.pubkey[i].clone());
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
                validator.public_key[i] = self.pubkey[index][i];
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
    let one_var = builder.constant(1);
    let mut has_first_flag = builder.constant(0);
    let mut aggregated_pubkey = pub_key[0].clone();
    has_first_flag = simple_select(builder, validator_agg_bits[0], one_var, has_first_flag);
    for i in 1..validator_agg_bits.len() {
        let tmp_agg_pubkey = g1.add(builder, &aggregated_pubkey, &pub_key[i]);
        aggregated_pubkey.x = g1.curve_f.select(builder, validator_agg_bits[i], &tmp_agg_pubkey.x, &aggregated_pubkey.x);
        aggregated_pubkey.y = g1.curve_f.select(builder, validator_agg_bits[i], &tmp_agg_pubkey.y, &aggregated_pubkey.y);
        let no_first_flag = builder.sub(1, has_first_flag);
        let is_first = builder.and(validator_agg_bits[i], no_first_flag);
        aggregated_pubkey.x = g1.curve_f.select(builder, is_first, &pub_key[i].x, &aggregated_pubkey.x);
        aggregated_pubkey.y = g1.curve_f.select(builder, is_first, &pub_key[i].y, &aggregated_pubkey.y);
        has_first_flag = simple_select(builder, validator_agg_bits[i], one_var, has_first_flag);
    }
    g1.curve_f.assert_isequal(builder, &aggregated_pubkey.x, &agg_pubkey.x);
    g1.curve_f.assert_isequal(builder, &aggregated_pubkey.y, &agg_pubkey.y);
}

#[test]
fn read_json_to_shuffle(){

    let plain_validators = read_validators("");
    let file_path = "shuffle_assignment.json";
    let shuffle_data:Vec<ShuffleJson> = read_from_json_file(file_path).unwrap();
    let file_path = "pubkeyBLSList.json";
    let public_key_bls_list: Vec<Vec<String>> = read_from_json_file(file_path).unwrap();

    let mut hint_registry = HintRegistry::<M31>::new();
    register_hint(&mut hint_registry);
    let mut assignment = ShuffleCircuit::<M31>::default();
    assignment.from_plains(shuffle_data[shuffle_data.len()-1].clone(), &plain_validators, &public_key_bls_list);

    stacker::grow(32 * 1024 * 1024 * 1024, ||    {debug_eval(&ShuffleCircuit::default(), &assignment, hint_registry)});
}



pub fn generate_shuffle_witnesses(dir: &str){
    stacker::grow(32 * 1024 * 1024 * 1024, ||    {
	    println!("preparing solver...");
        ensure_directory_exists("./witnesses/shuffle");
        let w_s: witness_solver::WitnessSolver::<M31Config>;
        if std::fs::metadata("shuffle.witness").is_ok() {
            println!("The file exists!");
            w_s = witness_solver::WitnessSolver::deserialize_from(std::fs::File::open("shuffle.witness").unwrap()).unwrap();
        } else {
            println!("The file does not exist.");
            let compile_result = compile_generic(&ShuffleCircuit::default(), CompileOptions::default()).unwrap();
            compile_result.witness_solver.serialize_into(std::fs::File::create("shuffle.witness").unwrap()).unwrap();
            w_s = compile_result.witness_solver;
        }
        let witness_solver = Arc::new(w_s);


        println!("generating witnesses...");
        let start_time = std::time::Instant::now();
        let plain_validators = read_validators(dir);
        let file_path = format!("{}/shuffle_assignment.json",dir);
        let shuffle_data: Vec<ShuffleJson> = read_from_json_file(&file_path).unwrap();
        let file_path = format!("{}/pubkeyBLSList.json",dir);
        let public_key_bls_list: Vec<Vec<String>> = read_from_json_file(&file_path).unwrap();
        let end_time = std::time::Instant::now();
        println!("loaed assignment data, time: {:?}", end_time.duration_since(start_time));

        let mut handles = vec![];
        // let max_threads = 16;
        // let semaphore = Arc::new(Mutex::new(AtomicUsize::new(0)));
        let plain_validators = Arc::new(plain_validators);
        let public_key_bls_list = Arc::new(public_key_bls_list);
        let assignments = Arc::new(Mutex::new(vec![None; shuffle_data.len()]));

        for (i, shuffle_item) in shuffle_data.into_iter().enumerate() {
            let assignments = Arc::clone(&assignments);
            let target_plain_validators = Arc::clone(&plain_validators);
            let target_public_key_bls_list = Arc::clone(&public_key_bls_list);
    
            let handle = thread::spawn(move || {
                let mut assignment = ShuffleCircuit::<M31>::default();
                assignment.from_plains(shuffle_item, &target_plain_validators, &target_public_key_bls_list);
    
                let mut assignments = assignments.lock().unwrap();
                assignments[i] = Some(assignment);
            });
    
            handles.push(handle);
        }
    
        for handle in handles {
            handle.join().expect("Thread panicked");
        }
    
        let end_time = std::time::Instant::now();
        println!("assigned assignment data, time: {:?}", end_time.duration_since(start_time));


        let assignments = assignments.lock().unwrap().iter().map(|x| x.clone().unwrap()).collect::<Vec<_>>();
        let assignment_chunks: Vec<Vec<ShuffleCircuit<M31>>> =
            assignments.chunks(16).map(|x| x.to_vec()).collect();
        println!("assignment_chunks.len:{}", assignment_chunks.len());
        let handles = assignment_chunks
            .into_iter()
            .enumerate()
            .map(|(i, assignments)| {
                let witness_solver = Arc::clone(&witness_solver);
                thread::spawn(move || {
                    let mut hint_registry1 = HintRegistry::<M31>::new();
                    register_hint(&mut hint_registry1);
                    let witness = witness_solver.solve_witnesses_with_hints(&assignments, &mut hint_registry1).unwrap();
                    let file_name = format!("./witnesses/shuffle/witness_{}.txt", i);
                    let file = std::fs::File::create(file_name).unwrap();
                    let writer = std::io::BufWriter::new(file);
                    witness.serialize_into(writer).unwrap();
                }
                )
            })
            .collect::<Vec<_>>();
        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.join().unwrap());
        }
        let end_time = std::time::Instant::now();
        println!("Generate shuffle witness Time: {:?}", end_time.duration_since(start_time));
    });
}


#[test]
fn test_generate_shuffle_witnesses(){
    generate_shuffle_witnesses("");
}