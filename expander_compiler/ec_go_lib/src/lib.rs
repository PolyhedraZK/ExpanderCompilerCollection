use expander_compiler::circuit::config::Config;
use libc::c_void;
use libc::{c_uchar, c_ulong, malloc};
use std::any::Any;

const ABI_VERSION: c_ulong = 5;

#[macro_export]
macro_rules! match_config_id {
    ($config_id:ident, $inner:ident, $args:tt) => {
        match $config_id {
            x if x == config::M31Config::CONFIG_ID as u64 => $inner::<config::M31Config> $args,
            x if x == config::BN254Config::CONFIG_ID as u64 => $inner::<config::BN254Config> $args,
            x if x == config::GF2Config::CONFIG_ID as u64 => $inner::<config::GF2Config> $args,
            _ => Err(format!("unknown config id: {}", $config_id)),
        }
    }
}

#[macro_export]
macro_rules! match_config_id_panic {
    ($config_id:ident, $inner:ident, $args:tt) => {
        match $config_id {
            x if x == config::M31Config::CONFIG_ID as u64 => $inner::<config::M31Config> $args,
            x if x == config::BN254Config::CONFIG_ID as u64 => $inner::<config::BN254Config> $args,
            x if x == config::GF2Config::CONFIG_ID as u64 => $inner::<config::GF2Config> $args,
            _ => panic!("unknown config id: {}", $config_id),
        }
    }
}

pub mod compile;
pub mod field_array;
pub mod proving;
pub mod witness_solver;

#[repr(C)]
pub struct ByteArray {
    data: *mut c_uchar,
    length: c_ulong,
}

#[repr(C)]
pub struct PointerResult {
    pointer: *mut c_void,
    error: ByteArray,
}

type PointerResultRust = Result<*mut c_void, String>;

impl From<PointerResultRust> for PointerResult {
    fn from(result: PointerResultRust) -> Self {
        match result {
            Ok(pointer) => PointerResult {
                pointer,
                error: ByteArray {
                    data: std::ptr::null_mut(),
                    length: 0,
                },
            },
            Err(error) => {
                let error = error.into_bytes();
                let length = error.len();
                let data = if length > 0 {
                    unsafe {
                        let ptr = malloc(length) as *mut u8;
                        ptr.copy_from(error.as_ptr(), length);
                        ptr
                    }
                } else {
                    std::ptr::null_mut()
                };
                PointerResult {
                    pointer: std::ptr::null_mut(),
                    error: ByteArray {
                        data,
                        length: length as c_ulong,
                    },
                }
            }
        }
    }
}

pub type Boxed = Box<dyn Any>;
pub type BoxBoxed = Box<Boxed>;

#[no_mangle]
pub extern "C" fn free_object(pointer: *mut c_void) {
    unsafe {
        let _ = Box::from_raw(pointer as *mut Boxed);
    }
}

#[no_mangle]
pub extern "C" fn abi_version() -> c_ulong {
    ABI_VERSION
}
