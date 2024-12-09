use halo2curves::bn256::Fr;

// we use 18 limbs, each with 120 bits, to store a 2048 bit integer
pub const N_LIMBS: usize = 18;
// // 2048 bits = 256 bytes
// pub(crate) const N_BYTES: usize = 256;
// Each 120 bits limb needs 30 hex number to store
pub(crate) const HEX_PER_LIMB: usize = 30;

// 2^120 - 1
pub(crate) const MASK120: u128 = (1 << 120) - 1;
// 2^60 - 1
pub(crate) const MASK60: u128 = (1 << 60) - 1;
// 2^8 - 1
pub(crate) const MASK8: u128 = (1 << 8) - 1;
// 2^120 in Fr
pub const BN_TWO_TO_120: Fr = Fr::from_raw([0, 1 << 56, 0, 0]);
