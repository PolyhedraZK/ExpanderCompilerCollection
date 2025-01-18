use std::str::FromStr;

use crate::gnark::element::*;
use crate::gnark::field::*;
use crate::gnark::emparam::Bls12381Fp;
use crate::gnark::emulated::field_bls12381::e2::Ext2;
use crate::gnark::emulated::field_bls12381::e2::GE2;
use crate::gnark::utils::print_e2;
use expander_compiler::declare_circuit;
use expander_compiler::frontend::{Config, Variable, RootAPI};
use num_bigint::BigInt;
#[derive(Default, Clone)]
pub struct G2AffP {
    pub x: GE2,
    pub y: GE2,
}

impl G2AffP {
    pub fn new(x: GE2, y: GE2) -> Self {
        Self { x, y }
    }
    pub fn from_vars(
        x0: Vec<Variable>,
        y0: Vec<Variable>,
        x1: Vec<Variable>,
        y1: Vec<Variable>,
    ) -> Self {
        Self {
            x: GE2::from_vars(x0, y0),
            y: GE2::from_vars(x1, y1),
        }
    }
}

pub struct G2 {
    pub ext2: Ext2,
	pub u1: Element<Bls12381Fp>,
    pub v: GE2,
}

impl G2 {
    pub fn new<C: Config, B: RootAPI<C>>(native: &mut B) -> Self {
        let curve_f = Ext2::new(native);
        let u1 = value_of::<C, B, Bls12381Fp>(native, Box::new("4002409555221667392624310435006688643935503118305586438271171395842971157480381377015405980053539358417135540939437".to_string()));
        let v0 = value_of::<C, B, Bls12381Fp>(native, Box::new("2973677408986561043442465346520108879172042883009249989176415018091420807192182638567116318576472649347015917690530".to_string()));
        let v1 = value_of::<C, B, Bls12381Fp>(native, Box::new("1028732146235106349975324479215795277384839936929757896155643118032610843298655225875571310552543014690878354869257".to_string()));
        let v = GE2::from_vars(v0.limbs, v1.limbs);
        Self { ext2: curve_f, u1, v }
    }
    pub fn neg<C: Config, B: RootAPI<C>>(&mut self, native: &mut B, p: &G2AffP) -> G2AffP {
        let yr = self.ext2.neg(native, &p.y);
        G2AffP::new(p.x.my_clone(), yr)
    }
    pub fn copy_g2_aff_p<C: Config, B: RootAPI<C>>(
        &mut self,
        native: &mut B,
        q: &G2AffP,
    ) -> G2AffP {
        let copy_q_acc_x = self.ext2.copy(native, &q.x);
        let copy_q_acc_y = self.ext2.copy(native, &q.y);
        G2AffP {
            x: copy_q_acc_x,
            y: copy_q_acc_y,
        }
    }
    pub fn g2_double<C: Config, B: RootAPI<C>>(
        &mut self,
        native: &mut B,
        p: &G2AffP,
    ) -> G2AffP {
        let xx3a = self.ext2.square(native, &p.x);
        let xx3a = self.ext2.mul_by_const_element(native, &xx3a, &BigInt::from(3));
        let y2 = self.ext2.double(native, &p.y);
        let λ = self.ext2.div(native, &xx3a, &y2);

        let x2 = self.ext2.double(native, &p.x);
        let λλ = self.ext2.square(native, &λ);
        let xr = self.ext2.sub(native, &λλ, &x2);

        let pxrx = self.ext2.sub(native, &p.x, &xr);
        let λpxrx = self.ext2.mul(native, &λ, &pxrx);
        let yr = self.ext2.sub(native, &λpxrx, &p.y);

        G2AffP::new(xr, yr)
    }
    pub fn assert_is_equal<C: Config, B: RootAPI<C>>(
        &mut self,
        native: &mut B,
        p: &G2AffP,
        q: &G2AffP,
    ) {
        self.ext2.assert_isequal(native, &p.x, &q.x);
        self.ext2.assert_isequal(native, &p.y, &q.y);
    }
    pub fn g2_add<C: Config, B: RootAPI<C>>(
        &mut self,
        native: &mut B,
        p: &G2AffP,
        q: &G2AffP,
    ) -> G2AffP {
        let qypy = self.ext2.sub(native, &q.y, &p.y);
        let qxpx = self.ext2.sub(native, &q.x, &p.x);
        let λ = self.ext2.div(native, &qypy, &qxpx);

        let λλ = self.ext2.square(native, &λ);
        let qxpx = self.ext2.add(native, &p.x, &q.x);
        let xr = self.ext2.sub(native, &λλ, &qxpx);

        let pxrx = self.ext2.sub(native, &p.x, &xr);
        let λpxrx = self.ext2.mul(native, &λ, &pxrx);
        let yr = self.ext2.sub(native, &λpxrx, &p.y);

        G2AffP::new(xr, yr)
    }
    pub fn psi<C: Config, B: RootAPI<C>>(&mut self, native: &mut B, q: &G2AffP) -> G2AffP {
        let x = self.ext2.mul_by_element(native, &q.x, &self.u1);
        let y = self.ext2.conjugate(native, &q.y);
        let y = self.ext2.mul(native, &y, &self.v);
        G2AffP::new(GE2::from_vars(x.a1.limbs, x.a0.limbs), y)
    }
    pub fn mul_windowed<C: Config, B: RootAPI<C>>(
        &mut self,
        native: &mut B,
        q: &G2AffP,
        s: BigInt,
    ) -> G2AffP {
        let mut ops = [
            self.copy_g2_aff_p(native, q),
            self.copy_g2_aff_p(native, q),
            self.copy_g2_aff_p(native, q),
        ];
        ops[1] = self.g2_double(native, &ops[1]);
        ops[2] = self.g2_add(native, &ops[0], &ops[1]);
        let b = s.to_bytes_be();
        let b = &b.1[1..];
        let mut res = self.copy_g2_aff_p(native, &ops[2]);

        res = self.g2_double(native, &res);
        res = self.g2_double(native, &res);
        res = self.g2_add(native, &res, &ops[0]);
        
        res = self.g2_double(native, &res);
        res = self.g2_double(native, &res);
        
        res = self.g2_double(native, &res);
        res = self.g2_double(native, &res);
        res = self.g2_add(native, &res, &ops[1]);
        for w in b {
            let mut mask = 0xc0;
            for j in 0..4 {
                res = self.g2_double(native, &res);
                res = self.g2_double(native, &res);
                let c = (w & mask) >> (6 - 2*j);
                if c != 0 {
                    res = self.g2_add(native, &res, &ops[(c - 1) as usize]);
                }
                mask >>= 2;
            }
        }
        res
    }
    pub fn clear_cofactor<C: Config, B: RootAPI<C>>(
        &mut self,
        native: &mut B,
        p: &G2AffP,
    ) -> G2AffP {
        let p_neg = self.neg(native, &p);
        let x_big = BigInt::from_str("15132376222941642752").expect("Invalid string for BigInt");

        let xg_neg = self.mul_windowed(native, &p, x_big.clone());
        let xg = self.neg(native, &xg_neg);

        let xxg = self.mul_windowed(native, &xg, x_big.clone());
        let xxg = self.neg(native, &xxg);

        let mut res = self.g2_add(native, &xxg, &xg_neg);
        res = self.g2_add(native, &res, &p_neg);

        let mut t =  self.g2_add(native, &xg, &p_neg);
        t = self.psi(native, &t);

        res = self.g2_add(native, &res, &t);

        let t_double = self.g2_double(native, &p);

        let third_root_one_g1 = value_of::<C, B, Bls12381Fp>(native, Box::new("4002409555221667392624310435006688643935503118305586438271171395842971157480381377015405980053539358417135540939436".to_string()));
        
        let mut t_double_mul = G2AffP::new(t_double.x.my_clone(), t_double.y.my_clone());
        t_double_mul.x = self.ext2.mul_by_element(native, &t_double_mul.x, &third_root_one_g1);
        t_double_mul = self.neg(native, &t_double_mul);

        self.g2_add(native, &res, &t_double_mul)
    }
    pub fn map_to_g2<C: Config, B: RootAPI<C>>(
        &mut self,
        native: &mut B,
        in0: GE2,
    ) -> G2AffP {
        let a = GE2::from_vars(value_of::<C, B, Bls12381Fp>(native, Box::new(0)).limbs, value_of::<C, B, Bls12381Fp>(native, Box::new(240)).limbs);
        let b = GE2::from_vars(value_of::<C, B, Bls12381Fp>(native, Box::new(1012)).limbs, value_of::<C, B, Bls12381Fp>(native, Box::new(1012)).limbs);

        let xi = GE2::from_vars(value_of::<C, B, Bls12381Fp>(native, Box::new(-2i32)).limbs, value_of::<C, B, Bls12381Fp>(native, Box::new(-1i32)).limbs);

        let t_sq = self.ext2.square(native, &in0);
        let xi_t_sq = self.ext2.mul(native, &t_sq, &xi);

        let xi_2_t_4 = self.ext2.square(native, &xi_t_sq);
        let num_den_common = self.ext2.add(native, &xi_2_t_4, &xi_t_sq);

        let a_neg = self.ext2.neg(native, &a);
        let x0_den = self.ext2.mul(native, &a_neg, &num_den_common);

        let x1_den = GE2::from_vars(
            value_of::<C, B, Bls12381Fp>(native, Box::new(240)).limbs, 
            value_of::<C, B, Bls12381Fp>(native, Box::new("4002409555221667392624310435006688643935503118305586438271171395842971157480381377015405980053539358417135540939436".to_string())).limbs,
        );

        let exception = self.ext2.is_zero(native, &x0_den);

        let one = self.ext2.one().clone();
        let num_den_common = self.ext2.add(native, &num_den_common, &one);
        let x0_num = self.ext2.mul(native, &num_den_common, &b);

        let denom = self.ext2.select(native, exception, &x1_den, &x0_den);

        let x0 = self.ext2.div(native, &x0_num, &denom);

        let x0_sq = self.ext2.square(native, &x0);
        let x0_cub = self.ext2.mul(native, &x0, &x0_sq);
        let x0_a = self.ext2.mul(native, &a, &x0);
        let g_x0_tmp = self.ext2.add(native, &x0_cub, &x0_a);
        let g_x0 = self.ext2.add(native, &g_x0_tmp, &b);

        let x1 = self.ext2.mul(native, &xi_t_sq, &x0);

        let xi_3_t_6_tmp = self.ext2.mul(native, &xi_t_sq, &xi_t_sq);
        let xi_3_t_6 = self.ext2.mul(native, &xi_3_t_6_tmp, &xi_t_sq);

        let g_x1 = self.ext2.mul(native, &xi_3_t_6, &g_x0);



        let inputs = vec![g_x0.a0.my_clone(), g_x0.a1.my_clone(), g_x1.a0.my_clone(), g_x1.a1.my_clone(), in0.a0.my_clone(), in0.a1.my_clone()];
        let output = self.ext2.curve_f.new_hint(native, "myhint.getsqrtx0x1new", 3, inputs);
        let is_square = self.ext2.curve_f.is_zero(native, &output[0]);   // is_square = 0 if g_x0 has not square root, 1 otherwise
        let y = GE2 {
            a0: output[1].my_clone(),
            a1: output[2].my_clone(),
        };
        
        let y_sq = self.ext2.square(native, &y);
        let expected = self.ext2.select(native, is_square, &g_x1, &g_x0);

        self.ext2.assert_isequal(native, &expected, &y_sq);

        let in_x0_zero = self.ext2.curve_f.is_zero(native, &in0.a0);
        let y_x0_zero = self.ext2.curve_f.is_zero(native, &y.a0);
        let sgn_in = self.ext2.get_e2_sign(native, &in0, in_x0_zero);
        let sgn_y = self.ext2.get_e2_sign(native, &y, y_x0_zero);

        native.assert_is_equal(sgn_in, sgn_y);

        let out_b0 = self.ext2.select(native, is_square, &x1, &x0);
        let out_b1 = y.my_clone();
        G2AffP{
            x: out_b0,
            y: out_b1,
        }
    }
    pub fn g2_eval_polynomial<C: Config, B: RootAPI<C>>(
        &mut self,
        native: &mut B,
        monic: bool,
        coefficients: Vec<GE2>,
        x: &GE2,
    ) -> GE2 {
        let mut dst = coefficients[coefficients.len() - 1].my_clone();
        if monic {
            dst = self.ext2.add(native, &dst, &x);
        }
        for i in (0..coefficients.len() - 1).rev() {
            dst = self.ext2.mul(native, &dst, &x);
            dst = self.ext2.add(native, &dst, &coefficients[i]);
        }
        dst
    }
    pub fn g2_isogeny_x_numerator<C: Config, B: RootAPI<C>>(
        &mut self,
        native: &mut B,
        x: &GE2,
    ) -> GE2 {
        let coeff0 = GE2::from_vars(
            value_of::<C, B, Bls12381Fp>(native, Box::new("889424345604814976315064405719089812568196182208668418962679585805340366775741747653930584250892369786198727235542".to_string())).limbs,
            value_of::<C, B, Bls12381Fp>(native, Box::new("889424345604814976315064405719089812568196182208668418962679585805340366775741747653930584250892369786198727235542".to_string())).limbs,
        );
        let coeff1 = GE2::from_vars(
            value_of::<C, B, Bls12381Fp>(native, Box::new(0)).limbs,
            value_of::<C, B, Bls12381Fp>(native, Box::new("2668273036814444928945193217157269437704588546626005256888038757416021100327225242961791752752677109358596181706522".to_string())).limbs,
        );
        let coeff2 = GE2::from_vars(
            value_of::<C, B, Bls12381Fp>(native, Box::new("2668273036814444928945193217157269437704588546626005256888038757416021100327225242961791752752677109358596181706526".to_string())).limbs,
            value_of::<C, B, Bls12381Fp>(native, Box::new("1334136518407222464472596608578634718852294273313002628444019378708010550163612621480895876376338554679298090853261".to_string())).limbs,
        );
        let coeff3 = GE2::from_vars(
            value_of::<C, B, Bls12381Fp>(native, Box::new("3557697382419259905260257622876359250272784728834673675850718343221361467102966990615722337003569479144794908942033".to_string())).limbs,
            value_of::<C, B, Bls12381Fp>(native, Box::new(0)).limbs,
        );
        self.g2_eval_polynomial(native, false, vec![coeff0, coeff1, coeff2, coeff3], x)
    }
    pub fn g2_isogeny_y_numerator<C: Config, B: RootAPI<C>>(
        &mut self,
        native: &mut B,
        x: &GE2,
        y: &GE2,
    ) -> GE2 {
        let coeff0 = GE2::from_vars(
            value_of::<C, B, Bls12381Fp>(native, Box::new("3261222600550988246488569487636662646083386001431784202863158481286248011511053074731078808919938689216061999863558".to_string())).limbs,
            value_of::<C, B, Bls12381Fp>(native, Box::new("3261222600550988246488569487636662646083386001431784202863158481286248011511053074731078808919938689216061999863558".to_string())).limbs,
        );
        let coeff1 = GE2::from_vars(
            value_of::<C, B, Bls12381Fp>(native, Box::new(0)).limbs,
            value_of::<C, B, Bls12381Fp>(native, Box::new("889424345604814976315064405719089812568196182208668418962679585805340366775741747653930584250892369786198727235518".to_string())).limbs,
        );
        let coeff2 = GE2::from_vars(
            value_of::<C, B, Bls12381Fp>(native, Box::new("2668273036814444928945193217157269437704588546626005256888038757416021100327225242961791752752677109358596181706524".to_string())).limbs,
            value_of::<C, B, Bls12381Fp>(native, Box::new("1334136518407222464472596608578634718852294273313002628444019378708010550163612621480895876376338554679298090853263".to_string())).limbs,
        );
        let coeff3 = GE2::from_vars(
            value_of::<C, B, Bls12381Fp>(native, Box::new("2816510427748580758331037284777117739799287910327449993381818688383577828123182200904113516794492504322962636245776".to_string())).limbs,
            value_of::<C, B, Bls12381Fp>(native, Box::new(0)).limbs,
        );
        let dst = self.g2_eval_polynomial(native, false, vec![coeff0, coeff1, coeff2, coeff3], x);
        self.ext2.mul(native, &dst, &y)
    }
    pub fn g2_isogeny_x_denominator<C: Config, B: RootAPI<C>>(
        &mut self,
        native: &mut B,
        x: &GE2,
    ) -> GE2 {
        let coeff0 = GE2::from_vars(
            value_of::<C, B, Bls12381Fp>(native, Box::new(0)).limbs,
            value_of::<C, B, Bls12381Fp>(native, Box::new(-72)).limbs,
        );
        let coeff1 = GE2::from_vars(
            value_of::<C, B, Bls12381Fp>(native, Box::new(12)).limbs,
            value_of::<C, B, Bls12381Fp>(native, Box::new(-12)).limbs,
        );
        self.g2_eval_polynomial(native, true, vec![coeff0, coeff1], x)
    }
    pub fn g2_isogeny_y_denominator<C: Config, B: RootAPI<C>>(
        &mut self,
        native: &mut B,
        x: &GE2,
    ) -> GE2 {
        let coeff0 = GE2::from_vars(
            value_of::<C, B, Bls12381Fp>(native, Box::new(-432)).limbs,
            value_of::<C, B, Bls12381Fp>(native, Box::new(-432)).limbs,
        );
        let coeff1 = GE2::from_vars(
            value_of::<C, B, Bls12381Fp>(native, Box::new(0)).limbs,
            value_of::<C, B, Bls12381Fp>(native, Box::new(-216)).limbs,
        );
        let coeff2 = GE2::from_vars(
            value_of::<C, B, Bls12381Fp>(native, Box::new(18)).limbs,
            value_of::<C, B, Bls12381Fp>(native, Box::new(-18)).limbs,
        );
        self.g2_eval_polynomial(native, true, vec![coeff0, coeff1, coeff2], x)
    }
    pub fn g2_isogeny<C: Config, B: RootAPI<C>>(
        &mut self,
        native: &mut B,
        p: &G2AffP,
    ) -> G2AffP {
        let mut p = G2AffP {
            x: p.x.my_clone(),
            y: p.y.my_clone(),
        };
        let den1 = self.g2_isogeny_y_denominator(native, &p.x);
        let den0 = self.g2_isogeny_x_denominator(native, &p.x);
        let mut den = vec![den0, den1];
        p.y = self.g2_isogeny_y_numerator(native, &p.x, &p.y);
        p.x = self.g2_isogeny_x_numerator(native, &p.x);

        den[0] = self.ext2.inverse(native, &den[0]);
        den[1] = self.ext2.inverse(native, &den[1]);

        p.x = self.ext2.mul(native, &p.x, &den[0]);
        p.y = self.ext2.mul(native, &p.y, &den[1]);
        p
    }

}
#[derive(Default)]
pub struct LineEvaluation {
    pub r0: GE2,
    pub r1: GE2,
}

