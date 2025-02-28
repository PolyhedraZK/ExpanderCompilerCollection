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

use expander_compiler::{
    declare_circuit,
    frontend::{extra::debug_eval, Define, HintRegistry, M31Config, RootAPI, Variable, M31},
};

#[derive(Clone, Copy, Debug)]
pub struct PairingParams {
    pub in1_g1: G1Affine,
    pub in2_g1: AffinePoint<Bls12381Fp>,
    pub in1_g2: AffinePoint<Bls12381Fp>,
    pub in2_g2: AffinePoint<Bls12381Fp>,
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

impl StdCircuit for PairingCheckGKRCircuit<Variable> {
    type Params = LogUpParams;
    type Assignment = _LogUpCircuit<C::CircuitField>;
}
