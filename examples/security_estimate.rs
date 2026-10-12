//! Print the attack cost table for every parameter set.
//!
//!   cargo run --release --example security_estimate
use eigen_seta::security::{estimate, render};
use eigen_seta::sets::set_info;
use eigen_seta::wire::SetId;

fn main() {
    println!("Eigen-Seta attack cost model (log2; see docs/SECURITY.md for the formulas)\n");
    for id in SetId::ALL {
        let s = set_info(id);
        println!("{}", render(&s, &estimate(&s)));
    }
}
