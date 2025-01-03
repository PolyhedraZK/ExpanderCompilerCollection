use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;
use crate::gnark::limbs::*;
use crate::gnark::utils::*;
use crate::gnark::emparam::FieldParams;
use crate::gnark::element::*;
use ark_bls12_381::Fq12;
use ark_bls12_381::Fq6;
use ark_ff::Zero;
use expander_compiler::frontend::extra::*;
use expander_compiler::{circuit::layered::InputType, frontend::*};
use expander_compiler::frontend::builder::*;
use num_bigint::BigInt;
use num_bigint::BigUint;
use num_bigint::Sign;
use num_traits::Signed;
use num_traits::ToPrimitive;
use ark_bls12_381::Fq;
use ark_bls12_381::Fq2;
use ark_ff::fields::Field;
use num_traits::One;

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

pub fn div_e2_hint(inputs: &[M31], outputs: &mut [M31]) -> Result<(), Error> {
    if let Err(err) = unwrap_hint(true, true, inputs, outputs, 
        //divE2Hint
        |inputs| {
            let biguint_inputs = inputs.iter().map(|x| x.to_biguint().unwrap()).collect::<Vec<_>>();
            let a = Fq2::new(Fq::from(biguint_inputs[0].clone()), Fq::from(biguint_inputs[1].clone()));
            let b = Fq2::new(Fq::from(biguint_inputs[2].clone()), Fq::from(biguint_inputs[3].clone()));
            let c = a / b;
            let c0_bigint = c.c0.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c1_bigint = c.c1.to_string().parse::<BigInt>().expect("Invalid decimal string");
            return vec![c0_bigint, c1_bigint];
        }
    ) {
        panic!("divE2Hint: {}", err);
    }
    Ok(())
}

pub fn inverse_e2_hint(inputs: &[M31], outputs: &mut [M31]) -> Result<(), Error> {
    if let Err(err) = unwrap_hint(true, true, inputs, outputs, 
        //inverseE2Hint
        |inputs| {
            let biguint_inputs = inputs.iter().map(|x| x.to_biguint().unwrap()).collect::<Vec<_>>();
            let a = Fq2::new(Fq::from(biguint_inputs[0].clone()), Fq::from(biguint_inputs[1].clone()));
            let c = a.inverse().unwrap();
            let c0_bigint = c.c0.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c1_bigint = c.c1.to_string().parse::<BigInt>().expect("Invalid decimal string");
            return vec![c0_bigint, c1_bigint];
        }
    ) {
        panic!("inverseE2Hint: {}", err);
    }
    Ok(())
}

pub fn div_e6_hint(inputs: &[M31], outputs: &mut [M31]) -> Result<(), Error> {
    if let Err(err) = unwrap_hint(true, true, inputs, outputs, 
        //divE6Hint
        |inputs| {
            let biguint_inputs = inputs.iter().map(|x| x.to_biguint().unwrap()).collect::<Vec<_>>();
            let a_b0 = Fq2::new(Fq::from(biguint_inputs[0].clone()), Fq::from(biguint_inputs[1].clone()));
            let a_b1 = Fq2::new(Fq::from(biguint_inputs[2].clone()), Fq::from(biguint_inputs[3].clone()));
            let a_b2 = Fq2::new(Fq::from(biguint_inputs[4].clone()), Fq::from(biguint_inputs[5].clone()));
            let a = Fq6::new(a_b0, a_b1, a_b2);
            let b_b0 = Fq2::new(Fq::from(biguint_inputs[6].clone()), Fq::from(biguint_inputs[7].clone()));
            let b_b1 = Fq2::new(Fq::from(biguint_inputs[8].clone()), Fq::from(biguint_inputs[9].clone()));
            let b_b2 = Fq2::new(Fq::from(biguint_inputs[10].clone()), Fq::from(biguint_inputs[11].clone()));
            let b = Fq6::new(b_b0, b_b1, b_b2);
            let c = a / b;
            let c_c0_c0_bigint = c.c0.c0.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c0_c1_bigint = c.c0.c1.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c1_c0_bigint = c.c1.c0.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c1_c1_bigint = c.c1.c1.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c2_c0_bigint = c.c2.c0.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c2_c1_bigint = c.c2.c1.to_string().parse::<BigInt>().expect("Invalid decimal string");
            return vec![c_c0_c0_bigint, c_c0_c1_bigint, c_c1_c0_bigint, c_c1_c1_bigint, c_c2_c0_bigint, c_c2_c1_bigint];
        }
    ) {
        panic!("divE6Hint: {}", err);
    }
    Ok(())
}

