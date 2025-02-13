use expander_compiler::frontend::*;
use expander_compiler::zkcuda::proving_system::DummyProvingSystem;
use expander_compiler::zkcuda::{context::*, kernel::*};
use num_bigint::BigInt;
use num_traits::cast::ToPrimitive;
use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
const SHA256LEN: usize = 32;
const CHUNK: usize = 64;
const INIT0: u32 = 0x6A09E667;
const INIT1: u32 = 0xBB67AE85;
const INIT2: u32 = 0x3C6EF372;
const INIT3: u32 = 0xA54FF53A;
const INIT4: u32 = 0x510E527F;
const INIT5: u32 = 0x9B05688C;
const INIT6: u32 = 0x1F83D9AB;
const INIT7: u32 = 0x5BE0CD19;
//for m31 field (2^31-1), split each one to 2 30-bit element
const INIT00: u32 = INIT0 & 0x3FFFFFFF;
const INIT01: u32 = INIT0 >> 30;
const INIT10: u32 = INIT1 & 0x3FFFFFFF;
const INIT11: u32 = INIT1 >> 30;
const INIT20: u32 = INIT2 & 0x3FFFFFFF;
const INIT21: u32 = INIT2 >> 30;
const INIT30: u32 = INIT3 & 0x3FFFFFFF;
const INIT31: u32 = INIT3 >> 30;
const INIT40: u32 = INIT4 & 0x3FFFFFFF;
const INIT41: u32 = INIT4 >> 30;
const INIT50: u32 = INIT5 & 0x3FFFFFFF;
const INIT51: u32 = INIT5 >> 30;
const INIT60: u32 = INIT6 & 0x3FFFFFFF;
const INIT61: u32 = INIT6 >> 30;
const INIT70: u32 = INIT7 & 0x3FFFFFFF;
const INIT71: u32 = INIT7 >> 30;
const _K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];
struct MyDigest {
    h: [[Variable; 2]; 8],
    nx: usize,
    len: u64,
    kbits: [[Variable; 32]; 64],
}

