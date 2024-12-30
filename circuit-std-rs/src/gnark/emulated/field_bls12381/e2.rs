use crate::gnark::emparam::FieldParams;
use crate::gnark::element::*;
use crate::gnark::field::Field as GField;
use crate::gnark::emparam::*;
use crate::gnark::hints::{mul_hint, simple_rangecheck_hint};
use std::collections::HashMap;
use expander_compiler::frontend::extra::*;
use expander_compiler::{circuit::layered::InputType, frontend::*};

pub type CurveF = GField<bls12381_fp>;
#[derive(Default, Clone)]
pub struct GE2 {
    a0: Element<bls12381_fp>,
    a1: Element<bls12381_fp>,
}

pub struct Ext2 {
    pub fp: CurveF,
    non_residues: HashMap<u32, HashMap<u32, GE2>>,
}


impl Ext2{
    pub fn new<'a, C:Config, B:RootAPI<C>>(api: &'a mut B) -> Self {
        let mut non_residues:HashMap<u32, HashMap<u32, GE2>> = HashMap::new();
        let mut pwrs:HashMap<u32, HashMap<u32, GE2>> = HashMap::new();
        let a1_1_0 = value_of::<C, B, bls12381_fp>(api, Box::new("3850754370037169011952147076051364057158807420970682438676050522613628423219637725072182697113062777891589506424760".to_string()));
        let a1_1_1 = value_of::<C, B, bls12381_fp>(api, Box::new("151655185184498381465642749684540099398075398968325446656007613510403227271200139370504932015952886146304766135027".to_string()));
        let a1_2_0 = value_of::<C, B, bls12381_fp>(api, Box::new("0".to_string()));
        let a1_2_1 = value_of::<C, B, bls12381_fp>(api, Box::new("4002409555221667392624310435006688643935503118305586438271171395842971157480381377015405980053539358417135540939436".to_string()));
        let a1_3_0 = value_of::<C, B, bls12381_fp>(api, Box::new("1028732146235106349975324479215795277384839936929757896155643118032610843298655225875571310552543014690878354869257".to_string()));
        let a1_3_1 = value_of::<C, B, bls12381_fp>(api, Box::new("1028732146235106349975324479215795277384839936929757896155643118032610843298655225875571310552543014690878354869257".to_string()));
        let a1_4_0 = value_of::<C, B, bls12381_fp>(api, Box::new("4002409555221667392624310435006688643935503118305586438271171395842971157480381377015405980053539358417135540939437".to_string()));
        let a1_4_1 = value_of::<C, B, bls12381_fp>(api, Box::new("0".to_string()));
        let a1_5_0 = value_of::<C, B, bls12381_fp>(api, Box::new("877076961050607968509681729531255177986764537961432449499635504522207616027455086505066378536590128544573588734230".to_string()));
        let a1_5_1 = value_of::<C, B, bls12381_fp>(api, Box::new("3125332594171059424908108096204648978570118281977575435832422631601824034463382777937621250592425535493320683825557".to_string()));
        let a2_1_0 = value_of::<C, B, bls12381_fp>(api, Box::new("793479390729215512621379701633421447060886740281060493010456487427281649075476305620758731620351".to_string()));
        let a2_2_0 = value_of::<C, B, bls12381_fp>(api, Box::new("793479390729215512621379701633421447060886740281060493010456487427281649075476305620758731620350".to_string()));
        let a2_3_0 = value_of::<C, B, bls12381_fp>(api, Box::new("4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559786".to_string()));
        let a2_4_0 = value_of::<C, B, bls12381_fp>(api, Box::new("4002409555221667392624310435006688643935503118305586438271171395842971157480381377015405980053539358417135540939436".to_string()));
        let a2_5_0 = value_of::<C, B, bls12381_fp>(api, Box::new("4002409555221667392624310435006688643935503118305586438271171395842971157480381377015405980053539358417135540939437".to_string()));
        pwrs.insert(1, HashMap::new());
        pwrs.get_mut(&1).unwrap().insert(1, GE2 {
            a0: a1_1_0,
            a1: a1_1_1,
        });
        pwrs.get_mut(&1).unwrap().insert(2, GE2 {
            a0: a1_2_0,
            a1: a1_2_1,
        });
        pwrs.get_mut(&1).unwrap().insert(3, GE2 {
            a0: a1_3_0,
            a1: a1_3_1,
        });
        pwrs.get_mut(&1).unwrap().insert(4, GE2 {
            a0: a1_4_0,
            a1: a1_4_1,
        });
        pwrs.get_mut(&1).unwrap().insert(5, GE2 {
            a0: a1_5_0,
            a1: a1_5_1,
        });
        pwrs.insert(2, HashMap::new());
        let a_zero = value_of::<C, B, bls12381_fp>(api, Box::new("0".to_string()));
        pwrs.get_mut(&2).unwrap().insert(1, GE2 {
            a0: a2_1_0,
            a1: a_zero,
        });
        let a_zero = value_of::<C, B, bls12381_fp>(api, Box::new("0".to_string()));
        pwrs.get_mut(&2).unwrap().insert(2, GE2 {
            a0: a2_2_0,
            a1: a_zero,
        });
        let a_zero = value_of::<C, B, bls12381_fp>(api, Box::new("0".to_string()));
        pwrs.get_mut(&2).unwrap().insert(3, GE2 {
            a0: a2_3_0,
            a1: a_zero,
        });
        let a_zero = value_of::<C, B, bls12381_fp>(api, Box::new("0".to_string()));
        pwrs.get_mut(&2).unwrap().insert(4, GE2 {
            a0: a2_4_0,
            a1: a_zero,
        });
        let a_zero = value_of::<C, B, bls12381_fp>(api, Box::new("0".to_string()));
        pwrs.get_mut(&2).unwrap().insert(5, GE2 {
            a0: a2_5_0,
            a1: a_zero,
        });
        let fp = CurveF::new(api, bls12381_fp{});
        Ext2 {
            fp,
            non_residues: pwrs,
        }
    }
    pub fn add<'a, C:Config, B:RootAPI<C>>(&mut self, native: &'a mut B, x: &GE2, y: &GE2) -> GE2 {
        let z0 = self.fp.add(native, x.a0.clone(), y.a0.clone());
        let z1 = self.fp.add(native,x.a1.clone(), y.a1.clone());
        GE2 {
            a0: z0,
            a1: z1,
        }
    }
}

