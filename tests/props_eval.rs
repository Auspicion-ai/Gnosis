//! §7.8 F6 RAG evaluation — **property-based-testing (PBT) gate — executed layer**.
//!
//! Implements **every** register row of
//! `docs/specs/6-f6-eval-property-register.md` (8 rows: P-IM-1/2/3/4, P-SM-1/2,
//! P-TP-1/2) as **one `#[test]` per row**, each backed by a tiny **deterministic
//! property harness** (`Xoshiro256**` seeded by `SplitMix64`) — no third-party PBT
//! crate, dependencies unchanged.
//!
//! ## Pinned deterministic seed
//!
//! Every row derives its `Rng` from a single fixed master seed mixed with a
//! per-row tag (the `P-<CLASS>-<N>` bytes), so the whole layer is reproducible.
//!
//! ## Budget (register gate: ≤100 cases/row, ≤400 total across the layer)
//!
//! | Row    | Budget | Row    | Budget |
//! |--------|--------|--------|--------|
//! | P-IM-1 | 60     | P-SM-1 | 40     |
//! | P-IM-2 | 40     | P-SM-2 | 30     |
//! | P-IM-3 | 40     | P-TP-1 | 40     |
//! | P-IM-4 | 40     | P-TP-2 | 40     |
//!
//! Sum = **330** generated cases (≤ 400). stop-after-5: a row aborts and reports
//! at most 5 distinct counterexamples.
//!
//! **RED-stage.** All assertions route through the `src/retrieval/eval.rs` stub
//! fns (`todo!()`), so every row FAILS at runtime — the failing red set. The
//! Implementer fills the stub bodies to go green.

use gnosis::{contextual_precision, contextual_recall, mrr_at_k, ndcg_at_k};
use gnosis::{DocumentId, NodeId, RagResultItem, Source};
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Deterministic PRNG: SplitMix64-seeded Xoshiro256** (house harness, §4.1–§4.5).
// ---------------------------------------------------------------------------

/// Master deterministic seed for the whole property layer.
const SEED: u64 = 0x9E37_79B9_7F4A_7C15;

/// SplitMix64 (also serves as the Xoshiro seed generator).
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Xoshiro256** — deterministic, dependency-free.
struct Rng {
    s: [u64; 4],
}

impl Rng {
    fn seeded(seed: u64) -> Self {
        let mut sm = seed;
        Rng {
            s: [
                splitmix64(&mut sm),
                splitmix64(&mut sm),
                splitmix64(&mut sm),
                splitmix64(&mut sm),
            ],
        }
    }
    fn rotl(x: u64, k: u32) -> u64 {
        x.rotate_left(k)
    }
    fn next(&mut self) -> u64 {
        let result = Self::rotl(self.s[1].wrapping_mul(5), 7).wrapping_mul(9);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[1] ^= t;
        self.s[2] = Self::rotl(self.s[2], 16);
        result
    }
    fn below(&mut self, n: u64) -> u64 {
        if n == 0 {
            0
        } else {
            self.next() % n
        }
    }
    fn pick<T: Copy>(&mut self, items: &[T]) -> T {
        items[self.below(items.len() as u64) as usize]
    }
    fn yes(&mut self) -> bool {
        self.next() & 1 == 1
    }
}

/// Per-row deterministic sub-seed from the master seed + a row tag.
fn row_seed(tag: u64) -> u64 {
    splitmix64(&mut (SEED ^ tag))
}

/// Row tags (`P-<CLASS>-<N>` bytes).
const PIM1: u64 = 0x50494D31; // "PIM1"
const PIM2: u64 = 0x50494D32; // "PIM2"
const PIM3: u64 = 0x50494D33; // "PIM3"
const PIM4: u64 = 0x50494D34; // "PIM4"
const PSM1: u64 = 0x50534D31; // "PSM1"
const PSM2: u64 = 0x50534D32; // "PSM2"
const PTP1: u64 = 0x50545031; // "PTP1"
const PTP2: u64 = 0x50545032; // "PTP2"

/// Per-row budget caps (sum = 330 ≤ 400).
const B_IM1: u32 = 60;
const B_IM2: u32 = 40;
const B_IM3: u32 = 40;
const B_IM4: u32 = 40;
const B_SM1: u32 = 40;
const B_SM2: u32 = 30;
const B_TP1: u32 = 40;
const B_TP2: u32 = 40;

