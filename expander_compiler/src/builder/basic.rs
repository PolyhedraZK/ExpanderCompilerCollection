//! Basic builder.

use std::collections::{BinaryHeap, HashMap};

use crate::{
    circuit::{
        config::Config,
        ir::{
            self,
            common::{Constraint, Instruction as _, IrConfig},
            expr::{Expression, LinComb, Term, VarSpec},
        },
    },
    field::{Field, FieldArith},
    frontend::CircuitField,
    utils::{error::Error, pool::Pool},
};

/// The root builder is used to process the input root circuit, generating an output circuit.
///
/// Builder process:
/// Ir(in_vars) --> Builder(mid_vars) --> Ir(out_vars)
/// Each in_var corresponds to an out_var.
/// Each mid_var corresponds to 1. an out_var, or 2. an internal variable of mid_vars.
/// Each out_var corresponds to an expression of mid_vars.
/// Also, each internal variable points to kx+b where x is an out_var.
///
/// A "var" means mid_var by default.
///
/// The root builder can process different input and output IR configurations,
/// allowing for flexibility in how circuits are built and transformed.
pub struct RootBuilder<'a, C: Config, IrcIn: IrConfig<Config = C>, IrcOut: IrConfig<Config = C>> {
    /// The root circuit being processed.
    pub rc: &'a ir::common::RootCircuit<IrcIn>,
    /// Builders for each circuit in the root circuit.
    pub builders: HashMap<usize, Builder<'a, C, IrcIn, IrcOut>>,
    /// Output circuits generated from the input circuits.
    pub out_circuits: HashMap<usize, ir::common::Circuit<IrcOut>>,
}

/// The builder for a specific circuit
pub struct Builder<'a, C: Config, IrcIn: IrConfig<Config = C>, IrcOut: IrConfig<Config = C>> {
    /// The input circuit being processed.
    pub in_circuit: &'a ir::common::Circuit<IrcIn>,
    /// The ID of the input circuit.
    pub in_circuit_id: usize,

    /// Map for constraints.
    ///
    /// If it's known to be true (e.g. in previous gates or in sub circuits), mark it.
    /// If it's required to be true, assert it.
    pub constraints: HashMap<
        <IrcOut::Constraint as Constraint<C>>::Type,
        HashMap<Expression<C>, ConstraintStatus>,
    >,

    /// Out_var mapped to expression of mid_vars
    pub out_var_exprs: Vec<Expression<C>>,

    /// Pool of mid_vars
    ///
    /// For internal variables, the expression is actual expression.
    /// For in_vars, the expression is a fake expression with only one term.
    pub mid_vars: Pool<Expression<C>>,
    /// Each internal variable points to kx+b where x is an out_var
    pub mid_to_out: Vec<Option<OutVarRef<C>>>,
    /// Inverse of out_var_exprs
    pub mid_expr_to_out: HashMap<Expression<C>, usize>,

    /// In_var to out_var
    pub in_to_out: Vec<usize>,

    /// Output instructions
    pub out_insns: Vec<IrcOut::Instruction>,
}

/// Reference to an output variable in the circuit.
/// This reference means that the variable is represented as kx + b,
/// where x is the index of an output variable.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OutVarRef<C: Config> {
    /// The index of the output variable.
    pub x: usize,
    /// The coefficient k in the expression kx + b.
    pub k: CircuitField<C>,
    /// The constant term b in the expression kx + b.
    pub b: CircuitField<C>,
}

/// Status of a constraint in the circuit.
///
/// `Marked` means the constraint is known to be true, while `Asserted` means it is required to be true.
///
/// For example, in an binary AND operation, if we asserted that `a` and `b` are both binary,
/// we can mark the result `c = a * b` as binary without asserting it again.
#[derive(Debug, Clone)]
pub enum ConstraintStatus {
    Marked,
    Asserted,
}

