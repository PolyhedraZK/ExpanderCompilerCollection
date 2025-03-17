use super::kernel::Kernel;

use crate::circuit::config::Config;

#[derive(Clone)]
pub struct ProofTemplate {
    pub kernel_id: usize,
    pub commitment_indices: Vec<usize>,
    pub parallel_count: usize,
    pub is_broadcast: Vec<bool>,
}

pub struct ComputationGraph<C: Config> {
    pub kernels: Vec<Kernel<C>>,
    pub commitments_lens: Vec<usize>,
    pub proof_templates: Vec<ProofTemplate>,
}
