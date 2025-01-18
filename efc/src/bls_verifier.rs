use std::sync::Arc;
use std::thread;

use circuit_std_rs::gnark::emulated::sw_bls12381::g1::*;
use circuit_std_rs::gnark::emulated::sw_bls12381::g2::*;
use circuit_std_rs::gnark::emulated::sw_bls12381::pairing::*;
use circuit_std_rs::utils::register_hint;
use expander_compiler::circuit::ir::hint_normalized::witness_solver;
use expander_compiler::compile::CompileOptions;
use expander_compiler::declare_circuit;
use expander_compiler::frontend::compile_generic;
use expander_compiler::frontend::internal::Serde;
use expander_compiler::frontend::GenericDefine;
use expander_compiler::frontend::HintRegistry;
use expander_compiler::frontend::M31Config;
use expander_compiler::frontend::{RootAPI, Variable, M31};

use serde::Deserialize;

use crate::utils::ensure_directory_exists;
use crate::utils::read_from_json_file;

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

declare_circuit!(PairingCircuit {
    pubkey: [[Variable; 48]; 2],
    hm: [[[Variable; 48]; 2]; 2],
    sig: [[[Variable; 48]; 2]; 2]
});

impl PairingCircuit<M31> {
    pub fn from_entry(entry: &PairingEntry) -> Self {
        fn convert_limbs(limbs: Vec<u8>) -> [M31; 48] {
            let converted: Vec<M31> = limbs.into_iter().map(|x| M31::from(x as u32)).collect();
            converted.try_into().expect("Limbs should have 48 elements")
        }

        fn convert_point(point: Coordinate) -> [[M31; 48]; 2] {
            [convert_limbs(point.a0.limbs), convert_limbs(point.a1.limbs)]
        }

        PairingCircuit {
            pubkey: [
                convert_limbs(entry.pub_key.x.limbs.clone()),
                convert_limbs(entry.pub_key.y.limbs.clone()),
            ],
            hm: [
                convert_point(entry.hm.p.x.clone()),
                convert_point(entry.hm.p.y.clone()),
            ],
            sig: [
                convert_point(entry.signature.p.x.clone()),
                convert_point(entry.signature.p.y.clone()),
            ],
        }
    }
}
impl GenericDefine<M31Config> for PairingCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let mut pairing = Pairing::new(builder);
        let one_g1 = G1Affine::one(builder);
        let pubkey_g1 = G1Affine::from_vars(self.pubkey[0].to_vec(), self.pubkey[1].to_vec());
        let hm_g2 = G2AffP::from_vars(
            self.hm[0][0].to_vec(),
            self.hm[0][1].to_vec(),
            self.hm[1][0].to_vec(),
            self.hm[1][1].to_vec(),
        );
        let sig_g2 = G2AffP::from_vars(
            self.sig[0][0].to_vec(),
            self.sig[0][1].to_vec(),
            self.sig[1][0].to_vec(),
            self.sig[1][1].to_vec(),
        );

        let mut g2 = G2::new(builder);
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
    }
}

