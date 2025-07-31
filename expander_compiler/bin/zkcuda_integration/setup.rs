mod cg_def;
use expander_compiler::zkcuda::{
    context::ComputationGraphDefine,
    proving_system::{
        expander::config::{GetFieldConfig, GetPCS, ZKCudaBN254KZG, ZKCudaConfig},
        ExpanderNoOverSubscribe, ProvingSystem,
    },
};
use gkr_engine::ExpanderPCS;
use serdes::ExpSerde;

use crate::cg_def::MyCGDef;

fn main_impl<ZC: ZKCudaConfig, CG: ComputationGraphDefine<ZC::ECCConfig>>()
where
    <GetPCS<ZC> as ExpanderPCS<GetFieldConfig<ZC>>>::Commitment:
        AsRef<<GetPCS<ZC> as ExpanderPCS<GetFieldConfig<ZC>>>::Commitment>,
{
    let (computation_graph, _) = CG::gen_computation_graph_and_witness(None);
    let (prover_setup, verifier_setup) = ExpanderNoOverSubscribe::<ZC>::setup(&computation_graph);

    let mut bytes = vec![];
    prover_setup.serialize_into(&mut bytes).unwrap();
    std::fs::write("/tmp/prover_setup.bin", &bytes).unwrap();

    bytes.clear();
    verifier_setup.serialize_into(&mut bytes).unwrap();
    std::fs::write("/tmp/verifier_setup.bin", &bytes).unwrap();
}

fn main() {
    main_impl::<ZKCudaBN254KZG, MyCGDef>();
}
