use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use circuit_std_rs::poseidon::poseidon_m31::*;
use circuit_std_rs::poseidon::poseidon_u32::PoseidonParams;
use circuit_std_rs::poseidon::utils::*;
use circuit_std_rs::sha256;
use circuit_std_rs::utils::register_hint;
use circuit_std_rs::utils::simple_select;
use expander_compiler::frontend::*;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::beacon;
use crate::merkle;
use crate::utils::*;
pub const SUBTREE_DEPTH: usize = 10;
pub const SUBTREE_NUM_DEPTH: usize = 11;
pub const SUBTREE_SIZE: usize = 1 << SUBTREE_DEPTH;
pub const SUBTREE_NUM: usize = 1 << SUBTREE_NUM_DEPTH;

// SUBTREENUMDEPTH           = 11
// SUBTREENUM                = 1 << SUBTREENUMDEPTH
// PADDINGDEPTH              = MAXBEACONVALIDATORDEPTH - SUBTREEDEPTH - SUBTREENUMDEPTH
pub const PADDING_DEPTH: usize =
    beacon::MAXBEACONVALIDATORDEPTH - SUBTREE_DEPTH - SUBTREE_NUM_DEPTH;
#[derive(Debug, Deserialize, Clone)]
pub struct ValidatorPlain {
    #[serde(default)]
    pub public_key: String,
    #[serde(default)]
    pub withdrawal_credentials: String,
    #[serde(default)]
    pub effective_balance: u64,
    #[serde(default)]
    pub slashed: bool,
    #[serde(default)]
    pub activation_eligibility_epoch: u64,
    #[serde(default)]
    pub activation_epoch: u64,
    #[serde(default)]
    pub exit_epoch: u64,
    #[serde(default)]
    pub withdrawable_epoch: u64,
}
impl ValidatorPlain {
    pub fn hash(&self) -> Vec<u32> {
        let pubkey = STANDARD.decode(self.public_key.clone()).unwrap();
        let withdrawal_credentials = STANDARD
            .decode(self.withdrawal_credentials.clone())
            .unwrap();

        let mut inputs: Vec<u8> = Vec::with_capacity(
            48 + 32 + 8 + 1 + 8 + 8 + 8 + 8, // total expected bytes
        );

        inputs.extend_from_slice(&pubkey);
        inputs.extend_from_slice(&withdrawal_credentials);
        inputs.extend_from_slice(&self.effective_balance.to_le_bytes());
        inputs.extend_from_slice(&if self.slashed { 1u64 } else { 0 }.to_le_bytes());
        inputs.extend_from_slice(&self.activation_eligibility_epoch.to_le_bytes());
        inputs.extend_from_slice(&self.activation_epoch.to_le_bytes());
        inputs.extend_from_slice(&self.exit_epoch.to_le_bytes());
        inputs.extend_from_slice(&self.withdrawable_epoch.to_le_bytes());
        let params = PoseidonParams::new(
            POSEIDON_M31X16_RATE,
            16,
            POSEIDON_M31X16_FULL_ROUNDS,
            POSEIDON_M31X16_PARTIAL_ROUNDS,
        );
        let inputs_u32 = inputs.iter().map(|x| *x as u32).collect::<Vec<u32>>();
        params.hash_to_state(&inputs_u32)
    }
}
#[derive(Clone, Copy)]
pub struct ValidatorSSZ {
    pub public_key: [Variable; 48],
    pub withdrawal_credentials: [Variable; 32],
    pub effective_balance: [Variable; 8],
    pub slashed: [Variable; 1],
    pub activation_eligibility_epoch: [Variable; 8],
    pub activation_epoch: [Variable; 8],
    pub exit_epoch: [Variable; 8],
    pub withdrawable_epoch: [Variable; 8],
}
impl Default for ValidatorSSZ {
    fn default() -> Self {
        Self {
            public_key: [Variable::default(); 48],
            withdrawal_credentials: [Variable::default(); 32],
            effective_balance: [Variable::default(); 8],
            slashed: [Variable::default(); 1],
            activation_eligibility_epoch: [Variable::default(); 8],
            activation_epoch: [Variable::default(); 8],
            exit_epoch: [Variable::default(); 8],
            withdrawable_epoch: [Variable::default(); 8],
        }
    }
}
impl ValidatorSSZ {
    pub fn new() -> Self {
        Self {
            public_key: [Variable::default(); 48],
            withdrawal_credentials: [Variable::default(); 32],
            effective_balance: [Variable::default(); 8],
            slashed: [Variable::default(); 1],
            activation_eligibility_epoch: [Variable::default(); 8],
            activation_epoch: [Variable::default(); 8],
            exit_epoch: [Variable::default(); 8],
            withdrawable_epoch: [Variable::default(); 8],
        }
    }
    pub fn poseidon_hash<C: Config, B: RootAPI<C>>(&self, builder: &mut B) -> Vec<Variable> {
        let inputs = [
            &self.public_key[..],
            &self.withdrawal_credentials[..],
            &self.effective_balance[..],
            &self.slashed[..],
            &self.activation_eligibility_epoch[..],
            &self.activation_epoch[..],
            &self.exit_epoch[..],
            &self.withdrawable_epoch[..],
        ]
        .concat();
        let params = PoseidonM31Params::new(
            builder,
            POSEIDON_M31X16_RATE,
            16,
            POSEIDON_M31X16_FULL_ROUNDS,
            POSEIDON_M31X16_PARTIAL_ROUNDS,
        );
        params.hash_to_state_flatten(builder, &inputs)[..POSEIDON_M31X16_RATE].to_vec()
    }
    pub fn sha256_hash<C: Config, B: RootAPI<C>>(&self, builder: &mut B) -> Vec<Variable> {
        let inputs = [
            &self.public_key[..],
            &self.withdrawal_credentials[..],
            &self.effective_balance[..],
            &self.slashed[..],
            &self.activation_eligibility_epoch[..],
            &self.activation_epoch[..],
            &self.exit_epoch[..],
            &self.withdrawable_epoch[..],
        ]
        .concat();
        sha256::m31::sha256_var_bytes(builder, &inputs)
    }
}
declare_circuit!(UpdateValidatorTreeCircuit {
    index: Variable, //validator index
    // old validator
    old_pubkey: [Variable; 48],
    old_withdrawal_credentials: [Variable; 32],
    old_effective_balance: [Variable; 8], //all -1 if empty
    old_slashed: [Variable; 1],
    old_activation_eligibility_epoch: [Variable; 8],
    old_activation_epoch: [Variable; 8],
    old_exit_epoch: [Variable; 8],
    old_withdrawable_epoch: [Variable; 8],
    // new validator
    new_pubkey: [Variable; 48],
    new_withdrawal_credentials: [Variable; 32],
    new_effective_balance: [Variable; 8],
    new_slashed: [Variable; 1],
    new_activation_eligibility_epoch: [Variable; 8],
    new_activation_epoch: [Variable; 8],
    new_exit_epoch: [Variable; 8],
    new_withdrawable_epoch: [Variable; 8],
    //sha256 tree path, aunts, root hash
    sha256_path: [Variable; beacon::MAXBEACONVALIDATORDEPTH],
    sha256_aunts: [[Variable; 32]; beacon::MAXBEACONVALIDATORDEPTH],
    sha256_root_hash: [Variable; 32],
    // next_sha256_root_hash: [Variable; 32],
    sha256_root_mix_in: [Variable; 32],
    next_sha256_root_mix_in: [Variable; 32],
    //poseidon tree path, aunts, root hash
    poseidon_path: [Variable; beacon::MAXBEACONVALIDATORDEPTH],
    poseidon_aunts: [[Variable; POSEIDON_M31X16_RATE]; beacon::MAXBEACONVALIDATORDEPTH],
    poseidon_root_hash: [Variable; POSEIDON_M31X16_RATE],
    // next_poseidon_root_hash: [Variable; POSEIDON_M31X16_RATE],
    poseidon_root_mix_in: [Variable; POSEIDON_M31X16_RATE],
    next_poseidon_root_mix_in: [Variable; POSEIDON_M31X16_RATE],
    //validatorlist.len()
    validator_count: [Variable; 8],
    next_validator_count: [Variable; 8],
});
impl GenericDefine<M31Config> for UpdateValidatorTreeCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        //if this is a new "insert" validator, then set the old validator hash to all 0
        let mut validator_count_var = builder.constant(0);
        let mut next_validator_count_var = builder.constant(0);
        let zero_var = builder.constant(0);
        for i in 0..8 {
            if i < 3 {
                validator_count_var = builder.add(self.validator_count[i], validator_count_var);
                validator_count_var = builder.mul(validator_count_var, 1 << 8);
                next_validator_count_var =
                    builder.add(self.next_validator_count[i], next_validator_count_var);
                next_validator_count_var = builder.mul(next_validator_count_var, 1 << 8);
            } else {
                //assume the validator count is less than 2^24
                builder.assert_is_equal(self.validator_count[i], zero_var);
                builder.assert_is_equal(self.next_validator_count[i], zero_var);
            }
        }
        let validator_count_diff = builder.sub(next_validator_count_var, validator_count_var);
        builder.assert_is_bool(validator_count_diff);
        let modified_validator_flag = builder.is_zero(validator_count_diff); //if the index is equal to the validator count, then it is a new insert validator
        let old_validator_ssz = ValidatorSSZ {
            public_key: self.old_pubkey,
            withdrawal_credentials: self.old_withdrawal_credentials,
            effective_balance: self.old_effective_balance,
            slashed: self.old_slashed,
            activation_eligibility_epoch: self.old_activation_eligibility_epoch,
            activation_epoch: self.old_activation_epoch,
            exit_epoch: self.old_exit_epoch,
            withdrawable_epoch: self.old_withdrawable_epoch,
        };
        let new_validator_ssz = ValidatorSSZ {
            public_key: self.new_pubkey,
            withdrawal_credentials: self.new_withdrawal_credentials,
            effective_balance: self.new_effective_balance,
            slashed: self.new_slashed,
            activation_eligibility_epoch: self.new_activation_eligibility_epoch,
            activation_epoch: self.new_activation_epoch,
            exit_epoch: self.new_exit_epoch,
            withdrawable_epoch: self.new_withdrawable_epoch,
        };
        let mut old_validator_sha256_hash = old_validator_ssz.sha256_hash(builder);
        //if this is a new "insert" validator, then set the old validator hash to all 0
        old_validator_sha256_hash
            .iter_mut()
            .for_each(|x| *x = simple_select(builder, modified_validator_flag, *x, zero_var));
        let new_validator_sha256_hash = new_validator_ssz.sha256_hash(builder);
        let mut old_validator_poseidon_hash = old_validator_ssz.poseidon_hash(builder);
        //if this is a new "insert" validator, then set the old validator hash to all 0
        old_validator_poseidon_hash
            .iter_mut()
            .for_each(|x| *x = simple_select(builder, modified_validator_flag, *x, zero_var));

        let new_validator_poseidon_hash = new_validator_ssz.poseidon_hash(builder);
        let params = PoseidonM31Params::new(
            builder,
            POSEIDON_M31X16_RATE,
            16,
            POSEIDON_M31X16_FULL_ROUNDS,
            POSEIDON_M31X16_PARTIAL_ROUNDS,
        );

        //############ SHA256 ############
        let aunts_vec: Vec<Vec<Variable>> =
            self.sha256_aunts.iter().map(|arr| arr.to_vec()).collect();
        //make sure the old one is correct
        merkle::verify_merkle_tree_path_var(
            builder,
            &self.sha256_root_hash,
            &old_validator_sha256_hash,
            &self.sha256_path,
            &aunts_vec,
            &params,
            zero_var,
        );
        //check mixin
        let mut mixin_input = self.sha256_root_hash.to_vec();
        mixin_input.extend_from_slice(&self.validator_count);
        let tree_root_mix_in = sha256::m31::sha256_var_bytes(builder, &mixin_input);
        (0..32)
            .for_each(|i| builder.assert_is_equal(tree_root_mix_in[i], self.sha256_root_mix_in[i]));

        //make sure the new one is correct
        let new_sha256_root_hash = merkle::calculate_merkle_tree_root_var(
            builder,
            &aunts_vec,
            &self.sha256_path,
            new_validator_sha256_hash,
            &params,
        );
        //check mixin
        let mut mixin_input = new_sha256_root_hash;
        mixin_input.extend_from_slice(&self.next_validator_count);
        let new_tree_root_mix_in = sha256::m31::sha256_var_bytes(builder, &mixin_input);
        (0..32).for_each(|i| {
            builder.assert_is_equal(new_tree_root_mix_in[i], self.next_sha256_root_mix_in[i])
        });

        // //############ POSEIDON ############
        let aunts_vec: Vec<Vec<Variable>> =
            self.poseidon_aunts.iter().map(|arr| arr.to_vec()).collect();
        //make sure the old one is correct
        merkle::verify_merkle_tree_path_var(
            builder,
            &self.poseidon_root_hash,
            &old_validator_poseidon_hash,
            &self.poseidon_path,
            &aunts_vec,
            &params,
            zero_var,
        );
        //check mixin
        let mut mixin_input = self.poseidon_root_hash.to_vec();
        mixin_input.extend_from_slice(&self.validator_count);
        let tree_root_mix_in = params.hash_to_state_flatten(builder, &mixin_input);
        (0..POSEIDON_M31X16_RATE).for_each(|i| {
            builder.assert_is_equal(tree_root_mix_in[i], self.poseidon_root_mix_in[i])
        });
        //make sure the new one is correct
        let new_poseidon_root_hash = merkle::calculate_merkle_tree_root_var(
            builder,
            &aunts_vec,
            &self.poseidon_path,
            new_validator_poseidon_hash,
            &params,
        );
        //check mixin
        let mut mixin_input = new_poseidon_root_hash;
        mixin_input.extend_from_slice(&self.next_validator_count);
        let new_tree_root_mix_in = params.hash_to_state_flatten(builder, &mixin_input);
        (0..POSEIDON_M31X16_RATE).for_each(|i| {
            builder.assert_is_equal(new_tree_root_mix_in[i], self.next_poseidon_root_mix_in[i])
        });
    }
}
pub struct MerkleProof {
    pub path: Vec<u32>,
    pub aunts: Vec<Vec<u32>>,
    pub root_hash: Vec<u32>,
    pub root_mix_in: Vec<u32>,
    pub next_root_mix_in: Vec<u32>,
}
impl UpdateValidatorTreeCircuit<M31> {
    pub fn from_beacon(
        validator_index: usize,
        old_validator: ValidatorPlain,
        new_validator: ValidatorPlain,
        sha256_merkle_proof: MerkleProof,
        poseidon_merkle_proof: MerkleProof,
        validator_count: u64,
        next_validator_count: u64,
    ) -> Self {
        let mut assignment = Self::default();
        //assign pubkey
        let raw_pubkey = old_validator.public_key.clone();
        let pubkey = STANDARD.decode(raw_pubkey).unwrap();
        for (i, pubkey_byte) in pubkey.iter().enumerate().take(48) {
            assignment.old_pubkey[i] = M31::from(*pubkey_byte as u32);
        }
        //assign withdrawal_credentials
        let raw_withdrawal_credentials = old_validator.withdrawal_credentials.clone();
        let withdrawal_credentials = STANDARD.decode(raw_withdrawal_credentials).unwrap();
        for (i, withdrawal_credentials_byte) in withdrawal_credentials.iter().enumerate().take(32) {
            assignment.old_withdrawal_credentials[i] =
                M31::from(*withdrawal_credentials_byte as u32);
        }
        //assign effective_balance
        let effective_balance = old_validator.effective_balance.to_le_bytes();
        for (i, effective_balance_byte) in effective_balance.iter().enumerate() {
            assignment.old_effective_balance[i] = M31::from(*effective_balance_byte as u32);
        }
        //assign slashed
        let slashed = if old_validator.slashed { 1 } else { 0 };
        assignment.old_slashed[0] = M31::from(slashed);
        //assign activation_eligibility_epoch
        let activation_eligibility_epoch = old_validator.activation_eligibility_epoch.to_le_bytes();
        for (i, activation_eligibility_epoch_byte) in
            activation_eligibility_epoch.iter().enumerate()
        {
            assignment.old_activation_eligibility_epoch[i] =
                M31::from(*activation_eligibility_epoch_byte as u32);
        }
        //assign activation_epoch
        let activation_epoch = old_validator.activation_epoch.to_le_bytes();
        for (i, activation_epoch_byte) in activation_epoch.iter().enumerate() {
            assignment.old_activation_epoch[i] = M31::from(*activation_epoch_byte as u32);
        }
        //assign exit_epoch
        let exit_epoch = old_validator.exit_epoch.to_le_bytes();
        for (i, exit_epoch_byte) in exit_epoch.iter().enumerate() {
            assignment.old_exit_epoch[i] = M31::from(*exit_epoch_byte as u32);
        }
        //assign withdrawable_epoch
        let withdrawable_epoch = old_validator.withdrawable_epoch.to_le_bytes();
        for (i, withdrawable_epoch_byte) in withdrawable_epoch.iter().enumerate() {
            assignment.old_withdrawable_epoch[i] = M31::from(*withdrawable_epoch_byte as u32);
        }

        //assign pubkey
        let raw_pubkey = new_validator.public_key.clone();
        let pubkey = STANDARD.decode(raw_pubkey).unwrap();
        for (i, pubkey_byte) in pubkey.iter().enumerate().take(48) {
            assignment.new_pubkey[i] = M31::from(*pubkey_byte as u32);
        }
        //assign withdrawal_credentials
        let raw_withdrawal_credentials = new_validator.withdrawal_credentials.clone();
        let withdrawal_credentials = STANDARD.decode(raw_withdrawal_credentials).unwrap();
        for (i, withdrawal_credentials_byte) in withdrawal_credentials.iter().enumerate().take(32) {
            assignment.new_withdrawal_credentials[i] =
                M31::from(*withdrawal_credentials_byte as u32);
        }
        //assign effective_balance
        let effective_balance = new_validator.effective_balance.to_le_bytes();
        for (i, effective_balance_byte) in effective_balance.iter().enumerate() {
            assignment.new_effective_balance[i] = M31::from(*effective_balance_byte as u32);
        }
        //assign slashed
        let slashed = if new_validator.slashed { 1 } else { 0 };
        assignment.new_slashed[0] = M31::from(slashed);
        //assign activation_eligibility_epoch
        let activation_eligibility_epoch = new_validator.activation_eligibility_epoch.to_le_bytes();
        for (i, activation_eligibility_epoch_byte) in
            activation_eligibility_epoch.iter().enumerate()
        {
            assignment.new_activation_eligibility_epoch[i] =
                M31::from(*activation_eligibility_epoch_byte as u32);
        }
        //assign activation_epoch
        let activation_epoch = new_validator.activation_epoch.to_le_bytes();
        for (i, activation_epoch_byte) in activation_epoch.iter().enumerate() {
            assignment.new_activation_epoch[i] = M31::from(*activation_epoch_byte as u32);
        }
        //assign exit_epoch
        let exit_epoch = new_validator.exit_epoch.to_le_bytes();
        for (i, exit_epoch_byte) in exit_epoch.iter().enumerate() {
            assignment.new_exit_epoch[i] = M31::from(*exit_epoch_byte as u32);
        }
        //assign withdrawable_epoch
        let withdrawable_epoch = new_validator.withdrawable_epoch.to_le_bytes();
        for (i, withdrawable_epoch_byte) in withdrawable_epoch.iter().enumerate() {
            assignment.new_withdrawable_epoch[i] = M31::from(*withdrawable_epoch_byte as u32);
        }

        //assign the rest
        assignment.index = M31::from(validator_index as u32);
        for i in 0..beacon::MAXBEACONVALIDATORDEPTH {
            if i < sha256_merkle_proof.path.len() {
                for j in 0..32 {
                    if j < sha256_merkle_proof.aunts[i].len() {
                        assignment.sha256_aunts[i][j] = M31::from(sha256_merkle_proof.aunts[i][j]);
                    } else {
                        assignment.sha256_aunts[i][j] = M31::from(0);
                    }
                }
                assignment.sha256_path[i] = M31::from(sha256_merkle_proof.path[i]);
            } else {
                for j in 0..32 {
                    assignment.sha256_aunts[i][j] = M31::from(0);
                }
                assignment.sha256_path[i] = M31::from(0);
            }
            if i < poseidon_merkle_proof.path.len() {
                for j in 0..POSEIDON_M31X16_RATE {
                    if j < poseidon_merkle_proof.aunts[i].len() {
                        assignment.poseidon_aunts[i][j] =
                            M31::from(poseidon_merkle_proof.aunts[i][j]);
                    } else {
                        assignment.poseidon_aunts[i][j] = M31::from(0);
                    }
                }
                assignment.poseidon_path[i] = M31::from(poseidon_merkle_proof.path[i]);
            } else {
                for j in 0..POSEIDON_M31X16_RATE {
                    assignment.poseidon_aunts[i][j] = M31::from(0);
                }
                assignment.poseidon_path[i] = M31::from(0);
            }
        }
        for i in 0..32 {
            assignment.sha256_root_hash[i] = M31::from(sha256_merkle_proof.root_hash[i]);
            assignment.sha256_root_mix_in[i] = M31::from(sha256_merkle_proof.root_mix_in[i]);
            assignment.next_sha256_root_mix_in[i] =
                M31::from(sha256_merkle_proof.next_root_mix_in[i]);
        }
        for i in 0..POSEIDON_M31X16_RATE {
            assignment.poseidon_root_hash[i] = M31::from(poseidon_merkle_proof.root_hash[i]);
            assignment.poseidon_root_mix_in[i] = M31::from(poseidon_merkle_proof.root_mix_in[i]);
            assignment.next_poseidon_root_mix_in[i] =
                M31::from(poseidon_merkle_proof.next_root_mix_in[i]);
        }
        let validator_count = validator_count.to_le_bytes();
        for (i, validator_count_byte) in validator_count.iter().enumerate() {
            assignment.validator_count[i] = M31::from(*validator_count_byte as u32);
        }
        let next_validator_count = next_validator_count.to_le_bytes();
        for (i, next_validator_count_byte) in next_validator_count.iter().enumerate() {
            assignment.next_validator_count[i] = M31::from(*next_validator_count_byte as u32);
        }
        assignment
    }
}
#[derive(Debug, Deserialize, Clone)]
pub struct ValidatorSubTreeJson {
    #[serde(rename = "ValidatorHashChunk")]
    pub validators_hash_chunk: Vec<Vec<u32>>,
    #[serde(rename = "SubtreeRoot")]
    pub subtree_root: Vec<u32>,
}

