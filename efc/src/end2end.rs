use expander_compiler::frontend::{M31Config, WitnessSolver};

use crate::attestation::Attestation;
use crate::bls_verifier::{
    end2end_pairing_witness, generate_pairing_witnesses, PairingCircuit, PairingEntry,
};
use crate::hashtable::{
    end2end_hashtable_witnesses, generate_hash_witnesses, HASHTABLECircuit, HashTableJson,
};
use crate::permutation::{
    end2end_permutation_hashbit_witness, generate_permutation_hashes_witness, PermutationHashEntry,
    PermutationIndicesValidatorHashBitCircuit, VALIDATOR_COUNT,
};
use crate::shuffle::{
    end2end_shuffle_witnesses, generate_shuffle_witnesses, ShuffleCircuit, ShuffleJson,
};
use crate::utils::{get_solver, read_from_json_file, wait_for_file};
use crate::validator::ValidatorPlain;
use std::thread;

pub fn end2end_witness(dir: &str) {
    let start_time = std::time::Instant::now();
    let dir_str1 = dir.to_string();
    let shuffle_thread = thread::spawn(move || {
        generate_shuffle_witnesses(&dir_str1);
    });

    let dir_str = dir.to_string();
    let hash_thread = thread::spawn(move || {
        generate_hash_witnesses(&dir_str);
    });

    let dir_str = dir.to_string();
    let pairing_thread = thread::spawn(move || {
        generate_pairing_witnesses(&dir_str);
    });

    let dir_str = dir.to_string();
    let permutation_hash_thread = thread::spawn(move || {
        generate_permutation_hashes_witness(&dir_str);
    });

    shuffle_thread.join().expect("Shuffle thread panicked");
    hash_thread.join().expect("Hash thread panicked");
    pairing_thread.join().expect("Pairing thread panicked");
    permutation_hash_thread
        .join()
        .expect("Permutation hash thread panicked");
    let end_time = std::time::Instant::now();
    println!(
        "generate end2end witness, time: {:?}",
        end_time.duration_since(start_time)
    );
}

//at the end of the last prove process (e.g., epoch = N - 1), generate the following witnesses for next epoch (epoch = N):
//1. the first half of the shuffle witnesses (slot 0 to 15)
//2. the first half of the bls_verifier witnesses (slot 0 to 15)
//3. all hash witnesses
//4. all permutation_hash witnesses
pub fn end2end_witness_streamline_end(
    dir: &str,
    solver_shuffle: WitnessSolver<M31Config>,
    solver_hash: WitnessSolver<M31Config>,
    solver_pairing: WitnessSolver<M31Config>,
    solver_permutation_hash: WitnessSolver<M31Config>,
) {
    println!("loading assignment data...");
    let start_time = std::time::Instant::now();

    let file_path = format!("{}/validatorList.json", dir);
    let plain_validators: Vec<ValidatorPlain> = read_from_json_file(&file_path).unwrap();

    let file_path = format!("{}/shuffle_assignment.json", dir);
    let shuffle_data: Vec<ShuffleJson> = read_from_json_file(&file_path).unwrap();

    let file_path = format!("{}/pubkeyBLSList.json", dir);
    let public_key_bls_list: Vec<Vec<String>> = read_from_json_file(&file_path).unwrap();

    let file_path = format!("{}/slotAttestationsFolded.json", dir);
    let attestations: Vec<Attestation> = read_from_json_file(&file_path).unwrap();

    let file_path = format!("{}/pairing_assignment.json", dir);
    let pairing_data: Vec<PairingEntry> = read_from_json_file(&file_path).unwrap();

    let file_path = format!("{}/hash_assignment.json", dir);
    let hashtable_data: Vec<HashTableJson> = read_from_json_file(&file_path).unwrap();

    let file_path = format!("{}/permutationhash_assignment.json", dir);
    let permutation_hash_data: Vec<PermutationHashEntry> = read_from_json_file(&file_path).unwrap();
    let end_time = std::time::Instant::now();
    println!(
        "loaed assignment data, time: {:?}",
        end_time.duration_since(start_time)
    );

    // let shuffle_pairing_thread = thread::spawn(move || {
    end2end_shuffle_witnesses(
        solver_shuffle,
        plain_validators,
        shuffle_data,
        public_key_bls_list,
        attestations,
        pairing_data.clone(),
    );
    end2end_pairing_witness(solver_pairing, pairing_data);
    // });

    // let hash_thread = thread::spawn(move || {
    end2end_hashtable_witnesses(solver_hash, hashtable_data);
    // });

    // let permutation_hash_thread = thread::spawn(move || {
    end2end_permutation_hashbit_witness(solver_permutation_hash, permutation_hash_data);
    // });

    // shuffle_pairing_thread
    //     .join()
    //     .expect("ShufflePairing thread panicked");
    // hash_thread.join().expect("Hash thread panicked");
    // permutation_hash_thread
    //     .join()
    //     .expect("Permutation hash thread panicked");
    let end_time = std::time::Instant::now();
    println!(
        "generate end2end end witness, time: {:?}",
        end_time.duration_since(start_time)
    );
}

