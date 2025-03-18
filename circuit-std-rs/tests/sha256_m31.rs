use circuit_std_rs::sha256::{m31::*, m31_utils::to_binary_hint};
use expander_compiler::frontend::*;
use extra::*;
use sha2::{Digest, Sha256};

declare_circuit!(SHA256COMPARECircuit {
    input: [Variable; 37],
    output: [Variable; 32],
});


impl Define<M31Config> for SHA256COMPARECircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        for _ in 0..64{ 
            let mut data = self.input.to_vec();
            let n = data.len();
            if n != 32 + 1 + 4 {
                panic!("len(orignData) !=  32+1+4")
            }
            let mut pre_pad = vec![builder.constant(0); 64 - 37];
            pre_pad[0] = builder.constant(128); //0x80
            pre_pad[64 - 37 - 2] = builder.constant((37) * 8 / 256); //length byte
            pre_pad[64 - 37 - 1] = builder.constant((32 + 1 + 4) * 8 - 256); //length byte
            data.append(&mut pre_pad); //append padding
            let mut d = MyDigest::new(builder);
            d.chunk_write(builder, &data);
            let res = d.return_sum(builder).to_vec();
            for (i, val) in res.iter().enumerate() {
                builder.assert_is_equal(*val, self.output[i]);
            }
        }
    }
}

#[test]
fn test_sha256_sha256_comparecircuit() {
    let mut hint_registry = HintRegistry::<M31>::new();
    hint_registry.register("myhint.tobinary", to_binary_hint);
    compile(&SHA256COMPARECircuit::default(), CompileOptions::default()).unwrap();
}

declare_circuit!(SHA25637BYTESCircuit {
    input: [Variable; 37],
    output: [Variable; 32],
});

pub fn check_sha256<C: Config, B: RootAPI<C>>(
    builder: &mut B,
    origin_data: &Vec<Variable>,
) -> Vec<Variable> {
    let output = origin_data[37..].to_vec();
    let result = sha256_37bytes(builder, &origin_data[..37]);
    for i in 0..32 {
        builder.assert_is_equal(result[i], output[i]);
    }
    result
}

impl Define<M31Config> for SHA25637BYTESCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        for _ in 0..1 {
            let mut data = self.input.to_vec();
            data.append(&mut self.output.to_vec());
            builder.memorized_simple_call(check_sha256, &data);
        }
    }
}

#[test]
fn test_sha256_37bytes() {
    let mut hint_registry = HintRegistry::<M31>::new();
    hint_registry.register("myhint.tobinary", to_binary_hint);
    let compile_result =
        compile(&SHA25637BYTESCircuit::default(), CompileOptions::default()).unwrap();
    for i in 0..1 {
        let data = [i; 37];
        let mut hash = Sha256::new();
        hash.update(data);
        let output = hash.finalize();
        let mut assignment = SHA25637BYTESCircuit::default();
        for i in 0..37 {
            assignment.input[i] = M31::from(data[i] as u32);
        }
        for i in 0..32 {
            assignment.output[i] = M31::from(output[i] as u32);
        }
        let witness = compile_result
            .witness_solver
            .solve_witness_with_hints(&assignment, &mut hint_registry)
            .unwrap();
        let start_time = std::time::Instant::now();
        let output = compile_result.layered_circuit.run(&witness);
        assert_eq!(output, vec![true]);
        let elapsed = start_time.elapsed();
        println!("Time elapsed in run() is: {:?}", elapsed);
    }
}

#[test]
fn debug_sha256_37bytes() {
    let mut hint_registry = HintRegistry::<M31>::new();
    hint_registry.register("myhint.tobinary", to_binary_hint);
    let data = [255; 37];
    let mut hash = Sha256::new();
    hash.update(data);
    let output = hash.finalize();
    let mut assignment = SHA25637BYTESCircuit::default();
    for i in 0..37 {
        assignment.input[i] = M31::from(data[i] as u32);
    }
    for i in 0..32 {
        assignment.output[i] = M31::from(output[i] as u32);
    }
    debug_eval(&SHA25637BYTESCircuit::default(), &assignment, hint_registry);
}
