use std::collections::HashMap;
use std::rc::Rc;
use crate::gnark::limbs::*;
use crate::gnark::utils::*;
use crate::gnark::emparam::FieldParams;
use crate::gnark::element::*;
use expander_compiler::frontend::extra::*;
use expander_compiler::{circuit::layered::InputType, frontend::*};
use expander_compiler::frontend::builder::*;
use num_bigint::BigInt;
use num_traits::Signed;
use num_traits::ToPrimitive;
use ark_bls12_381::Fq;
use ark_bls12_381::Fq2;

// pub fn to_binary_hint(x: &[M31], y: &mut [M31]) -> Result<(), Error> {
//     let t = x[0].to_u256();
//     for (i, k) in y.iter_mut().enumerate() {
//         *k = M31::from_u256(t >> i as u32 & 1);
//     }
//     Ok(())
// }

pub fn mul_hint(inputs: &[M31], outputs: &mut [M31]) -> Result<(), Error> {
    let nb_bits = inputs[0].to_u256().as_usize();
    let nb_limbs = inputs[1].to_u256().as_usize();
    let nb_a_len = inputs[2].to_u256().as_usize();
    let nb_quo_len = inputs[3].to_u256().as_usize();
    let nb_b_len = inputs.len() - 4 - nb_limbs - nb_a_len;
    let mut ptr = 4;
    let plimbs_m31 = &inputs[ptr..ptr + nb_limbs as usize];
    let plimbs_u32:Vec<u32> = (0..nb_limbs).map(|i| plimbs_m31[i].to_u256().as_u32()).collect();
    let plimbs:Vec<BigInt> = plimbs_u32.iter().map(|x| BigInt::from(*x)).collect();
    ptr += nb_limbs;
    let alimbs_m31 = &inputs[ptr..ptr + nb_a_len as usize];
    let alimbs_u32:Vec<u32> = (0..nb_a_len).map(|i| alimbs_m31[i].to_u256().as_u32()).collect();
    let alimbs:Vec<BigInt> = alimbs_u32.iter().map(|x| BigInt::from(*x)).collect();
    ptr += nb_a_len;
    let blimbs_m31 = &inputs[ptr..ptr + nb_b_len as usize];
    let blimbs_u32:Vec<u32> = (0..nb_b_len).map(|i| blimbs_m31[i].to_u256().as_u32()).collect();
    let blimbs:Vec<BigInt> = blimbs_u32.iter().map(|x| BigInt::from(*x)).collect();

    let nb_carry_len = std::cmp::max(nb_multiplication_res_limbs(nb_a_len, nb_b_len), nb_multiplication_res_limbs(nb_quo_len, nb_limbs)) - 1;

    let mut p = BigInt::default();
    let mut a = BigInt::default();
    let mut b = BigInt::default();
    p = recompose(plimbs.clone(), nb_bits as u32);
    a = recompose(alimbs.clone(), nb_bits as u32);
    println!("a: {:?}", a);
    println!("blimbs: {:?}", blimbs);
    b = recompose(blimbs.clone(), nb_bits as u32);
    println!("b: {:?}", b);
    let mut quo = BigInt::default();
    let mut rem = BigInt::default();
    let mut ab = a.clone() * b.clone();
    println!("ab: {:?}", ab);
    quo = ab.clone() / p.clone();
    println!("quo: {:?}", quo);
    rem = ab.clone() % p.clone();
    println!("rem: {:?}", rem);
    let mut quo_limbs = vec![BigInt::default(); nb_quo_len];
    if let Err(err) = decompose(&quo, nb_bits as u32, &mut quo_limbs) {
        panic!("decompose value: {}", err);
    }
    let mut rem_limbs = vec![BigInt::default(); nb_limbs];
    if let Err(err) = decompose(&rem, nb_bits as u32, &mut rem_limbs) {
        panic!("decompose value: {}", err);
    }
    let mut xp = vec![BigInt::default(); nb_multiplication_res_limbs(nb_a_len, nb_b_len)];
    let mut yp = vec![BigInt::default(); nb_multiplication_res_limbs(nb_quo_len, nb_limbs)];
    let mut tmp = BigInt::default();
    for i in 0..xp.len() {
        xp[i] = BigInt::default();
    }
    for i in 0..yp.len() {
        yp[i] = BigInt::default();
    }
    // we know compute the schoolbook multiprecision multiplication of a*b and
    // r+k*p
    for i in 0..nb_a_len {
        for j in 0..nb_b_len {
            tmp = alimbs[i].clone();
            tmp *= &blimbs[j];
            xp[i + j] += &tmp;
        }
    }
    for i in 0..nb_limbs {
        yp[i] += &rem_limbs[i];
        for j in 0..nb_quo_len {
            tmp = quo_limbs[j].clone();
            tmp *= &plimbs[i];
            yp[i + j] += &tmp;
        }
    }
    let mut carry = BigInt::default();
    let mut carry_limbs = vec![BigInt::default(); nb_carry_len];
    for i in 0..carry_limbs.len() {
        if i < xp.len() {
            carry += &xp[i];
        }
        if i < yp.len() {
            carry -= &yp[i];
        }
        carry >>= nb_bits as u32;
        //if carry is negative, we need to add 2^nb_bits to it
        carry_limbs[i] = carry.clone();
    }
    //convert limbs to m31 output
    // println!("quo_limbs: {:?}", quo_limbs);
    let mut outptr = 0;
    for i in 0..nb_quo_len {
        outputs[outptr+i] = M31::from(quo_limbs[i].to_u64().unwrap() as u32);
    }
    // println!("rem_limbs: {:?}", rem_limbs);
    outptr += nb_quo_len;
    for i in 0..nb_limbs {
        outputs[outptr+i] = M31::from(rem_limbs[i].to_u64().unwrap() as u32);
    }
    outptr += nb_limbs;
    // println!("carry_limbs: {:?}", carry_limbs);
    for i in 0..nb_carry_len {
        if carry_limbs[i] < BigInt::default() {
            outputs[outptr+i] = -M31::from(carry_limbs[i].abs().to_u64().unwrap() as u32);
        } else {
            outputs[outptr+i] = M31::from(carry_limbs[i].to_u64().unwrap() as u32);
        }
    }
    Ok(())
}

