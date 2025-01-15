use expander_compiler::circuit::config::Config;
use libc::{c_uchar, c_ulong};

const ABI_VERSION: c_ulong = 4;

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

pub mod compile;
pub mod proving;

#[repr(C)]
pub struct ByteArray {
    data: *mut c_uchar,
    length: c_ulong,
}

#[no_mangle]
pub extern "C" fn abi_version() -> c_ulong {
    ABI_VERSION
}