pub fn bytes_to_bits<C: Config>(api: &mut API<C>, vals: &[Variable]) -> Vec<Variable> {
    let mut ret = to_binary(api, vals[0], 8);
    for val in vals.iter().skip(1) {
        ret = to_binary(api, *val, 8)
            .into_iter()
            .chain(ret.into_iter())
            .collect();
    }
    ret
}
pub fn right_shift<C: Config>(
    api: &mut API<C>,
    bits: &[Variable],
    shift: usize,
) -> Vec<Variable> {
    if bits.len() != 32 {
        panic!("RightShift: len(bits) != 32");
    }
    let mut shifted_bits = bits[shift..].to_vec();
    for _ in 0..shift {
        shifted_bits.push(api.constant(0));
    }
    shifted_bits
}
pub fn rotate_right(bits: &[Variable], shift: usize) -> Vec<Variable> {
    if bits.len() != 32 {
        panic!("RotateRight: len(bits) != 32");
    }
    let mut rotated_bits = bits[shift..].to_vec();
    rotated_bits.extend_from_slice(&bits[..shift]);
    rotated_bits
}
pub fn sigma0<C: Config>(api: &mut API<C>, bits: &[Variable]) -> Vec<Variable> {
    if bits.len() != 32 {
        panic!("Sigma0: len(bits) != 32");
    }
    let bits1 = bits.to_vec();
    let bits2 = bits.to_vec();
    let bits3 = bits.to_vec();
    let v1 = rotate_right(&bits1, 7);
    let v2 = rotate_right(&bits2, 18);
    let v3 = right_shift(api, &bits3, 3);
    let mut ret = vec![];
    for i in 0..32 {
        let tmp = api.xor(v1[i], v2[i]);
        ret.push(api.xor(tmp, v3[i]));
    }
    ret
}
pub fn sigma1<C: Config>(api: &mut API<C>, bits: &[Variable]) -> Vec<Variable> {
    if bits.len() != 32 {
        panic!("Sigma1: len(bits) != 32");
    }
    let bits1 = bits.to_vec();
    let bits2 = bits.to_vec();
    let bits3 = bits.to_vec();
    let v1 = rotate_right(&bits1, 17);
    let v2 = rotate_right(&bits2, 19);
    let v3 = right_shift(api, &bits3, 10);
    let mut ret = vec![];
    for i in 0..32 {
        let tmp = api.xor(v1[i], v2[i]);
        ret.push(api.xor(tmp, v3[i]));
    }
    ret
}
pub fn cap_sigma0<C: Config>(api: &mut API<C>, bits: &[Variable]) -> Vec<Variable> {
    if bits.len() != 32 {
        panic!("CapSigma0: len(bits) != 32");
    }
    let bits1 = bits.to_vec();
    let bits2 = bits.to_vec();
    let bits3 = bits.to_vec();
    let v1 = rotate_right(&bits1, 2);
    let v2 = rotate_right(&bits2, 13);
    let v3 = rotate_right(&bits3, 22);
    let mut ret = vec![];
    for i in 0..32 {
        let tmp = api.xor(v1[i], v2[i]);
        ret.push(api.xor(tmp, v3[i]));
    }
    ret
}
pub fn cap_sigma1<C: Config>(api: &mut API<C>, bits: &[Variable]) -> Vec<Variable> {
    if bits.len() != 32 {
        panic!("CapSigma1: len(bits) != 32");
    }
    let bits1 = bits.to_vec();
    let bits2 = bits.to_vec();
    let bits3 = bits.to_vec();
    let v1 = rotate_right(&bits1, 6);
    let v2 = rotate_right(&bits2, 11);
    let v3 = rotate_right(&bits3, 25);
    let mut ret = vec![];
    for i in 0..32 {
        let tmp = api.xor(v1[i], v2[i]);
        ret.push(api.xor(tmp, v3[i]));
    }
    ret
}
pub fn ch<C: Config>(
    api: &mut API<C>,
    x: &[Variable],
    y: &[Variable],
    z: &[Variable],
) -> Vec<Variable> {
    if x.len() != 32 || y.len() != 32 || z.len() != 32 {
        panic!("Ch: len(x) != 32 || len(y) != 32 || len(z) != 32");
    }
    let mut ret = vec![];
    for i in 0..32 {
        let tmp1 = api.and(x[i], y[i]);
        let tmp2 = api.xor(x[i], 1);
        let tmp3 = api.and(tmp2, z[i]);
        ret.push(api.xor(tmp1, tmp3));
    }
    ret
}
pub fn maj<C: Config>(
    api: &mut API<C>,
    x: &[Variable],
    y: &[Variable],
    z: &[Variable],
) -> Vec<Variable> {
    if x.len() != 32 || y.len() != 32 || z.len() != 32 {
        panic!("Maj: len(x) != 32 || len(y) != 32 || len(z) != 32");
    }
    let mut ret = vec![];
    for i in 0..32 {
        let tmp1 = api.and(x[i], y[i]);
        let tmp2 = api.and(x[i], z[i]);
        let tmp3 = api.and(y[i], z[i]);
        let tmp4 = api.xor(tmp1, tmp2);
        ret.push(api.xor(tmp3, tmp4));
    }
    ret
}
pub fn big_array_add<C: Config>(
    api: &mut API<C>,
    a: &[Variable],
    b: &[Variable],
    nb_bits: usize,
) -> Vec<Variable> {
    if a.len() != b.len() {
        panic!("BigArrayAdd: length of a and b must be equal");
    }
    let mut c = vec![api.constant(0); a.len()];
    let mut carry = api.constant(0);
    for i in 0..a.len() {
        c[i] = api.add(a[i], b[i]);
        c[i] = api.add(c[i], carry);
        carry = to_binary(api, c[i], nb_bits + 1)[nb_bits];
        let tmp = api.mul(carry, 1 << nb_bits);
        c[i] = api.sub(c[i], tmp);
    }
    c
}
pub fn bit_array_to_m31<C: Config>(api: &mut API<C>, bits: &[Variable]) -> [Variable; 2] {
    if bits.len() >= 60 {
        panic!("BitArrayToM31: length of bits must be less than 60");
    }
    [
        from_binary(api, bits[..30].to_vec()),
        from_binary(api, bits[30..].to_vec()),
    ]
}

