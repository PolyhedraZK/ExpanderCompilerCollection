use crate::gnark::emparam::bls12381_fp;
use crate::gnark::emulated::field_bls12381::e12::*;
use crate::gnark::emulated::field_bls12381::e2::*;
use crate::gnark::emulated::field_bls12381::e2::print_e2;
use crate::gnark::emulated::field_bls12381::e2::print_element;
use crate::gnark::emulated::field_bls12381::e6::GE6;
use crate::gnark::hints::register_hint;
use crate::gnark::field::*;
use crate::gnark::limbs::*;
use crate::gnark::utils::*;
use crate::gnark::emparam::FieldParams;
use crate::gnark::element::*;
use crate::gnark::emulated::point;
use expander_compiler::frontend::extra::*;
use expander_compiler::{circuit::layered::InputType, frontend::*};
use expander_compiler::frontend::builder::*;
use num_bigint::BigInt;

use super::g1::G1Affine;
use super::g2::G2AffP;
use super::g2::G2Affine;
use super::g2::LineEvaluation;
use super::g2::LineEvaluations;
const loop_counter:[i8;64] = [
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
	0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
	0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
	0, 0, 1, 0, 0, 1, 0, 1, 1,
];
pub struct Pairing {
    pub ext12: Ext12,
    pub curve_f: CurveF,
}

