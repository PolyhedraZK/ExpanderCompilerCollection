use expander_compiler::{
    circuit::config, circuit::ir::hint_normalized::witness_solver::WitnessSolver, field::Field,
    hints::registry::HintCaller, utils::error::Error, utils::serde::Serde,
};
use libc::{c_ulong, c_void, malloc};
use std::slice;

use super::*;

fn load_witness_solver_inner<C: config::Config>(
    witness_solver: &[u8],
) -> Result<*mut c_void, String> {
    let witness_solver = WitnessSolver::<C>::deserialize_from(witness_solver)
        .map_err(|e| format!("failed to deserialize the witness solver: {}", e))?;
    let witness_solver: BoxBoxed = Box::new(Box::new(witness_solver));
    Ok(Box::into_raw(witness_solver) as *mut c_void)
}

#[no_mangle]
pub extern "C" fn load_witness_solver(
    witness_solver: ByteArray,
    config_id: c_ulong,
) -> PointerResult {
    let witness_solver =
        unsafe { slice::from_raw_parts(witness_solver.data, witness_solver.length as usize) };
    let result = match_config_id!(config_id, load_witness_solver_inner, (witness_solver));
    result.into()
}

fn dump_witness_solver_inner<C: config::Config>(
    pointer: *mut c_void,
    res_length: *mut c_ulong,
) -> Result<*mut c_void, String> {
    let pointer: BoxBoxed = unsafe { Box::from_raw(pointer as *mut Boxed) };
    let mut data = Vec::new();
    let res = (|| {
        let witness_solver = match pointer.downcast_ref::<WitnessSolver<C>>() {
            Some(witness_solver) => witness_solver,
            None => return Err("failed to downcast the witness solver".to_string()),
        };
        witness_solver
            .serialize_into(&mut data)
            .map_err(|e| format!("failed to dump the witness solver: {}", e))?;

        unsafe {
            res_length.write(data.len() as c_ulong);
            let ptr = malloc(data.len()) as *mut u8;
            ptr.copy_from(data.as_ptr(), data.len());
            Ok(ptr as *mut c_void)
        }
    })();
    let _ = Box::into_raw(pointer);
    res
}

#[no_mangle]
pub extern "C" fn dump_witness_solver(
    pointer: *mut c_void,
    res_length: *mut c_ulong,
    config_id: c_ulong,
) -> PointerResult {
    let result = match_config_id!(config_id, dump_witness_solver_inner, (pointer, res_length));
    result.into()
}

// CallHint(hintId, inputs, outputs) -> err
type GoHintCaller =
    extern "C" fn(c_ulong, *mut c_uchar, c_ulong, *mut c_uchar, c_ulong, c_ulong) -> *mut c_uchar;

struct GoHintCallerWrapper {
    caller: GoHintCaller,
    config_id: usize,
}

impl GoHintCallerWrapper {
    fn new(caller: GoHintCaller, config_id: usize) -> Self {
        GoHintCallerWrapper { caller, config_id }
    }
}

impl<F: Field> HintCaller<F> for GoHintCallerWrapper {
    fn call(&mut self, id: usize, args: &[F], num_outputs: usize) -> Result<Vec<F>, Error> {
        let mut inputs_vec: Vec<u8> = vec![0; args.len() * F::SIZE];
        let mut outputs_vec: Vec<u8> = vec![0; num_outputs * F::SIZE];
        for i in 0..args.len() {
            args[i]
                .serialize_into(&mut inputs_vec[i * F::SIZE..(i + 1) * F::SIZE])
                .unwrap();
        }
        let result = (self.caller)(
            id as c_ulong,
            inputs_vec.as_mut_ptr(),
            args.len() as c_ulong,
            outputs_vec.as_mut_ptr(),
            num_outputs as c_ulong,
            self.config_id as c_ulong,
        );
        if !result.is_null() {
            // read 0-terminated string
            let mut len = 0;
            while unsafe { *result.offset(len) } != 0 {
                len += 1;
            }
            let slice = unsafe { slice::from_raw_parts(result, len as usize) };
            return Err(Error::UserError(format!(
                "golang hint error: {}",
                std::str::from_utf8(slice).unwrap()
            )));
        }
        let mut res = Vec::with_capacity(num_outputs);
        for i in 0..num_outputs {
            res.push(
                F::deserialize_from(&outputs_vec[i * F::SIZE..(i + 1) * F::SIZE]).map_err(|e| {
                    Error::InternalError(format!("failed to deserialize the hint output: {}", e))
                })?,
            );
        }
        Ok(res)
    }
}

fn solve_witnesses_inner<C: config::Config>(
    witness_solver: *mut c_void,
    raw_inputs: *mut c_void,
    num_witnesses: c_ulong,
    hint_caller: GoHintCaller,
    res_num_inputs_per_witness: *mut c_ulong,
    res_num_public_inputs_per_witness: *mut c_ulong,
) -> Result<*mut c_void, String> {
    let witness_solver_box: BoxBoxed = unsafe { Box::from_raw(witness_solver as *mut Boxed) };
    let raw_inputs_box: BoxBoxed = unsafe { Box::from_raw(raw_inputs as *mut Boxed) };
    let res = (|| {
        let witness_solver = match witness_solver_box.downcast_ref::<WitnessSolver<C>>() {
            Some(witness_solver) => witness_solver,
            None => return Err("failed to downcast the witness solver".to_string()),
        };
        let raw_inputs = match raw_inputs_box.downcast_ref::<Vec<C::CircuitField>>() {
            Some(raw_inputs) => raw_inputs,
            None => return Err("failed to downcast the raw inputs".to_string()),
        };
        let a = witness_solver.circuit.circuits[&0].num_inputs;
        let b = witness_solver.circuit.num_public_inputs;
        if (a + b) * num_witnesses as usize != raw_inputs.len() {
            return Err("invalid number of raw inputs".to_string());
        }
        let witness = witness_solver
            .solve_witnesses_from_raw_inputs(
                num_witnesses as usize,
                |i| {
                    let (x, y) = raw_inputs[(a + b) * i..(a + b) * (i + 1)].split_at(a);
                    (x.to_vec(), y.to_vec())
                },
                &mut GoHintCallerWrapper::new(hint_caller, C::CONFIG_ID),
            )
            .map_err(|e| format!("failed to solve the witnesses: {}", e))?;

        unsafe {
            res_num_inputs_per_witness.write(witness.num_inputs_per_witness as c_ulong);
            res_num_public_inputs_per_witness
                .write(witness.num_public_inputs_per_witness as c_ulong);
        }

        let witness_vals: BoxBoxed = Box::new(Box::new(witness.values));
        Ok(Box::into_raw(witness_vals) as *mut c_void)
    })();
    let _ = Box::into_raw(witness_solver_box);
    let _ = Box::into_raw(raw_inputs_box);
    res
}

#[no_mangle]
pub extern "C" fn solve_witnesses(
    witness_solver: *mut c_void,
    raw_inputs: *mut c_void,
    num_witnesses: c_ulong,
    hint_caller: GoHintCaller,
    config_id: c_ulong,
    res_num_inputs_per_witness: *mut c_ulong,
    res_num_public_inputs_per_witness: *mut c_ulong,
) -> PointerResult {
    let result = match_config_id!(
        config_id,
        solve_witnesses_inner,
        (
            witness_solver,
            raw_inputs,
            num_witnesses,
            hint_caller,
            res_num_inputs_per_witness,
            res_num_public_inputs_per_witness
        )
    );
    result.into()
}