// ---------------------------------------------------------------------------
// Generators / helpers
// ---------------------------------------------------------------------------

fn did(id: &str) -> DocumentId {
    DocumentId(id.to_string())
}
fn nid(id: &str) -> NodeId {
    NodeId(id.to_string())
}

fn item(doc: &str, node: &str) -> RagResultItem {
    RagResultItem {
        document_id: did(doc),
        node_id: nid(node),
        score: 0.0,
        snippet: String::new(),
        source: Source::Local,
        parent: None,
        stale: None,
    }
}

fn results_from_keys(keys: &[(String, String)]) -> Vec<RagResultItem> {
    keys.iter().map(|(d, n)| item(d, n)).collect()
}

fn rel_from_keys(keys: &[(String, String)]) -> HashSet<(DocumentId, NodeId)> {
    keys.iter().map(|(d, n)| (did(d), nid(n))).collect()
}

/// A pool of distinct `(doc, node)` key strings.
fn key_pool(n: usize) -> Vec<(String, String)> {
    (0..n).map(|i| (format!("d{i}"), format!("n{i}"))).collect()
}

/// Fisher–Yates shuffle of a slice (deterministic via the Rng).
fn shuffle<T>(rng: &mut Rng, items: &mut [T]) {
    for i in (1..items.len()).rev() {
        let j = rng.below((i + 1) as u64) as usize;
        items.swap(i, j);
    }
}

/// Generate an arbitrary `results` list (length 0..=8, keys drawn from a pool of
/// 10, **duplicates allowed**) plus an arbitrary `relevant` subset (0..=10, may
/// include keys not in `results`) plus an arbitrary `k`.
fn gen_arbitrary(rng: &mut Rng) -> (Vec<RagResultItem>, HashSet<(DocumentId, NodeId)>, usize) {
    let pool = key_pool(10);
    let len = rng.below(9) as usize; // 0..=8
    let mut keys: Vec<(String, String)> = Vec::with_capacity(len);
    for _ in 0..len {
        keys.push(pool[rng.below(pool.len() as u64) as usize].clone());
    }
    // Relevant: a random subset of the pool (0..=10), possibly disjoint from results.
    let mut rel_keys: Vec<(String, String)> = Vec::new();
    for k in &pool {
        if rng.yes() {
            rel_keys.push(k.clone());
        }
    }
    let results = results_from_keys(&keys);
    let relevant = rel_from_keys(&rel_keys);
    let k = rng.pick(&[0usize, 1, len, len + 1, 5, 50]);
    (results, relevant, k)
}

// ---------------------------------------------------------------------------
// P-IM-1 (IM) — strat:metric-bounds — boundedness over arbitrary inputs.
// ---------------------------------------------------------------------------
//
// Invariant: ∀ results/relevant/k: each of the four metrics is finite and in
// [0, 1] — never NaN, never inf, never negative, never > 1.
#[test]
fn p_im_1_metric_bounds() {
    let mut rng = Rng::seeded(row_seed(PIM1));
    let mut cases: u32 = 0;
    let mut cexes: Vec<String> = Vec::new();
    for _ in 0..B_IM1 {
        let (results, relevant, k) = gen_arbitrary(&mut rng);
        for (name, v) in [
            (
                "contextual_precision",
                contextual_precision(&results, &relevant, k),
            ),
            (
                "contextual_recall",
                contextual_recall(&results, &relevant, k),
            ),
            ("ndcg_at_k", ndcg_at_k(&results, &relevant, k)),
            ("mrr_at_k", mrr_at_k(&results, &relevant, k)),
        ] {
            if !v.is_finite() || !(0.0..=1.0).contains(&v) {
                let cex = format!("{name} = {v} out of [0,1] for k={k}");
                if !cexes.contains(&cex) {
                    cexes.push(cex);
                }
            }
        }
        cases += 1;
        if cexes.len() >= 5 {
            break; // stop-after-5
        }
    }
    assert!(
        cexes.is_empty(),
        "[P-IM-1][strat:metric-bounds] BROKEN: {cexes:?}"
    );
    assert!(cases <= B_IM1, "B_IM1 budget exceeded: {cases}");
    println!(
        "[P-IM-1] generated cases: {cases} HELD={}",
        cexes.is_empty()
    );
}

