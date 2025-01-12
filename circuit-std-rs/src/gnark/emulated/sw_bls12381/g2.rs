use crate::gnark::emparam::Bls12381Fp;
use crate::gnark::emulated::field_bls12381::e2::Ext2;
use crate::gnark::emulated::field_bls12381::e2::GE2;
use expander_compiler::frontend::*;
#[derive(Default,Clone)]
pub struct G2AffP {
    pub x: GE2,
    pub y: GE2
}

impl G2AffP {
    pub fn new(x: GE2, y: GE2) -> Self {
        Self {
            x,
            y,
        }
    }
    pub fn from_vars(x0: Vec<Variable>, y0: Vec<Variable>, x1: Vec<Variable>, y1: Vec<Variable>) -> Self {
        Self {
            x: GE2::from_vars(x0, y0),
            y: GE2::from_vars(x1, y1),
        }
    }
}

pub struct G2 {
    pub curve_f: Ext2,
}

impl G2 {
    pub fn new<'a, C: Config, B: RootAPI<C>>(native: &'a mut B) -> Self {
        let curve_f = Ext2::new(native);
        Self {
            curve_f,
        }
    }
    pub fn neg<'a, C: Config, B: RootAPI<C>>(&mut self, native: &'a mut B, p: &G2AffP) -> G2AffP {
        let yr = self.curve_f.neg(native, &p.y);
        G2AffP::new(p.x.clone(), yr)
    }
}

pub struct LineEvaluation {
    pub r0: GE2,
    pub r1: GE2
}
impl Default for LineEvaluation {
    fn default() -> Self {
        LineEvaluation { r0: GE2::default(), r1: GE2::default() }
    }
}
// pub type LineEvaluations = [[Option<Box<LineEvaluation>>; 64 - 1]; 2];
type LineEvaluationArray = [[Option<Box<LineEvaluation>>; 63]; 2];

pub struct LineEvaluations(pub LineEvaluationArray);

impl Default for LineEvaluations {
    fn default() -> Self {
        LineEvaluations([[None; 63]; 2].map(|row:[Option<Bls12381Fp>; 63] | row.map(|_| None)))
    }
}
impl LineEvaluations {
    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|row| row.iter().all(|cell| cell.is_none()))
    }
}
pub struct G2Affine {
    pub p: G2AffP,
    pub lines: LineEvaluations
}
