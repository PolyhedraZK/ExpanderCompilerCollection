pub use super::environment_utils::slice_types::ArithmeticExpression;
use super::execute::RuntimeInformation;
use super::execution_data::ExecutedTemplate;
use num_bigint::BigInt;
use program_structure::ast::{ExpressionInfixOpcode, ExpressionPrefixOpcode};
pub use program_structure::utils::memory_slice::MemorySlice;
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    hash::Hash,
};

pub type RawAExpr = ArithmeticExpression<String>;

#[derive(Clone)]
pub struct AExpr {
    pub aexpr: RawAExpr,
    pub trace_identifier: usize,
}

pub type AExpressionSlice = MemorySlice<AExpr>;

impl Default for AExpr {
    fn default() -> Self {
        AExpr {
            aexpr: ArithmeticExpression::default(),
            trace_identifier: TRACE_UNKNOWN,
        }
    }
}

impl Eq for AExpr {}
impl PartialEq for AExpr {
    fn eq(&self, other: &Self) -> bool {
        self.aexpr == other.aexpr
    }
}
impl Display for AExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{}", self.aexpr).as_str())
    }
}

impl AExpr {
    pub fn from_raw(aexpr: RawAExpr) -> Self {
        AExpr {
            aexpr: aexpr.clone(),
            trace_identifier: TRACE_UNKNOWN,
        }
    }
    pub fn unknown() -> Self {
        AExpr {
            aexpr: RawAExpr::NonQuadratic,
            trace_identifier: TRACE_UNKNOWN,
        }
    }
    pub fn from_number(num: &BigInt, runtime: &mut RuntimeInformation) -> Self {
        let a = RawAExpr::Number { value: num.clone() };
        AExpr {
            aexpr: a,
            trace_identifier: runtime
                .trace_registry
                .index(&TraceItem::Number { value: num.clone() }),
        }
    }
    pub fn from_number_registry(num: &BigInt, trace_registry: &mut TraceRegistry) -> Self {
        let a = RawAExpr::Number { value: num.clone() };
        AExpr {
            aexpr: a,
            trace_identifier: trace_registry.index(&TraceItem::Number { value: num.clone() }),
        }
    }
    pub fn from_signal_registry(
        symbol: &String,
        trace_registry: &mut TraceRegistry,
        actual_node: &mut Option<ExecutedTemplate>,
    ) -> Self {
        let a = RawAExpr::Signal {
            symbol: symbol.clone(),
        };
        if let Some(node) = actual_node {
            if let Some(old_id) = node.final_signal_traces.get(symbol) {
                return AExpr {
                    aexpr: a,
                    trace_identifier: *old_id,
                };
            }
        }
        AExpr {
            aexpr: a,
            trace_identifier: trace_registry.index(&TraceItem::Signal {
                symbol: symbol.clone(),
            }),
        }
    }
    pub fn from_infix(
        a: RawAExpr,
        l_id: usize,
        r_id: usize,
        op: ExpressionInfixOpcode,
        runtime: &mut RuntimeInformation,
    ) -> Self {
        AExpr {
            aexpr: a,
            trace_identifier: runtime.trace_registry.index(&TraceItem::InfixOp {
                l_id: l_id,
                r_id: r_id,
                op: to_self_infix_opcode(op),
            }),
        }
    }
    pub fn from_prefix(
        a: RawAExpr,
        id: usize,
        op: ExpressionPrefixOpcode,
        runtime: &mut RuntimeInformation,
    ) -> Self {
        AExpr {
            aexpr: a,
            trace_identifier: runtime.trace_registry.index(&TraceItem::PrefixOp {
                id: id,
                op: to_self_prefix_opcode(op),
            }),
        }
    }
    pub fn from_inline_switch(
        cond: usize,
        if_true: usize,
        if_false: usize,
        runtime: &mut RuntimeInformation,
    ) -> Self {
        let a = RawAExpr::NonQuadratic;
        AExpr {
            aexpr: a,
            trace_identifier: runtime.trace_registry.index(&TraceItem::InlineSwitch {
                cond: cond,
                if_true: if_true,
                if_false: if_false,
            }),
        }
    }
}

#[derive(Default, Clone)]
pub struct TraceRegistry {
    pub map: HashMap<TraceItem, usize>,
    pub vec: Vec<TraceItem>,
}

#[derive(Copy, Clone, PartialEq, Hash, Eq, Debug)]
pub enum InfixOpcode {
    Mul,
    Div,
    Add,
    Sub,
    Pow,
    IntDiv,
    Mod,
    ShiftL,
    ShiftR,
    LesserEq,
    GreaterEq,
    Lesser,
    Greater,
    Eq,
    NotEq,
    BoolOr,
    BoolAnd,
    BitOr,
    BitAnd,
    BitXor,
}

#[derive(Copy, Clone, PartialEq, Hash, Eq, Debug)]
pub enum PrefixOpcode {
    Sub,
    BoolNot,
    Complement,
}

fn to_self_infix_opcode(op: ExpressionInfixOpcode) -> InfixOpcode {
    use ExpressionInfixOpcode::*;
    match op {
        Mul => InfixOpcode::Mul,
        Div => InfixOpcode::Div,
        Add => InfixOpcode::Add,
        Sub => InfixOpcode::Sub,
        Pow => InfixOpcode::Pow,
        IntDiv => InfixOpcode::IntDiv,
        Mod => InfixOpcode::Mod,
        ShiftL => InfixOpcode::ShiftL,
        ShiftR => InfixOpcode::ShiftR,
        LesserEq => InfixOpcode::LesserEq,
        GreaterEq => InfixOpcode::GreaterEq,
        Lesser => InfixOpcode::Lesser,
        Greater => InfixOpcode::Greater,
        Eq => InfixOpcode::Eq,
        NotEq => InfixOpcode::NotEq,
        BoolOr => InfixOpcode::BoolOr,
        BoolAnd => InfixOpcode::BoolAnd,
        BitOr => InfixOpcode::BitOr,
        BitAnd => InfixOpcode::BitAnd,
        BitXor => InfixOpcode::BitXor,
    }
}

fn to_self_prefix_opcode(op: ExpressionPrefixOpcode) -> PrefixOpcode {
    use ExpressionPrefixOpcode::*;
    match op {
        Sub => PrefixOpcode::Sub,
        BoolNot => PrefixOpcode::BoolNot,
        Complement => PrefixOpcode::Complement,
    }
}

#[derive(Hash, PartialEq, Clone)]
pub enum TraceItem {
    Number {
        value: BigInt,
    },
    Signal {
        symbol: String,
    },
    InfixOp {
        l_id: usize,
        r_id: usize,
        op: InfixOpcode,
    },
    PrefixOp {
        id: usize,
        op: PrefixOpcode,
    },
    InlineSwitch {
        cond: usize,
        if_true: usize,
        if_false: usize,
    },
    Unknown,
}

impl Eq for TraceItem {}

pub const TRACE_UNKNOWN: usize = 0;

impl TraceRegistry {
    pub fn new() -> Self {
        let mut r = TraceRegistry {
            map: HashMap::new(),
            vec: Vec::new(),
        };
        r.index(&TraceItem::Unknown);
        return r;
    }
    pub fn index(&mut self, x: &TraceItem) -> usize {
        if let Some(id) = self.map.get(x) {
            *id
        } else {
            let r = self.map.len();
            self.map.insert(x.clone(), r);
            self.vec.push(x.clone());
            r
        }
    }
}
