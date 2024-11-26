use halo2curves::bn256::Fr;

pub(crate) const N_LIMBS: usize = 18;
pub(crate) const MASK120: u128 = (1 << 120) - 1;
pub(crate) const MASK60: u128 = (1 << 60) - 1;
pub(crate) const MASK8: u128 = (1 << 8) - 1;
pub(crate) const HEX_PER_LIMB: usize = 30;
pub(crate) const BN_TWO_TO_120: Fr = Fr::from_raw([0, 1 << 56, 0, 0]);