// ---------------------------------------------------------------------------
// P-IM-2 (IM) — strat:metric-repeat — determinism (pure function).
// ---------------------------------------------------------------------------
//
// Invariant: repeating each metric on the same inputs yields the identical f64.
#[test]
fn p_im_2_metric_repeat() {
    let mut rng = Rng::seeded(row_seed(PIM2));
    let mut cases: u32 = 0;
    let mut cexes: Vec<String> = Vec::new();
    for _ in 0..B_IM2 {
        let (results, relevant, k) = gen_arbitrary(&mut rng);
        let checks = [
            (
                "contextual_precision",
                contextual_precision(&results, &relevant, k),
                contextual_precision(&results, &relevant, k),
            ),
            (
                "contextual_recall",
                contextual_recall(&results, &relevant, k),
                contextual_recall(&results, &relevant, k),
            ),
            (
                "ndcg_at_k",
                ndcg_at_k(&results, &relevant, k),
                ndcg_at_k(&results, &relevant, k),
            ),
            (
                "mrr_at_k",
                mrr_at_k(&results, &relevant, k),
                mrr_at_k(&results, &relevant, k),
            ),
        ];
        for (name, a, b) in checks {
            if a != b {
                let cex = format!("{name} non-deterministic: {a} != {b} for k={k}");
                if !cexes.contains(&cex) {
                    cexes.push(cex);
                }
            }
        }
        cases += 1;
        if cexes.len() >= 5 {
            break;
        }
    }
    assert!(
        cexes.is_empty(),
        "[P-IM-2][strat:metric-repeat] BROKEN: {cexes:?}"
    );
    assert!(cases <= B_IM2, "B_IM2 budget exceeded: {cases}");
    println!(
        "[P-IM-2] generated cases: {cases} HELD={}",
        cexes.is_empty()
    );
}

// ---------------------------------------------------------------------------
// P-IM-3 (IM) — strat:empty-results — empty results, non-empty relevant → all 0.
// ---------------------------------------------------------------------------
#[test]
fn p_im_3_empty_results() {
    let mut rng = Rng::seeded(row_seed(PIM3));
    let mut cases: u32 = 0;
    let mut cexes: Vec<String> = Vec::new();
    let pool = key_pool(6);
    for _ in 0..B_IM3 {
        // Non-empty relevant.
        let mut rel_keys: Vec<(String, String)> = Vec::new();
        for k in &pool {
            if rng.yes() {
                rel_keys.push(k.clone());
            }
        }
        if rel_keys.is_empty() {
            rel_keys.push(pool[0].clone());
        }
        let relevant = rel_from_keys(&rel_keys);
        let k = rng.pick(&[0usize, 1, 5, 50]);
        let empty: Vec<RagResultItem> = Vec::new();
        let checks = [
            (
                "contextual_precision",
                contextual_precision(&empty, &relevant, k),
            ),
            ("contextual_recall", contextual_recall(&empty, &relevant, k)),
            ("ndcg_at_k", ndcg_at_k(&empty, &relevant, k)),
            ("mrr_at_k", mrr_at_k(&empty, &relevant, k)),
        ];
        for (name, v) in checks {
            if v != 0.0 {
                let cex = format!("{name} = {v} != 0.0 for empty results, k={k}");
                if !cexes.contains(&cex) {
                    cexes.push(cex);
                }
            }
        }
        cases += 1;
        if cexes.len() >= 5 {
            break;
        }
    }
    assert!(
        cexes.is_empty(),
        "[P-IM-3][strat:empty-results] BROKEN: {cexes:?}"
    );
    assert!(cases <= B_IM3, "B_IM3 budget exceeded: {cases}");
    println!(
        "[P-IM-3] generated cases: {cases} HELD={}",
        cexes.is_empty()
    );
}

