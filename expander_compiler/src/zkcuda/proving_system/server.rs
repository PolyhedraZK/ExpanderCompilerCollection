use serde::{Deserialize, Serialize};

pub static SERVER_URL: &str = "http://127.0.0.1:3000/";

#[derive(Serialize, Deserialize)]
pub enum RequestType {
    CommitInput(usize), // Parallelizaion count
    Prove(usize, usize), // (Parallelization count, kernel_id)
    Verify(usize, usize), // (Parallelization count, kernel_id)
    Exit,
}
