use crate::gnark::element::*;
use crate::gnark::emparam::Bls12381Fp;
use crate::gnark::emulated::field_bls12381::e2::CurveF;
use expander_compiler::frontend::*;

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
}
