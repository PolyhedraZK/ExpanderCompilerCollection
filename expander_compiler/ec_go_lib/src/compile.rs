use libc::{c_ulong, malloc};
use std::ptr;
use std::slice;

use expander_compiler::circuit::layered::NormalInputType;
use expander_compiler::circuit::{config, ir};
use serdes::ExpSerde;

use super::{match_config_id, ByteArray, Config};

#[repr(C)]
pub struct CompileResult {
    ir_witness_gen: ByteArray,
    layered: ByteArray,
    error: ByteArray,
}

fn compile_inner_with_config<C>(ir_source: Vec<u8>) -> Result<(Vec<u8>, Vec<u8>), String>
where
    C: config::Config,
{
    let ir_source = ir::source::RootCircuit::<C>::deserialize_from(&ir_source[..])
        .map_err(|e| format!("failed to deserialize the source circuit: {}", e))?;
    let (ir_witness_gen, layered) =
        expander_compiler::compile::compile::<_, NormalInputType>(&ir_source)
            .map_err(|e| e.to_string())?;
    let mut ir_wg_s: Vec<u8> = Vec::new();
    ir_witness_gen
        .serialize_into(&mut ir_wg_s)
        .map_err(|e| format!("failed to serialize the witness generator: {}", e))?;
    let mut layered_s: Vec<u8> = Vec::new();
    layered
        .serialize_into(&mut layered_s)
        .map_err(|e| format!("failed to serialize the layered circuit: {}", e))?;
    Ok((ir_wg_s, layered_s))
}

fn compile_inner(ir_source: Vec<u8>, config_id: u64) -> Result<(Vec<u8>, Vec<u8>), String> {
    match_config_id!(config_id, compile_inner_with_config, (ir_source))
}

fn to_compile_result(result: Result<(Vec<u8>, Vec<u8>), String>) -> CompileResult {
    match result {
        Ok((ir_witness_gen, layered)) => {
            let ir_wg_len = ir_witness_gen.len();
            let layered_len = layered.len();
            let ir_wg_ptr = if ir_wg_len > 0 {
                unsafe {
                    let ptr = malloc(ir_wg_len) as *mut u8;
                    ptr.copy_from(ir_witness_gen.as_ptr(), ir_wg_len);
                    ptr
                }
            } else {
                ptr::null_mut()
            };
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
                ir_witness_gen: ByteArray {
                    data: ir_wg_ptr,
                    length: ir_wg_len as c_ulong,
                },
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
                ir_witness_gen: ByteArray {
                    data: ptr::null_mut(),
                    length: 0,
                },
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
