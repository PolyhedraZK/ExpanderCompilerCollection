use circuit_std_rs::sha256::gf2::SHA256GF2;
use expander_compiler::frontend::*;
use rand::RngCore;
use sha2::{Digest, Sha256};

const INPUT_LEN: usize = 8; // input size in bits, must be a power of 8
const OUTPUT_LEN: usize = 256; // FIXED 256

declare_circuit!(SHA256Circuit {
    input: [Variable; INPUT_LEN],
    output: [Variable; OUTPUT_LEN],
});

impl GenericDefine<GF2Config> for SHA256Circuit<Variable> {
    fn define<Builder: RootAPI<GF2Config>>(&self, api: &mut Builder) {
        let mut hasher = SHA256GF2::new(api);
        hasher.update(&self.input);
        let output = hasher.finalize(api);
        (0..OUTPUT_LEN).for_each(|i| api.assert_is_equal(output[i], self.output[i]));
    }
}

#[test]
fn test_sha256_37bytes() {
    let compile_result =
        compile_generic(&SHA256Circuit::default(), CompileOptions::default()).unwrap();
    
    let mut rng = rand::thread_rng();
    for _ in 0..1 {
        let data = [rng.next_u32() as u8; INPUT_LEN / 8];
        let mut hash = Sha256::new();
        hash.update(&data);
        let output = hash.finalize();
        let mut assignment = SHA256Circuit::default();
        for i in 0..INPUT_LEN {
            for j in 0..8 {
                assignment.input[i] = (((data[i / 8] >> (7 - j)) & 1) as u32).into();
            }
        }
        for i in 0..OUTPUT_LEN {
            for j in 0..8 {
                assignment.output[i] = (((output[i / 8] >> (7 - j) as u32) & 1) as u32).into();                
            }
        }
        let witness = compile_result
            .witness_solver
            .solve_witness(&assignment)
            .unwrap();
        let output = compile_result.layered_circuit.run(&witness);
        assert_eq!(output, vec![true]);
    }
}
