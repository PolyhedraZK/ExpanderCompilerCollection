// allow range loop for better readability
#![allow(clippy::needless_range_loop)]

mod native;
pub use native::*;

#[cfg(test)]
mod tests;
