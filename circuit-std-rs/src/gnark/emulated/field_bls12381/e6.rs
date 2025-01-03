use crate::gnark::emparam::FieldParams;
use crate::gnark::element::*;
use crate::gnark::field::Field as GField;
use crate::gnark::emparam::*;
use crate::gnark::hints::{div_e2_hint, inverse_e2_hint, mul_hint, simple_rangecheck_hint};
use std::collections::HashMap;
use std::hint;
use crate::logup::*;
use expander_compiler::frontend::extra::*;
use expander_compiler::{circuit::layered::InputType, frontend::*};
use num_bigint::BigInt;

use super::e2::*;
#[derive(Default, Clone)]
pub struct GE6 {
    pub b0: GE2,
    pub b1: GE2,
    pub b2: GE2,
}
impl GE6 {
    pub fn clone(&self) -> Self {
        GE6 {
            b0: self.b0.clone(),
            b1: self.b1.clone(),
            b2: self.b2.clone(),
        }
    }
}
pub struct Ext6 {
    pub ext2: Ext2,
}

impl Ext6{
    pub fn new<'a, C:Config, B:RootAPI<C>>(api: &'a mut B) -> Self {
        Self {
            ext2: Ext2::new(api),
        }
    }
    pub fn one(&mut self) -> GE6 {
        let b0 = self.ext2.one();
        let b1 = self.ext2.zero();
        let b2 = self.ext2.zero();
        GE6 {
            b0,
            b1,
            b2
        }
    }
    pub fn zero<'a, C:Config, B:RootAPI<C>>(&mut self) -> GE6 {
        let b0 = self.ext2.zero();
        let b1 = self.ext2.zero();
        let b2 = self.ext2.zero();
        GE6 {
            b0,
            b1,
            b2
        }
    }
    pub fn is_zero<'a, C:Config, B:RootAPI<C>>(&mut self, native: &'a mut B, z: &GE6) -> Variable {
        let b0 = self.ext2.is_zero(native, &z.b0.clone());
        let b1 = self.ext2.is_zero(native, &z.b1.clone());
        let b2 = self.ext2.is_zero(native, &z.b2.clone());
        let tmp = native.and(b0, b1);
        native.and(tmp, b2)
    }
    pub fn add<'a, C:Config, B:RootAPI<C>>(&mut self, native: &'a mut B, x: &GE6, y: &GE6) -> GE6 {
        let z0 = self.ext2.add(native, &x.b0.clone(), &y.b0.clone());
        let z1 = self.ext2.add(native, &x.b1.clone(), &y.b1.clone());
        let z2 = self.ext2.add(native, &x.b2.clone(), &y.b2.clone());
        GE6 {
            b0: z0,
            b1: z1,
            b2: z2,
        }
    }
    pub fn neg<'a, C:Config, B:RootAPI<C>>(&mut self, native: &'a mut B, x: &GE6) -> GE6 {
        let z0 = self.ext2.neg(native, &x.b0.clone());
        let z1 = self.ext2.neg(native, &x.b1.clone());
        let z2 = self.ext2.neg(native, &x.b2.clone());
        GE6 {
            b0: z0,
            b1: z1,
            b2: z2,
        }
    }
    pub fn sub<'a, C:Config, B:RootAPI<C>>(&mut self, native: &'a mut B, x: &GE6, y: &GE6) -> GE6 {
        let z0 = self.ext2.sub(native, &x.b0.clone(), &y.b0.clone());
        let z1 = self.ext2.sub(native, &x.b1.clone(), &y.b1.clone());
        let z2 = self.ext2.sub(native, &x.b2.clone(), &y.b2.clone());
        GE6 {
            b0: z0,
            b1: z1,
            b2: z2,
        }
    }
    pub fn double<'a, C:Config, B:RootAPI<C>>(&mut self, native: &'a mut B, x: &GE6) -> GE6 {
        let z0 = self.ext2.double(native, &x.b0.clone());
        let z1 = self.ext2.double(native, &x.b1.clone());
        let z2 = self.ext2.double(native, &x.b2.clone());
        GE6 {
            b0: z0,
            b1: z1,
            b2: z2,
        }
    }
    pub fn square<'a, C:Config, B:RootAPI<C>>(&mut self, native: &'a mut B, x: &GE6) -> GE6 {
        let c4 = self.ext2.mul(native, &x.b0.clone(), &x.b1.clone());
        let c4 = self.ext2.double(native, &c4);
        let c5 = self.ext2.square(native, &x.b2.clone());
        let c1 = self.ext2.mul_by_non_residue(native, &c5);
        let c1 = self.ext2.add(native, &c1, &c4);
        let c2 = self.ext2.sub(native, &c4, &c5);
        let c3 = self.ext2.square(native, &x.b0.clone());
        let c4 = self.ext2.sub(native, &x.b0.clone(), &x.b1.clone());
        let c4 = self.ext2.add(native, &c4, &x.b2.clone());
        let c5 = self.ext2.mul(native, &x.b1.clone(), &x.b2.clone());
        let c5 = self.ext2.double(native, &c5);
        let c4 = self.ext2.square(native, &c4);
        let c0 = self.ext2.mul_by_non_residue(native, &c5);
        let c0 = self.ext2.add(native, &c0, &c3);
        let z2 = self.ext2.add(native, &c2, &c4);
        let z2 = self.ext2.add(native, &z2, &c5);
        let z2 = self.ext2.sub(native, &z2, &c3);
        let z0 = c0;
        let z1 = c1;
        GE6 {
            b0: z0,
            b1: z1,
            b2: z2,
        }
    }
    pub fn mul_by_e2<'a, C:Config, B:RootAPI<C>>(&mut self, native: &'a mut B, x: &GE6, y: &GE2) -> GE6 {
        let z0 = self.ext2.mul(native, &x.b0.clone(), y);
        let z1 = self.ext2.mul(native, &x.b1.clone(), y);
        let z2 = self.ext2.mul(native, &x.b2.clone(), y);
        GE6 {
            b0: z0,
            b1: z1,
            b2: z2,
        }
    }
    pub fn mul_by_12<'a, C:Config, B:RootAPI<C>>(&mut self, native: &'a mut B, x: &GE6, b1: &GE2, b2: &GE2) -> GE6 {
        let t1 = self.ext2.mul(native, &x.b1.clone(), b1);
        let t2 = self.ext2.mul(native, &x.b2.clone(), b2);
        let mut c0 = self.ext2.add(native, &x.b1.clone(), &x.b2.clone());
        let mut tmp = self.ext2.add(native, b1, b2);
        c0 = self.ext2.mul(native, &c0, &tmp);
        tmp = self.ext2.add(native, &t1, &t2);
        c0 = self.ext2.sub(native, &c0, &tmp);
        c0 = self.ext2.mul_by_non_residue(native, &c0);
        let mut c1 = self.ext2.add(native, &x.b0.clone(), &x.b1.clone());
        c1 = self.ext2.mul(native, &c1, b1);
        c1 = self.ext2.sub(native, &c1, &t1);
        tmp = self.ext2.mul_by_non_residue(native, &t2);
        c1 = self.ext2.add(native, &c1, &tmp);
        tmp = self.ext2.add(native, &x.b0.clone(), &x.b2.clone());
        let mut c2 = self.ext2.mul(native, b2, &tmp);
        c2 = self.ext2.sub(native, &c2, &t2);
        c2 = self.ext2.add(native, &c2, &t1);
        GE6 {
            b0: c0,
            b1: c1,
            b2: c2,
        }
    }
    pub fn mul_by_0<'a, C:Config, B:RootAPI<C>>(&mut self, native: &'a mut B, z: &GE6, c0: &GE2) -> GE6 {
        let a = self.ext2.mul(native, &z.b0.clone(), c0);
        let tmp = self.ext2.add(native, &z.b0.clone(), &z.b2.clone());
        let mut t2 = self.ext2.mul(native, c0, &tmp);
        t2 = self.ext2.sub(native, &t2, &a);
        let tmp = self.ext2.add(native, &z.b0.clone(), &z.b1.clone());
        let mut t1 = self.ext2.mul(native, c0, &tmp);
        t1 = self.ext2.sub(native, &t1, &a);
        GE6 {
            b0: a,
            b1: t1,
            b2: t2,
        }
    }
    pub fn mul_by_01<'a, C:Config, B:RootAPI<C>>(&mut self, native: &'a mut B, z: &GE6, c0: &GE2, c1: &GE2) -> GE6 {
        let a = self.ext2.mul(native, &z.b0.clone(), c0);
        let b = self.ext2.mul(native, &z.b1.clone(), c1);
        let tmp = self.ext2.add(native, &z.b1.clone(), &z.b2.clone());
        let mut t0 = self.ext2.mul(native, c1, &tmp);
        t0 = self.ext2.sub(native, &t0, &b);
        t0 = self.ext2.mul_by_non_residue(native, &t0);
        t0 = self.ext2.add(native, &t0, &a);
        let t2 = self.ext2.mul(native, &z.b2.clone(), c0);
        t0 = self.ext2.add(native, &t2, &b);
        let mut t1 = self.ext2.add(native, c0, c1);
        let tmp = self.ext2.add(native, &z.b0.clone(), &z.b1.clone());
        t1 = self.ext2.mul(native, &t1, &tmp);
        let tmp = self.ext2.add(native, &a, &b);
        t1 = self.ext2.sub(native, &t1, &tmp);
        GE6 {
            b0: t0,
            b1: t1,
            b2: t2,
        }
    }
    pub fn mul_by_non_residue<'a, C:Config, B:RootAPI<C>>(&mut self, native: &'a mut B, x: &GE6) -> GE6 {
        let z0 = self.ext2.mul_by_non_residue(native, &x.b2.clone());
        GE6 {
            b0: z0,
            b1: x.b0.clone(),
            b2: x.b1.clone(),
        }
    }
    pub fn assert_isequal<'a, C:Config, B:RootAPI<C>>(&mut self, native: &'a mut B, x: &GE6, y: &GE6) {
        self.ext2.assert_isequal(native, &x.b0, &y.b0);
        self.ext2.assert_isequal(native, &x.b1, &y.b1);
        self.ext2.assert_isequal(native, &x.b2, &y.b2);
    }
    pub fn select<'a, C:Config, B:RootAPI<C>>(&mut self, native: &'a mut B, selector: Variable, z1: &GE6, z0: &GE6) -> GE6 {
        let b0 = self.ext2.select(native, selector, &z1.b0.clone(), &z0.b0.clone());
        let b1 = self.ext2.select(native, selector, &z1.b1.clone(), &z0.b1.clone());
        let b2 = self.ext2.select(native, selector, &z1.b2.clone(), &z0.b2.clone());
        GE6 {
            b0: b0,
            b1: b1,
            b2: b2,
        }
    }
    pub fn mul_karatsuba_over_karatsuba<'a, C:Config, B:RootAPI<C>>(&mut self, native: &'a mut B, x: &GE6, y: &GE6) -> GE6 {
        let t0 = self.ext2.mul(native, &x.b0.clone(), &y.b0.clone());
        let t1 = self.ext2.mul(native, &x.b1.clone(), &y.b1.clone());
        let t2 = self.ext2.mul(native, &x.b2.clone(), &y.b2.clone());
        let mut c0 = self.ext2.add(native, &x.b1.clone(), &x.b2.clone());
        let mut tmp = self.ext2.add(native, &y.b1.clone(), &y.b2.clone());
        c0 = self.ext2.mul(native, &c0, &tmp);
        tmp = self.ext2.add(native, &t2, &t1);
        c0 = self.ext2.sub(native, &c0, &tmp);
        c0 = self.ext2.mul_by_non_residue(native, &c0);
        c0 = self.ext2.add(native, &c0, &t0);
        let mut c1 = self.ext2.add(native, &x.b0.clone(), &x.b1.clone());
        tmp = self.ext2.add(native, &y.b0.clone(), &y.b1.clone());
        c1 = self.ext2.mul(native, &c1, &tmp);
        tmp = self.ext2.add(native, &t0, &t1);
        c1 = self.ext2.sub(native, &c1, &tmp);
        tmp = self.ext2.mul_by_non_residue(native, &t2);
        c1 = self.ext2.add(native, &c1, &tmp);
        let mut c2 = self.ext2.add(native, &x.b0.clone(), &x.b2.clone());
        let mut tmp = self.ext2.add(native, &y.b0.clone(), &y.b2.clone());
        c2 = self.ext2.mul(native, &c2, &tmp);
        tmp = self.ext2.add(native, &t0, &t2);
        c2 = self.ext2.sub(native, &c2, &tmp);
        c2 = self.ext2.add(native, &c2, &t1);
        GE6 {
            b0: c0,
            b1: c1,
            b2: c2,
        }
    }
    pub fn mul<'a, C:Config, B:RootAPI<C>>(&mut self, native: &'a mut B, x: &GE6, y: &GE6) -> GE6 {
        self.mul_karatsuba_over_karatsuba(native, x, y)
    }
    pub fn div<'a, C:Config, B:RootAPI<C>>(&mut self, native: &'a mut B, x: &GE6, y: &GE6) -> GE6 {
        let inputs = vec![x.b0.a0.clone(), x.b0.a1.clone(), x.b1.a0.clone(), x.b1.a1.clone(), x.b2.a0.clone(), x.b2.a1.clone(), y.b0.a0.clone(), y.b0.a1.clone(), y.b1.a0.clone(), y.b1.a1.clone(), y.b2.a0.clone(), y.b2.a1.clone()];
        let output = self.ext2.fp.new_hint(native, "dive6hint", 6, inputs);
        let div = GE6 {
            b0: GE2 {
                a0: output[0].clone(),
                a1: output[1].clone(),
            },
            b1: GE2 {
                a0: output[2].clone(),
                a1: output[3].clone(),
            },
            b2: GE2 {
                a0: output[4].clone(),
                a1: output[5].clone(),
            },
        };
        let _x = self.mul(native, &div, y);
        self.assert_isequal(native, &x.clone(), &_x);
        div
    }
    pub fn inverse_div<'a, C:Config, B:RootAPI<C>>(&mut self, native: &'a mut B, x: &GE6) -> GE6 {
        let one = self.one();
        self.div(native, &one, x)
    }
    pub fn inverse<'a, C:Config, B:RootAPI<C>>(&mut self, native: &'a mut B, x: &GE6) -> GE6 {
        let inputs = vec![x.b0.a0.clone(), x.b0.a1.clone(), x.b1.a0.clone(), x.b1.a1.clone(), x.b2.a0.clone(), x.b2.a1.clone()];
        let output = self.ext2.fp.new_hint(native, "inversee6hint", 6, inputs);
        let inv = GE6 {
            b0: GE2 {
                a0: output[0].clone(),
                a1: output[1].clone(),
            },
            b1: GE2 {
                a0: output[2].clone(),
                a1: output[3].clone(),
            },
            b2: GE2 {
                a0: output[4].clone(),
                a1: output[5].clone(),
            },
        };
        let one = self.one();
        let _one = self.mul(native, &inv, x);
        self.assert_isequal(native, &one, &_one);
        inv
    }
    /*
    func (e Ext6) divE6By6(x [6]*baseEl) [6]*baseEl {
	res, err := e.fp.NewHint(divE6By6Hint, 6, x[0], x[1], x[2], x[3], x[4], x[5])
	if err != nil {
		// err is non-nil only for invalid number of inputs
		panic(err)
	}

	y0 := *res[0]
	y1 := *res[1]
	y2 := *res[2]
	y3 := *res[3]
	y4 := *res[4]
	y5 := *res[5]

	// xi == 6 * yi
	x0 := e.fp.MulConst(&y0, big.NewInt(6))
	x1 := e.fp.MulConst(&y1, big.NewInt(6))
	x2 := e.fp.MulConst(&y2, big.NewInt(6))
	x3 := e.fp.MulConst(&y3, big.NewInt(6))
	x4 := e.fp.MulConst(&y4, big.NewInt(6))
	x5 := e.fp.MulConst(&y5, big.NewInt(6))
	e.fp.AssertIsEqual(x[0], x0)
	e.fp.AssertIsEqual(x[1], x1)
	e.fp.AssertIsEqual(x[2], x2)
	e.fp.AssertIsEqual(x[3], x3)
	e.fp.AssertIsEqual(x[4], x4)
	e.fp.AssertIsEqual(x[5], x5)

	return [6]*baseEl{&y0, &y1, &y2, &y3, &y4, &y5}
}
     */
    pub fn div_e6_by_6<'a, C:Config, B:RootAPI<C>>(&mut self, native: &'a mut B, x: &[Element<bls12381_fp>; 6]) -> [Element<bls12381_fp>; 6] {
        let inputs = vec![x[0].clone(), x[1].clone(), x[2].clone(), x[3].clone(), x[4].clone(), x[5].clone()];
        let output = self.ext2.fp.new_hint(native, "dive6by6hint", 6, inputs);
        let y0 = output[0].clone();
        let y1 = output[1].clone();
        let y2 = output[2].clone();
        let y3 = output[3].clone();
        let y4 = output[4].clone();
        let y5 = output[5].clone();
        let x0 = self.ext2.fp.mul_const(native, &y0, BigInt::from(6));
        let x1 = self.ext2.fp.mul_const(native, &y1, BigInt::from(6));
        let x2 = self.ext2.fp.mul_const(native, &y2, BigInt::from(6));
        let x3 = self.ext2.fp.mul_const(native, &y3, BigInt::from(6));
        let x4 = self.ext2.fp.mul_const(native, &y4, BigInt::from(6));
        let x5 = self.ext2.fp.mul_const(native, &y5, BigInt::from(6));
        self.ext2.fp.assert_isequal(native, &x[0], &x0);
        self.ext2.fp.assert_isequal(native, &x[1], &x1);
        self.ext2.fp.assert_isequal(native, &x[2], &x2);
        self.ext2.fp.assert_isequal(native, &x[3], &x3);
        self.ext2.fp.assert_isequal(native, &x[4], &x4);
        self.ext2.fp.assert_isequal(native, &x[5], &x5);
        [y0, y1, y2, y3, y4, y5]
    }
    // pub fn conjugate<'a, C:Config, B:RootAPI<C>>(&mut self, native: &'a mut B, x: &GE2) -> GE2 {
    //     let z0 = x.a0.clone();
    //     let z1 = self.fp.neg(native, x.a1.clone());
    //     GE2 {
    //         a0: z0,
    //         a1: z1,
    //     }
    // }
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
    let compile_result =
    compile_generic(&E2DoubleCircuit::default(), CompileOptions::default()).unwrap();
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
    let compile_result =
    compile_generic(&E2MulCircuit::default(), CompileOptions::default()).unwrap();
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
        let z = ext2.square(builder, &x_e2);
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
    let compile_result =
    compile_generic(&E2SquareCircuit::default(), CompileOptions::default()).unwrap();
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

