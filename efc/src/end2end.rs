use expander_compiler::frontend::{M31Config, WitnessSolver};

use crate::attestation::Attestation;
use crate::bls_verifier::{
    end2end_pairing_witness, generate_pairing_witnesses, PairingCircuit, PairingEntry,
};
use crate::hashtable::{
    end2end_hashtable_witnesses, generate_hash_witnesses, HASHTABLECircuit, HashTableJson,
};
use crate::permutation::{
    end2end_permutation_hashbit_witness, generate_permutation_hashes_witnesses, PermutationHashEntry,
    PermutationIndicesValidatorHashBitCircuit, VALIDATOR_COUNT,
};
use crate::shuffle::{
    end2end_shuffle_witnesses, generate_shuffle_witnesses, ShuffleCircuit, ShuffleJson,
};
use crate::utils::{get_solver, read_from_json_file, wait_for_file};
use crate::validator::ValidatorPlain;
use std::sync::{Arc, Mutex};
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
        generate_permutation_hashes_witnesses(&dir_str);
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

    let plain_validators = Arc::new(Mutex::new(Vec::<ValidatorPlain>::new()));
    let shuffle_data = Arc::new(Mutex::new(Vec::<ShuffleJson>::new()));
    let public_key_bls_list = Arc::new(Mutex::new(Vec::<Vec<String>>::new()));
    let attestations = Arc::new(Mutex::new(Vec::<Attestation>::new()));
    let pairing_data = Arc::new(Mutex::new(Vec::<PairingEntry>::new()));
    let hashtable_data = Arc::new(Mutex::new(Vec::<HashTableJson>::new()));
    let permutation_hash_data = Arc::new(Mutex::new(Vec::<PermutationHashEntry>::new()));

    let plain_validators_clone = Arc::clone(&plain_validators);
    let dir_clone1 = dir.to_string();
    let handle_validators = thread::spawn(move || {
        let file_path = format!("{}/validatorList.json", dir_clone1);
        if let Ok(data) = read_from_json_file::<Vec<ValidatorPlain>>(&file_path) {
            let mut validators = plain_validators_clone.lock().unwrap();
            *validators = data;
        }
    });

    let shuffle_data_clone = Arc::clone(&shuffle_data);
    let dir_clone2 = dir.to_string();
    let handle_shuffle = thread::spawn(move || {
        let file_path = format!("{}/shuffle_assignment.json", dir_clone2);
        if let Ok(data) = read_from_json_file::<Vec<ShuffleJson>>(&file_path) {
            let mut shuffle_data = shuffle_data_clone.lock().unwrap();
            *shuffle_data = data;
        }
    });

    let public_key_bls_list_clone = Arc::clone(&public_key_bls_list);
    let dir_clone3 = dir.to_string();
    let handle_pubkey = thread::spawn(move || {
        let file_path = format!("{}/pubkeyBLSList.json", dir_clone3);
        if let Ok(data) = read_from_json_file::<Vec<Vec<String>>>(&file_path) {
            let mut public_key_bls_list = public_key_bls_list_clone.lock().unwrap();
            *public_key_bls_list = data;
        }
    });

    let attestations_clone = Arc::clone(&attestations);
    let dir_clone4 = dir.to_string();
    let handle_att = thread::spawn(move || {
        let file_path = format!("{}/slotAttestationsFolded.json", dir_clone4);
        if let Ok(data) = read_from_json_file::<Vec<Attestation>>(&file_path) {
            let mut attestations = attestations_clone.lock().unwrap();
            *attestations = data;
        }
    });

    let pairing_data_clone = Arc::clone(&pairing_data);
    let dir_clone5 = dir.to_string();
    let handle_pairing = thread::spawn(move || {
        let file_path = format!("{}/pairing_assignment.json", dir_clone5);
        if let Ok(data) = read_from_json_file::<Vec<PairingEntry>>(&file_path) {
            let mut pairing_data = pairing_data_clone.lock().unwrap();
            *pairing_data = data;
        }
    });

    let hashtable_data_clone = Arc::clone(&hashtable_data);
    let dir_clone6 = dir.to_string();
    let handle_hashtable = thread::spawn(move || {
        let file_path = format!("{}/hash_assignment.json", dir_clone6);
        if let Ok(data) = read_from_json_file::<Vec<HashTableJson>>(&file_path) {
            let mut hashtable_data = hashtable_data_clone.lock().unwrap();
            *hashtable_data = data;
        }
    });

    let permutation_hash_data_clone = Arc::clone(&permutation_hash_data);
    let dir_clone7 = dir.to_string();
    let handle_permutation = thread::spawn(move || {
        let file_path = format!("{}/permutationhash_assignment.json", dir_clone7);
        if let Ok(data) = read_from_json_file::<Vec<PermutationHashEntry>>(&file_path) {
            let mut permutation_hash_data = permutation_hash_data_clone.lock().unwrap();
            *permutation_hash_data = data;
        }
    });

    handle_validators.join().expect("handle_validators panicked");
    handle_shuffle.join().expect("handle_shuffle panicked");
    handle_pubkey.join().expect("handle_pubkey panicked");
    handle_att.join().expect("handle_att panicked");
    handle_pairing.join().expect("handle_pairing panicked");
    handle_hashtable.join().expect("handle_hashtable panicked");
    handle_permutation.join().expect("handle_permutation panicked");
    let plain_validators_result = Arc::try_unwrap(plain_validators).unwrap().into_inner().unwrap();
    let shuffle_data_result = Arc::try_unwrap(shuffle_data).unwrap().into_inner().unwrap();
    let public_key_bls_list_result = Arc::try_unwrap(public_key_bls_list)
        .unwrap()
        .into_inner()
        .unwrap();
    let attestations_result = Arc::try_unwrap(attestations).unwrap().into_inner().unwrap();
    let pairing_data_result = Arc::try_unwrap(pairing_data).unwrap().into_inner().unwrap();
    let hashtable_data_result = Arc::try_unwrap(hashtable_data).unwrap().into_inner().unwrap();
    let permutation_hash_data_result =
        Arc::try_unwrap(permutation_hash_data).unwrap().into_inner().unwrap();
    let end_time = std::time::Instant::now();
    println!(
        "loaed assignment data, time: {:?}",
        end_time.duration_since(start_time)
    );

    let shuffle_pairing_thread = thread::spawn(move || {
        end2end_shuffle_witnesses(
            solver_shuffle,
            plain_validators_result,
            shuffle_data_result,
            public_key_bls_list_result,
            attestations_result,
            pairing_data_result.clone(),
        );
        end2end_pairing_witness(solver_pairing, pairing_data_result);
    });

    let hash_thread = thread::spawn(move || {
        end2end_hashtable_witnesses(solver_hash, hashtable_data_result);
    });

    let permutation_hash_thread = thread::spawn(move || {
        end2end_permutation_hashbit_witness(solver_permutation_hash, permutation_hash_data_result);
    });

    shuffle_pairing_thread
        .join()
        .expect("ShufflePairing thread panicked");
    hash_thread.join().expect("Hash thread panicked");
    permutation_hash_thread
        .join()
        .expect("Permutation hash thread panicked");
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
