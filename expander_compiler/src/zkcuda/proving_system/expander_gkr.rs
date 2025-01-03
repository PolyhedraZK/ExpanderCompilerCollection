use crate::circuit::config::Config;
use crate::field::FieldArith;

use super::{Commitment, ProvingSystem, Proof};
use super::super::kernel::Kernel;

use poly_commit::PCSForExpanderGKR;
use expander_field_config::GKRFieldConfig;
use expander_transcript::Transcript;

pub struct ExpanderGKRCommitment<C: GKRFieldConfig, T: Transcript<C::ChallengeField>, PCS: PCSForExpanderGKR<C, T>> {
    commitment: PCS::Commitment,
    scratch: PCS::ScratchPad, // used to store extra data for the commitment, such as the Merkle tree
}