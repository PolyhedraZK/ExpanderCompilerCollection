mod cg_def;
use expander_compiler::zkcuda::{
    context::{ComputationGraph, ComputationGraphDefine},
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
    let input = CG::get_input();
    let (_computation_graph, extended_witness) = CG::gen_computation_graph_and_witness(Some(input));

    // Note: we've saved the computation graph and setup in the server. In order to generate a proof, we only need to submit the witness.
    let dummy_prover_setup =
        <ExpanderNoOverSubscribe<ZC> as ProvingSystem<ZC::ECCConfig>>::ProverSetup::default();
    let dummy_computation_graph = ComputationGraph::<ZC::ECCConfig>::default();

    let proof = ExpanderNoOverSubscribe::<ZC>::prove(
        &dummy_prover_setup,
        &dummy_computation_graph,
        extended_witness.unwrap(),
    );

    let mut bytes = vec![];
    proof.serialize_into(&mut bytes).unwrap();
    std::fs::write("/tmp/proof.bin", &bytes).unwrap();
}

fn main() {
    main_impl::<ZKCudaBN254KZG, MyCGDef>();
}
