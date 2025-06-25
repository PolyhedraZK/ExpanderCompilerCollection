//! This module provides the FFI API for the Expander Compiler, especially for the use of the Go language.

use expander_compiler::circuit::config::Config;
use libc::{c_uchar, c_ulong};

/// ABI version for the Expander Compiler.
/// The ABI version is used to ensure compatibility between different versions of the library.
const ABI_VERSION: c_ulong = 4;

#[macro_export]
macro_rules! match_config_id {
    ($config_id:ident, $inner:ident, $args:tt) => {
        match $config_id {
            x if x == M31Config::CONFIG_ID as u64 => $inner::<M31Config> $args,
            x if x == BN254Config::CONFIG_ID as u64 => $inner::<BN254Config> $args,
            x if x == GF2Config::CONFIG_ID as u64 => $inner::<GF2Config> $args,
            x if x == GoldilocksConfig::CONFIG_ID as u64 => $inner::<GoldilocksConfig> $args,
            _ => Err(format!("unknown config id: {}", $config_id)),
        }
    }
}

pub mod compile;
pub mod proving;

/// This struct represents a byte array used in the FFI API.
#[repr(C)]
pub struct ByteArray {
    data: *mut c_uchar,
    length: c_ulong,
}

/// This function returns the ABI version for the Expander Compiler.
#[no_mangle]
pub extern "C" fn abi_version() -> c_ulong {
    ABI_VERSION
}
