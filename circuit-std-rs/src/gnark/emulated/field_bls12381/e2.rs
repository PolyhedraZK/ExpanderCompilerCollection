use crate::gnark::emparam::FieldParams;
use crate::gnark::element::*;
use crate::gnark::field::Field as GField;
use crate::gnark::emparam::*;
use crate::gnark::hints::{mul_hint, simple_rangecheck_hint};
use std::collections::HashMap;
use std::hint;
use crate::logup::*;
use expander_compiler::frontend::extra::*;
use expander_compiler::{circuit::layered::InputType, frontend::*};
use num_bigint::BigInt;

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
    pub fn sub<'a, C:Config, B:RootAPI<C>>(&mut self, native: &'a mut B, x: &GE2, y: &GE2) -> GE2 {
        let z0 = self.fp.sub(native, x.a0.clone(), y.a0.clone());
        let z1 = self.fp.sub(native,x.a1.clone(), y.a1.clone());
        GE2 {
            a0: z0,
            a1: z1,
        }
    }
    pub fn double<'a, C:Config, B:RootAPI<C>>(&mut self, native: &'a mut B, x: &GE2) -> GE2 {
        let two = BigInt::from(2);
        let z0 = self.fp.mul_const(native, x.a0.clone(), two.clone());
        let z1 = self.fp.mul_const(native, x.a1.clone(), two.clone());
        GE2 {
            a0: z0,
            a1: z1,
        }
    }
    pub fn mul<'a, C:Config, B:RootAPI<C>>(&mut self, native: &'a mut B, x: &GE2, y: &GE2) -> GE2 {

        let v0 = self.fp.mul(native, x.a0.clone(), y.a0.clone());
        let v1 = self.fp.mul(native, x.a1.clone(), y.a1.clone());
        let b0 = self.fp.sub(native, v0.clone(), v1.clone());
        // println!("v0");
        // print_element(native, &v0);
        // println!("v1");
        // print_element(native, &v1);
        // println!("b0");
        // print_element(native, &b0);
        let mut b1 = self.fp.add(native, x.a0.clone(), x.a1.clone());
        let mut tmp = self.fp.add(native, y.a0.clone(), y.a1.clone());
        // println!("b1");
        // print_element(native, &b1);
        // println!("tmp");
        // print_element(native, &tmp);
        b1 = self.fp.mul(native, b1, tmp);
        // println!("b1");
        // print_element(native, &b1);
        tmp = self.fp.add(native, v0.clone(), v1.clone());
        // println!("tmp");
        // print_element(native, &tmp);
        b1 = self.fp.sub(native, b1, tmp);
        // println!("b1");
        // print_element(native, &b1);
        GE2 {
            a0: b0,
            a1: b1,
        }
    }
    pub fn square<'a, C:Config, B:RootAPI<C>>(&mut self, native: &'a mut B, x: &GE2, y: &GE2) -> GE2 {
        let a = self.fp.add(native, x.a0.clone(), x.a1.clone());
        let b = self.fp.sub(native, x.a0.clone(), x.a1.clone());
        let a = self.fp.mul(native, a, b);
        let b = self.fp.mul(native, x.a0.clone(), x.a1.clone());
        let b = self.fp.mul_const(native, b, BigInt::from(2));
        GE2 {
            a0: a,
            a1: b,
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
        let mut z = ext2.add(builder, &x_e2, &y_e2);
        // for i in 0..65536{
        //     z = ext2.add(builder, &z, &y_e2);
        // }
        let z_reduce_a0 = ext2.fp.reduce(builder, z.a0.clone(), false);
        let z_reduce_a1 = ext2.fp.reduce(builder, z.a1.clone(), false);

        // for i in 0..48 {
        //     println!("{}: {:?} {:?}", i, builder.value_of(z_reduce_a0.limbs[i]), builder.value_of(self.z[0][i]));
        //     println!("{}: {:?} {:?}", i, builder.value_of(z_reduce_a1.limbs[i]), builder.value_of(self.z[1][i]));
        //     builder.assert_is_equal(z_reduce_a0.limbs[i], self.z[0][i]);
        //     builder.assert_is_equal(z_reduce_a1.limbs[i], self.z[1][i]);
        // }
        ext2.fp.check_mul(builder);
        ext2.fp.table.final_check(builder);
    }
}