pub fn inverse_e6_hint(inputs: &[M31], outputs: &mut [M31]) -> Result<(), Error> {
    if let Err(err) = unwrap_hint(true, true, inputs, outputs, 
        //inverseE6Hint
        |inputs| {
            let biguint_inputs = inputs.iter().map(|x| x.to_biguint().unwrap()).collect::<Vec<_>>();
            let a_b0 = Fq2::new(Fq::from(biguint_inputs[0].clone()), Fq::from(biguint_inputs[1].clone()));
            let a_b1 = Fq2::new(Fq::from(biguint_inputs[2].clone()), Fq::from(biguint_inputs[3].clone()));
            let a_b2 = Fq2::new(Fq::from(biguint_inputs[4].clone()), Fq::from(biguint_inputs[5].clone()));
            let a = Fq6::new(a_b0, a_b1, a_b2);
            let c = a.inverse().unwrap();
            let c_c0_c0_bigint = c.c0.c0.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c0_c1_bigint = c.c0.c1.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c1_c0_bigint = c.c1.c0.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c1_c1_bigint = c.c1.c1.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c2_c0_bigint = c.c2.c0.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c2_c1_bigint = c.c2.c1.to_string().parse::<BigInt>().expect("Invalid decimal string");
            return vec![c_c0_c0_bigint, c_c0_c1_bigint, c_c1_c0_bigint, c_c1_c1_bigint, c_c2_c0_bigint, c_c2_c1_bigint];
        }
    ) {
        panic!("inverseE6Hint: {}", err);
    }
    Ok(())
}

pub fn div_e6_by_6_hint(inputs: &[M31], outputs: &mut [M31]) -> Result<(), Error> {
    if let Err(err) = unwrap_hint(true, true, inputs, outputs, 
        //divE6By6Hint
        |inputs| {
            let biguint_inputs = inputs.iter().map(|x| x.to_biguint().unwrap()).collect::<Vec<_>>();
            let a_b0 = Fq2::new(Fq::from(biguint_inputs[0].clone()), Fq::from(biguint_inputs[1].clone()));
            let a_b1 = Fq2::new(Fq::from(biguint_inputs[2].clone()), Fq::from(biguint_inputs[3].clone()));
            let a_b2 = Fq2::new(Fq::from(biguint_inputs[4].clone()), Fq::from(biguint_inputs[5].clone()));
            let mut a = Fq6::new(a_b0, a_b1, a_b2);
            let six_inv = Fq::from(6u32).inverse().unwrap();
            a.c0.mul_assign_by_fp(&six_inv);
            a.c1.mul_assign_by_fp(&six_inv);
            a.c2.mul_assign_by_fp(&six_inv);
            let c_c0_c0_bigint = a.c0.c0.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c0_c1_bigint = a.c0.c1.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c1_c0_bigint = a.c1.c0.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c1_c1_bigint = a.c1.c1.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c2_c0_bigint = a.c2.c0.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c2_c1_bigint = a.c2.c1.to_string().parse::<BigInt>().expect("Invalid decimal string");
            return vec![c_c0_c0_bigint, c_c0_c1_bigint, c_c1_c0_bigint, c_c1_c1_bigint, c_c2_c0_bigint, c_c2_c1_bigint];
        }
    ) {
        panic!("divE6By6Hint: {}", err);
    }
    Ok(())
}