pub fn big_endian_m31_array_put_uint32<C: Config>(
    api: &mut API<C>,
    b: &mut [Variable],
    x: [Variable; 2],
) {
    let mut quo = x[0];
    for i in (1..=3).rev() {
        let (q, r) = idiv_mod_bit(api, quo, 8);
        b[i] = r;
        quo = q;
    }
    let shift = api.mul(x[1], 1 << 6);
    b[0] = api.add(quo, shift);
}

pub fn big_endian_put_uint64<C: Config>(
    api: &mut API<C>,
    b: &mut [Variable],
    x: Variable,
) {
    let mut quo = x;
    for i in (1..=7).rev() {
        let (q, r) = idiv_mod_bit(api, quo, 8);
        b[i] = r;
        quo = q;
    }
    b[0] = quo;
}
pub fn m31_to_bit_array<C: Config>(api: &mut API<C>, m31: &[Variable]) -> Vec<Variable> {
    let mut bits = vec![];
    for val in m31 {
        bits.extend_from_slice(&to_binary(api, *val, 30));
    }
    bits
}
// pub fn to_binary<C: Config>(
//     api: &mut API<C>,
//     x: Variable,
//     n_bits: usize,
// ) -> Vec<Variable> {
//     api.new_hint("myhint.tobinary", &[x], n_bits)
// }

pub fn extract_and_remove_tobinary_section(file_path: &str) -> io::Result<Vec<u32>> {
    let file = OpenOptions::new().read(true).write(true).open(file_path)?;
    let reader = BufReader::new(&file);

    let mut lines = reader.lines().filter_map(Result::ok).collect::<Vec<String>>();
    let mut extracted_numbers = Vec::new();
    let mut new_lines = Vec::new();

    let mut found_tobinary = false;
    let mut found_section = false;

    for line in lines.iter() {
        if line.trim() == "tobinary:" {
            if found_tobinary {
                new_lines.push(line.clone()); 
                found_section = true;
            } else {
                found_tobinary = true; 
            }
        } else if found_tobinary && !found_section {
            let nums = line
                .split(',')
                .filter_map(|s| s.trim().parse::<u32>().ok())
                .collect::<Vec<u32>>();

            extracted_numbers.extend(nums);
        } else {
            new_lines.push(line.clone()); 
        }
    }
    fs::write(file_path, new_lines.join("\n"))?;

    Ok(extracted_numbers)
}
pub fn to_binary<C: Config>(
    api: &mut API<C>,
    x: Variable,
    n_bits: usize,
) -> Vec<Variable> {
    let mut res = vec![];
    match extract_and_remove_tobinary_section("./log.txt") {
        Ok(numbers) => {
            println!("Extracted numbers: {:?}", numbers);
            for i in 0..n_bits {
                let number_var = api.constant(numbers[i]);
                res.push(number_var);
            }
        },
        Err(e) => eprintln!("Error: {}", e),
    }
    res
}
pub fn from_binary<C: Config>(api: &mut API<C>, bits: Vec<Variable>) -> Variable {
    let mut res = api.constant(0);
    for (i, bit) in bits.iter().enumerate() {
        let coef = 1 << i;
        let cur = api.mul(coef, *bit);
        res = api.add(res, cur);
    }
    res
}

pub fn to_binary_hint(x: &[M31], y: &mut [M31]) -> Result<(), Error> {
    let t = x[0].to_u256();
    for (i, k) in y.iter_mut().enumerate() {
        *k = M31::from_u256(t >> i as u32 & 1);
    }
    Ok(())
}

pub fn big_is_zero<C: Config>(api: &mut API<C>, k: usize, in_: &[Variable]) -> Variable {
    let mut total = api.constant(k as u32);
    for val in in_.iter().take(k) {
        let tmp = api.is_zero(val);
        total = api.sub(total, tmp);
    }
    api.is_zero(total)
}