pub type ValidatorSubMTAssignmentChunks = Vec<Vec<ValidatorSubMTCircuit<M31>>>;
declare_circuit!(ValidatorSubMTCircuit {
    validator_hash_chunk: [[Variable; POSEIDON_M31X16_RATE]; SUBTREE_SIZE],
    subtree_root: [Variable; POSEIDON_M31X16_RATE], // Public input
});
impl ValidatorSubMTCircuit<M31> {
    pub fn from_assignment(&mut self, validator_subtree_json: &ValidatorSubTreeJson) {
        for i in 0..SUBTREE_SIZE {
            for j in 0..POSEIDON_M31X16_RATE {
                self.validator_hash_chunk[i][j] =
                    M31::from(validator_subtree_json.validators_hash_chunk[i][j]);
            }
        }
        for i in 0..POSEIDON_M31X16_RATE {
            self.subtree_root[i] = M31::from(validator_subtree_json.subtree_root[i]);
        }
    }
    pub fn get_assignments_from_data(
        validator_subtree_data: Vec<ValidatorSubTreeJson>,
    ) -> Vec<Self> {
        let mut handles = vec![];
        let assignments = Arc::new(Mutex::new(vec![None; validator_subtree_data.len()]));

        for (i, validator_subtree_item) in validator_subtree_data.into_iter().enumerate() {
            let assignments = Arc::clone(&assignments);

            let handle = thread::spawn(move || {
                let mut assignment = ValidatorSubMTCircuit::<M31>::default();
                assignment.from_assignment(&validator_subtree_item);

                let mut assignments = assignments.lock().unwrap();
                assignments[i] = Some(assignment);
            });

            handles.push(handle);
        }
        for handle in handles {
            handle.join().expect("Thread panicked");
        }

        let assignments = assignments
            .lock()
            .unwrap()
            .iter()
            .map(|x| x.clone().unwrap())
            .collect::<Vec<_>>();
        assignments
    }
    pub fn get_assignments_from_json(dir: &str) -> Vec<Self> {
        let file_path = format!("{}/validatorsubtree_assignment.json", dir);
        let validator_subtree_data: Vec<ValidatorSubTreeJson> =
            read_from_json_file(&file_path).unwrap();
        Self::get_assignments_from_data(validator_subtree_data)
    }
    pub fn get_assignments_from_beacon_data(validator_tree: &[Vec<Vec<u32>>]) -> Vec<Self> {
        let validator_hashes = validator_tree.last().unwrap();
        let mut assignments = vec![];
        for i in 0..SUBTREE_NUM {
            let mut assignment = ValidatorSubMTCircuit::<M31>::default();
            for j in 0..SUBTREE_SIZE {
                for k in 0..POSEIDON_M31X16_RATE {
                    assignment.validator_hash_chunk[j][k] = M31::from(
                        *validator_hashes
                            .get(i * SUBTREE_SIZE + j)
                            .and_then(|row| row.get(k))
                            .unwrap_or(&0),
                    );
                }
            }
            for j in 0..POSEIDON_M31X16_RATE {
                assignment.subtree_root[j] =
                    M31::from(validator_tree[validator_tree.len() - SUBTREE_DEPTH][i][j]);
            }
            assignments.push(assignment);
        }
        assignments
    }
}
impl GenericDefine<M31Config> for ValidatorSubMTCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let mut inputs = vec![];

        // Flatten `validator_hash_chunk` into a single input vector
        for i in 0..SUBTREE_SIZE {
            inputs.extend_from_slice(&self.validator_hash_chunk[i]);
        }

        // Compute the Poseidon hash
        let params = PoseidonM31Params::new(
            builder,
            POSEIDON_M31X16_RATE,
            16,
            POSEIDON_M31X16_FULL_ROUNDS,
            POSEIDON_M31X16_PARTIAL_ROUNDS,
        );
        let sub_tree_root = params.hash_to_state_flatten(builder, &inputs);

        // Enforce equality between computed root and given root
        for (i, elem) in sub_tree_root.iter().enumerate().take(POSEIDON_M31X16_RATE) {
            builder.assert_is_equal(elem, self.subtree_root[i]);
        }
    }
}