declare_circuit!(E2AddCircuit {
    x: [[Variable; 48];2],
    y: [[Variable; 48];2],
    z: [[Variable; 48];2],
});

impl GenericDefine<M31Config> for E2AddCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let mut ext2 = Ext2::new(builder);
        let x_e2 = GE2 {
            a0: new_internal_element(self.x[0].to_vec(), 0),
            a1: new_internal_element(self.x[1].to_vec(), 0),
        };
        let y_e2 = GE2 {
            a0: new_internal_element(self.y[0].to_vec(), 0),
            a1: new_internal_element(self.y[1].to_vec(), 0),
        };
        let z = ext2.add(builder, &x_e2, &y_e2);
        let z_reduce_a0 = ext2.fp.reduce(builder, z.a0.clone(), false);
        let z_reduce_a1 = ext2.fp.reduce(builder, z.a1.clone(), false);

        for i in 0..48 {
            // println!("{}: {:?} {:?}", i, builder.value_of(z.a0.limbs[i]), builder.value_of(z.a1.limbs[i]));
            // println!("{}: {:?} {:?}", i, builder.value_of(z_reduce_a0.limbs[i]), builder.value_of(z_reduce_a1.limbs[i]));
            println!("{}: {:?} {:?}", i, builder.value_of(z_reduce_a0.limbs[i]), builder.value_of(self.z[0][i]));
            println!("{}: {:?} {:?}", i, builder.value_of(z_reduce_a1.limbs[i]), builder.value_of(self.z[1][i]));
            builder.assert_is_equal(z_reduce_a0.limbs[i], self.z[0][i]);
            builder.assert_is_equal(z_reduce_a1.limbs[i], self.z[1][i]);
        }
    }
}