// ---------------------------------------------------------------------------
// P-IM-4 (IM) — strat:empty-relevant — empty relevant → precision 0 / recall 1
// (vacuous) / nDCG 0 / MRR 0.
// ---------------------------------------------------------------------------
#[test]
fn p_im_4_empty_relevant() {
    let mut rng = Rng::seeded(row_seed(PIM4));
    let mut cases: u32 = 0;
    let mut cexes: Vec<String> = Vec::new();
    let pool = key_pool(6);
    for _ in 0..B_IM4 {
        // Arbitrary results (empty and non-empty).
        let len = rng.below(7) as usize; // 0..=6
        let mut keys: Vec<(String, String)> = Vec::with_capacity(len);
        for _ in 0..len {
            keys.push(pool[rng.below(pool.len() as u64) as usize].clone());
        }
        let results = results_from_keys(&keys);
        let empty: HashSet<(DocumentId, NodeId)> = HashSet::new();
        let k = rng.pick(&[0usize, 1, 5, 50]);
        let checks = [
            (
                "contextual_precision",
                contextual_precision(&results, &empty, k),
            ),
            ("contextual_recall", contextual_recall(&results, &empty, k)),
            ("ndcg_at_k", ndcg_at_k(&results, &empty, k)),
            ("mrr_at_k", mrr_at_k(&results, &empty, k)),
        ];
        for (name, v) in checks {
            let expected = if name == "contextual_recall" {
                1.0
            } else {
                0.0
            };
            if v != expected {
                let cex = format!("{name} = {v} != {expected} for empty relevant, k={k}");
                if !cexes.contains(&cex) {
                    cexes.push(cex);
                }
            }
        }
        cases += 1;
        if cexes.len() >= 5 {
            break;
        }
    }
    assert!(
        cexes.is_empty(),
        "[P-IM-4][strat:empty-relevant] BROKEN: {cexes:?}"
    );
    assert!(cases <= B_IM4, "B_IM4 budget exceeded: {cases}");
    println!(
        "[P-IM-4] generated cases: {cases} HELD={}",
        cexes.is_empty()
    );
}

// ---------------------------------------------------------------------------
// P-SM-1 (SM) — strat:rank-improve — rank-improvement monotonicity over
// DISTINCT-KEY results.
// ---------------------------------------------------------------------------
//
// Invariant: moving a relevant item to a strictly higher rank (lower index) in a
// distinct-key `results` does not decrease any of the four metrics.
#[test]
fn p_sm_1_rank_improve() {
    let mut rng = Rng::seeded(row_seed(PSM1));
    let mut cases: u32 = 0;
    let mut cexes: Vec<String> = Vec::new();
    for _ in 0..B_SM1 {
        // Distinct-key results: a random permutation of a random subset (size
        // 2..=6) of an 8-key pool.
        let pool = key_pool(8);
        let size = 2 + rng.below(5) as usize; // 2..=6
        let mut chosen: Vec<(String, String)> = pool[..size].to_vec();
        shuffle(&mut rng, &mut chosen);
        // Relevant: a non-empty subset of the results keys.
        let mut rel_keys: Vec<(String, String)> = Vec::new();
        for k in &chosen {
            if rng.yes() {
                rel_keys.push(k.clone());
            }
        }
        if rel_keys.is_empty() {
            rel_keys.push(chosen[0].clone());
        }
        let relevant = rel_from_keys(&rel_keys);
        // Pick a relevant item at index i (i > 0 so it can move up) and move it
        // to index j < i, preserving the relative order of all other items.
        let mut candidates: Vec<usize> = Vec::new();
        for (i, k) in chosen.iter().enumerate() {
            if i > 0 && relevant.contains(&(did(&k.0), nid(&k.1))) {
                candidates.push(i);
            }
        }
        if candidates.is_empty() {
            // No movable relevant item (e.g. the only relevant item is at rank 1);
            // skip this case (the property is vacuous).
            cases += 1;
            continue;
        }
        let i = candidates[rng.below(candidates.len() as u64) as usize];
        let j = rng.below(i as u64) as usize; // 0..i
        let mut moved = chosen.clone();
        let item = moved.remove(i);
        moved.insert(j, item);
        let k = moved.len(); // full list evaluated
        let results = results_from_keys(&chosen);
        let results_moved = results_from_keys(&moved);
        let checks = [
            (
                "contextual_precision",
                contextual_precision(&results, &relevant, k),
                contextual_precision(&results_moved, &relevant, k),
            ),
            (
                "contextual_recall",
                contextual_recall(&results, &relevant, k),
                contextual_recall(&results_moved, &relevant, k),
            ),
            (
                "ndcg_at_k",
                ndcg_at_k(&results, &relevant, k),
                ndcg_at_k(&results_moved, &relevant, k),
            ),
            (
                "mrr_at_k",
                mrr_at_k(&results, &relevant, k),
                mrr_at_k(&results_moved, &relevant, k),
            ),
        ];
        for (name, before, after) in checks {
            if after < before {
                let cex = format!(
                    "{name} decreased {before} -> {after} moving rank {} to rank {}",
                    i + 1,
                    j + 1
                );
                if !cexes.contains(&cex) {
                    cexes.push(cex);
                }
            }
        }
        cases += 1;
        if cexes.len() >= 5 {
            break;
        }
    }
    assert!(
        cexes.is_empty(),
        "[P-SM-1][strat:rank-improve] BROKEN: {cexes:?}"
    );
    assert!(cases <= B_SM1, "B_SM1 budget exceeded: {cases}");
    println!(
        "[P-SM-1] generated cases: {cases} HELD={}",
        cexes.is_empty()
    );
}