/// Metadata for a linear combination in the circuit.
///
/// This struct is used in the `lin_comb_inner` function to compute linear combinations of variables.
/// For details, see the `lin_comb_inner` function documentation.
pub struct LinMeta {
    pub l_id: usize,
    pub t_id: usize,
    pub vars: VarSpec,
}

impl PartialEq for LinMeta {
    fn eq(&self, other: &Self) -> bool {
        self.vars == other.vars
    }
}

impl Eq for LinMeta {}

impl Ord for LinMeta {
    /// Compare two `LinMeta` instances.
    /// Since `BinaryHeap` is a max-heap, we reverse the order to get a min-heap behavior.
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.vars.cmp(&other.vars).reverse()
    }
}

impl PartialOrd for LinMeta {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Result of transforming an instruction from input IR to output IR.
pub enum InsnTransformResult<C: Config, IrcOut: IrConfig<Config = C>> {
    /// The transformed instruction in the output IR.
    Insn(IrcOut::Instruction),
    /// A list of output variable IDs that correspond to the transformed instruction.
    Vars(Vec<usize>),
    /// An error occurred during the transformation.
    Err(Error),
}

/// Trait for transforming and executing instructions.
pub trait InsnTransformAndExecute<
    'a,
    C: Config,
    IrcIn: IrConfig<Config = C>,
    IrcOut: IrConfig<Config = C>,
>
{
    /// Transforms an input instruction to an output instruction.
    fn transform_in_to_out(
        &mut self,
        in_insn: &IrcIn::Instruction,
    ) -> InsnTransformResult<C, IrcOut>;
    /// Executes an output instruction, potentially using a root builder for additional context.
    fn execute_out<'b>(
        &mut self,
        out_insn: &IrcOut::Instruction,
        root: Option<&'b RootBuilder<'a, C, IrcIn, IrcOut>>,
    ) where
        'a: 'b;
    /// Transforms an input constraint to an output constraint.
    fn transform_in_con_to_out(
        &mut self,
        in_con: &IrcIn::Constraint,
    ) -> Result<IrcOut::Constraint, Error>;
}

