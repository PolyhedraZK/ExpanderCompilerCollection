use crate::attestation::Attestation;
use crate::bls::{aggregate_attestation_public_key_flatten, check_pubkey_key_bls};
use crate::bls_verifier::G1Json;
use crate::utils::{get_solver, read_from_json_file, write_witness_to_file};
use crate::validator::{ValidatorPlain, ValidatorSSZ};
use crate::{beacon, bls};
use ark_bls12_381::G1Affine as BlsG1Affine;
use ark_serialize::CanonicalDeserialize;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use circuit_std_rs::gnark::emulated::sw_bls12381::g1::*;
use circuit_std_rs::sha256::m31_utils::big_array_add;
use circuit_std_rs::utils::{register_hint, simple_select};
use core::panic;
use expander_compiler::frontend::extra::*;
use expander_compiler::frontend::*;
use serde::de::{Deserializer, SeqAccess, Visitor};
use serde::Deserialize;
use std::fmt;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
pub const SHUFFLE_ROUND: usize = 90;
pub const VALIDATOR_CHUNK_SIZE: usize = 128 * 4;
pub const MAX_VALIDATOR_EXP: usize = 29;
pub const POSEIDON_HASH_LENGTH: usize = 8;

#[derive(Debug, Clone)]
pub struct ValidatorData {
    pub validator_hashes: Vec<Vec<u32>>,
    pub plain_validators: Vec<ValidatorPlain>,
    pub aggregated_pubkeys: Vec<BlsG1Affine>,
}
#[derive(Debug, Deserialize, Clone)]
pub struct ShuffleJson {
    #[serde(rename = "StartIndex")]
    pub start_index: u32,
    #[serde(rename = "ChunkLength")]
    pub chunk_length: u32,
    #[serde(rename = "ShuffleIndices", deserialize_with = "deserialize_1d_u32_m31")]
    pub shuffle_indices: Vec<u32>,
    #[serde(
        rename = "CommitteeIndices",
        deserialize_with = "deserialize_1d_u32_m31"
    )]
    pub committee_indices: Vec<u32>,
    #[serde(rename = "Pivots", deserialize_with = "deserialize_1d_u32_m31")]
    pub pivots: Vec<u32>,
    #[serde(rename = "IndexCount")]
    pub index_count: u32,
    #[serde(
        rename = "PositionResults",
        deserialize_with = "deserialize_1d_u32_m31"
    )]
    pub position_results: Vec<u32>,
    #[serde(
        rename = "PositionBitResults",
        deserialize_with = "deserialize_1d_u32_m31"
    )]
    pub position_bit_results: Vec<u32>,
    #[serde(rename = "FlipResults", deserialize_with = "deserialize_1d_u32_m31")]
    pub flip_results: Vec<u32>,
    #[serde(rename = "Slot")]
    pub slot: u32,
    #[serde(
        rename = "ValidatorHashes",
        deserialize_with = "deserialize_2d_u32_m31"
    )]
    pub validator_hashes: Vec<Vec<u32>>,
    #[serde(
        rename = "AggregationBits",
        deserialize_with = "deserialize_1d_u32_m31"
    )]
    pub aggregation_bits: Vec<u32>,
    #[serde(rename = "AggregatedPubkey")]
    pub aggregated_pubkey: G1Json,
    #[serde(rename = "AttestationBalance")]
    pub attestation_balance: Vec<u32>,
}
fn process_i64_value(value: i64) -> u32 {
    if value == -1 {
        (1u32 << 31) - 2 // p - 1
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
    start_index: Variable,                             //PUBLIC
    chunk_length: Variable,                            //PUBLIC
    shuffle_indices: [Variable; VALIDATOR_CHUNK_SIZE], //PCS: share with permutation hash circuit
    committee_indices: [Variable; VALIDATOR_CHUNK_SIZE],
    pivots: [Variable; SHUFFLE_ROUND],
    index_count: Variable,                                              //PUBLIC
    position_results: [Variable; SHUFFLE_ROUND * VALIDATOR_CHUNK_SIZE], //HINT
    position_bit_results: [Variable; SHUFFLE_ROUND * VALIDATOR_CHUNK_SIZE], //HINT
    flip_results: [Variable; SHUFFLE_ROUND * VALIDATOR_CHUNK_SIZE],     //HINT
    aggregation_bits: [Variable; VALIDATOR_CHUNK_SIZE],                 //PUBLIC
    validator_hashes: [[Variable; POSEIDON_HASH_LENGTH]; VALIDATOR_CHUNK_SIZE], //HINT, share with permutation circuit
    aggregated_pubkey: [[Variable; 48]; 2], //PCS: public public_key, share with bls_verifier circuit
    attestation_balance: [Variable; 8],     //PUBLIC
    pubkeys_bls: [[[Variable; 48]; 2]; VALIDATOR_CHUNK_SIZE], //HINT
    // validators:      [ValidatorSSZ;VALIDATOR_CHUNK_SIZE],  //HINT
    pubkey: [[Variable; 48]; VALIDATOR_CHUNK_SIZE], //HINT
    withdrawal_credentials: [[Variable; 32]; VALIDATOR_CHUNK_SIZE], //HINT
    effective_balance: [[Variable; 8]; VALIDATOR_CHUNK_SIZE], //HINT
    slashed: [[Variable; 1]; VALIDATOR_CHUNK_SIZE], //HINT
    activation_eligibility_epoch: [[Variable; 8]; VALIDATOR_CHUNK_SIZE], //HINT
    activation_epoch: [[Variable; 8]; VALIDATOR_CHUNK_SIZE], //HINT
    exit_epoch: [[Variable; 8]; VALIDATOR_CHUNK_SIZE], //HINT
    withdrawable_epoch: [[Variable; 8]; VALIDATOR_CHUNK_SIZE], //HINT
});
pub type ShuffleAssignmentChunks = Vec<Vec<ShuffleCircuit<M31>>>;
impl ShuffleCircuit<M31> {
    pub fn from_plains(
        &mut self,
        shuffle_json: &ShuffleJson,
        plain_validators: &[ValidatorPlain],
        pubkey_bls: &[Vec<String>],
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
            for (j, pubkey_byte) in pubkey.iter().enumerate().take(48) {
                self.pubkey[i][j] = M31::from(*pubkey_byte as u32);
            }
            //assign withdrawal_credentials
            let raw_withdrawal_credentials = validator.withdrawal_credentials.clone();
            let withdrawal_credentials = STANDARD.decode(raw_withdrawal_credentials).unwrap();
            for (j, withdrawal_credentials_byte) in
                withdrawal_credentials.iter().enumerate().take(32)
            {
                self.withdrawal_credentials[i][j] = M31::from(*withdrawal_credentials_byte as u32);
            }
            //assign effective_balance
            let effective_balance = validator.effective_balance.to_le_bytes();
            for (j, effective_balance_byte) in effective_balance.iter().enumerate() {
                self.effective_balance[i][j] = M31::from(*effective_balance_byte as u32);
            }
            //assign slashed
            let slashed = if validator.slashed { 1 } else { 0 };
            self.slashed[i][0] = M31::from(slashed);
            //assign activation_eligibility_epoch
            let activation_eligibility_epoch = validator.activation_eligibility_epoch.to_le_bytes();
            for (j, activation_eligibility_epoch_byte) in
                activation_eligibility_epoch.iter().enumerate()
            {
                self.activation_eligibility_epoch[i][j] =
                    M31::from(*activation_eligibility_epoch_byte as u32);
            }
            //assign activation_epoch
            let activation_epoch = validator.activation_epoch.to_le_bytes();
            for (j, activation_epoch_byte) in activation_epoch.iter().enumerate() {
                self.activation_epoch[i][j] = M31::from(*activation_epoch_byte as u32);
            }
            //assign exit_epoch
            let exit_epoch = validator.exit_epoch.to_le_bytes();
            for (j, exit_epoch_byte) in exit_epoch.iter().enumerate() {
                self.exit_epoch[i][j] = M31::from(*exit_epoch_byte as u32);
            }
            //assign withdrawable_epoch
            let withdrawable_epoch = validator.withdrawable_epoch.to_le_bytes();
            for (j, withdrawable_epoch_byte) in withdrawable_epoch.iter().enumerate() {
                self.withdrawable_epoch[i][j] = M31::from(*withdrawable_epoch_byte as u32);
            }
        }
    }
    pub fn from_beacon(
        circuit_id: usize,
        committee_data: &beacon::CommitteeData,
        shuffle_data: &beacon::ShuffleData,
        slot_attestations: &[Vec<Attestation>],
        validator_data: &ValidatorData,
        balance_list: &[u64],
    ) -> Self {
        let mut assignment = Self::default();
        let start: usize = committee_data
            .real_committee_size
            .iter()
            .take(circuit_id)
            .copied()
            .sum::<u64>() as usize;
        let real_validator_count = shuffle_data.shuffle_indices.len();
        //assign shuffle_json
        assignment.start_index = M31::from(start as u32);
        assignment.chunk_length = M31::from(VALIDATOR_CHUNK_SIZE as u32);
        let lower = start.min(real_validator_count);
        let upper = (start + VALIDATOR_CHUNK_SIZE).min(real_validator_count);
        let sub_shuffle_indices = &shuffle_data.shuffle_indices[lower..upper];
        let sub_committee_indices = &committee_data.committee_indices[lower..upper];
        let slot_idx = circuit_id / beacon::MAXCOMMITTEESPERSLOT;
        let committee_idx = circuit_id % beacon::MAXCOMMITTEESPERSLOT;
        let mut att = &slot_attestations[0][0];
        let mut pubkey: [[u8; 48]; 2] = [[0; 48]; 2];
        if slot_attestations.is_empty() {
            panic!("slot_attestations is empty");
        }
        if !slot_attestations[slot_idx].is_empty()
            && slot_attestations[slot_idx].len() > committee_idx
        {
            println!("slot_idx{}, commitee_idx{}", slot_idx, committee_idx);
            pubkey = bls::affine_point_to_bytes_g1(&validator_data.aggregated_pubkeys[circuit_id]);
            att = &slot_attestations[slot_idx][committee_idx];
            println!("attestation: {:?}", att);
        }
        println!("attestation: {:?}", att);
        let aggregated_bits =
            beacon::attestation_get_aggregation_bits_from_bytes(&att.aggregation_bits);
        for i in 0..VALIDATOR_CHUNK_SIZE {
            assignment.shuffle_indices[i] = M31::from(0);
            assignment.committee_indices[i] = M31::from(0);
            assignment.aggregation_bits[i] = M31::from(0);
            if i < sub_shuffle_indices.len() {
                assignment.shuffle_indices[i] = M31::from(sub_shuffle_indices[i] as u32);
            }
            if i < sub_committee_indices.len() {
                assignment.committee_indices[i] = M31::from(sub_committee_indices[i] as u32);
            }
            if i < aggregated_bits.len() - 1 {
                assignment.aggregation_bits[i] = M31::from(aggregated_bits[i] as u32);
            }
        }
        assignment
            .pivots
            .iter_mut()
            .zip(shuffle_data.pivots.iter())
            .for_each(|(a, &b)| *a = M31::from(b as u32));

        assignment.index_count = M31::from(real_validator_count as u32);
        let neg_one = M31::from((1 << 31) - 2);
        let sub_flips = &shuffle_data.flips[lower..upper];
        let sub_positions = &shuffle_data.positions[lower..upper];
        let sub_flip_bits = &shuffle_data.flip_bits[lower..upper];
        for i in 0..SHUFFLE_ROUND {
            for j in 0..VALIDATOR_CHUNK_SIZE {
                assignment.position_results[i * VALIDATOR_CHUNK_SIZE + j] = neg_one;
                assignment.position_bit_results[i * VALIDATOR_CHUNK_SIZE + j] = M31::from(0);
                assignment.flip_results[i * VALIDATOR_CHUNK_SIZE + j] = neg_one;
                if j < sub_flips.len() {
                    assignment.flip_results[i * VALIDATOR_CHUNK_SIZE + j] =
                        M31::from(sub_flips[j][i] as u32);
                }
                if j < sub_positions.len() {
                    assignment.position_results[i * VALIDATOR_CHUNK_SIZE + j] =
                        M31::from(sub_positions[j][i] as u32);
                }
                if j < sub_flip_bits.len() {
                    assignment.position_bit_results[i * VALIDATOR_CHUNK_SIZE + j] =
                        M31::from(sub_flip_bits[j][i] as u32);
                }
            }
        }

        //assign validator_hashes
        for i in 0..VALIDATOR_CHUNK_SIZE {
            for j in 0..POSEIDON_HASH_LENGTH {
                assignment.validator_hashes[i][j] =
                    M31::from(validator_data.validator_hashes[0][j]);
                if i < sub_committee_indices.len() {
                    assignment.validator_hashes[i][j] = M31::from(
                        validator_data.validator_hashes[sub_committee_indices[i] as usize][j],
                    );
                }
            }
        }

        //assign aggregated_pubkey
        for i in 0..48 {
            assignment.aggregated_pubkey[0][i] = M31::from(pubkey[0][i] as u32);
            assignment.aggregated_pubkey[1][i] = M31::from(pubkey[1][i] as u32);
        }

        //assign attestation_balance
        let balance = balance_list[circuit_id];
        let balance_bytes = balance.to_le_bytes(); // [u8; 8]

        assignment
            .attestation_balance
            .iter_mut()
            .zip(balance_bytes.iter().take(8))
            .for_each(|(a, &b)| *a = M31::from(b as u32));

        for i in 0..VALIDATOR_CHUNK_SIZE {
            //assign validator
            let validator = validator_data.plain_validators
                [*sub_committee_indices.get(i).unwrap_or(&0) as usize]
                .clone();

            //assign pubkey
            let raw_pubkey = validator.public_key.clone();
            let pubkey = STANDARD.decode(raw_pubkey).unwrap();
            for (j, pubkey_byte) in pubkey.iter().enumerate().take(48) {
                assignment.pubkey[i][j] = M31::from(*pubkey_byte as u32);
            }
            //assign pubkey_bls
            let raw_pubkey_bls = BlsG1Affine::deserialize_compressed(&pubkey[..]).unwrap();
            let pubkey_bls_bytes = bls::affine_point_to_bytes_g1(&raw_pubkey_bls);
            for k in 0..48 {
                assignment.pubkeys_bls[i][0][k] = M31::from(pubkey_bls_bytes[0][47 - k] as u32);
                assignment.pubkeys_bls[i][1][k] = M31::from(pubkey_bls_bytes[1][47 - k] as u32);
            }

            //assign withdrawal_credentials
            let raw_withdrawal_credentials = validator.withdrawal_credentials.clone();
            let withdrawal_credentials = STANDARD.decode(raw_withdrawal_credentials).unwrap();
            for (j, withdrawal_credentials_byte) in
                withdrawal_credentials.iter().enumerate().take(32)
            {
                assignment.withdrawal_credentials[i][j] =
                    M31::from(*withdrawal_credentials_byte as u32);
            }
            //assign effective_balance
            let effective_balance = validator.effective_balance.to_le_bytes();
            for (j, effective_balance_byte) in effective_balance.iter().enumerate() {
                assignment.effective_balance[i][j] = M31::from(*effective_balance_byte as u32);
            }
            //assign slashed
            let slashed = if validator.slashed { 1 } else { 0 };
            assignment.slashed[i][0] = M31::from(slashed);
            //assign activation_eligibility_epoch
            let activation_eligibility_epoch = validator.activation_eligibility_epoch.to_le_bytes();
            for (j, activation_eligibility_epoch_byte) in
                activation_eligibility_epoch.iter().enumerate()
            {
                assignment.activation_eligibility_epoch[i][j] =
                    M31::from(*activation_eligibility_epoch_byte as u32);
            }
            //assign activation_epoch
            let activation_epoch = validator.activation_epoch.to_le_bytes();
            for (j, activation_epoch_byte) in activation_epoch.iter().enumerate() {
                assignment.activation_epoch[i][j] = M31::from(*activation_epoch_byte as u32);
            }
            //assign exit_epoch
            let exit_epoch = validator.exit_epoch.to_le_bytes();
            for (j, exit_epoch_byte) in exit_epoch.iter().enumerate() {
                assignment.exit_epoch[i][j] = M31::from(*exit_epoch_byte as u32);
            }
            //assign withdrawable_epoch
            let withdrawable_epoch = validator.withdrawable_epoch.to_le_bytes();
            for (j, withdrawable_epoch_byte) in withdrawable_epoch.iter().enumerate() {
                assignment.withdrawable_epoch[i][j] = M31::from(*withdrawable_epoch_byte as u32);
            }
        }
        assignment
    }
    pub fn from_beacon_whole(
        circuit_id: usize,
        beacon_assignment_data: &beacon::BeaconAssignmentData,
    ) -> Self {
        let mut assignment = Self::default();
        let start: usize = beacon_assignment_data
            .committee_data
            .real_committee_size
            .iter()
            .take(circuit_id)
            .copied()
            .sum::<u64>() as usize;
        let real_validator_count = beacon_assignment_data.shuffle_data.shuffle_indices.len();
        //assign shuffle_json
        assignment.start_index = M31::from(start as u32);
        assignment.chunk_length = M31::from(VALIDATOR_CHUNK_SIZE as u32);
        let lower = start.min(real_validator_count);
        let upper = (start + VALIDATOR_CHUNK_SIZE).min(real_validator_count);
        let sub_shuffle_indices =
            &beacon_assignment_data.shuffle_data.shuffle_indices[lower..upper];
        let sub_committee_indices =
            &beacon_assignment_data.committee_data.committee_indices[lower..upper];
        let slot_idx = circuit_id / beacon::MAXCOMMITTEESPERSLOT;
        let committee_idx = circuit_id % beacon::MAXCOMMITTEESPERSLOT;
        let mut pubkey: [[u8; 48]; 2] = [[0; 48]; 2];
        if beacon_assignment_data.attestations.is_empty() {
            panic!("slot_attestations is empty");
        }
        let mut att = &beacon_assignment_data.attestations[0][0];
        if !beacon_assignment_data.attestations[slot_idx].is_empty()
            && beacon_assignment_data.attestations[slot_idx].len() > committee_idx
        {
            println!("slot_idx{}, commitee_idx{}", slot_idx, committee_idx);
            pubkey = bls::affine_point_to_bytes_g1(
                &beacon_assignment_data.aggregated_pubkeys[circuit_id],
            );
            att = &beacon_assignment_data.attestations[slot_idx][committee_idx];
            println!("attestation: {:?}", att);
        }
        println!("attestation: {:?}", att);
        let aggregated_bits =
            beacon::attestation_get_aggregation_bits_from_bytes(&att.aggregation_bits);
        for i in 0..VALIDATOR_CHUNK_SIZE {
            assignment.shuffle_indices[i] = M31::from(0);
            assignment.committee_indices[i] = M31::from(0);
            assignment.aggregation_bits[i] = M31::from(0);
            if i < sub_shuffle_indices.len() {
                assignment.shuffle_indices[i] = M31::from(sub_shuffle_indices[i] as u32);
            }
            if i < sub_committee_indices.len() {
                assignment.committee_indices[i] = M31::from(sub_committee_indices[i] as u32);
            }
            if i < aggregated_bits.len() - 1 {
                assignment.aggregation_bits[i] = M31::from(aggregated_bits[i] as u32);
            }
        }
        assignment
            .pivots
            .iter_mut()
            .zip(beacon_assignment_data.shuffle_data.pivots.iter())
            .for_each(|(a, &b)| *a = M31::from(b as u32));

        assignment.index_count = M31::from(real_validator_count as u32);
        let neg_one = M31::from((1 << 31) - 2);
        let sub_flips = &beacon_assignment_data.shuffle_data.flips[lower..upper];
        let sub_positions = &beacon_assignment_data.shuffle_data.positions[lower..upper];
        let sub_flip_bits = &beacon_assignment_data.shuffle_data.flip_bits[lower..upper];
        for i in 0..SHUFFLE_ROUND {
            for j in 0..VALIDATOR_CHUNK_SIZE {
                assignment.position_results[i * VALIDATOR_CHUNK_SIZE + j] = neg_one;
                assignment.position_bit_results[i * VALIDATOR_CHUNK_SIZE + j] = M31::from(0);
                assignment.flip_results[i * VALIDATOR_CHUNK_SIZE + j] = neg_one;
                if j < sub_flips.len() {
                    assignment.flip_results[i * VALIDATOR_CHUNK_SIZE + j] =
                        M31::from(sub_flips[j][i] as u32);
                }
                if j < sub_positions.len() {
                    assignment.position_results[i * VALIDATOR_CHUNK_SIZE + j] =
                        M31::from(sub_positions[j][i] as u32);
                }
                if j < sub_flip_bits.len() {
                    assignment.position_bit_results[i * VALIDATOR_CHUNK_SIZE + j] =
                        M31::from(sub_flip_bits[j][i] as u32);
                }
            }
        }

        //assign validator_hashes
        let validator_tree = &beacon_assignment_data.validator_tree;
        let validator_hashes = &validator_tree[validator_tree.len() - 1];
        for i in 0..VALIDATOR_CHUNK_SIZE {
            for j in 0..POSEIDON_HASH_LENGTH {
                assignment.validator_hashes[i][j] = M31::from(validator_hashes[0][j]);
                if i < sub_committee_indices.len() {
                    assignment.validator_hashes[i][j] =
                        M31::from(validator_hashes[sub_committee_indices[i] as usize][j]);
                }
            }
        }

        //assign aggregated_pubkey
        for i in 0..48 {
            assignment.aggregated_pubkey[0][i] = M31::from(pubkey[0][i] as u32);
            assignment.aggregated_pubkey[1][i] = M31::from(pubkey[1][i] as u32);
        }

        //assign attestation_balance
        let balance = beacon_assignment_data.balance_list[circuit_id];
        let balance_bytes = balance.to_le_bytes(); // [u8; 8]

        assignment
            .attestation_balance
            .iter_mut()
            .zip(balance_bytes.iter().take(8))
            .for_each(|(a, &b)| *a = M31::from(b as u32));

        for i in 0..VALIDATOR_CHUNK_SIZE {
            //assign validator
            let validator = beacon_assignment_data.validator_list
                [*sub_committee_indices.get(i).unwrap_or(&0) as usize]
                .clone();

            //assign pubkey
            let raw_pubkey = validator.public_key.clone();
            let pubkey = STANDARD.decode(raw_pubkey).unwrap();
            for (j, pubkey_byte) in pubkey.iter().enumerate().take(48) {
                assignment.pubkey[i][j] = M31::from(*pubkey_byte as u32);
            }
            //assign pubkey_bls
            let raw_pubkey_bls = BlsG1Affine::deserialize_compressed(&pubkey[..]).unwrap();
            let pubkey_bls_bytes = bls::affine_point_to_bytes_g1(&raw_pubkey_bls);
            for k in 0..48 {
                assignment.pubkeys_bls[i][0][k] = M31::from(pubkey_bls_bytes[0][47 - k] as u32);
                assignment.pubkeys_bls[i][1][k] = M31::from(pubkey_bls_bytes[1][47 - k] as u32);
            }

            //assign withdrawal_credentials
            let raw_withdrawal_credentials = validator.withdrawal_credentials.clone();
            let withdrawal_credentials = STANDARD.decode(raw_withdrawal_credentials).unwrap();
            for (j, withdrawal_credentials_byte) in
                withdrawal_credentials.iter().enumerate().take(32)
            {
                assignment.withdrawal_credentials[i][j] =
                    M31::from(*withdrawal_credentials_byte as u32);
            }
            //assign effective_balance
            let effective_balance = validator.effective_balance.to_le_bytes();
            for (j, effective_balance_byte) in effective_balance.iter().enumerate() {
                assignment.effective_balance[i][j] = M31::from(*effective_balance_byte as u32);
            }
            //assign slashed
            let slashed = if validator.slashed { 1 } else { 0 };
            assignment.slashed[i][0] = M31::from(slashed);
            //assign activation_eligibility_epoch
            let activation_eligibility_epoch = validator.activation_eligibility_epoch.to_le_bytes();
            for (j, activation_eligibility_epoch_byte) in
                activation_eligibility_epoch.iter().enumerate()
            {
                assignment.activation_eligibility_epoch[i][j] =
                    M31::from(*activation_eligibility_epoch_byte as u32);
            }
            //assign activation_epoch
            let activation_epoch = validator.activation_epoch.to_le_bytes();
            for (j, activation_epoch_byte) in activation_epoch.iter().enumerate() {
                assignment.activation_epoch[i][j] = M31::from(*activation_epoch_byte as u32);
            }
            //assign exit_epoch
            let exit_epoch = validator.exit_epoch.to_le_bytes();
            for (j, exit_epoch_byte) in exit_epoch.iter().enumerate() {
                assignment.exit_epoch[i][j] = M31::from(*exit_epoch_byte as u32);
            }
            //assign withdrawable_epoch
            let withdrawable_epoch = validator.withdrawable_epoch.to_le_bytes();
            for (j, withdrawable_epoch_byte) in withdrawable_epoch.iter().enumerate() {
                assignment.withdrawable_epoch[i][j] = M31::from(*withdrawable_epoch_byte as u32);
            }
        }
        assignment
    }
    pub fn get_assignments_from_data(
        shuffle_data: Vec<ShuffleJson>,
        plain_validators: Vec<ValidatorPlain>,
        public_key_bls_list: Vec<Vec<String>>,
    ) -> Vec<Self> {
        let mut handles = vec![];
        let plain_validators = Arc::new(plain_validators);
        let public_key_bls_list = Arc::new(public_key_bls_list);
        let assignments = Arc::new(Mutex::new(vec![None; shuffle_data.len()]));
        for (i, shuffle_item) in shuffle_data.into_iter().enumerate() {
            let assignments = Arc::clone(&assignments);
            let target_plain_validators = Arc::clone(&plain_validators);
            let target_public_key_bls_list = Arc::clone(&public_key_bls_list);

            let handle = thread::spawn(move || {
                let mut assignment = ShuffleCircuit::<M31>::default();
                assignment.from_plains(
                    &shuffle_item,
                    &target_plain_validators,
                    &target_public_key_bls_list,
                );

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
    pub fn get_assignments_from_beacon_data(
        validator_data: ValidatorData,
        committee_data: beacon::CommitteeData,
        shuffle_data: beacon::ShuffleData,
        slot_attestations: Vec<Vec<Attestation>>,
        balance_list: Vec<u64>,
        range: [usize; 2],
    ) -> Vec<Self> {
        let start = range[0] * beacon::MAXCOMMITTEESPERSLOT;
        let end = range[1] * beacon::MAXCOMMITTEESPERSLOT;
        let mut handles = vec![];
        let validator_data = Arc::new(validator_data);
        let committee_data = Arc::new(committee_data);
        let shuffle_data = Arc::new(shuffle_data);
        let slot_attestations = Arc::new(slot_attestations);
        let balance_list = Arc::new(balance_list);
        let assignments = Arc::new(Mutex::new(vec![None; end - start]));
        for i in start..end {
            let assignments = Arc::clone(&assignments);
            let target_validator_data = Arc::clone(&validator_data);
            let target_committee_data = Arc::clone(&committee_data);
            let target_shuffle_data = Arc::clone(&shuffle_data);
            let target_slot_attestations = Arc::clone(&slot_attestations);
            let target_balance_list = Arc::clone(&balance_list);

            let handle = thread::spawn(move || {
                let assignment = ShuffleCircuit::<M31>::from_beacon(
                    i,
                    &target_committee_data,
                    &target_shuffle_data,
                    &target_slot_attestations,
                    &target_validator_data,
                    &target_balance_list,
                );

                let mut assignments = assignments.lock().unwrap();
                assignments[i - start] = Some(assignment);
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
        let shuffle_data: Vec<ShuffleJson> = read_from_json_file(&format!("{}/shuffle.json", dir))
            .expect("Failed to read shuffle.json");
        let plain_validators: Vec<ValidatorPlain> =
            read_from_json_file(&format!("{}/validator.json", dir))
                .expect("Failed to read validator.json");
        let public_key_bls_list: Vec<Vec<String>> =
            read_from_json_file(&format!("{}/pubkeyBLSList.json", dir))
                .expect("Failed to read pubkey_bls.json");
        Self::get_assignments_from_data(shuffle_data, plain_validators, public_key_bls_list)
    }
    pub fn get_assignments_from_beacon_data_whole(
        beacon_assignment_data: Arc<beacon::BeaconAssignmentData>,
        range: [usize; 2],
    ) -> Vec<Self> {
        let start = range[0] * beacon::MAXCOMMITTEESPERSLOT;
        let end = range[1] * beacon::MAXCOMMITTEESPERSLOT;
        let mut handles = vec![];
        let assignments = Arc::new(Mutex::new(vec![None; end - start]));
        for i in start..end {
            let assignments = Arc::clone(&assignments);
            let target_beacon_assignment_data = Arc::clone(&beacon_assignment_data);

            let handle = thread::spawn(move || {
                let assignment =
                    ShuffleCircuit::<M31>::from_beacon_whole(i, &target_beacon_assignment_data);

                let mut assignments = assignments.lock().unwrap();
                assignments[i - start] = Some(assignment);
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
        for (i, chunk) in indices_chunk.iter_mut().enumerate() {
            let tmp = builder.add(self.flip_results[i], 1);
            let ignore_flag = builder.is_zero(tmp);
            *chunk = simple_select(builder, ignore_flag, zero_var, *chunk);
        }
        //flip the indices based on the hashbit
        let mut copy_cur_indices = indices_chunk.clone();
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
                g1.curve_f
                    .table
                    .rangeproof(builder, diff, MAX_VALIDATOR_EXP);
            }
            copy_cur_indices =
                builder.new_hint("myhint.copyvarshint", &cur_indices, cur_indices.len());
        }
        //check the final curIndices, should be equal to the shuffleIndex
        for (i, cur_index) in copy_cur_indices
            .iter_mut()
            .enumerate()
            .take(self.shuffle_indices.len())
        {
            let tmp = builder.add(self.flip_results[i], 1);
            let is_minus_one = builder.is_zero(tmp);
            *cur_index = simple_select(builder, is_minus_one, self.shuffle_indices[i], *cur_index);
            let tmp = builder.sub(self.shuffle_indices[i], *cur_index);
            let tmp_res = builder.is_zero(tmp);
            builder.assert_is_equal(tmp_res, 1);
        }

        let mut pubkey_list = vec![];
        let mut acc_balance = vec![];
        for i in 0..VALIDATOR_CHUNK_SIZE {
            pubkey_list.push(self.pubkey[i]);
            acc_balance.push(self.effective_balance[i]);
        }
        let effect_balance = calculate_balance(builder, &mut acc_balance, &self.aggregation_bits);
        for (i, cur_effect_balance) in effect_balance.iter().enumerate() {
            builder.assert_is_equal(cur_effect_balance, self.attestation_balance[i]);
        }

        let mut pubkey_list_bls = vec![];
        for cur_pubkey in pubkey_list.iter() {
            let pubkey_g1 = g1.uncompressed(builder, cur_pubkey);
            pubkey_list_bls.push(pubkey_g1);
        }

        let mut aggregated_pubkey = G1Affine::from_vars(
            self.aggregated_pubkey[0].to_vec(),
            self.aggregated_pubkey[1].to_vec(),
        );
        aggregate_attestation_public_key_flatten(
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
            let hash = validator.poseidon_hash(builder);
            for (i, hashbit) in hash.iter().enumerate().take(8) {
                builder.assert_is_equal(hashbit, self.validator_hashes[index][i]);
            }
        }
        g1.curve_f.check_mul(builder);
        g1.curve_f.table.final_check(builder);
        g1.curve_f.table.final_check(builder);
        g1.curve_f.table.final_check(builder);
    }
}

pub fn get_indice_chunk<C: Config, B: RootAPI<C>>(
    builder: &mut B,
    start: Variable,
    length: Variable,
    max_len: usize,
) -> Vec<Variable> {
    let mut res = vec![];
    //M31_MOD = 2147483647
    let neg_one = builder.constant(2147483647 - 1);
    for i in 0..max_len {
        let tmp = builder.sub(length, i as u32);
        let reach_end = builder.is_zero(tmp);
        let mut tmp = builder.add(start, i as u32);
        tmp = simple_select(builder, reach_end, neg_one, tmp);
        res.push(tmp);
    }
    res
}
pub fn calculate_balance<C: Config, B: RootAPI<C>>(
    builder: &mut B,
    acc_balance: &mut [[Variable; 8]],
    aggregation_bits: &[Variable],
) -> Vec<Variable> {
    if acc_balance.is_empty() || acc_balance[0].is_empty() {
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
            acc_balance[i][j] =
                simple_select(builder, aggregation_bits[i], acc_balance[i][j], zero_var);
        }
    }
    //since balance is [8]frontend.Variable, we need to support Array addition
    for balance in acc_balance {
        cur_balance = big_array_add(builder, &cur_balance, balance, cur_balance.len());
    }
    cur_balance
}
pub fn flip_with_hash_bits<C: Config, B: RootAPI<C>>(
    builder: &mut B,
    pivot: Variable,
    index_count: Variable,
    cur_indices: &[Variable],
    position_results: &[Variable],
    position_bit_results: &[Variable],
    flip_results: &[Variable],
) -> (Vec<Variable>, Vec<Variable>) {
    let mut res = vec![];
    let mut position_diffs = vec![];
    for i in 0..cur_indices.len() {
        let tmp = builder.add(flip_results[i], 1);
        let ignore_flag = builder.is_zero(tmp);

        //flip_results must be (pivot - cur_index) % indexCount
        let tmp = builder.sub(pivot, cur_indices[i]);
        let tmp = builder.sub(tmp, flip_results[i]);
        let flip_flag1 = builder.is_zero(tmp);
        let tmp = builder.add(index_count, pivot);
        let tmp = builder.sub(tmp, cur_indices[i]);
        let tmp = builder.sub(tmp, flip_results[i]);
        let flip_flag2 = builder.is_zero(tmp);
        let tmp_flag = builder.or(flip_flag1, flip_flag2);
        let flip_flag = builder.or(tmp_flag, ignore_flag);
        builder.assert_is_equal(flip_flag, 1);

        //position_results must be max(cur_index, flip_results)
        let tmp = builder.sub(position_results[i], flip_results[i]);
        let position_flag1 = builder.is_zero(tmp);
        let tmp = builder.sub(position_results[i], cur_indices[i]);
        let position_flag2 = builder.is_zero(tmp);
        let tmp = builder.or(position_flag1, position_flag2);
        let position_flag = builder.or(tmp, ignore_flag);
        builder.assert_is_equal(position_flag, 1);
        let tmp = builder.mul(2, position_results[i]);
        let tmp = builder.sub(tmp, flip_results[i]);
        let position_diff = builder.sub(tmp, cur_indices[i]);
        let zero_var = builder.constant(0);
        let position_diff = simple_select(builder, ignore_flag, zero_var, position_diff);
        //check the position_diff later, should be < indexCount
        position_diffs.push(position_diff);
        res.push(simple_select(
            builder,
            position_bit_results[i],
            flip_results[i],
            cur_indices[i],
        ));
    }
    (res, position_diffs)
}
pub fn generate_shuffle_witnesses(dir: &str) {
    stacker::grow(32 * 1024 * 1024 * 1024, || {
        let circuit_name = &format!("shuffle_{}", VALIDATOR_CHUNK_SIZE);

        //get solver
        log::debug!("preparing {} solver...", circuit_name);
        let witnesses_dir = format!("./witnesses/{}", circuit_name);
        let w_s = get_solver(&witnesses_dir, circuit_name, ShuffleCircuit::default());

        //get assignments
        let start_time = std::time::Instant::now();
        let assignments = ShuffleCircuit::get_assignments_from_json(dir);
        let end_time = std::time::Instant::now();
        log::debug!(
            "assigned shuffle assignments time: {:?}",
            end_time.duration_since(start_time)
        );

        let assignment_chunks: ShuffleAssignmentChunks =
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

pub fn end2end_shuffle_witnesses(
    w_s: WitnessSolver<M31Config>,
    plain_validators: Vec<ValidatorPlain>,
    shuffle_data: Vec<ShuffleJson>,
    public_key_bls_list: Vec<Vec<String>>,
) {
    stacker::grow(32 * 1024 * 1024 * 1024, || {
        let circuit_name = &format!("shuffle_{}", VALIDATOR_CHUNK_SIZE);

        let witnesses_dir = format!("./witnesses/{}", circuit_name);

        //get assignments
        let start_time = std::time::Instant::now();
        let assignments = ShuffleCircuit::get_assignments_from_data(
            shuffle_data,
            plain_validators,
            public_key_bls_list,
        );
        let end_time = std::time::Instant::now();
        log::debug!(
            "assigned shuffle assignments time: {:?}",
            end_time.duration_since(start_time)
        );
        let assignment_chunks: ShuffleAssignmentChunks =
            assignments.chunks(16).map(|x| x.to_vec()).collect();

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

pub fn end2end_shuffle_witnesses_with_assignments(
    w_s: WitnessSolver<M31Config>,
    assignment_chunks: ShuffleAssignmentChunks,
    offset: usize,
) {
    let circuit_name = &format!("shuffle_{}", VALIDATOR_CHUNK_SIZE);

    let start_time = std::time::Instant::now();
    let witnesses_dir = format!("./witnesses/{}", circuit_name);
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
                    &format!("{}/witness_{}.txt", witnesses_dir_clone, i + offset),
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

pub fn end2end_shuffle_assignments_with_beacon_data(
    validator_data: ValidatorData,
    committee_data: beacon::CommitteeData,
    shuffle_data: beacon::ShuffleData,
    slot_attestations: Vec<Vec<Attestation>>,
    balance_list: Vec<u64>,
    range: [usize; 2],
) -> ShuffleAssignmentChunks {
    //get assignments
    let start_time = std::time::Instant::now();
    let assignments = ShuffleCircuit::get_assignments_from_beacon_data(
        validator_data,
        committee_data,
        shuffle_data,
        slot_attestations,
        balance_list,
        range,
    );
    let end_time = std::time::Instant::now();
    log::debug!(
        "assigned shuffle assignments time: {:?}",
        end_time.duration_since(start_time)
    );
    let assignment_chunks: ShuffleAssignmentChunks =
        assignments.chunks(16).map(|x| x.to_vec()).collect();
    assignment_chunks
}

pub fn end2end_shuffle_assignments_with_beacon_data_whole(
    beacon_assignment_data: Arc<beacon::BeaconAssignmentData>,
    range: [usize; 2],
) -> ShuffleAssignmentChunks {
    //get assignments
    let start_time = std::time::Instant::now();
    let assignments =
        ShuffleCircuit::get_assignments_from_beacon_data_whole(beacon_assignment_data, range);
    let end_time = std::time::Instant::now();
    log::debug!(
        "assigned shuffle assignments time: {:?}",
        end_time.duration_since(start_time)
    );
    let assignment_chunks: ShuffleAssignmentChunks =
        assignments.chunks(16).map(|x| x.to_vec()).collect();
    assignment_chunks
}

pub fn end2end_shuffle_witnesses_with_beacon_data(
    w_s: WitnessSolver<M31Config>,
    validator_data: ValidatorData,
    committee_data: beacon::CommitteeData,
    shuffle_data: beacon::ShuffleData,
    slot_attestations: Vec<Vec<Attestation>>,
    balance_list: Vec<u64>,
    range: [usize; 2],
) {
    stacker::grow(32 * 1024 * 1024 * 1024, || {
        log::debug!("preparing shuffle witnesses...");
        //get assignments
        let assignment_chunks: ShuffleAssignmentChunks =
            end2end_shuffle_assignments_with_beacon_data(
                validator_data,
                committee_data,
                shuffle_data,
                slot_attestations,
                balance_list,
                range,
            );

        //generate witnesses (multi-thread)
        end2end_shuffle_witnesses_with_assignments(
            w_s,
            assignment_chunks,
            range[0] * beacon::MAXBEACONVALIDATORDEPTH / 16,
        );
    });
}

pub fn end2end_shuffle_witnesses_with_beacon_data_whole(
    w_s: WitnessSolver<M31Config>,
    beacon_assignment_data: Arc<beacon::BeaconAssignmentData>,
    range: [usize; 2],
) {
    stacker::grow(32 * 1024 * 1024 * 1024, || {
        log::debug!("preparing shuffle witnesses...");
        //get assignments
        let assignment_chunks: ShuffleAssignmentChunks =
            end2end_shuffle_assignments_with_beacon_data_whole(beacon_assignment_data, range);

        //generate witnesses (multi-thread)
        end2end_shuffle_witnesses_with_assignments(
            w_s,
            assignment_chunks,
            range[0] * beacon::MAXBEACONVALIDATORDEPTH / 16,
        );
    });
}
// #[test]
// fn test_end2end_shuffle_assignments() {
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
//     let assignments = end2end_shuffle_assignments_with_beacon_data(
//         plain_validators,
//         real_committee_size,
//         shuffle_indices,
//         committee_indices,
//         attestations,
//         aggregated_pubkeys,
//         pivots,
//         flips,
//         positions,
//         flip_bits,
//         validator_tree[validator_tree.len() - 1].clone(),
//         balance_list,
//         0,
//         16,
//     );
// }

// #[test]
// fn test_shuffle_witnesses_end() {
//     stacker::grow(128 * 1024 * 1024 * 1024, || {
//         let epoch = 290000;
//         let slot = epoch * 32;
//         let shuffle_handle = thread::spawn(|| {
//             let circuit_name = format!("shuffle_{}", VALIDATOR_CHUNK_SIZE);
//             let circuit = ShuffleCircuit::default();
//             let witnesses_dir = format!("./witnesses/{}", circuit_name);
//             get_solver(&witnesses_dir, &circuit_name, circuit)
//         });
//         let solver_shuffle = shuffle_handle.join().unwrap();
//         let (
//             seed,
//             shuffle_indices,
//             committee_indices,
//             pivots,
//             activated_indices,
//             flips,
//             positions,
//             flip_bits,
//             round_hash_bits,
//             attestations,
//             aggregated_pubkeys,
//             balance_list,
//             real_committee_size,
//             validator_tree,
//             hash_bytes,
//             plain_validators,
//         ) = beacon::prepare_assignment_data(slot, slot + 16);
//         let assignments = end2end_shuffle_assignments_with_beacon_data(
//             plain_validators,
//             real_committee_size,
//             shuffle_indices,
//             committee_indices,
//             attestations,
//             aggregated_pubkeys,
//             pivots,
//             flips,
//             positions,
//             flip_bits,
//             validator_tree[validator_tree.len() - 1].clone(),
//             balance_list,
//             0,
//             16,
//         );
//         end2end_shuffle_witnesses_with_assignments(solver_shuffle, assignments, 0);
//     });
// }

// #[test]
// fn test_shuffle_witnesses_start() {
//     stacker::grow(128 * 1024 * 1024 * 1024, || {
//         let epoch = 290000;
//         let slot = epoch * 32;
//         let shuffle_handle = thread::spawn(|| {
//             let circuit_name = format!("shuffle_{}", VALIDATOR_CHUNK_SIZE);
//             let circuit = ShuffleCircuit::default();
//             let witnesses_dir = format!("./witnesses/{}", circuit_name);
//             get_solver(&witnesses_dir, &circuit_name, circuit)
//         });
//         let solver_shuffle = shuffle_handle.join().unwrap();
//         let (
//             seed,
//             shuffle_indices,
//             committee_indices,
//             pivots,
//             activated_indices,
//             flips,
//             positions,
//             flip_bits,
//             round_hash_bits,
//             attestations,
//             aggregated_pubkeys,
//             balance_list,
//             real_committee_size,
//             validator_tree,
//             hash_bytes,
//             plain_validators,
//         ) = beacon::prepare_assignment_data(slot + 16, slot + 32);
//         let assignments = end2end_shuffle_assignments_with_beacon_data(
//             plain_validators,
//             real_committee_size,
//             shuffle_indices,
//             committee_indices,
//             attestations,
//             aggregated_pubkeys,
//             pivots,
//             flips,
//             positions,
//             flip_bits,
//             validator_tree[validator_tree.len() - 1].clone(),
//             balance_list,
//             16,
//             32,
//         );
//         end2end_shuffle_witnesses_with_assignments(solver_shuffle, assignments, 16 * 64 / 16);
//     });
// }