declare_circuit!(E2DivCircuit {
    x: [[Variable; 48];2],
    y: [[Variable; 48];2],
    z: [[Variable; 48];2],
});

impl GenericDefine<M31Config> for E2DivCircuit<Variable> {
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
        let z = ext2.div(builder, &x_e2, &y_e2);
        // let z_reduce_a0 = ext2.fp.reduce(builder, z.a0.clone(), false);
        // let z_reduce_a1 = ext2.fp.reduce(builder, z.a1.clone(), false);

        for i in 0..48 {
            println!("{}: {:?} {:?}", i, builder.value_of(z.a0.limbs[i]), builder.value_of(self.z[0][i]));
            println!("{}: {:?} {:?}", i, builder.value_of(z.a1.limbs[i]), builder.value_of(self.z[1][i]));
            builder.assert_is_equal(z.a0.limbs[i], self.z[0][i]);
            builder.assert_is_equal(z.a1.limbs[i], self.z[1][i]);
        }
        ext2.fp.check_mul(builder);
        ext2.fp.table.final_check(builder);
    }
}

#[test]
fn test_e2_div(){
    // let compile_result = compile(&E2DivCircuit::default()).unwrap();
    let compile_result =
    compile_generic(&E2DivCircuit::default(), CompileOptions::default()).unwrap();
    let mut hint_registry = HintRegistry::<M31>::new();
    hint_registry.register("myhint.mulhint", mul_hint);
    hint_registry.register("myhint.simple_rangecheck_hint", simple_rangecheck_hint);
    hint_registry.register("myhint.querycounthint", query_count_hint);
    hint_registry.register("myhint.dive2hint", div_e2_hint);
    let mut assignment = E2DivCircuit::<M31> {
        x: [[M31::from(0); 48], [M31::from(0); 48]],
        y: [[M31::from(0); 48], [M31::from(0); 48]],
        z: [[M31::from(0); 48], [M31::from(0); 48]],
    };

    let x0_bytes = [89,156,69,194,144,213,244,116,63,190,210,105,4,3,175,7,101,54,28,7,18,172,79,84,237,54,73,82,129,140,106,156,148,208,55,92,9,173,33,66,123,235,204,136,44,150,98,10,];
    let x1_bytes = [236,205,45,143,165,12,10,61,83,59,118,233,115,199,99,173,46,152,211,133,250,124,121,183,156,51,67,26,197,238,173,72,255,131,102,60,79,157,114,50,88,209,73,233,20,196,157,18,];
    let y0_bytes = [101,10,8,84,22,11,97,20,107,192,229,172,173,2,120,227,179,177,150,202,54,114,18,66,169,184,198,77,8,75,97,100,206,62,149,101,48,222,77,137,6,205,25,24,76,102,118,25,];
    let y1_bytes = [243,203,189,51,238,238,208,177,106,92,9,174,126,219,65,8,25,127,0,66,228,241,244,28,252,165,248,4,63,218,226,161,203,55,182,127,95,228,71,202,31,217,66,238,3,35,127,14,];
    let z0_bytes = [153,184,22,74,13,182,120,88,173,188,79,252,223,69,219,113,24,134,224,254,32,98,137,82,111,109,147,178,206,57,2,59,140,168,221,75,120,184,199,120,106,250,243,94,234,159,235,8,];
    let z1_bytes = [177,188,16,148,100,119,79,251,253,76,250,108,166,218,213,148,139,44,125,158,121,112,238,245,236,191,74,85,188,152,34,142,65,72,66,245,76,125,71,123,203,25,122,132,192,59,181,2,];
   for i in 0..48 {
        assignment.x[0][i] = M31::from(x0_bytes[i] as u32);
        assignment.x[1][i] = M31::from(x1_bytes[i] as u32);
        assignment.y[0][i] = M31::from(y0_bytes[i] as u32);
        assignment.y[1][i] = M31::from(y1_bytes[i] as u32);
        assignment.z[0][i] = M31::from(z0_bytes[i] as u32);
        assignment.z[1][i] = M31::from(z1_bytes[i] as u32);
    }

    debug_eval(
        &E2DivCircuit::default(),
        &assignment,
        hint_registry,
    );
}

