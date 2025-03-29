//read beacon info from files
use ark_bls12_381::G1Affine;
use ark_ec::AffineRepr;
use ark_serialize::CanonicalDeserialize;
use base64::engine::general_purpose;
use base64::Engine;
use bincode;
use byteorder::{ByteOrder, LittleEndian};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::path::Path;
use std::{error::Error, fs};

use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::attestation::{Attestation, CheckpointPlain};
use crate::merkle;
use crate::merkle::{merkle_tree_element_with_limit, merkleize_with_mixin_poseidon};
use crate::validator::ValidatorPlain;
use circuit_std_rs::poseidon::poseidon::*;
use circuit_std_rs::poseidon::utils::*;

const SUBCIRCUIT_TREE_CACHE_DIR: &str = "./data/subcircuitTreeCache/";
const CACHE_DIR: &str = "./data/cache/";
const LOCAL_TREE_DIR: &str = "./data/localTree/";
const RANDAO_DIR: &str = "./data/beacon/randao/";
const VALIDATOR_DIR: &str = "./data/beacon/validator/";
const COMMITTEE_DIR: &str = "./data/beacon/committee/";
const ATTESTATION_DIR: &str = "./data/beacon/attestations/";
const DOMAIN_BEACON_ATTESTER: &str = "01000000";

pub const SLOTSPEREPOCH: u64 = 32;
pub const SHUFFLEROUND: usize = 90;
pub const MAXCOMMITTEESPERSLOT: usize = 64;
pub const MAXBEACONVALIDATORDEPTH: usize = merkle::MAX_BEACON_VALIDATOR_DEPTH;
pub const MAXBEACONVALIDATORSIZE: usize = 1 << MAXBEACONVALIDATORDEPTH;

#[derive(Debug, Deserialize, Clone)]
pub struct BeaconCommitteeJson {
    // Adjust these fields based on your actual Go struct
    pub slot: String,
    pub index: String,
    pub validators: Vec<String>,
}
#[derive(Deserialize, Debug)]
pub struct AttestationsWithBytes {
    pub attestations: Vec<Attestation>,
    #[serde(with = "base64_standard")]
    pub data: Vec<u8>,
}

// Helper module for base64 encoding/decoding
mod base64_standard {
    use base64::{engine::general_purpose, Engine};
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        general_purpose::STANDARD
            .decode(s.as_bytes())
            .map_err(serde::de::Error::custom)
    }
}

pub fn init_directories() -> std::io::Result<()> {
    fs::create_dir_all(Path::new(SUBCIRCUIT_TREE_CACHE_DIR))?;
    fs::create_dir_all(Path::new(CACHE_DIR))?;
    fs::create_dir_all(Path::new(LOCAL_TREE_DIR))?;
    fs::create_dir_all(Path::new(RANDAO_DIR))?;
    fs::create_dir_all(Path::new(VALIDATOR_DIR))?;
    fs::create_dir_all(Path::new(COMMITTEE_DIR))?;
    Ok(())
}

fn get_seed(epoch: u64, randao_mix: &[u8], domain_type: &str) -> Option<Vec<u8>> {
    let epoch_bytes = epoch.to_le_bytes().to_vec();
    let domain_type_bytes = hex::decode(domain_type).ok()?;

    let mut to_hash = Vec::new();
    to_hash.extend(domain_type_bytes);
    to_hash.extend(epoch_bytes);
    to_hash.extend(randao_mix);

    let hash = Sha256::digest(to_hash);
    Some(hash.to_vec())
}

pub fn get_beacon_seed(epoch: u64) -> Option<Vec<u8>> {
    let seed_slot = if epoch < 2 {
        0
    } else {
        (epoch - 2) * SLOTSPEREPOCH + SLOTSPEREPOCH - 1
    };

    let randao = get_randao_from_slot(seed_slot).unwrap();
    get_seed(epoch, &randao, DOMAIN_BEACON_ATTESTER)
}

