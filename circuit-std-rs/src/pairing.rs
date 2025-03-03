use crate::{
    gnark::{
        element::Element,
        emparam::Bls12381Fp,
        emulated::{
            field_bls12381::e2::GE2,
            sw_bls12381::{g1::*, g2::*, pairing::Pairing, point::AffinePoint},
        },
    },
    StdCircuit,
};
use ark_bls12_381::{
    Bls12_381, Fr, G1Affine as BlsG1Affine, G1Projective, G2Affine as BlsG2Affine, G2Projective,
};
use ark_ec::{AffineCurve, CurveCycle, ProjectiveCurve};
use ark_ff::{AdditiveGroup, PrimeField, UniformRand};

use expander_compiler::{
    declare_circuit,
    frontend::{extra::debug_eval, Define, HintRegistry, M31Config, RootAPI, Variable, M31},
};
use std::ops::{Mul, Neg};

use ark_serialize::CanonicalSerialize;

#[derive(Clone, Debug)]
pub struct PairingParams {
    pub in1_g1: G1Affine,
    pub in2_g1: G1Affine,
    pub in1_g2: G2Affine,
    pub in2_g2: G2Affine,
}

declare_circuit!(PairingCheckGKRCircuit {
    in1_g1: [[Variable; 48]; 2],
    in2_g1: [[Variable; 48]; 2],
    in1_g2: [[[Variable; 48]; 2]; 2],
    in2_g2: [[[Variable; 48]; 2]; 2],
});

impl Define<M31Config> for PairingCheckGKRCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let mut pairing = Pairing::new(builder);
        let p1_g1 = G1Affine {
            x: Element::new(
                self.in1_g1[0].to_vec(),
                0,
                false,
                false,
                false,
                Variable::default(),
            ),
            y: Element::new(
                self.in1_g1[1].to_vec(),
                0,
                false,
                false,
                false,
                Variable::default(),
            ),
        };
        let p2_g1 = G1Affine {
            x: Element::new(
                self.in2_g1[0].to_vec(),
                0,
                false,
                false,
                false,
                Variable::default(),
            ),
            y: Element::new(
                self.in2_g1[1].to_vec(),
                0,
                false,
                false,
                false,
                Variable::default(),
            ),
        };
        let q1_g2 = G2AffP {
            x: GE2 {
                a0: Element::new(
                    self.in1_g2[0][0].to_vec(),
                    0,
                    false,
                    false,
                    false,
                    Variable::default(),
                ),
                a1: Element::new(
                    self.in1_g2[0][1].to_vec(),
                    0,
                    false,
                    false,
                    false,
                    Variable::default(),
                ),
            },
            y: GE2 {
                a0: Element::new(
                    self.in1_g2[1][0].to_vec(),
                    0,
                    false,
                    false,
                    false,
                    Variable::default(),
                ),
                a1: Element::new(
                    self.in1_g2[1][1].to_vec(),
                    0,
                    false,
                    false,
                    false,
                    Variable::default(),
                ),
            },
        };
        let q2_g2 = G2AffP {
            x: GE2 {
                a0: Element::new(
                    self.in2_g2[0][0].to_vec(),
                    0,
                    false,
                    false,
                    false,
                    Variable::default(),
                ),
                a1: Element::new(
                    self.in2_g2[0][1].to_vec(),
                    0,
                    false,
                    false,
                    false,
                    Variable::default(),
                ),
            },
            y: GE2 {
                a0: Element::new(
                    self.in2_g2[1][0].to_vec(),
                    0,
                    false,
                    false,
                    false,
                    Variable::default(),
                ),
                a1: Element::new(
                    self.in2_g2[1][1].to_vec(),
                    0,
                    false,
                    false,
                    false,
                    Variable::default(),
                ),
            },
        };
        pairing.assert_is_on_curve(builder, p1_g1.clone());
        pairing.assert_is_on_curve(builder, p2_g1.clone());

        pairing
            .pairing_check(
                builder,
                &[p1_g1, p2_g1],
                &mut [
                    G2Affine {
                        p: q1_g2,
                        lines: LineEvaluations::default(),
                    },
                    G2Affine {
                        p: q2_g2,
                        lines: LineEvaluations::default(),
                    },
                ],
            )
            .unwrap();
        pairing.ext12.ext6.ext2.curve_f.check_mul(builder);
        pairing.ext12.ext6.ext2.curve_f.table.final_check(builder);
        pairing.ext12.ext6.ext2.curve_f.table.final_check(builder);
        pairing.ext12.ext6.ext2.curve_f.table.final_check(builder);
    }
}

impl StdCircuit<M31Config> for PairingCheckGKRCircuit<Variable> {
    type Params = PairingParams;
    type Assignment = PairingCheckGKRCircuit<<expander_compiler::frontend::M31Config as expander_compiler::frontend::Config>::CircuitField>;