pub fn generate_pairing_witnesses(dir: &str) {
    println!("preparing solver...");
    ensure_directory_exists("./witnesses/pairing");
    let file_name = "pairing.witness";
    let w_s = if std::fs::metadata(file_name).is_ok() {
        println!("The solver exists!");
        witness_solver::WitnessSolver::deserialize_from(std::fs::File::open(file_name).unwrap())
            .unwrap()
    } else {
        println!("The solver does not exist.");
        let compile_result =
            compile_generic(&PairingCircuit::default(), CompileOptions::default()).unwrap();
        compile_result
            .witness_solver
            .serialize_into(std::fs::File::create(file_name).unwrap())
            .unwrap();
        compile_result.witness_solver
    };

    println!("Start generating witnesses...");
    let start_time = std::time::Instant::now();
    let file_path = format!("{}/pairing_assignment.json", dir);

    let pairing_data: Vec<PairingEntry> = read_from_json_file(&file_path).unwrap();
    let end_time = std::time::Instant::now();
    println!(
        "loaded pairing data time: {:?}",
        end_time.duration_since(start_time)
    );
    let mut assignments = vec![];
    for cur_pairing_data in &pairing_data {
        let pairing_assignment = PairingCircuit::from_entry(cur_pairing_data);
        assignments.push(pairing_assignment);
    }
    let end_time = std::time::Instant::now();
    println!(
        "assigned assignments time: {:?}",
        end_time.duration_since(start_time)
    );
    let assignment_chunks: Vec<Vec<PairingCircuit<M31>>> =
        assignments.chunks(16).map(|x| x.to_vec()).collect();
    let witness_solver = Arc::new(w_s);
    let handles = assignment_chunks
        .into_iter()
        .enumerate()
        .map(|(i, assignments)| {
            let witness_solver = Arc::clone(&witness_solver);
            thread::spawn(move || {
                let mut hint_registry1 = HintRegistry::<M31>::new();
                register_hint(&mut hint_registry1);
                let witness = witness_solver
                    .solve_witnesses_with_hints(&assignments, &mut hint_registry1)
                    .unwrap();
                let file_name = format!("./witnesses/pairing/witness_{}.txt", i);
                let file = std::fs::File::create(file_name).unwrap();
                let writer = std::io::BufWriter::new(file);
                witness.serialize_into(writer).unwrap();
            })
        })
        .collect::<Vec<_>>();
    for handle in handles {
        handle.join().unwrap();
    }
    let end_time = std::time::Instant::now();
    println!(
        "Generate pairing witness Time: {:?}",
        end_time.duration_since(start_time)
    );
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::utils::ensure_directory_exists;
//     use std::fs::File;
//     use std::io::Write;

//     declare_circuit!(VerifySigCircuit {
//         pubkey: [[Variable; 48]; 2],
//         slot: [Variable; 8],
//         committee_index: [Variable; 8],
//         beacon_block_root: [[Variable; 8]; 32],
//         source_epoch: [Variable; 8],
//         target_epoch: [Variable; 8],
//         source_root: [Variable; 32],
//         target_root: [Variable; 32],
//         sig_byte: [Variable; 48]
//     });

//     impl GenericDefine<M31Config> for VerifySigCircuit<Variable> {
//         fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
//             let mut pairing = Pairing::new(builder);
//             let one_g1 = G1Affine::one(builder);
//             let pubkey_g1 = G1Affine::from_vars(self.pubkey[0].to_vec(), self.pubkey[1].to_vec());
//             let sig_g2 = G2AffP::from_vars(
//                 self.sig[0][0].to_vec(),
//                 self.sig[0][1].to_vec(),
//                 self.sig[1][0].to_vec(),
//                 self.sig[1][1].to_vec(),
//             );

//             let mut g2 = G2::new(builder);
//             let neg_sig_g2 = g2.neg(builder, &sig_g2);

//             let (hm0, hm1) = g2.hash_to_fp(builder, self.msg.to_vec());
//             let res = g2.map_to_g2(builder, &hm0, &hm1);

//             let p_array = vec![one_g1, pubkey_g1];
//             let mut q_array = [
//                 G2Affine {
//                     p: neg_sig_g2,
//                     lines: LineEvaluations::default(),
//                 },
//                 G2Affine {
//                     p: res,
//                     lines: LineEvaluations::default(),
//                 },
//             ];
//             pairing
//                 .pairing_check(builder, &p_array, &mut q_array)
//                 .unwrap();
//             pairing.ext12.ext6.ext2.curve_f.check_mul(builder);
//             pairing.ext12.ext6.ext2.curve_f.table.final_check(builder);
//             pairing.ext12.ext6.ext2.curve_f.table.final_check(builder);
//             pairing.ext12.ext6.ext2.curve_f.table.final_check(builder);
//         }
//     }

//     #[test]
//     fn test_pairing_circuit() {

//         /*
//         att 0
//         att.Data.Slot 9280000
//         att.Data.CommitteeIndex 0
//         att.Data.BeaconBlockRoot [31 28 22 87 106 251 75 169 100 167 224 201 6 63 144 105 213 235 18 224 169 157 122 56 47 48 28 31 124 69 38 248]
//         att.Data.Source 289999 [194 212 152 232 56 145 101 103 73 230 240 242 89 129 63 184 38 157 86 185 251 148 157 68 227 144 241 74 228 200 206 199]
//         att.Data.Target 290000 [31 28 22 87 106 251 75 169 100 167 224 201 6 63 144 105 213 235 18 224 169 157 122 56 47 48 28 31 124 69 38 248]
//         att.Signature [170 121 191 2 187 22 51 113 109 233 89 181 237 140 207 117 72 230 115 61 124 161 23 145 241 245 211 134 175 182 206 188 124 240 51 154 121 27 217 24 126 83 70 24 90 206 50 148 2 182 65 209 6 215 131 231 254 32 229 193 207 91 52 22 89 10 212 80 4 160 179 150 246 97 120 81 28 231 36 195 223 118 194 250 230 31 182 130 163 236 45 222 26 229 163 89]
//          */
