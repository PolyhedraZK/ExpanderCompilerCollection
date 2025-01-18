use circuit_std_rs::{
    gnark::emulated::{field_bls12381::e2::GE2, sw_bls12381::g2::*},
    utils::register_hint,
};
use expander_compiler::{
    declare_circuit,
    frontend::{extra::debug_eval, GenericDefine, HintRegistry, M31Config, RootAPI, Variable, M31},
};

declare_circuit!(MapToG2Circuit {
    in0: [[Variable; 48]; 2],
    in1: [[Variable; 48]; 2],
    out: [[[Variable; 48]; 2]; 2],
});

impl GenericDefine<M31Config> for MapToG2Circuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let mut g2 = G2::new(builder);
        let in0 = GE2::from_vars(self.in0[0].to_vec(), self.in0[1].to_vec());
        let in1 = GE2::from_vars(self.in1[0].to_vec(), self.in1[1].to_vec());
        let res = g2.map_to_g2(builder, &in0, &in1);
        let target_out = G2AffP {
            x: GE2::from_vars(self.out[0][0].to_vec(), self.out[0][1].to_vec()),
            y: GE2::from_vars(self.out[1][0].to_vec(), self.out[1][1].to_vec()),
        };
        g2.assert_is_equal(builder, &res, &target_out);
        g2.ext2.curve_f.check_mul(builder);
        g2.ext2.curve_f.table.final_check(builder);
        g2.ext2.curve_f.table.final_check(builder);
        g2.ext2.curve_f.table.final_check(builder);
    }
}

#[test]
fn test_map_to_g2() {
    let mut hint_registry = HintRegistry::<M31>::new();
    register_hint(&mut hint_registry);
    let mut assignment = MapToG2Circuit::<M31> {
        in0: [[M31::from(0); 48]; 2],
        in1: [[M31::from(0); 48]; 2],
        out: [[[M31::from(0); 48]; 2]; 2],
    };
    let p1_x_bytes = [
        75, 240, 55, 239, 72, 231, 76, 188, 20, 26, 234, 236, 23, 166, 182, 159, 239, 165, 10, 98,
        220, 117, 40, 167, 160, 143, 63, 57, 113, 82, 97, 238, 36, 48, 226, 19, 210, 13, 216, 163,
        51, 199, 31, 228, 211, 18, 125, 25,
    ];
    let p1_y_bytes = [
        161, 161, 201, 159, 90, 241, 214, 89, 177, 71, 235, 130, 168, 37, 237, 255, 26, 105, 22,
        122, 136, 28, 83, 245, 117, 135, 212, 63, 208, 241, 109, 4, 109, 188, 74, 50, 63, 41, 78,
        174, 164, 121, 104, 77, 56, 23, 100, 5,
    ];
    let p2_x_bytes = [
        161, 152, 122, 79, 206, 47, 160, 114, 196, 82, 17, 183, 227, 115, 71, 7, 9, 141, 33, 224,
        127, 254, 158, 109, 69, 225, 184, 146, 239, 137, 146, 138, 224, 79, 56, 100, 184, 236, 99,
        77, 28, 117, 111, 179, 106, 181, 35, 21,
    ];
    let p2_y_bytes = [
        199, 231, 196, 205, 165, 5, 112, 203, 238, 82, 8, 79, 245, 151, 226, 80, 154, 146, 230, 51,
        79, 60, 20, 190, 9, 171, 34, 41, 131, 165, 60, 0, 10, 197, 177, 140, 108, 41, 99, 113, 151,
        51, 253, 219, 105, 227, 25, 24,
    ];
    let out0_x_bytes = [
        215, 186, 167, 113, 176, 255, 84, 123, 163, 0, 104, 202, 139, 197, 29, 119, 253, 35, 206,
        68, 130, 75, 218, 109, 179, 63, 65, 197, 67, 206, 64, 89, 30, 201, 95, 238, 5, 66, 143, 94,
        37, 238, 150, 113, 159, 165, 110, 3,
    ];
    let out0_y_bytes = [
        88, 110, 24, 185, 208, 195, 142, 173, 176, 12, 228, 155, 64, 223, 147, 25, 37, 234, 200, 3,
        123, 119, 193, 221, 234, 253, 199, 190, 120, 135, 32, 215, 32, 118, 55, 230, 74, 204, 56,
        12, 24, 221, 240, 188, 188, 76, 233, 20,
    ];
    let out1_x_bytes = [
        202, 105, 74, 230, 255, 158, 238, 160, 121, 234, 219, 154, 239, 176, 232, 81, 56, 53, 154,
        76, 221, 53, 156, 165, 215, 18, 148, 34, 124, 242, 154, 218, 243, 171, 88, 53, 13, 182, 39,
        84, 254, 161, 96, 192, 154, 242, 71, 15,
    ];
    let out1_y_bytes = [
        66, 124, 60, 101, 29, 246, 150, 109, 233, 119, 212, 23, 132, 79, 170, 0, 178, 98, 151, 189,
        214, 70, 171, 93, 2, 98, 194, 243, 38, 160, 178, 224, 91, 20, 11, 209, 190, 76, 182, 253,
        89, 144, 170, 191, 128, 66, 207, 1,
    ];

    for i in 0..48 {
        assignment.in0[0][i] = M31::from(p1_x_bytes[i]);
        assignment.in0[1][i] = M31::from(p1_y_bytes[i]);
        assignment.in1[0][i] = M31::from(p2_x_bytes[i]);
        assignment.in1[1][i] = M31::from(p2_y_bytes[i]);
        assignment.out[0][0][i] = M31::from(out0_x_bytes[i]);
        assignment.out[0][1][i] = M31::from(out0_y_bytes[i]);
        assignment.out[1][0][i] = M31::from(out1_x_bytes[i]);
        assignment.out[1][1][i] = M31::from(out1_y_bytes[i]);
    }

    debug_eval(&MapToG2Circuit::default(), &assignment, hint_registry);
}