pub fn bigint_to_m31_array<C: Config>(
    api: &mut API<C>,
    x: BigInt,
    n_bits: usize,
    limb_len: usize,
) -> Vec<Variable> {
    let mut res = vec![];
    let mut a = x.clone();
    let mut mask = BigInt::from(1) << n_bits;
    mask -= 1;
    for _ in 0..limb_len {
        let tmp = a.clone() & mask.clone();
        let tmp = api.constant(tmp.to_u32().unwrap());
        res.push(tmp);
        a >>= n_bits;
    }
    res
}
pub fn big_less_than<C: Config>(
    api: &mut API<C>,
    n: usize,
    k: usize,
    a: &[Variable],
    b: &[Variable],
) -> Variable {
    let mut lt = vec![];
    let mut eq = vec![];
    for i in 0..k {
        lt.push(my_is_less(api, n, a[i], b[i]));
        let diff = api.sub(a[i], b[i]);
        eq.push(api.is_zero(diff));
    }
    let mut ors = vec![Variable::default(); k - 1];
    let mut ands = vec![Variable::default(); k - 1];
    let mut eq_ands = vec![Variable::default(); k - 1];
    for i in (0..k - 1).rev() {
        if i == k - 2 {
            ands[i] = api.and(eq[k - 1], lt[k - 2]);
            eq_ands[i] = api.and(eq[k - 1], eq[k - 2]);
            ors[i] = api.or(lt[k - 1], ands[k - 2]);
        } else {
            ands[i] = api.and(eq_ands[i + 1], lt[i]);
            eq_ands[i] = api.and(eq_ands[i + 1], eq[i]);
            ors[i] = api.or(ors[i + 1], ands[i]);
        }
    }
    ors[0]
}
pub fn my_is_less<C: Config>(
    api: &mut API<C>,
    n: usize,
    a: Variable,
    b: Variable,
) -> Variable {
    let neg_b = api.neg(b);
    let tmp = api.add(a, 1 << n);
    let tmp = api.add(tmp, neg_b);
    let bi1 = to_binary(api, tmp, n + 1);
    let one = api.constant(1);
    api.sub(one, bi1[n])
}

pub fn idiv_mod_bit<C: Config>(
    builder: &mut API<C>,
    a: Variable,
    b: u64,
) -> (Variable, Variable) {
    let bits = to_binary(builder, a, 30);
    let quotient = from_binary(builder, bits[b as usize..].to_vec());
    let remainder = from_binary(builder, bits[..b as usize].to_vec());
    (quotient, remainder)
}