pub fn generate_validator_subtree_witnesses(dir: &str) {
    stacker::grow(32 * 1024 * 1024 * 1024, || {
        let circuit_name = &format!("validatorsubtree{}", SUBTREE_SIZE);

        //get solver
        log::debug!("preparing {} solver...", circuit_name);
        let witnesses_dir = format!("./witnesses/{}", circuit_name);
        let w_s = get_solver(
            &witnesses_dir,
            circuit_name,
            ValidatorSubMTCircuit::default(),
        );

        let start_time = std::time::Instant::now();
        let assignments = ValidatorSubMTCircuit::get_assignments_from_json(dir);
        let end_time = std::time::Instant::now();
        log::debug!(
            "assigned validator subtree assignment data, time: {:?}",
            end_time.duration_since(start_time)
        );
        let assignment_chunks: Vec<Vec<ValidatorSubMTCircuit<M31>>> =
            assignments.chunks(16).map(|x| x.to_vec()).collect();

        //generate witnesses (multi-thread)
        log::debug!("Start generating witnesses...");
        let witness_solver = Arc::new(w_s);
        let handles = assignment_chunks
            .into_iter()
            .enumerate()
            .map(|(i, assignments)| {
                let witness_solver = Arc::clone(&witness_solver);
                let witnesses_dir_clone = witnesses_dir.clone();
                thread::spawn(move || {
                    let mut hint_registry = HintRegistry::<M31>::new();
                    register_hint(&mut hint_registry);
                    let witness = witness_solver
                        .solve_witnesses_with_hints(&assignments, &mut hint_registry)
                        .unwrap();
                    write_witness_to_file(
                        &format!("{}/witness_{}.txt", witnesses_dir_clone, i),
                        witness,
                    )
                })
            })
            .collect::<Vec<_>>();
        for handle in handles {
            handle.join().unwrap();
        }
        let end_time = std::time::Instant::now();
        log::debug!(
            "Generate {} witness Time: {:?}",
            circuit_name,
            end_time.duration_since(start_time)
        );
    });
}

