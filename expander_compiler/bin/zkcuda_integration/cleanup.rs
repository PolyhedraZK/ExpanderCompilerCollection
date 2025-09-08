use expander_compiler::{
    frontend::BN254Config,
    zkcuda::proving_system::{
        expander::config::ZKCudaBN254MIMCKZGBatchPCS, ExpanderNoOverSubscribe, ProvingSystem,
    },
};

fn main() {
    // The exact config doesn't matter for post_process, it sends a http request to the server, asking for shut down.
    <ExpanderNoOverSubscribe<ZKCudaBN254MIMCKZGBatchPCS> as ProvingSystem<BN254Config>>::post_process();
}