pub fn string_to_m31_array(s: &str, nb_bits: u32) -> [M31; 48] {
    let mut big =
        BigInt::parse_bytes(s.as_bytes(), 10).unwrap_or_else(|| panic!("Failed to parse BigInt"));
    let mut res = [M31::from(0); 48];
    let base = BigInt::from(1) << nb_bits;
    for cur_res in &mut res {
        let tmp = &big % &base;
        *cur_res = M31::from(tmp.to_u32().unwrap());
        big >>= nb_bits;
    }
    res
}
impl MyDigest {
    fn new<C: Config>(api: &mut API<C>) -> Self {
        let mut h = [[api.constant(0); 2]; 8];
        h[0][0] = api.constant(INIT00);
        h[0][1] = api.constant(INIT01);
        h[1][0] = api.constant(INIT10);
        h[1][1] = api.constant(INIT11);
        h[2][0] = api.constant(INIT20);
        h[2][1] = api.constant(INIT21);
        h[3][0] = api.constant(INIT30);
        h[3][1] = api.constant(INIT31);
        h[4][0] = api.constant(INIT40);
        h[4][1] = api.constant(INIT41);
        h[5][0] = api.constant(INIT50);
        h[5][1] = api.constant(INIT51);
        h[6][0] = api.constant(INIT60);
        h[6][1] = api.constant(INIT61);
        h[7][0] = api.constant(INIT70);
        h[7][1] = api.constant(INIT71);
        let mut kbits_u8 = [[0; 32]; 64];
        for i in 0..64 {
            for j in 0..32 {
                kbits_u8[i][j] = ((_K[i] >> j) & 1) as u8;
            }
        }
        let mut kbits = [[api.constant(0); 32]; 64];
        for i in 0..64 {
            for j in 0..32 {
                kbits[i][j] = api.constant(kbits_u8[i][j] as u32);
            }
        }
        MyDigest {
            h,
            nx: 0,
            len: 0,
            kbits,
        }
    }
    fn reset<C: Config>(&mut self, api: &mut API<C>) {
        for i in 0..8 {
            self.h[i] = [api.constant(0); 2];
        }
        self.h[0][0] = api.constant(INIT00);
        self.h[0][1] = api.constant(INIT01);
        self.h[1][0] = api.constant(INIT10);
        self.h[1][1] = api.constant(INIT11);
        self.h[2][0] = api.constant(INIT20);
        self.h[2][1] = api.constant(INIT21);
        self.h[3][0] = api.constant(INIT30);
        self.h[3][1] = api.constant(INIT31);
        self.h[4][0] = api.constant(INIT40);
        self.h[4][1] = api.constant(INIT41);
        self.h[5][0] = api.constant(INIT50);
        self.h[5][1] = api.constant(INIT51);
        self.h[6][0] = api.constant(INIT60);
        self.h[6][1] = api.constant(INIT61);
        self.h[7][0] = api.constant(INIT70);
        self.h[7][1] = api.constant(INIT71);
        self.nx = 0;
        self.len = 0;
    }
    //always write a chunk
    fn chunk_write<C: Config>(&mut self, api: &mut API<C>, p: &[Variable]) {
        if p.len() != CHUNK || self.nx != 0 {
            panic!("p.len() != CHUNK || self.nx != 0");
        }
        self.len += CHUNK as u64;
        let tmp_h = self.h;
        self.h = self.block(api, tmp_h, p);
    }
    fn return_sum<C: Config>(&mut self, api: &mut API<C>) -> [Variable; SHA256LEN] {
        let mut digest = [api.constant(0); SHA256LEN];

        big_endian_m31_array_put_uint32(api, &mut digest[0..], self.h[0]);
        big_endian_m31_array_put_uint32(api, &mut digest[4..], self.h[1]);
        big_endian_m31_array_put_uint32(api, &mut digest[8..], self.h[2]);
        big_endian_m31_array_put_uint32(api, &mut digest[12..], self.h[3]);
        big_endian_m31_array_put_uint32(api, &mut digest[16..], self.h[4]);
        big_endian_m31_array_put_uint32(api, &mut digest[20..], self.h[5]);
        big_endian_m31_array_put_uint32(api, &mut digest[24..], self.h[6]);
        big_endian_m31_array_put_uint32(api, &mut digest[28..], self.h[7]);
        digest
    }