#[test]
fn test_e2_add() {
    // let compile_result = compile(&E2AddCircuit::default()).unwrap();
	let mut hint_registry = HintRegistry::<M31>::new();
	hint_registry.register("myhint.mulhint", mul_hint);
	hint_registry.register("myhint.simple_rangecheck_hint", simple_rangecheck_hint);
    let mut assignment = E2AddCircuit::<M31> {
        x: [[M31::from(0); 48], [M31::from(0); 48]],
        y: [[M31::from(0); 48], [M31::from(0); 48]],
        z: [[M31::from(0); 48], [M31::from(0); 48]],
    };
    // let x0_bytes = [89 156 69 194 144 213 244 116 63 190 210 105 4 3 175 7 101 54 28 7 18 172 79 84 237 54 73 82 129 140 106 156 148 208 55 92 9 173 33 66 123 235 204 136 44 150 98 10];
    let x0_bytes = [89, 156, 69, 194, 144, 213, 244, 116, 63, 190, 210, 105, 4, 3, 175, 7, 101, 54, 28, 7, 18, 172, 79, 84, 237, 54, 73, 82, 129, 140, 106, 156, 148, 208, 55, 92, 9, 173, 33, 66, 123, 235, 204, 136, 44, 150, 98, 10];
    println!("x0-string:{}", hex::encode(x0_bytes));
    // let x1_bytes = [236 205 45 143 165 12 10 61 83 59 118 233 115 199 99 173 46 152 211 133 250 124 121 183 156 51 67 26 197 238 173 72 255 131 102 60 79 157 114 50 88 209 73 233 20 196 157 18] 
    let x1_bytes = [236, 205, 45, 143, 165, 12, 10, 61, 83, 59, 118, 233, 115, 199, 99, 173, 46, 152, 211, 133, 250, 124, 121, 183, 156, 51, 67, 26, 197, 238, 173, 72, 255, 131, 102, 60, 79, 157, 114, 50, 88, 209, 73, 233, 20, 196, 157, 18];
    println!("x1-string:{}", hex::encode(x1_bytes));
    // let y0_bytes = [101 10 8 84 22 11 97 20 107 192 229 172 173 2 120 227 179 177 150 202 54 114 18 66 169 184 198 77 8 75 97 100 206 62 149 101 48 222 77 137 6 205 25 24 76 102 118 25] 
    let y0_bytes = [101, 10, 8, 84, 22, 11, 97, 20, 107, 192, 229, 172, 173, 2, 120, 227, 179, 177, 150, 202, 54, 114, 18, 66, 169, 184, 198, 77, 8, 75, 97, 100, 206, 62, 149, 101, 48, 222, 77, 137, 6, 205, 25, 24, 76, 102, 118, 25];
    println!("y0-string:{}", hex::encode(y0_bytes));
    // let y1_bytes = [243 203 189 51 238 238 208 177 106 92 9 174 126 219 65 8 25 127 0 66 228 241 244 28 252 165 248 4 63 218 226 161 203 55 182 127 95 228 71 202 31 217 66 238 3 35 127 14] 
    let y1_bytes = [243, 203, 189, 51, 238, 238, 208, 177, 106, 92, 9, 174, 126, 219, 65, 8, 25, 127, 0, 66, 228, 241, 244, 28, 252, 165, 248, 4, 63, 218, 226, 161, 203, 55, 182, 127, 95, 228, 71, 202, 31, 217, 66, 238, 3, 35, 127, 14];
    println!("y1-string:{}", hex::encode(y1_bytes));
    // let z0_bytes = [19 252 77 22 167 224 86 207 170 126 100 101 179 5 123 204 244 241 1 219 167 75 49 47 215 220 138 172 4 140 84 156 139 98 129 126 131 227 83 128 231 209 102 103 142 234 215 9] 
    let z0_bytes = [19, 252, 77, 22, 167, 224, 86, 207, 170, 126, 100, 101, 179, 5, 123, 204, 244, 241, 1, 219, 167, 75, 49, 47, 215, 220, 138, 172, 4, 140, 84, 156, 139, 98, 129, 126, 131, 227, 83, 128, 231, 209, 102, 103, 142, 234, 215, 9];
    println!("z0-string:{}", hex::encode(z0_bytes));
    // let z1_bytes = [52 239 235 194 147 251 219 52 190 151 43 230 243 162 249 150 35 33 35 209 61 156 61 109 217 198 182 43 127 125 25 134 243 14 209 120 248 217 158 177 221 195 12 158 46 213 27 7] 
    let z1_bytes = [52, 239, 235, 194, 147, 251, 219, 52, 190, 151, 43, 230, 243, 162, 249, 150, 35, 33, 35, 209, 61, 156, 61, 109, 217, 198, 182, 43, 127, 125, 25, 134, 243, 14, 209, 120, 248, 217, 158, 177, 221, 195, 12, 158, 46, 213, 27, 7];
    println!("z1-string:{}", hex::encode(z1_bytes));

    for i in 0..48 {
        assignment.x[0][i] = M31::from(x0_bytes[i] as u32);
        assignment.x[1][i] = M31::from(x1_bytes[i] as u32);
        assignment.y[0][i] = M31::from(y0_bytes[i] as u32);
        assignment.y[1][i] = M31::from(y1_bytes[i] as u32);
        assignment.z[0][i] = M31::from(z0_bytes[i] as u32);
        assignment.z[1][i] = M31::from(z1_bytes[i] as u32);
    }
    
    debug_eval(
        &E2AddCircuit::default(),
        &assignment,
        hint_registry,
    );
    // let witness = compile_result
    //     .witness_solver
    //     .solve_witness(&assignment)
    //     .unwrap();
    // let output = compile_result.layered_circuit.run(&witness);
    // assert_eq!(output, vec![true]);
}

