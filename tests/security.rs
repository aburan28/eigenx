//! The cost model must put every named parameter set at or above its
//! declared classical security level, and the quantum-hardened sets at or
//! above it in quantum queries as well.  The toy sets are excluded.

use eigen_seta::security::{
    aes_grover_gates_log2, classical_floor_log2, estimate, quantum_gate_floor_log2,
    quantum_query_floor_log2, render,
};
use eigen_seta::sets::set_info;
use eigen_seta::wire::SetId;

#[test]
fn parameter_sets_meet_their_levels() {
    for id in SetId::ALL {
        let s = set_info(id);
        let costs = estimate(&s);
        println!("{}", render(&s, &costs));
        if s.lambda < 128 {
            continue;
        }
        let cf = classical_floor_log2(&costs);
        assert!(
            cf >= s.lambda as f64,
            "{}: classical floor 2^{cf:.1} below lambda {}",
            s.name,
            s.lambda
        );
        let qf = quantum_query_floor_log2(&costs);
        if s.quantum_hardened {
            assert!(
                qf >= s.lambda as f64,
                "{}: quantum query floor 2^{qf:.1} below lambda {}",
                s.name,
                s.lambda
            );
        } else {
            // The classical-only sets are expected to sit at about lambda/2
            // quantum queries (FESTA-type mask search under Grover).
            assert!(qf >= s.lambda as f64 / 2.0 - 1.0, "{}: {qf}", s.name);
        }
        // Gate counts under MAXDEPTH compared with AES key search.
        for (i, md) in [40.0, 64.0, 96.0].iter().enumerate() {
            let g = quantum_gate_floor_log2(&costs, i);
            let aes = aes_grover_gates_log2(s.lambda, *md);
            println!(
                "{} MAXDEPTH 2^{md}: gates 2^{g:.1} vs AES-{} 2^{aes:.0} ({})",
                s.name,
                s.lambda,
                if g >= aes { "meets" } else { "below" }
            );
            assert!(
                g >= aes,
                "{}: quantum gate floor 2^{g:.1} below AES-{} reference 2^{aes:.0} at MAXDEPTH 2^{md}",
                s.name,
                s.lambda
            );
        }
    }
    // L5 doubles as the quantum-hardened 128-bit set.
    let l5 = set_info(SetId::L5);
    assert!(quantum_query_floor_log2(&estimate(&l5)) >= 128.0);
}