    fn block<C: Config>(
        &mut self,
        api: &mut API<C>,
        h: [[Variable; 2]; 8],
        p: &[Variable],
    ) -> [[Variable; 2]; 8] {
        let mut p = p;
        let mut hh = h;
        while p.len() >= CHUNK {
            let mut msg_schedule = vec![];
            for t in 0..64 {
                if t <= 15 {
                    msg_schedule.push(bytes_to_bits(api, &p[t * 4..t * 4 + 4]));
                } else {
                    let term1_tmp = sigma1(api, &msg_schedule[t - 2]);
                    let term1 = bit_array_to_m31(api, &term1_tmp);
                    let term2 = bit_array_to_m31(api, &msg_schedule[t - 7]);
                    let term3_tmp = sigma0(api, &msg_schedule[t - 15]);
                    let term3 = bit_array_to_m31(api, &term3_tmp);
                    let term4 = bit_array_to_m31(api, &msg_schedule[t - 16]);
                    let schedule_tmp1 = big_array_add(api, &term1, &term2, 30);
                    let schedule_tmp2 = big_array_add(api, &term3, &term4, 30);
                    let schedule = big_array_add(api, &schedule_tmp1, &schedule_tmp2, 30);
                    let schedule_bits = m31_to_bit_array(api, &schedule)[..32].to_vec();
                    msg_schedule.push(schedule_bits);
                }
            }
            let mut a = hh[0].to_vec();
            let mut b = hh[1].to_vec();
            let mut c = hh[2].to_vec();
            let mut d = hh[3].to_vec();
            let mut e = hh[4].to_vec();
            let mut f = hh[5].to_vec();
            let mut g = hh[6].to_vec();
            let mut h = hh[7].to_vec();

            //rewrite
            let mut a_bit = m31_to_bit_array(api, &a)[..32].to_vec();
            let mut b_bit = m31_to_bit_array(api, &b)[..32].to_vec();
            let mut c_bit = m31_to_bit_array(api, &c)[..32].to_vec();
            let mut e_bit = m31_to_bit_array(api, &e)[..32].to_vec();
            let mut f_bit = m31_to_bit_array(api, &f)[..32].to_vec();
            let mut g_bit = m31_to_bit_array(api, &g)[..32].to_vec();
            for (t, schedule) in msg_schedule.iter().enumerate().take(64) {
                let mut t1_term1 = [api.constant(0); 2];
                t1_term1[0] = h[0];
                t1_term1[1] = h[1];
                let t1_term2_tmp = cap_sigma1(api, &e_bit);
                let t1_term2 = bit_array_to_m31(api, &t1_term2_tmp);
                let t1_term3_tmp = ch(api, &e_bit, &f_bit, &g_bit);
                let t1_term3 = bit_array_to_m31(api, &t1_term3_tmp);
                let t1_term4 = bit_array_to_m31(api, &self.kbits[t]); //rewrite to [2]frontend.Variable
                let t1_term5 = bit_array_to_m31(api, schedule);
                let tmp1 = big_array_add(api, &t1_term1, &t1_term2, 30);
                let tmp2 = big_array_add(api, &t1_term3, &t1_term4, 30);
                let tmp3 = big_array_add(api, &tmp1, &tmp2, 30);
                let tmp4 = big_array_add(api, &tmp3, &t1_term5, 30);
                let t1 = tmp4;
                let t2_tmp1 = cap_sigma0(api, &a_bit);
                let t2_tmp2 = bit_array_to_m31(api, &t2_tmp1);
                let t2_tmp3 = maj(api, &a_bit, &b_bit, &c_bit);
                let t2_tmp4 = bit_array_to_m31(api, &t2_tmp3);
                let t2 = big_array_add(api, &t2_tmp2, &t2_tmp4, 30);
                let new_a_bit_tmp = big_array_add(api, &t1, &t2, 30);
                let new_a_bit = m31_to_bit_array(api, &new_a_bit_tmp)[..32].to_vec();
                let new_e_bit_tmp = big_array_add(api, &d[..2], &t1, 30);
                let new_e_bit = m31_to_bit_array(api, &new_e_bit_tmp)[..32].to_vec();
                h = g.to_vec();
                g = f.to_vec();
                f = e.to_vec();
                d = c.to_vec();
                c = b.to_vec();
                b = a.to_vec();
                a = bit_array_to_m31(api, &new_a_bit).to_vec();
                e = bit_array_to_m31(api, &new_e_bit).to_vec();
                g_bit = f_bit.to_vec();
                f_bit = e_bit.to_vec();
                c_bit = b_bit.to_vec();
                b_bit = a_bit.to_vec();
                a_bit = new_a_bit.to_vec();
                e_bit = new_e_bit.to_vec();
            }
            let hh0_tmp1 = big_array_add(api, &hh[0], &a, 30);
            let hh0_tmp2 = m31_to_bit_array(api, &hh0_tmp1);
            hh[0] = bit_array_to_m31(api, &hh0_tmp2[..32])
                .as_slice()
                .try_into()
                .unwrap();
            let hh1_tmp1 = big_array_add(api, &hh[1], &b, 30);
            let hh1_tmp2 = m31_to_bit_array(api, &hh1_tmp1);
            hh[1] = bit_array_to_m31(api, &hh1_tmp2[..32])
                .as_slice()
                .try_into()
                .unwrap();
            let hh2_tmp1 = big_array_add(api, &hh[2], &c, 30);
            let hh2_tmp2 = m31_to_bit_array(api, &hh2_tmp1);
            hh[2] = bit_array_to_m31(api, &hh2_tmp2[..32])
                .as_slice()
                .try_into()
                .unwrap();
            let hh3_tmp1 = big_array_add(api, &hh[3], &d, 30);
            let hh3_tmp2 = m31_to_bit_array(api, &hh3_tmp1);
            hh[3] = bit_array_to_m31(api, &hh3_tmp2[..32])
                .as_slice()
                .try_into()
                .unwrap();
            let hh4_tmp1 = big_array_add(api, &hh[4], &e, 30);
            let hh4_tmp2 = m31_to_bit_array(api, &hh4_tmp1);
            hh[4] = bit_array_to_m31(api, &hh4_tmp2[..32])
                .as_slice()
                .try_into()
                .unwrap();
            let hh5_tmp1 = big_array_add(api, &hh[5], &f, 30);
            let hh5_tmp2 = m31_to_bit_array(api, &hh5_tmp1);
            hh[5] = bit_array_to_m31(api, &hh5_tmp2[..32])
                .as_slice()
                .try_into()
                .unwrap();
            let hh6_tmp1 = big_array_add(api, &hh[6], &g, 30);
            let hh6_tmp2 = m31_to_bit_array(api, &hh6_tmp1);
            hh[6] = bit_array_to_m31(api, &hh6_tmp2[..32])
                .as_slice()
                .try_into()
                .unwrap();
            let hh7_tmp1 = big_array_add(api, &hh[7], &h, 30);
            let hh7_tmp2 = m31_to_bit_array(api, &hh7_tmp1);
            hh[7] = bit_array_to_m31(api, &hh7_tmp2[..32])
                .as_slice()
                .try_into()
                .unwrap();
            p = &p[CHUNK..];
        }
        hh
    }
}
#[kernel]
fn sha256_37bytes<C: Config>(
    builder: &mut API<C>,
    orign_data: &[InputVariable; 32],
    output_data: &mut [OutputVariable; SHA256LEN],
) -> Vec<Variable> {
    let mut data = orign_data.to_vec();
    for _ in 32..37 {
        data.push(builder.constant(255));
    }
    let n = data.len();
    if n != 32 + 1 + 4 {
        panic!("len(orignData) !=  32+1+4")
    }
    let mut pre_pad = vec![builder.constant(0); 64 - 37];
    pre_pad[0] = builder.constant(128); //0x80
    pre_pad[64 - 37 - 2] = builder.constant((37) * 8 / 256); //length byte
    pre_pad[64 - 37 - 1] = builder.constant((32 + 1 + 4) * 8 - 256); //length byte
    data.append(&mut pre_pad); //append padding
    let mut d = MyDigest::new(builder);
    d.reset(builder);
    d.chunk_write(builder, &data);
    let res = d.return_sum(builder).to_vec();
    for (i, val) in res.iter().enumerate() {
        output_data[i] = *val;
    }
}


#[test]
fn zkcuda_sha256_37bytes() {
    let kernel_check_sha256_37bytes: Kernel<M31Config> = compile_sha256_37bytes().unwrap();
        let data = [255; 32];
        let mut hash = Sha256::new();
        hash.update(data);
        let output = hash.finalize();
        let mut input_vars = vec![];
        let mut output_vars = vec![];
        for i in 0..32 {
            input_vars.push(M31::from(data[i] as u32));
        }
        for i in 0..32 {
            output_vars.push(M31::from(output[i] as u32));
        }

        let mut ctx: Context<M31Config, DummyProvingSystem<M31Config>> = Context::default();

        let a = ctx.copy_to_device(&input_vars, false);
        let mut c = None;
        call_kernel!(ctx, kernel_check_sha256_37bytes, a, mut c);
        let c = c.reshape(&[]);
        let result: Vec<M31> = ctx.copy_to_host(c);
        assert_eq!(result, output_vars);
}