impl<'a, C: Config, IrcIn: IrConfig<Config = C>, IrcOut: IrConfig<Config = C>>
    Builder<'a, C, IrcIn, IrcOut>
{
    /// Creates a new builder for the given input circuit.
    pub fn new(in_circuit_id: usize, in_circuit: &'a ir::common::Circuit<IrcIn>) -> Self {
        let mut res: Builder<'a, C, IrcIn, IrcOut> = Builder {
            in_circuit,
            in_circuit_id,
            constraints: HashMap::new(),
            out_var_exprs: vec![Expression::default()],
            mid_vars: Pool::new(),
            mid_to_out: Vec::new(),
            mid_expr_to_out: HashMap::new(),
            in_to_out: vec![0],
            out_insns: Vec::new(),
        };
        res.mid_vars.add(&Expression::invalid());
        res.mid_to_out.push(None);
        res
    }

    /// Returns the constant value of an output variable by its ID, if it is constant.
    pub fn constant_value(&self, out_var_id: usize) -> Option<CircuitField<C>> {
        self.out_var_exprs[out_var_id].constant_value()
    }

    /// Creates a new variable in the mid_vars pool and returns its ID.
    fn new_var(&mut self) -> usize {
        let id = self.mid_vars.len();
        assert_eq!(
            self.mid_vars
                .add(&Expression::new_linear(CircuitField::<C>::one(), id)),
            id
        );
        id
    }

    /// Adds `n` new output variables to the builder.
    pub fn add_out_vars(&mut self, n: usize) {
        let start = self.mid_vars.len();
        for i in 0..n {
            self.new_var();
            self.out_var_exprs
                .push(Expression::new_linear(CircuitField::<C>::one(), start + i));
        }
        self.fix_mid_to_out(n);
    }

    /// Adds a linear combination to the output variables.
    pub fn add_lin_comb(&mut self, lcs: &LinComb<C>) {
        let mut vars: Vec<&Expression<C>> = lcs
            .terms
            .iter()
            .map(|lc| &self.out_var_exprs[lc.var])
            .collect();
        let cst = Expression::new_const(lcs.constant);
        if !lcs.constant.is_zero() {
            vars.push(&cst);
        }
        let expr = lin_comb_inner(
            vars,
            |i| {
                if i < lcs.terms.len() {
                    lcs.terms[i].coef
                } else {
                    CircuitField::<C>::one()
                }
            },
            &mut self.mid_vars,
        );
        self.out_var_exprs.push(expr);
        self.fix_mid_to_out(1);
    }

    /// Adds a multiplication of two output variables to the output variables.
    pub fn add_mul_vec(&mut self, mut vars: Vec<usize>) {
        assert!(vars.len() >= 2);
        vars.sort_by(|a, b| self.out_var_exprs[*a].cmp(&self.out_var_exprs[*b]));
        let mut cur = mul_two_expr(
            &self.out_var_exprs[vars[0]],
            &self.out_var_exprs[vars[1]],
            &mut self.mid_vars,
        );
        for var in vars.iter().skip(2) {
            cur = mul_two_expr(&cur, &self.out_var_exprs[*var], &mut self.mid_vars);
        }
        self.out_var_exprs.push(cur);
        self.fix_mid_to_out(1);
    }

    /// Adds a constant to the output variables.
    pub fn add_const(&mut self, c: CircuitField<C>) {
        self.out_var_exprs.push(Expression::new_const(c));
        self.fix_mid_to_out(1);
    }

    /// Adds an assertion to a constraint on an output variable.
    pub fn assert(
        &mut self,
        constraint_type: <IrcOut::Constraint as Constraint<C>>::Type,
        out_var_id: usize,
    ) {
        let expr = self.out_var_exprs[out_var_id].clone();
        self.constraints
            .entry(constraint_type)
            .or_default()
            .entry(expr)
            .or_insert(ConstraintStatus::Asserted);
    }

    /// Marks a constraint on an output variable as known to be true.
    pub fn mark(
        &mut self,
        constraint_type: <IrcOut::Constraint as Constraint<C>>::Type,
        out_var_id: usize,
    ) {
        let expr = self.out_var_exprs[out_var_id].clone();
        self.constraints
            .entry(constraint_type)
            .or_default()
            .insert(expr, ConstraintStatus::Marked);
    }

    /// Adds input variables to the builder.
    fn add_input(&mut self) {
        let n = self.in_circuit.get_num_inputs_all();
        self.add_out_vars(n);
        for i in 1..=n {
            self.in_to_out.push(i);
        }
    }

    /// Fixes the mapping from mid_vars to out_vars after adding new output variables.
    pub fn fix_mid_to_out(&mut self, n: usize) {
        for i in 1..=n {
            let id = self.out_var_exprs.len() - i;
            self.mid_expr_to_out
                .insert(self.out_var_exprs[id].clone(), id);
            let (eid, coef, constant) =
                to_single_stripped(&mut self.mid_vars, &self.out_var_exprs[id]);
            while self.mid_to_out.len() < self.mid_vars.len() {
                self.mid_to_out.push(None);
            }
            if eid == 0 {
                assert_eq!(n, 1);
                assert!(coef.is_zero());
                *self.out_insns.last_mut().unwrap() =
                    IrcOut::Instruction::from_kx_plus_b(0, CircuitField::<C>::zero(), constant);
                return;
            }
            match &self.mid_to_out[eid] {
                Some(ovr) => {
                    assert_eq!(n, 1);
                    let k = ovr.k * coef;
                    let b = ovr.b * coef + constant;
                    *self.out_insns.last_mut().unwrap() =
                        IrcOut::Instruction::from_kx_plus_b(ovr.x, k, b);
                }
                None => {
                    let coef_inv = coef.optimistic_inv().unwrap();
                    self.mid_to_out[eid] = Some(OutVarRef {
                        x: id,
                        k: coef_inv,
                        b: -coef_inv * constant,
                    });
                }
            }
        }
    }

    /// Prepare input variables and add output variables for a sub-circuit call.
    pub fn sub_circuit_call<'b>(
        &mut self,
        _sub_circuit_id: usize,
        inputs: &[usize],
        num_outputs: usize,
        _root: Option<&'b RootBuilder<'a, C, IrcIn, IrcOut>>,
    ) where
        'a: 'b,
    {
        // TODO: constraint propagation
        for &i in inputs.iter() {
            to_really_single(&mut self.mid_vars, &self.out_var_exprs[i]);
        }
        self.add_out_vars(num_outputs);
    }
}