impl Pairing {
    pub fn new<'a, C: Config, B: RootAPI<C>>(native: &'a mut B) -> Self {
        let curve_f = CurveF::new(native, bls12381_fp{});
        let ext12 = Ext12::new(native);
        Self {
            curve_f,
            ext12,
        }
    }

    /*
    func (pr Pairing) PairingCheck(P []*G1Affine, Q []*G2Affine) error {
	// pr.MillerLoopTest(P, Q)
	f, err := pr.MillerLoop(P, Q)
	// f, err := pr.MillerLoopOnlyLines(P, Q)
	if err != nil {
		return err
	}
	// We perform the easy part of the final exp to push f to the cyclotomic
	// subgroup so that AssertFinalExponentiationIsOne is carried with optimized
	// cyclotomic squaring (e.g. Karabina12345).
	//
	// f = f^(p⁶-1)(p²+1)
	buf := pr.Conjugate(f)
	buf = pr.DivUnchecked(buf, f)
	f = pr.FrobeniusSquare(buf)
	f = pr.Mul(f, buf)

	pr.AssertFinalExponentiationIsOne(f)

	return nil
}
     */
    pub fn pairing_check<'a, C: Config, B: RootAPI<C>>(&mut self, native: &'a mut B, p: &[G1Affine], q: &mut [G2Affine]) -> Result<(), Error> {
        let f = self.miller_loop(native, p, q).unwrap();
        println!("f");
        print_e2(native, &f.c1.b0);
        panic!("stop");
        let buf = self.ext12.conjugate(native, &f);
        println!("buff");
        print_e2(native, &buf.c1.b0);
        let buf = self.ext12.div(native, &buf, &f);
        println!("buff");
        print_e2(native, &buf.c1.b0);
        let f = self.ext12.frobenius_square(native, &buf);
        println!("f");
        print_e2(native, &f.c1.b0);
        let f = self.ext12.mul(native, &f, &buf);
        println!("f");
        print_e2(native, &f.c1.b0);

        panic!("stop");
        self.ext12.assert_final_exponentiation_is_one(native, &f);

        Ok(())
    }
    pub fn miller_loop<'a, C: Config, B: RootAPI<C>>(&mut self, native: &'a mut B, p: &[G1Affine], q: &mut [G2Affine]) -> Result<GE12, String> {
        let n = p.len();
        if n == 0 || n != q.len() {
            return Err("nvalid inputs sizes".to_string());
        }
        let mut lines = vec![];
        for i in 0..q.len() {
            if q[i].lines.is_empty() {
                let qlines = self.compute_lines_with_hint(native, &q[i].p);
                q[i].lines = qlines;
                let tmp_r0 = q[i].lines.0[0][0].as_ref().unwrap().r0.clone();
                let tmp_r1 = q[i].lines.0[0][0].as_ref().unwrap().r1.clone();
                // panic!("stop");
            }
            let line_evaluations = std::mem::take(&mut q[i].lines);
            lines.push(line_evaluations);
            // lines.push(q[i].lines.clone());
        }
        self.miller_loop_lines_with_hint(native, p, lines)
    }
    pub fn miller_loop_lines_with_hint<'a, C: Config, B: RootAPI<C>>(&mut self, native: &'a mut B, p: &[G1Affine], lines: Vec<LineEvaluations>) -> Result<GE12, String> {
        let n = p.len();
        if n == 0 || n != lines.len() {
            return Err("invalid inputs sizes".to_string());
        }
        let mut y_inv = vec![];
        let mut x_neg_over_y = vec![];
        for k in 0..n {
            let y_inv_k = self.curve_f.inverse(native, &p[k].y);
            let x_neg_over_y_k = self.curve_f.mul(native, &p[k].x, &y_inv_k);
            let x_neg_over_y_k = self.curve_f.neg(native, &x_neg_over_y_k);
            y_inv.push(y_inv_k);
            x_neg_over_y.push(x_neg_over_y_k);
        }

        let mut res = self.ext12.one();

        if let Some(line_evaluation) = &lines[0].0[0][62] {
            let line = line_evaluation; 
            res.c0.b0 = self.ext12.ext6.ext2.mul_by_element(native, &line.r1, &y_inv[0]);
            res.c0.b1 = self.ext12.ext6.ext2.mul_by_element(native, &line.r0, &x_neg_over_y[0]);
        } else {
            return Err("line evaluation is None".to_string());
        }
        res.c1.b1 = self.ext12.ext6.ext2.one();

        if let Some(line_evaluation) = &lines[0].0[1][62] {
            let line = line_evaluation; 
            let tmp0 = self.ext12.ext6.ext2.mul_by_element(native, &line.r1, &y_inv[0]);
            let tmp1 = self.ext12.ext6.ext2.mul_by_element(native, &line.r0, &x_neg_over_y[0]);
            let prod_lines = self.ext12.mul_014_by_014(
                native,
                &tmp0,
                &tmp1,
                &res.c0.b0,
                &res.c0.b1,
            );
            res = GE12{
                c0: GE6{
                    b0: prod_lines[0].clone(),
                    b1: prod_lines[1].clone(),
                    b2: prod_lines[2].clone(),
                },
                c1: GE6{
                    b0: res.c1.b0.clone(),
                    b1: prod_lines[3].clone(),
                    b2: prod_lines[4].clone(),
                },
            };
        } else {
            return Err("line evaluation is None".to_string());
        }

        for k in 1..n {
            if let Some(line_evaluation) = &lines[k].0[0][62] {
                let line = line_evaluation; 
                let tmp0 = self.ext12.ext6.ext2.mul_by_element(native, &line.r1, &y_inv[k]);
                let tmp1 = self.ext12.ext6.ext2.mul_by_element(native, &line.r0, &x_neg_over_y[k]);
                res = self.ext12.mul_by_014(native, &res, &tmp0, &tmp1);
            } else {
                return Err("line evaluation is None".to_string());
            }
            if let Some(line_evaluation) = &lines[k].0[1][62] {
                let line = line_evaluation;
                let tmp0 = self.ext12.ext6.ext2.mul_by_element(native, &line.r1, &y_inv[k]);
                let tmp1 = self.ext12.ext6.ext2.mul_by_element(native, &line.r0, &x_neg_over_y[k]);
                res = self.ext12.mul_by_014(native, &res, &tmp0, &tmp1);
            } else {
                return Err("line evaluation is None".to_string());
            }
        }

        let mut copy_res = self.ext12.copy(native, &res);

        for i in (0..=61).rev() {
            res = self.ext12.square(native, &copy_res);
            copy_res = self.ext12.copy(native, &res);
            for k in 0..n {
                if loop_counter[i as usize] == 0 {
                    if let Some(line_evaluation) = &lines[k].0[0][i as usize] {
                        let line = line_evaluation;
                        let tmp0 = self.ext12.ext6.ext2.mul_by_element(native, &line.r1, &y_inv[k]);
                        let tmp1 = self.ext12.ext6.ext2.mul_by_element(native, &line.r0, &x_neg_over_y[k]);
                        res = self.ext12.mul_by_014(native, &copy_res, &tmp0, &tmp1);
                        copy_res = self.ext12.copy(native, &res);
                    } else {
                        return Err("line evaluation is None".to_string());
                    }
                } else {
                    if let Some(line_evaluation) = &lines[k].0[0][i as usize] {
                        let line = line_evaluation;
                        let tmp0 = self.ext12.ext6.ext2.mul_by_element(native, &line.r1, &y_inv[k]);
                        let tmp1 = self.ext12.ext6.ext2.mul_by_element(native, &line.r0, &x_neg_over_y[k]);
                        res = self.ext12.mul_by_014(native, &copy_res, &tmp0, &tmp1);
                        copy_res = self.ext12.copy(native, &res);
                    } else {
                        return Err("line evaluation is None".to_string());
                    }
                    if let Some(line_evaluation) = &lines[k].0[1][i as usize] {
                        let line = line_evaluation;
                        let tmp0 = self.ext12.ext6.ext2.mul_by_element(native, &line.r1, &y_inv[k]);
                        let tmp1 = self.ext12.ext6.ext2.mul_by_element(native, &line.r0, &x_neg_over_y[k]);
                        res = self.ext12.mul_by_014(native, &copy_res, &tmp0, &tmp1);
                        copy_res = self.ext12.copy(native, &res);
                    } else {
                        return Err("line evaluation is None".to_string());
                    }
                }
            }
        }
        res = self.ext12.conjugate(native, &copy_res);
        // println!("res");
        // print_e2(native, &res.c1.b0);
        // print_e2(native, &res.c1.b1);
        // print_e2(native, &res.c1.b2);
        // panic!("stop");
        Ok(res)
    }
    pub fn compute_lines_with_hint<'a, C: Config, B: RootAPI<C>>(&mut self, native: &'a mut B, q: &G2AffP) -> LineEvaluations {
        // let mut c_lines = LineEvaluations::default();
        let mut c_lines: LineEvaluations = LineEvaluations::default();
        let q_acc = q.clone();
        let mut copy_q_acc = self.copy_g2_aff_p(native, q_acc);
        let n = loop_counter.len();
        let (q_acc, line1, line2) = self.triple_step(native, copy_q_acc);
        println!("QACC##########");
        print_e2(native, &q_acc.x);
        print_e2(native, &q_acc.y);
        let tmp_r0 = line1.as_ref().unwrap().r0.clone();
        let tmp_r1 = line1.as_ref().unwrap().r1.clone();
        println!("tmp_r0");
        print_e2(native, &tmp_r0);
        println!("tmp_r1");
        print_e2(native, &tmp_r1);
        // panic!("stop");
        c_lines.0[0][n-2] = line1;
        c_lines.0[1][n-2] = line2;
        copy_q_acc = self.copy_g2_aff_p(native, &q_acc);
        for i in (1..=n-3).rev() {
            if loop_counter[i] == 0 {
                let (q_acc, c_lines_0_i) = self.double_step(native, copy_q_acc);
                copy_q_acc = self.copy_g2_aff_p(native, &q_acc);
                c_lines.0[0][i] = c_lines_0_i;
            } else {
                let (q_acc, c_lines_0_i, c_lines_1_i) = self.double_and_add_step(native, copy_q_acc, q);
                copy_q_acc = self.copy_g2_aff_p(native, &q_acc);
                c_lines.0[0][i] = c_lines_0_i;
                c_lines.0[1][i] = c_lines_1_i;
            }
        }
        c_lines.0[0][0] = self.tangent_compute(native, copy_q_acc);
        c_lines
    }
    pub fn double_and_add_step<'a, C: Config, B: RootAPI<C>>(&mut self, native: &'a mut B, p1: G2AffP, p2: &G2AffP) -> (G2AffP, Option<Box<LineEvaluation>>, Option<Box<LineEvaluation>>) {
        let n = self.ext12.ext6.ext2.sub(native, &p1.y, &p2.y);
        let d = self.ext12.ext6.ext2.sub(native, &p1.x, &p2.x);
        let λ1 = self.ext12.ext6.ext2.div(native, &n, &d);

        let xr = self.ext12.ext6.ext2.square(native, &λ1);
        let tmp = self.ext12.ext6.ext2.add(native, &p1.x, &p2.x);
        let xr = self.ext12.ext6.ext2.sub(native, &xr, &tmp);

        let r0 = λ1.clone();
        let mut r1 = self.ext12.ext6.ext2.mul(native, &λ1, &p1.x);
        r1 = self.ext12.ext6.ext2.sub(native, &r1, &p1.y);

        let line1  = Some(Box::new(LineEvaluation {
            r0,
            r1,
        }));

        let d = self.ext12.ext6.ext2.sub(native, &xr, &p1.x);
        let n = self.ext12.ext6.ext2.double(native, &p1.y);
        let λ2 = self.ext12.ext6.ext2.div(native, &n, &d);
        let λ2 = self.ext12.ext6.ext2.add(native, &λ2, &λ1);
        let λ2 = self.ext12.ext6.ext2.neg(native, &λ2);

        let x4 = self.ext12.ext6.ext2.square(native, &λ2);
        let tmp = self.ext12.ext6.ext2.add(native, &p1.x, &xr);
        let x4 = self.ext12.ext6.ext2.sub(native, &x4, &tmp);

        let y4 = self.ext12.ext6.ext2.sub(native, &p1.x, &x4);
        let y4 = self.ext12.ext6.ext2.mul(native, &λ2, &y4);
        let y4 = self.ext12.ext6.ext2.sub(native, &y4, &p1.y);

        let p = G2AffP{
            x: x4,
            y: y4,
        };

        let r0 = λ2.clone();
        let mut r1 = self.ext12.ext6.ext2.mul(native, &λ2, &p1.x);
        r1 = self.ext12.ext6.ext2.sub(native, &r1, &p1.y);


        let line2  = Some(Box::new(LineEvaluation {
            r0,
            r1,
        }));

        (p, line1, line2)
    }
    pub fn double_step<'a, C: Config, B: RootAPI<C>>(&mut self, native: &'a mut B, p1: G2AffP) -> (G2AffP, Option<Box<LineEvaluation>>) {
        let n = self.ext12.ext6.ext2.square(native, &p1.x);
        let three = BigInt::from(3);
        let n = self.ext12.ext6.ext2.mul_by_const_element(native, &n, &three);
        let d = self.ext12.ext6.ext2.double(native, &p1.y);
        let λ = self.ext12.ext6.ext2.div(native, &n, &d);

        let xr = self.ext12.ext6.ext2.square(native, &λ);
        let tmp = self.ext12.ext6.ext2.mul_by_const_element(native, &p1.x, &BigInt::from(2));
        let xr = self.ext12.ext6.ext2.sub(native, &xr, &tmp);

        let pxr = self.ext12.ext6.ext2.sub(native, &p1.x, &xr);
        let λpxr = self.ext12.ext6.ext2.mul(native, &λ, &pxr);
        let yr = self.ext12.ext6.ext2.sub(native, &λpxr, &p1.y);

        let res = G2AffP{
            x: xr,
            y: yr,
        };

        let r0 = λ.clone();
        let mut r1 = self.ext12.ext6.ext2.mul(native, &λ, &p1.x);
        r1 = self.ext12.ext6.ext2.sub(native, &r1, &p1.y);

        let line  = Some(Box::new(LineEvaluation {
            r0,
            r1,
        }));

        (res, line)
    }
    pub fn triple_step<'a, C: Config, B: RootAPI<C>>(&mut self, native: &'a mut B, p1: G2AffP) -> (G2AffP, Option<Box<LineEvaluation>>, Option<Box<LineEvaluation>>) {
        

        let n = self.ext12.ext6.ext2.square(native, &p1.x);
        let three = BigInt::from(3);
        let n = self.ext12.ext6.ext2.mul_by_const_element(native, &n, &three);
        let d = self.ext12.ext6.ext2.double(native, &p1.y);
        let λ1 = self.ext12.ext6.ext2.div(native, &n, &d);

        let r0 = λ1.clone();
        let mut r1 = self.ext12.ext6.ext2.mul(native, &λ1, &p1.x);
        r1 = self.ext12.ext6.ext2.sub(native, &r1, &p1.y);

        let line1  = Some(Box::new(LineEvaluation {
            r0,
            r1,
        }));

        let x2 = self.ext12.ext6.ext2.square(native, &λ1);
        let tmp = self.ext12.ext6.ext2.mul_by_const_element(native, &p1.x, &BigInt::from(2));
        let x2 = self.ext12.ext6.ext2.sub(native, &x2, &tmp);

        let x1x2 = self.ext12.ext6.ext2.sub(native, &p1.x, &x2);
        let λ2 = self.ext12.ext6.ext2.div(native, &d, &x1x2);
        let λ2 = self.ext12.ext6.ext2.sub(native, &λ2, &λ1);

        let r0 = λ2.clone();
        let mut r1 = self.ext12.ext6.ext2.mul(native, &λ2, &p1.x);
        r1 = self.ext12.ext6.ext2.sub(native, &r1, &p1.y);

        let line2  = Some(Box::new(LineEvaluation {
            r0,
            r1,
        }));

        let λ2λ2 = self.ext12.ext6.ext2.mul(native, &λ2, &λ2);
        let qxrx = self.ext12.ext6.ext2.add(native, &x2, &p1.x);
        let xr = self.ext12.ext6.ext2.sub(native, &λ2λ2, &qxrx);

        let pxrx = self.ext12.ext6.ext2.sub(native, &p1.x, &xr);
        let λ2pxrx = self.ext12.ext6.ext2.mul(native, &λ2, &pxrx);
        let yr = self.ext12.ext6.ext2.sub(native, &λ2pxrx, &p1.y);

        let res = G2AffP{
            x: xr,
            y: yr,
        };

        (res, line1, line2)
    }
    pub fn tangent_compute<'a, C: Config, B: RootAPI<C>>(&mut self, native: &'a mut B, p1: G2AffP) -> Option<Box<LineEvaluation>> {
        let n = self.ext12.ext6.ext2.square(native, &p1.x);
        let three = BigInt::from(3);
        let n = self.ext12.ext6.ext2.mul_by_const_element(native, &n, &three);
        let d = self.ext12.ext6.ext2.double(native, &p1.y);
        let λ = self.ext12.ext6.ext2.div(native, &n, &d);

        let r0 = λ.clone();
        let mut r1 = self.ext12.ext6.ext2.mul(native, &λ, &p1.x);
        r1 = self.ext12.ext6.ext2.sub(native, &r1, &p1.y);

        let line = Some(Box::new(LineEvaluation {
            r0,
            r1,
        }));

        line
    }
    pub fn copy_g2_aff_p<'a, C: Config, B: RootAPI<C>>(&mut self, native: &'a mut B, q: &G2AffP) -> G2AffP {
        let copy_q_acc_x = self.ext12.ext6.ext2.copy(native, &q.x);
        let copy_q_acc_y = self.ext12.ext6.ext2.copy(native, &q.y);
        G2AffP {
            x: copy_q_acc_x,
            y: copy_q_acc_y,
        }
    }
}


