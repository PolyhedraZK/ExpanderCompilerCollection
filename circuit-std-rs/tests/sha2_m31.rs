use circuit_std_rs::{big_int::to_binary_hint, sha2_m31::{check_sha256_37bytes, sha256_var_bytes}};
use expander_compiler::frontend::*;
use extra::*;
use sha2::{Digest, Sha256};

declare_circuit!(SHA25637BYTESCircuit {
    input: [Variable; 37],
    output: [Variable; 32],
});
impl GenericDefine<M31Config> for SHA25637BYTESCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        for _ in 0..8 {
            let mut data = self.input.to_vec();
            data.append(&mut self.output.to_vec());
            check_sha256_37bytes(builder, &data);
        }
    }
}
#[test]
fn test_sha256_37bytes() {
    let mut hint_registry = HintRegistry::<M31>::new();
    hint_registry.register("myhint.tobinary", to_binary_hint);
    let compile_result =
        compile_generic(&SHA25637BYTESCircuit::default(), CompileOptions::default()).unwrap();
    for i in 0..1 {
        let data = [i; 37];
        let mut hash = Sha256::new();
        hash.update(&data);
        let output = hash.finalize();
        let mut assignment = SHA25637BYTESCircuit::default();
        for j in 0..37 {
            assignment.input[j] = M31::from(data[j] as u32);
        }
        for j in 0..32 {
            assignment.output[j] = M31::from(output[j] as u32);
        }
        let witness = compile_result
            .witness_solver
            .solve_witness_with_hints(&assignment, &mut hint_registry)
            .unwrap();
        let output = compile_result.layered_circuit.run(&witness);
        assert_eq!(output, vec![true]);
    }
}
#[test]
fn debug_sha256_37bytes() {
    let mut hint_registry = HintRegistry::<M31>::new();
    hint_registry.register("myhint.tobinary", to_binary_hint);
    let data = [255; 37];
    let mut hash = Sha256::new();
    hash.update(&data);
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


const INPUT_LEN : usize = 64+56;
declare_circuit!(SHA256Circuit {
    input: [Variable; INPUT_LEN],
    output: [Variable; 32],
});
impl GenericDefine<M31Config> for SHA256Circuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
		let data = self.input.to_vec();
		let hash = sha256_var_bytes(builder, &data);
		for i in 0..32 {
			println!("hash[{}]: {:?}, {:?}", i, builder.value_of(hash[i]), builder.value_of(self.output[i]));
			builder.assert_is_equal(hash[i], self.output[i]);
		}
    }
}

#[test]
fn test_sha256_var_bytes() {
    let mut hint_registry = HintRegistry::<M31>::new();
    hint_registry.register("myhint.tobinary", to_binary_hint);
    let data = [22; INPUT_LEN];
	let mut hash = Sha256::new();
	hash.update(&data);
	let output = hash.finalize();
	let mut assignment = SHA256Circuit::default();
	for j in 0..INPUT_LEN {
		assignment.input[j] = M31::from(data[j] as u32);
	}
	for j in 0..32 {
		assignment.output[j] = M31::from(output[j] as u32);
	}
    debug_eval(&SHA256Circuit::default(), &assignment, hint_registry);
}