impl<'a, C: Config, IrcIn: IrConfig<Config = C>, IrcOut: IrConfig<Config = C>>
    Builder<'a, C, IrcIn, IrcOut>
where
    Builder<'a, C, IrcIn, IrcOut>: InsnTransformAndExecute<'a, C, IrcIn, IrcOut>,
{
    /// Executes an output instruction, potentially using a root builder for additional context.
    pub fn push_insn_with_root<'b>(
        &mut self,
        out_insn: IrcOut::Instruction,
        root: Option<&'b RootBuilder<'a, C, IrcIn, IrcOut>>,
    ) -> Option<usize>
    where
        'a: 'b,
    {
        let num_out = out_insn.num_outputs();
        self.out_insns.push(out_insn.clone());
        self.execute_out(&out_insn, root);
        if num_out == 1 {
            Some(self.out_var_exprs.len() - 1)
        } else {
            None
        }
    }
    /// Pushes an output instruction to the builder and returns the ID of the output variable if it has one output.
    pub fn push_insn(&mut self, out_insn: IrcOut::Instruction) -> Option<usize> {
        self.push_insn_with_root(out_insn, None)
    }
    /// Pushes an output instruction with multiple outputs to the builder and returns the IDs of the output variables.
    pub fn push_insn_multi_out(&mut self, out_insn: IrcOut::Instruction) -> Vec<usize> {
        let num_out = out_insn.num_outputs();
        self.out_insns.push(out_insn.clone());
        self.execute_out(&out_insn, None);
        let mut out_var_ids = Vec::new();
        for i in 0..num_out {
            out_var_ids.push(self.out_var_exprs.len() - num_out + i);
        }
        out_var_ids
    }

    /// Processes an input instruction and transforms it to an output instruction.
    fn process_insn<'b>(
        &mut self,
        in_insn: &IrcIn::Instruction,
        root: &'b RootBuilder<'a, C, IrcIn, IrcOut>,
    ) -> Result<(), Error>
    where
        'a: 'b,
    {
        let in_mapped = in_insn.replace_vars(|x| self.in_to_out[x]);
        match self.transform_in_to_out(&in_mapped) {
            InsnTransformResult::Insn(out_insn) => {
                assert_eq!(out_insn.num_outputs(), in_insn.num_outputs());
                let start_id = self.out_var_exprs.len();
                self.push_insn_with_root(out_insn, Some(root));
                for i in 0..in_insn.num_outputs() {
                    self.in_to_out.push(start_id + i);
                }
            }
            InsnTransformResult::Vars(vars) => {
                assert_eq!(vars.len(), in_insn.num_outputs());
                self.in_to_out.extend(vars);
            }
            InsnTransformResult::Err(err) => return Err(err),
        }
        Ok(())
    }

    /// Processes an input constraint and transforms it to an output constraint.
    fn process_con(&mut self, in_con: &IrcIn::Constraint) -> Result<(), Error> {
        let in_mapped = in_con.replace_var(|x| self.in_to_out[x]);
        let out_con = self.transform_in_con_to_out(&in_mapped)?;
        self.assert(out_con.typ(), out_con.var());
        Ok(())
    }
}

/// Converts an expression to a single variable expression.
/// The result is guaranteed to be in the form `kx + b`.
fn to_single<C: Config>(mid_vars: &mut Pool<Expression<C>>, expr: &Expression<C>) -> Expression<C> {
    let (e, coef, constant) = strip_constants(expr);
    if e.len() == 1 && e.degree() <= 1 {
        return expr.clone();
    }
    let idx = mid_vars.add(&e);
    unstrip_constants_single(idx, coef, constant)
}