declare_circuit!(E2MulByElementCircuit {
    a: [[Variable; 48];2],
    b: [Variable; 48],
    c: [[Variable; 48];2],
});

impl GenericDefine<M31Config> for E2MulByElementCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let mut ext2 = Ext2::new(builder);
        let a_e2 = GE2 {
            a0: new_internal_element(self.a[0].to_vec(), 0),
            a1: new_internal_element(self.a[1].to_vec(), 0),
        };
        let b = new_internal_element(self.b.to_vec(), 0);
        let c = ext2.mul_by_element(builder, &a_e2, &b);
        let c_reduce_a0 = ext2.fp.reduce(builder, c.a0.clone(), false);
        let c_reduce_a1 = ext2.fp.reduce(builder, c.a1.clone(), false);

        for i in 0..48 {
            println!("{}: {:?} {:?}", i, builder.value_of(c_reduce_a0.limbs[i]), builder.value_of(self.c[0][i]));
            println!("{}: {:?} {:?}", i, builder.value_of(c_reduce_a1.limbs[i]), builder.value_of(self.c[1][i]));
            builder.assert_is_equal(c_reduce_a0.limbs[i], self.c[0][i]);
            builder.assert_is_equal(c_reduce_a1.limbs[i], self.c[1][i]);
        }
        ext2.fp.check_mul(builder);
        ext2.fp.table.final_check(builder);
    }
}

