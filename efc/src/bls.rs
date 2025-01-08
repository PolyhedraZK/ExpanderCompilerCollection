use std::str::FromStr;
use circuit_std_rs::gnark::emulated::sw_bls12381::g1::{G1Affine, G1};
use circuit_std_rs::utils::{simple_lookup2, simple_select};
use expander_compiler::frontend::*;
use num_bigint::BigInt;
use circuit_std_rs::big_int::{big_is_zero, big_less_than, bigint_to_m31_array, to_binary};

const K: usize = 48;
const N: usize = 8;
const M_COMPRESSED_SMALLEST: u8 = 0b100 << 5;
const M_COMPRESSED_LARGEST: u8 = 0b101 << 5;
const M_COMPRESSED_INFINITY: u8 = 0b110 << 5;

pub fn convert_to_public_key_bls<C: Config, B: RootAPI<C>>(api: &mut B, pubkey: Vec<Variable>) -> (G1Affine, Variable) {
    let mut empty_flag = api.constant(1); //if pubkey is empty (all -1), emptyFlag = 1
    for _ in 0..pubkey.len() {
        let tmp = api.add(pubkey[0].clone(), 1);
        let flag = api.is_zero(tmp);
        empty_flag = api.and(empty_flag, flag); //if pubkey is not empty, emptyFlag = 0
    }
    let mut inputs = pubkey.clone();
    inputs.insert(0, empty_flag);
    //use a hint to get the bls publickey
    let outputs = api.new_hint("getPublicKeyBLSHint", &inputs, pubkey.len()*2);
    let public_key_bls = G1Affine::from_vars(outputs[0..K].to_vec(), outputs[K..2*K].to_vec());
    let logup_var = assert_public_key_and_bls(api, pubkey, &public_key_bls, empty_flag);

    (public_key_bls, logup_var)
}

pub fn check_pubkey_key_bls<C: Config, B: RootAPI<C>>(api: &mut B, pubkey: Vec<Variable>, public_key_bls: &G1Affine) -> Variable {
    let empty_flag = api.constant(0);
    let logup_var = assert_public_key_and_bls(api, pubkey, public_key_bls, empty_flag);
    logup_var
}