/*
type PairingCheckGKRCircuit struct {
	In1G1 G1Affine
	In2G1 G1Affine
	In1G2 G2Affine
	In2G2 G2Affine
	// In3G2 G2Affine
	// TmpRes [62 * 3]GTEl
}

func (c *PairingCheckGKRCircuit) Define(api frontend.API) error {
	logup.Reset()
	logup.NewRangeProof(12)
	pairing, err := NewPairing(api)
	if err != nil {
		return fmt.Errorf("new pairing: %w", err)
	}
	// err = pairing.PairingCheck([]*G1Affine{&c.In1G1, &c.In1G1, &c.In2G1, &c.In2G1}, []*G2Affine{&c.In1G2, &c.In2G2, &c.In1G2, &c.In2G2})
	// hintRes := make([]*GTEl, 62*3)
	// for i := 0; i < 62*3; i++ {
	// 	hintRes[i] = &c.TmpRes[i]
	// }
	err = pairing.PairingCheck([]*G1Affine{&c.In1G1, &c.In2G1}, []*G2Affine{&c.In1G2, &c.In2G2})
	if err != nil {
		return fmt.Errorf("pair: %w", err)
	}
	logup.FinalCheck(api, logup.ColumnCombineOption)
	return nil
}

*/

declare_circuit!(PairingCheckGKRCircuit {
    in1_g1: [[Variable;48];2],
    in2_g1: [[Variable;48];2],
    in1_g2: [[[Variable;48];2];2],
    in2_g2: [[[Variable;48];2];2],
});