#[test]
fn test_e2_mul_by_element(){
    // let compile_result = compile(&E2MulByElementCircuit::default()).unwrap();
    let compile_result =
    compile_generic(&E2MulByElementCircuit::default(), CompileOptions::default()).unwrap();
    let mut hint_registry = HintRegistry::<M31>::new();
    hint_registry.register("myhint.mulhint", mul_hint);
    hint_registry.register("myhint.simple_rangecheck_hint", simple_rangecheck_hint);
    hint_registry.register("myhint.querycounthint", query_count_hint);
    hint_registry.register("myhint.dive2hint", div_e2_hint);
    let mut assignment = E2MulByElementCircuit::<M31> {
        a: [[M31::from(0); 48], [M31::from(0); 48]],
        b: [M31::from(0); 48],
        c: [[M31::from(0); 48], [M31::from(0); 48]],
    };

    let x0_bytes = [89,156,69,194,144,213,244,116,63,190,210,105,4,3,175,7,101,54,28,7,18,172,79,84,237,54,73,82,129,140,106,156,148,208,55,92,9,173,33,66,123,235,204,136,44,150,98,10,];
    let x1_bytes = [236,205,45,143,165,12,10,61,83,59,118,233,115,199,99,173,46,152,211,133,250,124,121,183,156,51,67,26,197,238,173,72,255,131,102,60,79,157,114,50,88,209,73,233,20,196,157,18,];
    let y0_bytes = [101,10,8,84,22,11,97,20,107,192,229,172,173,2,120,227,179,177,150,202,54,114,18,66,169,184,198,77,8,75,97,100,206,62,149,101,48,222,77,137,6,205,25,24,76,102,118,25,];
    let z0_bytes = [182,22,7,253,0,12,198,225,34,100,90,32,63,141,75,146,131,75,234,238,183,203,163,40,205,44,246,38,124,126,21,66,113,12,134,89,79,157,177,199,10,108,231,138,198,51,108,16,];
    let z1_bytes = [99,158,220,37,153,125,46,222,184,169,143,169,208,242,197,124,114,180,20,50,232,149,134,129,164,99,50,252,99,116,250,173,155,113,102,35,155,201,251,48,142,96,192,33,247,46,83,10,];

    for i in 0..48 {
        assignment.a[0][i] = M31::from(x0_bytes[i] as u32);
        assignment.a[1][i] = M31::from(x1_bytes[i] as u32);
        assignment.b[i] = M31::from(y0_bytes[i] as u32);
        assignment.c[0][i] = M31::from(z0_bytes[i] as u32);
        assignment.c[1][i] = M31::from(z1_bytes[i] as u32);
    }

    debug_eval(
        &E2MulByElementCircuit::default(),
        &assignment,
        hint_registry,
    );
}


