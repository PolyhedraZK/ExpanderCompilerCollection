use crate::bls_verifier::{convert_limbs, convert_point, PairingEntry};
use crate::utils::read_from_json_file;
use circuit_std_rs::gnark::emulated::sw_bls12381::g1::*;
use circuit_std_rs::gnark::emulated::sw_bls12381::g2::*;
use circuit_std_rs::gnark::emulated::sw_bls12381::pairing::*;
use expander_compiler::frontend::*;
use expander_compiler::zkcuda::context::{call_kernel, Context, Reshape};
use expander_compiler::zkcuda::kernel::Kernel;
use expander_compiler::zkcuda::kernel::*;
use mersenne31::M31;
use crate::zkcuda_hashtable::{HASHTABLESIZE, SHA256LEN};

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
fn test_zkcuda_bls_verify() {
    let kernel: Kernel<M31Config> = compile_bls_verify().unwrap();
    println!("compile ok");
    let dir = ".";
    let file_path = format!("{}/pairing_assignment.json", dir);

    let pairing_datas: Vec<PairingEntry> = read_from_json_file(&file_path).unwrap();
    let entry = &pairing_datas[0];
    let  pubkey =  [
        convert_limbs(entry.pub_key.x.limbs.clone()),
        convert_limbs(entry.pub_key.y.limbs.clone()),
    ];
    let hm =  [
        convert_point(entry.hm.p.x.clone()),
        convert_point(entry.hm.p.y.clone()),
    ];
    let sig = [
            convert_point(entry.signature.p.x.clone()),
            convert_point(entry.signature.p.y.clone()),
        ];

    let mut p: Vec<M31> = vec![];
    for i in 0..2 {
        for j in 0..48{
            p.push(pubkey[i][j]);
        }
    }
    for i in 0..2 {
        for j in 0..2{
            for k  in 0..48 {
                p.push(hm[i][j][k]);
            }
        }
    }
    for i in 0..2 {
        for j in 0..2{
            for k  in 0..48 {
                p.push(sig[i][j][k]);
            }
        }
    }

    println!("prepare data ok");
    let mut ctx: Context<M31Config> = Context::default();

    let p = ctx.copy_to_device(&vec![p], false);
    println!("copy to device ok");

    // println!("p: {:?}", p.clone().unwrap().shape.unwrap());

    println!("compile ok");

    let mut out = None;
    call_kernel!(ctx, kernel, p, mut out);
    println!("call kernel ok");

    println!("out shape: {:?}", out.clone().unwrap().shape.unwrap());

    let out = out.reshape(&[SHA256LEN*HASHTABLESIZE]);

    println!("out shape: {:?}", out.clone().unwrap().shape.unwrap());

}

