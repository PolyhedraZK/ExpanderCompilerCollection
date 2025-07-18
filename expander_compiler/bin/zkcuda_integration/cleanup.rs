use expander_compiler::{
    frontend::BN254Config,
    zkcuda::proving_system::{
        expander::config::ZKCudaBN254Hyrax, ExpanderNoOverSubscribe, ProvingSystem,
    },
};

fn main() {
    <ExpanderNoOverSubscribe<ZKCudaBN254Hyrax> as ProvingSystem<BN254Config>>::post_process();
}
