mod constants;
mod native;

use constants::N_LIMBS;
use expander_compiler::declare_circuit;
use native::RSAFieldElement;

// A RSA signature verification requires to compute x^e mod n, where
// - e is fixed to 2^16 + 1
// - x and n are both 2048 bits integers
// usually e is the hash of the message to sign -- for now we choose to ignore this part
declare_circuit!(RSACircuit {
    x: [Variable; N_LIMBS],
    n: [Variable; N_LIMBS],
    result: [Variable; 2 * N_LIMBS],
});

// To build this circuit we will need to compute intermediate results:
// e^2, e^4, e^8, e^16, e^32, e^64, e^128, e^256, e^512, e^1024, e^2048
// e^4096, e^8192, e^16384, e^32768, e^65536
pub fn build_rsa_traces(x: &RSAFieldElement, n: &RSAFieldElement) -> [RSAFieldElement; 16] {
    let mut traces = [RSAFieldElement::new([0u128; N_LIMBS]); 16];
    // traces[0] = x.clone();
    // for i in 1..16 {
    //     traces[i] = traces[i - 1].mul(&traces[i - 1], n);
    // }
    traces
}

fn main() {
    println!("Hello, world!");
}
