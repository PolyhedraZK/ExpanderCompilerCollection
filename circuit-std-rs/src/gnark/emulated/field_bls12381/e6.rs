use crate::gnark::{element::Element, emparam::Bls12381Fp};
use expander_compiler::frontend::{Config, RootAPI, Variable};
use num_bigint::BigInt;

use super::e2::*;
#[derive(Default, Clone)]
pub struct GE6 {
    pub b0: GE2,
    pub b1: GE2,
    pub b2: GE2,
}
impl GE6 {
    pub fn my_clone(&self) -> Self {
        GE6 {
            b0: self.b0.my_clone(),
            b1: self.b1.my_clone(),
            b2: self.b2.my_clone(),
        }
    }
}
pub struct Ext6 {
    pub ext2: Ext2,
}

impl Ext6 {
    pub fn new<C: Config, B: RootAPI<C>>(api: &mut B) -> Self {
        Self {
            ext2: Ext2::new(api),
        }
    }
    pub fn one(&mut self) -> GE6 {
        let b0 = self.ext2.one();
        let b1 = self.ext2.zero();
        let b2 = self.ext2.zero();
        GE6 { b0, b1, b2 }
    }
    pub fn zero<C: Config, B: RootAPI<C>>(&mut self) -> GE6 {
        let b0 = self.ext2.zero();
        let b1 = self.ext2.zero();
        let b2 = self.ext2.zero();
        GE6 { b0, b1, b2 }
    }
    pub fn is_zero<C: Config, B: RootAPI<C>>(&mut self, native: &mut B, z: &GE6) -> Variable {
        let b0 = self.ext2.is_zero(native, &z.b0.my_clone());
        let b1 = self.ext2.is_zero(native, &z.b1.my_clone());
        let b2 = self.ext2.is_zero(native, &z.b2.my_clone());
        let tmp = native.and(b0, b1);
        native.and(tmp, b2)
    }
    pub fn add<C: Config, B: RootAPI<C>>(&mut self, native: &mut B, x: &GE6, y: &GE6) -> GE6 {
        let z0 = self.ext2.add(native, &x.b0.my_clone(), &y.b0.my_clone());
        let z1 = self.ext2.add(native, &x.b1.my_clone(), &y.b1.my_clone());
        let z2 = self.ext2.add(native, &x.b2.my_clone(), &y.b2.my_clone());
        GE6 {
            b0: z0,
            b1: z1,
            b2: z2,
        }
    }
    pub fn neg<C: Config, B: RootAPI<C>>(&mut self, native: &mut B, x: &GE6) -> GE6 {
        let z0 = self.ext2.neg(native, &x.b0.my_clone());
        let z1 = self.ext2.neg(native, &x.b1.my_clone());
        let z2 = self.ext2.neg(native, &x.b2.my_clone());
        GE6 {
            b0: z0,
            b1: z1,
            b2: z2,
        }
    }
    pub fn sub<C: Config, B: RootAPI<C>>(&mut self, native: &mut B, x: &GE6, y: &GE6) -> GE6 {
        let z0 = self.ext2.sub(native, &x.b0.my_clone(), &y.b0.my_clone());
        let z1 = self.ext2.sub(native, &x.b1.my_clone(), &y.b1.my_clone());
        let z2 = self.ext2.sub(native, &x.b2.my_clone(), &y.b2.my_clone());
        GE6 {
            b0: z0,
            b1: z1,
            b2: z2,
        }
    }
    pub fn double<C: Config, B: RootAPI<C>>(&mut self, native: &mut B, x: &GE6) -> GE6 {
        let z0 = self.ext2.double(native, &x.b0.my_clone());
        let z1 = self.ext2.double(native, &x.b1.my_clone());
        let z2 = self.ext2.double(native, &x.b2.my_clone());
        GE6 {
            b0: z0,
            b1: z1,
            b2: z2,
        }
    }
    pub fn square<C: Config, B: RootAPI<C>>(&mut self, native: &mut B, x: &GE6) -> GE6 {
        let c4 = self.ext2.mul(native, &x.b0.my_clone(), &x.b1.my_clone());
        let c4 = self.ext2.double(native, &c4);
        let c5 = self.ext2.square(native, &x.b2.my_clone());
        let c1 = self.ext2.mul_by_non_residue(native, &c5);
        let c1 = self.ext2.add(native, &c1, &c4);
        let c2 = self.ext2.sub(native, &c4, &c5);
        let c3 = self.ext2.square(native, &x.b0.my_clone());
        let c4 = self.ext2.sub(native, &x.b0.my_clone(), &x.b1.my_clone());
        let c4 = self.ext2.add(native, &c4, &x.b2.my_clone());
        let c5 = self.ext2.mul(native, &x.b1.my_clone(), &x.b2.my_clone());
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
    pub fn mul_by_e2<C: Config, B: RootAPI<C>>(&mut self, native: &mut B, x: &GE6, y: &GE2) -> GE6 {
        let z0 = self.ext2.mul(native, &x.b0.my_clone(), y);
        let z1 = self.ext2.mul(native, &x.b1.my_clone(), y);
        let z2 = self.ext2.mul(native, &x.b2.my_clone(), y);
        GE6 {
            b0: z0,
            b1: z1,
            b2: z2,
        }
    }
    pub fn mul_by_12<C: Config, B: RootAPI<C>>(
        &mut self,
        native: &mut B,
        x: &GE6,
        b1: &GE2,
        b2: &GE2,
    ) -> GE6 {
        let t1 = self.ext2.mul(native, &x.b1.my_clone(), b1);
        let t2 = self.ext2.mul(native, &x.b2.my_clone(), b2);
        let mut c0 = self.ext2.add(native, &x.b1.my_clone(), &x.b2.my_clone());
        let mut tmp = self.ext2.add(native, b1, b2);
        c0 = self.ext2.mul(native, &c0, &tmp);
        tmp = self.ext2.add(native, &t1, &t2);
        c0 = self.ext2.sub(native, &c0, &tmp);
        c0 = self.ext2.mul_by_non_residue(native, &c0);
        let mut c1 = self.ext2.add(native, &x.b0.my_clone(), &x.b1.my_clone());
        c1 = self.ext2.mul(native, &c1, b1);
        c1 = self.ext2.sub(native, &c1, &t1);
        tmp = self.ext2.mul_by_non_residue(native, &t2);
        c1 = self.ext2.add(native, &c1, &tmp);
        tmp = self.ext2.add(native, &x.b0.my_clone(), &x.b2.my_clone());
        let mut c2 = self.ext2.mul(native, b2, &tmp);
        c2 = self.ext2.sub(native, &c2, &t2);
        c2 = self.ext2.add(native, &c2, &t1);
        GE6 {
            b0: c0,
            b1: c1,
            b2: c2,
        }
    }
    pub fn mul_by_0<C: Config, B: RootAPI<C>>(&mut self, native: &mut B, z: &GE6, c0: &GE2) -> GE6 {
        let a = self.ext2.mul(native, &z.b0.my_clone(), c0);
        let tmp = self.ext2.add(native, &z.b0.my_clone(), &z.b2.my_clone());
        let mut t2 = self.ext2.mul(native, c0, &tmp);
        t2 = self.ext2.sub(native, &t2, &a);
        let tmp = self.ext2.add(native, &z.b0.my_clone(), &z.b1.my_clone());
        let mut t1 = self.ext2.mul(native, c0, &tmp);
        t1 = self.ext2.sub(native, &t1, &a);
        GE6 {
            b0: a,
            b1: t1,
            b2: t2,
        }
    }
    pub fn mul_by_01<C: Config, B: RootAPI<C>>(
        &mut self,
        native: &mut B,
        z: &GE6,
        c0: &GE2,
        c1: &GE2,
    ) -> GE6 {
        let a = self.ext2.mul(native, &z.b0, c0);
        let b = self.ext2.mul(native, &z.b1, c1);
        let tmp = self.ext2.add(native, &z.b1.my_clone(), &z.b2.my_clone());
        let mut t0 = self.ext2.mul(native, c1, &tmp);

        t0 = self.ext2.sub(native, &t0, &b);
        t0 = self.ext2.mul_by_non_residue(native, &t0);
        t0 = self.ext2.add(native, &t0, &a);
        let mut t2 = self.ext2.mul(native, &z.b2.my_clone(), c0);
        t2 = self.ext2.add(native, &t2, &b);
        let mut t1 = self.ext2.add(native, c0, c1);
        let tmp = self.ext2.add(native, &z.b0.my_clone(), &z.b1.my_clone());
        t1 = self.ext2.mul(native, &t1, &tmp);
        let tmp = self.ext2.add(native, &a, &b);
        t1 = self.ext2.sub(native, &t1, &tmp);
        GE6 {
            b0: t0,
            b1: t1,
            b2: t2,
        }
    }
    pub fn mul_by_non_residue<C: Config, B: RootAPI<C>>(&mut self, native: &mut B, x: &GE6) -> GE6 {
        let z0 = self.ext2.mul_by_non_residue(native, &x.b2.my_clone());
        GE6 {
            b0: z0,
            b1: x.b0.my_clone(),
            b2: x.b1.my_clone(),
        }
    }
    pub fn assert_isequal<C: Config, B: RootAPI<C>>(&mut self, native: &mut B, x: &GE6, y: &GE6) {
        self.ext2.assert_isequal(native, &x.b0, &y.b0);
        self.ext2.assert_isequal(native, &x.b1, &y.b1);
        self.ext2.assert_isequal(native, &x.b2, &y.b2);
    }
    pub fn select<C: Config, B: RootAPI<C>>(
        &mut self,
        native: &mut B,
        selector: Variable,
        z1: &GE6,
        z0: &GE6,
    ) -> GE6 {
        let b0 = self
            .ext2
            .select(native, selector, &z1.b0.my_clone(), &z0.b0.my_clone());
        let b1 = self
            .ext2
            .select(native, selector, &z1.b1.my_clone(), &z0.b1.my_clone());
        let b2 = self
            .ext2
            .select(native, selector, &z1.b2.my_clone(), &z0.b2.my_clone());
        GE6 { b0, b1, b2 }
    }
    pub fn mul_karatsuba_over_karatsuba<C: Config, B: RootAPI<C>>(
        &mut self,
        native: &mut B,
        x: &GE6,
        y: &GE6,
    ) -> GE6 {
        let t0 = self.ext2.mul(native, &x.b0.my_clone(), &y.b0.my_clone());
        let t1 = self.ext2.mul(native, &x.b1.my_clone(), &y.b1.my_clone());
        let t2 = self.ext2.mul(native, &x.b2.my_clone(), &y.b2.my_clone());
        let mut c0 = self.ext2.add(native, &x.b1.my_clone(), &x.b2.my_clone());
        let mut tmp = self.ext2.add(native, &y.b1.my_clone(), &y.b2.my_clone());
        c0 = self.ext2.mul(native, &c0, &tmp);
        tmp = self.ext2.add(native, &t2, &t1);
        c0 = self.ext2.sub(native, &c0, &tmp);
        c0 = self.ext2.mul_by_non_residue(native, &c0);
        c0 = self.ext2.add(native, &c0, &t0);
        let mut c1 = self.ext2.add(native, &x.b0.my_clone(), &x.b1.my_clone());
        tmp = self.ext2.add(native, &y.b0.my_clone(), &y.b1.my_clone());
        c1 = self.ext2.mul(native, &c1, &tmp);
        tmp = self.ext2.add(native, &t0, &t1);
        c1 = self.ext2.sub(native, &c1, &tmp);
        tmp = self.ext2.mul_by_non_residue(native, &t2);
        c1 = self.ext2.add(native, &c1, &tmp);
        let mut tmp = self.ext2.add(native, &x.b0.my_clone(), &x.b2.my_clone());
        let mut c2 = self.ext2.add(native, &y.b0.my_clone(), &y.b2.my_clone());
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
    pub fn mul<C: Config, B: RootAPI<C>>(&mut self, native: &mut B, x: &GE6, y: &GE6) -> GE6 {
        self.mul_karatsuba_over_karatsuba(native, x, y)
    }
    pub fn div<C: Config, B: RootAPI<C>>(&mut self, native: &mut B, x: &GE6, y: &GE6) -> GE6 {
        let inputs = vec![
            x.b0.a0.my_clone(),
            x.b0.a1.my_clone(),
            x.b1.a0.my_clone(),
            x.b1.a1.my_clone(),
            x.b2.a0.my_clone(),
            x.b2.a1.my_clone(),
            y.b0.a0.my_clone(),
            y.b0.a1.my_clone(),
            y.b1.a0.my_clone(),
            y.b1.a1.my_clone(),
            y.b2.a0.my_clone(),
            y.b2.a1.my_clone(),
        ];
        let output = self
            .ext2
            .curve_f
            .new_hint(native, "myhint.dive6hint", 6, inputs);
        let div = GE6 {
            b0: GE2 {
                a0: output[0].my_clone(),
                a1: output[1].my_clone(),
            },
            b1: GE2 {
                a0: output[2].my_clone(),
                a1: output[3].my_clone(),
            },
            b2: GE2 {
                a0: output[4].my_clone(),
                a1: output[5].my_clone(),
            },
        };
        let _x = self.mul(native, &div, y);
        self.assert_isequal(native, &x.my_clone(), &_x);
        div
    }
    pub fn inverse_div<C: Config, B: RootAPI<C>>(&mut self, native: &mut B, x: &GE6) -> GE6 {
        let one = self.one();
        self.div(native, &one, x)
    }
    pub fn inverse<C: Config, B: RootAPI<C>>(&mut self, native: &mut B, x: &GE6) -> GE6 {
        let inputs = vec![
            x.b0.a0.my_clone(),
            x.b0.a1.my_clone(),
            x.b1.a0.my_clone(),
            x.b1.a1.my_clone(),
            x.b2.a0.my_clone(),
            x.b2.a1.my_clone(),
        ];
        let output = self
            .ext2
            .curve_f
            .new_hint(native, "myhint.inversee6hint", 6, inputs);
        let inv = GE6 {
            b0: GE2 {
                a0: output[0].my_clone(),
                a1: output[1].my_clone(),
            },
            b1: GE2 {
                a0: output[2].my_clone(),
                a1: output[3].my_clone(),
            },
            b2: GE2 {
                a0: output[4].my_clone(),
                a1: output[5].my_clone(),
            },
        };
        let one = self.one();
        let _one = self.mul(native, &inv, x);
        self.assert_isequal(native, &one, &_one);
        inv
    }
    pub fn div_e6_by_6<C: Config, B: RootAPI<C>>(
        &mut self,
        native: &mut B,
        x: &[Element<Bls12381Fp>; 6],
    ) -> [Element<Bls12381Fp>; 6] {
        let inputs = vec![
            x[0].my_clone(),
            x[1].my_clone(),
            x[2].my_clone(),
            x[3].my_clone(),
            x[4].my_clone(),
            x[5].my_clone(),
        ];
        let output = self
            .ext2
            .curve_f
            .new_hint(native, "myhint.dive6by6hint", 6, inputs);
        let y0 = output[0].my_clone();
        let y1 = output[1].my_clone();
        let y2 = output[2].my_clone();
        let y3 = output[3].my_clone();
        let y4 = output[4].my_clone();
        let y5 = output[5].my_clone();
        let x0 = self.ext2.curve_f.mul_const(native, &y0, BigInt::from(6));
        let x1 = self.ext2.curve_f.mul_const(native, &y1, BigInt::from(6));
        let x2 = self.ext2.curve_f.mul_const(native, &y2, BigInt::from(6));
        let x3 = self.ext2.curve_f.mul_const(native, &y3, BigInt::from(6));
        let x4 = self.ext2.curve_f.mul_const(native, &y4, BigInt::from(6));
        let x5 = self.ext2.curve_f.mul_const(native, &y5, BigInt::from(6));
        self.ext2.curve_f.assert_isequal(native, &x[0], &x0);
        self.ext2.curve_f.assert_isequal(native, &x[1], &x1);
        self.ext2.curve_f.assert_isequal(native, &x[2], &x2);
        self.ext2.curve_f.assert_isequal(native, &x[3], &x3);
        self.ext2.curve_f.assert_isequal(native, &x[4], &x4);
        self.ext2.curve_f.assert_isequal(native, &x[5], &x5);
        [y0, y1, y2, y3, y4, y5]
    }
}