declare_circuit!(HashToG2Circuit {
    msg: [Variable; 32],
    out: [[[Variable; 48]; 2]; 2],
});

impl GenericDefine<M31Config> for HashToG2Circuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let mut g2 = G2::new(builder);
        let (hm0, hm1) = g2.hash_to_fp(builder, self.msg.to_vec());
        let res = g2.map_to_g2(builder, &hm0, &hm1);
        let target_out = G2AffP {
            x: GE2::from_vars(self.out[0][0].to_vec(), self.out[0][1].to_vec()),
            y: GE2::from_vars(self.out[1][0].to_vec(), self.out[1][1].to_vec()),
        };
        g2.assert_is_equal(builder, &res, &target_out);
        g2.ext2.curve_f.check_mul(builder);
        g2.ext2.curve_f.table.final_check(builder);
        g2.ext2.curve_f.table.final_check(builder);
        g2.ext2.curve_f.table.final_check(builder);
    }
}

#[test]
fn test_hash_to_g2() {
    let mut hint_registry = HintRegistry::<M31>::new();
    register_hint(&mut hint_registry);
    let mut assignment = HashToG2Circuit::<M31> {
        msg: [M31::from(0); 32],
        out: [[[M31::from(0); 48]; 2]; 2],
    };
    let msg_bytes = [
        140, 148, 79, 140, 170, 85, 208, 7, 114, 138, 47, 198, 231, 255, 48, 104, 221, 225, 3, 237,
        99, 251, 57, 156, 89, 194, 79, 31, 130, 109, 228, 200,
    ];
    let out0_x_bytes = [
        215, 186, 167, 113, 176, 255, 84, 123, 163, 0, 104, 202, 139, 197, 29, 119, 253, 35, 206,
        68, 130, 75, 218, 109, 179, 63, 65, 197, 67, 206, 64, 89, 30, 201, 95, 238, 5, 66, 143, 94,
        37, 238, 150, 113, 159, 165, 110, 3,
    ];
    let out0_y_bytes = [
        88, 110, 24, 185, 208, 195, 142, 173, 176, 12, 228, 155, 64, 223, 147, 25, 37, 234, 200, 3,
        123, 119, 193, 221, 234, 253, 199, 190, 120, 135, 32, 215, 32, 118, 55, 230, 74, 204, 56,
        12, 24, 221, 240, 188, 188, 76, 233, 20,
    ];
    let out1_x_bytes = [
        202, 105, 74, 230, 255, 158, 238, 160, 121, 234, 219, 154, 239, 176, 232, 81, 56, 53, 154,
        76, 221, 53, 156, 165, 215, 18, 148, 34, 124, 242, 154, 218, 243, 171, 88, 53, 13, 182, 39,
        84, 254, 161, 96, 192, 154, 242, 71, 15,
    ];
    let out1_y_bytes = [
        66, 124, 60, 101, 29, 246, 150, 109, 233, 119, 212, 23, 132, 79, 170, 0, 178, 98, 151, 189,
        214, 70, 171, 93, 2, 98, 194, 243, 38, 160, 178, 224, 91, 20, 11, 209, 190, 76, 182, 253,
        89, 144, 170, 191, 128, 66, 207, 1,
    ];
    for i in 0..32 {
        assignment.msg[i] = M31::from(msg_bytes[i]);
    }
    for i in 0..48 {
        assignment.out[0][0][i] = M31::from(out0_x_bytes[i]);
        assignment.out[0][1][i] = M31::from(out0_y_bytes[i]);
        assignment.out[1][0][i] = M31::from(out1_x_bytes[i]);
        assignment.out[1][1][i] = M31::from(out1_y_bytes[i]);
    }

    debug_eval(&HashToG2Circuit::default(), &assignment, hint_registry);
}
