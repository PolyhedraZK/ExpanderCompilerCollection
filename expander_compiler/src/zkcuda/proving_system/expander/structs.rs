use std::collections::HashMap;

use arith::Field;
use gkr_engine::{ExpanderPCS, FieldEngine, Proof as BytesProof, StructuredReferenceString};
use serdes::ExpSerde;

use crate::{frontend::Config, zkcuda::proving_system::Commitment};

/// A wrapper for the PCS Commitment that includes the length of the values committed to.
#[allow(clippy::type_complexity)]
#[derive(ExpSerde)]
pub struct ExpanderCommitment<PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> {
    pub vals_len: usize,
    pub commitment: PCS::Commitment,
}

impl<PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> Clone
    for ExpanderCommitment<PCSField, F, PCS>
{
    fn clone(&self) -> Self {
        Self {
            vals_len: self.vals_len,
            commitment: self.commitment.clone(),
        }
    }
}

impl<
        F: FieldEngine,
        PCS: ExpanderPCS<F, F::SimdCircuitField>,
        ECCConfig: Config<FieldConfig = F>,
    > Commitment<ECCConfig> for ExpanderCommitment<F::SimdCircuitField, F, PCS>
{
    fn vals_len(&self) -> usize {
        self.vals_len
    }
}

/// Used for stateful PCS such as Orion, where the PCS needs to maintain some state after commitment.
/// For Raw, KZG, and Hyrax, this is not needed, so the scratchpad can be empty.
#[allow(clippy::type_complexity)]
#[derive(ExpSerde)]
pub struct ExpanderCommitmentState<PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> {
    pub scratch: PCS::ScratchPad,
}

impl<PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> Clone
    for ExpanderCommitmentState<PCSField, F, PCS>
{
    fn clone(&self) -> Self {
        Self {
            scratch: self.scratch.clone(),
        }
    }
}

/// The prover setup contains the public keys for the prover, which are used to commit to the values.
/// The keys are indexed by the length of values committed to, allowing for different setups based on the length of the values.
#[allow(clippy::type_complexity)]
#[derive(ExpSerde)]
pub struct ExpanderProverSetup<PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> {
    pub p_keys: HashMap<usize, <PCS::SRS as StructuredReferenceString>::PKey>,
}

impl<PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> Default
    for ExpanderProverSetup<PCSField, F, PCS>
{
    fn default() -> Self {
        Self {
            p_keys: HashMap::new(),
        }
    }
}

impl<PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> Clone
    for ExpanderProverSetup<PCSField, F, PCS>
{
    fn clone(&self) -> Self {
        Self {
            p_keys: self.p_keys.clone(),
        }
    }
}

/// The verifier setup contains the verification keys for the verifier, which are used to verify the proofs.
/// The keys are indexed by the length of values committed to, allowing for different setups based on the length of the values.
#[allow(clippy::type_complexity)]
#[derive(ExpSerde)]
pub struct ExpanderVerifierSetup<PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> {
    pub v_keys: HashMap<usize, <PCS::SRS as StructuredReferenceString>::VKey>,
}

// implement default
impl<PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> Default
    for ExpanderVerifierSetup<PCSField, F, PCS>
{
    fn default() -> Self {
        Self {
            v_keys: HashMap::new(),
        }
    }
}

impl<PCSField: Field, F: FieldEngine, PCS: ExpanderPCS<F, PCSField>> Clone
    for ExpanderVerifierSetup<PCSField, F, PCS>
{
    fn clone(&self) -> Self {
        Self {
            v_keys: self.v_keys.clone(),
        }
    }
}

#[derive(Clone, ExpSerde)]
pub struct ExpanderProof {
    pub data: Vec<BytesProof>,
}