#[test]
fn test_e2_add() {
    // let compile_result = compile(&E2AddCircuit::default()).unwrap();
    let compile_result =
    compile_generic(&E2AddCircuit::default(), CompileOptions::default()).unwrap();
	let mut hint_registry = HintRegistry::<M31>::new();
	hint_registry.register("myhint.mulhint", mul_hint);
	hint_registry.register("myhint.simple_rangecheck_hint", simple_rangecheck_hint);
    hint_registry.register("myhint.querycounthint", query_count_hint);
    let mut assignment = E2AddCircuit::<M31> {
        x: [[M31::from(0); 48], [M31::from(0); 48]],
        y: [[M31::from(0); 48], [M31::from(0); 48]],
        z: [[M31::from(0); 48], [M31::from(0); 48]],
    };

    let x0_bytes = [89,156,69,194,144,213,244,116,63,190,210,105,4,3,175,7,101,54,28,7,18,172,79,84,237,54,73,82,129,140,106,156,148,208,55,92,9,173,33,66,123,235,204,136,44,150,98,10,];
    let x1_bytes = [236,205,45,143,165,12,10,61,83,59,118,233,115,199,99,173,46,152,211,133,250,124,121,183,156,51,67,26,197,238,173,72,255,131,102,60,79,157,114,50,88,209,73,233,20,196,157,18,];
    let y0_bytes = [101,10,8,84,22,11,97,20,107,192,229,172,173,2,120,227,179,177,150,202,54,114,18,66,169,184,198,77,8,75,97,100,206,62,149,101,48,222,77,137,6,205,25,24,76,102,118,25,];
    let y1_bytes = [243,203,189,51,238,238,208,177,106,92,9,174,126,219,65,8,25,127,0,66,228,241,244,28,252,165,248,4,63,218,226,161,203,55,182,127,95,228,71,202,31,217,66,238,3,35,127,14,];
    let z0_bytes = [218,253,64,116,175,52,24,151,151,215,179,170,76,250,69,90,88,37,34,244,208,51,26,6,74,174,1,199,44,146,237,75,240,250,248,226,161,68,67,49,204,164,203,228,12,79,238,5,];
    let z1_bytes = [162,191,112,190,81,47,128,118,149,112,222,152,142,11,49,60,180,34,229,197,248,214,150,237,125,100,177,224,222,18,165,199,250,85,240,222,198,4,78,217,202,6,85,164,7,27,109,21,];
    for i in 0..48 {
        assignment.x[0][i] = M31::from(x0_bytes[i] as u32);
        assignment.x[1][i] = M31::from(x1_bytes[i] as u32);
        assignment.y[0][i] = M31::from(y0_bytes[i] as u32);
        assignment.y[1][i] = M31::from(y1_bytes[i] as u32);
        assignment.z[0][i] = M31::from(z0_bytes[i] as u32);
        assignment.z[1][i] = M31::from(z1_bytes[i] as u32);
    }
    
    // debug_eval(
    //     &E2AddCircuit::default(),
    //     &assignment,
    //     hint_registry,
    // );
}

declare_circuit!(E2SubCircuit {
    x: [[Variable; 48];2],
    y: [[Variable; 48];2],
    z: [[Variable; 48];2],
});

impl GenericDefine<M31Config> for E2SubCircuit<Variable> {
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
        let mut z = ext2.sub(builder, &x_e2, &y_e2);

        for i in 0..32{
            println!("{}", i);
            z = ext2.sub(builder, &z, &y_e2);
        }
        let z_reduce_a0 = ext2.fp.reduce(builder, z.a0.clone(), false);
        let z_reduce_a1 = ext2.fp.reduce(builder, z.a1.clone(), false);

        for i in 0..48 {
            println!("{}: {:?} {:?}", i, builder.value_of(z_reduce_a0.limbs[i]), builder.value_of(self.z[0][i]));
            println!("{}: {:?} {:?}", i, builder.value_of(z_reduce_a1.limbs[i]), builder.value_of(self.z[1][i]));
            builder.assert_is_equal(z_reduce_a0.limbs[i], self.z[0][i]);
            builder.assert_is_equal(z_reduce_a1.limbs[i], self.z[1][i]);
        }
        ext2.fp.check_mul(builder);
        ext2.fp.table.final_check(builder);
    }
}

