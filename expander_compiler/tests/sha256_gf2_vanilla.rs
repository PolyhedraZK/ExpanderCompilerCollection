use arith::Field;
// credit: https://github.com/PolyhedraZK/proof-arena/blob/main/problems/sha256_hash/expander-sha256/src/main.rs
use expander_compiler::frontend::*;

mod sha256_utils;
use rand::RngCore;
use sha2::{Digest, Sha256};
use sha256_utils::*;

const N_HASHES: usize = 1;

declare_circuit!(SHA256Circuit {
    input: [[Variable; 64 * 8]; N_HASHES],
    output: [[Variable; 256]; N_HASHES], // TODO: use public inputs
});

impl Define<GF2Config> for SHA256Circuit<Variable> {
    fn define(&self, api: &mut API<GF2Config>) {
        for j in 0..N_HASHES {
            let out = compute_sha256(api, &self.input[j].to_vec());
            for i in 0..256 {
                api.assert_is_equal(out[i].clone(), self.output[j][i].clone());
            }
        }
    }
}

fn compute_sha256<C: Config>(api: &mut API<C>, input: &Vec<Variable>) -> Vec<Variable> {
    let h32: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];

    let mut h: Vec<Vec<Variable>> = (0..8).map(|x| int2bit(api, h32[x])).collect();

    let k32: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c48, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa11, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    //    let k: Vec<Vec<Variable>> = (0..64).map(|x| int2bit(api, k32[x])).collect();

    let mut w = vec![vec![api.constant(0); 32]; 64];
    for i in 0..16 {
        w[i] = input[(i * 32)..((i + 1) * 32) as usize].to_vec();
    }
    for i in 16..64 {
        let tmp = xor(
            api,
            rotate_right(&w[i - 15], 7),
            rotate_right(&w[i - 15], 18),
        );
        let shft = shift_right(api, w[i - 15].clone(), 3);
        let s0 = xor(api, tmp, shft);
        let tmp = xor(
            api,
            rotate_right(&w[i - 2], 17),
            rotate_right(&w[i - 2], 19),
        );
        let shft = shift_right(api, w[i - 2].clone(), 10);
        let s1 = xor(api, tmp, shft);
        let s0 = add_crosslayer(api, w[i - 16].clone(), s0);
        let s1 = add_crosslayer(api, w[i - 7].clone(), s1);
        let s1 = add_const(api, s1, k32[i]);
        w[i] = add_crosslayer(api, s0, s1);
    }

    for i in 0..64 {
        let s1 = sigma1(api, h[4].clone());
        let c = ch(api, h[4].clone(), h[5].clone(), h[6].clone());
        w[i] = add_crosslayer(api, w[i].clone(), h[7].clone());
        let c = add_const(api, c, k32[i].clone());
        let s1 = add_crosslayer(api, s1, w[i].clone());
        let s1 = add_crosslayer(api, s1, c);
        let s0 = sigma0(api, h[0].clone());
        let m = maj(api, h[0].clone(), h[1].clone(), h[2].clone());
        let s0 = add_crosslayer(api, s0, m);

        h[7] = h[6].clone();
        h[6] = h[5].clone();
        h[5] = h[4].clone();
        h[4] = add_crosslayer(api, h[3].clone(), s1.clone());
        h[3] = h[2].clone();
        h[2] = h[1].clone();
        h[1] = h[0].clone();
        h[0] = add_crosslayer(api, s1, s0);
    }

    let mut result = add_const(api, h[0].clone(), h32[0].clone());
    for i in 1..8 {
        result.append(&mut add_const(api, h[i].clone(), h32[i].clone()));
    }

    result
}

fn gen_assignment(
    n_assignments: usize,
    n_hashes_per_assignments: usize,
    mut rng: impl RngCore,
) -> Vec<SHA256Circuit<GF2>> {
    let input_bytes = (0..n_assignments)
        .map(|_| {
            (0..(64 * n_hashes_per_assignments))
                .map(|_| rng.next_u32() as u8)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    input_bytes
        .iter()
        .map(|input| {
            let mut assignment = SHA256Circuit::<GF2>::default();
            for (k, data) in input.chunks_exact(64).enumerate() {
                let mut hash = Sha256::new();
                hash.update(&data);
                let output = hash.finalize();
                for i in 0..64 {
                    for j in 0..8 {
                        assignment.input[k % n_hashes_per_assignments][i * 8 + j] =
                            ((data[i] >> j) as u32 & 1).into();
                    }
                }
                for i in 0..32 {
                    for j in 0..8 {
                        assignment.output[k % n_hashes_per_assignments][i * 8 + j] =
                            ((output[i] >> j) as u32 & 1).into();
                    }
                }
            }
            assignment
        })
        .collect()
}

#[test]
fn test_sha256_gf2() {
    let compile_result = compile_cross_layer(&SHA256Circuit::default()).unwrap();
    let CompileResultCrossLayer {
        witness_solver,
        layered_circuit,
    } = compile_result;

    let n_assignments = 8;
    let rng = rand::thread_rng();
    let mut assignments = gen_assignment(n_assignments, N_HASHES, rng);

    let witness = witness_solver.solve_witnesses(&assignments).unwrap();
    let res = layered_circuit.run(&witness);
    let expected_res = vec![true; n_assignments];
    assert_eq!(res, expected_res);

    // Test with wrong input
    for i in 0..n_assignments {
        for j in 0..N_HASHES {
            assignments[i].input[j][0] = assignments[i].input[j][0].clone() - GF2::ONE;
        }
    }
    let witness_incorrect = witness_solver.solve_witnesses(&assignments).unwrap();
    let res_incorrect = layered_circuit.run(&witness_incorrect);
    let expected_res_incorrect = vec![false; n_assignments];
    assert_eq!(res_incorrect, expected_res_incorrect);
}
