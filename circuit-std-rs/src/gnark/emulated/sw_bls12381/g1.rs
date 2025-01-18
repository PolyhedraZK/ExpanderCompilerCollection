use std::str::FromStr;

use crate::big_int::*;
use crate::gnark::element::*;
use crate::gnark::emparam::Bls12381Fp;
use crate::gnark::emulated::field_bls12381::e2::CurveF;
use crate::utils::simple_select;
use expander_compiler::frontend::*;
use num_bigint::BigInt;

const M_COMPRESSED_SMALLEST: u8 = 0b100 << 5;
const M_COMPRESSED_LARGEST: u8 = 0b101 << 5;

#[derive(Default, Clone)]
pub struct G1Affine {
    pub x: Element<Bls12381Fp>,
    pub y: Element<Bls12381Fp>,
}
impl G1Affine {
    pub fn new(x: Element<Bls12381Fp>, y: Element<Bls12381Fp>) -> Self {
        Self { x, y }
    }
    pub fn from_vars(x: Vec<Variable>, y: Vec<Variable>) -> Self {
        Self {
            x: Element::new(x, 0, false, false, false, Variable::default()),
            y: Element::new(y, 0, false, false, false, Variable::default()),
        }
    }
    pub fn one<C: Config, B: RootAPI<C>>(native: &mut B) -> Self {
        //g1Gen.X.SetString("3685416753713387016781088315183077757961620795782546409894578378688607592378376318836054947676345821548104185464507")
        //g1Gen.Y.SetString("1339506544944476473020471379941921221584933875938349620426543736416511423956333506472724655353366534992391756441569")
        Self {
            x: value_of::<C, B, Bls12381Fp>(native, Box::new("3685416753713387016781088315183077757961620795782546409894578378688607592378376318836054947676345821548104185464507".to_string())),
            y: value_of::<C, B, Bls12381Fp>(native, Box::new("1339506544944476473020471379941921221584933875938349620426543736416511423956333506472724655353366534992391756441569".to_string())),
        }
    }
}
pub struct G1 {
    pub curve_f: CurveF,
    pub w: Element<Bls12381Fp>,
}

impl G1 {
    pub fn new<C: Config, B: RootAPI<C>>(native: &mut B) -> Self {
        let curve_f = CurveF::new(native, Bls12381Fp {});
        let w = value_of::<C, B, Bls12381Fp>( native, Box::new("4002409555221667392624310435006688643935503118305586438271171395842971157480381377015405980053539358417135540939436".to_string()));

        Self { curve_f, w }
    }
    pub fn add<C: Config, B: RootAPI<C>>(
        &mut self,
        native: &mut B,
        p: &G1Affine,
        q: &G1Affine,
    ) -> G1Affine {
        let qypy = self.curve_f.sub(native, &q.y, &p.y);
        let qxpx = self.curve_f.sub(native, &q.x, &p.x);
        let λ = self.curve_f.div(native, &qypy, &qxpx);

        let λλ = self.curve_f.mul(native, &λ, &λ);
        let qxpx = self.curve_f.add(native, &p.x, &q.x);
        let xr = self.curve_f.sub(native, &λλ, &qxpx);

        let pxrx = self.curve_f.sub(native, &p.x, &xr);
        let λpxrx = self.curve_f.mul(native, &λ, &pxrx);
        let yr = self.curve_f.sub(native, &λpxrx, &p.y);

        G1Affine { x: xr, y: yr }
    }
    pub fn uncompressed<C: Config, B: RootAPI<C>>(
        &mut self,
        native: &mut B,
        bytes: &[Variable],
    ) -> G1Affine {
        let mut buf_x = bytes.to_vec();
        let buf0 = to_binary(native, buf_x[0], 8);
        let pad = vec![native.constant(0); 5];
        let m_data = from_binary(native, [pad, buf0[5..].to_vec()].concat()); //buf0 & mMask
        let buf0_and_non_mask = from_binary(native, buf0[..5].to_vec()); //buf0 & ^mMask
        buf_x[0] = buf0_and_non_mask;

        //get p.x
        let rev_buf = buf_x.iter().rev().cloned().collect::<Vec<_>>();
        let px = new_internal_element(rev_buf, 0);

        //get YSquared
        let ysquared = self.curve_f.mul(native, &px, &px);
        let ysquared = self.curve_f.mul(native, &ysquared, &px);
        let b_curve_coeff = value_of::<C, B, Bls12381Fp>( native, Box::new(4));
        let ysquared = self.curve_f.add(native, &ysquared, &b_curve_coeff);

        let inputs = vec![ysquared.clone()];
        let outputs = self
            .curve_f
            .new_hint(native, "myhint.getelementsqrthint", 2, inputs);

        //is_square should be one
        let is_square = outputs[0].clone();
        let one = self.curve_f.one_const.clone();
        self.curve_f.assert_isequal(native, &is_square, &one);

        //get Y
        let y = outputs[1].clone();
        //y^2 = ysquared
        let y_squared = self.curve_f.mul(native, &y, &y);
        self.curve_f.assert_isequal(native, &y_squared, &ysquared);


        //if y is lexicographically largest
        let half_fp = BigInt::from_str("4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559787").unwrap() / 2;
        let half_fp_var = value_of::<C, B, Bls12381Fp>( native, Box::new(half_fp));
        let is_large = big_less_than(native, Bls12381Fp::bits_per_limb() as usize, Bls12381Fp::nb_limbs() as usize, &half_fp_var.limbs, &y.limbs);

        //if Y > -Y --> check if mData == mCompressedSmallest
        //if Y <= -Y --> check if mData == mCompressedLargest
        let m_compressed_largest = native.constant(M_COMPRESSED_LARGEST as u32);
        let m_compressed_smallest = native.constant(M_COMPRESSED_SMALLEST as u32);
        let check_m_data = simple_select(native, is_large, m_compressed_smallest, m_compressed_largest);

        let check_res = native.sub(m_data, check_m_data);
        let neg_flag = native.is_zero(check_res);

        let neg_y = self.curve_f.neg(native, &y);

        let y = self.curve_f.select(native, neg_flag, &neg_y, &y);

        //TBD: subgroup check, do we need to do that? Since we are pretty sure that the public key bytes are correct, its unmashalling must be on the right curve
        G1Affine { x: px, y }
    }
}


