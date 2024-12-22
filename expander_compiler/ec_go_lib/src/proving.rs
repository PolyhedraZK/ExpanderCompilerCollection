use arith::FieldSerde;
use expander_compiler::circuit::layered;
use libc::{c_uchar, c_ulong, malloc};
use std::io::Cursor;
use std::ptr;
use std::slice;

use expander_compiler::{circuit::config, utils::serde::Serde};

use super::*;

fn dump_proof_and_claimed_v<F: arith::Field + arith::FieldSerde>(
    proof: &expander_transcript::Proof,
    claimed_v: &F,
) -> Vec<u8> {
    let mut bytes = Vec::new();

    proof.serialize_into(&mut bytes).unwrap(); // TODO: error propagation
    claimed_v.serialize_into(&mut bytes).unwrap(); // TODO: error propagation

    bytes
}

fn load_proof_and_claimed_v<F: arith::Field + arith::FieldSerde>(
    bytes: &[u8],
) -> Result<(expander_transcript::Proof, F), ()> {
    let mut cursor = Cursor::new(bytes);

    let proof = expander_transcript::Proof::deserialize_from(&mut cursor).map_err(|_| ())?;
    let claimed_v = F::deserialize_from(&mut cursor).map_err(|_| ())?;

    Ok((proof, claimed_v))
}

fn prove_circuit_file_inner<C: config::Config>(circuit_filename: &str, witness: &[u8]) -> Vec<u8> {
    let config = expander_config::Config::<C::DefaultGKRConfig>::new(
        expander_config::GKRScheme::Vanilla,
        expander_config::MPIConfig::new(),
    );
    let mut circuit =
        expander_circuit::Circuit::<C::DefaultGKRConfig>::load_circuit(circuit_filename);
    let witness = layered::witness::Witness::<C>::deserialize_from(witness).unwrap();
    let (simd_input, simd_public_input) = witness.to_simd::<C::DefaultSimdField>();
    circuit.layers[0].input_vals = simd_input;
    circuit.public_input = simd_public_input;
    circuit.evaluate();
    let mut prover = gkr::Prover::new(&config);
    prover.prepare_mem(&circuit);
    let (claimed_v, proof) = prover.prove(&mut circuit);
    dump_proof_and_claimed_v(&proof, &claimed_v)
}

fn verify_circuit_file_inner<C: config::Config>(
    circuit_filename: &str,
    witness: &[u8],
    proof_and_claimed_v: &[u8],
) -> u8 {
    let config = expander_config::Config::<C::DefaultGKRConfig>::new(
        expander_config::GKRScheme::Vanilla,
        expander_config::MPIConfig::new(),
    );
    let mut circuit =
        expander_circuit::Circuit::<C::DefaultGKRConfig>::load_circuit(circuit_filename);
    let witness = layered::witness::Witness::<C>::deserialize_from(witness).unwrap();
    let (simd_input, simd_public_input) = witness.to_simd::<C::DefaultSimdField>();
    circuit.layers[0].input_vals = simd_input;
    circuit.public_input = simd_public_input.clone();
    let (proof, claimed_v) = match load_proof_and_claimed_v(proof_and_claimed_v) {
        Ok((proof, claimed_v)) => (proof, claimed_v),
        Err(_) => {
            return 0;
        }
    };
    let verifier = gkr::Verifier::new(&config);
    verifier.verify(&mut circuit, &simd_public_input, &claimed_v, &proof) as u8
}

#[no_mangle]
pub extern "C" fn prove_circuit_file(
    circuit_filename: ByteArray,
    witness: ByteArray,
    config_id: c_ulong,
) -> ByteArray {
    let circuit_filename = unsafe {
        let slice = slice::from_raw_parts(circuit_filename.data, circuit_filename.length as usize);
        std::str::from_utf8(slice).unwrap()
    };
    let witness = unsafe { slice::from_raw_parts(witness.data, witness.length as usize) };
    let proof = match_config_id_panic!(
        config_id,
        prove_circuit_file_inner,
        (circuit_filename, witness)
    );
    let proof_len = proof.len();
    let proof_ptr = if proof_len > 0 {
        unsafe {
            let ptr = malloc(proof_len) as *mut u8;
            ptr.copy_from(proof.as_ptr(), proof_len);
            ptr
        }
    } else {
        ptr::null_mut()
    };
    ByteArray {
        data: proof_ptr,
        length: proof_len as c_ulong,
    }
}

#[no_mangle]
pub extern "C" fn verify_circuit_file(
    circuit_filename: ByteArray,
    witness: ByteArray,
    proof: ByteArray,
    config_id: c_ulong,
) -> c_uchar {
    let circuit_filename = unsafe {
        let slice = slice::from_raw_parts(circuit_filename.data, circuit_filename.length as usize);
        std::str::from_utf8(slice).unwrap()
    };
    let witness = unsafe { slice::from_raw_parts(witness.data, witness.length as usize) };
    let proof = unsafe { slice::from_raw_parts(proof.data, proof.length as usize) };
    match_config_id_panic!(
        config_id,
        verify_circuit_file_inner,
        (circuit_filename, witness, proof)
    )
}
