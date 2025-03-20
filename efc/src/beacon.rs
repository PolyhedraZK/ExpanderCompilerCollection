//read beacon info from files
use std::{error::Error, fs};
use std::path::Path;
use byteorder::{ByteOrder, LittleEndian};

use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::attestation::Attestation;

const SUBCIRCUIT_TREE_CACHE_DIR: &str = "./data/subcircuitTreeCache/";
const CACHE_DIR: &str = "./data/cache/";
const LOCAL_TREE_DIR: &str = "./data/localTree/";
const RANDAO_DIR: &str = "./data/beacon/randao/";
const VALIDATOR_DIR: &str = "./data/beacon/validator/";
const COMMITTEE_DIR: &str = "./data/beacon/committee/";
const ATTESTATION_DIR: &str = "./data/beacon/attestations/";
const DOMAIN_BEACON_ATTESTER: &str = "01000000";

const SLOTSPEREPOCH: u64 = 32;

#[derive(Debug, Deserialize)]
pub struct BeaconCommitteeJson {
    // Adjust these fields based on your actual Go struct
    pub slot: String,
    pub index: String,
    pub validators: Vec<String>,
}
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
        general_purpose::STANDARD.decode(s.as_bytes()).map_err(serde::de::Error::custom)
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
    table_inputs.iter()
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

fn shuffle_index(index: u64, index_count: u64, seed: &[u8], round: usize, bits: &[u8]) -> (u64, Vec<u64>, Vec<u64>, Vec<u64>, Vec<u64>, Vec<u8>) {
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

    (current_index, flips, positions, pivots, round_index, flip_bits)
}

pub fn shuffle_indices(indices: &[u64], seed: &[u8], bits: &[u8], shuffle_round: usize) -> (Vec<u64>, Vec<Vec<u64>>, Vec<Vec<u64>>, Vec<Vec<u64>>, Vec<u64>, Vec<Vec<u8>>) {
    let mut shuffle_indices = vec![0u64; indices.len()];
    let mut flips = vec![vec![]; indices.len()];
    let mut positions = vec![vec![]; indices.len()];
    let mut flip_bits = vec![vec![]; indices.len()];
    let mut pivots = vec![0u64; indices.len()];
    let mut shuffle_round_indices = vec![vec![]; indices.len()];

    for i in 0..indices.len() {
        let (shuffled_index, f, pos, piv, round_idx, flip_b) = shuffle_index(i as u64, indices.len() as u64, seed, shuffle_round, bits);
        shuffle_indices[i] = shuffled_index;
        flips[i] = f;
        positions[i] = pos;
        pivots = piv;
        shuffle_round_indices[i] = round_idx;
        flip_bits[i] = flip_b;
    }

    (shuffle_indices, flips, positions, shuffle_round_indices, pivots, flip_bits)
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
fn test_shuffle_indices(){
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
    let (shuffle_indices, flips, positions, shuffle_round_indices, pivots, flip_bits) = shuffle_indices(&indices, &seed, &hash_bits, 90);
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