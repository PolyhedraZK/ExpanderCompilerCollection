use crate::gnark::emparam::bls12381_fp;
use crate::gnark::emulated::field_bls12381::e2::CurveF;
use crate::gnark::emulated::field_bls12381::e2::GE2;
use crate::gnark::hints::register_hint;
use crate::gnark::limbs::*;
use crate::gnark::utils::*;
use crate::gnark::emparam::FieldParams;
use crate::gnark::element::*;
use crate::gnark::emulated::point;
use expander_compiler::frontend::extra::*;
use expander_compiler::{circuit::layered::InputType, frontend::*};
use expander_compiler::frontend::builder::*;

/*
type g2AffP struct {
	X, Y fields_bls12381.E2
}
type lineEvaluation struct {
	R0, R1 fields_bls12381.E2
}
type lineEvaluations [2][len(bls12381.LoopCounter) - 1]*lineEvaluation
// G2Affine represents G2 element with optional embedded line precomputations.
type G2Affine struct {
	P     g2AffP
	Lines *lineEvaluations
}
*/

pub struct G2AffP {
    pub x: GE2,
    pub y: GE2
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
        LineEvaluations([[None; 63]; 2].map(|row:[Option<bls12381_fp>; 63] | row.map(|_| None)))
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
