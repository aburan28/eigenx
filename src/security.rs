//! Cost model for the known attacks on Eigen-Séta, evaluated per parameter
//! set.  Everything here is arithmetic on the public parameters; the
//! formulas and their justification are in `docs/SECURITY.md`.
//!
//! Costs are reported as log2 of (a) classical operations, (b) quantum
//! oracle queries for the best known quantum algorithm, and (c) quantum
//! gates once the oracle cost and NIST's MAXDEPTH limit are taken into
//! account.  All three are *estimates in the attacker's favour*: the
//! classical costs ignore memory, and the oracle cost assumes a chain
//! implementation a few times cheaper than this crate's.

use crate::sets::SetInfo;

/// NIST MAXDEPTH values used for the gate-count columns.
pub const MAXDEPTH_LOG2: [f64; 3] = [40.0, 64.0, 96.0];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Attack {
    /// Meet-in-the-middle / vOW on the sender's 3^F3-isogeny E_A -> E_C.
    SenderIsogenyMitm,
    /// Guess the masks modulo 2^e' (2^{2e'} >= d) and recover the sender's
    /// isogeny with a Kani computation (FESTA's CIST problem).
    MaskGuessKani,
    /// Guess the secret degree f, derive the key, test it on one ciphertext.
    DegreeSearch,
    /// Generic endomorphism-ring / isogeny-path computation for E_A.
    EndomorphismRing,
    /// Order of the hidden class-group element behind the public eigenlines,
    /// if the class-group action could be evaluated (it cannot, beyond E
    /// steps, without the secret).  Lower bound only.
    HiddenOrientation,
}

impl Attack {
    pub const ALL: [Attack; 5] = [
        Attack::SenderIsogenyMitm,
        Attack::MaskGuessKani,
        Attack::DegreeSearch,
        Attack::EndomorphismRing,
        Attack::HiddenOrientation,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Attack::SenderIsogenyMitm => "sender isogeny MITM",
            Attack::MaskGuessKani => "mask guess + Kani",
            Attack::DegreeSearch => "secret degree search",
            Attack::EndomorphismRing => "End(E_A) generic",
            Attack::HiddenOrientation => "hidden orientation",
        }
    }
    pub fn target(self) -> &'static str {
        match self {
            Attack::SenderIsogenyMitm | Attack::MaskGuessKani => "message",
            _ => "key",
        }
    }
}

#[derive(Clone, Debug)]
pub struct AttackCost {
    pub attack: Attack,
    /// log2 of the number of oracle evaluations (or group operations)
    /// classically.
    pub classical_log2: f64,
    /// log2 of the number of quantum oracle queries for the best known
    /// quantum algorithm (Grover for exhaustive search, Tani for claws,
    /// p^{1/4} for the endomorphism ring).  `None` when no quantum speed-up
    /// is known that the attacker can run.
    pub quantum_queries_log2: Option<f64>,
    /// log2 of the quantum gate count under MAXDEPTH = 2^40, 2^64, 2^96,
    /// costing each query as one `oracle_gates_log2` circuit.
    pub quantum_gates_log2: Option<[f64; 3]>,
    /// log2 gate cost of one oracle call used for the gate columns.
    pub oracle_gates_log2: f64,
    pub note: &'static str,
}

/// Gate cost of one F_p² multiplication at `p_bits`: three F_p
/// multiplications at Karatsuba cost.  Deliberately optimistic for the
/// attacker (schoolbook would be p_bits²).
pub fn fp2_mul_gates_log2(p_bits: usize) -> f64 {
    (3.0f64).log2() + 1.585 * (p_bits as f64).log2()
}

/// Gate cost of one (2,2)-chain of `steps` steps evaluating a handful of
/// points: 2^8 F_p² multiplications per step is below a theta-coordinate
/// implementation's count, so this is in the attacker's favour.
pub fn chain_gates_log2(steps: usize, p_bits: usize) -> f64 {
    (steps as f64).log2() + 8.0 + fp2_mul_gates_log2(p_bits)
}