impl GenericDefine<M31Config> for PairingCheckGKRCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let mut pairing = Pairing::new(builder);
        let p1_g1 = G1Affine {
            x: Element::new(self.in1_g1[0].to_vec(), 0, false, false, false, Variable::default()),
            y: Element::new(self.in1_g1[1].to_vec(), 0, false, false, false, Variable::default()),
        };
        let p2_g1 = G1Affine {
            x: Element::new(self.in2_g1[0].to_vec(), 0, false, false, false, Variable::default()),
            y: Element::new(self.in2_g1[1].to_vec(), 0, false, false, false, Variable::default()),
        };
        let q1_g2 = G2AffP {
            x: GE2{
                a0: Element::new(self.in1_g2[0][0].to_vec(), 0, false, false, false, Variable::default()),
                a1: Element::new(self.in1_g2[0][1].to_vec(), 0, false, false, false, Variable::default()),
            },
            y: GE2{
                a0: Element::new(self.in1_g2[1][0].to_vec(), 0, false, false, false, Variable::default()),
                a1: Element::new(self.in1_g2[1][1].to_vec(), 0, false, false, false, Variable::default()),
            }
        };
        let q2_g2 = G2AffP {
            x: GE2{
                a0: Element::new(self.in2_g2[0][0].to_vec(), 0, false, false, false, Variable::default()),
                a1: Element::new(self.in2_g2[0][1].to_vec(), 0, false, false, false, Variable::default()),
            },
            y: GE2{
                a0: Element::new(self.in2_g2[1][0].to_vec(), 0, false, false, false, Variable::default()),
                a1: Element::new(self.in2_g2[1][1].to_vec(), 0, false, false, false, Variable::default()),
            }
        };
        let mut r = pairing.pairing_check(builder, &[p1_g1, p2_g1], &mut [G2Affine{p: q1_g2, lines: LineEvaluations::default()}, G2Affine{p: q2_g2, lines: LineEvaluations::default()}]).unwrap();
        pairing.ext12.ext6.ext2.fp.check_mul(builder);
        pairing.ext12.ext6.ext2.fp.table.final_check(builder);
    }
}