declare_circuit!(E2MulByNonResidueCircuit {
    a: [[Variable; 48];2],
    c: [[Variable; 48];2],
});

impl GenericDefine<M31Config> for E2MulByNonResidueCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let mut ext2 = Ext2::new(builder);
        let a_e2 = GE2 {
            a0: new_internal_element(self.a[0].to_vec(), 0),
            a1: new_internal_element(self.a[1].to_vec(), 0),
        };
        let c = ext2.mul_by_non_residue(builder, &a_e2);
        let c_reduce_a0 = ext2.fp.reduce(builder, c.a0.clone(), false);
        let c_reduce_a1 = ext2.fp.reduce(builder, c.a1.clone(), false);

        for i in 0..48 {
            println!("{}: {:?} {:?}", i, builder.value_of(c_reduce_a0.limbs[i]), builder.value_of(self.c[0][i]));
            println!("{}: {:?} {:?}", i, builder.value_of(c_reduce_a1.limbs[i]), builder.value_of(self.c[1][i]));
            builder.assert_is_equal(c_reduce_a0.limbs[i], self.c[0][i]);
            builder.assert_is_equal(c_reduce_a1.limbs[i], self.c[1][i]);
        }
        ext2.fp.check_mul(builder);
        ext2.fp.table.final_check(builder);
    }
}

