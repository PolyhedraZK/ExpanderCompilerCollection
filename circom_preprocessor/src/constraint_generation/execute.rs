use super::environment_utils::{
    environment::{
        environment_check_all_components_assigned, environment_shortcut_add_component,
        environment_shortcut_add_input, environment_shortcut_add_intermediate,
        environment_shortcut_add_output, environment_shortcut_add_variable, ExecutionEnvironment,
        ExecutionEnvironmentError,
    },
    slice_types::{
        AExpressionSlice, ArithmeticExpression as ArithmeticExpressionGen, ComponentRepresentation,
        ComponentSlice, MemoryError, MemorySlice, SignalSlice, SliceCapacity, TagInfo, TagState,
        TypeAssignmentError, TypeInvalidAccess,
    },
};

use program_structure::constants::UsefulConstants;

use super::execution_data::analysis::Analysis;
use super::execution_data::{ExecutedProgram, ExecutedTemplate, NodePointer, PreExecutedTemplate};
use super::{
    ast::*, ArithmeticError, FileID, ProgramArchive, Report, ReportCode, ReportCollection,
};
use crate::constraint_generation::{
    expression_trace::{InfixOpcode, TraceItem, TraceRegistry},
    FlagsExecution,
};
use circom_algebra::num_bigint::BigInt;
use std::collections::{BTreeMap, HashMap};
use std::mem;
type AnonymousComponentsInfo = BTreeMap<String, (Meta, Vec<Expression>)>;

use super::expression_trace::AExpr as WrappedAExpr;
type AExpr = ArithmeticExpressionGen<String>;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
enum BlockType {
    Known,
    Unknown,
}

pub struct RuntimeInformation {
    pub block_type: BlockType,
    pub analysis: Analysis,
    pub public_inputs: Vec<String>,
    pub constants: UsefulConstants,
    pub call_trace: Vec<String>,
    pub current_file: FileID,
    pub runtime_errors: ReportCollection,
    pub environment: ExecutionEnvironment,
    pub exec_program: ExecutedProgram,
    pub anonymous_components: AnonymousComponentsInfo,

    pub trace_registry: TraceRegistry,
}
impl RuntimeInformation {
    pub fn new(current_file: FileID, id_max: usize, prime: &String) -> RuntimeInformation {
        RuntimeInformation {
            current_file,
            block_type: BlockType::Known,
            analysis: Analysis::new(id_max),
            public_inputs: vec![],
            constants: UsefulConstants::new(prime),
            call_trace: Vec::new(),
            runtime_errors: ReportCollection::new(),
            environment: ExecutionEnvironment::new(),
            exec_program: ExecutedProgram::new(prime),
            anonymous_components: AnonymousComponentsInfo::new(),
            trace_registry: TraceRegistry::new(),
        }
    }
}

struct FoldedValue {
    pub arithmetic_slice: Option<AExpressionSlice>,
    pub node_pointer: Option<NodePointer>,
    pub is_parallel: Option<bool>,
    pub tags: Option<TagInfo>,
}
impl FoldedValue {
    pub fn valid_arithmetic_slice(f_value: &FoldedValue) -> bool {
        f_value.arithmetic_slice.is_some()
            && f_value.node_pointer.is_none()
            && f_value.is_parallel.is_none()
    }
    pub fn valid_node_pointer(f_value: &FoldedValue) -> bool {
        f_value.node_pointer.is_some()
            && f_value.is_parallel.is_some()
            && f_value.arithmetic_slice.is_none()
    }
}

impl Default for FoldedValue {
    fn default() -> Self {
        FoldedValue {
            arithmetic_slice: Option::None,
            node_pointer: Option::None,
            is_parallel: Option::None,
            tags: Option::None,
        }
    }
}

enum ExecutionError {
    NonQuadraticConstraint,
    ConstraintInUnknown,
    DeclarationInUnknown,
    TagAssignmentInUnknown,
    UnknownTemplate,
    NonValidTagAssignment,
    FalseAssert,
    ArraySizeTooBig,
}

enum ExecutionWarning {
    CanBeQuadraticConstraintSingle(),
    CanBeQuadraticConstraintMultiple(Vec<String>),
}

pub fn constraint_execution(
    program_archive: &ProgramArchive,
    flags: FlagsExecution,
    prime: &String,
) -> Result<(ExecutedProgram, ReportCollection), ReportCollection> {
    let main_file_id = program_archive.get_file_id_main();
    let mut runtime_information =
        RuntimeInformation::new(*main_file_id, program_archive.id_max, prime);
    use Expression::Call;

    runtime_information.public_inputs = program_archive.get_public_inputs_main_component().clone();

    let folded_value_result = if let Call { id, args, .. } = &program_archive.get_main_expression()
    {
        let mut arg_values = Vec::new();
        for arg_expression in args.iter() {
            let f_arg = execute_expression(
                arg_expression,
                program_archive,
                &mut runtime_information,
                &mut None,
                flags,
            );
            arg_values.push(safe_unwrap_to_arithmetic_slice(f_arg.unwrap(), line!()));
            // improve
        }
        execute_template_call_complete(
            id,
            arg_values,
            BTreeMap::new(),
            program_archive,
            &mut runtime_information,
            flags,
        )
    } else {
        unreachable!("The main expression should be a call.");
    };

    match folded_value_result {
        Result::Err(_) => Result::Err(runtime_information.runtime_errors),
        Result::Ok(folded_value) => {
            debug_assert!(FoldedValue::valid_node_pointer(&folded_value));
            runtime_information.exec_program.trace_registry = runtime_information.trace_registry;
            Result::Ok((
                runtime_information.exec_program,
                runtime_information.runtime_errors,
            ))
        }
    }
}

pub fn execute_constant_expression(
    expression: &Expression,
    program_archive: &ProgramArchive,
    environment: ExecutionEnvironment,
    flags: FlagsExecution,
    prime: &String,
) -> Result<BigInt, ReportCollection> {
    let current_file = expression.get_meta().get_file_id();
    let mut runtime_information =
        RuntimeInformation::new(current_file, program_archive.id_max, prime);
    runtime_information.environment = environment;
    let folded_value_result = execute_expression(
        expression,
        program_archive,
        &mut runtime_information,
        &mut None,
        flags,
    );
    match folded_value_result {
        Result::Err(_) => Result::Err(runtime_information.runtime_errors),
        Result::Ok(folded_value) => {
            debug_assert!(FoldedValue::valid_arithmetic_slice(&folded_value));
            let value = safe_unwrap_to_single_arithmetic_expression(folded_value, line!());
            if let AExpr::Number { value } = value.aexpr {
                Result::Ok(value)
            } else {
                unreachable!();
            }
        }
    }
}