pub fn assert_public_key_and_bls<C: Config, B: RootAPI<C>>(api: &mut B, pubkey: Vec<Variable>, public_key_bls: &G1Affine, empty_flag: Variable) -> Variable{
    let x_is_zero = big_is_zero(api, K, &public_key_bls.x.limbs);
    let y_is_zero = big_is_zero(api, K, &public_key_bls.y.limbs);
    let is_infinity = api.mul(x_is_zero, y_is_zero);

    let half_fp = BigInt::from_str("4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559787").unwrap() / 2;
    let half_fp_var = bigint_to_m31_array(api, half_fp, N, K);
    let lex_large = big_less_than(api, N, K, &half_fp_var, &public_key_bls.y.limbs);
    //
    // 0 0: mCompressedSmallest
    // 1 0: mCompressedInfinity
    // 0 1: mCompressedLargest
    // 1 1: 0
    let m_compressed_infinity_var = api.constant(M_COMPRESSED_INFINITY as u32);
    let m_compressed_smallest_var = api.constant(M_COMPRESSED_SMALLEST as u32);
    let m_compressed_largest_var = api.constant(M_COMPRESSED_LARGEST as u32);
    let zero_var = api.constant(0);
    let mask = simple_lookup2(api, is_infinity, lex_large, m_compressed_smallest_var, m_compressed_infinity_var, m_compressed_largest_var, zero_var);

    let mut out_tmp = pubkey.clone();
    out_tmp[0] = api.sub(out_tmp[0], mask);
    // logup::range_proof_single_chunk(api, out_tmp[0], 5); //return the value, and logup it to the range of 5 after this function call
    compare_two_scalars(api, &public_key_bls.x.limbs, N, &out_tmp, 8, empty_flag);
    out_tmp[0]
}
pub fn compare_two_scalars<C: Config, B: RootAPI<C>>(api: &mut B, scalar1: &Vec<Variable>, n_bit1: usize, scalar2: &Vec<Variable>, n_bit2: usize, empty_flag: Variable) {
    //first, we need to check the length of the field, i.e., m31 = 31 bits, bn254 = 254 bits
    //we can compose scalar1 and scalar2 to bigInts, but they should have a length less than the field length
    let available_bits = 31 - 1; 
    //Now, find a best way to compose scalar1 and scalar2 to bigInts
    let gcd_n_bit1_n_bit2 = lcm_int(n_bit1, n_bit2);
    let max_bits = scalar1.len() * n_bit1;
    let expansion = (max_bits / gcd_n_bit1_n_bit2) / ((max_bits + available_bits - 1) / available_bits);
    if expansion == 0 { //means the lcm is still too large, let's compare two scalars bit-by-bit
        let scalar1_bits = decompose_vars(api, scalar1, n_bit1);
        let scalar2_bits = decompose_vars(api, scalar2, n_bit2);
        assert_eq!(scalar1_bits.len(), scalar2_bits.len());
        for i in 0..scalar1_bits.len() {
            api.assert_is_equal(scalar1_bits[i], scalar2_bits[i]);
        }
    } else {
        let target_bits = expansion * gcd_n_bit1_n_bit2; //we will compose the scalar1 and scalar2 to bigInts with targetBits
        let chunk1_len = target_bits / n_bit1;
        let mut scalar1_big = vec![api.constant(0); scalar1.len()/chunk1_len];
        for i in 0..scalar1_big.len() {
            scalar1_big[i] = compose_var_little(api, &scalar1[i*chunk1_len..(i+1)*chunk1_len], n_bit1);
        }
        let chunk2_len = target_bits / n_bit2;
        let mut scalar2_big = vec![api.constant(0); scalar2.len()/chunk2_len];
        for i in 0..scalar2_big.len() {
            scalar2_big[i] = compose_var_big(api, &scalar2[i*chunk2_len..(i+1)*chunk2_len], n_bit2);
        }

        //the length of scalar1Big and scalar2Big should be the same
        assert_eq!(scalar1_big.len(), scalar2_big.len());
        //scalar1Big and scalar2Big should be the same
        let scalar_big_len = scalar1_big.len();
        for i in 0..scalar_big_len {
            scalar1_big[i] = simple_select(api, empty_flag, scalar2_big[scalar_big_len-i-1], scalar1_big[i]);
            api.assert_is_equal(scalar1_big[i], scalar2_big[scalar_big_len-i-1]);
        }
    }
}

fn gcd(a: usize, b: usize) -> usize {
    let mut a = a;
    let mut b = b;
    while b != 0 {
        let tmp = a;
        a = b;
        b = tmp % b;
    }
    a
}
fn lcm_int(a: usize, b: usize) -> usize {
    (a * b) / gcd(a, b)
}

pub fn compose_var_little<C: Config, B: RootAPI<C>>(api: &mut B, scalar: &[Variable], n_bit: usize) -> Variable {
    if scalar.len() == 1 {
        return scalar[0].clone();
    }
    //compose the scalar to a bigInt
    let scalar_len = scalar.len();
    let mut scalar_big = scalar[scalar_len-1].clone();
    for i in 1..scalar_len {
        scalar_big = api.mul(scalar_big, 1<<n_bit);
        scalar_big = api.add(scalar_big, scalar[scalar_len-i-1].clone());
    }
    scalar_big
}
pub fn compose_var_big<C: Config, B: RootAPI<C>>(api: &mut B, scalar: &[Variable], n_bit: usize) -> Variable {
    if scalar.len() == 1 {
        return scalar[0].clone();
    }
    //compose the scalar to a bigInt
    let scalar_len = scalar.len();
    let mut scalar_big = scalar[0].clone();
    for i in 1..scalar_len {
        scalar_big = api.mul(scalar_big, 1<<n_bit);
        scalar_big = api.add(scalar_big, scalar[i].clone());
    }
    scalar_big
}
pub fn decompose_vars<C: Config, B: RootAPI<C>>(api: &mut B, scalar: &Vec<Variable>, n_bit: usize) -> Vec<Variable> {
    //decompose the scalar to a []big.Int
    let mut scalar_array = vec![];
    for i in 0..scalar.len() {
        scalar_array.extend(to_binary(api, scalar[i].clone(), n_bit));
    }
    scalar_array
}