#[test]
fn test_e2_mul_by_non_residue(){
    // let compile_result = compile(&E2MulByNonResidueCircuit::default()).unwrap();
    let compile_result =
    compile_generic(&E2MulByNonResidueCircuit::default(), CompileOptions::default()).unwrap();
    let mut hint_registry = HintRegistry::<M31>::new();
    hint_registry.register("myhint.mulhint", mul_hint);
    hint_registry.register("myhint.simple_rangecheck_hint", simple_rangecheck_hint);
    hint_registry.register("myhint.querycounthint", query_count_hint);
    hint_registry.register("myhint.dive2hint", div_e2_hint);
    let mut assignment = E2MulByNonResidueCircuit::<M31> {
        a: [[M31::from(0); 48], [M31::from(0); 48]],
        c: [[M31::from(0); 48], [M31::from(0); 48]],
    };

    let x0_bytes = [89,156,69,194,144,213,244,116,63,190,210,105,4,3,175,7,101,54,28,7,18,172,79,84,237,54,73,82,129,140,106,156,148,208,55,92,9,173,33,66,123,235,204,136,44,150,98,10,];
    let x1_bytes = [236,205,45,143,165,12,10,61,83,59,118,233,115,199,99,173,46,152,211,133,250,124,121,183,156,51,67,26,197,238,173,72,255,131,102,60,79,157,114,50,88,209,73,233,20,196,157,18,];
    let z0_bytes = [24,121,23,51,235,200,233,241,235,130,176,49,143,59,247,120,90,148,249,119,184,1,7,4,16,22,139,43,65,233,51,184,108,249,28,99,112,183,202,90,189,0,3,217,1,228,197,17,];
    let z1_bytes = [154,191,115,81,54,226,255,247,146,249,244,161,121,202,102,150,111,216,62,150,107,86,152,164,202,87,7,121,193,47,161,128,188,167,82,85,162,162,120,41,57,214,150,56,87,72,255,2,];

    for i in 0..48 {
        assignment.a[0][i] = M31::from(x0_bytes[i] as u32);
        assignment.a[1][i] = M31::from(x1_bytes[i] as u32);
        assignment.c[0][i] = M31::from(z0_bytes[i] as u32);
        assignment.c[1][i] = M31::from(z1_bytes[i] as u32);
    }

    debug_eval(
        &E2MulByNonResidueCircuit::default(),
        &assignment,
        hint_registry,
    );
}


declare_circuit!(E2NegCircuit {
    a: [[Variable; 48];2],
    c: [[Variable; 48];2],
});