#[test]
fn test_pairing_check_gkr() {
    // let compile_result =
    // compile_generic(&PairingCheckGKRCircuit::default(), CompileOptions::default()).unwrap();
    let mut hint_registry = HintRegistry::<M31>::new();
    register_hint(&mut hint_registry);
    let mut assignment = PairingCheckGKRCircuit::<M31> {
        in1_g1: [[M31::from(0);48];2],
        in2_g1: [[M31::from(0);48];2],
        in1_g2: [[[M31::from(0);48];2];2],
        in2_g2: [[[M31::from(0);48];2];2],
    };
    let p1_x_bytes = [138,209,41,52,20,222,185,9,48,234,53,109,218,26,76,112,204,195,135,184,95,253,141,179,243,220,94,195,151,34,112,210,63,186,25,221,129,128,76,209,101,191,44,36,248,25,127,3,];
    let p1_y_bytes = [97,193,54,196,208,241,229,252,144,121,89,115,226,242,251,60,142,182,216,242,212,30,189,82,97,228,230,80,38,19,77,187,242,96,65,136,115,75,173,136,35,202,199,3,37,33,182,19,];
    let p2_x_bytes = [53,43,44,191,248,216,253,96,84,253,43,36,151,202,77,190,19,71,28,215,161,72,57,211,182,58,152,199,107,235,238,63,160,97,190,43,89,195,111,179,72,18,109,141,133,74,215,16,];
    let p2_y_bytes = [96,0,147,41,253,168,205,45,124,150,80,188,171,228,217,34,233,192,87,38,176,98,88,196,41,115,40,174,52,234,97,53,209,179,91,66,107,130,187,171,10,254,6,227,50,212,34,8,];
    let q1_x0_bytes = [115,71,82,0,253,98,21,231,188,204,204,250,44,169,184,249,132,60,132,14,34,48,165,84,111,109,143,182,32,72,227,210,133,144,154,196,16,169,138,79,19,122,34,156,176,236,114,22,];
    let q1_x1_bytes = [182,57,221,84,50,87,48,115,6,98,38,176,152,25,126,43,201,61,87,42,225,138,200,170,0,20,174,117,112,157,233,97,0,149,210,18,224,229,157,26,197,93,245,96,227,157,237,15,];
    let q1_y0_bytes = [185,67,44,184,194,122,245,73,123,160,144,28,83,227,9,222,52,33,74,97,66,113,234,143,125,244,115,58,79,29,83,208,130,83,146,30,95,202,3,189,0,6,81,73,107,141,234,1,];
    let q1_y1_bytes = [113,182,199,78,243,62,126,145,147,111,153,151,219,69,54,127,72,82,59,169,219,65,228,8,193,143,67,158,12,45,225,109,220,217,133,185,75,245,82,200,137,178,165,90,190,232,244,21,];
    let q2_x0_bytes = [48,100,73,236,161,161,88,235,92,188,236,139,70,238,43,160,189,118,66,116,44,222,23,195,67,252,105,112,240,119,247,53,3,24,156,3,178,117,41,16,120,114,244,103,65,157,255,21,];
    let q2_x1_bytes = [87,198,239,80,28,107,195,211,220,50,148,176,2,30,65,17,206,180,103,123,161,64,40,77,84,98,25,164,111,180,209,62,23,78,4,174,123,52,30,19,149,4,6,56,6,173,138,12,];
    let q2_y0_bytes = [178,164,255,33,62,219,245,30,146,252,242,196,23,5,90,103,75,9,67,186,155,40,106,209,158,161,142,60,109,58,29,180,3,126,95,225,244,243,36,82,32,223,19,39,202,170,158,12,];
    let q2_y1_bytes = [47,93,130,172,91,197,69,2,220,41,78,230,47,199,202,197,177,54,53,90,233,76,186,248,212,121,120,208,231,195,87,150,233,33,103,94,11,15,108,247,78,10,223,139,186,5,53,8,];

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

    debug_eval(&PairingCheckGKRCircuit::default(), &assignment, hint_registry);
    
}