#[test]
fn test_e2_sub() {
    // let compile_result = compile(&E2SubCircuit::default()).unwrap();
    let compile_result =
        compile_generic(&E2SubCircuit::default(), CompileOptions::default()).unwrap();
    let mut hint_registry = HintRegistry::<M31>::new();
    hint_registry.register("myhint.mulhint", mul_hint);
    hint_registry.register("myhint.simple_rangecheck_hint", simple_rangecheck_hint);
    hint_registry.register("myhint.querycounthint", query_count_hint);
    let mut assignment = E2SubCircuit::<M31> {
        x: [[M31::from(0); 48], [M31::from(0); 48]],
        y: [[M31::from(0); 48], [M31::from(0); 48]],
        z: [[M31::from(0); 48], [M31::from(0); 48]],
    };

    let x0_bytes = [89,156,69,194,144,213,244,116,63,190,210,105,4,3,175,7,101,54,28,7,18,172,79,84,237,54,73,82,129,140,106,156,148,208,55,92,9,173,33,66,123,235,204,136,44,150,98,10,];
    let x1_bytes = [236,205,45,143,165,12,10,61,83,59,118,233,115,199,99,173,46,152,211,133,250,124,121,183,156,51,67,26,197,238,173,72,255,131,102,60,79,157,114,50,88,209,73,233,20,196,157,18,];
    let y0_bytes = [101,10,8,84,22,11,97,20,107,192,229,172,173,2,120,227,179,177,150,202,54,114,18,66,169,184,198,77,8,75,97,100,206,62,149,101,48,222,77,137,6,205,25,24,76,102,118,25,];
    let y1_bytes = [243,203,189,51,238,238,208,177,106,92,9,174,126,219,65,8,25,127,0,66,228,241,244,28,252,165,248,4,63,218,226,161,203,55,182,127,95,228,71,202,31,217,66,238,3,35,127,14,];
    let z0_bytes = [180,154,49,237,175,103,82,20,105,240,180,74,119,170,182,138,184,18,206,191,32,71,9,182,8,193,77,188,13,81,201,58,230,82,112,173,148,255,140,242,236,80,118,157,164,163,65,2,];
    let z1_bytes = [159,131,176,227,240,63,9,101,141,81,41,242,7,124,254,196,126,132,52,92,223,29,85,61,146,31,145,149,254,27,211,122,228,121,59,129,208,247,31,103,24,11,170,61,11,131,77,8,];
    for i in 0..48 {
        assignment.x[0][i] = M31::from(x0_bytes[i] as u32);
        assignment.x[1][i] = M31::from(x1_bytes[i] as u32);
        assignment.y[0][i] = M31::from(y0_bytes[i] as u32);
        assignment.y[1][i] = M31::from(y1_bytes[i] as u32);
        assignment.z[0][i] = M31::from(z0_bytes[i] as u32);
        assignment.z[1][i] = M31::from(z1_bytes[i] as u32);
    }

    debug_eval(
        &E2SubCircuit::default(),
        &assignment,
        hint_registry,
    );
}

declare_circuit!(E2DoubleCircuit {
    x: [[Variable; 48];2],
    z: [[Variable; 48];2],
});

impl GenericDefine<M31Config> for E2DoubleCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let mut ext2 = Ext2::new(builder);
        let x_e2 = GE2 {
            a0: new_internal_element(self.x[0].to_vec(), 0),
            a1: new_internal_element(self.x[1].to_vec(), 0),
        };
        let z = ext2.double(builder, &x_e2);
        let z_reduce_a0 = ext2.fp.reduce(builder, z.a0.clone(), false);
        let z_reduce_a1 = ext2.fp.reduce(builder, z.a1.clone(), false);

        for i in 0..48 {
            println!("{}: {:?} {:?}", i, builder.value_of(z_reduce_a0.limbs[i]), builder.value_of(self.z[0][i]));
            println!("{}: {:?} {:?}", i, builder.value_of(z_reduce_a1.limbs[i]), builder.value_of(self.z[1][i]));
            builder.assert_is_equal(z_reduce_a0.limbs[i], self.z[0][i]);
            builder.assert_is_equal(z_reduce_a1.limbs[i], self.z[1][i]);
        }
        ext2.fp.check_mul(builder);
        ext2.fp.table.final_check(builder);
    }
}

