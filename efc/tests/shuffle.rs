use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use circuit_std_rs::gnark::emulated::sw_bls12381::g1::*;
use circuit_std_rs::utils::{register_hint, simple_select};
use efc::bls::check_pubkey_key_bls;
use efc::shuffle::*;
use efc::utils::read_from_json_file;
use efc::validator::{read_validators, ValidatorPlain, ValidatorSSZ};
use expander_compiler::frontend::extra::*;
use expander_compiler::frontend::*;

// Define defines the circuit
declare_circuit!(ShuffleCircuit {
    start_index: Variable,
    chunk_length: Variable,
    shuffle_indices: [Variable; VALIDATOR_CHUNK_SIZE],
    committee_indices: [Variable; VALIDATOR_CHUNK_SIZE],
    pivots: [Variable; SHUFFLE_ROUND],
    index_count: Variable,
    position_results: [Variable; SHUFFLE_ROUND * VALIDATOR_CHUNK_SIZE],
    position_bit_results: [Variable; SHUFFLE_ROUND * VALIDATOR_CHUNK_SIZE],
    flip_results: [Variable; SHUFFLE_ROUND * VALIDATOR_CHUNK_SIZE],
    slot: Variable,
    aggregation_bits: [Variable; VALIDATOR_CHUNK_SIZE],
    validator_hashes: [[Variable; POSEIDON_HASH_LENGTH]; VALIDATOR_CHUNK_SIZE],
    aggregated_pubkey: [[Variable; 48]; 2],
    attestation_balance: [Variable; 8],
    pubkeys_bls: [[[Variable; 48]; 2]; VALIDATOR_CHUNK_SIZE],
    // validators:      [ValidatorSSZ;VALIDATOR_CHUNK_SIZE],
    pubkey: [[Variable; 48]; VALIDATOR_CHUNK_SIZE],
    withdrawal_credentials: [[Variable; 32]; VALIDATOR_CHUNK_SIZE],
    effective_balance: [[Variable; 8]; VALIDATOR_CHUNK_SIZE],
    slashed: [[Variable; 1]; VALIDATOR_CHUNK_SIZE],
    activation_eligibility_epoch: [[Variable; 8]; VALIDATOR_CHUNK_SIZE],
    activation_epoch: [[Variable; 8]; VALIDATOR_CHUNK_SIZE],
    exit_epoch: [[Variable; 8]; VALIDATOR_CHUNK_SIZE],
    withdrawable_epoch: [[Variable; 8]; VALIDATOR_CHUNK_SIZE],
});