declare_circuit!(G1UncompressCircuit {
    x: [Variable; 48],
    y: [[Variable; 48];2],
});

impl GenericDefine<M31Config> for G1UncompressCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let mut g1 = G1::new(builder);
        let public_key = g1.uncompressed(builder, &self.x);
        let expected_g1 = G1Affine::from_vars(self.y[0].to_vec(), self.y[1].to_vec());
        g1.curve_f.assert_isequal(builder, &public_key.x, &expected_g1.x);
        g1.curve_f.assert_isequal(builder, &public_key.y, &expected_g1.y);
        g1.curve_f.check_mul(builder);
        g1.curve_f.table.final_check(builder);
        g1.curve_f.table.final_check(builder);
        g1.curve_f.table.final_check(builder);
        
    }
}

#[cfg(test)]
mod tests {
    use super::G1UncompressCircuit;
    use expander_compiler::frontend::*;
    use num_bigint::BigInt;
    use num_traits::Num;
    use crate::utils::register_hint;
    use extra::debug_eval;
    #[test]
    fn test_uncompress_g1(){
        let mut hint_registry = HintRegistry::<M31>::new();
        register_hint(&mut hint_registry);
        let mut assignment = G1UncompressCircuit::<M31> {
            x: [M31::default(); 48],
            y: [[M31::default(); 48]; 2],
        };
        let x_bigint = BigInt::from_str_radix("a637bd4aefa20593ff82bdf832db2a98ca60c87796bca1d04a5a0206d52b4ede0e906d903360e04b69f8daec631f79fe", 16).unwrap();
        
        let x_bytes = x_bigint.to_bytes_be();

        let y_a0_bigint = BigInt::from_str_radix("956996561804650125715590823042978408716123343953697897618645235063950952926609558156980737775438019700668816652798", 10).unwrap();
        let y_a1_bigint = BigInt::from_str_radix("3556009343530533802204184826723274316816769528634825602353881354158551671080148026501040863742187196667680827782849", 10).unwrap();

        let y_a0_bytes = y_a0_bigint.to_bytes_le();
        let y_a1_bytes = y_a1_bigint.to_bytes_le();

        for i in 0..48{
            assignment.x[i] = M31::from(x_bytes.1[i] as u32);
            assignment.y[0][i] = M31::from(y_a0_bytes.1[i] as u32);
            assignment.y[1][i] = M31::from(y_a1_bytes.1[i] as u32);
        }

        debug_eval(&G1UncompressCircuit::default(), &assignment, hint_registry);
    }
}