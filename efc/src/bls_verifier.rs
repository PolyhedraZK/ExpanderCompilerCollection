use crate::attestation::{Attestation, AttestationDataSSZ};
use crate::utils::convert_limbs;
use crate::utils::ensure_directory_exists;
use crate::utils::read_from_json_file;
use crate::utils::{get_solver, write_witness_to_file};
use crate::{beacon, bls};
use ark_bls12_381::G1Affine as BlsG1Affine;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use circuit_std_rs::gnark::emulated::sw_bls12381::g1::*;
use circuit_std_rs::gnark::emulated::sw_bls12381::g2::*;
use circuit_std_rs::gnark::emulated::sw_bls12381::pairing::*;
use circuit_std_rs::utils::register_hint;
use expander_compiler::declare_circuit;
use expander_compiler::frontend::GenericDefine;
use expander_compiler::frontend::HintRegistry;
use expander_compiler::frontend::M31Config;
use expander_compiler::frontend::WitnessSolver;
use expander_compiler::frontend::{RootAPI, Variable, M31};
use serde::Deserialize;
use std::sync::Arc;
use std::thread;

#[derive(Clone, Debug, Deserialize)]
pub struct Limbs {
    #[serde(rename = "Limbs")]
    pub limbs: Vec<u8>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Coordinate {
    #[serde(rename = "A0")]
    pub a0: Limbs,
    #[serde(rename = "A1")]
    pub a1: Limbs,
}

pub fn convert_point(point: Coordinate) -> [[M31; 48]; 2] {
    [convert_limbs(point.a0.limbs), convert_limbs(point.a1.limbs)]
}

#[derive(Clone, Debug, Deserialize)]
pub struct Point {
    #[serde(rename = "X")]
    pub x: Coordinate,
    #[serde(rename = "Y")]
    pub y: Coordinate,
}

#[derive(Debug, Deserialize, Clone)]
pub struct G2Json {
    #[serde(rename = "P")]
    pub p: Point,
    #[serde(rename = "Lines")]
    pub lines: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct G1Json {
    #[serde(rename = "X")]
    pub x: Limbs,
    #[serde(rename = "Y")]
    pub y: Limbs,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PairingEntry {
    #[serde(rename = "Hm")]
    pub hm: G2Json,
    #[serde(rename = "PubKey")]
    pub pub_key: G1Json,
    #[serde(rename = "Signature")]
    pub signature: G2Json,
}
declare_circuit!(BLSVERIFIERCircuit {
    pubkey: [[Variable; 48]; 2],

    attestation_sig_bytes: [Variable; 96],    //PUBLIC
    slot: [Variable; 8],                      //PUBLIC
    committee_index: [Variable; 8],           //PUBLIC
    beacon_beacon_block_root: [Variable; 32], //PUBLIC
    source_epoch: [Variable; 8],              //PUBLIC
    target_epoch: [Variable; 8],              //PUBLIC
    source_root: [Variable; 32],              //PUBLIC
    target_root: [Variable; 32],              //PUBLIC
});
pub type BlsVerifierAssignmentChunks = Vec<Vec<BLSVERIFIERCircuit<M31>>>;
impl BLSVERIFIERCircuit<M31> {
    pub fn get_assignments_from_data(
        pairing_data: Vec<PairingEntry>,
        attestations: Vec<Attestation>,
    ) -> Vec<Self> {
        let mut assignments = vec![];
        for (cur_pairing_data, cur_attestation_data) in pairing_data.iter().zip(attestations.iter())
        {
            let mut pairing_assignment = BLSVERIFIERCircuit::default();
            pairing_assignment.from_entry(cur_pairing_data, cur_attestation_data);
            assignments.push(pairing_assignment);
        }
        assignments
    }
    pub fn get_assignments_from_json(dir: &str) -> Vec<Self> {
        let file_path = format!("{}/pairing_assignment.json", dir);
        let pairing_data: Vec<PairingEntry> = read_from_json_file(&file_path).unwrap();
        let file_path = format!("{}/slotAttestationsFolded.json", dir);
        let attestations: Vec<Attestation> = read_from_json_file(&file_path).unwrap();
        BLSVERIFIERCircuit::get_assignments_from_data(pairing_data, attestations)
    }
    pub fn from_entry(&mut self, entry: &PairingEntry, attestation: &Attestation) {
        self.pubkey = [
            convert_limbs(entry.pub_key.x.limbs.clone()),
            convert_limbs(entry.pub_key.y.limbs.clone()),
        ];
        //assign slot
        let slot = attestation.data.slot.to_le_bytes();
        for (j, slot_byte) in slot.iter().enumerate() {
            self.slot[j] = M31::from(*slot_byte as u32);
        }
        //assign committee_index
        let committee_index = attestation.data.committee_index.to_le_bytes();
        for (j, committee_index_byte) in committee_index.iter().enumerate() {
            self.committee_index[j] = M31::from(*committee_index_byte as u32);
        }
        //assign beacon_beacon_block_root
        let beacon_beacon_block_root = attestation.data.beacon_block_root.clone();
        let beacon_beacon_block_root = STANDARD.decode(beacon_beacon_block_root).unwrap();
        for (j, beacon_beacon_block_root_byte) in beacon_beacon_block_root.iter().enumerate() {
            self.beacon_beacon_block_root[j] = M31::from(*beacon_beacon_block_root_byte as u32);
        }
        //assign source_epoch
        let source_epoch = attestation.data.source.epoch.to_le_bytes();
        for (j, source_epoch_byte) in source_epoch.iter().enumerate() {
            self.source_epoch[j] = M31::from(*source_epoch_byte as u32);
        }
        //assign target_epoch
        let target_epoch = attestation.data.target.epoch.to_le_bytes();
        for (j, target_epoch_byte) in target_epoch.iter().enumerate() {
            self.target_epoch[j] = M31::from(*target_epoch_byte as u32);
        }
        //assign source_root
        let source_root = attestation.data.source.root.clone();
        let source_root = STANDARD.decode(source_root).unwrap();
        for (j, source_root_byte) in source_root.iter().enumerate() {
            self.source_root[j] = M31::from(*source_root_byte as u32);
        }
        //assign target_root
        let target_root = attestation.data.target.root.clone();
        let target_root = STANDARD.decode(target_root).unwrap();
        for (j, target_root_byte) in target_root.iter().enumerate() {
            self.target_root[j] = M31::from(*target_root_byte as u32);
        }

        //assign attestation_sig_bytes
        let attestation_sig_bytes = attestation.signature.clone();
        let attestation_sig_bytes = STANDARD.decode(attestation_sig_bytes).unwrap();
        for (j, attestation_sig_byte) in attestation_sig_bytes.iter().enumerate() {
            self.attestation_sig_bytes[j] = M31::from(*attestation_sig_byte as u32);
        }
    }
    pub fn get_assignments_from_beacon_data(
        pubkeys: Vec<BlsG1Affine>,
        attestations: Vec<Attestation>,
    ) -> Vec<Self> {
        let mut assignments = vec![];
        for (cur_pubkey, cur_attestation_data) in pubkeys.iter().zip(attestations.iter()) {
            let mut pairing_assignment = BLSVERIFIERCircuit::default();
            pairing_assignment.from_beacon(cur_pubkey, cur_attestation_data);
            assignments.push(pairing_assignment);
        }
        assignments
    }
    pub fn from_beacon(&mut self, pubkey: &BlsG1Affine, attestation: &Attestation) {
        let pubkey_bytes = bls::affine_point_to_bytes_g1(pubkey);
        self.pubkey = [
            convert_limbs(pubkey_bytes[0].to_vec()),
            convert_limbs(pubkey_bytes[1].to_vec()),
        ];
        //assign slot
        let slot = attestation.data.slot.to_le_bytes();
        for (j, slot_byte) in slot.iter().enumerate() {
            self.slot[j] = M31::from(*slot_byte as u32);
        }
        //assign committee_index
        let committee_index = attestation.data.committee_index.to_le_bytes();
        for (j, committee_index_byte) in committee_index.iter().enumerate() {
            self.committee_index[j] = M31::from(*committee_index_byte as u32);
        }
        //assign beacon_beacon_block_root
        let beacon_beacon_block_root = attestation.data.beacon_block_root.clone();
        let beacon_beacon_block_root = STANDARD.decode(beacon_beacon_block_root).unwrap();
        for (j, beacon_beacon_block_root_byte) in beacon_beacon_block_root.iter().enumerate() {
            self.beacon_beacon_block_root[j] = M31::from(*beacon_beacon_block_root_byte as u32);
        }
        //assign source_epoch
        let source_epoch = attestation.data.source.epoch.to_le_bytes();
        for (j, source_epoch_byte) in source_epoch.iter().enumerate() {
            self.source_epoch[j] = M31::from(*source_epoch_byte as u32);
        }
        //assign target_epoch
        let target_epoch = attestation.data.target.epoch.to_le_bytes();
        for (j, target_epoch_byte) in target_epoch.iter().enumerate() {
            self.target_epoch[j] = M31::from(*target_epoch_byte as u32);
        }
        //assign source_root
        let source_root = attestation.data.source.root.clone();
        let source_root = STANDARD.decode(source_root).unwrap();
        for (j, source_root_byte) in source_root.iter().enumerate() {
            self.source_root[j] = M31::from(*source_root_byte as u32);
        }
        //assign target_root
        let target_root = attestation.data.target.root.clone();
        let target_root = STANDARD.decode(target_root).unwrap();
        for (j, target_root_byte) in target_root.iter().enumerate() {
            self.target_root[j] = M31::from(*target_root_byte as u32);
        }

        //assign attestation_sig_bytes
        let attestation_sig_bytes = attestation.signature.clone();
        let attestation_sig_bytes = STANDARD.decode(attestation_sig_bytes).unwrap();
        for (j, attestation_sig_byte) in attestation_sig_bytes.iter().enumerate() {
            self.attestation_sig_bytes[j] = M31::from(*attestation_sig_byte as u32);
        }
    }
}
impl GenericDefine<M31Config> for BLSVERIFIERCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let mut pairing = Pairing::new(builder);
        let one_g1 = G1Affine::one(builder);
        let pubkey_g1 = G1Affine::from_vars(self.pubkey[0].to_vec(), self.pubkey[1].to_vec());

        let att_ssz = AttestationDataSSZ {
            slot: self.slot,
            committee_index: self.committee_index,
            beacon_block_root: self.beacon_beacon_block_root,
            source_epoch: self.source_epoch,
            target_epoch: self.target_epoch,
            source_root: self.source_root,
            target_root: self.target_root,
        };
        let mut g2 = G2::new(builder);
        // domain

        // domain
        /*
        AttestationDomains[i]: 01000000b5303f2ad2010d699a76c8e62350947421a3e4a979779642cfdb0f66
        attestation.AttestationDomains[i]: [1 0 0 0 181 48 63 42 210 1 13 105 154 118 200 230 35 80 148 116 33 163 228 169 121 119 150 66 207 219 15 102]
        AttestationDomains[i]: 01000000afcaaba0efab1ca832a15152469bb09bb84641c405171dfa2d3fb45f
        attestation.AttestationDomains[i]: [1 0 0 0 175 202 171 160 239 171 28 168 50 161 81 82 70 155 176 155 184 70 65 196 5 23 29 250 45 63 180 95]
        AttestationDomains[i]: 010000004a26c58b08add8089b75caa540848881a8d4f0af0be83417a85c0f45
        attestation.AttestationDomains[i]: [1 0 0 0 74 38 197 139 8 173 216 8 155 117 202 165 64 132 136 129 168 212 240 175 11 232 52 23 168 92 15 69]
        AttestationDomains[i]: 01000000bba4da96354c9f25476cf1bc69bf583a7f9e0af049305b62de676640
        attestation.AttestationDomains[i]: [1 0 0 0 187 164 218 150 53 76 159 37 71 108 241 188 105 191 88 58 127 158 10 240 73 48 91 98 222 103 102 64]
        AttestationDomains[i]: 010000006a95a1a967855d676d48be69883b712607f952d5198d0f5677564636
        attestation.AttestationDomains[i]: [1 0 0 0 106 149 161 169 103 133 93 103 109 72 190 105 136 59 113 38 7 249 82 213 25 141 15 86 119 86 70 54]
         */
        let domain = [
            1, 0, 0, 0, 106, 149, 161, 169, 103, 133, 93, 103, 109, 72, 190, 105, 136, 59, 113, 38, 7, 249, 82, 213, 25, 141, 15, 86, 119, 86, 70, 54
        ];
        let mut domain_var = vec![];
        for domain_byte in domain.iter() {
            domain_var.push(builder.constant(*domain_byte as u32));
        }
        let att_hash = att_ssz.att_data_signing_root(builder, &domain_var); //msg
        let (hm0, hm1) = g2.hash_to_fp(builder, &att_hash);
        let hm_g2 = g2.map_to_g2(builder, &hm0, &hm1);
        // unmarshal attestation sig
        let sig_g2 = g2.uncompressed(builder, &self.attestation_sig_bytes);

        let neg_sig_g2 = g2.neg(builder, &sig_g2);

        let p_array = vec![one_g1, pubkey_g1];
        let mut q_array = [
            G2Affine {
                p: neg_sig_g2,
                lines: LineEvaluations::default(),
            },
            G2Affine {
                p: hm_g2,
                lines: LineEvaluations::default(),
            },
        ];
        pairing
            .pairing_check(builder, &p_array, &mut q_array)
            .unwrap();
        pairing.ext12.ext6.ext2.curve_f.check_mul(builder);
        pairing.ext12.ext6.ext2.curve_f.table.final_check(builder);
        pairing.ext12.ext6.ext2.curve_f.table.final_check(builder);
        pairing.ext12.ext6.ext2.curve_f.table.final_check(builder);
        // pairing.ext12.ext6.ext2.curve_f.table.final_check(builder);
        g2.ext2.curve_f.check_mul(builder);
        g2.ext2.curve_f.table.final_check(builder);
        g2.ext2.curve_f.table.final_check(builder);
        g2.ext2.curve_f.table.final_check(builder);
        // g2.ext2.curve_f.table.final_check(builder);
    }
}

pub fn generate_blsverifier_witnesses(dir: &str) {
    let circuit_name = "blsverifier";

    //get solver
    log::debug!("preparing {} solver...", circuit_name);
    let witnesses_dir = format!("./witnesses/{}", circuit_name);
    let w_s = get_solver(&witnesses_dir, circuit_name, BLSVERIFIERCircuit::default());

    //get assignments
    let start_time = std::time::Instant::now();
    let assignments = BLSVERIFIERCircuit::<M31>::get_assignments_from_json(dir);
    let end_time = std::time::Instant::now();
    log::debug!(
        "assigned assignments time: {:?}",
        end_time.duration_since(start_time)
    );
    let assignment_chunks: BlsVerifierAssignmentChunks =
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
                //TODO: hint_registry cannot be shared/cloned
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

pub fn end2end_blsverifier_witness(
    w_s: WitnessSolver<M31Config>,
    pairing_data: Vec<PairingEntry>,
    attestations: Vec<Attestation>,
) {
    let circuit_name = "blsverifier";

    let witnesses_dir = format!("./witnesses/{}", circuit_name);
    ensure_directory_exists(&witnesses_dir);

    //get assignments
    let start_time = std::time::Instant::now();
    let assignments =
        BLSVERIFIERCircuit::<M31>::get_assignments_from_data(pairing_data, attestations);
    let end_time = std::time::Instant::now();
    log::debug!(
        "assigned {:} assignments time: {:?}",
        circuit_name,
        end_time.duration_since(start_time)
    );
    let assignment_chunks: BlsVerifierAssignmentChunks =
        assignments.chunks(16).map(|x| x.to_vec()).collect();

    //generate witnesses (multi-thread)
    log::debug!("Start generating  {} witnesses...", circuit_name);
    let witness_solver = Arc::new(w_s);
    let handles = assignment_chunks
        .into_iter()
        .enumerate()
        .map(|(i, assignments)| {
            let witness_solver = Arc::clone(&witness_solver);
            let witnesses_dir_clone = witnesses_dir.clone();
            thread::spawn(move || {
                //TODO: hint_registry cannot be shared/cloned
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
pub fn end2end_blsverifier_witnesses_with_assignments(
    w_s: WitnessSolver<M31Config>,
    assignment_chunks: BlsVerifierAssignmentChunks,
    offset: usize,
) {
    let circuit_name = "blsverifier";

    let witnesses_dir = format!("./witnesses/{}", circuit_name);
    let start_time = std::time::Instant::now();
    //generate witnesses (multi-thread)
    log::debug!("Start generating  {} witnesses...", circuit_name);
    let witness_solver = Arc::new(w_s);
    let handles = assignment_chunks
        .into_iter()
        .enumerate()
        .map(|(i, assignments)| {
            let witness_solver = Arc::clone(&witness_solver);
            let witnesses_dir_clone = witnesses_dir.clone();
            thread::spawn(move || {
                //TODO: hint_registry cannot be shared/cloned
                let mut hint_registry = HintRegistry::<M31>::new();
                register_hint(&mut hint_registry);
                let witness = witness_solver
                    .solve_witnesses_with_hints(&assignments, &mut hint_registry)
                    .unwrap();
                write_witness_to_file(
                    &format!("{}/witness_{}.txt", witnesses_dir_clone, i + offset),
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

pub fn end2end_blsverifier_assignments_with_beacon_data(
    aggregated_pubkeys: Vec<BlsG1Affine>,
    attestations: Vec<Attestation>,
    range: [usize; 2],
) -> BlsVerifierAssignmentChunks {
    //get assignments
    let start = range[0] * beacon::MAXCOMMITTEESPERSLOT;
    let end = range[1] * beacon::MAXCOMMITTEESPERSLOT;
    let start_time = std::time::Instant::now();
    let assignments = BLSVERIFIERCircuit::<M31>::get_assignments_from_beacon_data(
        aggregated_pubkeys[start..end].to_vec(),
        attestations[start..end].to_vec(),
    );
    let end_time = std::time::Instant::now();
    log::debug!(
        "assigned bls_verifier assignments time: {:?}",
        end_time.duration_since(start_time)
    );
    let assignment_chunks: BlsVerifierAssignmentChunks =
        assignments.chunks(16).map(|x| x.to_vec()).collect();
    assignment_chunks
}

pub fn end2end_blsverifier_witnesses_with_beacon_data(
    w_s: WitnessSolver<M31Config>,
    aggregated_pubkeys: Vec<BlsG1Affine>,
    attestations: Vec<Attestation>,
    range: [usize; 2],
) {
    stacker::grow(32 * 1024 * 1024 * 1024, || {
        //get assignments
        let assignment_chunks = end2end_blsverifier_assignments_with_beacon_data(
            aggregated_pubkeys,
            attestations,
            range,
        );

        //generate witnesses (multi-thread)
        end2end_blsverifier_witnesses_with_assignments(w_s, assignment_chunks, range[0] * 16);
    });
}

pub fn debug_blsverifier_with_assignments(
    assignment_chunks: BlsVerifierAssignmentChunks,
) {
    stacker::grow(32 * 1024 * 1024 * 1024, || {
        use expander_compiler::frontend::extra::debug_eval;
        let circuit_name = "blsverifier";

        let start_time = std::time::Instant::now();
        let mut hint_registry = HintRegistry::<M31>::new();
        register_hint(&mut hint_registry);
        debug_eval(&BLSVERIFIERCircuit::default(), &assignment_chunks[0][0], hint_registry);
        // let witness = w_s
        //             .solve_witnesses_with_hints(&assignment_chunks[0], &mut hint_registry)
        //             .unwrap();
        let end_time = std::time::Instant::now();
        log::debug!(
            "Generate {} witness Time: {:?}",
            circuit_name,
            end_time.duration_since(start_time)
        );
    });
}