impl GenericDefine<M31Config> for E2NegCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let mut ext2 = Ext2::new(builder);
        let a_e2 = GE2 {
            a0: new_internal_element(self.a[0].to_vec(), 0),
            a1: new_internal_element(self.a[1].to_vec(), 0),
        };
        let c = ext2.neg(builder, &a_e2);
        let c_reduce_a0 = ext2.fp.reduce(builder, c.a0.clone(), false);
        let c_reduce_a1 = ext2.fp.reduce(builder, c.a1.clone(), false);

        for i in 0..48 {
            println!("{}: {:?} {:?}", i, builder.value_of(c_reduce_a0.limbs[i]), builder.value_of(self.c[0][i]));
            println!("{}: {:?} {:?}", i, builder.value_of(c_reduce_a1.limbs[i]), builder.value_of(self.c[1][i]));
            builder.assert_is_equal(c_reduce_a0.limbs[i], self.c[0][i]);
            builder.assert_is_equal(c_reduce_a1.limbs[i], self.c[1][i]);
        }
        ext2.fp.check_mul(builder);
        ext2.fp.table.final_check(builder);
    }
}

#[test]
fn test_e2_neg(){
    // let compile_result = compile(&E2NegCircuit::default()).unwrap();
    let compile_result =
    compile_generic(&E2NegCircuit::default(), CompileOptions::default()).unwrap();
    let mut hint_registry = HintRegistry::<M31>::new();
    hint_registry.register("myhint.mulhint", mul_hint);
    hint_registry.register("myhint.simple_rangecheck_hint", simple_rangecheck_hint);
    hint_registry.register("myhint.querycounthint", query_count_hint);
    hint_registry.register("myhint.dive2hint", div_e2_hint);
    let mut assignment = E2NegCircuit::<M31> {
        a: [[M31::from(0); 48], [M31::from(0); 48]],
        c: [[M31::from(0); 48], [M31::from(0); 48]],
    };

    let x0_bytes = [89,156,69,194,144,213,244,116,63,190,210,105,4,3,175,7,101,54,28,7,18,172,79,84,237,54,73,82,129,140,106,156,148,208,55,92,9,173,33,66,123,235,204,136,44,150,98,10,];
    let x1_bytes = [236,205,45,143,165,12,10,61,83,59,118,233,115,199,99,173,46,152,211,133,250,124,121,183,156,51,67,26,197,238,173,72,255,131,102,60,79,157,114,50,88,209,73,233,20,196,157,18,];
    let z0_bytes = [82,14,186,61,111,42,10,69,192,65,129,71,250,252,252,22,191,191,148,239,142,38,225,18,210,219,59,161,3,191,12,200,66,220,19,231,172,250,249,8,31,251,178,176,189,123,158,15,];
    let z1_bytes = [191,220,209,112,90,243,244,124,172,196,221,199,138,56,72,113,245,93,221,112,166,85,183,175,34,223,65,217,191,92,201,27,216,40,229,6,103,10,169,24,66,21,54,80,213,77,99,7,];

    for i in 0..48 {
        assignment.a[0][i] = M31::from(x0_bytes[i] as u32);
        assignment.a[1][i] = M31::from(x1_bytes[i] as u32);
        assignment.c[0][i] = M31::from(z0_bytes[i] as u32);
        assignment.c[1][i] = M31::from(z1_bytes[i] as u32);
    }

    debug_eval(
        &E2NegCircuit::default(),
        &assignment,
        hint_registry,
    );
}

declare_circuit!(E2ConjugateCircuit {
    a: [[Variable; 48];2],
    c: [[Variable; 48];2],
});

impl GenericDefine<M31Config> for E2ConjugateCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let mut ext2 = Ext2::new(builder);
        let a_e2 = GE2 {
            a0: new_internal_element(self.a[0].to_vec(), 0),
            a1: new_internal_element(self.a[1].to_vec(), 0),
        };
        let c = ext2.conjugate(builder, &a_e2);
        let c_reduce_a0 = ext2.fp.reduce(builder, c.a0.clone(), false);
        let c_reduce_a1 = ext2.fp.reduce(builder, c.a1.clone(), false);

        for i in 0..48 {
            println!("{}: {:?} {:?}", i, builder.value_of(c_reduce_a0.limbs[i]), builder.value_of(self.c[0][i]));
            println!("{}: {:?} {:?}", i, builder.value_of(c_reduce_a1.limbs[i]), builder.value_of(self.c[1][i]));
            builder.assert_is_equal(c_reduce_a0.limbs[i], self.c[0][i]);
            builder.assert_is_equal(c_reduce_a1.limbs[i], self.c[1][i]);
        }
        ext2.fp.check_mul(builder);
        ext2.fp.table.final_check(builder);
    }
}

