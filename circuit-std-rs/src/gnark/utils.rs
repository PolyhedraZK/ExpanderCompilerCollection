use num_bigint::BigInt;

use crate::gnark::limbs::recompose;
use crate::gnark::limbs::decompose;

pub fn nb_multiplication_res_limbs(len_left: usize, len_right: usize) -> usize {
    let res = len_left + len_right - 1;
    if res < 0 {
        0
    } else {
        res
    }
}

/*
func subPadding(modulus *big.Int, bitsPerLimbs uint, overflow uint, nbLimbs uint) []*big.Int {
	if modulus.Cmp(big.NewInt(0)) == 0 {
		panic("modulus is zero")
	}
	// first, we build a number nLimbs, such that nLimbs > b;
	// here b is defined by its bounds, that is b is an element with nbLimbs of (bitsPerLimbs+overflow)
	// so a number nLimbs > b, is simply taking the next power of 2 over this bound .
	nLimbs := make([]*big.Int, nbLimbs)
	for i := 0; i < len(nLimbs); i++ {
		nLimbs[i] = new(big.Int).SetUint64(1)
		nLimbs[i].Lsh(nLimbs[i], overflow+bitsPerLimbs)
	}

	// recompose n as the sum of the coefficients weighted by the limbs
	n := new(big.Int)
	if err := limbs.Recompose(nLimbs, bitsPerLimbs, n); err != nil {
		panic(fmt.Sprintf("recompose: %v", err))
	}
	fmt.Println("n", n)
	// mod reduce n, and negate it
	n.Mod(n, modulus)
	n.Sub(modulus, n)

	// construct pad such that:
	// pad := n - neg(n mod p) == kp
	pad := make([]*big.Int, nbLimbs)
	for i := range pad {
		pad[i] = new(big.Int)
	}
	if err := limbs.Decompose(n, bitsPerLimbs, pad); err != nil {
		panic(fmt.Sprintf("decompose: %v", err))
	}
	for i := range pad {
		pad[i].Add(pad[i], nLimbs[i])
	}
	return pad
}
*/
pub fn sub_padding(modulus: &BigInt, bits_per_limbs: u32, overflow: u32, nb_limbs: u32) -> Vec<BigInt> {
    if modulus == &BigInt::default() {
        panic!("modulus is zero");
    }
    let mut n_limbs = vec![BigInt::default(); nb_limbs as usize];
    for i in 0..n_limbs.len() {
        n_limbs[i] = BigInt::from(1) << (overflow + bits_per_limbs);
    }
    let mut n = recompose(n_limbs.clone(), bits_per_limbs);
    n = n % modulus;
    n = modulus - n;
    let mut pad = vec![BigInt::default(); nb_limbs as usize];
    if let Err(err) = decompose(&n, bits_per_limbs, &mut pad) {
        panic!("decompose: {}", err);
    }
    let mut new_pad = vec![BigInt::default(); nb_limbs as usize];
    for i in 0..pad.len() {
        new_pad[i] = pad[i].clone() + n_limbs[i].clone();
    }
    new_pad
}