/// Converts an expression to a single variable expression.
/// The result is guaranteed to be in the form `kx + b`, and `x`, `k`, `b` are returned separately.
fn to_single_stripped<C: Config>(
    mid_vars: &mut Pool<Expression<C>>,
    expr: &Expression<C>,
) -> (usize, CircuitField<C>, CircuitField<C>) {
    let (e, coef, constant) = strip_constants(expr);
    if e.degree() == 0 {
        return (0, coef, constant);
    }
    let idx = mid_vars.add(&e);
    (idx, coef, constant)
}

/// Converts an expression to a single variable expression.
/// The result is guaranteed to be in the form `x`, where `x` is a variable ID. No constant term or coefficient is allowed.
pub fn to_really_single<C: Config>(
    mid_vars: &mut Pool<Expression<C>>,
    e: &Expression<C>,
) -> Expression<C> {
    if e.len() == 1 && e.degree() == 1 && e[0].coef == CircuitField::<C>::one() {
        return e.clone();
    }
    let idx = mid_vars.add(e);
    Expression::from_terms_sorted(vec![Term::new_linear(CircuitField::<C>::one(), idx)])
}

/// Tries to get a single variable ID from an expression.
/// If the expression is already registered as a single variable in `mid_vars`, it returns the ID.
pub fn try_get_really_single_id<C: Config>(
    mid_vars: &Pool<Expression<C>>,
    e: &Expression<C>,
) -> Option<usize> {
    if e.len() == 1 && e.degree() == 1 && e[0].coef == CircuitField::<C>::one() {
        let t: Vec<usize> = e.get_vars();
        return Some(t[0]);
    }
    mid_vars.try_get_idx(e)
}

/// Strips constants from an expression and returns the expression without constants,
/// the coefficient of the first term, and the constant term.
fn strip_constants<C: Config>(
    expr: &Expression<C>,
) -> (Expression<C>, CircuitField<C>, CircuitField<C>) {
    let mut e = Vec::new();
    let mut cst = CircuitField::<C>::zero();
    for term in expr.iter() {
        if term.vars == VarSpec::Const {
            cst = term.coef;
        } else {
            e.push(term.clone());
        }
    }
    if e.is_empty() {
        return (Expression::default(), CircuitField::<C>::zero(), cst);
    }
    let v = e[0].coef;
    let vi = v.optimistic_inv().unwrap();
    for term in e.iter_mut() {
        term.coef *= vi;
    }
    (Expression::from_terms_sorted(e), v, cst)
}

/// Unstrips constants from a single variable expression.
/// It takes a variable ID, a coefficient, and a constant term,
/// and returns an expression in the form `coef * var + constant`.
fn unstrip_constants_single<C: Config>(
    vid: usize,
    coef: CircuitField<C>,
    constant: CircuitField<C>,
) -> Expression<C> {
    assert_ne!(vid, 0);
    let mut e = vec![Term::new_linear(coef, vid)];
    if !constant.is_zero() {
        e.push(Term::new_const(constant));
    }
    Expression::from_terms(e)
}

/// Thresholds for compressing expressions in linear combinations and multiplications.
/// If an expression has more than `COMPRESS_THRESHOLD_ADD` terms, it will be compressed
/// to a single variable expression.
const COMPRESS_THRESHOLD_ADD: usize = 10;
const COMPRESS_THRESHOLD_MUL: usize = 40;