/*
func(mod *big.Int, inputs, outputs []*big.Int) error {
			startTime := time.Now()
			var a, b, c bls12381.E2

			a.A0.SetBigInt(inputs[0])
			a.A1.SetBigInt(inputs[1])
			b.A0.SetBigInt(inputs[2])
			b.A1.SetBigInt(inputs[3])

			c.Inverse(&b).Mul(&c, &a)

			c.A0.BigInt(outputs[0])
			c.A1.BigInt(outputs[1])
			fmt.Println("divE2Hint time: ", time.Since(startTime))
			return nil
		})
*/
// pub fn div_e2_hint(inputs: &[M31], outputs: &mut [M31]) -> Result<(), Error> {
//     let a = Fq2::new(Fq::from(inputs[0].to_u256().as_i64()), Fq::from(inputs[1].to_u256().as_i64()));
//     let b = Fq2::new(Fq::from(inputs[2].to_u256().as_i64()), Fq::from(inputs[3].to_u256().as_i64()));
//     let c = a / b;
//     let c0 = c.c0;
//     let c1 = c.c1;
//     let c0_bigint = c0.0;
//     outputs[0] = M31::from(c.a0().to_u256().as_u32());
//     outputs[1] = M31::from(c.a1().to_u256().as_u32());
//     Ok(())
// }

pub fn simple_rangecheck_hint(inputs: &[M31], outputs: &mut [M31]) -> Result<(), Error> {
    let nb_bits = inputs[0].to_u256().as_u32();
    let number = inputs[1].to_u256().as_f64();
    let number_bit = if number > 1.0 {number.log2().ceil() as u32} else {1};
    if number_bit > nb_bits {
        panic!("number is out of range");
    } 
    
    Ok(())
}