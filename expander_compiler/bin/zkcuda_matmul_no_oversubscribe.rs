#![allow(unused)]
mod zkcuda_matmul;
use expander_compiler::{
    frontend::BN254Config,
    zkcuda::proving_system::ExpanderNoOverSubscribe,
};
use gkr::{BN254ConfigSha2Hyrax, BN254ConfigSha2KZG};
use zkcuda_matmul::zkcuda_matmul;

fn main() {
    zkcuda_matmul::<BN254Config, ExpanderNoOverSubscribe<BN254ConfigSha2KZG>, 4>();
    zkcuda_matmul::<BN254Config, ExpanderNoOverSubscribe<BN254ConfigSha2KZG>, 8>();
    zkcuda_matmul::<BN254Config, ExpanderNoOverSubscribe<BN254ConfigSha2KZG>, 16>();

    zkcuda_matmul::<BN254Config, ExpanderNoOverSubscribe<BN254ConfigSha2KZG>, 1024>();
}