#[test]
fn test_e2_conjugate(){
    // let compile_result = compile(&E2ConjugateCircuit::default()).unwrap();
    let compile_result =
    compile_generic(&E2ConjugateCircuit::default(), CompileOptions::default()).unwrap();
    let mut hint_registry = HintRegistry::<M31>::new();
    hint_registry.register("myhint.mulhint", mul_hint);
    hint_registry.register("myhint.simple_rangecheck_hint", simple_rangecheck_hint);
    hint_registry.register("myhint.querycounthint", query_count_hint);
    hint_registry.register("myhint.dive2hint", div_e2_hint);
    let mut assignment = E2ConjugateCircuit::<M31> {
        a: [[M31::from(0); 48], [M31::from(0); 48]],
        c: [[M31::from(0); 48], [M31::from(0); 48]],
    };

    let x0_bytes = [89,156,69,194,144,213,244,116,63,190,210,105,4,3,175,7,101,54,28,7,18,172,79,84,237,54,73,82,129,140,106,156,148,208,55,92,9,173,33,66,123,235,204,136,44,150,98,10,];
    let x1_bytes = [236,205,45,143,165,12,10,61,83,59,118,233,115,199,99,173,46,152,211,133,250,124,121,183,156,51,67,26,197,238,173,72,255,131,102,60,79,157,114,50,88,209,73,233,20,196,157,18,];
    let z0_bytes = [89,156,69,194,144,213,244,116,63,190,210,105,4,3,175,7,101,54,28,7,18,172,79,84,237,54,73,82,129,140,106,156,148,208,55,92,9,173,33,66,123,235,204,136,44,150,98,10,];
    let z1_bytes = [191,220,209,112,90,243,244,124,172,196,221,199,138,56,72,113,245,93,221,112,166,85,183,175,34,223,65,217,191,92,201,27,216,40,229,6,103,10,169,24,66,21,54,80,213,77,99,7,];

    for i in 0..48 {
        assignment.a[0][i] = M31::from(x0_bytes[i] as u32);
        assignment.a[1][i] = M31::from(x1_bytes[i] as u32);
        assignment.c[0][i] = M31::from(z0_bytes[i] as u32);
        assignment.c[1][i] = M31::from(z1_bytes[i] as u32);
    }

    debug_eval(
        &E2ConjugateCircuit::default(),
        &assignment,
        hint_registry,
    );
}

declare_circuit!(E2InverseCircuit {
    a: [[Variable; 48];2],
    c: [[Variable; 48];2],
});

impl GenericDefine<M31Config> for E2InverseCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let mut ext2 = Ext2::new(builder);
        let a_e2 = GE2 {
            a0: new_internal_element(self.a[0].to_vec(), 0),
            a1: new_internal_element(self.a[1].to_vec(), 0),
        };
        let c = ext2.inverse(builder, &a_e2);
        let c_reduce_a0 = ext2.fp.reduce(builder, c.a0.clone(), false);
        let c_reduce_a1 = ext2.fp.reduce(builder, c.a1.clone(), false);

        for i in 0..48 {
            println!("{}: {:?} {:?}", i, builder.value_of(c_reduce_a0.limbs[i]), builder.value_of(self.c[0][i]));
            println!("{}: {:?} {:?}", i, builder.value_of(c_reduce_a1.limbs[i]), builder.value_of(self.c[1][i]));
            builder.assert_is_equal(c_reduce_a0.limbs[i], self.c[0][i]);
            builder.assert_is_equal(c_reduce_a1.limbs[i], self.c[1][i]);
        }
        ext2.fp.check_mul(builder);
        ext2.fp.table.final_check(builder);
    }
}

#[test]
fn test_e2_inverse(){
    let compile_result =
    compile_generic(&E2InverseCircuit::default(), CompileOptions::default()).unwrap();
    let mut hint_registry = HintRegistry::<M31>::new();
    hint_registry.register("myhint.mulhint", mul_hint);
    hint_registry.register("myhint.simple_rangecheck_hint", simple_rangecheck_hint);
    hint_registry.register("myhint.querycounthint", query_count_hint);
    hint_registry.register("myhint.dive2hint", div_e2_hint);
    hint_registry.register("myhint.inversee2hint", inverse_e2_hint);
    let mut assignment = E2InverseCircuit::<M31> {
        a: [[M31::from(0); 48], [M31::from(0); 48]],
        c: [[M31::from(0); 48], [M31::from(0); 48]],
    };

    let x0_bytes = [89,156,69,194,144,213,244,116,63,190,210,105,4,3,175,7,101,54,28,7,18,172,79,84,237,54,73,82,129,140,106,156,148,208,55,92,9,173,33,66,123,235,204,136,44,150,98,10,];
    let x1_bytes = [236,205,45,143,165,12,10,61,83,59,118,233,115,199,99,173,46,152,211,133,250,124,121,183,156,51,67,26,197,238,173,72,255,131,102,60,79,157,114,50,88,209,73,233,20,196,157,18,];
    let z0_bytes = [188,73,170,2,86,109,56,49,4,214,214,65,170,212,146,167,82,42,230,70,169,141,41,214,126,246,187,34,14,112,134,20,9,143,115,7,74,103,198,27,169,146,135,186,148,116,195,13,];
    let z1_bytes = [25,50,4,38,189,74,213,48,113,22,13,43,46,44,21,243,221,101,44,217,100,12,139,227,50,156,163,74,52,27,167,130,108,55,41,186,118,30,138,246,64,0,64,43,180,117,173,10,];

    for i in 0..48 {
        assignment.a[0][i] = M31::from(x0_bytes[i] as u32);
        assignment.a[1][i] = M31::from(x1_bytes[i] as u32);
        assignment.c[0][i] = M31::from(z0_bytes[i] as u32);
        assignment.c[1][i] = M31::from(z1_bytes[i] as u32);
    }

    debug_eval(
        &E2InverseCircuit::default(),
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