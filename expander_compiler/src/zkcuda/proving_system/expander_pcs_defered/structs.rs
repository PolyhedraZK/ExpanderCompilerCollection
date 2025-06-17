use gkr_engine::GKREngine;
use serdes::ExpSerde;

use crate::zkcuda::proving_system::expander::structs::{ExpanderCommitment, ExpanderProof};


#[derive(ExpSerde)]
pub struct KernelWiseProofPCSDefered<C: GKREngine> {
    pub commitments: Vec<ExpanderCommitment<C::PCSField, C::FieldConfig, C::PCSConfig>>,
    pub proofs: Vec<ExpanderProof>, // Multiple proofs for each kernel
    pub defered_pcs_opening: ExpanderProof,
}

// It's so weird that this can not be derived automatically
impl<C: GKREngine> Clone for KernelWiseProofPCSDefered<C> {
    fn clone(&self) -> Self {
        Self {
            commitments: self.commitments.clone(),
            proofs: self.proofs.clone(),
            defered_pcs_opening: self.defered_pcs_opening.clone(),
        }
    }
}