type LineEvaluationArray = [[Option<Box<LineEvaluation>>; 63]; 2];

pub struct LineEvaluations(pub LineEvaluationArray);

impl Default for LineEvaluations {
    fn default() -> Self {
        LineEvaluations([[None; 63]; 2].map(|row: [Option<Bls12381Fp>; 63]| row.map(|_| None)))
    }
}
impl LineEvaluations {
    pub fn is_empty(&self) -> bool {
        self.0
            .iter()
            .all(|row| row.iter().all(|cell| cell.is_none()))
    }
}
pub struct G2Affine {
    pub p: G2AffP,
    pub lines: LineEvaluations,
}


/*

type MapToG2Circuit struct {
	In0 fields_bls12381_m31.E2
	In1 fields_bls12381_m31.E2
	Out G2Affine
}

func (c *MapToG2Circuit) Define(api frontend.API) error {
	logup.Reset()
	logup.NewRangeProof(8)
	g2 := NewG2(api)
	e := g2.Ext2
	out0 := MapToG2(e, api, c.In0)
	out1 := MapToG2(e, api, c.In1)
	out := g2.G2Add(&out0, &out1)
	new_out := g2Isogeny(e, out)

	res := ClearCofactor(g2, new_out)

	e.AssertIsEqual(&res.P.X, &c.Out.P.X)
	e.AssertIsEqual(&res.P.Y, &c.Out.P.Y)
	return nil
}
*/