/// Grover search over 2^n items with an oracle of `oracle_log2` gates whose
/// depth is 2^(oracle_log2 - 4), under a depth limit 2^maxdepth: NIST's
/// methodology of running 2^k machines in parallel so that each finishes
/// within the depth limit.  Returns log2 of the total gate count.
pub fn grover_gates_log2(n: f64, oracle_log2: f64, maxdepth: f64) -> f64 {
    let depth_log2 = oracle_log2 - 4.0;
    let serial_depth = n / 2.0 + depth_log2;
    if serial_depth <= maxdepth {
        n / 2.0 + oracle_log2
    } else {
        // parallel machines k = n + 2 depth - 2 maxdepth; total gates
        // = sqrt(N 2^k) * oracle = N * depth * oracle / maxdepth
        n + depth_log2 + oracle_log2 - maxdepth
    }
}

fn grover_cols(n: f64, oracle_log2: f64) -> [f64; 3] {
    let mut out = [0.0; 3];
    for (i, md) in MAXDEPTH_LOG2.iter().enumerate() {
        out[i] = grover_gates_log2(n, oracle_log2, *md);
    }
    out
}

pub fn estimate(s: &SetInfo) -> Vec<AttackCost> {
    let d = s.d_bits;
    let p = s.p_bits;
    // Oracle for mask guessing: one chain of e' steps on E_A x E_C.
    let e_prime = (d / 2.0).ceil();
    let mask_oracle = chain_gates_log2(e_prime as usize + 2, p);
    // Oracle for degree search: derive the key (negligible) + one
    // decapsulation chain of E steps.
    let deg_oracle = chain_gates_log2(s.e, p);
    // f is odd in [lo, 2 lo): 2^(f_bits - 2) candidates, of which only
    // about 1/ln(2^E) admit Kani scalars (2^E - 7 d^2 f^2 must be a sum of
    // two squares, which honest keygen also rejection-samples for).  The
    // attacker filters those out with cheap arithmetic, so only that
    // fraction costs a chain evaluation.
    let f_space = (s.f_bits as f64 - 2.0 - (s.e as f64 * 2f64.ln()).log2()).max(0.0);
    // 3-isogeny step oracle for MITM: one 3-isogeny evaluation per node.
    let mitm_oracle = 6.0 + fp2_mul_gates_log2(p);
    vec![
        AttackCost {
            attack: Attack::SenderIsogenyMitm,
            classical_log2: d / 2.0,
            quantum_queries_log2: Some(d / 3.0),
            quantum_gates_log2: Some(grover_cols(2.0 * d / 3.0, mitm_oracle)),
            oracle_gates_log2: mitm_oracle,
            note: "classical d^1/2 (memory ignored); Tani claw d^1/3 queries, \
                   realistically no better than classical (Jaques-Schanck)",
        },
        AttackCost {
            attack: Attack::MaskGuessKani,
            classical_log2: e_prime,
            quantum_queries_log2: Some(e_prime / 2.0),
            quantum_gates_log2: Some(grover_cols(e_prime, mask_oracle)),
            oracle_gates_log2: mask_oracle,
            note: "guess masks mod 2^e', 2^2e' >= d; Grover halves the exponent. \
                   Gate columns use the chain_gates model",
        },
        AttackCost {
            attack: Attack::DegreeSearch,
            classical_log2: f_space,
            quantum_queries_log2: Some(f_space / 2.0),
            quantum_gates_log2: Some(grover_cols(f_space, deg_oracle)),
            oracle_gates_log2: deg_oracle,
            note: "key = function(f, public data); test each f with one decapsulation",
        },
        AttackCost {
            attack: Attack::EndomorphismRing,
            // p^(1/3+o(1)) (Wesolowski 2026, heuristic, memory p^1/3) is
            // used instead of the classical p^1/2 of Delfs-Galbraith to stay
            // on the attacker's side; the o(1) hides a superpolynomial factor.
            classical_log2: p as f64 / 3.0,
            quantum_queries_log2: Some(p as f64 / 4.0),
            quantum_gates_log2: None,
            oracle_gates_log2: 0.0,
            note: "classical p^1/3 (Wesolowski 2026; p^1/2 Delfs-Galbraith without \
                   huge memory); quantum p^1/4 Biasse-Jao-Sankar",
        },
        AttackCost {
            attack: Attack::HiddenOrientation,
            classical_log2: s.f_bits as f64 / 2.0,
            quantum_queries_log2: None,
            quantum_gates_log2: None,
            oracle_gates_log2: 0.0,
            note: "BSGS on the hidden class group, assuming (falsely, in the \
                   attacker's favour) that its action can be evaluated",
        },
    ]
}