// ---------------------------------------------------------------------------
// P-SM-2 (SM) — strat:k-monotone — recall@k non-decreasing in k; precision is
// NOT asserted monotonic (a precision decrease is expected and not a broken row).
// ---------------------------------------------------------------------------
#[test]
fn p_sm_2_k_monotone() {
    let mut rng = Rng::seeded(row_seed(PSM2));
    let mut cases: u32 = 0;
    let mut cexes: Vec<String> = Vec::new();
    let pool = key_pool(8);
    for _ in 0..B_SM2 {
        let len = 1 + rng.below(7) as usize; // 1..=7
        let mut keys: Vec<(String, String)> = Vec::with_capacity(len);
        for _ in 0..len {
            keys.push(pool[rng.below(pool.len() as u64) as usize].clone());
        }
        let results = results_from_keys(&keys);
        let mut rel_keys: Vec<(String, String)> = Vec::new();
        for k in &pool {
            if rng.yes() {
                rel_keys.push(k.clone());
            }
        }
        let relevant = rel_from_keys(&rel_keys);
        let k1 = rng.pick(&[0usize, 1, 5]);
        let k2 = rng.pick(&[1usize, 5, 50]);
        if k1 >= k2 {
            cases += 1;
            continue;
        }
        let r1 = contextual_recall(&results, &relevant, k1);
        let r2 = contextual_recall(&results, &relevant, k2);
        if r2 < r1 {
            let cex = format!("recall decreased {r1} -> {r2} for k1={k1} < k2={k2}");
            if !cexes.contains(&cex) {
                cexes.push(cex);
            }
        }
        cases += 1;
        if cexes.len() >= 5 {
            break;
        }
    }
    // The register's documented precision-decrease example: [B,A,C,D], {B,D},
    // k1=2 → CP=1.0, k2=4 → CP=0.75. This is EXPECTED and must NOT fail the row.
    let r = results_from_keys(&[
        ("B".to_string(), "b".to_string()),
        ("A".to_string(), "a".to_string()),
        ("C".to_string(), "c".to_string()),
        ("D".to_string(), "d".to_string()),
    ]);
    let rel = rel_from_keys(&[
        ("B".to_string(), "b".to_string()),
        ("D".to_string(), "d".to_string()),
    ]);
    let p1 = contextual_precision(&r, &rel, 2);
    let p2 = contextual_precision(&r, &rel, 4);
    assert!(
        p1 > p2,
        "[P-SM-2] expected the documented precision decrease 1.0 -> 0.75, got {p1} -> {p2}"
    );
    assert!(
        cexes.is_empty(),
        "[P-SM-2][strat:k-monotone] BROKEN: {cexes:?}"
    );
    assert!(cases <= B_SM2, "B_SM2 budget exceeded: {cases}");
    println!(
        "[P-SM-2] generated cases: {cases} HELD={}",
        cexes.is_empty()
    );
}