pub fn div_e12_hint(inputs: &[M31], outputs: &mut [M31]) -> Result<(), Error> {
    if let Err(err) = unwrap_hint(true, true, inputs, outputs, 
        //divE12Hint
        |inputs| {
            let biguint_inputs = inputs.iter().map(|x| x.to_biguint().unwrap()).collect::<Vec<_>>();

            let a_c0_b0 = Fq2::new(Fq::from(biguint_inputs[0].clone()), Fq::from(biguint_inputs[1].clone()));
            let a_c0_b1 = Fq2::new(Fq::from(biguint_inputs[2].clone()), Fq::from(biguint_inputs[3].clone()));
            let a_c0_b2 = Fq2::new(Fq::from(biguint_inputs[4].clone()), Fq::from(biguint_inputs[5].clone()));
            let a_c0 = Fq6::new(a_c0_b0, a_c0_b1, a_c0_b2);
            let a_c1_b0 = Fq2::new(Fq::from(biguint_inputs[6].clone()), Fq::from(biguint_inputs[7].clone()));
            let a_c1_b1 = Fq2::new(Fq::from(biguint_inputs[8].clone()), Fq::from(biguint_inputs[9].clone()));
            let a_c1_b2 = Fq2::new(Fq::from(biguint_inputs[10].clone()), Fq::from(biguint_inputs[11].clone()));
            let a_c1 = Fq6::new(a_c1_b0, a_c1_b1, a_c1_b2);
            let a = Fq12::new(a_c0, a_c1);

            let b_c0_b0 = Fq2::new(Fq::from(biguint_inputs[12].clone()), Fq::from(biguint_inputs[13].clone()));
            let b_c0_b1 = Fq2::new(Fq::from(biguint_inputs[14].clone()), Fq::from(biguint_inputs[15].clone()));
            let b_c0_b2 = Fq2::new(Fq::from(biguint_inputs[16].clone()), Fq::from(biguint_inputs[17].clone()));
            let b_c0 = Fq6::new(b_c0_b0, b_c0_b1, b_c0_b2);
            let b_c1_b0 = Fq2::new(Fq::from(biguint_inputs[18].clone()), Fq::from(biguint_inputs[19].clone()));
            let b_c1_b1 = Fq2::new(Fq::from(biguint_inputs[20].clone()), Fq::from(biguint_inputs[21].clone()));
            let b_c1_b2 = Fq2::new(Fq::from(biguint_inputs[22].clone()), Fq::from(biguint_inputs[23].clone()));
            let b_c1 = Fq6::new(b_c1_b0, b_c1_b1, b_c1_b2);
            let b = Fq12::new(b_c0, b_c1);

            let c = a / b;
            let c_c0_b0_a0_bigint = c.c0.c0.c0.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c0_b0_a1_bigint = c.c0.c0.c1.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c0_b1_a0_bigint = c.c0.c1.c0.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c0_b1_a1_bigint = c.c0.c1.c1.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c0_b2_a0_bigint = c.c0.c2.c0.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c0_b2_a1_bigint = c.c0.c2.c1.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c1_b0_a0_bigint = c.c1.c0.c0.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c1_b0_a1_bigint = c.c1.c0.c1.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c1_b1_a0_bigint = c.c1.c1.c0.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c1_b1_a1_bigint = c.c1.c1.c1.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c1_b2_a0_bigint = c.c1.c2.c0.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c1_b2_a1_bigint = c.c1.c2.c1.to_string().parse::<BigInt>().expect("Invalid decimal string");

            return vec![c_c0_b0_a0_bigint, c_c0_b0_a1_bigint, c_c0_b1_a0_bigint, c_c0_b1_a1_bigint, c_c0_b2_a0_bigint, c_c0_b2_a1_bigint, c_c1_b0_a0_bigint, c_c1_b0_a1_bigint, c_c1_b1_a0_bigint, c_c1_b1_a1_bigint, c_c1_b2_a0_bigint, c_c1_b2_a1_bigint];
        }
    ) {
        panic!("divE12Hint: {}", err);
    }
    Ok(())
}