pub fn get_randao_from_slot(slot: u64) -> Result<Vec<u8>, Box<dyn Error>> {
    let path = format!("{}{}.json", RANDAO_DIR, slot);
    let json_content = fs::read_to_string(path)?;
    let randao_hex: String = serde_json::from_str(&json_content)?;
    let randao_bytes = hex::decode(randao_hex)?;
    Ok(randao_bytes)
}

pub fn generate_hash_table(seed: &[u8], count: usize, shuffle_round: usize) -> Vec<[u8; 32]> {
    let count_exp = (count as f64).log2().ceil() as usize;
    let adjusted_count = 1 << count_exp;
    let size_per_round = (adjusted_count + 255) / 256;
    let table_size = shuffle_round * size_per_round;

    let mut table_inputs = vec![vec![0u8; 37]; table_size]; // MaxInputLength assumed 64

    table_inputs.iter_mut().enumerate().for_each(|(i, input)| {
        input[..32].copy_from_slice(&seed[..32]);
        input[32] = (i / size_per_round) as u8;
        LittleEndian::write_u32(&mut input[33..37], (i % size_per_round) as u32);
    });

    table_inputs
        .iter()
        .map(|input| {
            let hash = Sha256::digest(input);
            let mut result = [0u8; 32];
            result.copy_from_slice(&hash);
            result
        })
        .collect()
}

pub fn get_activated_validator_indices(slot: u64) -> Result<Vec<u64>, Box<dyn Error>> {
    let path = format!("{}ActivatedValidatorIndices{}.json", VALIDATOR_DIR, slot);
    let json_content = fs::read_to_string(path)?;
    let activated_validator_indices: Vec<u64> = serde_json::from_str(&json_content)?;
    Ok(activated_validator_indices)
}

fn shuffle_index(
    index: u64,
    index_count: u64,
    seed: &[u8],
    round: usize,
    bits: &[u8],
) -> (u64, Vec<u64>, Vec<u64>, Vec<u64>, Vec<u8>) {
    let mut flips = vec![0u64; round];
    let mut positions = vec![0u64; round];
    let mut flip_bits = vec![0u8; round];
    let mut pivots = vec![0u64; round];
    let mut current_index = index;

    for current_round in 0..round {
        let round_byte = current_round as u8;
        let mut to_hash = [seed, &[round_byte]].concat();
        let hash_res = Sha256::digest(&to_hash);

        let pivot = LittleEndian::read_u64(&hash_res[0..8]) % index_count;
        pivots[current_round] = pivot;

        let flip = (pivot + index_count - current_index) % index_count;
        let position = current_index.max(flip);
        let position_div = (position / 256) as u32;
        let mut position_bytes = [0u8; 4];
        LittleEndian::write_u32(&mut position_bytes, position_div);

        to_hash = [seed, &[round_byte], &position_bytes].concat();
        let source = Sha256::digest(&to_hash);
        let source_byte = source[((position % 256) / 8) as usize];
        let bit = (source_byte >> (position % 8)) % 2;

        if bit != bits[position as usize + current_round * bits.len() / round] {
            panic!("bit not equal");
        }
        if bit == 1 {
            current_index = flip;
        }
        flips[current_round] = flip;
        positions[current_round] = position;
        flip_bits[current_round] = bit;
    }

    (current_index, flips, positions, pivots, flip_bits)
}

pub fn shuffle_indices(
    indices: &[u64],
    seed: &[u8],
    bits: &[u8],
    shuffle_round: usize,
) -> (
    Vec<u64>,
    Vec<Vec<u64>>,
    Vec<Vec<u64>>,
    Vec<u64>,
    Vec<Vec<u8>>,
) {
    let mut shuffle_indices = vec![0u64; indices.len()];
    let mut flips = vec![vec![]; indices.len()];
    let mut positions = vec![vec![]; indices.len()];
    let mut flip_bits = vec![vec![]; indices.len()];
    let mut pivots = vec![0u64; indices.len()];
    // let mut shuffle_round_indices = vec![vec![]; indices.len()];

    for i in 0..indices.len() {
        let (shuffled_index, f, pos, piv, flip_b) =
            shuffle_index(i as u64, indices.len() as u64, seed, shuffle_round, bits);
        shuffle_indices[i] = shuffled_index;
        flips[i] = f;
        positions[i] = pos;
        pivots = piv;
        flip_bits[i] = flip_b;
    }

    (shuffle_indices, flips, positions, pivots, flip_bits)
}

