use circuit_std_rs::{
    gnark::{
        element::Element,
        emulated::{
            field_bls12381::e2::GE2,
            sw_bls12381::{g1::*, g2::*, pairing::Pairing},
        },
    },
    utils::register_hint,
};
use expander_compiler::{
    declare_circuit,
    frontend::{extra::debug_eval, Define, HintRegistry, M31Config, RootAPI, Variable, M31},
};

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

#[test]
fn test_pairing_check_gkr() {
    // let compile_result =
    // compile_generic(&PairingCheckGKRCircuit::default(), CompileOptions::default()).unwrap();
    let mut hint_registry = HintRegistry::<M31>::new();
    register_hint(&mut hint_registry);
    let mut assignment = PairingCheckGKRCircuit::<M31> {
        in1_g1: [[M31::from(0); 48]; 2],
        in2_g1: [[M31::from(0); 48]; 2],
        in1_g2: [[[M31::from(0); 48]; 2]; 2],
        in2_g2: [[[M31::from(0); 48]; 2]; 2],
    };
    let p1_x_bytes = [
        138, 209, 41, 52, 20, 222, 185, 9, 48, 234, 53, 109, 218, 26, 76, 112, 204, 195, 135, 184,
        95, 253, 141, 179, 243, 220, 94, 195, 151, 34, 112, 210, 63, 186, 25, 221, 129, 128, 76,
        209, 101, 191, 44, 36, 248, 25, 127, 3,
    ];
    let p1_y_bytes = [
        97, 193, 54, 196, 208, 241, 229, 252, 144, 121, 89, 115, 226, 242, 251, 60, 142, 182, 216,
        242, 212, 30, 189, 82, 97, 228, 230, 80, 38, 19, 77, 187, 242, 96, 65, 136, 115, 75, 173,
        136, 35, 202, 199, 3, 37, 33, 182, 19,
    ];
    let p2_x_bytes = [
        53, 43, 44, 191, 248, 216, 253, 96, 84, 253, 43, 36, 151, 202, 77, 190, 19, 71, 28, 215,
        161, 72, 57, 211, 182, 58, 152, 199, 107, 235, 238, 63, 160, 97, 190, 43, 89, 195, 111,
        179, 72, 18, 109, 141, 133, 74, 215, 16,
    ];
    let p2_y_bytes = [
        96, 0, 147, 41, 253, 168, 205, 45, 124, 150, 80, 188, 171, 228, 217, 34, 233, 192, 87, 38,
        176, 98, 88, 196, 41, 115, 40, 174, 52, 234, 97, 53, 209, 179, 91, 66, 107, 130, 187, 171,
        10, 254, 6, 227, 50, 212, 34, 8,
    ];
    let q1_x0_bytes = [
        115, 71, 82, 0, 253, 98, 21, 231, 188, 204, 204, 250, 44, 169, 184, 249, 132, 60, 132, 14,
        34, 48, 165, 84, 111, 109, 143, 182, 32, 72, 227, 210, 133, 144, 154, 196, 16, 169, 138,
        79, 19, 122, 34, 156, 176, 236, 114, 22,
    ];
    let q1_x1_bytes = [
        182, 57, 221, 84, 50, 87, 48, 115, 6, 98, 38, 176, 152, 25, 126, 43, 201, 61, 87, 42, 225,
        138, 200, 170, 0, 20, 174, 117, 112, 157, 233, 97, 0, 149, 210, 18, 224, 229, 157, 26, 197,
        93, 245, 96, 227, 157, 237, 15,
    ];
    let q1_y0_bytes = [
        185, 67, 44, 184, 194, 122, 245, 73, 123, 160, 144, 28, 83, 227, 9, 222, 52, 33, 74, 97,
        66, 113, 234, 143, 125, 244, 115, 58, 79, 29, 83, 208, 130, 83, 146, 30, 95, 202, 3, 189,
        0, 6, 81, 73, 107, 141, 234, 1,
    ];
    let q1_y1_bytes = [
        113, 182, 199, 78, 243, 62, 126, 145, 147, 111, 153, 151, 219, 69, 54, 127, 72, 82, 59,
        169, 219, 65, 228, 8, 193, 143, 67, 158, 12, 45, 225, 109, 220, 217, 133, 185, 75, 245, 82,
        200, 137, 178, 165, 90, 190, 232, 244, 21,
    ];
    let q2_x0_bytes = [
        48, 100, 73, 236, 161, 161, 88, 235, 92, 188, 236, 139, 70, 238, 43, 160, 189, 118, 66,
        116, 44, 222, 23, 195, 67, 252, 105, 112, 240, 119, 247, 53, 3, 24, 156, 3, 178, 117, 41,
        16, 120, 114, 244, 103, 65, 157, 255, 21,
    ];
    let q2_x1_bytes = [
        87, 198, 239, 80, 28, 107, 195, 211, 220, 50, 148, 176, 2, 30, 65, 17, 206, 180, 103, 123,
        161, 64, 40, 77, 84, 98, 25, 164, 111, 180, 209, 62, 23, 78, 4, 174, 123, 52, 30, 19, 149,
        4, 6, 56, 6, 173, 138, 12,
    ];
    let q2_y0_bytes = [
        178, 164, 255, 33, 62, 219, 245, 30, 146, 252, 242, 196, 23, 5, 90, 103, 75, 9, 67, 186,
        155, 40, 106, 209, 158, 161, 142, 60, 109, 58, 29, 180, 3, 126, 95, 225, 244, 243, 36, 82,
        32, 223, 19, 39, 202, 170, 158, 12,
    ];
    let q2_y1_bytes = [
        47, 93, 130, 172, 91, 197, 69, 2, 220, 41, 78, 230, 47, 199, 202, 197, 177, 54, 53, 90,
        233, 76, 186, 248, 212, 121, 120, 208, 231, 195, 87, 150, 233, 33, 103, 94, 11, 15, 108,
        247, 78, 10, 223, 139, 186, 5, 53, 8,
    ];

    for i in 0..48 {
        assignment.in1_g1[0][i] = M31::from(p1_x_bytes[i]);
        assignment.in1_g1[1][i] = M31::from(p1_y_bytes[i]);
        assignment.in2_g1[0][i] = M31::from(p2_x_bytes[i]);
        assignment.in2_g1[1][i] = M31::from(p2_y_bytes[i]);
        assignment.in1_g2[0][0][i] = M31::from(q1_x0_bytes[i]);
        assignment.in1_g2[0][1][i] = M31::from(q1_x1_bytes[i]);
        assignment.in1_g2[1][0][i] = M31::from(q1_y0_bytes[i]);
        assignment.in1_g2[1][1][i] = M31::from(q1_y1_bytes[i]);
        assignment.in2_g2[0][0][i] = M31::from(q2_x0_bytes[i]);
        assignment.in2_g2[0][1][i] = M31::from(q2_x1_bytes[i]);
        assignment.in2_g2[1][0][i] = M31::from(q2_y0_bytes[i]);
        assignment.in2_g2[1][1][i] = M31::from(q2_y1_bytes[i]);
    }

    debug_eval(
        &PairingCheckGKRCircuit::default(),
        &assignment,
        hint_registry,
    );
}