// returns the value and if it can be simplified
fn execute_statement(
    stmt: &Statement,
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    actual_node: &mut Option<ExecutedTemplate>,
    flags: FlagsExecution,
) -> Result<(Option<FoldedValue>, bool), ()> {
    use Statement::*;
    let id = stmt.get_meta().elem_id;
    Analysis::reached(&mut runtime.analysis, id);
    let mut can_be_simplified = true;
    let res = match stmt {
        MultSubstitution { .. } => unreachable!(),
        InitializationBlock {
            initializations, ..
        } => {
            let (possible_fold, _) = execute_sequence_of_statements(
                initializations,
                program_archive,
                runtime,
                actual_node,
                flags,
                false,
            )?;
            debug_assert!(possible_fold.is_none());
            possible_fold
        }
        Declaration {
            meta,
            xtype,
            name,
            dimensions,
            ..
        } => {
            match xtype {
                VariableType::AnonymousComponent => {
                    if runtime.block_type == BlockType::Unknown {
                        // Case not valid constraint Known/Unknown
                        let err = Result::Err(ExecutionError::DeclarationInUnknown);
                        treat_result_with_execution_error(
                            err,
                            meta,
                            &mut runtime.runtime_errors,
                            &runtime.call_trace,
                        )?;
                    }
                    execute_anonymous_component_declaration(
                        name,
                        meta.clone(),
                        &dimensions,
                        &mut runtime.environment,
                        &mut runtime.anonymous_components,
                    );
                }
                _ => {
                    let mut arithmetic_values = Vec::new();
                    for dimension in dimensions.iter() {
                        let f_dimensions = execute_expression(
                            dimension,
                            program_archive,
                            runtime,
                            actual_node,
                            flags,
                        )?;
                        arithmetic_values.push(
                            safe_unwrap_to_single_arithmetic_expression(f_dimensions, line!())
                                .aexpr,
                        );
                    }
                    treat_result_with_memory_error_void(
                        valid_array_declaration(&arithmetic_values),
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?;
                    let usable_dimensions =
                        if let Option::Some(dimensions) = cast_indexing(&arithmetic_values) {
                            dimensions
                        } else {
                            let err = Result::Err(ExecutionError::ArraySizeTooBig);
                            treat_result_with_execution_error(
                                err,
                                meta,
                                &mut runtime.runtime_errors,
                                &runtime.call_trace,
                            )?
                        };
                    match xtype {
                        VariableType::Component => {
                            if runtime.block_type == BlockType::Unknown {
                                // Case not valid constraint Known/Unknown
                                let err = Result::Err(ExecutionError::DeclarationInUnknown);
                                treat_result_with_execution_error(
                                    err,
                                    meta,
                                    &mut runtime.runtime_errors,
                                    &runtime.call_trace,
                                )?;
                            }
                            execute_component_declaration(
                                name,
                                &usable_dimensions,
                                &mut runtime.environment,
                                actual_node,
                            )
                        }
                        VariableType::Var => environment_shortcut_add_variable(
                            &mut runtime.environment,
                            name,
                            &usable_dimensions,
                        ),
                        VariableType::Signal(signal_type, tag_list) => {
                            if runtime.block_type == BlockType::Unknown {
                                // Case not valid constraint Known/Unknown
                                let err = Result::Err(ExecutionError::DeclarationInUnknown);
                                treat_result_with_execution_error(
                                    err,
                                    meta,
                                    &mut runtime.runtime_errors,
                                    &runtime.call_trace,
                                )?;
                            }
                            execute_signal_declaration(
                                name,
                                &usable_dimensions,
                                tag_list,
                                *signal_type,
                                &mut runtime.environment,
                                actual_node,
                            )
                        }
                        _ => {
                            unreachable!()
                        }
                    }
                }
            }
            Option::None
        }
        Substitution {
            meta,
            var,
            access,
            op,
            rhe,
            ..
        } => {
            let access_information =
                treat_accessing(meta, access, program_archive, runtime, actual_node, flags)?;
            let r_folded = execute_expression(rhe, program_archive, runtime, actual_node, flags)?;
            let possible_constraint = perform_assign(
                meta,
                var,
                *op,
                &access_information,
                r_folded,
                actual_node,
                runtime,
                program_archive,
                flags,
            )?;
            if let Option::Some(node) = actual_node {
                if *op == AssignOp::AssignConstraintSignal
                    || (*op == AssignOp::AssignSignal && flags.inspect)
                {
                    debug_assert!(possible_constraint.is_some());

                    if *op == AssignOp::AssignConstraintSignal
                        && runtime.block_type == BlockType::Unknown
                    {
                        // Case not valid constraint Known/Unknown
                        let err = Result::Err(ExecutionError::ConstraintInUnknown);
                        treat_result_with_execution_error(
                            err,
                            meta,
                            &mut runtime.runtime_errors,
                            &runtime.call_trace,
                        )?;
                    }

                    let constrained = possible_constraint.unwrap();

                    let mut needs_double_arrow = Vec::new();
                    for i in 0..AExpressionSlice::get_number_of_cells(&constrained.right) {
                        let value_right = treat_result_with_memory_error(
                            AExpressionSlice::access_value_by_index(&constrained.right, i),
                            meta,
                            &mut runtime.runtime_errors,
                            &runtime.call_trace,
                        )?;

                        let access_left = treat_result_with_memory_error(
                            AExpressionSlice::get_access_index(&constrained.right, i),
                            meta,
                            &mut runtime.runtime_errors,
                            &runtime.call_trace,
                        )?;

                        let full_symbol = format!(
                            "{}{}",
                            constrained.left,
                            create_index_appendix(&access_left),
                        );
                        if let AssignOp::AssignConstraintSignal = op {
                            if value_right.aexpr.is_nonquadratic() {
                                let err = Result::Err(ExecutionError::NonQuadraticConstraint);
                                treat_result_with_execution_error(
                                    err,
                                    meta,
                                    &mut runtime.runtime_errors,
                                    &runtime.call_trace,
                                )?;
                            } else {
                                let p = runtime.constants.get_p().clone();
                                let symbol = AExpr::Signal {
                                    symbol: full_symbol,
                                };
                                let expr = AExpr::sub(&symbol, &value_right.aexpr, &p);
                                let ctr = AExpr::transform_expression_to_constraint_form(expr, &p)
                                    .unwrap();
                                //println!("constraint by assign: {:?}", ctr);
                                node.add_constraint(ctr);
                                // no need to add trace_constraint
                            }
                        } else if let AssignOp::AssignSignal = op {
                            // needs fix, check case arrays
                            //debug_assert!(possible_constraint.is_some());
                            if !value_right.aexpr.is_nonquadratic() && !node.is_custom_gate {
                                needs_double_arrow.push(full_symbol);
                            }
                        }
                    }

                    if !needs_double_arrow.is_empty() && flags.inspect {
                        // in case we can subsitute the complete expression to ==>
                        if needs_double_arrow.len()
                            == AExpressionSlice::get_number_of_cells(&constrained.right)
                        {
                            let err: Result<(), ExecutionWarning> =
                                Result::Err(ExecutionWarning::CanBeQuadraticConstraintSingle());

                            treat_result_with_execution_warning(
                                err,
                                meta,
                                &mut runtime.runtime_errors,
                                &runtime.call_trace,
                            )?;
                        } else {
                            let err: Result<(), ExecutionWarning> =
                                Result::Err(ExecutionWarning::CanBeQuadraticConstraintMultiple(
                                    needs_double_arrow,
                                ));

                            treat_result_with_execution_warning(
                                err,
                                meta,
                                &mut runtime.runtime_errors,
                                &runtime.call_trace,
                            )?;
                        }
                    }
                }
            }
            Option::None
        }
        ConstraintEquality { meta, lhe, rhe, .. } => {
            debug_assert!(actual_node.is_some());

            if runtime.block_type == BlockType::Unknown {
                // Case not valid constraint Known/Unknown
                let err = Result::Err(ExecutionError::ConstraintInUnknown);
                treat_result_with_execution_error(
                    err,
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?;
            }

            let f_left = execute_expression(lhe, program_archive, runtime, actual_node, flags)?;
            let f_right = execute_expression(rhe, program_archive, runtime, actual_node, flags)?;
            let arith_left = safe_unwrap_to_arithmetic_slice(f_left, line!());
            let arith_right = safe_unwrap_to_arithmetic_slice(f_right, line!());

            let correct_dims_result =
                AExpressionSlice::check_correct_dims(&arith_left, &Vec::new(), &arith_right, true);
            treat_result_with_memory_error_void(
                correct_dims_result,
                meta,
                &mut runtime.runtime_errors,
                &runtime.call_trace,
            )?;
            for i in 0..AExpressionSlice::get_number_of_cells(&arith_left) {
                let value_left = treat_result_with_memory_error(
                    AExpressionSlice::access_value_by_index(&arith_left, i),
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?;
                let value_right = treat_result_with_memory_error(
                    AExpressionSlice::access_value_by_index(&arith_right, i),
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?;
                let possible_non_quadratic = AExpr::sub(
                    &value_left.aexpr,
                    &value_right.aexpr,
                    &runtime.constants.get_p(),
                );
                if possible_non_quadratic.is_nonquadratic() {
                    treat_result_with_execution_error(
                        Result::Err(ExecutionError::NonQuadraticConstraint),
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?;
                }
                let quadratic_expression = possible_non_quadratic;
                let constraint_expression = AExpr::transform_expression_to_constraint_form(
                    quadratic_expression,
                    runtime.constants.get_p(),
                )
                .unwrap();
                if let Option::Some(node) = actual_node {
                    //println!("direct constraint: {:?}", constraint_expression);
                    node.add_constraint(constraint_expression);
                    let trace_constraint = runtime.trace_registry.index(&TraceItem::InfixOp {
                        l_id: value_left.trace_identifier,
                        r_id: value_right.trace_identifier,
                        op: InfixOpcode::Sub,
                    });
                    node.add_trace_constraint(trace_constraint);
                }
            }
            Option::None
        }
        Return { value, .. } => {
            let mut f_return =
                execute_expression(value, program_archive, runtime, actual_node, flags)?;
            if let Option::Some(slice) = &mut f_return.arithmetic_slice {
                if runtime.block_type == BlockType::Unknown {
                    *slice =
                        AExpressionSlice::new_with_route(slice.route(), &WrappedAExpr::unknown());
                }
            }
            debug_assert!(FoldedValue::valid_arithmetic_slice(&f_return));
            Option::Some(f_return)
        }
        IfThenElse {
            cond,
            if_case,
            else_case,
            ..
        } => {
            let else_case = else_case.as_ref().map(|e| e.as_ref());
            let (possible_return, can_simplify, _) = execute_conditional_statement(
                cond,
                if_case,
                else_case,
                program_archive,
                runtime,
                actual_node,
                flags,
            )?;
            can_be_simplified = can_simplify;
            possible_return
        }
        While { cond, stmt, .. } => loop {
            let (returned, can_simplify, condition_result) = execute_conditional_statement(
                cond,
                stmt,
                Option::None,
                program_archive,
                runtime,
                actual_node,
                flags,
            )?;
            can_be_simplified &= can_simplify;
            if returned.is_some() {
                break returned;
            } else if condition_result.is_none() {
                let (returned, _, _) = execute_conditional_statement(
                    cond,
                    stmt,
                    None,
                    program_archive,
                    runtime,
                    actual_node,
                    flags,
                )?;
                break returned;
            } else if !condition_result.unwrap() {
                break returned;
            }
        },
        Block { stmts, .. } => {
            ExecutionEnvironment::add_variable_block(&mut runtime.environment);
            let (return_value, can_simplify_block) = execute_sequence_of_statements(
                stmts,
                program_archive,
                runtime,
                actual_node,
                flags,
                false,
            )?;
            ExecutionEnvironment::remove_variable_block(&mut runtime.environment);
            can_be_simplified = can_simplify_block;
            return_value
        }
        LogCall { args, .. } => {
            can_be_simplified = false;
            if flags.verbose {
                let mut index = 0;
                for arglog in args {
                    if let LogArgument::LogExp(arg) = arglog {
                        let f_result =
                            execute_expression(arg, program_archive, runtime, actual_node, flags)?;
                        let arith =
                            safe_unwrap_to_single_arithmetic_expression(f_result, line!()).aexpr;
                        if AExpr::is_number(&arith) {
                            print!("{}", arith);
                        } else {
                            print!("Unknown")
                        }
                    } else if let LogArgument::LogStr(s) = arglog {
                        print!("{}", s);
                    }
                    if index != args.len() - 1 {
                        print!(" ");
                    }
                    index += 1;
                }
                println!("");
            } else {
                for arglog in args {
                    if let LogArgument::LogExp(arg) = arglog {
                        let f_result =
                            execute_expression(arg, program_archive, runtime, actual_node, flags)?;
                        let _arith = safe_unwrap_to_single_arithmetic_expression(f_result, line!());
                    }
                }
            }
            Option::None
        }
        Assert { arg, meta, .. } => {
            let f_result = execute_expression(arg, program_archive, runtime, actual_node, flags)?;
            let arith = safe_unwrap_to_single_arithmetic_expression(f_result, line!()).aexpr;
            let possible_bool = AExpr::get_boolean_equivalence(&arith, runtime.constants.get_p());
            let result = match possible_bool {
                Some(b) if !b => Err(ExecutionError::FalseAssert),
                Some(b) if b => Ok(None),
                _ => {
                    can_be_simplified = false;
                    Ok(None)
                }
            };
            treat_result_with_execution_error(
                result,
                meta,
                &mut runtime.runtime_errors,
                &runtime.call_trace,
            )?
        }
        UnderscoreSubstitution { meta, rhe, op } => {
            let f_result = execute_expression(rhe, program_archive, runtime, actual_node, flags)?;
            let arithmetic_slice = safe_unwrap_to_arithmetic_slice(f_result, line!());
            if *op == AssignOp::AssignConstraintSignal {
                for i in 0..AExpressionSlice::get_number_of_cells(&arithmetic_slice) {
                    let value_cell = treat_result_with_memory_error(
                        AExpressionSlice::access_value_by_index(&arithmetic_slice, i),
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?;
                    let constraint_expression = AExpr::transform_expression_to_constraint_form(
                        value_cell.aexpr,
                        runtime.constants.get_p(),
                    )
                    .unwrap();
                    if let Option::Some(node) = actual_node {
                        for signal in constraint_expression.take_signals() {
                            node.add_underscored_signal(signal);
                        }
                    }
                }
            }
            Option::None
        }
    };
    Result::Ok((res, can_be_simplified))
}

fn execute_expression(
    expr: &Expression,
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    actual_node: &mut Option<ExecutedTemplate>,
    flags: FlagsExecution,
) -> Result<FoldedValue, ()> {
    use Expression::*;
    let mut can_be_simplified = true;
    let res = match expr {
        Number(_, value) => {
            let ae_slice = AExpressionSlice::new(&WrappedAExpr::from_number(value, runtime));
            FoldedValue {
                arithmetic_slice: Option::Some(ae_slice),
                ..FoldedValue::default()
            }
        }
        Variable {
            meta, name, access, ..
        } => {
            if ExecutionEnvironment::has_signal(&runtime.environment, name) {
                execute_signal(
                    meta,
                    name,
                    access,
                    program_archive,
                    runtime,
                    actual_node,
                    flags,
                )?
            } else if ExecutionEnvironment::has_component(&runtime.environment, name) {
                execute_component(
                    meta,
                    name,
                    access,
                    program_archive,
                    runtime,
                    actual_node,
                    flags,
                )?
            } else if ExecutionEnvironment::has_variable(&runtime.environment, name) {
                execute_variable(
                    meta,
                    name,
                    access,
                    program_archive,
                    runtime,
                    actual_node,
                    flags,
                )?
            } else {
                unreachable!();
            }
        }
        ArrayInLine { meta, values, .. } => {
            let mut arithmetic_slice_array = Vec::new();
            for value in values.iter() {
                let f_value =
                    execute_expression(value, program_archive, runtime, actual_node, flags)?;
                let slice_value = safe_unwrap_to_arithmetic_slice(f_value, line!());
                arithmetic_slice_array.push(slice_value);
            }
            debug_assert!(!arithmetic_slice_array.is_empty());

            let mut dims = vec![values.len()];
            for dim in arithmetic_slice_array[0].route() {
                dims.push(*dim);
            }
            let mut array_slice = AExpressionSlice::new_with_route(&dims, &WrappedAExpr::unknown());
            let mut row: SliceCapacity = 0;
            while row < arithmetic_slice_array.len() {
                let memory_insert_result = AExpressionSlice::insert_values(
                    &mut array_slice,
                    &[row],
                    &arithmetic_slice_array[row],
                    false,
                );
                treat_result_with_memory_error_void(
                    memory_insert_result,
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?;
                row += 1;
            }
            FoldedValue {
                arithmetic_slice: Option::Some(array_slice),
                ..FoldedValue::default()
            }
        }
        UniformArray {
            meta,
            value,
            dimension,
            ..
        } => {
            let f_dimension =
                execute_expression(dimension, program_archive, runtime, actual_node, flags)?;
            let arithmetic_dimension =
                safe_unwrap_to_single_arithmetic_expression(f_dimension, line!()).aexpr;
            let usable_dimension =
                if let Option::Some(dimension) = cast_index(&arithmetic_dimension) {
                    dimension
                } else {
                    unreachable!()
                };

            let f_value = execute_expression(value, program_archive, runtime, actual_node, flags)?;
            let slice_value = safe_unwrap_to_arithmetic_slice(f_value, line!());

            let mut dims = vec![usable_dimension];
            for dim in slice_value.route() {
                dims.push(*dim);
            }

            let mut array_slice = AExpressionSlice::new_with_route(&dims, &WrappedAExpr::unknown());
            let mut row: SliceCapacity = 0;
            while row < usable_dimension {
                let memory_insert_result =
                    AExpressionSlice::insert_values(&mut array_slice, &[row], &slice_value, false);
                treat_result_with_memory_error_void(
                    memory_insert_result,
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?;
                row += 1;
            }
            FoldedValue {
                arithmetic_slice: Option::Some(array_slice),
                ..FoldedValue::default()
            }
        }
        InfixOp {
            meta,
            lhe,
            infix_op,
            rhe,
            ..
        } => {
            let l_fold = execute_expression(lhe, program_archive, runtime, actual_node, flags)?;
            let r_fold = execute_expression(rhe, program_archive, runtime, actual_node, flags)?;
            let l_value = safe_unwrap_to_single_arithmetic_expression(l_fold, line!());
            let r_value = safe_unwrap_to_single_arithmetic_expression(r_fold, line!());
            let re_value =
                execute_infix_op(meta, *infix_op, &l_value.aexpr, &r_value.aexpr, runtime)?;
            let re_wrapped = if let ArithmeticExpressionGen::Number { value } = re_value {
                WrappedAExpr::from_number(&value, runtime)
            } else {
                WrappedAExpr::from_infix(
                    re_value,
                    l_value.trace_identifier,
                    r_value.trace_identifier,
                    *infix_op,
                    runtime,
                )
            };
            let r_slice = AExpressionSlice::new(&re_wrapped);
            FoldedValue {
                arithmetic_slice: Option::Some(r_slice),
                ..FoldedValue::default()
            }
        }
        PrefixOp { prefix_op, rhe, .. } => {
            let folded_value =
                execute_expression(rhe, program_archive, runtime, actual_node, flags)?;
            let arithmetic_value =
                safe_unwrap_to_single_arithmetic_expression(folded_value, line!());
            let arithmetic_result =
                execute_prefix_op(*prefix_op, &arithmetic_value.aexpr, runtime)?;
            let ar_wrapped = if let ArithmeticExpressionGen::Number { value } = arithmetic_result {
                WrappedAExpr::from_number(&value, runtime)
            } else {
                WrappedAExpr::from_prefix(
                    arithmetic_result,
                    arithmetic_value.trace_identifier,
                    *prefix_op,
                    runtime,
                )
            };
            let slice_result = AExpressionSlice::new(&ar_wrapped);
            FoldedValue {
                arithmetic_slice: Option::Some(slice_result),
                ..FoldedValue::default()
            }
        }
        InlineSwitchOp {
            cond,
            if_true,
            if_false,
            ..
        } => {
            let f_cond = execute_expression(cond, program_archive, runtime, actual_node, flags)?;
            let ae_cond = safe_unwrap_to_single_arithmetic_expression(f_cond, line!());
            let possible_bool_cond =
                AExpr::get_boolean_equivalence(&ae_cond.aexpr, runtime.constants.get_p());
            if let Option::Some(bool_cond) = possible_bool_cond {
                if bool_cond {
                    execute_expression(if_true, program_archive, runtime, actual_node, flags)?
                } else {
                    execute_expression(if_false, program_archive, runtime, actual_node, flags)?
                }
            } else {
                let if_true_fold =
                    execute_expression(if_true, program_archive, runtime, actual_node, flags)?;
                let if_false_fold =
                    execute_expression(if_false, program_archive, runtime, actual_node, flags)?;
                let if_true_value =
                    safe_unwrap_to_single_arithmetic_expression(if_true_fold, line!());
                let if_false_value =
                    safe_unwrap_to_single_arithmetic_expression(if_false_fold, line!());
                let arithmetic_slice =
                    Option::Some(AExpressionSlice::new(&WrappedAExpr::from_inline_switch(
                        ae_cond.trace_identifier,
                        if_true_value.trace_identifier,
                        if_false_value.trace_identifier,
                        runtime,
                    )));
                FoldedValue {
                    arithmetic_slice,
                    ..FoldedValue::default()
                }
            }
        }
        Call { id, args, meta, .. } => {
            let (value, can_simplify) =
                execute_call(id, meta, args, program_archive, runtime, actual_node, flags)?;
            can_be_simplified = can_simplify;
            value
        }
        ParallelOp { rhe, .. } => {
            let folded_value =
                execute_expression(rhe, program_archive, runtime, actual_node, flags)?;
            let (node_pointer, _) = safe_unwrap_to_valid_node_pointer(folded_value, line!());
            FoldedValue {
                node_pointer: Option::Some(node_pointer),
                is_parallel: Option::Some(true),
                ..FoldedValue::default()
            }
        }
        _ => {
            unreachable!("Anonymous calls should not be reachable at this point.");
        }
    };
    let expr_id = expr.get_meta().elem_id;
    let res_p = res.arithmetic_slice.clone();
    if let Some(slice) = res_p {
        if slice.is_single() && can_be_simplified {
            let value = AExpressionSlice::unwrap_to_single(slice);
            Analysis::computed(&mut runtime.analysis, expr_id, value.aexpr);
        }
    }
    Result::Ok(res)
}

//************************************************* Statement execution support *************************************************

fn execute_call(
    id: &String,
    meta: &Meta,
    args: &Vec<Expression>,
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    actual_node: &mut Option<ExecutedTemplate>,
    flags: FlagsExecution,
) -> Result<(FoldedValue, bool), ()> {
    let mut arg_values = Vec::new();

    let is_template = program_archive.contains_template(id);

    for arg_expression in args.iter() {
        let f_arg =
            execute_expression(arg_expression, program_archive, runtime, actual_node, flags)?;
        let safe_f_arg = safe_unwrap_to_arithmetic_slice(f_arg, line!());
        if is_template {
            // check that all the arguments are known
            for value in MemorySlice::get_reference_values(&safe_f_arg) {
                if !AExpr::is_number(&value.aexpr) {
                    treat_result_with_execution_error(
                        Result::Err(ExecutionError::UnknownTemplate),
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?;
                }
            }
        }
        arg_values.push(safe_f_arg);
    }
    if program_archive.contains_function(id) {
        // in this case we execute
        let new_environment = prepare_environment_for_call(id, &arg_values, program_archive);
        let previous_environment = std::mem::replace(&mut runtime.environment, new_environment);
        let previous_block_type = std::mem::replace(&mut runtime.block_type, BlockType::Known);
        let previous_anonymous_components = std::mem::replace(
            &mut runtime.anonymous_components,
            AnonymousComponentsInfo::new(),
        );

        let new_file_id = program_archive.get_function_data(id).get_file_id();
        let previous_id = std::mem::replace(&mut runtime.current_file, new_file_id);

        runtime.call_trace.push(id.clone());
        let folded_result = execute_function_call(id, program_archive, runtime, flags)?;

        runtime.environment = previous_environment;
        runtime.current_file = previous_id;
        runtime.block_type = previous_block_type;
        runtime.anonymous_components = previous_anonymous_components;
        runtime.call_trace.pop();
        Ok(folded_result)
    } else {
        // in this case we preexecute and check if it needs tags
        let folded_result = preexecute_template_call(id, &arg_values, program_archive, runtime)?;
        Ok((folded_result, true))
    }
}

fn execute_template_call_complete(
    id: &String,
    arg_values: Vec<AExpressionSlice>,
    tags: BTreeMap<String, TagInfo>,
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    flags: FlagsExecution,
) -> Result<FoldedValue, ()> {
    if program_archive.contains_template(id) {
        // in this case we execute
        let new_environment = prepare_environment_for_call(id, &arg_values, program_archive);
        let previous_environment = std::mem::replace(&mut runtime.environment, new_environment);
        let previous_block_type = std::mem::replace(&mut runtime.block_type, BlockType::Known);
        let previous_anonymous_components = std::mem::replace(
            &mut runtime.anonymous_components,
            AnonymousComponentsInfo::new(),
        );

        let new_file_id = program_archive.get_template_data(id).get_file_id();
        let previous_id = std::mem::replace(&mut runtime.current_file, new_file_id);

        runtime.call_trace.push(id.clone());
        let folded_result =
            execute_template_call(id, arg_values, tags, program_archive, runtime, flags)?;

        runtime.environment = previous_environment;
        runtime.current_file = previous_id;
        runtime.block_type = previous_block_type;
        runtime.anonymous_components = previous_anonymous_components;
        runtime.call_trace.pop();
        Ok(folded_result)
    } else {
        unreachable!();
    }
}

fn execute_component_declaration(
    component_name: &str,
    dimensions: &[SliceCapacity],
    environment: &mut ExecutionEnvironment,
    actual_node: &mut Option<ExecutedTemplate>,
) {
    if let Option::Some(node) = actual_node {
        node.add_component(component_name, dimensions);
        environment_shortcut_add_component(environment, component_name, dimensions);
    } else {
        unreachable!()
    }
}

fn execute_anonymous_component_declaration(
    component_name: &str,
    meta: Meta,
    dimensions: &Vec<Expression>,
    environment: &mut ExecutionEnvironment,
    anonymous_components: &mut AnonymousComponentsInfo,
) {
    environment_shortcut_add_component(environment, component_name, &Vec::new());
    anonymous_components.insert(component_name.to_string(), (meta, dimensions.clone()));
}

fn execute_signal_declaration(
    signal_name: &str,
    dimensions: &[SliceCapacity],
    list_tags: &Vec<String>,
    signal_type: SignalType,
    environment: &mut ExecutionEnvironment,
    actual_node: &mut Option<ExecutedTemplate>,
) {
    use SignalType::*;
    let mut tags = TagInfo::new();
    for t in list_tags {
        tags.insert(t.clone(), None);
    }
    if let Option::Some(node) = actual_node {
        node.add_ordered_signal(signal_name, dimensions);
        match signal_type {
            Input => {
                if let Some(tags_input) = node.tag_instances().get(signal_name) {
                    environment_shortcut_add_input(
                        environment,
                        signal_name,
                        dimensions,
                        &tags_input,
                    );
                } else {
                    environment_shortcut_add_input(environment, signal_name, dimensions, &tags);
                }
                node.add_input(signal_name, dimensions);
            }
            Output => {
                environment_shortcut_add_output(environment, signal_name, dimensions, &tags);
                node.add_output(signal_name, dimensions);
            }
            Intermediate => {
                environment_shortcut_add_intermediate(environment, signal_name, dimensions, &tags);
                node.add_intermediate(signal_name, dimensions);
            }
        }
    } else {
        unreachable!();
    }
}

/*
    In case the assignment could be a constraint generator the returned value is the constraint
    that will be created
*/
struct Constrained {
    left: String,
    right: AExpressionSlice,
}
fn perform_assign(
    meta: &Meta,
    symbol: &str,
    op: AssignOp,
    accessing_information: &AccessingInformation,
    r_folded: FoldedValue,
    actual_node: &mut Option<ExecutedTemplate>,
    runtime: &mut RuntimeInformation,
    program_archive: &ProgramArchive,
    flags: FlagsExecution,
) -> Result<Option<Constrained>, ()> {
    use super::execution_data::type_definitions::SubComponentData;
    let full_symbol = create_symbol(symbol, &accessing_information);
    //println!("perform_assign: full_symbol={:?} r_folded={:?}", full_symbol, r_folded);

    let possible_arithmetic_slice =
        if ExecutionEnvironment::has_variable(&runtime.environment, symbol) {
            debug_assert!(accessing_information.signal_access.is_none());
            debug_assert!(accessing_information.after_signal.is_empty());
            let environment_result =
                ExecutionEnvironment::get_mut_variable_mut(&mut runtime.environment, symbol);
            let (symbol_tags, symbol_content) = treat_result_with_environment_error(
                environment_result,
                meta,
                &mut runtime.runtime_errors,
                &runtime.call_trace,
            )?;
            let mut r_tags = if r_folded.tags.is_some() {
                r_folded.tags.as_ref().unwrap().clone()
            } else {
                TagInfo::new()
            };
            let mut r_slice = safe_unwrap_to_arithmetic_slice(r_folded, line!());
            if runtime.block_type == BlockType::Unknown {
                r_slice =
                    AExpressionSlice::new_with_route(r_slice.route(), &WrappedAExpr::unknown());
                r_tags = TagInfo::new();
            }
            if accessing_information.undefined {
                let new_value = AExpressionSlice::new_with_route(
                    symbol_content.route(),
                    &WrappedAExpr::unknown(),
                );
                let memory_result =
                    AExpressionSlice::insert_values(symbol_content, &vec![], &new_value, false);
                treat_result_with_memory_error_void(
                    memory_result,
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?;
                *symbol_tags = TagInfo::new();
            } else {
                let memory_result = AExpressionSlice::insert_values(
                    symbol_content,
                    &accessing_information.before_signal,
                    &r_slice,
                    false,
                );
                treat_result_with_memory_error_void(
                    memory_result,
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?;
                // in case it is a complete assignment assign the tags, if not set the tags to empty
                if AExpressionSlice::get_number_of_cells(symbol_content)
                    == AExpressionSlice::get_number_of_cells(&r_slice)
                {
                    *symbol_tags = r_tags;
                } else {
                    *symbol_tags = TagInfo::new();
                }
            }
            Option::None
        } else if ExecutionEnvironment::has_signal(&runtime.environment, symbol)
            && accessing_information.signal_access.is_some()
        {
            if ExecutionEnvironment::has_input(&runtime.environment, symbol) {
                treat_result_with_memory_error(
                    Result::Err(MemoryError::AssignmentTagInput),
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?
            }

            if runtime.block_type == BlockType::Unknown {
                // Case not valid constraint Known/Unknown
                let err = Result::Err(ExecutionError::TagAssignmentInUnknown);
                treat_result_with_execution_error(
                    err,
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?;
            }

            let tag = accessing_information.signal_access.clone().unwrap();
            let environment_response =
                ExecutionEnvironment::get_mut_signal_res(&mut runtime.environment, symbol);
            let (reference_to_tags, reference_to_tags_defined, reference_to_signal_content) =
                treat_result_with_environment_error(
                    environment_response,
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?;

            if SignalSlice::get_number_of_inserts(&reference_to_signal_content) > 0 {
                treat_result_with_memory_error(
                    Result::Err(MemoryError::AssignmentTagAfterInit),
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?
            } else if let Some(a_slice) = r_folded.arithmetic_slice {
                let value = AExpressionSlice::unwrap_to_single(a_slice);
                match value.aexpr {
                    ArithmeticExpressionGen::Number { value } => {
                        let possible_tag = reference_to_tags.get(&tag.clone());
                        if let Some(val) = possible_tag {
                            if let Some(_) = val {
                                treat_result_with_memory_error(
                                    Result::Err(MemoryError::AssignmentTagTwice),
                                    meta,
                                    &mut runtime.runtime_errors,
                                    &runtime.call_trace,
                                )?
                            } else {
                                // we add the info saying that the tag is defined
                                reference_to_tags.insert(tag.clone(), Option::Some(value.clone()));
                                let tag_state = reference_to_tags_defined.get_mut(&tag).unwrap();
                                tag_state.value_defined = true;
                                if let Option::Some(node) = actual_node {
                                    node.add_tag_signal(symbol, &tag, Some(value));
                                } else {
                                    unreachable!();
                                }
                            }
                        } else {
                            unreachable!()
                        }
                    }

                    _ => {
                        treat_result_with_execution_error(
                            Result::Err(ExecutionError::NonValidTagAssignment),
                            meta,
                            &mut runtime.runtime_errors,
                            &runtime.call_trace,
                        )?;
                    }
                }
            } else {
                unreachable!()
            }
            Option::None
        } else if ExecutionEnvironment::has_signal(&runtime.environment, symbol) {
            debug_assert!(accessing_information.signal_access.is_none());
            debug_assert!(accessing_information.after_signal.is_empty());

            // to ensure that input signals are not assigned twice, improving error message
            if ExecutionEnvironment::has_input(&runtime.environment, symbol) {
                treat_result_with_memory_error(
                    Err(MemoryError::AssignmentError(
                        TypeAssignmentError::AssignmentInput(symbol.to_string()),
                    )),
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?
            }

            let environment_response =
                ExecutionEnvironment::get_mut_signal_res(&mut runtime.environment, symbol);
            let (reference_to_tags, reference_to_tags_defined, reference_to_signal_content) =
                treat_result_with_environment_error(
                    environment_response,
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?;
            let memory_response_for_signal_previous_value = SignalSlice::access_values(
                reference_to_signal_content,
                &accessing_information.before_signal,
            );
            let signal_previous_value = treat_result_with_memory_error(
                memory_response_for_signal_previous_value,
                meta,
                &mut runtime.runtime_errors,
                &runtime.call_trace,
            )?;

            // Study the tags: add the new ones and copy their content.
            /*
            Cases:

                Inherance in arrays => We only have a tag in case it inherites the tag in all positions

                - Tag defined by user:
                    * Already with value defined by user => do not copy new values
                    * No value defined by user
                       - Already initialized:
                         * If same value as previous preserve
                         * If not set value to None
                       - No initialized:
                         * Set value to new one
                - Tag not defined by user:
                    * Already initialized:
                       - If contains same tag with same value preserve
                       - No tag or different value => do not save tag or loose it
                    * No initialized:
                       - Save tag


            */
            let previous_tags = mem::take(reference_to_tags);

            let new_tags = if r_folded.tags.is_some() && op == AssignOp::AssignConstraintSignal {
                r_folded.tags.clone().unwrap()
            } else {
                TagInfo::new()
            };

            let signal_is_init =
                SignalSlice::get_number_of_inserts(reference_to_signal_content) > 0;

            for (tag, value) in previous_tags {
                let tag_state = reference_to_tags_defined.get(&tag).unwrap();
                if tag_state.defined {
                    // is signal defined by user
                    if tag_state.value_defined {
                        // already with value, store the same value
                        reference_to_tags.insert(tag, value);
                    } else {
                        if signal_is_init {
                            // only keep value if same as previous
                            let to_store_value = if new_tags.contains_key(&tag) {
                                let value_new = new_tags.get(&tag).unwrap();
                                if value != *value_new {
                                    None
                                } else {
                                    value
                                }
                            } else {
                                None
                            };
                            reference_to_tags.insert(tag, to_store_value);
                        } else {
                            // always keep
                            if new_tags.contains_key(&tag) {
                                let value_new = new_tags.get(&tag).unwrap();
                                reference_to_tags.insert(tag, value_new.clone());
                            } else {
                                reference_to_tags.insert(tag, None);
                            }
                        }
                    }
                } else {
                    // it is not defined by user
                    if new_tags.contains_key(&tag) {
                        let value_new = new_tags.get(&tag).unwrap();
                        if value == *value_new {
                            reference_to_tags.insert(tag, value);
                        } else {
                            reference_to_tags_defined.remove(&tag);
                        }
                    } else {
                        reference_to_tags_defined.remove(&tag);
                    }
                }
            }

            if !signal_is_init {
                // first init, add new tags
                for (tag, value) in new_tags {
                    if !reference_to_tags.contains_key(&tag) {
                        // in case it is a new tag (not defined by user)
                        reference_to_tags.insert(tag.clone(), value.clone());
                        let state = TagState {
                            defined: false,
                            value_defined: false,
                            complete: false,
                        };
                        reference_to_tags_defined.insert(tag.clone(), state);
                    }
                }
            }

            let r_slice = safe_unwrap_to_arithmetic_slice(r_folded, line!());
            let new_value_slice = &SignalSlice::new_with_route(r_slice.route(), &true);

            let correct_dims_result = SignalSlice::check_correct_dims(
                &signal_previous_value,
                &Vec::new(),
                &new_value_slice,
                true,
            );
            treat_result_with_memory_error_void(
                correct_dims_result,
                meta,
                &mut runtime.runtime_errors,
                &runtime.call_trace,
            )?;

            for i in 0..SignalSlice::get_number_of_cells(&signal_previous_value) {
                //println!("assign {:?}",i);
                let signal_was_assigned = treat_result_with_memory_error(
                    SignalSlice::access_value_by_index(&signal_previous_value, i),
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?;
                if signal_was_assigned {
                    let access_response = Result::Err(MemoryError::AssignmentError(
                        TypeAssignmentError::MultipleAssignments,
                    ));
                    treat_result_with_memory_error(
                        access_response,
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?;
                }
            }

            if let Some(node) = actual_node {
                for i in 0..MemorySlice::get_number_of_cells(&r_slice) {
                    let access_index =
                        MemorySlice::get_access_index(&r_slice, i).unwrap_or_default();
                    let access_info = AccessingInformation {
                        undefined: false,
                        before_signal: Vec::new(),
                        signal_access: None,
                        after_signal: access_index,
                        tag_access: None,
                    };
                    let cur_symbol = create_symbol(&full_symbol, &access_info);
                    node.assign_signal_trace(
                        cur_symbol,
                        MemorySlice::get_reference_to_single_value_by_index_or_break(&r_slice, i)
                            .trace_identifier,
                    );
                }
            }

            let access_response = SignalSlice::insert_values(
                reference_to_signal_content,
                &accessing_information.before_signal,
                &new_value_slice,
                true,
            );

            let signal_is_completely_initialized =
                SignalSlice::get_number_of_inserts(reference_to_signal_content)
                    == SignalSlice::get_number_of_cells(reference_to_signal_content);

            if signal_is_completely_initialized {
                for (tag, value) in reference_to_tags {
                    let tag_state = reference_to_tags_defined.get_mut(tag).unwrap();
                    tag_state.complete = true;
                    if let Option::Some(node) = actual_node {
                        if !tag_state.value_defined {
                            node.add_tag_signal(symbol, &tag, value.clone());
                        }
                    } else {
                        unreachable!();
                    }
                }
            }

            treat_result_with_memory_error_void(
                access_response,
                meta,
                &mut runtime.runtime_errors,
                &runtime.call_trace,
            )?;

            Option::Some(r_slice)
        } else if ExecutionEnvironment::has_component(&runtime.environment, symbol) {
            if accessing_information.tag_access.is_some() {
                unreachable!()
            }
            let environment_response =
                ExecutionEnvironment::get_mut_component_res(&mut runtime.environment, symbol);
            let component_slice = treat_result_with_environment_error(
                environment_response,
                meta,
                &mut runtime.runtime_errors,
                &runtime.call_trace,
            )?;

            let is_anonymous_component = runtime.anonymous_components.contains_key(symbol);
            let memory_response = if is_anonymous_component {
                ComponentSlice::get_mut_reference_to_single_value(component_slice, &Vec::new())
            } else {
                ComponentSlice::get_mut_reference_to_single_value(
                    component_slice,
                    &accessing_information.before_signal,
                )
            };
            let component = treat_result_with_memory_error(
                memory_response,
                meta,
                &mut runtime.runtime_errors,
                &runtime.call_trace,
            )?;
            if accessing_information.signal_access.is_none() {
                let (prenode_pointer, is_parallel) =
                    safe_unwrap_to_valid_node_pointer(r_folded, line!());
                let memory_result = ComponentRepresentation::preinitialize_component(
                    component,
                    is_parallel,
                    prenode_pointer,
                    &runtime.exec_program,
                    is_anonymous_component,
                    meta,
                );
                treat_result_with_memory_error_void(
                    memory_result,
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?;
                if component.is_ready_initialize() {
                    // calls to execute and initialize the component
                    let pretemplate_info = runtime
                        .exec_program
                        .get_prenode_value(prenode_pointer)
                        .unwrap();
                    let inputs_tags = component.inputs_tags.clone();
                    let result = execute_template_call_complete(
                        pretemplate_info.template_name(),
                        pretemplate_info.parameter_instances().clone(),
                        inputs_tags,
                        program_archive,
                        runtime,
                        flags,
                    )?;
                    let (node_pointer, _is_parallel) =
                        safe_unwrap_to_valid_node_pointer(result, line!());

                    if let Some(node) = actual_node {
                        node.assign_component(full_symbol.clone(), node_pointer);
                    }

                    let environment_response = ExecutionEnvironment::get_mut_component_res(
                        &mut runtime.environment,
                        symbol,
                    );
                    let component_slice = treat_result_with_environment_error(
                        environment_response,
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?;
                    let memory_response = if is_anonymous_component {
                        ComponentSlice::get_mut_reference_to_single_value(
                            component_slice,
                            &Vec::new(),
                        )
                    } else {
                        ComponentSlice::get_mut_reference_to_single_value(
                            component_slice,
                            &accessing_information.before_signal,
                        )
                    };
                    let component = treat_result_with_memory_error(
                        memory_response,
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?;

                    let init_result = ComponentRepresentation::initialize_component(
                        component,
                        node_pointer,
                        &mut runtime.exec_program,
                    );
                    treat_result_with_memory_error(
                        init_result,
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?;
                    if let Option::Some(actual_node) = actual_node {
                        let data = SubComponentData {
                            name: symbol.to_string(),
                            is_parallel: component.is_parallel,
                            goes_to: node_pointer,
                            indexed_with: accessing_information.before_signal.clone(),
                        };
                        actual_node.add_arrow(full_symbol.clone(), data);
                    } else {
                        unreachable!();
                    }
                }
                Option::None
            } else {
                let signal_accessed = accessing_information.signal_access.clone().unwrap();
                debug_assert!(FoldedValue::valid_arithmetic_slice(&r_folded));
                let arithmetic_slice = r_folded.arithmetic_slice.unwrap();
                let tags = if r_folded.tags.is_some() {
                    r_folded.tags.unwrap()
                } else {
                    TagInfo::new()
                };

                if let Some(node) = actual_node {
                    for i in 0..MemorySlice::get_number_of_cells(&arithmetic_slice) {
                        let access_index =
                            MemorySlice::get_access_index(&arithmetic_slice, i).unwrap_or_default();
                        let access_info = AccessingInformation {
                            undefined: false,
                            before_signal: Vec::new(),
                            signal_access: None,
                            after_signal: access_index,
                            tag_access: None,
                        };
                        let cur_symbol = create_symbol(&full_symbol, &access_info);
                        node.assign_signal_trace(
                            cur_symbol,
                            MemorySlice::get_reference_to_single_value_by_index_or_break(
                                &arithmetic_slice,
                                i,
                            )
                            .trace_identifier,
                        );
                    }
                }

                let memory_response = ComponentRepresentation::assign_value_to_signal(
                    component,
                    &signal_accessed,
                    &accessing_information.after_signal,
                    &arithmetic_slice.route(),
                    tags,
                );
                treat_result_with_memory_error_void(
                    memory_response,
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?;
                if !component.is_initialized && component.is_ready_initialize() {
                    // calls to execute and initialize the component
                    let pretemplate_info = runtime
                        .exec_program
                        .get_prenode_value(component.node_pointer.unwrap())
                        .unwrap();
                    let inputs_tags = component.inputs_tags.clone();

                    let folded_result = execute_template_call_complete(
                        pretemplate_info.template_name(),
                        pretemplate_info.parameter_instances().clone(),
                        inputs_tags,
                        program_archive,
                        runtime,
                        flags,
                    )?;

                    let (node_pointer, _is_parallel) =
                        safe_unwrap_to_valid_node_pointer(folded_result, line!());

                    if let Some(node) = actual_node {
                        node.assign_component(full_symbol.clone(), node_pointer);
                    }

                    let environment_response = ExecutionEnvironment::get_mut_component_res(
                        &mut runtime.environment,
                        symbol,
                    );
                    let component_slice = treat_result_with_environment_error(
                        environment_response,
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?;
                    let memory_response = if is_anonymous_component {
                        ComponentSlice::get_mut_reference_to_single_value(
                            component_slice,
                            &Vec::new(),
                        )
                    } else {
                        ComponentSlice::get_mut_reference_to_single_value(
                            component_slice,
                            &accessing_information.before_signal,
                        )
                    };
                    let component = treat_result_with_memory_error(
                        memory_response,
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?;

                    let init_result = ComponentRepresentation::initialize_component(
                        component,
                        node_pointer,
                        &mut runtime.exec_program,
                    );
                    treat_result_with_memory_error_void(
                        init_result,
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?;
                    if let Option::Some(actual_node) = actual_node {
                        let data = SubComponentData {
                            name: symbol.to_string(),
                            goes_to: node_pointer,
                            is_parallel: component.is_parallel,
                            indexed_with: accessing_information.before_signal.clone(),
                        };
                        let component_symbol =
                            create_component_symbol(symbol, &accessing_information);
                        actual_node.add_arrow(component_symbol, data);
                    } else {
                        unreachable!();
                    }
                }
                Option::Some(arithmetic_slice)
            }
        } else {
            unreachable!();
        };
    if let Option::Some(arithmetic_slice) = possible_arithmetic_slice {
        let ret = Constrained {
            left: full_symbol,
            right: arithmetic_slice,
        };
        Result::Ok(Some(ret))
    } else {
        Result::Ok(None)
    }
}

// Evaluates the given condition and executes the corresponding statement. Returns a tuple (a,b) where a is the possible value returned and b is the value of the condition (in case the evaluation was successful)
fn execute_conditional_statement(
    condition: &Expression,
    true_case: &Statement,
    false_case: Option<&Statement>,
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    actual_node: &mut Option<ExecutedTemplate>,
    flags: FlagsExecution,
) -> Result<(Option<FoldedValue>, bool, Option<bool>), ()> {
    let f_cond = execute_expression(condition, program_archive, runtime, actual_node, flags)?;
    let ae_cond = safe_unwrap_to_single_arithmetic_expression(f_cond, line!()).aexpr;
    let possible_cond_bool_value =
        AExpr::get_boolean_equivalence(&ae_cond, runtime.constants.get_p());
    if let Some(cond_bool_value) = possible_cond_bool_value {
        let (ret_value, can_simplify) = match false_case {
            Some(else_stmt) if !cond_bool_value => {
                execute_statement(else_stmt, program_archive, runtime, actual_node, flags)?
            }
            None if !cond_bool_value => (None, true),
            _ => execute_statement(true_case, program_archive, runtime, actual_node, flags)?,
        };
        Result::Ok((ret_value, can_simplify, Option::Some(cond_bool_value)))
    } else {
        let previous_block_type = runtime.block_type;
        runtime.block_type = BlockType::Unknown;
        let (mut ret_value, mut can_simplify) =
            execute_statement(true_case, program_archive, runtime, actual_node, flags)?;
        if let Option::Some(else_stmt) = false_case {
            let (else_ret, can_simplify_else) =
                execute_statement(else_stmt, program_archive, runtime, actual_node, flags)?;
            can_simplify &= can_simplify_else;
            if ret_value.is_none() {
                ret_value = else_ret;
            }
        }
        runtime.block_type = previous_block_type;
        return Result::Ok((ret_value, can_simplify, Option::None));
    }
}

fn execute_sequence_of_statements(
    stmts: &[Statement],
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    actual_node: &mut Option<ExecutedTemplate>,
    flags: FlagsExecution,
    is_complete_template: bool,
) -> Result<(Option<FoldedValue>, bool), ()> {
    let mut can_be_simplified = true;
    for stmt in stmts.iter() {
        let (f_value, can_simplify) =
            execute_statement(stmt, program_archive, runtime, actual_node, flags)?;
        can_be_simplified &= can_simplify;
        if f_value.is_some() {
            return Result::Ok((f_value, can_be_simplified));
        }
    }
    if is_complete_template {
        execute_delayed_declarations(program_archive, runtime, actual_node, flags)?;
    }
    Result::Ok((Option::None, can_be_simplified))
}

fn execute_delayed_declarations(
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    actual_node: &mut Option<ExecutedTemplate>,
    flags: FlagsExecution,
) -> Result<(), ()> {
    for (component_name, (meta, dimensions)) in runtime.anonymous_components.clone() {
        let mut arithmetic_values = Vec::new();
        for dimension in dimensions.iter() {
            let f_dimensions =
                execute_expression(dimension, program_archive, runtime, actual_node, flags)?;
            arithmetic_values
                .push(safe_unwrap_to_single_arithmetic_expression(f_dimensions, line!()).aexpr);
        }
        treat_result_with_memory_error_void(
            valid_array_declaration(&arithmetic_values),
            &meta,
            &mut runtime.runtime_errors,
            &runtime.call_trace,
        )?;
        let usable_dimensions = if let Option::Some(dimensions) = cast_indexing(&arithmetic_values)
        {
            dimensions
        } else {
            let err = Result::Err(ExecutionError::ArraySizeTooBig);
            treat_result_with_execution_error(
                err,
                &meta,
                &mut runtime.runtime_errors,
                &runtime.call_trace,
            )?
        };
        if let Option::Some(node) = actual_node {
            node.add_component(&component_name, &usable_dimensions);
        }
    }
    Result::Ok(())
}

//************************************************* Expression execution support *************************************************

fn create_component_symbol(symbol: &str, access_information: &AccessingInformation) -> String {
    let mut appendix = "".to_string();
    let bf_signal = create_index_appendix(&access_information.before_signal);
    appendix.push_str(&bf_signal);
    format!("{}{}", symbol, appendix)
}

fn create_symbol(symbol: &str, access_information: &AccessingInformation) -> String {
    let mut appendix = "".to_string();
    let bf_signal = create_index_appendix(&access_information.before_signal);
    let af_signal = create_index_appendix(&access_information.after_signal);
    appendix.push_str(&bf_signal);
    if let Option::Some(signal_accessed) = &access_information.signal_access {
        let signal = format!(".{}", signal_accessed);
        appendix.push_str(&signal);
    }
    appendix.push_str(&af_signal);
    format!("{}{}", symbol, appendix)
}

fn create_index_appendix(indexing: &[usize]) -> String {
    let mut appendix = "".to_string();
    for index in indexing {
        let index = format!("[{}]", index);
        appendix.push_str(&index);
    }
    appendix
}

fn execute_variable(
    meta: &Meta,
    symbol: &str,
    access: &[Access],
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    actual_node: &mut Option<ExecutedTemplate>,
    flags: FlagsExecution,
) -> Result<FoldedValue, ()> {
    let access_information =
        treat_accessing(meta, access, program_archive, runtime, actual_node, flags)?;
    if access_information.undefined {
        let arithmetic_slice = Option::Some(AExpressionSlice::new(&WrappedAExpr::unknown()));
        return Result::Ok(FoldedValue {
            arithmetic_slice,
            ..FoldedValue::default()
        });
    }
    debug_assert!(access_information.signal_access.is_none());
    debug_assert!(access_information.after_signal.is_empty());
    let indexing = access_information.before_signal;
    let environment_response = ExecutionEnvironment::get_variable_res(&runtime.environment, symbol);
    let (var_tag, ae_slice) = treat_result_with_environment_error(
        environment_response,
        meta,
        &mut runtime.runtime_errors,
        &runtime.call_trace,
    )?;
    let memory_response = AExpressionSlice::access_values(&ae_slice, &indexing);
    let ae_slice = treat_result_with_memory_error(
        memory_response,
        meta,
        &mut runtime.runtime_errors,
        &runtime.call_trace,
    )?;
    Result::Ok(FoldedValue {
        arithmetic_slice: Option::Some(ae_slice),
        tags: Option::Some(var_tag.clone()),
        ..FoldedValue::default()
    })
}

fn execute_signal(
    meta: &Meta,
    symbol: &str,
    access: &[Access],
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    actual_node: &mut Option<ExecutedTemplate>,
    flags: FlagsExecution,
) -> Result<FoldedValue, ()> {
    let access_information =
        treat_accessing(meta, access, program_archive, runtime, actual_node, flags)?;
    if access_information.undefined {
        let arithmetic_slice = Option::Some(AExpressionSlice::new(&WrappedAExpr::unknown()));
        return Result::Ok(FoldedValue {
            arithmetic_slice,
            ..FoldedValue::default()
        });
    }
    debug_assert!(access_information.after_signal.is_empty());
    let indexing = &access_information.before_signal;
    let environment_response = if ExecutionEnvironment::has_input(&runtime.environment, symbol) {
        ExecutionEnvironment::get_input_res(&runtime.environment, symbol)
    } else if ExecutionEnvironment::has_output(&runtime.environment, symbol) {
        ExecutionEnvironment::get_output_res(&runtime.environment, symbol)
    } else if ExecutionEnvironment::has_intermediate(&runtime.environment, symbol) {
        ExecutionEnvironment::get_intermediate_res(&runtime.environment, symbol)
    } else {
        unreachable!();
    };
    let (tags, tags_definitions, signal_slice) = treat_result_with_environment_error(
        environment_response,
        meta,
        &mut runtime.runtime_errors,
        &runtime.call_trace,
    )?;
    if let Some(acc) = access_information.signal_access {
        if tags.contains_key(&acc) {
            let value_tag = tags.get(&acc).unwrap();
            let state = tags_definitions.get(&acc).unwrap();
            if let Some(value_tag) = value_tag {
                // tag has value
                // access only allowed when (1) it is value defined by user or (2) it is completely assigned
                if state.value_defined || state.complete {
                    let ae_slice = AExpressionSlice::new(&WrappedAExpr::from_number_registry(
                        value_tag,
                        &mut runtime.trace_registry,
                    ));
                    Result::Ok(FoldedValue {
                        arithmetic_slice: Option::Some(ae_slice),
                        ..FoldedValue::default()
                    })
                } else {
                    let error = MemoryError::TagValueNotInitializedAccess;
                    treat_result_with_memory_error(
                        Result::Err(error),
                        meta,
                        &mut runtime.runtime_errors,
                        &runtime.call_trace,
                    )?
                }
            } else {
                let error = MemoryError::TagValueNotInitializedAccess;
                treat_result_with_memory_error(
                    Result::Err(error),
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?
            }
        } else {
            unreachable!()
        }
    } else {
        let memory_response = SignalSlice::access_values(signal_slice, indexing);
        let signal_slice = treat_result_with_memory_error(
            memory_response,
            meta,
            &mut runtime.runtime_errors,
            &runtime.call_trace,
        )?;
        let full_symbol = create_symbol(symbol, &access_information);
        let signal_access = signal_to_arith(
            full_symbol,
            signal_slice,
            &mut runtime.trace_registry,
            actual_node,
        );
        let arith_slice = treat_result_with_memory_error(
            signal_access,
            meta,
            &mut runtime.runtime_errors,
            &runtime.call_trace,
        )?;

        let mut tags_propagated = TagInfo::new();
        for (tag, value) in tags {
            let state = tags_definitions.get(tag).unwrap();
            if state.value_defined || state.complete {
                tags_propagated.insert(tag.clone(), value.clone());
            } else if state.defined {
                tags_propagated.insert(tag.clone(), None);
            }
        }

        Result::Ok(FoldedValue {
            arithmetic_slice: Option::Some(arith_slice),
            tags: Option::Some(tags_propagated),
            ..FoldedValue::default()
        })
    }
}

fn signal_to_arith(
    symbol: String,
    slice: SignalSlice,
    trace_registry: &mut TraceRegistry,
    actual_node: &mut Option<ExecutedTemplate>,
) -> Result<AExpressionSlice, MemoryError> {
    let mut expressions = vec![];
    let (route, values) = slice.destruct();
    let mut symbols = vec![];
    unfold_signals(symbol, 0, &route, &mut symbols);
    let mut index = 0;
    while index < symbols.len() && values[index] {
        expressions.push(WrappedAExpr::from_signal_registry(
            &symbols[index],
            trace_registry,
            actual_node,
        ));
        index += 1;
    }
    if index == symbols.len() {
        Result::Ok(AExpressionSlice::new_array(route, expressions))
    } else {
        Result::Err(MemoryError::InvalidAccess(
            TypeInvalidAccess::NoInitializedSignal,
        ))
    }
}

fn unfold_signals(current: String, dim: usize, lengths: &[usize], result: &mut Vec<String>) {
    if dim == lengths.len() {
        result.push(current);
    } else {
        for i in 0..lengths[dim] {
            unfold_signals(format!("{}[{}]", current, i), dim + 1, lengths, result)
        }
    }
}

fn execute_component(
    meta: &Meta,
    symbol: &str,
    access: &[Access],
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    actual_node: &mut Option<ExecutedTemplate>,
    flags: FlagsExecution,
) -> Result<FoldedValue, ()> {
    let access_information =
        treat_accessing(meta, access, program_archive, runtime, actual_node, flags)?;
    if access_information.undefined {
        let arithmetic_slice = Option::Some(AExpressionSlice::new(&WrappedAExpr::unknown()));
        return Result::Ok(FoldedValue {
            arithmetic_slice,
            ..FoldedValue::default()
        });
    }
    let environment_response =
        ExecutionEnvironment::get_component_res(&runtime.environment, symbol);
    let component_slice = treat_result_with_environment_error(
        environment_response,
        meta,
        &mut runtime.runtime_errors,
        &runtime.call_trace,
    )?;
    let memory_response = if runtime.anonymous_components.contains_key(symbol) {
        ComponentSlice::access_values(component_slice, &Vec::new())
    } else {
        ComponentSlice::access_values(component_slice, &access_information.before_signal)
    };
    let slice_result = treat_result_with_memory_error(
        memory_response,
        meta,
        &mut runtime.runtime_errors,
        &runtime.call_trace,
    )?;
    let resulting_component = safe_unwrap_to_single(slice_result, line!());

    if let Some(acc) = access_information.tag_access {
        let (tags_signal, _) = treat_result_with_memory_error(
            resulting_component.get_signal(&access_information.signal_access.unwrap()),
            meta,
            &mut runtime.runtime_errors,
            &runtime.call_trace,
        )?;

        if tags_signal.contains_key(&acc) {
            let value_tag = tags_signal.get(&acc).unwrap();
            if let Some(value_tag) = value_tag {
                let ae_slice =
                    AExpressionSlice::new(&WrappedAExpr::from_number(value_tag, runtime));
                Result::Ok(FoldedValue {
                    arithmetic_slice: Option::Some(ae_slice),
                    ..FoldedValue::default()
                })
            } else {
                let error = MemoryError::TagValueNotInitializedAccess;
                treat_result_with_memory_error(
                    Result::Err(error),
                    meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?
            }
        } else {
            unreachable!()
        }
    } else if let Option::Some(signal_name) = &access_information.signal_access {
        let access_after_signal = &access_information.after_signal;
        let (tags_signal, signal) = treat_result_with_memory_error(
            resulting_component.get_signal(signal_name),
            meta,
            &mut runtime.runtime_errors,
            &runtime.call_trace,
        )?;
        let slice = SignalSlice::access_values(signal, &access_after_signal);
        let slice = treat_result_with_memory_error(
            slice,
            meta,
            &mut runtime.runtime_errors,
            &runtime.call_trace,
        )?;
        let symbol = create_symbol(symbol, &access_information);
        let result =
            signal_to_arith(symbol, slice, &mut runtime.trace_registry, actual_node).map(|s| {
                FoldedValue {
                    arithmetic_slice: Option::Some(s),
                    tags: Option::Some(tags_signal.clone()),
                    ..FoldedValue::default()
                }
            });
        treat_result_with_memory_error(
            result,
            meta,
            &mut runtime.runtime_errors,
            &runtime.call_trace,
        )
    } else {
        let read_result = if resulting_component.is_ready_initialize() {
            Result::Ok(resulting_component)
        } else {
            Result::Err(MemoryError::InvalidAccess(
                TypeInvalidAccess::NoInitializedComponent,
            ))
        };

        let checked_component = treat_result_with_memory_error(
            read_result,
            meta,
            &mut runtime.runtime_errors,
            &runtime.call_trace,
        )?;

        Result::Ok(FoldedValue {
            node_pointer: checked_component.node_pointer,
            is_parallel: Some(false),
            ..FoldedValue::default()
        })
    }
}

fn prepare_environment_for_call(
    id: &str,
    arg_values: &[AExpressionSlice],
    program_archive: &ProgramArchive,
) -> ExecutionEnvironment {
    let functions = program_archive.get_function_names();
    let arg_names = if functions.contains(id) {
        program_archive.get_function_data(id).get_name_of_params()
    } else {
        program_archive.get_template_data(id).get_name_of_params()
    };

    let mut environment = ExecutionEnvironment::new();
    debug_assert_eq!(arg_names.len(), arg_values.len());
    for (arg_name, arg_value) in arg_names.iter().zip(arg_values) {
        ExecutionEnvironment::add_variable(
            &mut environment,
            arg_name,
            (TagInfo::new(), arg_value.clone()),
        );
    }
    environment
}

fn execute_function_call(
    id: &str,
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    flags: FlagsExecution,
) -> Result<(FoldedValue, bool), ()> {
    let previous_block = runtime.block_type;
    runtime.block_type = BlockType::Known;
    let function_body = program_archive.get_function_data(id).get_body_as_vec();
    let (function_result, can_be_simplified) = execute_sequence_of_statements(
        function_body,
        program_archive,
        runtime,
        &mut Option::None,
        flags,
        true,
    )?;
    runtime.block_type = previous_block;
    let return_value = function_result.unwrap();
    debug_assert!(FoldedValue::valid_arithmetic_slice(&return_value));
    Result::Ok((return_value, can_be_simplified))
}

fn execute_template_call(
    id: &str,
    parameter_values: Vec<AExpressionSlice>,
    tag_values: BTreeMap<String, TagInfo>,
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    flags: FlagsExecution,
) -> Result<FoldedValue, ()> {
    debug_assert!(runtime.block_type == BlockType::Known);
    let is_main = std::mem::replace(&mut runtime.public_inputs, vec![]);
    let is_parallel = program_archive.get_template_data(id).is_parallel();
    let is_custom_gate = program_archive.get_template_data(id).is_custom_gate();
    let args_names = program_archive.get_template_data(id).get_name_of_params();
    let template_body = program_archive.get_template_data(id).get_body_as_vec();
    let mut args_to_values = BTreeMap::new();
    debug_assert_eq!(args_names.len(), parameter_values.len());
    let mut instantiation_name = format!("{}(", id);
    let mut not_empty_name = false;
    for (name, value) in args_names.iter().zip(parameter_values) {
        instantiation_name.push_str(&format!("{},", value.to_string()));
        not_empty_name = true;
        args_to_values.insert(name.clone(), value.clone());
    }
    for (_input, input_tags) in &tag_values {
        for (_tag, value) in input_tags {
            if value.is_none() {
                instantiation_name.push_str("null,");
            } else {
                let value = value.clone().unwrap();
                instantiation_name.push_str(&format!("{},", value.to_string()));
            }
            not_empty_name = true;
        }
    }

    if not_empty_name {
        instantiation_name.pop();
    }
    instantiation_name.push(')');
    let existent_node = runtime
        .exec_program
        .identify_node(id, &args_to_values, &tag_values);
    let node_pointer = if let Option::Some(pointer) = existent_node {
        pointer
    } else {
        let analysis =
            std::mem::replace(&mut runtime.analysis, Analysis::new(program_archive.id_max));
        let code = program_archive.get_template_data(id).get_body().clone();
        let mut node_wrap = Option::Some(ExecutedTemplate::new(
            is_main,
            id.to_string(),
            instantiation_name,
            args_to_values,
            tag_values,
            code,
            is_parallel,
            is_custom_gate,
        ));
        let (ret, _) = execute_sequence_of_statements(
            template_body,
            program_archive,
            runtime,
            &mut node_wrap,
            flags,
            true,
        )?;
        debug_assert!(ret.is_none());

        let result_check_components =
            environment_check_all_components_assigned(&runtime.environment);
        match result_check_components {
            Err((error, meta)) => {
                treat_result_with_memory_error_void(
                    Err(error),
                    &meta,
                    &mut runtime.runtime_errors,
                    &runtime.call_trace,
                )?;
            }
            Ok(_) => {}
        }

        let new_node = node_wrap.unwrap();
        let analysis = std::mem::replace(&mut runtime.analysis, analysis);
        let node_pointer = runtime.exec_program.add_node_to_scheme(new_node, analysis);
        node_pointer
    };
    Result::Ok(FoldedValue {
        node_pointer: Option::Some(node_pointer),
        is_parallel: Option::Some(false),
        ..FoldedValue::default()
    })
}

fn preexecute_template_call(
    id: &str,
    parameter_values: &[AExpressionSlice],
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
) -> Result<FoldedValue, ()> {
    debug_assert!(runtime.block_type == BlockType::Known);
    let inputs = program_archive.get_template_data(id).get_inputs();
    let outputs = program_archive.get_template_data(id).get_outputs();

    let mut inputs_to_tags = HashMap::new();
    let mut outputs_to_tags = HashMap::new();

    for (name, info_input) in inputs {
        inputs_to_tags.insert(name.clone(), info_input.1.clone());
    }

    for (name, info_output) in outputs {
        outputs_to_tags.insert(name.clone(), info_output.1.clone());
    }

    let node_wrap = Option::Some(PreExecutedTemplate::new(
        id.to_string(),
        parameter_values.to_vec(),
        inputs_to_tags,
        outputs_to_tags,
    ));

    let new_node = node_wrap.unwrap();
    let node_pointer = runtime.exec_program.add_prenode_to_scheme(new_node);
    Result::Ok(FoldedValue {
        node_pointer: Option::Some(node_pointer),
        is_parallel: Option::Some(false),
        ..FoldedValue::default()
    })
}

fn execute_infix_op(
    meta: &Meta,
    infix: ExpressionInfixOpcode,
    l_value: &AExpr,
    r_value: &AExpr,
    runtime: &mut RuntimeInformation,
) -> Result<AExpr, ()> {
    use ExpressionInfixOpcode::*;
    let field = runtime.constants.get_p();
    let possible_result = match infix {
        Mul => Result::Ok(AExpr::mul(l_value, r_value, field)),
        Div => AExpr::div(l_value, r_value, field),
        Add => Result::Ok(AExpr::add(l_value, r_value, field)),
        Sub => Result::Ok(AExpr::sub(l_value, r_value, field)),
        Pow => Result::Ok(AExpr::pow(l_value, r_value, field)),
        IntDiv => AExpr::idiv(l_value, r_value, field),
        Mod => AExpr::mod_op(l_value, r_value, field),
        ShiftL => AExpr::shift_l(l_value, r_value, field),
        ShiftR => AExpr::shift_r(l_value, r_value, field),
        LesserEq => Result::Ok(AExpr::lesser_eq(l_value, r_value, field)),
        GreaterEq => Result::Ok(AExpr::greater_eq(l_value, r_value, field)),
        Lesser => Result::Ok(AExpr::lesser(l_value, r_value, field)),
        Greater => Result::Ok(AExpr::greater(l_value, r_value, field)),
        Eq => Result::Ok(AExpr::eq(l_value, r_value, field)),
        NotEq => Result::Ok(AExpr::not_eq(l_value, r_value, field)),
        BoolOr => Result::Ok(AExpr::bool_or(l_value, r_value, field)),
        BoolAnd => Result::Ok(AExpr::bool_and(l_value, r_value, field)),
        BitOr => Result::Ok(AExpr::bit_or(l_value, r_value, field)),
        BitAnd => Result::Ok(AExpr::bit_and(l_value, r_value, field)),
        BitXor => Result::Ok(AExpr::bit_xor(l_value, r_value, field)),
    };
    treat_result_with_arithmetic_error(
        possible_result,
        meta,
        &mut runtime.runtime_errors,
        &runtime.call_trace,
    )
}

fn execute_prefix_op(
    prefix_op: ExpressionPrefixOpcode,
    value: &AExpr,
    runtime: &mut RuntimeInformation,
) -> Result<AExpr, ()> {
    use ExpressionPrefixOpcode::*;
    let field = runtime.constants.get_p();
    let result = match prefix_op {
        BoolNot => AExpr::not(value, field),
        Sub => AExpr::prefix_sub(value, field),
        Complement => AExpr::complement_256(value, field),
    };
    Result::Ok(result)
}

//************************************************* Indexing support *************************************************

/*
Returns (A,B,C) where:
    A = indexes before a component access as arithmetic expressions
    B = possible signal accessed
    C = index where the signal is accessed, C == access.len() if there is none
*/
fn treat_indexing(
    start: usize,
    access: &[Access],
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    actual_node: &mut Option<ExecutedTemplate>,
    flags: FlagsExecution,
) -> Result<(Vec<AExpr>, Option<String>, usize), ()> {
    let mut index_accesses = Vec::new();
    let mut signal_name = Option::None;
    let mut act = start;
    loop {
        if act >= access.len() {
            break;
        }
        match &access[act] {
            Access::ArrayAccess(index) => {
                let index_fold =
                    execute_expression(index, program_archive, runtime, actual_node, flags)?;
                let index_arithmetic_expression =
                    safe_unwrap_to_single_arithmetic_expression(index_fold, line!()).aexpr;
                index_accesses.push(index_arithmetic_expression);
                act += 1;
            }
            Access::ComponentAccess(name) => {
                signal_name = Option::Some(name.clone());
                break;
            }
        }
    }
    Result::Ok((index_accesses, signal_name, act))
}

/*
    ae_indexes are a valid indexing when
    all Number values fit in usize
*/
fn valid_indexing(ae_indexes: &[AExpr]) -> Result<(), MemoryError> {
    for ae_index in ae_indexes {
        if ae_index.is_number() && AExpr::get_usize(ae_index).is_none() {
            return Result::Err(MemoryError::OutOfBoundsError);
        }
    }
    Result::Ok(())
}

fn valid_array_declaration(ae_indexes: &[AExpr]) -> Result<(), MemoryError> {
    for ae_index in ae_indexes {
        if !ae_index.is_number() {
            return Result::Err(MemoryError::UnknownSizeDimension);
        }
    }
    Result::Ok(())
}

/*
    ae_indexes Numbers MUST fit in usize,
    this function must be call just
    if valid_indexing does not return
    Result::Err(..)
*/
fn cast_indexing(ae_indexes: &[AExpr]) -> Option<Vec<SliceCapacity>> {
    let mut sc_indexes = Vec::new();
    for ae_index in ae_indexes.iter() {
        if !ae_index.is_number() {
            return Option::None;
        }
        match AExpr::get_usize(ae_index) {
            Some(index) => {
                sc_indexes.push(index);
            }
            None => {
                return Option::None;
            }
        }
    }
    Option::Some(sc_indexes)
}

fn cast_index(ae_index: &AExpr) -> Option<SliceCapacity> {
    if !ae_index.is_number() {
        return Option::None;
    }
    match AExpr::get_usize(ae_index) {
        Some(index) => Option::Some(index),
        None => Option::None,
    }
}

/*
    Usable representation of a series of accesses performed over a symbol.
    AccessingInformation {
        pub undefined: bool ===> true if one of the index values could not be transformed into a SliceCapacity during the process,
        pub before_signal: Vec<SliceCapacity>,
        pub signal_access: Option<String> ==> may not appear,
        pub after_signal: Vec<SliceCapacity>
        pub tag_access: Option<String> ==> may not appear,
    }
*/
struct AccessingInformation {
    pub undefined: bool,
    pub before_signal: Vec<SliceCapacity>,
    pub signal_access: Option<String>,
    pub after_signal: Vec<SliceCapacity>,
    pub tag_access: Option<String>,
}
fn treat_accessing(
    meta: &Meta,
    access: &[Access],
    program_archive: &ProgramArchive,
    runtime: &mut RuntimeInformation,
    actual_node: &mut Option<ExecutedTemplate>,
    flags: FlagsExecution,
) -> Result<AccessingInformation, ()> {
    let (ae_before_signal, signal_name, signal_index) =
        treat_indexing(0, access, program_archive, runtime, actual_node, flags)?;
    let (ae_after_signal, tag_name, _tag_index) = treat_indexing(
        signal_index + 1,
        access,
        program_archive,
        runtime,
        actual_node,
        flags,
    )?;
    treat_result_with_memory_error(
        valid_indexing(&ae_before_signal),
        meta,
        &mut runtime.runtime_errors,
        &runtime.call_trace,
    )?;
    treat_result_with_memory_error(
        valid_indexing(&ae_after_signal),
        meta,
        &mut runtime.runtime_errors,
        &runtime.call_trace,
    )?;

    let possible_before_indexing = cast_indexing(&ae_before_signal);
    let possible_after_indexing = cast_indexing(&ae_after_signal);

    let undefined = possible_before_indexing.is_none() || possible_after_indexing.is_none();
    let signal_access = signal_name;
    let tag_access = tag_name;
    let (before_signal, after_signal) = if !undefined {
        (
            possible_before_indexing.unwrap(),
            possible_after_indexing.unwrap(),
        )
    } else {
        (Vec::new(), Vec::new())
    };
    Result::Ok(AccessingInformation {
        undefined,
        before_signal,
        after_signal,
        signal_access,
        tag_access,
    })
}

//************************************************* Safe transformations *************************************************

fn safe_unwrap_to_single_arithmetic_expression(
    folded_value: FoldedValue,
    line: u32,
) -> WrappedAExpr {
    let slice_result = safe_unwrap_to_arithmetic_slice(folded_value, line);
    safe_unwrap_to_single(slice_result, line)
}
fn safe_unwrap_to_arithmetic_slice(folded_value: FoldedValue, line: u32) -> AExpressionSlice {
    debug_assert!(
        FoldedValue::valid_arithmetic_slice(&folded_value),
        "Caused by call at {}",
        line
    );
    folded_value.arithmetic_slice.unwrap()
}
fn safe_unwrap_to_valid_node_pointer(folded_value: FoldedValue, line: u32) -> (NodePointer, bool) {
    debug_assert!(
        FoldedValue::valid_node_pointer(&folded_value),
        "Caused by call at {}",
        line
    );
    (
        folded_value.node_pointer.unwrap(),
        folded_value.is_parallel.unwrap(),
    )
}
fn safe_unwrap_to_single<C: Clone>(slice: MemorySlice<C>, line: u32) -> C {
    debug_assert!(slice.is_single(), "Caused by call at {}", line);
    MemorySlice::unwrap_to_single(slice)
}

//************************************************* Result handling *************************************************

fn treat_result_with_arithmetic_error<C>(
    arithmetic_error: Result<C, ArithmeticError>,
    meta: &Meta,
    runtime_errors: &mut ReportCollection,
    call_trace: &[String],
) -> Result<C, ()> {
    use ReportCode::RuntimeError;
    match arithmetic_error {
        Result::Ok(c) => Result::Ok(c),
        Result::Err(arithmetic_error) => {
            let report = match arithmetic_error {
                ArithmeticError::DivisionByZero => {
                    Report::error("Division by zero".to_string(), RuntimeError)
                }
                ArithmeticError::BitOverFlowInShift => {
                    Report::error("Shifting caused bit overflow".to_string(), RuntimeError)
                }
            };
            add_report_to_runtime(report, meta, runtime_errors, call_trace);
            Result::Err(())
        }
    }
}

fn treat_result_with_memory_error_void(
    memory_error: Result<(), MemoryError>,
    meta: &Meta,
    runtime_errors: &mut ReportCollection,
    call_trace: &[String],
) -> Result<(), ()> {
    use ReportCode::RuntimeError;
    match memory_error {
        Result::Ok(()) => Result::Ok(()),
        Result::Err(MemoryError::MismatchedDimensionsWeak(dim_given, dim_original)) => {
            let report = Report::warning(
                        format!("Typing warning: Mismatched dimensions, assigning to an array an expression of smaller length, the remaining positions are not modified. Initially all variables are initialized to 0.\n  Expected length: {}, given {}",
                            dim_original, dim_given),
                        RuntimeError);
            add_report_to_runtime(report, meta, runtime_errors, call_trace);
            Ok(())
        }
        Result::Err(memory_error) => {
            let report = match memory_error {
                MemoryError::InvalidAccess(type_invalid_access) => {
                    match type_invalid_access{
                        TypeInvalidAccess::MissingInputs(input) =>{
                            Report::error(
                                format!("Exception caused by invalid access: trying to access to an output signal of a component with not all its inputs initialized.\n Missing input: {}",
                                    input),
                                RuntimeError)
                        },
                        TypeInvalidAccess::MissingInputTags(input) =>{
                            Report::error(
                                format!("Exception caused by invalid access: trying to access to a signal of a component with not all its inputs with tags initialized.\n Missing input (with tags): {}",
                                    input),
                                RuntimeError)
                        },
                        TypeInvalidAccess::NoInitializedComponent =>{
                            Report::error("Exception caused by invalid access: trying to access to a component that is not initialized" .to_string(),
                                RuntimeError)
                        },
                        TypeInvalidAccess::NoInitializedSignal =>{
                            Report::error("Exception caused by invalid access: trying to access to a signal that is not initialized" .to_string(),
                                RuntimeError)
                        }
                    }
                }
                MemoryError::AssignmentError(type_asig_error) => {
                    match type_asig_error{
                        TypeAssignmentError::MultipleAssignments =>{
                            Report::error("Exception caused by invalid assignment: signal already assigned".to_string(),
                                RuntimeError)
                        },
                        TypeAssignmentError::AssignmentInput(signal) => Report::error(
                            format!("Invalid assignment: input signals of a template already have a value when the template is executed and cannot be re-assigned. \n Problematic input signal: {}",
                                signal),
                            RuntimeError,
                        ),
                        TypeAssignmentError::AssignmentOutput =>{
                            Report::error("Exception caused by invalid assignment: trying to assign a value to an output signal of a component".to_string(),
                                RuntimeError)
                        },
                        TypeAssignmentError::NoInitializedComponent =>{
                            Report::error("Exception caused by invalid assignment: trying to assign a value to a signal of a component that has not been initialized".to_string(),
                                RuntimeError)
                        }
                    }
                },
                MemoryError::OutOfBoundsError => {
                    Report::error("Out of bounds exception".to_string(), RuntimeError)
                },
                MemoryError::MismatchedDimensions(given, orig) => {
                    Report::error(
                        format!("Typing error found: mismatched dimensions.\n Expected length: {}, given {}",
                            orig, given),
                         RuntimeError)
                },

                MemoryError::UnknownSizeDimension => {
                    Report::error("Array dimension with unknown size".to_string(), RuntimeError)
                },
                MemoryError::AssignmentMissingTags(signal, tag) => Report::error(
                    format!("Invalid assignment: missing tags required by input signal. \n Missing tag: input signal {} requires tag {}",
                            signal, tag),
                    RuntimeError,
                ),
                MemoryError::AssignmentTagAfterInit => Report::error(
                    "Invalid assignment: tags cannot be assigned to a signal already initialized".to_string(),
                    RuntimeError,
                ),
                MemoryError::AssignmentTagTwice => Report::error(
                    "Invalid assignment: this tag already got a value".to_string(),
                    RuntimeError,
                ),
                MemoryError::AssignmentTagInputTwice(signal, tag) => Report::error(
                    format!("Invalid assignment: tags required by the input signal always have to have the same value. \n Problematic tag: input signal {} already has a different value for tag {}",
                        signal, tag),
                    RuntimeError,
                ),
                MemoryError::AssignmentTagInput => Report::error(
                    "Invalid assignment: this tag belongs to an input which already got a value".to_string(),
                    RuntimeError,
                ),
                MemoryError::MismatchedDimensionsWeak(..) => unreachable!(),
                MemoryError::TagValueNotInitializedAccess => Report::error(
                    "Tag value has not been previously initialized".to_string(), 
                    RuntimeError,
                ),
                MemoryError::MissingInputs(name) => Report::error(
                    format!("Component {} is created but not all its inputs are initialized", name),
                    RuntimeError,
                ),
            };
            add_report_to_runtime(report, meta, runtime_errors, call_trace);
            Result::Err(())
        }
    }
}

fn treat_result_with_memory_error<C>(
    memory_error: Result<C, MemoryError>,
    meta: &Meta,
    runtime_errors: &mut ReportCollection,
    call_trace: &[String],
) -> Result<C, ()> {
    use ReportCode::RuntimeError;
    match memory_error {
        Result::Ok(c) => Result::Ok(c),
        Result::Err(memory_error) => {
            let report = match memory_error {
                MemoryError::InvalidAccess(type_invalid_access) => {
                    match type_invalid_access{
                        TypeInvalidAccess::MissingInputs(input) =>{
                            Report::error(
                                format!("Exception caused by invalid access: trying to access to an output signal of a component with not all its inputs initialized.\n Missing input: {}",
                                    input),
                                RuntimeError)
                        },
                        TypeInvalidAccess::MissingInputTags(input) =>{
                            Report::error(
                                format!("Exception caused by invalid access: trying to access to a signal of a component with not all its inputs with tags initialized.\n Missing input (with tags): {}",
                                    input),
                                RuntimeError)
                        },
                        TypeInvalidAccess::NoInitializedComponent =>{
                            Report::error("Exception caused by invalid access: trying to access to a component that is not initialized" .to_string(),
                                RuntimeError)
                        },
                        TypeInvalidAccess::NoInitializedSignal =>{
                            Report::error("Exception caused by invalid access: trying to access to a signal that is not initialized" .to_string(),
                                RuntimeError)
                        }
                    }
                },
                MemoryError::AssignmentError(type_asig_error) => {
                    match type_asig_error{
                        TypeAssignmentError::MultipleAssignments =>{
                            Report::error("Exception caused by invalid assignment: signal already assigned".to_string(),
                                RuntimeError)
                        },
                        TypeAssignmentError::AssignmentInput(signal) => Report::error(
                            format!("Invalid assignment: input signals of a template already have a value when the template is executed and cannot be re-assigned. \n Problematic input signal: {}",
                                signal),
                            RuntimeError,
                        ),
                        TypeAssignmentError::AssignmentOutput =>{
                            Report::error("Exception caused by invalid assignment: trying to assign a value to an output signal of a component".to_string(),
                                RuntimeError)
                        },
                        TypeAssignmentError::NoInitializedComponent =>{
                            Report::error("Exception caused by invalid assignment: trying to assign a value to a signal of a component that has not been initialized".to_string(),
                                RuntimeError)
                        }
                    }
                },
                MemoryError::AssignmentMissingTags(signal, tag) => Report::error(
                    format!("Invalid assignment: missing tags required by input signal. \n Missing tag: input signal {} requires tag {}",
                            signal, tag),
                    RuntimeError,
                ),
                MemoryError::AssignmentTagAfterInit => Report::error(
                    "Invalid assignment: tags cannot be assigned to a signal already initialized".to_string(),
                    RuntimeError,
                ),
                MemoryError::AssignmentTagTwice => Report::error(
                    "Invalid assignment: this tag already got a value".to_string(),
                    RuntimeError,
                ),
                MemoryError::AssignmentTagInputTwice(signal, tag) => Report::error(
                    format!("Invalid assignment: tags required by the input signal always have to have the same value. \n Problematic tag: input signal {} already has a different value for tag {}",
                        signal, tag),
                    RuntimeError,
                ),
                MemoryError::AssignmentTagInput => Report::error(
                    "Invalid assignment: this tag belongs to an input which already got a value".to_string(),
                    RuntimeError,
                ),
                MemoryError::OutOfBoundsError => {
                    Report::error("Out of bounds exception".to_string(), RuntimeError)
                },
                MemoryError::MismatchedDimensions(given, orig) => {
                    Report::error(
                        format!("Typing error found: mismatched dimensions.\n Expected length: {}, given {}",
                            orig, given),
                         RuntimeError)
                },
                MemoryError::UnknownSizeDimension => {
                    Report::error("Array dimension with unknown size".to_string(), RuntimeError)
                }
                MemoryError::TagValueNotInitializedAccess => {
                    Report::error("Tag value has not been previously initialized".to_string(), RuntimeError)

                }
                MemoryError::MismatchedDimensionsWeak(..) => {
                    unreachable!()
                },
                MemoryError::MissingInputs(name) => Report::error(
                    format!("Component {} is created but not all its inputs are initialized", name),
                    RuntimeError,
                )
            };
            add_report_to_runtime(report, meta, runtime_errors, call_trace);
            Result::Err(())
        }
    }
}

fn treat_result_with_environment_error<C>(
    environment_error: Result<C, ExecutionEnvironmentError>,
    meta: &Meta,
    runtime_errors: &mut ReportCollection,
    call_trace: &[String],
) -> Result<C, ()> {
    use ReportCode::*;
    match environment_error {
        Result::Ok(c) => Result::Ok(c),
        Result::Err(environment_error) => {
            let report = match environment_error {
                ExecutionEnvironmentError::NonExistentSymbol => {
                    Report::error("Accessing non existent symbol".to_string(), RuntimeError)
                }
            };
            add_report_to_runtime(report, meta, runtime_errors, call_trace);
            Result::Err(())
        }
    }
}

fn treat_result_with_execution_error<C>(
    execution_error: Result<C, ExecutionError>,
    meta: &Meta,
    runtime_errors: &mut ReportCollection,
    call_trace: &[String],
) -> Result<C, ()> {
    use ExecutionError::*;
    match execution_error {
        Result::Ok(c) => Result::Ok(c),
        Result::Err(execution_error) => {
            let report = match execution_error {
                NonQuadraticConstraint => Report::error(
                    "Non quadratic constraints are not allowed!".to_string(),
                    ReportCode::RuntimeError,
                ),
                UnknownTemplate => Report::error(
                    "Every component instantiation must be resolved during the constraint generation phase. This component declaration uses a value that can be unknown during the constraint generation phase.".to_string(),
                    ReportCode::RuntimeError,
                ),
                NonValidTagAssignment => Report::error(
                    "Tags cannot be assigned to values that can be unknown during the constraint generation phase".to_string(),
                    ReportCode::RuntimeError,
                ),
                FalseAssert => {
                    Report::error("False assert reached".to_string(), ReportCode::RuntimeError)
                }
                ArraySizeTooBig => Report::error(
                    "The size of the array is expected to be a usize".to_string(),
                    ReportCode::RuntimeError,
                ),
                ConstraintInUnknown => Report::error(
                    "There are constraints depending on the value of a condition that can be unknown during the constraint generation phase".to_string(),
                    ReportCode::RuntimeError,
                ),
                DeclarationInUnknown => Report::error(
                    "There are signal or component declarations depending on the value of a condition that can be unknown during the constraint generation phase".to_string(),
                    ReportCode::RuntimeError,
                ),
                TagAssignmentInUnknown => Report::error(
                    "There are tag assignments depending on the value of a condition that can be unknown during the constraint generation phase".to_string(),
                    ReportCode::RuntimeError,
                )
            };
            add_report_to_runtime(report, meta, runtime_errors, call_trace);
            Result::Err(())
        }
    }
}

fn treat_result_with_execution_warning<C>(
    execution_error: Result<C, ExecutionWarning>,
    meta: &Meta,
    runtime_errors: &mut ReportCollection,
    call_trace: &[String],
) -> Result<(), ()> {
    use ExecutionWarning::*;
    match execution_error {
        Result::Ok(_) => Result::Ok(()),
        Result::Err(execution_error) => {
            let report = match execution_error {
                CanBeQuadraticConstraintSingle() => {
                    let msg = format!(
                        "Consider using <== instead of <-- to add the corresponding constraint.\n The constraint representing the assignment satisfies the R1CS format and can be added to the constraint system."
                    );
                    Report::warning(msg, ReportCode::RuntimeWarning)
                }
                CanBeQuadraticConstraintMultiple(positions) => {
                    let mut msg_positions = positions[0].clone();
                    for i in 1..positions.len() {
                        msg_positions = format!("{}, {}", msg_positions, positions[i].clone())
                    }

                    let msg = format!(
                        "Consider using <== instead of <-- for some of positions of the array of signals being assigned.\n The constraints representing the assignment of the positions {} satisfy the R1CS format and can be added to the constraint system.",
                        msg_positions
                    );
                    Report::warning(msg, ReportCode::RuntimeWarning)
                }
            };
            add_report_to_runtime(report, meta, runtime_errors, call_trace);
            Result::Ok(())
        }
    }
}

fn add_report_to_runtime(
    report: Report,
    meta: &Meta,
    runtime_errors: &mut ReportCollection,
    call_trace: &[String],
) {
    let mut report = report;
    report.add_primary(
        meta.location.clone(),
        meta.get_file_id(),
        "found here".to_string(),
    );

    let mut trace = "call trace:\n".to_string();
    let mut spacing = "".to_string();
    for call in call_trace.iter() {
        let msg = format!("{}->{}\n", spacing, call);
        trace.push_str(msg.as_str());
        spacing.push_str(" ");
    }
    report.add_note(trace);
    runtime_errors.push(report);
}