#[test]
fn test_e2_double(){
    // let compile_result = compile(&E2DoubleCircuit::default()).unwrap();
    let mut hint_registry = HintRegistry::<M31>::new();
    hint_registry.register("myhint.mulhint", mul_hint);
    hint_registry.register("myhint.simple_rangecheck_hint", simple_rangecheck_hint);
    hint_registry.register("myhint.querycounthint", query_count_hint);
    let mut assignment = E2DoubleCircuit::<M31> {
        x: [[M31::from(0); 48], [M31::from(0); 48]],
        z: [[M31::from(0); 48], [M31::from(0); 48]],
    };

    let x0_bytes = [15,12,79,128,139,180,205,255,209,222,213,222,254,248,10,230,191,105,202,47,136,213,107,173,156,11,113,96,198,183,126,251,141,187,41,102,110,132,31,81,75,249,2,47,228,206,81,3,];
    let x1_bytes = [240,227,119,201,24,76,33,152,185,85,45,193,110,41,147,127,248,176,165,66,82,161,225,108,180,84,20,69,127,71,121,72,69,230,93,22,77,43,82,119,31,115,198,136,207,8,46,2,];
    let z0_bytes = [30,24,158,0,23,105,155,255,163,189,171,189,253,241,21,204,127,211,148,95,16,171,215,90,57,23,226,192,140,111,253,246,27,119,83,204,220,8,63,162,150,242,5,94,200,157,163,6,];
    let z1_bytes = [224,199,239,146,49,152,66,48,115,171,90,130,221,82,38,255,240,97,75,133,164,66,195,217,104,169,40,138,254,142,242,144,138,204,187,44,154,86,164,238,62,230,140,17,159,17,92,4,];
    for i in 0..48 {
        assignment.x[0][i] = M31::from(x0_bytes[i] as u32);
        assignment.x[1][i] = M31::from(x1_bytes[i] as u32);
        assignment.z[0][i] = M31::from(z0_bytes[i] as u32);
        assignment.z[1][i] = M31::from(z1_bytes[i] as u32);
    }

    debug_eval(
        &E2DoubleCircuit::default(),
        &assignment,
        hint_registry,
    );
}

declare_circuit!(E2MulCircuit {
    x: [[Variable; 48];2],
    y: [[Variable; 48];2],
    z: [[Variable; 48];2],
});

impl GenericDefine<M31Config> for E2MulCircuit<Variable> {
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
        let z = ext2.mul(builder, &x_e2, &y_e2);
        let z_reduce_a0 = ext2.fp.reduce(builder, z.a0.clone(), false);
        let z_reduce_a1 = ext2.fp.reduce(builder, z.a1.clone(), false);

        for i in 0..48 {
            // println!("{}: {:?} {:?}", i, builder.value_of(z_reduce_a0.limbs[i]), builder.value_of(self.z[0][i]));
            // println!("{}: {:?} {:?}", i, builder.value_of(z_reduce_a1.limbs[i]), builder.value_of(self.z[1][i]));
            builder.assert_is_equal(z_reduce_a0.limbs[i], self.z[0][i]);
            builder.assert_is_equal(z_reduce_a1.limbs[i], self.z[1][i]);
        }
        ext2.fp.check_mul(builder);
        ext2.fp.table.final_check(builder);
    }
}

#[test]
fn test_e2_mul(){
    // let compile_result = compile(&E2MulCircuit::default()).unwrap();
    let mut hint_registry = HintRegistry::<M31>::new();
    hint_registry.register("myhint.mulhint", mul_hint);
    hint_registry.register("myhint.simple_rangecheck_hint", simple_rangecheck_hint);
    hint_registry.register("myhint.querycounthint", query_count_hint);
    let mut assignment = E2MulCircuit::<M31> {
        x: [[M31::from(0); 48], [M31::from(0); 48]],
        y: [[M31::from(0); 48], [M31::from(0); 48]],
        z: [[M31::from(0); 48], [M31::from(0); 48]],
    };

    let x0_bytes = [89,156,69,194,144,213,244,116,63,190,210,105,4,3,175,7,101,54,28,7,18,172,79,84,237,54,73,82,129,140,106,156,148,208,55,92,9,173,33,66,123,235,204,136,44,150,98,10,];
    let x1_bytes = [236,205,45,143,165,12,10,61,83,59,118,233,115,199,99,173,46,152,211,133,250,124,121,183,156,51,67,26,197,238,173,72,255,131,102,60,79,157,114,50,88,209,73,233,20,196,157,18,];
    let y0_bytes = [101,10,8,84,22,11,97,20,107,192,229,172,173,2,120,227,179,177,150,202,54,114,18,66,169,184,198,77,8,75,97,100,206,62,149,101,48,222,77,137,6,205,25,24,76,102,118,25,];
    let y1_bytes = [243,203,189,51,238,238,208,177,106,92,9,174,126,219,65,8,25,127,0,66,228,241,244,28,252,165,248,4,63,218,226,161,203,55,182,127,95,228,71,202,31,217,66,238,3,35,127,14,];
    let z0_bytes = [143,141,88,121,8,168,107,196,223,95,145,40,180,240,14,127,2,131,208,179,204,73,135,148,189,111,164,105,224,184,248,44,208,132,0,64,210,236,241,225,171,116,246,214,71,118,162,23,];
    let z1_bytes = [45,113,243,46,31,23,35,212,99,184,76,19,176,150,92,64,237,213,204,21,66,195,173,145,168,82,248,96,149,128,101,6,129,187,168,243,171,181,118,146,105,156,106,82,54,190,245,20,];

    for i in 0..48 {
        assignment.x[0][i] = M31::from(x0_bytes[i] as u32);
        assignment.x[1][i] = M31::from(x1_bytes[i] as u32);
        assignment.y[0][i] = M31::from(y0_bytes[i] as u32);
        assignment.y[1][i] = M31::from(y1_bytes[i] as u32);
        assignment.z[0][i] = M31::from(z0_bytes[i] as u32);
        assignment.z[1][i] = M31::from(z1_bytes[i] as u32);
    }

    debug_eval(
        &E2MulCircuit::default(),
        &assignment,
        hint_registry,
    );
}

