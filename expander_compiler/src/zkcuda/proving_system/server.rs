use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum RequestType {
    PCSSetup(usize, usize), // (local_val_len, mpi_world_size)
    RegisterKernel,
    CommitInput,
    Prove(usize), // kernel_id
    Exit,
}
