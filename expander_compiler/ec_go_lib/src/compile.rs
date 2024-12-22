use expander_compiler::circuit::layered::NormalInputType;
use expander_compiler::frontend::WitnessSolver;
use libc::{c_ulong, c_void, malloc};
use std::ptr;
use std::slice;

use expander_compiler::{
    circuit::{config, ir},
    utils::serde::Serde,
};

use super::*;

#[repr(C)]
pub struct CompileResult {
    witness_solver: *mut c_void,
    layered: ByteArray,
    error: ByteArray,
}

fn compile_inner_with_config<C>(ir_source: Vec<u8>) -> Result<(*mut c_void, Vec<u8>), String>
where
    C: config::Config,
{
    let ir_source = ir::source::RootCircuit::<C>::deserialize_from(&ir_source[..])
        .map_err(|e| format!("failed to deserialize the source circuit: {}", e))?;
    let (ir_witness_gen, layered) =
        expander_compiler::compile::compile::<_, NormalInputType>(&ir_source)
            .map_err(|e| e.to_string())?;
    let mut layered_s: Vec<u8> = Vec::new();
    layered
        .serialize_into(&mut layered_s)
        .map_err(|e| format!("failed to serialize the layered circuit: {}", e))?;
    let witness_solver = WitnessSolver {
        circuit: ir_witness_gen,
    };
    let witness_solver: BoxBoxed = Box::new(Box::new(witness_solver));
    let witness_solver_ptr = Box::into_raw(witness_solver) as *mut c_void;
    Ok((witness_solver_ptr, layered_s))
}

fn compile_inner(ir_source: Vec<u8>, config_id: u64) -> Result<(*mut c_void, Vec<u8>), String> {
    match_config_id!(config_id, compile_inner_with_config, (ir_source))
}

fn to_compile_result(result: Result<(*mut c_void, Vec<u8>), String>) -> CompileResult {
    match result {
        Ok((ir_witness_gen, layered)) => {
            let layered_len = layered.len();
            let layered_ptr = if layered_len > 0 {
                unsafe {
                    let ptr = malloc(layered_len) as *mut u8;
                    ptr.copy_from(layered.as_ptr(), layered_len);
                    ptr
                }
            } else {
                ptr::null_mut()
            };
            CompileResult {
                witness_solver: ir_witness_gen,
                layered: ByteArray {
                    data: layered_ptr,
                    length: layered_len as c_ulong,
                },
                error: ByteArray {
                    data: ptr::null_mut(),
                    length: 0,
                },
            }
        }
        Err(error) => {
            let error_len = error.len();
            let error_ptr = if error_len > 0 {
                unsafe {
                    let ptr = malloc(error_len) as *mut u8;
                    ptr.copy_from(error.as_ptr(), error_len);
                    ptr
                }
            } else {
                ptr::null_mut()
            };
            CompileResult {
                witness_solver: ptr::null_mut(),
                layered: ByteArray {
                    data: ptr::null_mut(),
                    length: 0,
                },
                error: ByteArray {
                    data: error_ptr,
                    length: error_len as c_ulong,
                },
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn compile(ir_source: ByteArray, config_id: c_ulong) -> CompileResult {
    let ir_source = unsafe { slice::from_raw_parts(ir_source.data, ir_source.length as usize) };
    let result = compile_inner(ir_source.to_vec(), config_id);
    to_compile_result(result)
}
