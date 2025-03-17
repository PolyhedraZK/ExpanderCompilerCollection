use crate::bls_verifier::{PairingCircuit, PairingEntry};
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

fn bls_verify_inner<C: Config>(api: &mut API<C>, p: &Vec<Variable>) -> Vec<Variable> {
    let pubkey = &p[..48 * 2];
    let hm = &p[48 * 2..48 * 2 + 48 * 2 * 2];
    let sig = &p[48 * 2 + 48 * 2 * 2..];
    let mut pairing = Pairing::new(api);
    let one_g1 = G1Affine::one(api);
    let pubkey_g1 = G1Affine::from_vars(pubkey[0..48].to_vec(), pubkey[48..].to_vec());
    let hm_g2 = G2AffP::from_vars(
        hm[0..48].to_vec(),
        hm[0..48].to_vec(),
        hm[96..144].to_vec(),
        hm[144..192].to_vec(),
    );
    let sig_g2 = G2AffP::from_vars(
        sig[0..48].to_vec(),
        sig[48..96].to_vec(),
        sig[96..144].to_vec(),
        sig[144..192].to_vec(),
    );
    let mut g2 = G2::new(api);
    let neg_sig_g2 = g2.neg(api, &sig_g2);

    let p_array = vec![one_g1, pubkey_g1];
    let mut q_array = [
        G2Affine {
            p: neg_sig_g2,
            lines: LineEvaluations::default(),
        },
        G2Affine {
            p: hm_g2,
            lines: LineEvaluations::default(),
        },
    ];
    pairing.pairing_check(api, &p_array, &mut q_array).unwrap();

    pairing.ext12.ext6.ext2.curve_f.check_mul(api);
    pairing.ext12.ext6.ext2.curve_f.table.final_check(api);
    pairing.ext12.ext6.ext2.curve_f.table.final_check(api);
    pairing.ext12.ext6.ext2.curve_f.table.final_check(api);

    return vec![api.constant(1)];
}

#[kernel]
fn bls_verify<C: Config>(
    api: &mut API<C>,
    input: &[InputVariable; 48 * 2 + 48 * 2 * 2 + 48 * 2 * 2],
    output: &mut OutputVariable,
) {
    let outc = api.memorized_simple_call(bls_verify_inner, input);
    *output = outc[0]
}

#[test]
fn test_zkcuda_hashtable() {
    let kernel: Kernel<M31Config> = compile_bls_verify().unwrap();
    println!("compile ok");
    let dir = ".";
    let file_path = format!("{}/pairing_assignment.json", dir);

    let pairing_data: Vec<PairingEntry> = read_from_json_file(&file_path).unwrap();
    let assignment = PairingCircuit::from_entry(pairing_data[0].clone());


}

