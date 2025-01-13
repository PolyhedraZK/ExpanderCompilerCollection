#[cfg(test)]
mod tests {
    use circuit_std_rs::gnark::{element::{from_interface, value_of}, emparam::Bls12381Fp};
    use expander_compiler::frontend::*;
    use num_bigint::BigInt;
    #[test]
    fn test_from_interface() {
        let v = 1111111u32;
        let r = from_interface(Box::new(v));
        assert_eq!(r, BigInt::from(1111111u32));
        let v = 22222222222222u64;
        let r = from_interface(Box::new(v));
        assert_eq!(r, BigInt::from(22222222222222u64));
        let v = 333333usize;
        let r = from_interface(Box::new(v));
        assert_eq!(r, BigInt::from(333333usize as u64));
        let v = 444444i32;
        let r = from_interface(Box::new(v));
        assert_eq!(r, BigInt::from(444444i32));
        let v = 555555555555555i64;
        let r = from_interface(Box::new(v));
        assert_eq!(r, BigInt::from(555555555555555i64));
        let v = 666isize;
        let r = from_interface(Box::new(v));
        assert_eq!(r, BigInt::from(666isize as i64));
        let v = "77777777777777777".to_string();
        let r = from_interface(Box::new(v));
        assert_eq!(r, BigInt::from(77777777777777777u64));
        let v = vec![7u8; 4];
        let r = from_interface(Box::new(v));
        assert_eq!(r, BigInt::from(0x07070707u32));
    }

    declare_circuit!(VALUECircuit {
        target: [[Variable; 48]; 8],
    });
    impl Define<M31Config> for VALUECircuit<Variable> {
        fn define(&self, builder: &mut API<M31Config>) {
            let v1 = 1111111u32;
            let v2 = 22222222222222u64;
            let v3 = 333333usize;
            let v4 = 444444i32;
            let v5 = 555555555555555i64;
            let v6 = 666isize;
            let v7 = "77777777777777777".to_string();
            let v8 = vec![8u8; 4];

            let r1 = value_of::<M31Config, _, Bls12381Fp>(builder, Box::new(v1));
            let r2 = value_of::<M31Config, _, Bls12381Fp>(builder, Box::new(v2));
            let r3 = value_of::<M31Config, _, Bls12381Fp>(builder, Box::new(v3));
            let r4 = value_of::<M31Config, _, Bls12381Fp>(builder, Box::new(v4));
            let r5 = value_of::<M31Config, _, Bls12381Fp>(builder, Box::new(v5));
            let r6 = value_of::<M31Config, _, Bls12381Fp>(builder, Box::new(v6));
            let r7 = value_of::<M31Config, _, Bls12381Fp>(builder, Box::new(v7));
            let r8 = value_of::<M31Config, _, Bls12381Fp>(builder, Box::new(v8));
            let rs = vec![r1, r2, r3, r4, r5, r6, r7, r8];
            for i in 0..rs.len() {
                for j in 0..rs[i].limbs.len() {
                    builder.assert_is_equal(rs[i].limbs[j], self.target[i][j]);
                }
            }
        }
    }

    #[test]
    fn test_value() {
        let values: Vec<u64> = vec![
            1111111,
            22222222222222,
            333333,
            444444,
            555555555555555,
            666,
            77777777777777777,
            0x08080808,
        ];
        let values_u8: Vec<Vec<u8>> = values.iter().map(|v| v.to_le_bytes().to_vec()).collect();
        let compile_result = compile(&VALUECircuit::default()).unwrap();
        let mut assignment = VALUECircuit::<M31>::default();
        for i in 0..values_u8.len() {
            for j in 0..values_u8[i].len() {
                assignment.target[i][j] = M31::from(values_u8[i][j] as u32);
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