    fn new_circuit(params: &Self::Params) -> Self {
        // let mut circuit = Self::default();
        // circuit.in1_g1 = params.in1_g1;
        // circuit.in2_g1 = params.in2_g1;
        // circuit.in1_g2 = params.in1_g2;

        // circuit.table_values.resize(
        //     params.n_table_rows,
        //     vec![Variable::default(); params.value_len],
        // );

        todo!()
    }

    fn new_assignment(params: &Self::Params, rng: impl rand::RngCore) -> Self::Assignment {
        let (p1, mut q1) = random_g1g2_affines();
        let mut p2: BlsG1Affine = G1Projective::from(p1).double().neg().into();
        let mut q2 = q1;
        q1 = G2Projective::from(q1).double().into();

        let mut assignment = PairingCheckGKRCircuit::default();
        let p1_bytes = affine_point_to_bytes_g1(&p1);
        let p2_bytes = affine_point_to_bytes_g1(&p2);
        let q1_bytes = affine_point_to_bytes_g2(&q1);
        let q2_bytes = affine_point_to_bytes_g2(&q2);

        assignment.in1_g1[0] = convert_to_m31(p1_bytes[0]);
        assignment.in1_g1[1] = convert_to_m31(p1_bytes[1]);
        assignment.in2_g1[0] = convert_to_m31(p2_bytes[0]);
        assignment.in2_g1[1] = convert_to_m31(p2_bytes[1]);
        assignment.in1_g2[0][0] = convert_to_m31(q1_bytes[0][0]);
        assignment.in1_g2[0][1] = convert_to_m31(q1_bytes[0][1]);
        assignment.in1_g2[1][0] = convert_to_m31(q1_bytes[1][0]);
        assignment.in1_g2[1][1] = convert_to_m31(q1_bytes[1][1]);

        assignment.in2_g2[0][0] = convert_to_m31(q2_bytes[0][0]);
        assignment.in2_g2[0][1] = convert_to_m31(q2_bytes[0][1]);
        assignment.in2_g2[1][0] = convert_to_m31(q2_bytes[1][0]);
        assignment.in2_g2[1][1] = convert_to_m31(q2_bytes[1][1]);

        assignment
    }
}

fn random_g1g2_affines() -> (BlsG1Affine, BlsG2Affine) {
    // Generate the generators for G1 and G2
    let g1_generator = BlsG1Affine::new(
        ark_bls12_381::g1::G1_GENERATOR_X,
        ark_bls12_381::g1::G1_GENERATOR_Y,
    );

    let g2_generator = BlsG2Affine::new(
        ark_bls12_381::g2::G2_GENERATOR_X,
        ark_bls12_381::g2::G2_GENERATOR_Y,
    );

    // Create a random number generator
    let mut rng = ark_std::rand::thread_rng();
    let random_scalar: Fr = Fr::rand(&mut rng);

    let p: BlsG1Affine = G1Projective::from(g1_generator).mul(random_scalar).into();
    let q: BlsG2Affine = G2Projective::from(g2_generator).mul(random_scalar).into();

    (p, q)
}

fn affine_point_to_bytes_g1(point: &BlsG1Affine) -> [[u8; 48]; 2] {
    let mut x_bytes = [0u8; 48];
    let mut y_bytes = [0u8; 48];

    // serialize x
    point.x.serialize_compressed(&mut x_bytes.as_mut()).unwrap();

    //serialize y
    point.y.serialize_compressed(&mut y_bytes.as_mut()).unwrap();

    [x_bytes, y_bytes]
}

fn affine_point_to_bytes_g2(point: &BlsG2Affine) -> [[[u8; 48]; 2]; 2] {
    let mut x_bytes = [[0u8; 48]; 2];
    let mut y_bytes = [[0u8; 48]; 2];

    // serialize x
    point
        .x
        .c0
        .serialize_compressed(&mut x_bytes[0].as_mut())
        .unwrap(); // x.c0
    point
        .x
        .c1
        .serialize_compressed(&mut x_bytes[1].as_mut())
        .unwrap(); // x.c1

    // serialize x
    point
        .y
        .c0
        .serialize_compressed(&mut y_bytes[0].as_mut())
        .unwrap(); // y.c0
    point
        .y
        .c1
        .serialize_compressed(&mut y_bytes[1].as_mut())
        .unwrap(); // y.c1

    [x_bytes, y_bytes]
}

fn convert_to_m31(input: [u8; 48]) -> [M31; 48] {
    let mut output = [M31::default(); 48];

    for i in 0..48 {
        output[i] = M31::from(input[i] as u32);
    }

    output
}
