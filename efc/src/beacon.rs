//read beacon info from files
use base64::engine::general_purpose;
use base64::Engine;
use bincode;
use byteorder::{ByteOrder, LittleEndian};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::path::Path;
use std::{error::Error, fs};

use crate::attestation::{Attestation, CheckpointPlain};
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

const SLOTSPEREPOCH: u64 = 32;
const SHUFFLEROUND: usize = 90;
const MAXCOMMITTEESPERSLOT: usize = 128;
const MAXBEACONVALIDATORDEPTH: usize = 40;
const MAXBEACONVALIDATORSIZE: usize = 1 << MAXBEACONVALIDATORDEPTH;

#[derive(Debug, Deserialize)]
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
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&general_purpose::STANDARD.encode(bytes))
    }

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

    for i in 0..table_size {
        table_inputs[i][..32].copy_from_slice(&seed[..32]);
        table_inputs[i][32] = (i / size_per_round) as u8;
        LittleEndian::write_u32(&mut table_inputs[i][33..37], (i % size_per_round) as u32);
    }
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
) -> (u64, Vec<u64>, Vec<u64>, Vec<u64>, Vec<u64>, Vec<u8>) {
    let mut flips = vec![0u64; round];
    let mut positions = vec![0u64; round];
    let mut flip_bits = vec![0u8; round];
    let mut pivots = vec![0u64; round];
    let mut round_index = vec![0u64; round];
    let mut current_index = index;

    for current_round in 0..round {
        round_index[current_round] = current_index;
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

    (
        current_index,
        flips,
        positions,
        pivots,
        round_index,
        flip_bits,
    )
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
    Vec<Vec<u64>>,
    Vec<u64>,
    Vec<Vec<u8>>,
) {
    let mut shuffle_indices = vec![0u64; indices.len()];
    let mut flips = vec![vec![]; indices.len()];
    let mut positions = vec![vec![]; indices.len()];
    let mut flip_bits = vec![vec![]; indices.len()];
    let mut pivots = vec![0u64; indices.len()];
    let mut shuffle_round_indices = vec![vec![]; indices.len()];

    for i in 0..indices.len() {
        let (shuffled_index, f, pos, piv, round_idx, flip_b) =
            shuffle_index(i as u64, indices.len() as u64, seed, shuffle_round, bits);
        shuffle_indices[i] = shuffled_index;
        flips[i] = f;
        positions[i] = pos;
        pivots = piv;
        shuffle_round_indices[i] = round_idx;
        flip_bits[i] = flip_b;
    }

    (
        shuffle_indices,
        flips,
        positions,
        shuffle_round_indices,
        pivots,
        flip_bits,
    )
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

pub fn load_target_attestations(start: u64, end: u64) -> Vec<Attestation> {
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
        root: source_beacon_root.try_into().unwrap(),
    };
    let target_checkpoint = CheckpointPlain {
        epoch: target_epoch,
        root: target_beacon_root.try_into().unwrap(),
    };
    let new_slot_attestations = vec![];
    /*
    epochAttestations := make([]*ethpb.Attestation, 0)
        beaconRoot := make([][]byte, 0)
        for i := 0; i < common.SLOTSPEREPOCH; i++ {
            //block is one slot ahead of attestation
            _, block, _, err := common.GetBeaconBlockBySlot(epoch*common.SLOTSPEREPOCH + uint64(i) + 1)
            if err != nil {
                panic("Error in getting block by slot")
            }
            attestations := structure.Attestations(block)
            parentRoot := structure.ParentRoot(block)
            epochAttestations = append(epochAttestations, attestations...)
            beaconRoot = append(beaconRoot, parentRoot)
        }
     */
    let mut slots_attestations = vec![];
    let mut slots_beacon_root = vec![String; SLOTSPEREPOCH as usize];
    for slot in start..end {
        let att_and_parent_root = load_attestations_and_bytes(slot).unwrap();
        slots_attestations.extend_from_slice(att_and_parent_root.attestations.as_slice());
        slots_beacon_root[(slot % SLOTSPEREPOCH) as usize] = att_and_parent_root.data;
    }
    /*
    //find the target attestation
        attestationsString := make([]string, len(epochAttestations))
        for i := 0; i < len(epochAttestations); i++ {
            attestationsString[i] = structure.AttestationCheckPointsToString(epochAttestations[i].Data)
        }
        targetAttIndices := FindTargetAttIndices(attestationsString, attDataToConsensusString)
        slotAttestations := make([][]*ethpb.Attestation, common.SLOTSPEREPOCH)
        for i := 0; i < len(targetAttIndices); i++ {
            leftString := string(epochAttestations[targetAttIndices[i]].Data.BeaconBlockRoot)
            rightString := string(beaconRoot[uint64(epochAttestations[targetAttIndices[i]].Data.Slot)-epoch*common.SLOTSPEREPOCH])
            if leftString != rightString {
                continue
            }
            slotAttestations[uint64(epochAttestations[targetAttIndices[i]].Data.Slot)-currentSlot] = append(slotAttestations[uint64(epochAttestations[targetAttIndices[i]].Data.Slot)-currentSlot], epochAttestations[targetAttIndices[i]])
        }
     */
    //find the target attestation
    let mut candidate_attestations = vec![vec![]; SLOTSPEREPOCH as usize];
    for i in 0..slots_attestations.len() {
        if slots_attestations[i].data.source == source_checkpoint
            && slots_attestations[i].data.target == target_checkpoint
        {
            let current_slot = (slots_attestations[i].data.slot % SLOTSPEREPOCH) as usize;
            let left_string = slots_attestations[i].data.beacon_block_root;
            let right_string = slots_beacon_root[current_slot];
            if left_string == right_string {
                candidate_attestations[current_slot].push(slots_attestations[i].clone());
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
    slots_attestations
}
pub fn count_ones_in_aggregation_bits(base64_str: &str) -> Result<u32, Box<dyn std::error::Error>> {
    // 1. base64 decode to bytes
    let decoded_bytes = general_purpose::STANDARD.decode(base64_str)?;

    // 2. Count all 1's (bits set) in every byte
    let ones_count = decoded_bytes.iter().map(|byte| byte.count_ones()).sum();

    Ok(ones_count)
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
    Vec<Vec<u64>>,
    Vec<Vec<u8>>,
    Vec<Vec<u8>>,
    Vec<Vec<Attestation>>,
    Vec<bls12381::G1Affine>,
    Vec<u64>,
    Vec<u64>,
    Vec<Vec<Vec<u64>>>,
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
    //shuffle the indices
    let (shuffle_index, flips, positions, shuffle_round_indices, pivots, flip_bits) =
        shuffle_indices(&activated_indices, &seed, &hash_bits, SHUFFLEROUND);

    //get committees from chain, check it with the shuffled indices
    let committees = load_committees(first_slot).unwrap();
    let mut real_committee_size = vec![];
    for committee in committees.iter() {
        real_committee_size.push(committee.validators.len());
    }

    let validator_list = load_validators_from_file(first_slot).unwrap();
    let mut total_effective_balance = 0;
    for i in 0..activated_indices.len() {
        total_effective_balance += validator_list[activated_indices[i] as usize].effective_balance;
    }

    let mut validator_tree = vec![];
    let validator_tree_filename = format!("{}poseidon_{}.txt", LOCAL_TREE_DIR, first_slot);
    if Path::new(&validator_tree_filename).exists() {
        validator_tree = load_nested_vec(&validator_tree_filename).unwrap();
    } else {
        let mut validator_hashes = vec![vec![]; validator_list.len()];
        let thread_num = 64;
        let chunk_size = (validator_list.len() + thread_num - 1) / thread_num;
        for i in 0..thread_num {
            for j in i * chunk_size..std::cmp::min((i + 1) * chunk_size, validator_list.len()) {
                let validator_hash = validator_list[j].hash();
                validator_hashes[j] = validator_hash;
            }
        }
        //calculate and save validator tree
        validator_tree =
            calculate_and_save_validator_tree(validator_tree_filename, validator_hashes);
    }
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

#[test]
fn test_get_beacon_seed() {
    init_directories().unwrap();
    let seed = get_beacon_seed(290000).unwrap();
    assert_eq!(seed.len(), 32);
    println!("{:?}", seed);
}

#[test]
fn test_get_activated_validator_indices() {
    let indices = get_activated_validator_indices(3988672).unwrap();
    println!("{:?}", indices.len());
}

#[test]
fn test_shuffle_indices() {
    let indices = get_activated_validator_indices(3988672).unwrap();
    let seed = get_beacon_seed(124646).unwrap();
    let hash_bytes = generate_hash_table(&seed, indices.len(), 90);
    let mut hash_bits = vec![0u8; hash_bytes.len() * 256];

    for (i, hash_byte) in hash_bytes.iter().enumerate() {
        for j in 0..32 {
            for k in 0..8 {
                hash_bits[i * 256 + j * 8 + k] = (hash_byte[j] >> k) & 1;
            }
        }
    }
    let (shuffle_indices, flips, positions, shuffle_round_indices, pivots, flip_bits) =
        shuffle_indices(&indices, &seed, &hash_bits, 90);
    // println!("{:?}", shuffle_indices);
    // println!("{:?}", flips);
    // println!("{:?}", positions);
    // println!("{:?}", shuffle_round_indices);
    // println!("{:?}", pivots);
    // println!("{:?}", flip_bits);
}

#[test]
fn test_load_committees() {
    let committees = load_committees(3988672).unwrap();
    println!("{:?}", committees.len());
    println!("{:?}", committees[0].validators);
}

#[test]
fn test_load_validators_from_file() {
    let validators = load_validators_from_file(3988672).unwrap();
    println!("{:?}", validators.len());
    println!("{:?}", validators[0].public_key);
}

#[test]
fn test_load_attestations_and_bytes() {
    let wrapper = load_attestations_and_bytes(3988672).unwrap();
    println!("{:?}", wrapper.attestations);
    println!("{:?}", wrapper.data);
}
