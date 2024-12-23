use libc::{c_ulong, c_void};
use std::slice;

use expander_compiler::{circuit::config, utils::serde::Serde};

use super::*;

fn load_field_array_inner<C: config::Config>(
    mut data: &[u8],
    len: u64,
) -> Result<*mut c_void, String> {
    let mut res = Vec::<C::CircuitField>::with_capacity(len as usize);
    for _ in 0..len {
        let field = C::CircuitField::deserialize_from(&mut data)
            .map_err(|e| format!("failed to load the field array: {}", e))?;
        res.push(field);
    }
    if !data.is_empty() {
        return Err("buffer has extra data".to_string());
    }
    let res: BoxBoxed = Box::new(Box::new(res));
    Ok(Box::into_raw(res) as *mut c_void)
}

#[no_mangle]
pub extern "C" fn load_field_array(
    data: ByteArray,
    len: c_ulong,
    config_id: c_ulong,
) -> PointerResult {
    let data = unsafe { slice::from_raw_parts(data.data, data.length as usize) };
    let result = match_config_id!(config_id, load_field_array_inner, (data, len));
    result.into()
}

fn dump_field_array_inner<C: config::Config>(
    pointer: *mut c_void,
    res_length: *mut c_ulong,
) -> Result<*mut c_void, String> {
    let pointer: BoxBoxed = unsafe { Box::from_raw(pointer as *mut Boxed) };
    let mut data = Vec::new();
    let res = (|| {
        let arr = match pointer.downcast_ref::<Vec<C::CircuitField>>() {
            Some(arr) => arr,
            None => return Err("failed to downcast the field array".to_string()),
        };
        for x in arr.iter() {
            x.serialize_into(&mut data)
                .map_err(|e| format!("failed to dump the field array: {}", e))?;
        }

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
pub extern "C" fn dump_field_array(
    pointer: *mut c_void,
    res_length: *mut c_ulong,
    config_id: c_ulong,
) -> PointerResult {
    let result = match_config_id!(config_id, dump_field_array_inner, (pointer, res_length));
    result.into()
}