//at the start of the current prove process (e.g., epoch = N - 1), generate the following witnesses for current epoch (e.g., epoch = N - 1):
//1. the second half of the shuffle witnesses (slot 16 to 31)
//2. the second half of the bls_verifier witnesses (slot 16 to 31)
pub fn end2end_witness_streamline_start(
    dir: &str,
    solver_shuffle: WitnessSolver<M31Config>,
    solver_pairing: WitnessSolver<M31Config>,
) {
    println!("loading assignment data...");
    let start_time = std::time::Instant::now();

    let file_path = format!("{}/validatorList.json", dir);
    let plain_validators: Vec<ValidatorPlain> = read_from_json_file(&file_path).unwrap();

    let file_path = format!("{}/shuffle_assignment.json", dir);
    let shuffle_data: Vec<ShuffleJson> = read_from_json_file(&file_path).unwrap();

    let file_path = format!("{}/pubkeyBLSList.json", dir);
    let public_key_bls_list: Vec<Vec<String>> = read_from_json_file(&file_path).unwrap();

    let file_path = format!("{}/slotAttestationsFolded.json", dir);
    let attestations: Vec<Attestation> = read_from_json_file(&file_path).unwrap();

    let file_path = format!("{}/pairing_assignment.json", dir);
    let pairing_data: Vec<PairingEntry> = read_from_json_file(&file_path).unwrap();

    let end_time = std::time::Instant::now();
    println!(
        "loaed assignment data, time: {:?}",
        end_time.duration_since(start_time)
    );

    let shuffle_pairing_thread = thread::spawn(move || {
        end2end_shuffle_witnesses(
            solver_shuffle,
            plain_validators,
            shuffle_data,
            public_key_bls_list,
            attestations,
            pairing_data.clone(),
        );
        end2end_pairing_witness(solver_pairing, pairing_data);
    });

    shuffle_pairing_thread
        .join()
        .expect("ShufflePairing thread panicked");
    let end_time = std::time::Instant::now();
    println!(
        "generate end2end start witness, time: {:?}",
        end_time.duration_since(start_time)
    );
}

pub fn end2end_witness_streamline(stage: &str) {
    if stage == "end" {
        println!("end stage");
        //get the solver for shuffle
        let dir = "./witnesses/shuffle";
        let circuit_name = "shuffle";
        let solver_shuffle = get_solver(dir, circuit_name, ShuffleCircuit::default());

        //get the solver for hash
        let dir = "./witnesses/hashtable";
        let circuit_name = "hashtable";
        let solver_hash = get_solver(dir, circuit_name, HASHTABLECircuit::default());

        //get the solver for pairing
        let dir = "./witnesses/pairing";
        let circuit_name = "pairing";
        let solver_pairing = get_solver(dir, circuit_name, PairingCircuit::default());

        //get the solver for permutation hash
        let dir = "./witnesses/permutationhashbit";
        let circuit_name = format!("permutationhashbit_{}", VALIDATOR_COUNT);
        let solver_permutation_hash = get_solver(
            dir,
            &circuit_name,
            PermutationIndicesValidatorHashBitCircuit::default(),
        );
        let dir = "./efc/data";
        end2end_witness_streamline_end(
            dir,
            solver_shuffle,
            solver_hash,
            solver_pairing,
            solver_permutation_hash,
        );
    } else if stage == "start" {
        println!("start stage");
        //get the solver for shuffle
        let dir = "./witnesses/shuffle";
        let circuit_name = "shuffle";
        let solver_shuffle = get_solver(dir, circuit_name, ShuffleCircuit::default());

        //get the solver for pairing
        let dir = "./witnesses/pairing";
        let circuit_name = "pairing";
        let solver_pairing = get_solver(dir, circuit_name, PairingCircuit::default());
        let dir = "./efc/data";
        end2end_witness_streamline_start(dir, solver_shuffle, solver_pairing);
    }
}