/// Computes a linear combination of multiple expressions.
///
/// This function computes `sum(var_coef(i) * vars[i])` for all `vars[i]`.
///
/// It maintains a heap of `LinMeta`, where each `LinMeta` is a pointer to a term in `vars[i]`.
///
/// In `LinMeta`, `l_id` is the index of the variable in `vars`, and `t_id` is the index of the term in that variable.
/// The `vars` field is the current variable's `VarSpec`, for comparing terms in the heap.
///
/// The function always takes the first term in the heap, and adds it to the result, until the heap is empty.
/// The result is guaranteed to be sorted by `VarSpec`.
fn lin_comb_inner<C: Config, F: Fn(usize) -> CircuitField<C>>(
    vars: Vec<&Expression<C>>,
    var_coef: F,
    mid_vars: &mut Pool<Expression<C>>,
) -> Expression<C> {
    if vars.is_empty() {
        return Expression::default();
    }
    if vars.len() == 1 {
        return vars[0].mul_constant(var_coef(0));
    }
    let mut heap: BinaryHeap<LinMeta> = BinaryHeap::new();
    let mut compressed_vars: Vec<Option<Expression<C>>> = vec![None; vars.len()];
    for l_id in 0..vars.len() {
        if var_coef(l_id).is_zero() {
            continue;
        }
        let v = vars[l_id];
        if v.len() > COMPRESS_THRESHOLD_ADD {
            let nv = to_single(mid_vars, v);
            heap.push(LinMeta {
                l_id: l_id + vars.len(),
                t_id: 0,
                vars: nv[0].vars.clone(),
            });
            compressed_vars[l_id] = Some(nv);
            continue;
        }
        heap.push(LinMeta {
            l_id,
            t_id: 0,
            vars: v[0].vars.clone(),
        });
    }
    let mut res: Vec<Term<C>> = Vec::new();
    while let Some(lm) = heap.peek() {
        let l_id = lm.l_id;
        let t_id = lm.t_id;
        let var = if l_id < vars.len() {
            vars[l_id]
        } else {
            compressed_vars[l_id - vars.len()].as_ref().unwrap()
        };
        if t_id == var.len() - 1 {
            heap.pop();
        } else {
            let mut lm = heap.peek_mut().unwrap();
            lm.t_id = t_id + 1;
            lm.vars = var[t_id + 1].vars.clone();
        }
        let t = &var[t_id];
        let new_coef = t.coef * var_coef(l_id % vars.len());
        if new_coef.is_zero() {
            continue;
        }
        if !res.is_empty() && res.last().unwrap().vars == t.vars {
            res.last_mut().unwrap().coef += new_coef;
            if res.last().unwrap().coef.is_zero() {
                res.pop();
            }
        } else {
            res.push(t.clone());
            res.last_mut().unwrap().coef = new_coef;
        }
    }
    if res.is_empty() {
        Expression::default()
    } else {
        Expression::from_terms_sorted(res)
    }
}

/// Multiplies two expressions and returns the result as a new expression.
fn mul_two_expr<C: Config>(
    a: &Expression<C>,
    b: &Expression<C>,
    mid_vars: &mut Pool<Expression<C>>,
) -> Expression<C> {
    let a_deg = a.degree();
    let b_deg = b.degree();
    if a_deg == 0 {
        return b.mul_constant(a.constant_value().unwrap());
    }
    if b_deg == 0 {
        return a.mul_constant(b.constant_value().unwrap());
    }
    let mut _v1_st = None;
    let mut _v2_st = None;
    let mut v1 = if a_deg == 2 {
        _v1_st = Some(to_single(mid_vars, a));
        _v1_st.as_ref().unwrap()
    } else {
        a
    };
    let mut v2 = if b_deg == 2 {
        _v2_st = Some(to_single(mid_vars, b));
        _v2_st.as_ref().unwrap()
    } else {
        b
    };
    if v1.len() > v2.len() {
        std::mem::swap(&mut v1, &mut v2);
    }
    // force compression to avoid too large expressions
    let mut _v1_st2 = None;
    let mut _v2_st2 = None;
    if v1.len() > COMPRESS_THRESHOLD_MUL {
        _v1_st2 = Some(to_single(mid_vars, v1));
        _v2_st2 = Some(to_single(mid_vars, v2));
        v1 = _v1_st2.as_ref().unwrap();
        v2 = _v2_st2.as_ref().unwrap();
    } else if v1.len() * v2.len() > COMPRESS_THRESHOLD_MUL {
        _v2_st2 = Some(to_single(mid_vars, v2));
        v2 = _v2_st2.as_ref().unwrap();
    }
    let mut vars: Vec<Expression<C>> = Vec::new();
    for x1 in v1.iter() {
        vars.push(Expression::from_terms(
            v2.iter().map(|x2| x1.mul(x2)).collect(),
        ));
    }
    lin_comb_inner(
        vars.iter().collect(),
        |_| CircuitField::<C>::one(),
        mid_vars,
    )
}

