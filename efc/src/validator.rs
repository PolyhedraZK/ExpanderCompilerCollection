use circuit_std_rs::poseidon_m31::*;
use circuit_std_rs::utils::register_hint;
use expander_compiler::frontend::*;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::utils::*;
pub const SUBTREE_DEPTH: usize = 10;
pub const SUBTREE_NUM_DEPTH: usize = 11;
pub const SUBTREE_SIZE: usize = 1 << SUBTREE_DEPTH;
pub const SUBTREE_NUM: usize = 1 << SUBTREE_NUM_DEPTH;
#[derive(Debug, Deserialize, Clone)]
pub struct ValidatorPlain {
    #[serde(default)]
    pub public_key: String,
    #[serde(default)]
    pub withdrawal_credentials: String,
    #[serde(default)]
    pub effective_balance: u64,
    #[serde(default)]
    pub slashed: bool,
    #[serde(default)]
    pub activation_eligibility_epoch: u64,
    #[serde(default)]
    pub activation_epoch: u64,
    #[serde(default)]
    pub exit_epoch: u64,
    #[serde(default)]
    pub withdrawable_epoch: u64,
}

#[derive(Clone, Copy)]
pub struct ValidatorSSZ {
    pub public_key: [Variable; 48],
    pub withdrawal_credentials: [Variable; 32],
    pub effective_balance: [Variable; 8],
    pub slashed: [Variable; 1],
    pub activation_eligibility_epoch: [Variable; 8],
    pub activation_epoch: [Variable; 8],
    pub exit_epoch: [Variable; 8],
    pub withdrawable_epoch: [Variable; 8],
}
impl Default for ValidatorSSZ {
    fn default() -> Self {
        Self {
            public_key: [Variable::default(); 48],
            withdrawal_credentials: [Variable::default(); 32],
            effective_balance: [Variable::default(); 8],
            slashed: [Variable::default(); 1],
            activation_eligibility_epoch: [Variable::default(); 8],
            activation_epoch: [Variable::default(); 8],
            exit_epoch: [Variable::default(); 8],
            withdrawable_epoch: [Variable::default(); 8],
        }
    }
}
impl ValidatorSSZ {
    pub fn new() -> Self {
        Self {
            public_key: [Variable::default(); 48],
            withdrawal_credentials: [Variable::default(); 32],
            effective_balance: [Variable::default(); 8],
            slashed: [Variable::default(); 1],
            activation_eligibility_epoch: [Variable::default(); 8],
            activation_epoch: [Variable::default(); 8],
            exit_epoch: [Variable::default(); 8],
            withdrawable_epoch: [Variable::default(); 8],
        }
    }
    pub fn hash<C: Config, B: RootAPI<C>>(&self, builder: &mut B) -> Vec<Variable> {
        let mut inputs = Vec::new();
        for i in 0..48 {
            inputs.push(self.public_key[i]);
        }
        for i in 0..32 {
            inputs.push(self.withdrawal_credentials[i]);
        }
        for i in 0..8 {
            inputs.push(self.effective_balance[i]);
        }
        for i in 0..1 {
            inputs.push(self.slashed[i]);
        }
        for i in 0..8 {
            inputs.push(self.activation_eligibility_epoch[i]);
        }
        for i in 0..8 {
            inputs.push(self.activation_epoch[i]);
        }
        for i in 0..8 {
            inputs.push(self.exit_epoch[i]);
        }
        for i in 0..8 {
            inputs.push(self.withdrawable_epoch[i]);
        }
        let params = PoseidonM31Params::new(
            builder,
            POSEIDON_M31X16_RATE,
            16,
            POSEIDON_M31X16_FULL_ROUNDS,
            POSEIDON_M31X16_PARTIAL_ROUNDS,
        );
        params.hash_to_state_flatten(builder, &inputs)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ValidatorSubTreeJson {
    #[serde(rename = "ValidatorHashChunk")]
    pub validators_hash_chunk: Vec<Vec<u32>>,
    #[serde(rename = "SubtreeRoot")]
    pub subtree_root: Vec<u32>,
}
declare_circuit!(ConvertValidatorListToMerkleTreeCircuit {
    validator_hash_chunk: [[Variable; POSEIDON_M31X16_RATE]; SUBTREE_SIZE],
    subtree_root: [Variable; POSEIDON_M31X16_RATE], // Public input
});
impl ConvertValidatorListToMerkleTreeCircuit<M31> {
    pub fn from_assignment(&mut self, validator_subtree_json: &ValidatorSubTreeJson) {
        for i in 0..SUBTREE_SIZE {
            for j in 0..POSEIDON_M31X16_RATE {
                self.validator_hash_chunk[i][j] =
                    M31::from(validator_subtree_json.validators_hash_chunk[i][j]);
            }
        }
        for i in 0..POSEIDON_M31X16_RATE {
            self.subtree_root[i] = M31::from(validator_subtree_json.subtree_root[i]);
        }
    }
    pub fn get_assignments_from_data(
        validator_subtree_data: Vec<ValidatorSubTreeJson>,
    ) -> Vec<Self> {
        let mut handles = vec![];
        let assignments = Arc::new(Mutex::new(vec![None; validator_subtree_data.len()]));

        for (i, validator_subtree_item) in validator_subtree_data.into_iter().enumerate() {
            let assignments = Arc::clone(&assignments);

            let handle = thread::spawn(move || {
                let mut assignment = ConvertValidatorListToMerkleTreeCircuit::<M31>::default();
                assignment.from_assignment(&validator_subtree_item);

                let mut assignments = assignments.lock().unwrap();
                assignments[i] = Some(assignment);
            });

            handles.push(handle);
        }
        for handle in handles {
            handle.join().expect("Thread panicked");
        }

        let assignments = assignments
            .lock()
            .unwrap()
            .iter()
            .map(|x| x.clone().unwrap())
            .collect::<Vec<_>>();
        assignments
    }
    pub fn get_assignments_from_json(dir: &str) -> Vec<Self> {
        let file_path = format!("{}/validatorsubtree_assignment.json", dir);
        let validator_subtree_data: Vec<ValidatorSubTreeJson> =
            read_from_json_file(&file_path).unwrap();
        Self::get_assignments_from_data(validator_subtree_data)
    }
}
impl GenericDefine<M31Config> for ConvertValidatorListToMerkleTreeCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let mut inputs = vec![];

        // Flatten `validator_hash_chunk` into a single input vector
        for i in 0..SUBTREE_SIZE {
            inputs.extend_from_slice(&self.validator_hash_chunk[i]);
        }

        // Compute the Poseidon hash
        let params = PoseidonM31Params::new(
            builder,
            POSEIDON_M31X16_RATE,
            16,
            POSEIDON_M31X16_FULL_ROUNDS,
            POSEIDON_M31X16_PARTIAL_ROUNDS,
        );
        let sub_tree_root = params.hash_to_state_flatten(builder, &inputs);

        // Enforce equality between computed root and given root
        for (i, elem) in sub_tree_root.iter().enumerate().take(POSEIDON_M31X16_RATE) {
            builder.assert_is_equal(elem, self.subtree_root[i]);
        }
    }
}

