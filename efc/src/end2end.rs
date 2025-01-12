
use crate::bls_verifier::generate_pairing_witnesses;
use crate::hashtable::generate_hash_witnesses;
use crate::shuffle::generate_shuffle_witnesses;
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

    shuffle_thread.join().expect("Shuffle thread panicked");
    hash_thread.join().expect("Hash thread panicked");
    pairing_thread.join().expect("Pairing thread panicked");
    let end_time = std::time::Instant::now();
    println!("generate end2end witness, time: {:?}", end_time.duration_since(start_time));
}

#[test]
fn test_end2end_witness() {
    let dir = "";
    end2end_witness(dir);
}