// ---------------------------------------------------------------------------
// P-TP-1 (TP) — strat:perfect-retrieval — perfect retrieval → all four = 1.0.
// ---------------------------------------------------------------------------
#[test]
fn p_tp_1_perfect_retrieval() {
    let mut rng = Rng::seeded(row_seed(PTP1));
    let mut cases: u32 = 0;
    let mut cexes: Vec<String> = Vec::new();
    let pool = key_pool(8);
    for _ in 0..B_TP1 {
        // Non-empty relevant set.
        let rel_size = 1 + rng.below(4) as usize; // 1..=4
        let mut rel_keys: Vec<(String, String)> = pool[..rel_size].to_vec();
        shuffle(&mut rng, &mut rel_keys);
        let relevant = rel_from_keys(&rel_keys);
        // results = exactly the relevant keys (all relevant, no non-relevant).
        let results = results_from_keys(&rel_keys);
        // k with |relevant| <= min(k, results.len()).
        let k = rng.pick(&[rel_size, rel_size + 1, 50]);
        let checks = [
            (
                "contextual_precision",
                contextual_precision(&results, &relevant, k),
            ),
            (
                "contextual_recall",
                contextual_recall(&results, &relevant, k),
            ),
            ("ndcg_at_k", ndcg_at_k(&results, &relevant, k)),
            ("mrr_at_k", mrr_at_k(&results, &relevant, k)),
        ];
        for (name, v) in checks {
            if v != 1.0 {
                let cex = format!("{name} = {v} != 1.0 for perfect retrieval, k={k}");
                if !cexes.contains(&cex) {
                    cexes.push(cex);
                }
            }
        }
        cases += 1;
        if cexes.len() >= 5 {
            break;
        }
    }
    assert!(
        cexes.is_empty(),
        "[P-TP-1][strat:perfect-retrieval] BROKEN: {cexes:?}"
    );
    assert!(cases <= B_TP1, "B_TP1 budget exceeded: {cases}");
    println!(
        "[P-TP-1] generated cases: {cases} HELD={}",
        cexes.is_empty()
    );
}

// ---------------------------------------------------------------------------
// P-TP-2 (TP) — strat:no-relevant — no relevant retrieved → all four = 0.0.
// ---------------------------------------------------------------------------
#[test]
fn p_tp_2_no_relevant() {
    let mut rng = Rng::seeded(row_seed(PTP2));
    let mut cases: u32 = 0;
    let mut cexes: Vec<String> = Vec::new();
    let pool = key_pool(8);
    for _ in 0..B_TP2 {
        // results whose keys are disjoint from a non-empty relevant set.
        let len = rng.below(6) as usize; // 0..=5
        let mut keys: Vec<(String, String)> = Vec::with_capacity(len);
        for _ in 0..len {
            keys.push(pool[rng.below(pool.len() as u64) as usize].clone());
        }
        let results = results_from_keys(&keys);
        // relevant = keys NOT in results (disjoint), non-empty.
        let mut rel_keys: Vec<(String, String)> = Vec::new();
        for k in &pool {
            if !keys.contains(k) {
                rel_keys.push(k.clone());
            }
        }
        if rel_keys.is_empty() {
            // All pool keys are in results; use a key outside the pool.
            rel_keys.push(("dX".to_string(), "nX".to_string()));
        }
        let relevant = rel_from_keys(&rel_keys);
        let k = rng.pick(&[0usize, 1, 5, 50]);
        let checks = [
            (
                "contextual_precision",
                contextual_precision(&results, &relevant, k),
            ),
            (
                "contextual_recall",
                contextual_recall(&results, &relevant, k),
            ),
            ("ndcg_at_k", ndcg_at_k(&results, &relevant, k)),
            ("mrr_at_k", mrr_at_k(&results, &relevant, k)),
        ];
        for (name, v) in checks {
            if v != 0.0 {
                let cex = format!("{name} = {v} != 0.0 for no-relevant, k={k}");
                if !cexes.contains(&cex) {
                    cexes.push(cex);
                }
            }
        }
        cases += 1;
        if cexes.len() >= 5 {
            break;
        }
    }
    assert!(
        cexes.is_empty(),
        "[P-TP-2][strat:no-relevant] BROKEN: {cexes:?}"
    );
    assert!(cases <= B_TP2, "B_TP2 budget exceeded: {cases}");
    println!(
        "[P-TP-2] generated cases: {cases} HELD={}",
        cexes.is_empty()
    );
}
