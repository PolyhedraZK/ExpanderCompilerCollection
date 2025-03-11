use expander_compiler::frontend::{M31Config, WitnessSolver};

use crate::attestation::Attestation;
use crate::bls_verifier::{
    end2end_blsverifier_witness, generate_blsverifier_witnesses, BLSVERIFIERCircuit, PairingEntry,
};
use crate::hashtable::{
    self, end2end_hashtable_witnesses, generate_hash_witnesses, HASHTABLECircuit, HashTableJson,
};
use crate::permutation::{
    self, end2end_permutation_hashbit_witness, end2end_permutation_query_witness,
    generate_permutation_hashes_witnesses, PermutationHashEntry,
    PermutationIndicesValidatorHashBitCircuit, PermutationQueryCircuit, PermutationQueryEntry,
};
use crate::shuffle::{
    self, end2end_shuffle_witnesses, generate_shuffle_witnesses, ShuffleCircuit, ShuffleJson,
};
use crate::utils::{get_solver, read_from_json_file};
use crate::validator::{
    self, ConvertValidatorListToMerkleTreeCircuit, ValidatorPlain, ValidatorSubTreeJson,
};
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
    let blsverifier_thread = thread::spawn(move || {
        generate_blsverifier_witnesses(&dir_str);
    });

    let dir_str = dir.to_string();
    let permutation_hash_thread = thread::spawn(move || {
        generate_permutation_hashes_witnesses(&dir_str);
    });

    shuffle_thread.join().expect("Shuffle thread panicked");
    hash_thread.join().expect("Hash thread panicked");
    blsverifier_thread.join().expect("Pairing thread panicked");
    permutation_hash_thread
        .join()
        .expect("Permutation hash thread panicked");
    let end_time = std::time::Instant::now();
    log::debug!(
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
    solver_permutation_query: WitnessSolver<M31Config>,
    solver_permutation_hash: WitnessSolver<M31Config>,
    solver_validator_subtree: WitnessSolver<M31Config>,
) {
    log::debug!("loading assignment data...");
    let start_time = std::time::Instant::now();

    let plain_validators = Arc::new(Mutex::new(Vec::<ValidatorPlain>::new()));
    let shuffle_data = Arc::new(Mutex::new(Vec::<ShuffleJson>::new()));
    let public_key_bls_list = Arc::new(Mutex::new(Vec::<Vec<String>>::new()));
    let attestations = Arc::new(Mutex::new(Vec::<Attestation>::new()));
    let pairing_data = Arc::new(Mutex::new(Vec::<PairingEntry>::new()));
    let hashtable_data = Arc::new(Mutex::new(Vec::<HashTableJson>::new()));
    let permutation_query_data = Arc::new(Mutex::new(Vec::<PermutationQueryEntry>::new()));
    let permutation_hash_data = Arc::new(Mutex::new(Vec::<PermutationHashEntry>::new()));
    let validator_subtree_data = Arc::new(Mutex::new(Vec::<ValidatorSubTreeJson>::new()));

    let plain_validators_clone = Arc::clone(&plain_validators);
    let dir_clone_validator = dir.to_string();
    let handle_validators = thread::spawn(move || {
        let file_path = format!("{}/validatorList.json", dir_clone_validator);
        if let Ok(data) = read_from_json_file::<Vec<ValidatorPlain>>(&file_path) {
            let mut validators = plain_validators_clone.lock().unwrap();
            *validators = data;
        }
    });

    let shuffle_data_clone = Arc::clone(&shuffle_data);
    let dir_shuffle = dir.to_string();
    let handle_shuffle = thread::spawn(move || {
        let file_path = format!("{}/shuffle_assignment.json", dir_shuffle);
        if let Ok(data) = read_from_json_file::<Vec<ShuffleJson>>(&file_path) {
            let mut shuffle_data = shuffle_data_clone.lock().unwrap();
            *shuffle_data = data;
        }
    });

    let public_key_bls_list_clone = Arc::clone(&public_key_bls_list);
    let dir_pubkey = dir.to_string();
    let handle_pubkey = thread::spawn(move || {
        let file_path = format!("{}/pubkeyBLSList.json", dir_pubkey);
        if let Ok(data) = read_from_json_file::<Vec<Vec<String>>>(&file_path) {
            let mut public_key_bls_list = public_key_bls_list_clone.lock().unwrap();
            *public_key_bls_list = data;
        }
    });

    let attestations_clone = Arc::clone(&attestations);
    let dir_attestations = dir.to_string();
    let handle_att = thread::spawn(move || {
        let file_path = format!("{}/slotAttestationsFolded.json", dir_attestations);
        if let Ok(data) = read_from_json_file::<Vec<Attestation>>(&file_path) {
            let mut attestations = attestations_clone.lock().unwrap();
            *attestations = data;
        }
    });

    let pairing_data_clone = Arc::clone(&pairing_data);
    let dir_pairing = dir.to_string();
    let handle_pairing = thread::spawn(move || {
        let file_path = format!("{}/pairing_assignment.json", dir_pairing);
        if let Ok(data) = read_from_json_file::<Vec<PairingEntry>>(&file_path) {
            let mut pairing_data = pairing_data_clone.lock().unwrap();
            *pairing_data = data;
        }
    });

    let hashtable_data_clone = Arc::clone(&hashtable_data);
    let dir_hashtable = dir.to_string();
    let handle_hashtable = thread::spawn(move || {
        let file_path = format!("{}/hash_assignment.json", dir_hashtable);
        if let Ok(data) = read_from_json_file::<Vec<HashTableJson>>(&file_path) {
            let mut hashtable_data = hashtable_data_clone.lock().unwrap();
            *hashtable_data = data;
        }
    });

    let permutation_query_data_clone = Arc::clone(&permutation_query_data);
    let dir_permutation_query = dir.to_string();
    let handle_permutation_query = thread::spawn(move || {
        let file_path = format!("{}/permutation_assignment.json", dir_permutation_query);
        if let Ok(data) = read_from_json_file::<Vec<PermutationQueryEntry>>(&file_path) {
            let mut permutation_query_data = permutation_query_data_clone.lock().unwrap();
            *permutation_query_data = data;
        }
    });

    let permutation_hash_data_clone = Arc::clone(&permutation_hash_data);
    let dir_permutation_hash = dir.to_string();
    let handle_permutation_hash = thread::spawn(move || {
        let file_path = format!("{}/permutationhash_assignment.json", dir_permutation_hash);
        if let Ok(data) = read_from_json_file::<Vec<PermutationHashEntry>>(&file_path) {
            let mut permutation_hash_data = permutation_hash_data_clone.lock().unwrap();
            *permutation_hash_data = data;
        }
    });

    let validator_subtree_data_clone = Arc::clone(&validator_subtree_data);
    let dir_validator_subtree = dir.to_string();
    let handle_validator_subtree = thread::spawn(move || {
        let file_path = format!("{}/validatorsubtree_assignment.json", dir_validator_subtree);
        if let Ok(data) = read_from_json_file::<Vec<ValidatorSubTreeJson>>(&file_path) {
            let mut validator_subtree_data = validator_subtree_data_clone.lock().unwrap();
            *validator_subtree_data = data;
        }
    });

    handle_validators
        .join()
        .expect("handle_validators panicked");
    handle_shuffle.join().expect("handle_shuffle panicked");
    handle_pubkey.join().expect("handle_pubkey panicked");
    handle_att.join().expect("handle_att panicked");
    handle_pairing.join().expect("handle_pairing panicked");
    handle_hashtable.join().expect("handle_hashtable panicked");
    handle_permutation_query
        .join()
        .expect("handle_permutation_query panicked");
    handle_permutation_hash
        .join()
        .expect("handle_permutation_hash panicked");
    handle_validator_subtree
        .join()
        .expect("handle_validator_subtree panicked");
    let plain_validators_result = Arc::try_unwrap(plain_validators)
        .unwrap()
        .into_inner()
        .unwrap();
    let shuffle_data_result = Arc::try_unwrap(shuffle_data).unwrap().into_inner().unwrap();
    let public_key_bls_list_result = Arc::try_unwrap(public_key_bls_list)
        .unwrap()
        .into_inner()
        .unwrap();
    let attestations_result = Arc::try_unwrap(attestations).unwrap().into_inner().unwrap();
    let pairing_data_result = Arc::try_unwrap(pairing_data).unwrap().into_inner().unwrap();
    let hashtable_data_result = Arc::try_unwrap(hashtable_data)
        .unwrap()
        .into_inner()
        .unwrap();
    let permutation_query_data_result = Arc::try_unwrap(permutation_query_data)
        .unwrap()
        .into_inner()
        .unwrap();
    let permutation_hash_data_result = Arc::try_unwrap(permutation_hash_data)
        .unwrap()
        .into_inner()
        .unwrap();
    let validator_subtree_data_result = Arc::try_unwrap(validator_subtree_data)
        .unwrap()
        .into_inner()
        .unwrap();
    let end_time = std::time::Instant::now();
    log::debug!(
        "loaed assignment data, time: {:?}",
        end_time.duration_since(start_time)
    );
    let shuffle_thread = thread::spawn(move || {
        end2end_shuffle_witnesses(
            solver_shuffle,
            plain_validators_result,
            shuffle_data_result,
            public_key_bls_list_result,
        );
    });

    let blsverifier_thread = thread::spawn(move || {
        end2end_blsverifier_witness(solver_pairing, pairing_data_result, attestations_result);
    });

    let hash_thread = thread::spawn(move || {
        end2end_hashtable_witnesses(solver_hash, hashtable_data_result);
    });

    let permutation_query_thread = thread::spawn(move || {
        end2end_permutation_query_witness(solver_permutation_query, permutation_query_data_result);
    });

    let permutation_hash_thread = thread::spawn(move || {
        end2end_permutation_hashbit_witness(solver_permutation_hash, permutation_hash_data_result);
    });

    let validator_subtree_thread = thread::spawn(move || {
        validator::end2end_validator_subtree_witnesses(
            solver_validator_subtree,
            validator_subtree_data_result,
        );
    });

    shuffle_thread
        .join()
        .expect("ShufflePairing thread panicked");
    blsverifier_thread.join().expect("Pairing thread panicked");
    hash_thread.join().expect("Hash thread panicked");
    permutation_query_thread
        .join()
        .expect("Permutation query thread panicked");
    permutation_hash_thread
        .join()
        .expect("Permutation hash thread panicked");
    validator_subtree_thread
        .join()
        .expect("Validator subtree thread panicked");
    let end_time = std::time::Instant::now();
    log::debug!(
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
    log::debug!("loading assignment data...");
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
    log::debug!(
        "loaed assignment data, time: {:?}",
        end_time.duration_since(start_time)
    );
    let shuffle_thread = thread::spawn(move || {
        end2end_shuffle_witnesses(
            solver_shuffle,
            plain_validators,
            shuffle_data,
            public_key_bls_list,
        );
    });

    let blsverifier_thread = thread::spawn(move || {
        end2end_blsverifier_witness(solver_pairing, pairing_data, attestations);
    });
    shuffle_thread
        .join()
        .expect("ShufflePairing thread panicked");
    blsverifier_thread.join().expect("Pairing thread panicked");
    let end_time = std::time::Instant::now();
    log::debug!(
        "generate end2end start witness, time: {:?}",
        end_time.duration_since(start_time)
    );
}

pub fn end2end_witness_streamline(stage: &str) {
    if stage == "end" {
        log::debug!("end stage");
        //get the solver for shuffle
        let circuit_name = &format!("shuffle_{}", shuffle::VALIDATOR_CHUNK_SIZE);
        let circuit = ShuffleCircuit::default();
        let witnesses_dir = &format!("./witnesses/{}", circuit_name);
        let solver_shuffle = get_solver(witnesses_dir, circuit_name, circuit);

        //get the solver for hash
        let circuit_name = &format!("hashtable{}", hashtable::HASHTABLESIZE);
        let circuit = HASHTABLECircuit::default();
        let witnesses_dir = &format!("./witnesses/{}", circuit_name);
        let solver_hash = get_solver(witnesses_dir, circuit_name, circuit);

        //get the solver for bls verifier
        let circuit_name = "blsverifier";
        let circuit = BLSVERIFIERCircuit::default();
        let witnesses_dir = &format!("./witnesses/{}", circuit_name);
        let solver_blsverifier = get_solver(witnesses_dir, circuit_name, circuit);

        //get the solver for permutation query
        let circuit_name = "permutationquery";
        let circuit = PermutationQueryCircuit::default();
        let witnesses_dir = &format!("./witnesses/{}", circuit_name);
        let solver_permutation_query = get_solver(witnesses_dir, circuit_name, circuit);

        //get the solver for permutation hash
        let circuit_name = &format!("permutationhashbit_{}", permutation::VALIDATOR_COUNT);
        let circuit = PermutationIndicesValidatorHashBitCircuit::default();
        let witnesses_dir = &format!("./witnesses/{}", circuit_name);
        let solver_permutation_hash = get_solver(witnesses_dir, circuit_name, circuit);

        //get the solver for validator subtree
        let circuit_name = &format!("validatorsubtree{}", validator::SUBTREE_SIZE);
        let circuit = ConvertValidatorListToMerkleTreeCircuit::default();
        let witnesses_dir = &format!("./witnesses/{}", circuit_name);
        let solver_validator_subtree = get_solver(witnesses_dir, circuit_name, circuit);

        let dir = "./efc/data";
        end2end_witness_streamline_end(
            dir,
            solver_shuffle,
            solver_hash,
            solver_blsverifier,
            solver_permutation_query,
            solver_permutation_hash,
            solver_validator_subtree,
        );
    } else if stage == "start" {
        log::debug!("start stage");
        //get the solver for shuffle
        let circuit_name = &format!("shuffle_{}", shuffle::VALIDATOR_CHUNK_SIZE);
        let circuit = ShuffleCircuit::default();
        let witnesses_dir = &format!("./witnesses/{}", circuit_name);
        let solver_shuffle = get_solver(witnesses_dir, circuit_name, circuit);

        //get the solver for bls verifier
        let circuit_name = "blsverifier";
        let circuit = BLSVERIFIERCircuit::default();
        let witnesses_dir = &format!("./witnesses/{}", circuit_name);
        let solver_blsverifier = get_solver(witnesses_dir, circuit_name, circuit);

        let dir = "./efc/data";
        end2end_witness_streamline_start(dir, solver_shuffle, solver_blsverifier);
    }
}
