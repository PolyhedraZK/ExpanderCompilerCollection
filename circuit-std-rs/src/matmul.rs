use crate::StdCircuitGeneric;
use arith::Field;
use expander_compiler::frontend::*;
use std::convert::From;
use std::ops::{AddAssign, Mul};

#[derive(Clone, Copy, Debug)]
pub struct MatMulParams {
    pub m1: usize,
    pub n1: usize,
    pub m2: usize,
    pub n2: usize,
}

declare_circuit!(_MatMulCircuit {
    // first matrix
    first_mat: [[Variable]],
    // second matrix
    second_mat: [[Variable]],
    // result matrix
    result_mat: [[Variable]],
});

pub type MatMulCircuit = _MatMulCircuit<Variable>;

impl<C: Config> GenericDefine<C> for MatMulCircuit {
    fn define<Builder: RootAPI<C>>(&self, builder: &mut Builder) {
        // [m1,n1] represents the first matrix's dimension
        let m1 = self.first_mat.len();
        let n1 = self.first_mat[0].len();

        // [m2,n2] represents the second matrix's dimension
        let m2 = self.second_mat.len();
        let n2 = self.second_mat[0].len();

        // [r1,r2] represents the result matrix's dimension
        let r1 = self.result_mat.len();
        let r2 = self.result_mat[0].len();
        let zero = builder.constant(0);

        builder.assert_is_equal(Variable::from(n1), Variable::from(m2));
        builder.assert_is_equal(Variable::from(r1), Variable::from(m1));
        builder.assert_is_equal(Variable::from(r2), Variable::from(n2));

        let randomness = builder.get_random_value();

        let mut aux_mat = Vec::new();
        let mut chanllenge = randomness;

        // construct the aux matrix = [randomness, randomness^2, randomness^3, ..., randomness^n2]
        for i in 0..n2 {
            aux_mat.push(chanllenge);
            chanllenge = builder.mul(chanllenge, randomness);
            aux_mat[i] = chanllenge;
        }

        let mut aux_second = vec![zero; m2];
        let mut aux_first = vec![zero; m1];

        let mut aux_res = vec![zero; m1];

        // calculate second_mat * aux_mat,
        // for i in 0..m2 {
        //     for j in 0..n2 {
        //         // aux_second[i] += self.second_mat[i][j] * aux_mat[j];
        //         let mul_second = builder.mul(self.second_mat[i][j], aux_mat[j]);
        //         aux_second[i] = builder.add(aux_second[i], mul_second);
        //     }
        // }
        for (i, aux_item) in aux_second.iter_mut().enumerate() {
            for (j, item) in aux_mat.iter().enumerate() {
                // aux_second[i] += self.second_mat[i][j] * aux_mat[j];
                let mul_second = builder.mul(self.second_mat[i][j], item);
                *aux_item = builder.add(*aux_item, mul_second);
            }
        }

        // calculate result_mat * aux_second
        // for i in 0..m1 {
        //     for j in 0..n2 {
        //         // aux_res[i] += self.result_mat[i][j] * aux_mat[j];
        //         let mul_res = builder.mul(self.result_mat[i][j], aux_mat[j]);
        //         aux_res[i] = builder.add(aux_res[i], mul_res);
        //     }
        // }
        for (i, aux_item) in aux_res.iter_mut().enumerate() {
            for (j, item) in aux_mat.iter().enumerate() {
                // aux_second[i] += self.second_mat[i][j] * aux_mat[j];
                let mul_res = builder.mul(self.result_mat[i][j], item);
                *aux_item = builder.add(*aux_item, mul_res);
            }
        }

        // calculate first_mat * aux_second
        // for i in 0..m1 {
        //     for j in 0..n1 {
        //         //self.first_mat[i] += self.first_mat[i][j] * aux_second[j];
        //         let mul_result = builder.mul(self.first_mat[i][j], aux_second[j]);
        //         aux_first[i] = builder.add(aux_first[i], mul_result);
        //     }
        // }
        for (i, aux_item) in aux_first.iter_mut().enumerate() {
            for (j, item) in aux_second.iter().enumerate() {
                // aux_second[i] += self.second_mat[i][j] * aux_mat[j];
                let mul_res = builder.mul(self.first_mat[i][j], item);
                *aux_item = builder.add(*aux_item, mul_res);
            }
        }

        // compare aux_first with aux_res
        for i in 0..m1 {
            builder.assert_is_equal(aux_first[i], aux_res[i]);
        }
    }
}

impl<C: Config> StdCircuitGeneric<C> for MatMulCircuit {
    type Params = MatMulParams;
    type Assignment = _MatMulCircuit<C::CircuitField>;

    fn new_circuit(params: &Self::Params) -> Self {
        let mut circuit = Self::default();

        circuit
            .first_mat
            .resize(params.m1, vec![Variable::default(); params.n1]);
        circuit
            .second_mat
            .resize(params.m2, vec![Variable::default(); params.n2]);

        circuit
            .result_mat
            .resize(params.m1, vec![Variable::default(); params.n2]);

        circuit
    }

    fn new_assignment(params: &Self::Params, mut rng: impl rand::RngCore) -> Self::Assignment {
        let mut assignment = _MatMulCircuit::<C::CircuitField>::default();
        assignment
            .first_mat
            .resize(params.m1, vec![C::CircuitField::zero(); params.n1]);
        assignment
            .second_mat
            .resize(params.m2, vec![C::CircuitField::zero(); params.n2]);
        assignment
            .result_mat
            .resize(params.m1, vec![C::CircuitField::zero(); params.n2]);

        for i in 0..params.m1 {
            for j in 0..params.n1 {
                assignment.first_mat[i][j] = C::CircuitField::random_unsafe(&mut rng);
            }
        }
        for i in 0..params.m2 {
            for j in 0..params.n2 {
                assignment.second_mat[i][j] = C::CircuitField::random_unsafe(&mut rng);
            }
        }

        // initialize the aux matrix with random values.
        // result matrix should be computed
        assignment.result_mat = matrix_multiply::<C>(&assignment.first_mat, &assignment.second_mat);

        assignment
    }
}

// this helper calculates matrix c = a * b;
#[allow(clippy::needless_range_loop)]
fn matrix_multiply<C: Config>(
    a: &[Vec<C::CircuitField>],
    b: &[Vec<C::CircuitField>],
) -> Vec<Vec<C::CircuitField>> {
    let m1 = a.len();
    let n1 = a[0].len();
    let m2 = b.len();
    let n2 = b[0].len();

    assert_eq!(n1, m2, "n1 ! = m2 ");

    // initilize the result matrix
    let mut c = vec![vec![C::CircuitField::default(); n2]; m1];

    // FIXME: optimize calculating the multiplication for super large matrix.
    for i in 0..m1 {
        for j in 0..n2 {
            for k in 0..n1 {
                c[i][j].add_assign(a[i][k].mul(b[k][j]));
            }
        }
    }

    c
}