/// Type alias for the result of processing a circuit.
pub type ProcessOk<'a, C, IrcIn, IrcOut> =
    (ir::common::Circuit<IrcOut>, Builder<'a, C, IrcIn, IrcOut>);

/// Processes a circuit and returns the transformed circuit along with the builder.
pub fn process_circuit<
    'b,
    'a: 'b,
    C: Config,
    IrcIn: IrConfig<Config = C>,
    IrcOut: IrConfig<Config = C>,
>(
    root: &'b mut RootBuilder<'a, C, IrcIn, IrcOut>,
    circuit_id: usize,
    circuit: &'a ir::common::Circuit<IrcIn>,
) -> Result<ProcessOk<'a, C, IrcIn, IrcOut>, Error>
where
    Builder<'a, C, IrcIn, IrcOut>: InsnTransformAndExecute<'a, C, IrcIn, IrcOut>,
{
    //let circuit = root.rc.circuits.get(&circuit_id).unwrap();
    let mut builder = Builder::new(circuit_id, circuit);
    builder.add_input();
    for insn in circuit.instructions.iter() {
        builder.process_insn(insn, root)?;
    }
    for con in circuit.constraints.iter() {
        builder.process_con(con)?;
    }
    let mut constraints = Vec::new();
    for (typ, cons) in builder.constraints.iter() {
        let mut cons_keys = cons.keys().collect::<Vec<_>>();
        cons_keys.sort();
        for expr in cons_keys {
            let status = cons.get(expr).unwrap();
            match status {
                ConstraintStatus::Marked => {}
                ConstraintStatus::Asserted => {
                    constraints.push(Constraint::new(builder.mid_expr_to_out[expr], *typ));
                }
            }
        }
    }
    let new_circuit = ir::common::Circuit {
        instructions: builder.out_insns.clone(),
        constraints,
        outputs: circuit
            .outputs
            .iter()
            .map(|x| builder.in_to_out[*x])
            .collect(),
        num_inputs: circuit.num_inputs,
    };
    Ok((new_circuit, builder))
}

/// Processes the root circuit and returns a new root circuit with transformed instructions and constraints.
pub fn process_root_circuit<
    'a,
    C: Config + 'a,
    IrcIn: IrConfig<Config = C> + 'a,
    IrcOut: IrConfig<Config = C> + 'a,
>(
    rc: &'a ir::common::RootCircuit<IrcIn>,
) -> Result<ir::common::RootCircuit<IrcOut>, Error>
where
    Builder<'a, C, IrcIn, IrcOut>: InsnTransformAndExecute<'a, C, IrcIn, IrcOut>,
{
    let mut root: RootBuilder<'a, C, IrcIn, IrcOut> = RootBuilder {
        builders: HashMap::new(),
        rc,
        out_circuits: HashMap::new(),
    };
    let order = rc.topo_order();
    for &circuit_id in order.iter().rev() {
        let (new_circuit, final_builder) =
            process_circuit(&mut root, circuit_id, rc.circuits.get(&circuit_id).unwrap())?;
        root.out_circuits.insert(circuit_id, new_circuit);
        root.builders.insert(circuit_id, final_builder);
    }
    Ok(ir::common::RootCircuit {
        num_public_inputs: rc.num_public_inputs,
        expected_num_output_zeroes: rc.expected_num_output_zeroes,
        circuits: root.out_circuits,
    })
}