pub fn generate_validator_subtree_witnesses(dir: &str) {
    stacker::grow(32 * 1024 * 1024 * 1024, || {
        let circuit_name = &format!("validatorsubtree{}", SUBTREE_SIZE);

        //get solver
        log::debug!("preparing {} solver...", circuit_name);
        let witnesses_dir = format!("./witnesses/{}", circuit_name);
        let w_s = get_solver(
            &witnesses_dir,
            circuit_name,
            ConvertValidatorListToMerkleTreeCircuit::default(),
        );

        let start_time = std::time::Instant::now();
        let assignments = ConvertValidatorListToMerkleTreeCircuit::get_assignments_from_json(dir);
        let end_time = std::time::Instant::now();
        log::debug!(
            "assigned assignment data, time: {:?}",
            end_time.duration_since(start_time)
        );
        let assignment_chunks: Vec<Vec<ConvertValidatorListToMerkleTreeCircuit<M31>>> =
            assignments.chunks(16).map(|x| x.to_vec()).collect();

        //generate witnesses (multi-thread)
        log::debug!("Start generating witnesses...");
        let witness_solver = Arc::new(w_s);
        let handles = assignment_chunks
            .into_iter()
            .enumerate()
            .map(|(i, assignments)| {
                let witness_solver = Arc::clone(&witness_solver);
                let witnesses_dir_clone = witnesses_dir.clone();
                thread::spawn(move || {
                    let mut hint_registry = HintRegistry::<M31>::new();
                    register_hint(&mut hint_registry);
                    let witness = witness_solver
                        .solve_witnesses_with_hints(&assignments, &mut hint_registry)
                        .unwrap();
                    write_witness_to_file(
                        &format!("{}/witness_{}.txt", witnesses_dir_clone, i),
                        witness,
                    )
                })
            })
            .collect::<Vec<_>>();
        for handle in handles {
            handle.join().unwrap();
        }
        let end_time = std::time::Instant::now();
        log::debug!(
            "Generate {} witness Time: {:?}",
            circuit_name,
            end_time.duration_since(start_time)
        );
    });
}

pub fn end2end_validator_subtree_witnesses(
    w_s: WitnessSolver<M31Config>,
    validator_subtree_data: Vec<ValidatorSubTreeJson>,
) {
    let circuit_name = &format!("validatorsubtree{}", SUBTREE_SIZE);

    log::debug!("preparing {} solver...", circuit_name);
    let witnesses_dir = format!("./witnesses/{}", circuit_name);

    let start_time = std::time::Instant::now();
    let assignments =
        ConvertValidatorListToMerkleTreeCircuit::get_assignments_from_data(validator_subtree_data);
    let end_time = std::time::Instant::now();
    log::debug!(
        "assigned assignment data, time: {:?}",
        end_time.duration_since(start_time)
    );
    let assignment_chunks: Vec<Vec<ConvertValidatorListToMerkleTreeCircuit<M31>>> =
        assignments.chunks(16).map(|x| x.to_vec()).collect();

    //generate witnesses (multi-thread)
    log::debug!("Start generating witnesses...");
    let witness_solver = Arc::new(w_s);
    let handles = assignment_chunks
        .into_iter()
        .enumerate()
        .map(|(i, assignments)| {
            let witness_solver = Arc::clone(&witness_solver);
            let witnesses_dir_clone = witnesses_dir.clone();
            thread::spawn(move || {
                let mut hint_registry = HintRegistry::<M31>::new();
                register_hint(&mut hint_registry);
                let witness = witness_solver
                    .solve_witnesses_with_hints(&assignments, &mut hint_registry)
                    .unwrap();
                write_witness_to_file(
                    &format!("{}/witness_{}.txt", witnesses_dir_clone, i),
                    witness,
                )
            })
        })
        .collect::<Vec<_>>();
    for handle in handles {
        handle.join().unwrap();
    }
    let end_time = std::time::Instant::now();
    log::debug!(
        "Generate {} witness Time: {:?}",
        circuit_name,
        end_time.duration_since(start_time)
    );
}