pub fn end2end_validator_subtree_witnesses(
    w_s: WitnessSolver<M31Config>,
    validator_subtree_data: Vec<ValidatorSubTreeJson>,
) {
    let circuit_name = &format!("validatorsubtree{}", SUBTREE_SIZE);

    log::debug!("preparing {} solver...", circuit_name);
    let witnesses_dir = format!("./witnesses/{}", circuit_name);

    let start_time = std::time::Instant::now();
    let assignments = ValidatorSubMTCircuit::get_assignments_from_data(validator_subtree_data);
    let end_time = std::time::Instant::now();
    log::debug!(
        "assigned validator subtree assignment data, time: {:?}",
        end_time.duration_since(start_time)
    );
    let assignment_chunks: Vec<Vec<ValidatorSubMTCircuit<M31>>> =
        assignments.chunks(16).map(|x| x.to_vec()).collect();

    //generate witnesses (multi-thread)
    log::debug!("Start generating witnesses...");
    let witness_solver = Arc::new(w_s);
    let handles = assignment_chunks
        .into_iter()
        .enumerate()
        .map(|(i, assignments)| {
            let witness_solver = Arc::clone(&witness_solver);
            let witnesses_dir_clone = witnesses_dir.clone();
            thread::spawn(move || {
                let mut hint_registry = HintRegistry::<M31>::new();
                register_hint(&mut hint_registry);
                let witness = witness_solver
                    .solve_witnesses_with_hints(&assignments, &mut hint_registry)
                    .unwrap();
                write_witness_to_file(
                    &format!("{}/witness_{}.txt", witnesses_dir_clone, i),
                    witness,
                )
            })
        })
        .collect::<Vec<_>>();
    for handle in handles {
        handle.join().unwrap();
    }
    let end_time = std::time::Instant::now();
    log::debug!(
        "Generate {} witness Time: {:?}",
        circuit_name,
        end_time.duration_since(start_time)
    );
}