pub fn load_committees(slot: u64) -> Result<Vec<BeaconCommitteeJson>, Box<dyn Error>> {
    let filepath = format!("{}{}.json", COMMITTEE_DIR, slot);
    let file_content = fs::read_to_string(filepath)?;
    let committees: Vec<BeaconCommitteeJson> = serde_json::from_str(&file_content)?;
    Ok(committees)
}

pub fn load_validators_from_file(slot: u64) -> Result<Vec<ValidatorPlain>, Box<dyn Error>> {
    let filepath = format!("{}validators{}.json", VALIDATOR_DIR, slot);
    let file_data = fs::read_to_string(filepath)?;
    let validators: Vec<ValidatorPlain> = serde_json::from_str(&file_data)?;
    Ok(validators)
}

pub fn load_attestations_and_bytes(slot: u64) -> Result<AttestationsWithBytes, Box<dyn Error>> {
    let path = format!("{}AttestationsAndParentRoot{}.json", ATTESTATION_DIR, slot);
    let json_data = fs::read_to_string(path)?;
    let wrapper: AttestationsWithBytes = serde_json::from_str(&json_data)?;
    Ok(wrapper)
}

pub fn load_target_attestations(start: u64, end: u64) -> Vec<Vec<Attestation>> {
    /*
       souceEpoch := epoch - 1
       targetEpoch := epoch - 0
       sourceBlockRoot := common.GetParentRootBySlot(souceEpoch*common.SLOTSPEREPOCH + 1)
       targetBlockRoot := common.GetParentRootBySlot(targetEpoch*common.SLOTSPEREPOCH + 1)
    */
    let epoch = start / SLOTSPEREPOCH;
    let source_epoch = epoch - 1;
    let target_epoch = epoch;
    let source_beacon_root = load_attestations_and_bytes(source_epoch * SLOTSPEREPOCH + 1)
        .unwrap()
        .data;
    let target_beacon_root = load_attestations_and_bytes(target_epoch * SLOTSPEREPOCH + 1)
        .unwrap()
        .data;
    let source_checkpoint = CheckpointPlain {
        epoch: source_epoch,
        root: general_purpose::STANDARD.encode(source_beacon_root),
    };
    let target_checkpoint = CheckpointPlain {
        epoch: target_epoch,
        root: general_purpose::STANDARD.encode(target_beacon_root),
    };
    let mut slots_attestations = vec![];
    let mut slots_beacon_root = vec!["".to_string(); SLOTSPEREPOCH as usize];
    for slot in start..end {
        let att_and_parent_root = load_attestations_and_bytes(slot + 1).unwrap();
        slots_attestations.extend_from_slice(att_and_parent_root.attestations.as_slice());
        slots_beacon_root[(slot % SLOTSPEREPOCH) as usize] =
            general_purpose::STANDARD.encode(att_and_parent_root.data);
    }
    //find the target attestation
    let mut candidate_attestations = vec![vec![]; SLOTSPEREPOCH as usize];
    for att in slots_attestations.iter() {
        if att.data.source == source_checkpoint && att.data.target == target_checkpoint {
            let current_slot = (att.data.slot % SLOTSPEREPOCH) as usize;
            if att.data.beacon_block_root == slots_beacon_root[current_slot] {
                candidate_attestations[current_slot].push(att.clone());
            }
        }
    }

    let mut final_attestations =
        vec![vec![Attestation::default(); MAXCOMMITTEESPERSLOT]; SLOTSPEREPOCH as usize];
    for i in 0..candidate_attestations.len() {
        for j in 0..candidate_attestations[i].len() {
            let cur_committee = candidate_attestations[i][j].data.committee_index as usize;
            if final_attestations[i][cur_committee] == Attestation::default() {
                final_attestations[i][cur_committee] = candidate_attestations[i][j].clone();
            } else {
                let cur_aggregation_count =
                    count_ones_in_aggregation_bits(&candidate_attestations[i][j].aggregation_bits)
                        .unwrap();
                let max_count = count_ones_in_aggregation_bits(
                    &final_attestations[i][cur_committee].aggregation_bits,
                )
                .unwrap();
                if cur_aggregation_count > max_count {
                    final_attestations[i][cur_committee] = candidate_attestations[i][j].clone();
                }
            }
        }
    }
    final_attestations
}
pub fn count_ones_in_aggregation_bits(base64_str: &str) -> Result<u32, Box<dyn std::error::Error>> {
    // 1. base64 decode to bytes
    let decoded_bytes = general_purpose::STANDARD.decode(base64_str)?;

    // 2. Count all 1's (bits set) in every byte
    let ones_count = decoded_bytes.iter().map(|byte| byte.count_ones()).sum();

    Ok(ones_count)
}
/// Converts a base64-encoded compressed G1 point into an `arkworks` G1Affine point.
pub fn base64_to_g1_point(base64_str: &str) -> Result<G1Affine, String> {
    // Decode the base64 string into a vector of bytes.
    let decoded_bytes = general_purpose::STANDARD
        .decode(base64_str)
        .map_err(|e| format!("Base64 decode error: {}", e))?;

    // Confirm the length (compressed G1Affine point should be 48 bytes)
    if decoded_bytes.len() != 48 {
        return Err(format!(
            "Expected 48 bytes for compressed G1Affine, got {} bytes",
            decoded_bytes.len()
        ));
    }

    // Deserialize the bytes into a G1Affine point.
    let point = G1Affine::deserialize_compressed(&*decoded_bytes)
        .map_err(|e| format!("Deserialization error: {:?}", e))?;

    // Return the result.
    Ok(point)
}
pub fn attestation_get_aggregation_bits_from_bytes(aggregation_bits: &str) -> Vec<u8> {
    // base64 decode to bytes
    let bytes = general_purpose::STANDARD.decode(aggregation_bits).unwrap();

    let mut bits = Vec::new();

    // Convert each byte into 8 bits and push them into the bits vector
    for byte in bytes {
        for j in 0..8 {
            bits.push(byte >> j & 1);
        }
    }

    // Remove trailing zeros after the last 1
    if let Some(last_one_index) = bits.iter().rposition(|&bit| bit == 1) {
        bits.truncate(last_one_index + 1);
    } else {
        // No 1 found, return an empty vector
        bits.clear();
    }

    bits
}
pub fn getting_pubkey_list_and_balance_with_validator_list_committee(
    slot: u64,
    committee_index: u64,
    aggregation_bits: &[u8], // borrowing instead of moving
    validator_list: &[ValidatorPlain],
    committees: &[BeaconCommitteeJson],
) -> (Vec<String>, u64) {
    const SLOTS_PER_EPOCH: u64 = 32;
    const MAX_COMMITTEES_PER_SLOT: u64 = 64;
    //committeeIndex = (slot%common.SLOTSPEREPOCH)*common.MAXCOMMITTEESPERSLOT + committeeIndex
    let committee_idx = (slot % SLOTS_PER_EPOCH) * MAX_COMMITTEES_PER_SLOT + committee_index;
    // Get a reference to validators directly (no cloning)
    let committee_indices = &committees[committee_idx as usize].validators;

    let mut new_pubkey_list: Vec<String> = Vec::new();
    let mut balances: u64 = 0;

    for (i, validator_index_string) in committee_indices.iter().enumerate() {
        if aggregation_bits.get(i) != Some(&1) {
            continue;
        }

        let validator_index = validator_index_string
            .parse::<usize>()
            .expect("Invalid validator index in committee");

        let validator = &validator_list[validator_index];
        new_pubkey_list.push(validator.public_key.clone());
        balances += validator.effective_balance;
    }

    (new_pubkey_list, balances)
}
pub fn bls_aggregate_pubkeys(pubkeys_string: Vec<String>) -> Option<G1Affine> {
    // base64 decode to bytes
    let pubkeys_bytes = (0..pubkeys_string.len())
        .map(|i| {
            general_purpose::STANDARD
                .decode(pubkeys_string[i].clone())
                .unwrap()
        })
        .collect::<Vec<Vec<u8>>>();
    if pubkeys_bytes.is_empty() {
        return None;
    }

    // Convert bytes to G1Affine points
    let mut pubkeys: Vec<G1Affine> = Vec::new();
    for bytes in pubkeys_bytes.iter() {
        match G1Affine::deserialize_compressed(bytes.as_slice()) {
            Ok(pk) => pubkeys.push(pk),
            Err(_) => continue, // Handle or log error as needed
        }
    }

    if pubkeys.is_empty() {
        return None;
    }

    // Initialize with the first pubkey
    let mut agg_proj = pubkeys[0].into_group();

    // Add remaining pubkeys
    for pk in pubkeys.iter().skip(1) {
        agg_proj += pk.into_group();
    }

    // Convert back to affine and serialize to compressed bytes
    let agg_affine: G1Affine = agg_proj.into();

    Some(agg_affine)
}
pub fn parallel_process_attestations(
    new_slot_attestations: &[Vec<Attestation>],
    current_slot: u64,
    validator_list: &[ValidatorPlain],  // Placeholder type
    committees: &[BeaconCommitteeJson], // Placeholder type
) -> (Vec<G1Affine>, Vec<u64>) {
    const MAX_COMMITTEES_PER_SLOT: usize = 64; // Example constant, replace with actual

    let aggregated_pubkey_list = Arc::new(Mutex::new(vec![
        G1Affine::default();
        new_slot_attestations.len()
            * MAX_COMMITTEES_PER_SLOT
    ]));
    let balances_list = Arc::new(Mutex::new(vec![
        0u64;
        new_slot_attestations.len()
            * MAX_COMMITTEES_PER_SLOT
    ]));

    new_slot_attestations
        .par_iter()
        .enumerate()
        .for_each(|(i, att_list)| {
            for (j, attestation) in att_list.iter().enumerate() {
                let aggregation_bits =
                    attestation_get_aggregation_bits_from_bytes(&attestation.aggregation_bits);
                let (att_pubkey_list, balances) =
                    getting_pubkey_list_and_balance_with_validator_list_committee(
                        i as u64 + current_slot,
                        j as u64,
                        &aggregation_bits,
                        validator_list,
                        committees,
                    );

                let aggregated_pubkey = bls_aggregate_pubkeys(att_pubkey_list);
                // test_aggregated_pubkey(&aggregated_pubkey, attestation);

                let mut agg_pubkey_list = aggregated_pubkey_list.lock().unwrap();
                match aggregated_pubkey {
                    Some(agg_pubkey) => {
                        agg_pubkey_list[i * MAX_COMMITTEES_PER_SLOT + j] = agg_pubkey
                    }
                    None => (),
                }

                let mut bal_list = balances_list.lock().unwrap();
                bal_list[i * MAX_COMMITTEES_PER_SLOT + j] = balances;
            }
        });

    let aggregated_pubkey_list = Arc::try_unwrap(aggregated_pubkey_list)
        .expect("Lock still held")
        .into_inner()
        .unwrap();
    let balances_list = Arc::try_unwrap(balances_list)
        .expect("Lock still held")
        .into_inner()
        .unwrap();

    (aggregated_pubkey_list, balances_list)
}
pub fn prepare_assignment_data(
    start: u64,
    end: u64,
) -> (
    Vec<u8>,
    Vec<u64>,
    Vec<u64>,
    Vec<u64>,
    Vec<u64>,
    Vec<Vec<u64>>,
    Vec<Vec<u64>>,
    Vec<Vec<u8>>,
    Vec<Vec<u8>>,
    Vec<Vec<Attestation>>,
    Vec<G1Affine>,
    Vec<u64>,
    Vec<u64>,
    Vec<Vec<Vec<u32>>>,
    Vec<[u8; 32]>,
    Vec<ValidatorPlain>,
) {
    let epoch = start / SLOTSPEREPOCH;
    let seed = get_beacon_seed(epoch).unwrap();
    let first_slot = start / SLOTSPEREPOCH * SLOTSPEREPOCH;
    let activated_indices = get_activated_validator_indices(first_slot).unwrap();

    //get the hash table
    let hash_bytes = generate_hash_table(&seed, activated_indices.len(), SHUFFLEROUND);
    //convert hash bytes to bits
    let mut hash_bits = vec![0u8; hash_bytes.len() * 256];
    for (i, hash_byte) in hash_bytes.iter().enumerate() {
        for j in 0..32 {
            for k in 0..8 {
                hash_bits[i * 256 + j * 8 + k] = (hash_byte[j] >> k) & 1;
            }
        }
    }
    let bits_per_round = hash_bits.len() / SHUFFLEROUND;
    let round_hash_bits: Vec<Vec<u8>> = (0..SHUFFLEROUND)
        .map(|i| hash_bits[i * bits_per_round..(i + 1) * bits_per_round].to_vec())
        .collect();
    println!("get round hash bits");

    //shuffle the indices
    let (shuffle_indices, flips, positions, pivots, flip_bits) =
        shuffle_indices(&activated_indices, &seed, &hash_bits, SHUFFLEROUND);

    println!("get shuffle_indices");
    //get committees from chain, check it with the shuffled indices
    let committees = load_committees(first_slot).unwrap();
    let mut real_committee_size = vec![];
    for committee in committees.iter() {
        real_committee_size.push(committee.validators.len() as u64);
    }
    let committee_indices: Vec<u64> = committees
        .iter()
        .flat_map(|c| c.validators.iter().map(|s| s.parse::<u64>().unwrap()))
        .collect();
    println!("get committee_indices");

    let validator_list = load_validators_from_file(first_slot).unwrap();
    // let mut total_effective_balance = 0;
    // for i in 0..activated_indices.len() {
    //     total_effective_balance += validator_list[activated_indices[i] as usize].effective_balance;
    // }

    let validator_tree;
    let validator_tree_filename = format!("{}poseidon_{}.txt", LOCAL_TREE_DIR, first_slot);
    if Path::new(&validator_tree_filename).exists() {
        validator_tree = load_nested_vec(&validator_tree_filename).unwrap();
    } else {
        let validator_list_arc = Arc::new(validator_list.clone());
        let validator_hashes = Arc::new(Mutex::new(vec![vec![]; validator_list_arc.len()]));
        let thread_num = 64;
        let chunk_size = (validator_list_arc.len() + thread_num - 1) / thread_num;
        let mut handles = vec![];
        for i in 0..thread_num {
            let validator_list = Arc::clone(&validator_list_arc);
            let validator_hashes = Arc::clone(&validator_hashes);
            let handle = thread::spawn(move || {
                let start = i * chunk_size;
                let end = std::cmp::min((i + 1) * chunk_size, validator_list.len());
                for j in start..end {
                    let validator_hash = validator_list[j].hash();
                    let mut hashes = validator_hashes.lock().unwrap();
                    hashes[j] = validator_hash;
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let validator_hashes = Arc::try_unwrap(validator_hashes)
            .expect("Arc still has multiple owners")
            .into_inner()
            .unwrap();
        //calculate and save validator tree
        validator_tree =
            calculate_and_save_validator_tree(validator_tree_filename, validator_hashes);
    }
    println!("get validator_tree");
    let attestations = load_target_attestations(start, end);
    let (aggregated_pubkeys, balance_list) =
        parallel_process_attestations(&attestations, first_slot, &validator_list, &committees);
    println!("get attestations");
    (
        seed,
        shuffle_indices,
        committee_indices,
        pivots,
        activated_indices,
        flips,
        positions,
        flip_bits,
        round_hash_bits,
        attestations,
        aggregated_pubkeys,
        balance_list,
        real_committee_size,
        validator_tree,
        hash_bytes,
        validator_list,
    )
}
pub fn calculate_and_save_validator_tree(
    filename: String,
    validator_hashes: Vec<Vec<u32>>,
) -> Vec<Vec<Vec<u32>>> {
    let params = PoseidonParams::new(
        POSEIDON_M31X16_RATE,
        16,
        POSEIDON_M31X16_FULL_ROUNDS,
        POSEIDON_M31X16_PARTIAL_ROUNDS,
    );
    let mut tree =
        merkle_tree_element_with_limit(&validator_hashes, &params, MAXBEACONVALIDATORSIZE).unwrap();

    let merkle_tree_root_mixin =
        merkleize_with_mixin_poseidon(&tree[0][0], validator_hashes.len() as u64, &params);

    let mut zero_level = vec![vec![merkle_tree_root_mixin.clone()]];

    zero_level.append(&mut tree);
    tree = zero_level;

    save_nested_vec(&filename, &tree).expect("Failed to save tree to file");

    tree
}
fn save_nested_vec(path: &str, data: &Vec<Vec<Vec<u32>>>) -> std::io::Result<()> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    bincode::serialize_into(writer, data).expect("bincode serialization failed");
    Ok(())
}
fn load_nested_vec(path: &str) -> std::io::Result<Vec<Vec<Vec<u32>>>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let result: Vec<Vec<Vec<u32>>> =
        bincode::deserialize_from(reader).expect("bincode deserialization failed");
    Ok(result)
}

// #[test]
// fn test_get_beacon_seed() {
//     init_directories().unwrap();
//     let seed = get_beacon_seed(290000).unwrap();
//     assert_eq!(seed.len(), 32);
//     println!("{:?}", seed);
// }

// #[test]
// fn test_get_activated_validator_indices() {
//     let indices = get_activated_validator_indices(3988672).unwrap();
//     println!("{:?}", indices.len());
// }

// #[test]
// fn test_shuffle_indices() {
//     let indices = get_activated_validator_indices(3988672).unwrap();
//     let seed = get_beacon_seed(124646).unwrap();
//     let hash_bytes = generate_hash_table(&seed, indices.len(), 90);
//     let mut hash_bits = vec![0u8; hash_bytes.len() * 256];

//     for (i, hash_byte) in hash_bytes.iter().enumerate() {
//         for j in 0..32 {
//             for k in 0..8 {
//                 hash_bits[i * 256 + j * 8 + k] = (hash_byte[j] >> k) & 1;
//             }
//         }
//     }
//     let (shuffle_indices, flips, positions, pivots, flip_bits) =
//         shuffle_indices(&indices, &seed, &hash_bits, 90);
// }

// #[test]
// fn test_load_committees() {
//     let committees = load_committees(3988672).unwrap();
//     println!("{:?}", committees.len());
//     println!("{:?}", committees[0].validators);
// }

// #[test]
// fn test_load_validators_from_file() {
//     let validators = load_validators_from_file(3988672).unwrap();
//     println!("{:?}", validators.len());
//     println!("{:?}", validators[0].public_key);
// }

// #[test]
// fn test_load_attestations_and_bytes() {
//     let wrapper = load_attestations_and_bytes(3988672).unwrap();
//     println!("{:?}", wrapper.attestations);
//     println!("{:?}", wrapper.data);
// }

// #[test]
// fn test_prepare_assignment_data() {
//     let epoch = 290000;
//     let slot = epoch * SLOTSPEREPOCH;
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
//     ) = prepare_assignment_data(slot, slot + 32);
//     println!("aggregated_pubkeys: {:?}", aggregated_pubkeys);
// }
