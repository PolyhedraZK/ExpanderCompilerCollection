use circuit_std_rs::utils::register_hint;
use expander_compiler::frontend::*;
use expander_compiler::utils::serde::Serde;
use expander_compiler::zkcuda::proving_system::ExpanderGKRProvingSystem;
use expander_compiler::zkcuda::{context::*, kernel::*};
use circuit_std_rs::gnark::emulated::sw_bls12381::{g1::*, g2::*, pairing::*};
use std::time;
use ark_ff::fields::AdditiveGroup;
use std::ops::Mul;
use ark_serialize::CanonicalSerialize;
use ark_bls12_381::{
    Fr, G1Affine as BlsG1Affine, G1Projective, G2Affine as BlsG2Affine, G2Projective,
};
declare_circuit!(PairingCircuit {
    pubkey: [[Variable; 48]; 2],
    hm: [[[Variable; 48]; 2]; 2],
    sig: [[[Variable; 48]; 2]; 2]
});

#[kernel]
fn pairing_check<C: Config>(
    builder: &mut API<C>,
    input: &[InputVariable; 48*10],
    output: &mut [OutputVariable; 1],
) -> Vec<Variable> {
    let mut pairing = Pairing::new(builder);
    let one_g1 = G1Affine::one(builder);
    let pubkey_g1 = G1Affine::from_vars(input[0..48].to_vec(), input[48*1..48*2].to_vec());
    let hm_g2 = G2AffP::from_vars(
        input[48*2..48*3].to_vec(),
        input[48*3..48*4].to_vec(),
        input[48*4..48*5].to_vec(),
        input[48*5..48*6].to_vec(),
    );
    let sig_g2 = G2AffP::from_vars(
        input[48*6..48*7].to_vec(),
        input[48*7..48*8].to_vec(),
        input[48*8..48*9].to_vec(),
        input[48*9..48*10].to_vec(),
    );

    let mut g2 = G2::new(builder);
    let neg_sig_g2 = g2.neg(builder, &sig_g2);

    let p_array = vec![one_g1, pubkey_g1];
    let mut q_array = [
        G2Affine {
            p: neg_sig_g2,
            lines: LineEvaluations::default(),
        },
        G2Affine {
            p: hm_g2,
            lines: LineEvaluations::default(),
        },
    ];
    pairing
        .pairing_check(builder, &p_array, &mut q_array)
        .unwrap();
    pairing.ext12.ext6.ext2.curve_f.check_mul(builder);
    pairing.ext12.ext6.ext2.curve_f.table.final_check(builder);
    pairing.ext12.ext6.ext2.curve_f.table.final_check(builder);
    pairing.ext12.ext6.ext2.curve_f.table.final_check(builder);
    output[0] = builder.constant(1);
}