pub fn end2end_validator_subtree_witnesses_with_assignments(
    w_s: WitnessSolver<M31Config>,
    assignment_chunks: Vec<Vec<ValidatorSubMTCircuit<M31>>>,
) {
    let circuit_name = &format!("validatorsubtree{}", SUBTREE_SIZE);

    log::debug!("preparing {} solver...", circuit_name);
    let witnesses_dir = format!("./witnesses/{}", circuit_name);

    let start_time = std::time::Instant::now();
    //generate witnesses (multi-thread)
    log::debug!("Start generating witnesses...");
    let witness_solver = Arc::new(w_s);
    let handles = assignment_chunks
        .into_iter()
        .enumerate()
        .map(|(i, assignments)| {
            let witness_solver = Arc::clone(&witness_solver);
            let witnesses_dir_clone = witnesses_dir.clone();
            thread::spawn(move || {
                let mut hint_registry = HintRegistry::<M31>::new();
                register_hint(&mut hint_registry);
                let witness = witness_solver
                    .solve_witnesses_with_hints(&assignments, &mut hint_registry)
                    .unwrap();
                write_witness_to_file(
                    &format!("{}/witness_{}.txt", witnesses_dir_clone, i),
                    witness,
                )
            })
        })
        .collect::<Vec<_>>();
    for handle in handles {
        handle.join().unwrap();
    }
    let end_time = std::time::Instant::now();
    log::debug!(
        "Generate {} witness Time: {:?}",
        circuit_name,
        end_time.duration_since(start_time)
    );
}

