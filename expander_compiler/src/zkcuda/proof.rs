use serdes::ExpSerde;

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

impl ExpSerde for ProofTemplate {
    const SERIALIZED_SIZE: usize = unimplemented!();
    fn serialize_into<W: std::io::Write>(&self, mut writer: W) -> serdes::SerdeResult<()> {
        self.kernel_id.serialize_into(&mut writer)?;
        self.commitment_indices.serialize_into(&mut writer)?;
        self.parallel_count.serialize_into(&mut writer)?;
        self.is_broadcast.serialize_into(&mut writer)?;
        Ok(())
    }
    fn deserialize_from<R: std::io::Read>(mut reader: R) -> serdes::SerdeResult<Self> {
        let kernel_id = usize::deserialize_from(&mut reader)?;
        let commitment_indices = Vec::<usize>::deserialize_from(&mut reader)?;
        let parallel_count = usize::deserialize_from(&mut reader)?;
        let is_broadcast = Vec::<bool>::deserialize_from(&mut reader)?;
        Ok(ProofTemplate {
            kernel_id,
            commitment_indices,
            parallel_count,
            is_broadcast,
        })
    }
}

impl<C: Config> ExpSerde for ComputationGraph<C> {
    const SERIALIZED_SIZE: usize = unimplemented!();
    fn serialize_into<W: std::io::Write>(&self, mut writer: W) -> serdes::SerdeResult<()> {
        self.kernels.serialize_into(&mut writer)?;
        self.commitments_lens.serialize_into(&mut writer)?;
        self.proof_templates.serialize_into(&mut writer)?;
        Ok(())
    }
    fn deserialize_from<R: std::io::Read>(mut reader: R) -> serdes::SerdeResult<Self> {
        let kernels = Vec::<Kernel<C>>::deserialize_from(&mut reader)?;
        let commitments_lens = Vec::<usize>::deserialize_from(&mut reader)?;
        let proof_templates = Vec::<ProofTemplate>::deserialize_from(&mut reader)?;
        Ok(ComputationGraph {
            kernels,
            commitments_lens,
            proof_templates,
        })
    }
}
