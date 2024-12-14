use arith::Field;
use expander_compiler::frontend::*;
use sha2::Digest;

use crate::{compute_sha256, u8_to_bits};

const N_HASHES: usize = 8;

// Sha2 hashes 64 bytes of input into 32 bytes of output
// in circuit we will have to operate on bits for better performance
declare_circuit!(SHA256Circuit {
    input: [[Variable; 64 * 8]; N_HASHES],
    output: [[Variable; 32 * 8]; N_HASHES],
});

impl Define<GF2Config> for SHA256Circuit<Variable> {
    fn define(&self, api: &mut API<GF2Config>) {
        for j in 0..N_HASHES {
            let out = compute_sha256(api, &self.input[j].to_vec());
            for i in 0..32 * 8 {
                api.assert_is_equal(out[i].clone(), self.output[j][i].clone());
            }
        }
    }
}

impl SHA256Circuit<GF2> {
    fn create_circuit(input: Vec<[u8; 64]>, output: Vec<[u8; 32]>) -> SHA256Circuit<GF2> {
        assert_eq!(input.len(), N_HASHES);
        assert_eq!(output.len(), N_HASHES);

        let mut input_vars = [[GF2::zero(); 64 * 8]; N_HASHES];
        let mut output_vars = [[GF2::zero(); 32 * 8]; N_HASHES];

        for j in 0..N_HASHES {
            for i in 0..64 {
                let bits = u8_to_bits(input[j][i]);
                for k in 0..8 {
                    input_vars[j][i * 8 + k] = GF2::from(bits[k] != 0);
                }
            }
            for i in 0..32 {
                let bits = u8_to_bits(output[j][i]);
                for k in 0..8 {
                    output_vars[j][i * 8 + k] = GF2::from(bits[k] != 0);
                }
            }
        }

        Self {
            input: input_vars,
            output: output_vars,
        }
    }
}

#[test]
fn test_sha2_circuit() {
    let compile_result = compile(&SHA256Circuit::default()).unwrap();

    let input = vec![
        [0u8; 64], [1u8; 64], [2u8; 64], [3u8; 64], [4u8; 64], [5u8; 64], [6u8; 64], [7u8; 64],
    ];
    let output = input
        .iter()
        .map(|x| {
            let mut hasher = sha2::Sha256::default();
            hasher.update(x);
            hasher
                .finalize()
                .into_iter()
                .collect::<Vec<u8>>()
                .try_into()
                .unwrap()
        })
        .collect();

    let assignment = SHA256Circuit::<GF2>::create_circuit(input, output);
    let witness = compile_result
        .witness_solver
        .solve_witness(&assignment)
        .unwrap();

    let output = compile_result.layered_circuit.run(&witness);
    assert_eq!(output, vec![true]);
}