pub type MergeSubMTLimitAssignmentChunks = Vec<Vec<MergeSubMTLimitCircuit<M31>>>;
declare_circuit!(MergeSubMTLimitCircuit {
    subtree_root: [[Variable; POSEIDON_M31X16_RATE]; SUBTREE_NUM], // public
    tree_root_mix_in: [Variable; POSEIDON_M31X16_RATE],            // public
    real_validator_count: [Variable; 8],                           // public, little endian u64
    tree_root: [Variable; POSEIDON_M31X16_RATE],
    path: [Variable; PADDING_DEPTH],
    aunts: [[Variable; POSEIDON_M31X16_RATE]; PADDING_DEPTH],
});

impl GenericDefine<M31Config> for MergeSubMTLimitCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let mut inputs = vec![];
        for i in 0..SUBTREE_NUM {
            inputs.extend_from_slice(&self.subtree_root[i]);
        }

        let params = PoseidonM31Params::new(
            builder,
            POSEIDON_M31X16_RATE,
            16,
            POSEIDON_M31X16_FULL_ROUNDS,
            POSEIDON_M31X16_PARTIAL_ROUNDS,
        );

        let sub_tree_root_root =
            params.hash_to_state_flatten(builder, &inputs)[..POSEIDON_M31X16_RATE].to_vec();

        let mut aunts_vec = vec![];
        for i in 0..PADDING_DEPTH {
            aunts_vec.push(self.aunts[i].to_vec());
        }

        let ignore_opt = builder.constant(0);
        merkle::verify_merkle_tree_path_var(
            builder,
            &self.tree_root,
            &sub_tree_root_root,
            &self.path,
            &aunts_vec,
            &params,
            ignore_opt,
        );

        let mut mixin_input = vec![];
        mixin_input.extend_from_slice(&self.tree_root);
        mixin_input.extend_from_slice(&self.real_validator_count);
        let tree_root_mix_in = params.hash_to_state_flatten(builder, &mixin_input);

        (0..POSEIDON_M31X16_RATE)
            .for_each(|i| builder.assert_is_equal(tree_root_mix_in[i], self.tree_root_mix_in[i]));
    }
}

