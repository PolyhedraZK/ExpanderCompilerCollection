mod cg_def;
use std::io::Cursor;

use expander_compiler::zkcuda::{
    context::ComputationGraphDefine,
    proving_system::{
        expander::config::{GetFieldConfig, GetPCS, ZKCudaBN254KZG, ZKCudaConfig},
        ExpanderNoOverSubscribe, ProvingSystem,
    },
};
use gkr_engine::ExpanderPCS;
use serdes::ExpSerde;

use cg_def::MyCGDef;

fn main_impl<ZC: ZKCudaConfig, CG: ComputationGraphDefine<ZC::ECCConfig>>()
where
    <GetPCS<ZC> as ExpanderPCS<GetFieldConfig<ZC>>>::Commitment:
        AsRef<<GetPCS<ZC> as ExpanderPCS<GetFieldConfig<ZC>>>::Commitment>,
{
    let (computation_graph, _) = CG::gen_computation_graph_and_witness(None);

    let verifier_setup_bytes = std::fs::read("/tmp/verifier_setup.bin").unwrap();
    let verifier_setup = <ExpanderNoOverSubscribe<ZC> as ProvingSystem<
        ZC::ECCConfig,
    >>::VerifierSetup::deserialize_from(Cursor::new(verifier_setup_bytes))
    .unwrap();

    let proof_bytes = std::fs::read("/tmp/proof.bin").unwrap();
    let proof =
        <ExpanderNoOverSubscribe<ZC> as ProvingSystem<ZC::ECCConfig>>::Proof::deserialize_from(
            Cursor::new(proof_bytes),
        )
        .unwrap();

    let verified = <ExpanderNoOverSubscribe<ZC> as ProvingSystem<ZC::ECCConfig>>::verify(
        &verifier_setup,
        &computation_graph,
        &proof,
    );
    assert!(verified, "Proof verification failed");
}

fn main() {
    main_impl::<ZKCudaBN254KZG, MyCGDef>();
}
