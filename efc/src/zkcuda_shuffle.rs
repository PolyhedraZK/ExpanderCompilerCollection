use ark_std::iterable::Iterable;
use crate::bls_verifier::{convert_limbs, convert_point, PairingCircuit, PairingEntry};
use crate::utils::read_from_json_file;
use circuit_std_rs::gnark::emulated::sw_bls12381::g1::*;
use circuit_std_rs::gnark::emulated::sw_bls12381::g2::*;
use circuit_std_rs::gnark::emulated::sw_bls12381::pairing::*;
use circuit_std_rs::sha256::m31::sha256_37bytes;
use circuit_std_rs::sha256::m31_utils::{big_array_add, to_binary_hint};
use expander_compiler::frontend::M31Config;
use expander_compiler::frontend::*;
use expander_compiler::zkcuda::context::{call_kernel, Context, Reshape};
use expander_compiler::zkcuda::kernel::Kernel;
use expander_compiler::zkcuda::kernel::*;
use expander_compiler::zkcuda::proving_system::ExpanderGKRProvingSystem;
use mersenne31::M31;
use crate::shuffle::{POSEIDON_HASH_LENGTH, SHUFFLE_ROUND, VALIDATOR_CHUNK_SIZE};
use crate::zkcuda_hashtable::{HASHTABLESIZE, SHA256LEN};

fn sub_vector<T>(vec: &Vec<T>, start: usize, length: usize) -> (Vec<T>, usize)
    where
        T: Clone,
{
    if start >= vec.len() {
        return (Vec::new(), start);
    }
    let end = std::cmp::min(start + length, vec.len());
    let sub_vec = vec[start..end].to_vec();
    let next_pos = end;
    (sub_vec, next_pos)
}

fn shuffle_inner<C: Config>(api: &mut API<C>, p: &Vec<Variable>) -> Vec<Variable> {
    println!("len p: {}", p.len());

    let (start_index,  pos) = sub_vector(p, 0, 1);
    let (chunk_length,  pos) = sub_vector(p, pos, 1);
    let (shuffle_indices,  pos) = sub_vector(p, pos, VALIDATOR_CHUNK_SIZE);
    let (committee_indices,  pos) = sub_vector(p, pos, VALIDATOR_CHUNK_SIZE);
    let (pivots,  pos) = sub_vector(p, pos, SHUFFLE_ROUND);
    let (index_count,  pos) = sub_vector(p, pos, 1);
    let (position_results,  pos) = sub_vector(p, pos, SHUFFLE_ROUND * VALIDATOR_CHUNK_SIZE);
    let (position_bit_results,  pos) = sub_vector(p, pos,  SHUFFLE_ROUND * VALIDATOR_CHUNK_SIZE);
    let (flip_results, pos) = sub_vector(p, pos, SHUFFLE_ROUND * VALIDATOR_CHUNK_SIZE);

    let (slot, pos) = sub_vector(p, pos, 8);
    let (committee_index, pos) = sub_vector(p, pos, 8);
    let (beacon_beacon_block_root, pos) = sub_vector(p, pos, 32);
    let (source_epoch, pos) = sub_vector(p, pos, 8);
    let (target_epoch, pos) = sub_vector(p, pos, 8);
    let (source_root, pos) = sub_vector(p, pos, 32);
    let (target_root, pos) = sub_vector(p, pos, 32);

    let (attestation_hm, pos) = sub_vector(p, pos, 48*2*2);

    let (attestation_sig_bytes, pos) = sub_vector(p, pos, 96);
    let (attestation_sig_g2, pos) = sub_vector(p, pos, 48*2*2);
    let (aggregation_bits, pos) = sub_vector(p, pos, VALIDATOR_CHUNK_SIZE);
    let (validator_hashes, pos) = sub_vector(p, pos, POSEIDON_HASH_LENGTH*VALIDATOR_CHUNK_SIZE);
    let (aggregated_pubkey, pos) = sub_vector(p, pos, 48*2);
    let (attestation_balance, pos) = sub_vector(p, pos, 8);
    let (pubkeys_bls, pos) = sub_vector(p, pos, 48*2*VALIDATOR_CHUNK_SIZE);
    let (pubkey, pos) = sub_vector(p, pos, 48*VALIDATOR_CHUNK_SIZE);
    let (withdrawal_credentials, pos) = sub_vector(p, pos, 32*VALIDATOR_CHUNK_SIZE);
    let (effective_balance, pos) = sub_vector(p, pos, 48*VALIDATOR_CHUNK_SIZE);
    let (slashed, pos) = sub_vector(p, pos, VALIDATOR_CHUNK_SIZE);
    let (activation_eligibility_epoch, pos) = sub_vector(p, pos, 8*VALIDATOR_CHUNK_SIZE);
    let (activation_epoch, pos) = sub_vector(p, pos, 8*VALIDATOR_CHUNK_SIZE);
    let (exit_epoch, pos) = sub_vector(p, pos, 8*VALIDATOR_CHUNK_SIZE);
    let (withdrawable_epoch, pos) = sub_vector(p, pos, 8*VALIDATOR_CHUNK_SIZE);

    //println!("pos: {}", pos);

    api.assert_is_equal(p[0], 1);

    return vec![api.constant(1)];
}


#[kernel]
fn compute_shuffle<C: Config>(
    api: &mut API<C>,
    input: &[InputVariable; 203674],
    output: &mut OutputVariable,
) {
    let outc = api.memorized_simple_call(shuffle_inner, input);
    *output = outc[0]
}
// 203674
#[test]
fn test_zkcuda_shuffle() {
    let kernel: Kernel<M31Config> = compile_compute_shuffle().unwrap();
    println!("compile ok");
}