impl MergeSubMTLimitCircuit<M31> {
    pub fn get_assignments_from_beacon_data(
        validator_tree: &[Vec<Vec<u32>>],
        real_validator_count: u64,
    ) -> Self {
        let mut assignment = MergeSubMTLimitCircuit::<M31>::default();
        let validator_tree_depth =
            (validator_tree.last().unwrap().len() as f64).log2().ceil() as usize;
        for i in 0..SUBTREE_NUM {
            for j in 0..POSEIDON_M31X16_RATE {
                assignment.subtree_root[i][j] =
                    M31::from(validator_tree[validator_tree.len() - SUBTREE_DEPTH][i][j]);
            }
        }
        for i in 0..PADDING_DEPTH {
            assignment.path[i] = M31::from(0);
            for j in 0..POSEIDON_M31X16_RATE {
                assignment.aunts[i][j] = M31::from(
                    validator_tree[validator_tree.len() - i - validator_tree_depth][1][j],
                );
            }
        }
        for i in 0..POSEIDON_M31X16_RATE {
            assignment.tree_root[i] = M31::from(validator_tree[1][0][i]);
            assignment.tree_root_mix_in[i] = M31::from(validator_tree[0][0][i]);
        }
        let real_validator_count = real_validator_count.to_le_bytes();
        for (i, &v) in real_validator_count.iter().enumerate() {
            assignment.real_validator_count[i] = M31::from(v as u32);
        }
        assignment
    }
}

