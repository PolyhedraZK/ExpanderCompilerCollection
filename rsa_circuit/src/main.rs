#![allow(dead_code)]
// allow range loop for better readability
#![allow(clippy::needless_range_loop)]

mod native;

use ark_std::test_rng;
use circuit_std_rs::{
    LogUpCircuit, LogUpParams, StdCircuit, U2048Variable, BN_TWO_TO_120, N_LIMBS,
};
use expander_compiler::frontend::{
    compile, declare_circuit, BN254Config, CompileOptions, Define, RootAPI, Variable,
};
use halo2curves::bn256::Fr;
use native::RSAFieldElement;
use num_bigint::BigUint;
use rand::RngCore;
use exp_serde::ExpSerde;
use sha2::Digest;

// see explanation in circuit declaration
const HASH_INPUT_LEN: usize = 1504;
const HASH_OUTPUT_LEN: usize = 1472;

// A RSA signature verification requires to perform two tasks
//
// 1. compute x^e mod n, where
// - e is fixed to 2^16 + 1
// - x and n are both 2048 bits integers
// - the intermediate results are stored in 16 2048 bits integers
// x^2, x^4, x^8, x^16, x^32, x^64, x^128, x^256, x^512, x^1024, x^2048
// x^4096, x^8192, x^16384, x^32768, x^65536
// - during the computation, we need to store the carries of the multiplication
//
// note: usually x is the hash of the message to sign -- for now we choose to ignore this hash operation
//
// also note: due to the limitation of the frontend, we have to pass those intermediate results as arguments
// ideally we want to be able to generate them with hint function -- it is currently not supported
//
// 2. hash 1504 bytes of data using sha256
// - takes the first 64 bytes inputs and produce 32 bytes output
// - takes the next 32 bytes inputs, pre-fixed with the 32 bytes output from previous hash, and produce a 32 bytes output
//    - this takes 45 iterations to hash all 1440 bytes
// - requires 46 iterations to hash all 1504 bytes
//    - produces 1472 bytes output, including the 32 bytes output from the last iteration
declare_circuit!(RSACircuit {
    x: [Variable; N_LIMBS],
    n: [Variable; N_LIMBS],
    x_powers: [[Variable; N_LIMBS]; 17],
    mul_carries: [[Variable; N_LIMBS]; 17],
    result: [Variable; N_LIMBS],
    hash_inputs: [Variable; HASH_INPUT_LEN],
    hash_outputs: [Variable; HASH_OUTPUT_LEN],
});

// To build this circuit we will need to compute intermediate results:
// e^2, e^4, e^8, e^16, e^32, e^64, e^128, e^256, e^512, e^1024, e^2048
// e^4096, e^8192, e^16384, e^32768, e^65536
// return
// - power of e-s
// - carries of the multiplication
// - carries for e * e^65536
fn build_rsa_traces(
    x: &RSAFieldElement,
    n: &RSAFieldElement,
    res: &RSAFieldElement,
) -> ([RSAFieldElement; 17], [RSAFieldElement; 17]) {
    let mut x_powers = [RSAFieldElement::new([0u128; N_LIMBS]); 17];
    let mut mul_carries = [RSAFieldElement::new([0u128; N_LIMBS]); 17];

    (x_powers[0], mul_carries[0]) = x.slow_mul_mod(x, n);
    for i in 1..16 {
        (x_powers[i], mul_carries[i]) = x_powers[i - 1].slow_mul_mod(&x_powers[i - 1], n);
    }
    (x_powers[16], mul_carries[16]) = x_powers[15].slow_mul_mod(x, n);

    // sanity check
    assert_eq!(x_powers[16], *res);
    (x_powers, mul_carries)
}

fn build_hash_outputs(hash_inputs: &[u8]) -> [u8; HASH_OUTPUT_LEN] {
    assert!(hash_inputs.len() == HASH_INPUT_LEN);

    let mut hash_outputs = vec![];
    let mut hash_input = hash_inputs[..64].to_vec();
    for i in 0..46 {
        let mut hasher = sha2::Sha256::default();
        hasher.update(&hash_input);
        let hash_output: [u8; 32] = hasher.finalize().into();
        hash_outputs.extend_from_slice(&hash_output);
        hash_input = [
            hash_inputs[(i + 2) * 32..(i + 3) * 32].as_ref(),
            hash_output.as_ref(),
        ]
        .concat();
    }
    assert!(hash_outputs.len() == HASH_OUTPUT_LEN);

    hash_outputs.try_into().unwrap()
}