fn test_g1g2_affines() -> (BlsG1Affine, BlsG2Affine) {
    // Generate the generators for G1 and G2
    let g1_generator = BlsG1Affine::new(
        ark_bls12_381::g1::G1_GENERATOR_X,
        ark_bls12_381::g1::G1_GENERATOR_Y,
    );

    let g2_generator = BlsG2Affine::new(
        ark_bls12_381::g2::G2_GENERATOR_X,
        ark_bls12_381::g2::G2_GENERATOR_Y,
    );

    let random_scalar: Fr = Fr::from(1234);
    let p: BlsG1Affine = G1Projective::from(g1_generator).into();
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
}fn affine_point_to_bytes_g2(point: &BlsG2Affine) -> [[[u8; 48]; 2]; 2] {
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
#[test]
fn zkcuda_pairing_hint() {
    let mut hint_registry = HintRegistry::<M31>::new();
    register_hint(&mut hint_registry);
    let kernel_pairing_check = 
    if std::fs::metadata("kernel_pairing_check.txt").is_ok() {
        let file = std::fs::File::open("kernel_pairing_check.txt").unwrap();
        let reader = std::io::BufReader::new(file);
        Kernel::<M31Config>::deserialize_from(reader).unwrap()
    } else {
        let kernel_pairing_check: Kernel<M31Config> = compile_pairing_check().unwrap();
        let kernel_name = "kernel_pairing_check.txt";
        let file = std::fs::File::create(&kernel_name).unwrap();
        let writer = std::io::BufWriter::new(file);
        kernel_pairing_check.serialize_into(writer).unwrap();
        kernel_pairing_check
    };
    // let witness_solver = kernel_pairing_check.clone().witness_solver;
    // let lc_stats =  witness_solver.get_stats();

    // print_info("built layered circuit");
    // print_stat("num_terms", lc_stats.num_terms, false);
    // print_stat("num_insns", lc_stats.num_insns, false);
    // print_stat("numUsedInputs", lc_stats.num_inputs, false);
    // print_stat("num_constraints", lc_stats.num_constraints, false);
    // print_stat("num_variables", lc_stats.num_variables, false);
    println!("compile_pairing_check() done");
    let repeat_time = 1;
    let (p1, mut q1) = test_g1g2_affines();
    let p2: BlsG1Affine = G1Projective::from(p1).double().into();
    let q2 = q1;
    q1 = G2Projective::from(q1).double().into();
    let p2_bytes = affine_point_to_bytes_g1(&p2);
    let q1_bytes = affine_point_to_bytes_g2(&q1);
    let q2_bytes = affine_point_to_bytes_g2(&q2);
    let mut input_vars = vec![];
    let mut output_vars = vec![];

    for i in 0..2 {
        for j in 0..48 {
            input_vars.push(M31::from(p2_bytes[i][j] as u32));
        }
    }
    for i in 0..2 {
        for j in 0..2 {
            for k in 0..48 {
                input_vars.push(M31::from(q1_bytes[i][j][k] as u32));
            }
        }
    }
    for i in 0..2 {
        for j in 0..2 {
            for k in 0..48 {
                input_vars.push(M31::from(q2_bytes[i][j][k] as u32));
            }
        }
    }
    output_vars.push(M31::from(1));
    let mut new_input_vars = vec![];
    for _ in 0..repeat_time {
        new_input_vars.push(input_vars.clone());
    }
    let mut ctx: Context<M31Config, ExpanderGKRProvingSystem<M31Config>, _> = Context::new(hint_registry);

    let a = ctx.copy_to_device(&new_input_vars, false);
    let mut c = None;
    let start_time = time::Instant::now();
    call_kernel!(ctx, kernel_pairing_check, a, mut c);
    let elapsed = start_time.elapsed();
    println!("Time elapsed in call_kernel!() is: {:?}", elapsed);
    // let c = c.reshape(&[repeat_time, 32]);
    let result: Vec<Vec<M31>> = ctx.copy_to_host(c);
    for i in 0..repeat_time {
        assert_eq!(result[i], output_vars);
    }
}
fn print_info(info: &str) {
    print!(
        "\x1b[90m{}\x1b[0m \x1b[32mINF\x1b[0m {} ",
        chrono::Local::now().format("%H:%M:%S"),
        info
    );
}

fn print_stat(stat_name: &str, stat: usize, is_last: bool) {
    print!("\x1b[36m{}=\x1b[0m{}", stat_name, stat);
    if !is_last {
        print!(" ");
    } else {
        println!();
    }
}



#[kernel]
fn g1_add_zkcuda<C: Config>(
    builder: &mut API<C>,
    input: &[InputVariable; 48*6],
    output: &mut [OutputVariable; 1],
) -> Vec<Variable> {
    let mut g1 = G1::new(builder);
    let p1_g1 = G1Affine::from_vars(input[0..48].to_vec(), input[48*1..48*2].to_vec());
    let p2_g1 = G1Affine::from_vars(input[48*2..48*3].to_vec(), input[48*3..48*4].to_vec());
    let r_g1 = G1Affine::from_vars(input[48*4..48*5].to_vec(), input[48*5..48*6].to_vec());
    let mut r = g1.add(builder, &p1_g1, &p2_g1);
    // for _ in 0..16 {
    //     r = g1.add(builder, &r, &p2_g1);
    // }
    g1.curve_f.assert_is_equal(builder, &r.x, &r_g1.x);
    g1.curve_f.assert_is_equal(builder, &r.y, &r_g1.y);
    g1.curve_f.check_mul(builder);
    g1.curve_f.table.final_check(builder);
    g1.curve_f.table.final_check(builder);
    g1.curve_f.table.final_check(builder);
    output[0] = builder.constant(1);
    builder.assert_is_equal(output[0], input[0]);
}



#[test]
fn test_g1_add_zkcuda() {
    let mut hint_registry = HintRegistry::<M31>::new();
    register_hint(&mut hint_registry);
    let file_name = "kernel_g1_add_zkcuda.txt";
    let kernel_g1_add = 
    if std::fs::metadata(file_name).is_ok() {
        let file = std::fs::File::open(file_name).unwrap();
        let reader = std::io::BufReader::new(file);
        Kernel::<M31Config>::deserialize_from(reader).unwrap()
    } else {
        let kernel_pairing_check: Kernel<M31Config> = compile_g1_add_zkcuda().unwrap();
        let file = std::fs::File::create(&file_name).unwrap();
        let writer = std::io::BufWriter::new(file);
        kernel_pairing_check.serialize_into(writer).unwrap();
        kernel_pairing_check
    };

    let p1_x_bytes = [
        169, 204, 143, 202, 195, 182, 32, 187, 150, 46, 27, 88, 137, 82, 209, 11, 255, 228, 147,
        72, 218, 149, 56, 139, 243, 28, 49, 146, 210, 5, 238, 232, 111, 204, 78, 170, 83, 191, 222,
        173, 137, 165, 150, 240, 62, 27, 213, 8,
    ];
    let p1_y_bytes = [
        85, 56, 238, 125, 65, 131, 108, 201, 186, 2, 96, 151, 226, 80, 22, 2, 111, 141, 203, 67,
        50, 147, 209, 102, 238, 82, 12, 96, 172, 239, 2, 177, 184, 146, 208, 150, 63, 214, 239,
        198, 101, 74, 169, 226, 148, 53, 104, 1,
    ];
    let p2_x_bytes = [
        108, 4, 52, 16, 255, 115, 116, 198, 234, 60, 202, 181, 169, 240, 221, 33, 38, 178, 114,
        195, 169, 16, 147, 33, 62, 116, 10, 191, 25, 163, 79, 192, 140, 43, 109, 235, 157, 42, 15,
        48, 115, 213, 48, 51, 19, 165, 178, 17,
    ];
    let p2_y_bytes = [
        130, 146, 65, 1, 211, 117, 217, 145, 69, 140, 76, 106, 43, 160, 192, 247, 96, 225, 2, 72,
        219, 238, 254, 202, 9, 210, 253, 111, 73, 49, 26, 145, 68, 161, 64, 101, 238, 0, 236, 128,
        164, 92, 95, 30, 143, 178, 6, 20,
    ];
    let res_x_bytes = [
        148, 92, 212, 64, 35, 246, 218, 14, 150, 169, 177, 191, 61, 6, 4, 120, 60, 253, 36, 139,
        95, 95, 14, 122, 89, 3, 62, 198, 100, 50, 114, 221, 144, 187, 29, 15, 203, 89, 220, 29,
        120, 25, 153, 169, 184, 184, 133, 16,
    ];
    let res_y_bytes = [
        41, 226, 254, 238, 50, 145, 74, 128, 160, 125, 237, 161, 93, 66, 241, 104, 218, 230, 154,
        134, 24, 204, 225, 220, 175, 115, 243, 57, 238, 157, 161, 175, 213, 34, 145, 106, 226, 230,
        19, 110, 196, 196, 229, 104, 152, 64, 12, 6,
    ];
    let mut input_vars = vec![];
    let mut output_vars = vec![];
    for i in 0..48 {
        input_vars.push(M31::from(p1_x_bytes[i]));
    }
    for i in 0..48 {
        input_vars.push(M31::from(p1_y_bytes[i]));
    }
    for i in 0..48 {
        input_vars.push(M31::from(p2_x_bytes[i]));
    }
    for i in 0..48 {
        input_vars.push(M31::from(p2_y_bytes[i]));
    }
    for i in 0..48 {
        input_vars.push(M31::from(res_x_bytes[i]));
    }
    for i in 0..48 {
        input_vars.push(M31::from(res_y_bytes[i]));
    }
    output_vars.push(M31::from(1));
    let mut new_input_vars = vec![];
    let repeat_time = 1;
    for _ in 0..repeat_time {
        new_input_vars.push(input_vars.clone());
    }
    let mut ctx: Context<M31Config, ExpanderGKRProvingSystem<M31Config>, _> = Context::new(hint_registry);

    let a = ctx.copy_to_device(&new_input_vars, false);
    let mut c = None;
    let start_time = time::Instant::now();
    call_kernel!(ctx, kernel_g1_add, a, mut c);
    let elapsed = start_time.elapsed();
    println!("Time elapsed in call_kernel!() is: {:?}", elapsed);
    // let c = c.reshape(&[repeat_time, 32]);
    let result: Vec<Vec<M31>> = ctx.copy_to_host(c);
    for i in 0..repeat_time {
        assert_eq!(result[i], output_vars);
    }
}