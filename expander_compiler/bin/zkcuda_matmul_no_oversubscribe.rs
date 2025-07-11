#![allow(unused)]
mod zkcuda_matmul;
use expander_compiler::{
    frontend::BN254Config,
    zkcuda::proving_system::{
        expander_pcs_defered::BN254ConfigSha2UniKZG, ExpanderNoOverSubscribe,
    },
};
use zkcuda_matmul::zkcuda_matmul;

fn main() {
    zkcuda_matmul::<BN254Config, ExpanderNoOverSubscribe<BN254ConfigSha2UniKZG>, 4>();
    zkcuda_matmul::<BN254Config, ExpanderNoOverSubscribe<BN254ConfigSha2UniKZG>, 8>();
    zkcuda_matmul::<BN254Config, ExpanderNoOverSubscribe<BN254ConfigSha2UniKZG>, 16>();

    zkcuda_matmul::<BN254Config, ExpanderNoOverSubscribe<BN254ConfigSha2UniKZG>, 1024>();
}