pub fn end2end_merkle_subtree_with_limit_witnesses_with_assignments(
    w_s: WitnessSolver<M31Config>,
    assignment_chunks: Vec<Vec<MergeSubMTLimitCircuit<M31>>>,
) {
    let circuit_name = &format!("merklesubtree{}", SUBTREE_SIZE);

    log::debug!("preparing {} solver...", circuit_name);
    let witnesses_dir = format!("./witnesses/{}", circuit_name);

    let start_time = std::time::Instant::now();
    //generate witnesses (multi-thread)
    log::debug!("Start generating witnesses...");
    let witness_solver = Arc::new(w_s);
    let handles = assignment_chunks
        .into_iter()
        .enumerate()
        .map(|(i, assignments)| {
            let witness_solver = Arc::clone(&witness_solver);
            let witnesses_dir_clone = witnesses_dir.clone();
            thread::spawn(move || {
                let mut hint_registry = HintRegistry::<M31>::new();
                register_hint(&mut hint_registry);
                let witness = witness_solver
                    .solve_witnesses_with_hints(&assignments, &mut hint_registry)
                    .unwrap();
                write_witness_to_file(
                    &format!("{}/witness_{}.txt", witnesses_dir_clone, i),
                    witness,
                )
            })
        })
        .collect::<Vec<_>>();
    for handle in handles {
        handle.join().unwrap();
    }
    let end_time = std::time::Instant::now();
    log::debug!(
        "Generate {} witness Time: {:?}",
        circuit_name,
        end_time.duration_since(start_time)
    );
}
pub fn end2end_validator_tree_assignments_with_beacon_data(
    validator_tree: Vec<Vec<Vec<u32>>>,
    real_validator_count: u64,
) -> (
    ValidatorSubMTAssignmentChunks,
    MergeSubMTLimitAssignmentChunks,
) {
    let start_time = std::time::Instant::now();
    let merkle_subtree_with_limit_assignment =
        MergeSubMTLimitCircuit::get_assignments_from_beacon_data(
            &validator_tree,
            real_validator_count,
        );
    let convert_validator_list_to_merkle_tree_assignment =
        ValidatorSubMTCircuit::get_assignments_from_beacon_data(&validator_tree);
    let convert_validator_list_to_merkle_tree_assignments_chunks: Vec<
        Vec<ValidatorSubMTCircuit<M31>>,
    > = convert_validator_list_to_merkle_tree_assignment
        .chunks(16)
        .map(|x| x.to_vec())
        .collect();
    let end_time = std::time::Instant::now();
    log::debug!(
        "assigned validator tree assignment data, time: {:?}",
        end_time.duration_since(start_time)
    );
    (
        convert_validator_list_to_merkle_tree_assignments_chunks,
        vec![vec![merkle_subtree_with_limit_assignment; 16]; 1],
    )
}

pub fn end2end_validator_tree_witnesses_with_beacon_data(
    w_s_subtree: WitnessSolver<M31Config>,
    w_s_merkle: WitnessSolver<M31Config>,
    validator_tree: &[Vec<Vec<u32>>],
    real_validator_count: u64,
) {
    stacker::grow(32 * 1024 * 1024 * 1024, || {
        let (
            convert_validator_list_to_merkle_tree_assignments_chunks,
            merkle_subtree_with_limit_assignment_chunks,
        ) = end2end_validator_tree_assignments_with_beacon_data(
            validator_tree.to_vec(),
            real_validator_count,
        );
        //generate witnesses (multi-thread)
        end2end_validator_subtree_witnesses_with_assignments(
            w_s_subtree,
            convert_validator_list_to_merkle_tree_assignments_chunks,
        );
        end2end_merkle_subtree_with_limit_witnesses_with_assignments(
            w_s_merkle,
            merkle_subtree_with_limit_assignment_chunks,
        );
    });
}
// #[test]
// fn test_end2end_validators_assignments() {
//     let slot = 290000 * 32;
//     let (
//         seed,
//         shuffle_indices,
//         committee_indices,
//         pivots,
//         activated_indices,
//         flips,
//         positions,
//         flip_bits,
//         round_hash_bits,
//         attestations,
//         aggregated_pubkeys,
//         balance_list,
//         real_committee_size,
//         validator_tree,
//         hash_bytes,
//         plain_validators,
//     ) = beacon::prepare_assignment_data(slot, slot + 32);
//     let assignments =
//         end2end_validator_tree_assignments(validator_tree, activated_indices.len() as u64);
// }
