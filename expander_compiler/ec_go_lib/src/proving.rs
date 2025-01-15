use expander_compiler::circuit::layered;
use libc::{c_uchar, c_ulong, malloc};
use std::ptr;
use std::slice;

use expander_compiler::{circuit::config, utils::serde::Serde};

use super::*;

fn prove_circuit_file_inner<C: config::Config>(
    circuit_filename: &str,
    witness: &[u8],
) -> Result<Vec<u8>, String> {
    let config = expander_config::Config::<C::DefaultGKRConfig>::new(
        expander_config::GKRScheme::Vanilla,
        mpi_config::MPIConfig::new(),
    );
    let mut circuit =
        expander_circuit::Circuit::<C::DefaultGKRFieldConfig>::load_circuit(circuit_filename);
    let witness =
        layered::witness::Witness::<C>::deserialize_from(witness).map_err(|e| e.to_string())?;
    let (simd_input, simd_public_input) = witness.to_simd::<C::DefaultSimdField>();
    circuit.layers[0].input_vals = simd_input;
    circuit.public_input = simd_public_input;
    circuit.evaluate();
    let (claimed_v, proof) = gkr::executor::prove(&mut circuit, &config);
    gkr::executor::dump_proof_and_claimed_v(&proof, &claimed_v).map_err(|e| e.to_string())
}

fn verify_circuit_file_inner<C: config::Config>(
    circuit_filename: &str,
    witness: &[u8],
    proof_and_claimed_v: &[u8],
) -> Result<u8, String> {
    let config = expander_config::Config::<C::DefaultGKRConfig>::new(
        expander_config::GKRScheme::Vanilla,
        mpi_config::MPIConfig::new(),
    );
    let mut circuit =
        expander_circuit::Circuit::<C::DefaultGKRFieldConfig>::load_circuit(circuit_filename);
    let witness =
        layered::witness::Witness::<C>::deserialize_from(witness).map_err(|e| e.to_string())?;
    let (simd_input, simd_public_input) = witness.to_simd::<C::DefaultSimdField>();
    circuit.layers[0].input_vals = simd_input;
    circuit.public_input = simd_public_input.clone();
    let (proof, claimed_v) = match gkr::executor::load_proof_and_claimed_v(proof_and_claimed_v) {
        Ok((proof, claimed_v)) => (proof, claimed_v),
        Err(_) => {
            return Ok(0);
        }
    };
    Ok(gkr::executor::verify(&mut circuit, &config, &proof, &claimed_v) as u8)
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
    let proof = match_config_id!(
        config_id,
        prove_circuit_file_inner,
        (circuit_filename, witness)
    )
    .unwrap(); // TODO: handle error
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
    match_config_id!(
        config_id,
        verify_circuit_file_inner,
        (circuit_filename, witness, proof)
    )
    .unwrap() // TODO: handle error
}
