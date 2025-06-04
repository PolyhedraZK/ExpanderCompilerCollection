use serdes::ExpSerde;

use super::super::{kernel::Kernel, proof::ComputationGraph};

use crate::{
    circuit::config::{Config, SIMDField}, zkcuda::context::DeviceMemory,
};

pub trait Commitment<C: Config>: Clone + ExpSerde {
    fn vals_len(&self) -> usize;
}

pub trait KernelWiseProvingSystem<C: Config> {
    type ProverSetup: Clone + Send + Sync + ExpSerde;
    type VerifierSetup: Clone + Send + Sync + ExpSerde;
    type Proof: Clone + Send + Sync + ExpSerde;
    type Commitment: Commitment<C> + Send + Sync + ExpSerde;
    type CommitmentExtraInfo: Clone + Send + Sync + ExpSerde;

    fn setup(computation_graph: &ComputationGraph<C>) -> (Self::ProverSetup, Self::VerifierSetup);

    fn commit(
        prover_setup: &Self::ProverSetup,
        vals: &[SIMDField<C>],
        parallel_count: usize,
        is_broadcast: bool,
    ) -> (Self::Commitment, Self::CommitmentExtraInfo);

    #[allow(clippy::too_many_arguments)]
    fn prove_kernel(
        prover_setup: &Self::ProverSetup,
        kernel_id: usize,
        kernel: &Kernel<C>,
        commitments: &[Self::Commitment],
        commitments_extra_info: &[Self::CommitmentExtraInfo],
        commitments_values: &[&[SIMDField<C>]],
        parallel_count: usize,
        is_broadcast: &[bool],
    ) -> Self::Proof;

    fn verify_kernel(
        verifier_setup: &Self::VerifierSetup,
        kernel_id: usize,
        kernel: &Kernel<C>,
        proof: &Self::Proof,
        commitments: &[Self::Commitment],
        parallel_count: usize,
        is_broadcast: &[bool],
    ) -> bool;

    fn post_process() {}
}

#[derive(ExpSerde)]
pub struct CombinedProof<C: Config, KP: KernelWiseProvingSystem<C>> {
    pub commitments: Vec<Vec<KP::Commitment>>, // a vector of commitments for each kernel
    pub proofs: Vec<KP::Proof>,
}

impl<C: Config, KP: KernelWiseProvingSystem<C>> Clone for CombinedProof<C, KP> {
    fn clone(&self) -> Self {
        CombinedProof {
            commitments: self.commitments.clone(),
            proofs: self.proofs.clone(),
        }
    }
}

pub trait ProvingSystem<C: Config> {
    type ProverSetup: Clone + Send + Sync + ExpSerde;
    type VerifierSetup: Clone + Send + Sync + ExpSerde;
    type Proof: Clone + Send + Sync + ExpSerde;

    fn setup(computation_graph: &ComputationGraph<C>) -> (Self::ProverSetup, Self::VerifierSetup);

    fn prove(
        prover_setup: &Self::ProverSetup,
        computation_graph: &ComputationGraph<C>,
        device_memories: &[DeviceMemory<C>],
    ) -> Self::Proof;

    fn verify(
        verifier_setup: &Self::VerifierSetup,
        computation_graph: &ComputationGraph<C>,
        proof: &Self::Proof,
    ) -> bool;

    /// This is a dedicated function to stop the running service
    /// For most proving systems, this is a no-op
    fn post_process() {}
}

#[macro_export]
macro_rules! impl_proving_system_for_kernel_wise_proving_system {
    ($kp:ty,$generics:tt) => {
        impl<$($generics)*> ProvingSystem<C> for $kp {
            type ProverSetup = <$kp as KernelWiseProvingSystem<C>>::ProverSetup;
            type VerifierSetup = <$kp as KernelWiseProvingSystem<C>>::VerifierSetup;
            type Proof = CombinedProof<C, $kp>;

            fn setup(computation_graph: &ComputationGraph<C>) -> (Self::ProverSetup, Self::VerifierSetup) {
                <$kp as KernelWiseProvingSystem<C>>::setup(computation_graph)
            }

            fn prove(
                prover_setup: &Self::ProverSetup,
                computation_graph: &ComputationGraph<C>,
                device_memories: &[DeviceMemory<C>],
            ) -> Self::Proof {
                let commitments = computation_graph
                    .proof_templates
                    .iter()
                    .map(|template| {
                        template
                            .commitment_indices
                            .iter()
                            .zip(template.is_broadcast.iter())
                            .map(|(x, is_broadcast)| {
                                <$kp as KernelWiseProvingSystem<C>>::commit(
                                    prover_setup,
                                    &device_memories[*x].values,
                                    next_power_of_two(template.parallel_count),
                                    *is_broadcast,
                                )
                            })
                            .unzip::<_, _, Vec<_>, Vec<_>>()
                    })
                    .collect::<Vec<_>>();

                let proofs: Vec<<$kp as KernelWiseProvingSystem<C>>::Proof> = computation_graph
                    .proof_templates
                    .iter()
                    .zip(commitments.iter())
                    .map(|(template, commitments_kernel)| {
                        <$kp as KernelWiseProvingSystem<C>>::prove_kernel(
                            prover_setup,
                            template.kernel_id,
                            &computation_graph.kernels[template.kernel_id],
                            &commitments_kernel.0,
                            &commitments_kernel.1,
                            &template
                                .commitment_indices
                                .iter()
                                .map(|x| &device_memories[*x].values[..])
                                .collect::<Vec<_>>(),
                            next_power_of_two(template.parallel_count),
                            &template.is_broadcast,
                        )
                    })
                    .collect::<Vec<_>>();

                CombinedProof {
                    commitments: commitments.into_iter().map(|x| x.0).collect(),
                    proofs,
                }
            }

            fn verify(
                verifier_setup: &Self::VerifierSetup,
                computation_graph: &ComputationGraph<C>,
                proof: &Self::Proof,
            ) -> bool {
                let verified = proof
                    .proofs
                    .par_iter()
                    .zip(computation_graph.proof_templates.par_iter())
                    .zip(proof.commitments.par_iter())
                    .map(|((proof, template), commitments_kernel)| {
                        <$kp as KernelWiseProvingSystem<C>>::verify_kernel(
                            verifier_setup,
                            template.kernel_id,
                            &computation_graph.kernels[template.kernel_id],
                            proof,
                            commitments_kernel,
                            next_power_of_two(template.parallel_count),
                            &template.is_broadcast,
                        )
                    })
                    .collect::<Vec<_>>();

                verified.iter().all(|x| *x)
            }

            fn post_process() {
                <$kp as KernelWiseProvingSystem<C>>::post_process();
            }
        }
    };
    () => {
        
    };
}