impl ShuffleCircuit<M31> {
    pub fn from_plains(
        &mut self,
        shuffle_json: ShuffleJson,
        plain_validators: &Vec<ValidatorPlain>,
        pubkey_bls: &Vec<Vec<String>>,
    ) {
        if shuffle_json.committee_indices.len() != VALIDATOR_CHUNK_SIZE {
            panic!("committee_indices length is not equal to VALIDATOR_CHUNK_SIZE");
        }
        //assign shuffle_json
        self.start_index = M31::from(shuffle_json.start_index);
        self.chunk_length = M31::from(shuffle_json.chunk_length);
        for i in 0..VALIDATOR_CHUNK_SIZE {
            self.shuffle_indices[i] = M31::from(shuffle_json.shuffle_indices[i]);
            self.committee_indices[i] = M31::from(shuffle_json.committee_indices[i]);
            self.aggregation_bits[i] = M31::from(shuffle_json.aggregation_bits[i]);
        }
        for i in 0..SHUFFLE_ROUND {
            self.pivots[i] = M31::from(shuffle_json.pivots[i]);
        }
        self.index_count = M31::from(shuffle_json.index_count);
        for i in 0..SHUFFLE_ROUND * VALIDATOR_CHUNK_SIZE {
            self.position_results[i] = M31::from(shuffle_json.position_results[i]);
            self.position_bit_results[i] = M31::from(shuffle_json.position_bit_results[i]);
            self.flip_results[i] = M31::from(shuffle_json.flip_results[i]);
        }
        self.slot = M31::from(shuffle_json.slot);

        //assign validator_hashes
        for i in 0..VALIDATOR_CHUNK_SIZE {
            for j in 0..POSEIDON_HASH_LENGTH {
                self.validator_hashes[i][j] = M31::from(shuffle_json.validator_hashes[i][j]);
            }
        }

        //assign aggregated_pubkey
        let pubkey = &shuffle_json.aggregated_pubkey;
        for i in 0..48 {
            self.aggregated_pubkey[0][i] = M31::from(pubkey.x.limbs[i] as u32);
            self.aggregated_pubkey[1][i] = M31::from(pubkey.y.limbs[i] as u32);
        }

        //assign attestation_balance
        for i in 0..8 {
            self.attestation_balance[i] = M31::from(shuffle_json.attestation_balance[i]);
        }

        for i in 0..VALIDATOR_CHUNK_SIZE {
            //assign pubkey_bls
            let raw_pubkey_bls = &pubkey_bls[shuffle_json.committee_indices[i] as usize];
            let pubkey_bls_x = STANDARD.decode(&raw_pubkey_bls[0]).unwrap();
            let pubkey_bls_y = STANDARD.decode(&raw_pubkey_bls[1]).unwrap();
            for k in 0..48 {
                self.pubkeys_bls[i][0][k] = M31::from(pubkey_bls_x[47 - k] as u32);
                self.pubkeys_bls[i][1][k] = M31::from(pubkey_bls_y[47 - k] as u32);
            }

            //assign validator
            let validator = plain_validators[shuffle_json.committee_indices[i] as usize].clone();

            //assign pubkey
            let raw_pubkey = validator.public_key.clone();
            let pubkey = STANDARD.decode(raw_pubkey).unwrap();
            for j in 0..48 {
                self.pubkey[i][j] = M31::from(pubkey[j] as u32);
            }
            //assign withdrawal_credentials
            let raw_withdrawal_credentials = validator.withdrawal_credentials.clone();
            let withdrawal_credentials = STANDARD.decode(raw_withdrawal_credentials).unwrap();
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
                self.activation_eligibility_epoch[i][j] =
                    M31::from(activation_eligibility_epoch[j] as u32);
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
    pub fn from_pubkey_bls(&mut self, committee_indices: Vec<u32>, pubkey_bls: Vec<Vec<String>>) {
        for i in 0..VALIDATOR_CHUNK_SIZE {
            let pubkey = &pubkey_bls[committee_indices[i] as usize];
            let pubkey_x = STANDARD.decode(&pubkey[0]).unwrap();
            let pubkey_y = STANDARD.decode(&pubkey[1]).unwrap();
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

        let mut indices_chunk = get_indice_chunk(
            builder,
            self.start_index,
            self.chunk_length,
            VALIDATOR_CHUNK_SIZE,
        );

        //set padding indices to 0
        let zero_var = builder.constant(0);
        for i in 0..indices_chunk.len() {
            let tmp = builder.add(self.flip_results[i], 1);
            let ignore_flag = builder.is_zero(tmp);
            indices_chunk[i] =
                simple_select(builder, ignore_flag, zero_var.clone(), indices_chunk[i]);
        }
        //flip the indices based on the hashbit
        let mut cur_indices = indices_chunk.clone();
        let mut copy_cur_indices =
            builder.new_hint("myhint.copyvarshint", &cur_indices, cur_indices.len());
        for i in 0..SHUFFLE_ROUND {
            let (cur_indices, diffs) = flip_with_hash_bits(
                builder,
                self.pivots[i],
                self.index_count,
                &copy_cur_indices,
                &self.position_results[i * VALIDATOR_CHUNK_SIZE..(i + 1) * VALIDATOR_CHUNK_SIZE],
                &self.position_bit_results
                    [i * VALIDATOR_CHUNK_SIZE..(i + 1) * VALIDATOR_CHUNK_SIZE],
                &self.flip_results[i * VALIDATOR_CHUNK_SIZE..(i + 1) * VALIDATOR_CHUNK_SIZE],
            );
            for diff in diffs {
                g1.curve_f.table.rangeproof(builder, diff, MAX_VALIDATOR_EXP);
            }
            copy_cur_indices =
                builder.new_hint("myhint.copyvarshint", &cur_indices, cur_indices.len());
        }

        //check the final curIndices, should be equal to the shuffleIndex
        for i in 0..self.shuffle_indices.len() {
            let tmp = builder.add(self.flip_results[i], 1);
            let is_minus_one = builder.is_zero(tmp);
            cur_indices[i] = simple_select(
                builder,
                is_minus_one,
                self.shuffle_indices[i],
                cur_indices[i],
            );
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
            let pubkey_g1 = G1Affine::from_vars(
                self.pubkeys_bls[i][0].to_vec(),
                self.pubkeys_bls[i][1].to_vec(),
            );
            let logup_var = check_pubkey_key_bls(builder, pubkey_list[i].to_vec(), &pubkey_g1);
            g1.curve_f.table.rangeproof(builder, logup_var, 5);
            pubkey_list_bls.push(pubkey_g1);
        }

        let mut aggregated_pubkey = G1Affine::from_vars(
            self.aggregated_pubkey[0].to_vec(),
            self.aggregated_pubkey[1].to_vec(),
        );
        aggregate_attestation_public_key(
            builder,
            &mut g1,
            &pubkey_list_bls,
            &self.aggregation_bits,
            &mut aggregated_pubkey,
        );

        for index in 0..VALIDATOR_CHUNK_SIZE {
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
                validator.activation_eligibility_epoch[i] =
                    self.activation_eligibility_epoch[index][i];
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

#[test]
fn read_json_to_shuffle() {
    let plain_validators = read_validators("");
    let file_path = "shuffle_assignment.json";
    let shuffle_data: Vec<ShuffleJson> = read_from_json_file(file_path).unwrap();
    let file_path = "pubkeyBLSList.json";
    let public_key_bls_list: Vec<Vec<String>> = read_from_json_file(file_path).unwrap();

    let mut hint_registry = HintRegistry::<M31>::new();
    register_hint(&mut hint_registry);
    let mut assignment = ShuffleCircuit::<M31>::default();
    assignment.from_plains(
        shuffle_data[shuffle_data.len() - 1].clone(),
        &plain_validators,
        &public_key_bls_list,
    );

    stacker::grow(32 * 1024 * 1024 * 1024, || {
        debug_eval(&ShuffleCircuit::default(), &assignment, hint_registry)
    });
}

#[test]
fn test_generate_shuffle_witnesses() {
    generate_shuffle_witnesses("");
}