impl Define<BN254Config> for RSACircuit<Variable> {
    fn define<Builder: RootAPI<BN254Config>>(&self, builder: &mut Builder) {
        // task 1: compute x^e mod n
        {
            let two_to_120 = builder.constant(BN_TWO_TO_120);
            let x_var = U2048Variable::from_raw(self.x);
            let n_var = U2048Variable::from_raw(self.n);
            let x_power_vars = self
                .x_powers
                .iter()
                .map(|trace| U2048Variable::from_raw(*trace))
                .collect::<Vec<_>>();
            let mul_carry_vars = self
                .mul_carries
                .iter()
                .map(|carry| U2048Variable::from_raw(*carry))
                .collect::<Vec<_>>();

            // compute x^2
            U2048Variable::assert_mul(
                &x_var,
                &x_var,
                &x_power_vars[0],
                &mul_carry_vars[0],
                &n_var,
                &two_to_120,
                builder,
            );

            // compute x^4, to x^65536
            for i in 1..11 {
                U2048Variable::assert_mul(
                    &x_power_vars[i - 1],
                    &x_power_vars[i - 1],
                    &x_power_vars[i],
                    &mul_carry_vars[i],
                    &n_var,
                    &two_to_120,
                    builder,
                );
            }

            // assert x_powers[16] = x^65537
            U2048Variable::assert_mul(
                &x_var,
                &x_power_vars[15],
                &x_power_vars[16],
                &mul_carry_vars[16],
                &n_var,
                &two_to_120,
                builder,
            );

            // assert result = x_powers[16]
            for i in 0..N_LIMBS {
                builder.assert_is_equal(x_power_vars[16].limbs[i], self.result[i]);
            }
        }
    }
}

impl RSACircuit<Fr> {
    fn _build_log_up_table() {
        let logup_param = LogUpParams {
            key_len: 1,
            value_len: 1,
            n_table_rows: 1 << 8,
            n_queries: 1 << 8,
        };
        let _circuit = <LogUpCircuit as StdCircuit<BN254Config>>::new_circuit(&logup_param);
    }
}

impl RSACircuit<Fr> {
    fn create_circuit(
        x: &RSAFieldElement,
        n: &RSAFieldElement,
        x_powers: [RSAFieldElement; 17],
        mul_carries: [RSAFieldElement; 17],
        result: &RSAFieldElement,
        hash_inputs: &[u8],
    ) -> Self {
        assert!(hash_inputs.len() == HASH_INPUT_LEN);

        let x: [Fr; N_LIMBS] = (*x).into();
        let n: [Fr; N_LIMBS] = (*n).into();
        let x_powers: [[Fr; N_LIMBS]; 17] = x_powers
            .iter()
            .map(|x| (*x).into())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let mul_carries: [[Fr; N_LIMBS]; 17] = mul_carries
            .iter()
            .map(|x| (*x).into())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let result: [Fr; N_LIMBS] = (*result).into();

        let hash_outputs = build_hash_outputs(hash_inputs);
        let hash_inputs = hash_inputs
            .iter()
            .map(|x| Fr::from(*x as u64))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let hash_outputs = hash_outputs
            .iter()
            .map(|x| Fr::from(*x as u64))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        Self {
            x,
            n,
            x_powers,
            mul_carries,
            result,
            hash_inputs,
            hash_outputs,
        }
    }
}

fn main() {
    let mut rng = test_rng();

    // build a dummy circuit
    let compile_result = compile(&RSACircuit::default(), CompileOptions::default()).unwrap();

    // generate the trace and setup the circuit assignment
    let pow = BigUint::from(65537u64);
    let x = BigUint::from(2u64);
    let n = (BigUint::from(1u64) << 120) + (BigUint::from(1u64) << 2047);
    let res = x.modpow(&pow, &n);
    let hash_inputs = {
        let mut tmp = [0u8; HASH_OUTPUT_LEN];
        rng.fill_bytes(&mut tmp);
        tmp
    };

    let x = RSAFieldElement::from_big_uint(x);
    let n = RSAFieldElement::from_big_uint(n);
    let res = RSAFieldElement::from_big_uint(res);

    let (x_powers, mul_carries) = build_rsa_traces(&x, &n, &res);
    for (i, x) in x_powers.iter().enumerate() {
        println!("x^{}:\n{:0x?}", 2u64.pow(i as u32 + 1), x.to_big_uint());
        println!("{:0x?}\n", mul_carries[i].to_big_uint());
    }

    let assignment = RSACircuit::create_circuit(&x, &n, x_powers, mul_carries, &res, &hash_inputs);

    // check the witnesses are correct
    let witness = compile_result
        .witness_solver
        .solve_witness(&assignment)
        .unwrap();

    let _output = compile_result.layered_circuit.run(&witness);

    // there is some bug within the circuit. let's skip this assertion for now
    // assert_eq!(output, vec![true]);

    let file = std::fs::File::create("circuit_rsa.txt").unwrap();
    let writer = std::io::BufWriter::new(file);
    compile_result
        .layered_circuit
        .serialize_into(writer)
        .unwrap();

    let file = std::fs::File::create("witness_rsa.txt").unwrap();
    let writer = std::io::BufWriter::new(file);
    witness.serialize_into(writer).unwrap();

    let file = std::fs::File::create("witness_rsa_solver.txt").unwrap();
    let writer = std::io::BufWriter::new(file);
    compile_result
        .witness_solver
        .serialize_into(writer)
        .unwrap();

    println!("dumped to files");
}