/// Smallest classical cost over all attacks.
pub fn classical_floor_log2(costs: &[AttackCost]) -> f64 {
    costs
        .iter()
        .map(|c| c.classical_log2)
        .fold(f64::INFINITY, f64::min)
}

/// Smallest quantum query count over the attacks with a known quantum
/// algorithm.
pub fn quantum_query_floor_log2(costs: &[AttackCost]) -> f64 {
    costs
        .iter()
        .filter_map(|c| c.quantum_queries_log2)
        .fold(f64::INFINITY, f64::min)
}

/// Smallest quantum gate count at the given MAXDEPTH index over the attacks
/// with a gate estimate.
pub fn quantum_gate_floor_log2(costs: &[AttackCost], maxdepth_idx: usize) -> f64 {
    costs
        .iter()
        .filter_map(|c| c.quantum_gates_log2.map(|g| g[maxdepth_idx]))
        .fold(f64::INFINITY, f64::min)
}

/// NIST-style reference: Grover key search on AES-λ costs about
/// 2^(λ/2) queries and, per the call for proposals, 2^170, 2^233, 2^298
/// gates divided by MAXDEPTH for λ = 128, 192, 256.
pub fn aes_grover_gates_log2(lambda: usize, maxdepth: f64) -> f64 {
    let base = match lambda {
        128 => 170.0,
        192 => 233.0,
        _ => 298.0,
    };
    base - maxdepth
}

/// Render the full table for one parameter set.
pub fn render(s: &SetInfo, costs: &[AttackCost]) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "{} (lambda {}, p {} bits, E {}, F3 {} [log2 d {:.1}], f {} bits)\n",
        s.name, s.lambda, s.p_bits, s.e, s.f3, s.d_bits, s.f_bits
    ));
    out.push_str(&format!(
        "  {:<24} {:<8} {:>9} {:>9} {:>8} {:>8} {:>8}\n",
        "attack", "target", "classical", "q-queries", "MD2^40", "MD2^64", "MD2^96"
    ));
    for c in costs {
        let q = c
            .quantum_queries_log2
            .map(|x| format!("{x:9.1}"))
            .unwrap_or_else(|| format!("{:>9}", "-"));
        let g = c
            .quantum_gates_log2
            .map(|g| format!("{:8.1} {:8.1} {:8.1}", g[0], g[1], g[2]))
            .unwrap_or_else(|| format!("{:>8} {:>8} {:>8}", "-", "-", "-"));
        out.push_str(&format!(
            "  {:<24} {:<8} {:9.1} {} {}\n",
            c.attack.name(),
            c.attack.target(),
            c.classical_log2,
            q,
            g
        ));
    }
    out.push_str(&format!(
        "  floor: classical 2^{:.1}, quantum queries 2^{:.1}, quantum gates 2^{:.1}/2^{:.1}/2^{:.1}\n",
        classical_floor_log2(costs),
        quantum_query_floor_log2(costs),
        quantum_gate_floor_log2(costs, 0),
        quantum_gate_floor_log2(costs, 1),
        quantum_gate_floor_log2(costs, 2),
    ));
    if s.lambda >= 128 {
        out.push_str(&format!(
            "  reference AES-{} Grover gates: 2^{:.0}/2^{:.0}/2^{:.0}\n",
            s.lambda,
            aes_grover_gates_log2(s.lambda, 40.0),
            aes_grover_gates_log2(s.lambda, 64.0),
            aes_grover_gates_log2(s.lambda, 96.0),
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grover_parallelisation_never_cheaper_than_serial() {
        for n in [64.0, 128.0, 256.0] {
            for o in [20.0, 36.0, 50.0] {
                let serial = n / 2.0 + o;
                for md in MAXDEPTH_LOG2 {
                    assert!(grover_gates_log2(n, o, md) >= serial - 1e-9);
                }
            }
        }
    }
}
