use circuit_std_rs::sha256::{
  m31::{check_sha256_37bytes, sha256_37bytes_compiled},
  m31_utils::to_binary_hint,
};
use expander_compiler::frontend::*;
use extra::*;
use sha2::{Digest, Sha256};

declare_circuit!(SHA25637BYTESCircuit {
  input: [Variable; 37],
  output: [Variable; 32],
});

declare_circuit!(SHA25637BYTESCircuitCompiled {
  input: [Variable; 37],
  output: [Variable; 32],
});

macro_rules! declare_check {
  ($checker:ident, $builder:ident) => {
    pub fn $checker<C: Config, B: RootAPI<C>>(
      builder: &mut B,
      origin_data: &Vec<Variable>,
    ) -> Vec<Variable> {
      let output = origin_data[37..].to_vec();
      let result = $builder(builder, &origin_data[..37]);
      for i in 0..32 {
        builder.assert_is_equal(result[i], output[i]);
      }
      result
    }
  };
}

declare_check!(check_sha256_compiled, sha256_37bytes_compiled);

macro_rules! declare_define {
  ($circuit: ident, $checker: ident) => {
    impl Define<M31Config> for $circuit<Variable> {
      fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        for _ in 0..8 {
          let mut data = self.input.to_vec();
          data.append(&mut self.output.to_vec());
          builder.memorized_simple_call($checker, &data);
        }
      }
    }
  };
}

declare_define!(SHA25637BYTESCircuit, check_sha256_37bytes);
declare_define!(SHA25637BYTESCircuitCompiled, check_sha256_compiled);

macro_rules! test_case {
  ($case: ident, $circuit: ident) => {
    #[test]
    fn $case() {
      let mut hint_registry = HintRegistry::<M31>::new();
      hint_registry.register("myhint.tobinary", to_binary_hint);
      let circuit = $circuit::default();
      let compile_result = compile(&circuit, CompileOptions::default()).unwrap();
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
        let output = compile_result.layered_circuit.run(&witness);
        assert_eq!(output, vec![true]);
      }
    }
  };
}

test_case!(test_sha256_37bytes, SHA25637BYTESCircuit);
test_case!(test_sha256_37bytes_compiled, SHA25637BYTESCircuitCompiled);

#[test]
fn debug_sha256_37bytes() {
  let mut hint_registry = HintRegistry::<M31>::new();
  hint_registry.register("myhint.tobinary", to_binary_hint);
  let data = [0; 37];
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
