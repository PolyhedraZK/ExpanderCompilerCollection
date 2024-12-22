use libc::{c_ulong, c_void};
use std::slice;

use expander_compiler::{circuit::config, utils::serde::Serde};

use super::*;

fn load_field_array_inner<C: config::Config>(data: &[u8]) -> Result<*mut c_void, String> {
    let res = Vec::<C::CircuitField>::deserialize_from(data)
        .map_err(|e| format!("failed to load the field array: {}", e))?;
    let res: BoxBoxed = Box::new(Box::new(res));
    Ok(Box::into_raw(res) as *mut c_void)
}

#[no_mangle]
pub extern "C" fn load_field_array(data: ByteArray, config_id: c_ulong) -> PointerResult {
    let data = unsafe { slice::from_raw_parts(data.data, data.length as usize) };
    let result = match_config_id!(config_id, load_field_array_inner, (data));
    result.into()
}

fn dump_field_array_inner<C: config::Config>(
    pointer: *mut c_void,
    mut data: &mut [u8],
) -> Result<*mut c_void, String> {
    // TODO: fix reading
    let pointer = pointer as *mut Boxed;
    let pointer = unsafe { &mut *pointer };
    let res = match pointer.downcast_mut::<Vec<C::CircuitField>>() {
        Some(res) => res,
        None => return Err("failed to downcast the field array".to_string()),
    };
    res.serialize_into(&mut data)
        .map_err(|e| format!("failed to dump the field array: {}", e))?;
    if !data.is_empty() {
        return Err("buffer too big".to_string());
    }
    Ok(std::ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn dump_field_array(
    pointer: *mut c_void,
    data: ByteArray,
    config_id: c_ulong,
) -> PointerResult {
    let mut data = unsafe { slice::from_raw_parts_mut(data.data, data.length as usize) };
    let result = match_config_id!(config_id, dump_field_array_inner, (pointer, &mut data));
    result.into()
}