/*

func inverseE12Hint(nativeMod *big.Int, nativeInputs, nativeOutputs []*big.Int) error {
	return emulated.UnwrapHint(nativeInputs, nativeOutputs,
		func(mod *big.Int, inputs, outputs []*big.Int) error {
			var a, c bls12381.E12

			a.C0.B0.A0.SetBigInt(inputs[0])
			a.C0.B0.A1.SetBigInt(inputs[1])
			a.C0.B1.A0.SetBigInt(inputs[2])
			a.C0.B1.A1.SetBigInt(inputs[3])
			a.C0.B2.A0.SetBigInt(inputs[4])
			a.C0.B2.A1.SetBigInt(inputs[5])
			a.C1.B0.A0.SetBigInt(inputs[6])
			a.C1.B0.A1.SetBigInt(inputs[7])
			a.C1.B1.A0.SetBigInt(inputs[8])
			a.C1.B1.A1.SetBigInt(inputs[9])
			a.C1.B2.A0.SetBigInt(inputs[10])
			a.C1.B2.A1.SetBigInt(inputs[11])

			c.Inverse(&a)

			c.C0.B0.A0.BigInt(outputs[0])
			c.C0.B0.A1.BigInt(outputs[1])
			c.C0.B1.A0.BigInt(outputs[2])
			c.C0.B1.A1.BigInt(outputs[3])
			c.C0.B2.A0.BigInt(outputs[4])
			c.C0.B2.A1.BigInt(outputs[5])
			c.C1.B0.A0.BigInt(outputs[6])
			c.C1.B0.A1.BigInt(outputs[7])
			c.C1.B1.A0.BigInt(outputs[8])
			c.C1.B1.A1.BigInt(outputs[9])
			c.C1.B2.A0.BigInt(outputs[10])
			c.C1.B2.A1.BigInt(outputs[11])

			return nil
		})
}
*/
pub fn inverse_e12_hint(inputs: &[M31], outputs: &mut [M31]) -> Result<(), Error> {
    if let Err(err) = unwrap_hint(true, true, inputs, outputs, 
        //inverseE12Hint
        |inputs| {
            let biguint_inputs = inputs.iter().map(|x| x.to_biguint().unwrap()).collect::<Vec<_>>();

            let a_c0_b0 = Fq2::new(Fq::from(biguint_inputs[0].clone()), Fq::from(biguint_inputs[1].clone()));
            let a_c0_b1 = Fq2::new(Fq::from(biguint_inputs[2].clone()), Fq::from(biguint_inputs[3].clone()));
            let a_c0_b2 = Fq2::new(Fq::from(biguint_inputs[4].clone()), Fq::from(biguint_inputs[5].clone()));
            let a_c0 = Fq6::new(a_c0_b0, a_c0_b1, a_c0_b2);
            let a_c1_b0 = Fq2::new(Fq::from(biguint_inputs[6].clone()), Fq::from(biguint_inputs[7].clone()));
            let a_c1_b1 = Fq2::new(Fq::from(biguint_inputs[8].clone()), Fq::from(biguint_inputs[9].clone()));
            let a_c1_b2 = Fq2::new(Fq::from(biguint_inputs[10].clone()), Fq::from(biguint_inputs[11].clone()));
            let a_c1 = Fq6::new(a_c1_b0, a_c1_b1, a_c1_b2);
            let mut a = Fq12::new(a_c0, a_c1);

            let c = a.inverse().unwrap();
            let c_c0_b0_a0_bigint = c.c0.c0.c0.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c0_b0_a1_bigint = c.c0.c0.c1.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c0_b1_a0_bigint = c.c0.c1.c0.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c0_b1_a1_bigint = c.c0.c1.c1.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c0_b2_a0_bigint = c.c0.c2.c0.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c0_b2_a1_bigint = c.c0.c2.c1.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c1_b0_a0_bigint = c.c1.c0.c0.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c1_b0_a1_bigint = c.c1.c0.c1.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c1_b1_a0_bigint = c.c1.c1.c0.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c1_b1_a1_bigint = c.c1.c1.c1.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c1_b2_a0_bigint = c.c1.c2.c0.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c1_b2_a1_bigint = c.c1.c2.c1.to_string().parse::<BigInt>().expect("Invalid decimal string");

            return vec![c_c0_b0_a0_bigint, c_c0_b0_a1_bigint, c_c0_b1_a0_bigint, c_c0_b1_a1_bigint, c_c0_b2_a0_bigint, c_c0_b2_a1_bigint, c_c1_b0_a0_bigint, c_c1_b0_a1_bigint, c_c1_b1_a0_bigint, c_c1_b1_a1_bigint, c_c1_b2_a0_bigint, c_c1_b2_a1_bigint];
        }
    ) {
        panic!("inverseE12Hint: {}", err);
    }
    Ok(())
}
pub fn copy_e12_hint(inputs: &[M31], outputs: &mut [M31]) -> Result<(), Error> {
    if let Err(err) = unwrap_hint(true, true, inputs, outputs, 
        //copyE12Hint
        |inputs| {
            return inputs;
        }
    ) {
        panic!("copyE12Hint: {}", err);
    }
    Ok(())
}
/*

func finalExpHint(nativeMod *big.Int, nativeInputs, nativeOutputs []*big.Int) error {
	// This is inspired from https://eprint.iacr.org/2024/640.pdf
	// and based on a personal communication with the author Andrija Novakovic.
	return emulated.UnwrapHint(nativeInputs, nativeOutputs,
		func(mod *big.Int, inputs, outputs []*big.Int) error {
			startTime := time.Now()
			var millerLoop bls12381.E12

			millerLoop.C0.B0.A0.SetBigInt(inputs[0])
			millerLoop.C0.B0.A1.SetBigInt(inputs[1])
			millerLoop.C0.B1.A0.SetBigInt(inputs[2])
			millerLoop.C0.B1.A1.SetBigInt(inputs[3])
			millerLoop.C0.B2.A0.SetBigInt(inputs[4])
			millerLoop.C0.B2.A1.SetBigInt(inputs[5])
			millerLoop.C1.B0.A0.SetBigInt(inputs[6])
			millerLoop.C1.B0.A1.SetBigInt(inputs[7])
			millerLoop.C1.B1.A0.SetBigInt(inputs[8])
			millerLoop.C1.B1.A1.SetBigInt(inputs[9])
			millerLoop.C1.B2.A0.SetBigInt(inputs[10])
			millerLoop.C1.B2.A1.SetBigInt(inputs[11])

			var root, rootPthInverse, root27thInverse, residueWitness, scalingFactor bls12381.E12
			var order3rd, order3rdPower, exponent, exponentInv, finalExpFactor, polyFactor big.Int
			// polyFactor = (1-x)/3
			polyFactor.SetString("5044125407647214251", 10)
			// finalExpFactor = ((q^12 - 1) / r) / (27 * polyFactor)
			finalExpFactor.SetString("2366356426548243601069753987687709088104621721678962410379583120840019275952471579477684846670499039076873213559162845121989217658133790336552276567078487633052653005423051750848782286407340332979263075575489766963251914185767058009683318020965829271737924625612375201545022326908440428522712877494557944965298566001441468676802477524234094954960009227631543471415676620753242466901942121887152806837594306028649150255258504417829961387165043999299071444887652375514277477719817175923289019181393803729926249507024121957184340179467502106891835144220611408665090353102353194448552304429530104218473070114105759487413726485729058069746063140422361472585604626055492939586602274983146215294625774144156395553405525711143696689756441298365274341189385646499074862712688473936093315628166094221735056483459332831845007196600723053356837526749543765815988577005929923802636375670820616189737737304893769679803809426304143627363860243558537831172903494450556755190448279875942974830469855835666815454271389438587399739607656399812689280234103023464545891697941661992848552456326290792224091557256350095392859243101357349751064730561345062266850238821755009430903520645523345000326783803935359711318798844368754833295302563158150573540616830138810935344206231367357992991289265295323280", 10)

			// 1. get pth-root inverse
			exponent.Mul(&finalExpFactor, big.NewInt(27))
			root.Exp(millerLoop, &exponent)
			if root.IsOne() {
				rootPthInverse.SetOne()
			} else {
				exponentInv.ModInverse(&exponent, &polyFactor)
				exponent.Neg(&exponentInv).Mod(&exponent, &polyFactor)
				rootPthInverse.Exp(root, &exponent)
			}

			// 2.1. get order of 3rd primitive root
			var three big.Int
			three.SetUint64(3)
			exponent.Mul(&polyFactor, &finalExpFactor)
			root.Exp(millerLoop, &exponent)
			if root.IsOne() {
				order3rdPower.SetUint64(0)
			}
			root.Exp(root, &three)
			if root.IsOne() {
				order3rdPower.SetUint64(1)
			}
			root.Exp(root, &three)
			if root.IsOne() {
				order3rdPower.SetUint64(2)
			}
			root.Exp(root, &three)
			if root.IsOne() {
				order3rdPower.SetUint64(3)
			}

			// 2.2. get 27th root inverse
			if order3rdPower.Uint64() == 0 {
				root27thInverse.SetOne()
			} else {
				order3rd.Exp(&three, &order3rdPower, nil)
				exponent.Mul(&polyFactor, &finalExpFactor)
				root.Exp(millerLoop, &exponent)
				exponentInv.ModInverse(&exponent, &order3rd)
				exponent.Neg(&exponentInv).Mod(&exponent, &order3rd)
				root27thInverse.Exp(root, &exponent)
			}

			// 2.3. shift the Miller loop result so that millerLoop * scalingFactor
			// is of order finalExpFactor
			scalingFactor.Mul(&rootPthInverse, &root27thInverse)
			millerLoop.Mul(&millerLoop, &scalingFactor)

			// 3. get the witness residue
			//
			// lambda = q - u, the optimal exponent
			var lambda big.Int
			lambda.SetString("4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129030796414117214202539", 10)
			exponent.ModInverse(&lambda, &finalExpFactor)
			residueWitness.Exp(millerLoop, &exponent)

			// return the witness residue
			residueWitness.C0.B0.A0.BigInt(outputs[0])
			residueWitness.C0.B0.A1.BigInt(outputs[1])
			residueWitness.C0.B1.A0.BigInt(outputs[2])
			residueWitness.C0.B1.A1.BigInt(outputs[3])
			residueWitness.C0.B2.A0.BigInt(outputs[4])
			residueWitness.C0.B2.A1.BigInt(outputs[5])
			residueWitness.C1.B0.A0.BigInt(outputs[6])
			residueWitness.C1.B0.A1.BigInt(outputs[7])
			residueWitness.C1.B1.A0.BigInt(outputs[8])
			residueWitness.C1.B1.A1.BigInt(outputs[9])
			residueWitness.C1.B2.A0.BigInt(outputs[10])
			residueWitness.C1.B2.A1.BigInt(outputs[11])

			// return the scaling factor
			scalingFactor.C0.B0.A0.BigInt(outputs[12])
			scalingFactor.C0.B0.A1.BigInt(outputs[13])
			scalingFactor.C0.B1.A0.BigInt(outputs[14])
			scalingFactor.C0.B1.A1.BigInt(outputs[15])
			scalingFactor.C0.B2.A0.BigInt(outputs[16])
			scalingFactor.C0.B2.A1.BigInt(outputs[17])

			fmt.Println("finalExpHint time: ", time.Since(startTime))
			return nil
		})
}

*/
pub fn final_exp_hint(inputs: &[M31], outputs: &mut [M31]) -> Result<(), Error> {
    if let Err(err) = unwrap_hint(true, true, inputs, outputs, 
        //finalExpHint
        |inputs| {
            let biguint_inputs = inputs.iter().map(|x| x.to_biguint().unwrap()).collect::<Vec<_>>();

            let mut miller_loop = Fq12::default();
            miller_loop.c0.c0.c0 = Fq::from(biguint_inputs[0].clone());
            miller_loop.c0.c0.c1 = Fq::from(biguint_inputs[1].clone());
            miller_loop.c0.c1.c0 = Fq::from(biguint_inputs[2].clone());
            miller_loop.c0.c1.c1 = Fq::from(biguint_inputs[3].clone());
            miller_loop.c0.c2.c0 = Fq::from(biguint_inputs[4].clone());
            miller_loop.c0.c2.c1 = Fq::from(biguint_inputs[5].clone());
            miller_loop.c1.c0.c0 = Fq::from(biguint_inputs[6].clone());
            miller_loop.c1.c0.c1 = Fq::from(biguint_inputs[7].clone());
            miller_loop.c1.c1.c0 = Fq::from(biguint_inputs[8].clone());
            miller_loop.c1.c1.c1 = Fq::from(biguint_inputs[9].clone());
            miller_loop.c1.c2.c0 = Fq::from(biguint_inputs[10].clone());
            miller_loop.c1.c2.c1 = Fq::from(biguint_inputs[11].clone());

            let mut root = Fq12::default();
            let mut root_pth_inverse = Fq12::default();
            let mut root_27th_inverse = Fq12::default();
            let mut residue_witness = Fq12::default();
            let mut scaling_factor = Fq12::default();
            let mut order3rd = BigInt::default();
            let mut order3rd_power = BigInt::default();
            let mut exponent = BigInt::default();
            let mut exponent_inv = BigInt::default();
            let mut final_exp_factor = BigInt::default();
            let mut poly_factor = BigInt::default();
            poly_factor = BigInt::from_str("5044125407647214251").expect("Invalid string for BigInt");
            final_exp_factor= BigInt::from_str("23663564265482436010697539876877090881046217216789624
            1037958312084001927595247157947768484667049903907687321355916284512198921765813
            3790336552276567078487633052653005423051750848782286407340332979263075575489766
            9632519141857670580096833180209658292717379246256123752015450223269084404285227
            1287749455794496529856600144146867680247752423409495496000922763154347141567662
            0753242466901942121887152806837594306028649150255258504417829961387165043999299
            0714448876523755142774777198171759232890191813938037299262495070241219571843401
            7946750210689183514422061140866509035310235319444855230442953010421847307011410
            5759487413726485729058069746063140422361472585604626055492939586602274983146215
            2946257741441563955534055257111436966897564412983652743411893856464990748627126
            8847393609331562816609422173505648345933283184500719660072305335683752674954376
            5815988577005929923802636375670820616189737737304893769679803809426304143627363
            8602435585378311729034944505567551904482798759429748304698558356668154542713894
            3858739973960765639981268928023410302346454589169794166199284855245632629079222
            4091557256350095392859243101357349751064730561345062266850238821755009430903520
            6455233450003267838039353597113187988443687548332953025631581505735406168301388
            10935344206231367357992991289265295323280").expect("Invalid string for BigInt");

            exponent = &final_exp_factor * 27;
            let exp_uint = exponent.to_biguint().unwrap();
            root = miller_loop.pow(exp_uint.to_u64_digits().iter());
            if root.is_one() {
                root_pth_inverse.set_one();
            } else {
                exponent_inv = mod_inverse(&BigInt::from(exponent.clone()), &poly_factor).unwrap();
                exponent = -exponent_inv;
                exponent = exponent % &poly_factor;
                let exp_uint = exponent.to_biguint().unwrap();
                root_pth_inverse = root.pow(exp_uint.to_u64_digits().iter());
            }

            let mut three = BigUint::from(3u32);
            exponent = &poly_factor * &final_exp_factor;
            let exp_uint = exponent.to_biguint().unwrap();
            root = miller_loop.pow(exp_uint.to_u64_digits().iter());
            if root.is_one() {
                order3rd_power = BigInt::from(0u32);
            }
            root = root.pow(three.to_u64_digits().iter());
            if root.is_one() {
                order3rd_power = BigInt::from(1u32);
            }
            root = root.pow(three.to_u64_digits().iter());
            if root.is_one() {
                order3rd_power = BigInt::from(2u32);
            }
            root = root.pow(three.to_u64_digits().iter());
            if root.is_one() {
                order3rd_power = BigInt::from(3u32);
            }

            if order3rd_power.is_zero() {
                root_27th_inverse.set_one();
            } else {
                let three_bigint = BigInt::from(3u32);
                order3rd = three_bigint.pow(order3rd_power.to_u32().unwrap());
                exponent = &poly_factor * &final_exp_factor;
                let exp_uint = exponent.to_biguint().unwrap();
                root = miller_loop.pow(exp_uint.to_u64_digits().iter());
                exponent_inv = mod_inverse(&exponent, &order3rd).unwrap();
                exponent = -exponent_inv;
                exponent = exponent % &order3rd;
                let exp_uint = exponent.to_biguint().unwrap();
                root_27th_inverse = root.pow(exp_uint.to_u64_digits().iter());
            }

            scaling_factor = root_pth_inverse * root_27th_inverse;
            miller_loop = miller_loop * scaling_factor;

            let mut lambda = BigInt::default();
            lambda = BigInt::from_str("4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129030796414117214202539").expect("Invalid string for BigInt");
            exponent = mod_inverse(&lambda, &final_exp_factor).unwrap();
            residue_witness = miller_loop.pow(exponent.to_biguint().unwrap().to_u64_digits().iter());

            let c_c0_b0_a0_bigint = residue_witness.c0.c0.c0.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c0_b0_a1_bigint = residue_witness.c0.c0.c1.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c0_b1_a0_bigint = residue_witness.c0.c1.c0.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c0_b1_a1_bigint = residue_witness.c0.c1.c1.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c0_b2_a0_bigint = residue_witness.c0.c2.c0.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c0_b2_a1_bigint = residue_witness.c0.c2.c1.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c1_b0_a0_bigint = residue_witness.c1.c0.c0.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c1_b0_a1_bigint = residue_witness.c1.c0.c1.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c1_b1_a0_bigint = residue_witness.c1.c1.c0.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c1_b1_a1_bigint = residue_witness.c1.c1.c1.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c1_b2_a0_bigint = residue_witness.c1.c2.c0.to_string().parse::<BigInt>().expect("Invalid decimal string");
            let c_c1_b2_a1_bigint = residue_witness.c1.c2.c1.to_string().parse::<BigInt>().expect("Invalid decimal string");

            return vec![c_c0_b0_a0_bigint, c_c0_b0_a1_bigint, c_c0_b1_a0_bigint, c_c0_b1_a1_bigint, c_c0_b2_a0_bigint, c_c0_b2_a1_bigint, c_c1_b0_a0_bigint, c_c1_b0_a1_bigint, c_c1_b1_a0_bigint, c_c1_b1_a1_bigint, c_c1_b2_a0_bigint, c_c1_b2_a1_bigint];
        }
    ) {
        panic!("inverseE12Hint: {}", err);
    }
    Ok(())
}

// fn from_u64_array_to_bigint_le(array: &[u64; 6]) -> BigInt {
//     let bytes: Vec<u8> = array.iter().flat_map(|&n| n.to_le_bytes()).collect();

//     BigInt::from_bytes_le(Sign::Plus, &bytes)
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


pub fn unwrap_hint(is_emulated_input: bool, is_emulated_output: bool, native_inputs: &[M31], native_outputs: &mut [M31], nonnative_hint: fn(Vec<BigInt>) -> Vec<BigInt>, ) -> Result<(), String> {
    if native_inputs.len() < 2 {
        return Err("hint wrapper header is 2 elements".to_string());
    }
    let i64_max = 1 << 63;
    if native_inputs[0].to_u256() >= i64_max || native_inputs[1].to_u256() >= i64_max {
        return Err("header must be castable to int64".to_string());
    }
    let nb_bits = native_inputs[0].to_u256().as_u32();
    let nb_limbs = native_inputs[1].to_u256().as_usize();
    if native_inputs.len() < 2 + nb_limbs {
        return Err("hint wrapper header is 2+nbLimbs elements".to_string());
    }
    let nonnative_mod_limbs = m31_to_bigint_array(native_inputs[2..2+nb_limbs].to_vec().as_slice());
    let nonnative_mod = recompose(nonnative_mod_limbs, nb_bits);
    let mut nonnative_inputs = vec![];
    if is_emulated_input {
        if native_inputs[2+nb_limbs].to_u256() >= i64_max {
            return Err("number of nonnative elements must be castable to int64".to_string());
        }
        let nb_inputs = native_inputs[2+nb_limbs].to_u256().as_usize();
        let mut read_ptr = 3 + nb_limbs;
        nonnative_inputs = vec![BigInt::default(); nb_inputs];
        for i in 0..nb_inputs {
            if native_inputs.len() < read_ptr + 1 {
                return Err(format!("can not read {}-th native input", i));
            }
            if native_inputs[read_ptr].to_u256() >= i64_max {
                return Err(format!("corrupted {}-th native input", i));
            }
            let current_input_len = native_inputs[read_ptr].to_u256().as_usize();
            if native_inputs.len() < read_ptr + 1 + current_input_len {
                return Err(format!("cannot read {}-th nonnative element", i));
            }
            let tmp_inputs = m31_to_bigint_array(native_inputs[read_ptr+1..read_ptr+1+current_input_len].to_vec().as_slice());
            println!("tmp_inputs: {:?}", tmp_inputs);
            nonnative_inputs[i] = recompose(tmp_inputs, nb_bits);
            println!("composed: {:?}", nonnative_inputs[i]);
            read_ptr += 1 + current_input_len;
        }
    } else {
        let nb_inputs = native_inputs[2+nb_limbs..].len();
        let mut read_ptr = 2 + nb_limbs;
        nonnative_inputs = vec![BigInt::default(); nb_inputs];
        for i in 0..nb_inputs {
            nonnative_inputs[i] = m31_to_bigint(native_inputs[read_ptr+i]);
        }
    }
    let mut nonnative_outputs = vec![BigInt::default(); native_outputs.len()];
    nonnative_outputs = nonnative_hint(nonnative_inputs);
    let mut tmp_outputs = vec![BigInt::default(); nb_limbs * nonnative_outputs.len()];
    if is_emulated_output {
        if native_outputs.len() % nb_limbs != 0 {
            return Err("output count doesn't divide limb count".to_string());
        }
        for i in 0..nonnative_outputs.len() {
            let mod_output = &nonnative_outputs[i] % &nonnative_mod;
            if let Err(e) = decompose(&mod_output, nb_bits, &mut tmp_outputs[i*nb_limbs..(i+1)*nb_limbs]) {
                return Err(format!("decompose {}-th element: {}", i, e));
            }
        }
    } else {
        for i in 0..nonnative_outputs.len() {
            tmp_outputs[i] = nonnative_outputs[i].clone();
        }
    }
    for i in 0..tmp_outputs.len() {
        native_outputs[i] = bigint_to_m31(&tmp_outputs[i]);
    }
    Ok(())
}

//TBD
fn mod_inverse(g: &BigInt, n: &BigInt) -> Option<BigInt> {
    let zero = BigInt::zero();
    let one = BigInt::one();

    let (mut old_r, mut r) = (n.clone(), g.clone());
    let (mut old_t, mut t) = (zero.clone(), one.clone());

    while !r.is_zero() {
        let quotient = &old_r / &r;

        let new_r = &old_r - &quotient * &r;
        old_r = r;
        r = new_r;

        let new_t = &old_t - &quotient * &t;
        old_t = t;
        t = new_t;
    }

    if old_r != one {
        None
    } else {
        Some((old_t % n + n) % n)
    }
}