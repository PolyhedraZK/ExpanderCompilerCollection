use crate::gnark::emparam::Bls12381Fp;
use crate::gnark::emulated::field_bls12381::e2::CurveF;
use crate::gnark::hints::register_hint;
use crate::gnark::element::*;
use expander_compiler::frontend::extra::*;
use expander_compiler::frontend::*;

#[derive(Default,Clone)]
pub struct G1Affine {
    pub x: Element<Bls12381Fp>,
    pub y: Element<Bls12381Fp>,
}
impl G1Affine {
    pub fn new(x: Element<Bls12381Fp>, y: Element<Bls12381Fp>) -> Self {
        Self {
            x,
            y,
        }
    }
    pub fn from_vars(x: Vec<Variable>, y: Vec<Variable>) -> Self {
        Self {
            x: Element::new(x, 0, false, false, false, Variable::default()),
            y: Element::new(y, 0, false, false, false, Variable::default()),
        }
    }
    pub fn one<'a, C: Config, B: RootAPI<C>>(native: &'a mut B) -> Self {
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
    pub fn new<'a, C: Config, B: RootAPI<C>>(native: &'a mut B) -> Self {
        let curve_f = CurveF::new(native, Bls12381Fp{});
        let w = value_of::<C, B, Bls12381Fp>( native, Box::new("4002409555221667392624310435006688643935503118305586438271171395842971157480381377015405980053539358417135540939436".to_string()));
        
        Self {
            curve_f,
            w,
        }
    }
    pub fn add<'a, C: Config, B: RootAPI<C>>(&mut self, native: &'a mut B, p: &G1Affine, q: &G1Affine) -> G1Affine {
        let qypy = self.curve_f.sub(native, &q.y, &p.y);
        let qxpx = self.curve_f.sub(native, &q.x, &p.x);
        let λ = self.curve_f.div(native, &qypy, &qxpx);

        let λλ = self.curve_f.mul(native, &λ, &λ);
        let qxpx = self.curve_f.add(native, &p.x, &q.x);
        let xr = self.curve_f.sub(native, &λλ, &qxpx);

        let pxrx = self.curve_f.sub(native, &p.x, &xr);
        let λpxrx = self.curve_f.mul(native, &λ, &pxrx);
        let yr = self.curve_f.sub(native, &λpxrx, &p.y);

        G1Affine {
            x: xr,
            y: yr,
        }
    }
}

declare_circuit!(G1AddCircuit {
    p: [[Variable;48];2],
    q: [[Variable;48];2],
    r: [[Variable;48];2],
});

impl GenericDefine<M31Config> for G1AddCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let mut g1 = G1::new(builder);
        let p1_g1 = G1Affine {
            x: Element::new(self.p[0].to_vec(), 0, false, false, false, Variable::default()),
            y: Element::new(self.p[1].to_vec(), 0, false, false, false, Variable::default()),
        };
        let p2_g1 = G1Affine {
            x: Element::new(self.q[0].to_vec(), 0, false, false, false, Variable::default()),
            y: Element::new(self.q[1].to_vec(), 0, false, false, false, Variable::default()),
        };
        let r_g1 = G1Affine {
            x: Element::new(self.r[0].to_vec(), 0, false, false, false, Variable::default()),
            y: Element::new(self.r[1].to_vec(), 0, false, false, false, Variable::default()),
        };
        let mut r = g1.add(builder, &p1_g1, &p2_g1);
        for i in 0..16{
            r = g1.add(builder, &r, &p2_g1);
        }
        g1.curve_f.assert_isequal(builder, &r.x, &r_g1.x);
        g1.curve_f.assert_isequal(builder, &r.y, &r_g1.y);
        g1.curve_f.check_mul(builder);
        g1.curve_f.table.final_check(builder);
    }
}

#[test]
fn test_g1_add() {
    let compile_result =
    compile_generic(&G1AddCircuit::default(), CompileOptions::default()).unwrap();
	let mut hint_registry = HintRegistry::<M31>::new();
    register_hint(&mut hint_registry);
    let mut assignment = G1AddCircuit::<M31> {
        p: [[M31::from(0); 48]; 2],
        q: [[M31::from(0); 48]; 2],
        r: [[M31::from(0); 48]; 2]
    };
    let p1_x_bytes = [169,204,143,202,195,182,32,187,150,46,27,88,137,82,209,11,255,228,147,72,218,149,56,139,243,28,49,146,210,5,238,232,111,204,78,170,83,191,222,173,137,165,150,240,62,27,213,8,];
    let p1_y_bytes = [85,56,238,125,65,131,108,201,186,2,96,151,226,80,22,2,111,141,203,67,50,147,209,102,238,82,12,96,172,239,2,177,184,146,208,150,63,214,239,198,101,74,169,226,148,53,104,1,];
    let p2_x_bytes = [108,4,52,16,255,115,116,198,234,60,202,181,169,240,221,33,38,178,114,195,169,16,147,33,62,116,10,191,25,163,79,192,140,43,109,235,157,42,15,48,115,213,48,51,19,165,178,17,];
    let p2_y_bytes = [130,146,65,1,211,117,217,145,69,140,76,106,43,160,192,247,96,225,2,72,219,238,254,202,9,210,253,111,73,49,26,145,68,161,64,101,238,0,236,128,164,92,95,30,143,178,6,20,];
    let res_x_bytes = [148,92,212,64,35,246,218,14,150,169,177,191,61,6,4,120,60,253,36,139,95,95,14,122,89,3,62,198,100,50,114,221,144,187,29,15,203,89,220,29,120,25,153,169,184,184,133,16,];
    let res_y_bytes = [41,226,254,238,50,145,74,128,160,125,237,161,93,66,241,104,218,230,154,134,24,204,225,220,175,115,243,57,238,157,161,175,213,34,145,106,226,230,19,110,196,196,229,104,152,64,12,6,];
    
    for i in 0..48 {
        assignment.p[0][i] = M31::from(p1_x_bytes[i]);
        assignment.p[1][i] = M31::from(p1_y_bytes[i]);
        assignment.q[0][i] = M31::from(p2_x_bytes[i]);
        assignment.q[1][i] = M31::from(p2_y_bytes[i]);
        assignment.r[0][i] = M31::from(res_x_bytes[i]);
        assignment.r[1][i] = M31::from(res_y_bytes[i]);
    }

    debug_eval(&G1AddCircuit::default(), &assignment, hint_registry);
    
}

