//read beacon info from files
use std::{error::Error, fs};
use std::path::Path;
use byteorder::{ByteOrder, LittleEndian};

use sha2::{Digest, Sha256};

const SUBCIRCUIT_TREE_CACHE_DIR: &str = "./data/subcircuitTreeCache/";
const CACHE_DIR: &str = "./data/cache/";
const LOCAL_TREE_DIR: &str = "./data/localTree/";
const RANDAO_DIR: &str = "./data/beacon/randao/";
const DOMAIN_BEACON_ATTESTER: &str = "01000000";
const SLOTSPEREPOCH: u64 = 32;


pub fn init_directories() -> std::io::Result<()> {
    fs::create_dir_all(Path::new(SUBCIRCUIT_TREE_CACHE_DIR))?;
    fs::create_dir_all(Path::new(CACHE_DIR))?;
    fs::create_dir_all(Path::new(LOCAL_TREE_DIR))?;
    fs::create_dir_all(Path::new(RANDAO_DIR))?;
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
#[test]
fn test_get_beacon_seed() {
    init_directories().unwrap();
    let seed = get_beacon_seed(290000).unwrap();
    assert_eq!(seed.len(), 32);
    println!("{:?}", seed);
}