declare_circuit!(E2SquareCircuit {
    x: [[Variable; 48];2],
    z: [[Variable; 48];2],
});

impl GenericDefine<M31Config> for E2SquareCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let mut ext2 = Ext2::new(builder);
        let x_e2 = GE2 {
            a0: new_internal_element(self.x[0].to_vec(), 0),
            a1: new_internal_element(self.x[1].to_vec(), 0),
        };
        let z = ext2.square(builder, &x_e2, &x_e2);
        let z_reduce_a0 = ext2.fp.reduce(builder, z.a0.clone(), false);
        let z_reduce_a1 = ext2.fp.reduce(builder, z.a1.clone(), false);

        for i in 0..48 {
            println!("{}: {:?} {:?}", i, builder.value_of(z_reduce_a0.limbs[i]), builder.value_of(self.z[0][i]));
            println!("{}: {:?} {:?}", i, builder.value_of(z_reduce_a1.limbs[i]), builder.value_of(self.z[1][i]));
            builder.assert_is_equal(z_reduce_a0.limbs[i], self.z[0][i]);
            builder.assert_is_equal(z_reduce_a1.limbs[i], self.z[1][i]);
        }
        ext2.fp.check_mul(builder);
        ext2.fp.table.final_check(builder);
    }
}

#[test]
fn test_e2_square(){
    // let compile_result = compile(&E2SquareCircuit::default()).unwrap();
    let mut hint_registry = HintRegistry::<M31>::new();
    hint_registry.register("myhint.mulhint", mul_hint);
    hint_registry.register("myhint.simple_rangecheck_hint", simple_rangecheck_hint);
    hint_registry.register("myhint.querycounthint", query_count_hint);
    let mut assignment = E2SquareCircuit::<M31> {
        x: [[M31::from(0); 48], [M31::from(0); 48]],
        z: [[M31::from(0); 48], [M31::from(0); 48]],
    };

    let x0_bytes = [89,156,69,194,144,213,244,116,63,190,210,105,4,3,175,7,101,54,28,7,18,172,79,84,237,54,73,82,129,140,106,156,148,208,55,92,9,173,33,66,123,235,204,136,44,150,98,10,];
    let x1_bytes = [236,205,45,143,165,12,10,61,83,59,118,233,115,199,99,173,46,152,211,133,250,124,121,183,156,51,67,26,197,238,173,72,255,131,102,60,79,157,114,50,88,209,73,233,20,196,157,18,];
    let z0_bytes = [76,190,203,175,214,65,32,217,101,144,196,235,159,76,190,209,46,223,169,88,25,193,105,217,115,6,68,7,79,4,154,56,167,2,202,34,126,222,83,233,137,224,221,96,140,156,5,18,];
    let z1_bytes = [170,117,86,12,84,70,123,39,30,83,226,114,113,237,118,58,194,47,111,221,135,155,127,91,79,86,4,68,107,170,254,51,102,128,53,134,93,97,103,22,243,175,90,255,163,111,193,25,];
    for i in 0..48 {
        assignment.x[0][i] = M31::from(x0_bytes[i] as u32);
        assignment.x[1][i] = M31::from(x1_bytes[i] as u32);
        assignment.z[0][i] = M31::from(z0_bytes[i] as u32);
        assignment.z[1][i] = M31::from(z1_bytes[i] as u32);
    }

    debug_eval(
        &E2SquareCircuit::default(),
        &assignment,
        hint_registry,
    );
}




pub fn print_e2<'a, C:Config, B:RootAPI<C>>(native: &'a mut B, v: &GE2)  {
    for i in 0..48 {
        println!("{}: {:?} {:?}", i, native.value_of(v.a0.limbs[i]), native.value_of(v.a1.limbs[i]));
    }
}
pub fn print_element<'a, C:Config, B:RootAPI<C>, T: FieldParams>(native: &'a mut B, v: &Element<T>)  {
    for i in 0..48 {
        println!("{}: {:?}", i, native.value_of(v.limbs[i]));
    }
}