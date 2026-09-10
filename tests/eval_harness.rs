//! §7.8 F6 — RAG evaluation harness conformance suite (`tests/eval_harness.rs`).
//!
//! TestWriter-derived from `docs/specs/6-f6-eval-spec.md` (the F6 behavior
//! contract) ALONE. Covers every metric state and fail-state: the §6
//! hand-computable reference values for all four metric fns (exact / `≈` with the
//! pinned 1e-4 tolerance), the §4 empty-set conventions (incl. the empty-relevant
//! precedence over `k=0` for recall), the `k` boundaries (`k=0`, `k=1`, `k=len`,
//! `k>len`), the duplicate-key conventions (precision/MRR per-position, recall
//! set-semantics, nDCG first-occurrence), the no-panic/no-NaN boundedness
//! cross-cutting state, the §8 corpus-parsing + metric-computation path, and the
//! §8 `mode` serde note (PascalCase `QueryMode`, lowercase→variant manual map).
//!
//! **RED-stage.** This suite compiles against the stubs in `src/retrieval/eval.rs`
//! (whose bodies are `todo!()` placeholders) and FAILS at runtime — the failing
//! red set. The Implementer replaces the stub bodies to go green.

use gnosis::{contextual_precision, contextual_recall, mrr_at_k, ndcg_at_k};
use gnosis::{DocumentId, NodeId, QueryMode, RagResultItem, Source};
use serde::Deserialize;
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Pinned `≈` tolerance (§6): `|actual − expected| ≤ 1e-4`.
const TOL: f64 = 1e-4;

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= TOL,
        "expected {expected} ± {TOL}, got {actual}"
    );
}

fn did(id: &str) -> DocumentId {
    DocumentId(id.to_string())
}
fn nid(id: &str) -> NodeId {
    NodeId(id.to_string())
}

/// Build a `RagResultItem` from a key; only `document_id`/`node_id` matter to the
/// metrics, so the rest are defaults.
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

/// Build a `results` list from a sequence of `(doc, node)` key strings.
fn results(keys: &[(&str, &str)]) -> Vec<RagResultItem> {
    keys.iter().map(|&(d, n)| item(d, n)).collect()
}

/// Build a `relevant` set from a sequence of `(doc, node)` key strings.
fn relevant(keys: &[(&str, &str)]) -> HashSet<(DocumentId, NodeId)> {
    keys.iter().map(|&(d, n)| (did(d), nid(n))).collect()
}

// ---------------------------------------------------------------------------
// §6.1 contextual_precision — hand-computable reference values
// ---------------------------------------------------------------------------
#[test]
fn contextual_precision_reference_values() {
    // All-relevant-at-top → 1.0.
    assert_eq!(
        contextual_precision(
            &results(&[("B", "b"), ("D", "d")]),
            &relevant(&[("B", "b"), ("D", "d")]),
            2
        ),
        1.0
    );
    // Partial ranking, hand-computable.
    assert_eq!(
        contextual_precision(
            &results(&[("A", "a"), ("B", "b"), ("C", "c"), ("D", "d")]),
            &relevant(&[("B", "b"), ("D", "d")]),
            4
        ),
        0.5
    );
    assert_eq!(
        contextual_precision(
            &results(&[("B", "b"), ("A", "a"), ("C", "c")]),
            &relevant(&[("B", "b")]),
            3
        ),
        1.0
    );
    assert_eq!(
        contextual_precision(
            &results(&[("A", "a"), ("B", "b"), ("C", "c")]),
            &relevant(&[("B", "b")]),
            3
        ),
        0.5
    );
    // Relevant beyond top-k ignored.
    assert_eq!(
        contextual_precision(
            &results(&[("B", "b"), ("A", "a"), ("C", "c"), ("D", "d")]),
            &relevant(&[("B", "b"), ("D", "d")]),
            2
        ),
        1.0
    );
    // k > len → effective_top = len.
    assert_eq!(
        contextual_precision(
            &results(&[("B", "b"), ("D", "d")]),
            &relevant(&[("B", "b"), ("D", "d")]),
            10
        ),
        1.0
    );
    // k = 1.
    assert_eq!(
        contextual_precision(
            &results(&[("B", "b"), ("A", "a")]),
            &relevant(&[("B", "b")]),
            1
        ),
        1.0
    );
    assert_eq!(
        contextual_precision(
            &results(&[("A", "a"), ("B", "b")]),
            &relevant(&[("B", "b")]),
            1
        ),
        0.0
    );
    // No relevant retrieved.
    assert_eq!(
        contextual_precision(
            &results(&[("A", "a"), ("B", "b"), ("C", "c")]),
            &relevant(&[("D", "d")]),
            3
        ),
        0.0
    );
    // Empty relevant.
    assert_eq!(
        contextual_precision(
            &results(&[("A", "a"), ("B", "b"), ("C", "c")]),
            &relevant(&[]),
            3
        ),
        0.0
    );
    // Empty results.
    assert_eq!(
        contextual_precision(&results(&[]), &relevant(&[("B", "b")]), 3),
        0.0
    );
    // k = 0.
    assert_eq!(
        contextual_precision(
            &results(&[("B", "b"), ("A", "a")]),
            &relevant(&[("B", "b")]),
            0
        ),
        0.0
    );
    // Duplicate relevant key counts at both positions.
    assert_eq!(
        contextual_precision(
            &results(&[("B", "b"), ("B", "b")]),
            &relevant(&[("B", "b")]),
            2
        ),
        1.0
    );
    // Relevant not in results.
    assert_eq!(
        contextual_precision(
            &results(&[("A", "a"), ("C", "c")]),
            &relevant(&[("B", "b")]),
            2
        ),
        0.0
    );
    // Relevant beyond top-k ignored (k=1).
    assert_eq!(
        contextual_precision(
            &results(&[("B", "b"), ("A", "a"), ("C", "c")]),
            &relevant(&[("B", "b"), ("C", "c")]),
            1
        ),
        1.0
    );
}

// ---------------------------------------------------------------------------
// §6.2 contextual_recall — hand-computable reference values
// ---------------------------------------------------------------------------
#[test]
fn contextual_recall_reference_values() {
    assert_eq!(
        contextual_recall(
            &results(&[("B", "b"), ("D", "d")]),
            &relevant(&[("B", "b"), ("D", "d")]),
            2
        ),
        1.0
    );
    assert_eq!(
        contextual_recall(
            &results(&[("B", "b"), ("A", "a"), ("C", "c")]),
            &relevant(&[("B", "b"), ("D", "d")]),
            3
        ),
        0.5
    );
    assert_eq!(
        contextual_recall(
            &results(&[("B", "b"), ("A", "a"), ("C", "c"), ("D", "d")]),
            &relevant(&[("B", "b"), ("D", "d")]),
            1
        ),
        0.5
    );
    assert_eq!(
        contextual_recall(
            &results(&[("B", "b")]),
            &relevant(&[("B", "b"), ("D", "d")]),
            10
        ),
        0.5
    );
    assert_eq!(
        contextual_recall(
            &results(&[("B", "b"), ("A", "a")]),
            &relevant(&[("B", "b")]),
            1
        ),
        1.0
    );
    assert_eq!(
        contextual_recall(
            &results(&[("A", "a"), ("B", "b")]),
            &relevant(&[("B", "b")]),
            1
        ),
        0.0
    );
    assert_eq!(
        contextual_recall(
            &results(&[("A", "a"), ("B", "b")]),
            &relevant(&[("C", "c")]),
            2
        ),
        0.0
    );
    // Empty relevant → vacuous 1.0.
    assert_eq!(
        contextual_recall(&results(&[("A", "a"), ("B", "b")]), &relevant(&[]), 2),
        1.0
    );
    // Empty results.
    assert_eq!(
        contextual_recall(&results(&[]), &relevant(&[("B", "b")]), 3),
        0.0
    );
    // k = 0.
    assert_eq!(
        contextual_recall(
            &results(&[("B", "b"), ("A", "a")]),
            &relevant(&[("B", "b")]),
            0
        ),
        0.0
    );
    // Duplicates do not inflate recall (set semantics).
    assert_eq!(
        contextual_recall(
            &results(&[("B", "b"), ("B", "b")]),
            &relevant(&[("B", "b")]),
            2
        ),
        1.0
    );
    // Relevant not in results.
    assert_eq!(
        contextual_recall(
            &results(&[("A", "a"), ("C", "c")]),
            &relevant(&[("B", "b")]),
            2
        ),
        0.0
    );
    // Relevant beyond top-k ignored (k=1).
    assert_eq!(
        contextual_recall(
            &results(&[("B", "b"), ("A", "a"), ("C", "c")]),
            &relevant(&[("B", "b"), ("C", "c")]),
            1
        ),
        0.5
    );
}

// ---------------------------------------------------------------------------
// §6.3 ndcg_at_k — hand-computable reference values
// ---------------------------------------------------------------------------
#[test]
fn ndcg_at_k_reference_values() {
    assert_eq!(
        ndcg_at_k(
            &results(&[("B", "b"), ("D", "d")]),
            &relevant(&[("B", "b"), ("D", "d")]),
            2
        ),
        1.0
    );
    assert_close(
        ndcg_at_k(
            &results(&[("A", "a"), ("B", "b"), ("C", "c"), ("D", "d")]),
            &relevant(&[("B", "b"), ("D", "d")]),
            4,
        ),
        0.6510,
    );
    assert_eq!(
        ndcg_at_k(
            &results(&[("B", "b"), ("A", "a"), ("C", "c")]),
            &relevant(&[("B", "b")]),
            3
        ),
        1.0
    );
    assert_close(
        ndcg_at_k(
            &results(&[("A", "a"), ("B", "b"), ("C", "c")]),
            &relevant(&[("B", "b")]),
            3,
        ),
        0.6309,
    );
    assert_close(
        ndcg_at_k(
            &results(&[("B", "b"), ("A", "a"), ("C", "c"), ("D", "d")]),
            &relevant(&[("B", "b"), ("D", "d")]),
            2,
        ),
        0.6131,
    );
    assert_eq!(
        ndcg_at_k(
            &results(&[("B", "b"), ("D", "d")]),
            &relevant(&[("B", "b"), ("D", "d")]),
            10
        ),
        1.0
    );
    assert_eq!(
        ndcg_at_k(
            &results(&[("B", "b"), ("A", "a")]),
            &relevant(&[("B", "b")]),
            1
        ),
        1.0
    );
    assert_eq!(
        ndcg_at_k(
            &results(&[("A", "a"), ("B", "b")]),
            &relevant(&[("B", "b")]),
            1
        ),
        0.0
    );
    assert_eq!(
        ndcg_at_k(
            &results(&[("A", "a"), ("B", "b")]),
            &relevant(&[("C", "c")]),
            2
        ),
        0.0
    );
    // Empty relevant → IDCG=0 → 0.0.
    assert_eq!(
        ndcg_at_k(&results(&[("A", "a"), ("B", "b")]), &relevant(&[]), 2),
        0.0
    );
    // Empty results.
    assert_eq!(ndcg_at_k(&results(&[]), &relevant(&[("B", "b")]), 3), 0.0);
    // k = 0.
    assert_eq!(
        ndcg_at_k(
            &results(&[("B", "b"), ("A", "a")]),
            &relevant(&[("B", "b")]),
            0
        ),
        0.0
    );
    // Duplicate relevant key: first-occurrence → rel=0 on the second → nDCG=1.0.
    assert_eq!(
        ndcg_at_k(
            &results(&[("B", "b"), ("B", "b")]),
            &relevant(&[("B", "b")]),
            2
        ),
        1.0
    );
    // Relevant not retrieved (D in relevant, only B retrieved).
    assert_close(
        ndcg_at_k(
            &results(&[("B", "b")]),
            &relevant(&[("B", "b"), ("D", "d")]),
            10,
        ),
        0.6131,
    );
}

// ---------------------------------------------------------------------------
// §6.4 mrr_at_k — hand-computable reference values
// ---------------------------------------------------------------------------
#[test]
fn mrr_at_k_reference_values() {
    assert_eq!(
        mrr_at_k(
            &results(&[("B", "b"), ("A", "a"), ("C", "c")]),
            &relevant(&[("B", "b")]),
            3
        ),
        1.0
    );
    assert_eq!(
        mrr_at_k(
            &results(&[("A", "a"), ("B", "b"), ("C", "c")]),
            &relevant(&[("B", "b")]),
            3
        ),
        0.5
    );
    assert_close(
        mrr_at_k(
            &results(&[("A", "a"), ("C", "c"), ("B", "b")]),
            &relevant(&[("B", "b")]),
            3,
        ),
        0.3333,
    );
    // Relevant beyond top-k → no relevant in effective top → 0.0.
    assert_eq!(
        mrr_at_k(
            &results(&[("B", "b"), ("A", "a"), ("C", "c"), ("D", "d")]),
            &relevant(&[("D", "d")]),
            2
        ),
        0.0
    );
    assert_eq!(
        mrr_at_k(
            &results(&[("A", "a"), ("B", "b")]),
            &relevant(&[("C", "c")]),
            2
        ),
        0.0
    );
    // Empty relevant.
    assert_eq!(
        mrr_at_k(&results(&[("A", "a"), ("B", "b")]), &relevant(&[]), 2),
        0.0
    );
    // Empty results.
    assert_eq!(mrr_at_k(&results(&[]), &relevant(&[("B", "b")]), 3), 0.0);
    // k = 0.
    assert_eq!(
        mrr_at_k(
            &results(&[("B", "b"), ("A", "a")]),
            &relevant(&[("B", "b")]),
            0
        ),
        0.0
    );
    // Duplicate relevant key: first at rank 1 → 1.0.
    assert_eq!(
        mrr_at_k(
            &results(&[("B", "b"), ("B", "b")]),
            &relevant(&[("B", "b")]),
            2
        ),
        1.0
    );
    // Relevant not in results.
    assert_eq!(
        mrr_at_k(
            &results(&[("A", "a"), ("C", "c")]),
            &relevant(&[("B", "b")]),
            2
        ),
        0.0
    );
}

// ---------------------------------------------------------------------------
// §4 empty-set conventions (pinned table + precedence)
// ---------------------------------------------------------------------------
#[test]
fn empty_set_conventions() {
    // Empty relevant → precision 0 / recall 1 (vacuous) / nDCG 0 / MRR 0.
    let r = results(&[("A", "a"), ("B", "b")]);
    let empty = relevant(&[]);
    assert_eq!(contextual_precision(&r, &empty, 2), 0.0);
    assert_eq!(contextual_recall(&r, &empty, 2), 1.0);
    assert_eq!(ndcg_at_k(&r, &empty, 2), 0.0);
    assert_eq!(mrr_at_k(&r, &empty, 2), 0.0);

    // Empty results (relevant non-empty) → all 0.
    let empty_res = results(&[]);
    let rel = relevant(&[("B", "b")]);
    assert_eq!(contextual_precision(&empty_res, &rel, 3), 0.0);
    assert_eq!(contextual_recall(&empty_res, &rel, 3), 0.0);
    assert_eq!(ndcg_at_k(&empty_res, &rel, 3), 0.0);
    assert_eq!(mrr_at_k(&empty_res, &rel, 3), 0.0);

    // k = 0 → all 0.
    let r2 = results(&[("B", "b"), ("A", "a")]);
    assert_eq!(contextual_precision(&r2, &rel, 0), 0.0);
    assert_eq!(contextual_recall(&r2, &rel, 0), 0.0);
    assert_eq!(ndcg_at_k(&r2, &rel, 0), 0.0);
    assert_eq!(mrr_at_k(&r2, &rel, 0), 0.0);

    // Precedence: empty relevant takes precedence over k=0 for recall → 1.0.
    assert_eq!(contextual_recall(&r2, &empty, 0), 1.0);
    // (precision/nDCG/MRR agree on 0.0 at k=0 with empty relevant.)
    assert_eq!(contextual_precision(&r2, &empty, 0), 0.0);
    assert_eq!(ndcg_at_k(&r2, &empty, 0), 0.0);
    assert_eq!(mrr_at_k(&r2, &empty, 0), 0.0);
}

// ---------------------------------------------------------------------------
// §4 / §6 k boundaries: k=0, k=1, k=len, k>len
// ---------------------------------------------------------------------------
#[test]
fn k_boundaries() {
    let r = results(&[("A", "a"), ("B", "b"), ("C", "c"), ("D", "d")]);
    let rel = relevant(&[("B", "b"), ("D", "d")]);
    let len = r.len();

    // k = len → full list evaluated. Spec §6 reference values for
    // `results=[A,B,C,D]`, `relevant={B,D}`, `k=4`: precision 0.5 (B@rank2 →
    // precision@2=0.5, D@rank4 → precision@4=0.5, CP=(1/2)(0.5+0.5)=0.5);
    // recall 1.0 (top_k={A,B,C,D}, |∩|=2, |relevant|=2); nDCG ≈0.6510;
    // MRR 0.5 (first relevant B@rank2 → 1/2).
    assert_eq!(contextual_precision(&r, &rel, len), 0.5);
    assert_eq!(contextual_recall(&r, &rel, len), 1.0);
    assert_close(ndcg_at_k(&r, &rel, len), 0.6510);
    assert_eq!(mrr_at_k(&r, &rel, len), 0.5);

    // k > len → effective_top = len; identical to k = len.
    assert_eq!(contextual_precision(&r, &rel, len + 5), 0.5);
    assert_eq!(contextual_recall(&r, &rel, len + 5), 1.0);
    assert_close(ndcg_at_k(&r, &rel, len + 5), 0.6510);
    assert_eq!(mrr_at_k(&r, &rel, len + 5), 0.5);

    // k = 1 → only the first item considered.
    let r1 = results(&[("A", "a"), ("B", "b")]);
    let rel1 = relevant(&[("B", "b")]);
    assert_eq!(contextual_precision(&r1, &rel1, 1), 0.0);
    assert_eq!(contextual_recall(&r1, &rel1, 1), 0.0);
    assert_eq!(ndcg_at_k(&r1, &rel1, 1), 0.0);
    assert_eq!(mrr_at_k(&r1, &rel1, 1), 0.0);
}

// ---------------------------------------------------------------------------
// §4 duplicate-key handling: precision/MRR per-position, recall set-semantics,
// nDCG first-occurrence.
// ---------------------------------------------------------------------------
#[test]
fn duplicate_key_handling() {
    let dup = results(&[("B", "b"), ("B", "b")]);
    let rel = relevant(&[("B", "b")]);

    // Precision counts the duplicate at both positions → 1.0.
    assert_eq!(contextual_precision(&dup, &rel, 2), 1.0);
    // MRR: first relevant at rank 1 → 1.0.
    assert_eq!(mrr_at_k(&dup, &rel, 2), 1.0);
    // Recall: set semantics, duplicate does not inflate → 1.0.
    assert_eq!(contextual_recall(&dup, &rel, 2), 1.0);
    // nDCG: first-occurrence, second B contributes rel=0 → 1.0.
    assert_eq!(ndcg_at_k(&dup, &rel, 2), 1.0);

    // A duplicate relevant key that pushes other relevant keys down reduces nDCG
    // below 1 (register P-SM-1 counterexample): [A,B,C,D,B], {B,C,D}, k=5.
    let r = results(&[("A", "a"), ("B", "b"), ("C", "c"), ("D", "d"), ("B", "b")]);
    let rel2 = relevant(&[("B", "b"), ("C", "c"), ("D", "d")]);
    assert_close(ndcg_at_k(&r, &rel2, 5), 0.7328);
}

// ---------------------------------------------------------------------------
// §4 / §6 cross-cutting: no panic / no NaN / bounded in [0,1] over every §6 input.
// ---------------------------------------------------------------------------
#[test]
fn no_panic_no_nan_bounded() {
    type Case = (Vec<RagResultItem>, HashSet<(DocumentId, NodeId)>, usize);
    let cases: Vec<Case> = vec![
        (
            results(&[("B", "b"), ("D", "d")]),
            relevant(&[("B", "b"), ("D", "d")]),
            2,
        ),
        (
            results(&[("A", "a"), ("B", "b"), ("C", "c"), ("D", "d")]),
            relevant(&[("B", "b"), ("D", "d")]),
            4,
        ),
        (
            results(&[("B", "b"), ("A", "a"), ("C", "c")]),
            relevant(&[("B", "b")]),
            3,
        ),
        (
            results(&[("A", "a"), ("B", "b"), ("C", "c")]),
            relevant(&[("B", "b")]),
            3,
        ),
        (
            results(&[("B", "b"), ("A", "a"), ("C", "c"), ("D", "d")]),
            relevant(&[("B", "b"), ("D", "d")]),
            2,
        ),
        (
            results(&[("B", "b"), ("D", "d")]),
            relevant(&[("B", "b"), ("D", "d")]),
            10,
        ),
        (
            results(&[("B", "b"), ("A", "a")]),
            relevant(&[("B", "b")]),
            1,
        ),
        (
            results(&[("A", "a"), ("B", "b")]),
            relevant(&[("B", "b")]),
            1,
        ),
        (
            results(&[("A", "a"), ("B", "b"), ("C", "c")]),
            relevant(&[("D", "d")]),
            3,
        ),
        (
            results(&[("A", "a"), ("B", "b"), ("C", "c")]),
            relevant(&[]),
            3,
        ),
        (results(&[]), relevant(&[("B", "b")]), 3),
        (
            results(&[("B", "b"), ("A", "a")]),
            relevant(&[("B", "b")]),
            0,
        ),
        (
            results(&[("B", "b"), ("B", "b")]),
            relevant(&[("B", "b")]),
            2,
        ),
        (
            results(&[("A", "a"), ("C", "c")]),
            relevant(&[("B", "b")]),
            2,
        ),
        (
            results(&[("B", "b"), ("A", "a"), ("C", "c")]),
            relevant(&[("B", "b"), ("C", "c")]),
            1,
        ),
        (
            results(&[("A", "a"), ("B", "b"), ("C", "c"), ("D", "d"), ("B", "b")]),
            relevant(&[("B", "b"), ("C", "c"), ("D", "d")]),
            5,
        ),
    ];
    for (r, rel, k) in cases {
        for v in [
            contextual_precision(&r, &rel, k),
            contextual_recall(&r, &rel, k),
            ndcg_at_k(&r, &rel, k),
            mrr_at_k(&r, &rel, k),
        ] {
            assert!(v.is_finite(), "non-finite metric value {v}");
            assert!((0.0..=1.0).contains(&v), "metric value {v} out of [0,1]");
        }
    }
}

// ---------------------------------------------------------------------------
// §8 corpus-parsing + metric-computation path (the bin's smoke, via public fns).
// The `gnosis-eval` bin is a RED stub, so the harness exercises the corpus
// schema + the metric-computation path through the public metric fns instead of
// invoking the (panicking) stub bin.
// ---------------------------------------------------------------------------

/// The §8 corpus schema (a JSON array of case objects).
#[derive(Deserialize)]
struct CorpusCase {
    query: String,
    wiki_id: String,
    mode: String,
    top_k: usize,
    relevant: Vec<(String, String)>,
}

#[test]
fn corpus_parsing_and_metric_path() {
    let raw = include_str!("fixtures/eval_corpus.json");
    let cases: Vec<CorpusCase> =
        serde_json::from_str(raw).expect("corpus must parse per §8 schema");

    // Schema invariants (§8): non-empty query, top_k in 1..=50, lowercase mode,
    // relevant is an array of [doc-id, node-id] pairs.
    assert!(!cases.is_empty(), "corpus must be non-empty");
    let mut modes: HashSet<String> = HashSet::new();
    let mut has_empty_relevant = false;
    let mut has_multi_relevant = false;
    for c in &cases {
        assert!(!c.query.is_empty(), "query must be non-empty");
        assert!(!c.wiki_id.is_empty(), "wiki_id must be non-empty");
        assert!((1..=50).contains(&c.top_k), "top_k must be in 1..=50");
        assert!(
            matches!(c.mode.as_str(), "flat" | "graph" | "vector" | "hybrid"),
            "mode must be a lowercase QueryMode string, got {}",
            c.mode
        );
        modes.insert(c.mode.clone());
        if c.relevant.is_empty() {
            has_empty_relevant = true;
        }
        if c.relevant.len() > 1 {
            has_multi_relevant = true;
        }
    }
    // Fixture requirements (§8): spans multiple modes, has an empty-relevant
    // case, and has a case whose relevant set is not trivially a single item.
    assert!(modes.len() >= 2, "corpus must span multiple modes");
    assert!(
        has_empty_relevant,
        "corpus must contain an empty-relevant case"
    );
    assert!(
        has_multi_relevant,
        "corpus must contain a multi-relevant case"
    );

    // Metric-computation path: for each non-empty-relevant case, build a
    // perfect-retrieval results list from the relevant keys and assert the four
    // metrics are 1.0 (exercises the public metric fns end-to-end). For the
    // empty-relevant case, assert the pinned empty-set conventions.
    for c in &cases {
        let rel: HashSet<(DocumentId, NodeId)> =
            c.relevant.iter().map(|(d, n)| (did(d), nid(n))).collect();
        if rel.is_empty() {
            let r = results(&[("X", "x")]);
            assert_eq!(contextual_precision(&r, &rel, c.top_k), 0.0);
            assert_eq!(contextual_recall(&r, &rel, c.top_k), 1.0);
            assert_eq!(ndcg_at_k(&r, &rel, c.top_k), 0.0);
            assert_eq!(mrr_at_k(&r, &rel, c.top_k), 0.0);
        } else {
            let keys: Vec<(&str, &str)> = c
                .relevant
                .iter()
                .map(|(d, n)| (d.as_str(), n.as_str()))
                .collect();
            let r = results(&keys);
            assert_eq!(contextual_precision(&r, &rel, c.top_k), 1.0);
            assert_eq!(contextual_recall(&r, &rel, c.top_k), 1.0);
            assert_eq!(ndcg_at_k(&r, &rel, c.top_k), 1.0);
            assert_eq!(mrr_at_k(&r, &rel, c.top_k), 1.0);
        }
    }
}

// ---------------------------------------------------------------------------
// §10 harness smoke — RUNS the `gnosis-eval` `[[bin]]` against the §8 corpus
// fixture and asserts the §7.3 report shape. This is a distinct concern from
// `corpus_parsing_and_metric_path` (which parses the corpus + computes metrics
// via the public fns): this test invokes the built bin itself, end-to-end.
// ---------------------------------------------------------------------------

/// §10 harness smoke: run the built `gnosis-eval` bin (default, non-`--live`, so
/// it uses the deterministic fake provider — reproducible in CI) against
/// `tests/fixtures/eval_corpus.json` and assert the §7.3 report shape: the four
/// metric field names and the aggregate block.
#[test]
fn bin_smoke_report_shape() {
    // Cargo sets `CARGO_BIN_EXE_gnosis-eval` for integration tests when a
    // `[[bin]]` named `gnosis-eval` exists (Cargo.toml §7.8).
    let bin = env!("CARGO_BIN_EXE_gnosis-eval");
    let corpus = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/eval_corpus.json"
    );

    let out = std::process::Command::new(bin)
        .arg(corpus)
        .output()
        .expect("gnosis-eval bin must run");

    // §7.2.6: exit 0 if every case ran (no hard failure).
    assert!(
        out.status.success(),
        "gnosis-eval must exit 0, got {:?}; stderr: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );

    let stdout = String::from_utf8_lossy(&out.stdout);

    // §7.3: the report must contain the four metric field names.
    for field in [
        "contextual_precision",
        "contextual_recall",
        "ndcg_at_k",
        "mrr_at_k",
    ] {
        assert!(
            stdout.contains(field),
            "report must contain metric field {field:?}; stdout:\n{stdout}"
        );
    }

    // §7.3: the report must contain the aggregate block (the arithmetic mean over
    // the cases that produced a RagResult).
    assert!(
        stdout.contains("aggregate (mean over"),
        "report must contain the aggregate block; stdout:\n{stdout}"
    );
}

// ---------------------------------------------------------------------------
// §8 `mode` serde note: `QueryMode` has no `#[serde(rename_all)]`, so its serde
// representation is PascalCase; the lowercase corpus string must be mapped to the
// variant manually, NOT deserialized directly.
// ---------------------------------------------------------------------------
#[test]
fn query_mode_serde_pascal_case() {
    // Lowercase "flat" must NOT deserialize into QueryMode.
    assert!(
        serde_json::from_str::<QueryMode>("\"flat\"").is_err(),
        "lowercase 'flat' must not deserialize into QueryMode (PascalCase serde)"
    );
    // PascalCase does deserialize.
    assert_eq!(
        serde_json::from_str::<QueryMode>("\"Flat\"").unwrap(),
        QueryMode::Flat
    );
    assert_eq!(
        serde_json::from_str::<QueryMode>("\"Graph\"").unwrap(),
        QueryMode::Graph
    );
    assert_eq!(
        serde_json::from_str::<QueryMode>("\"Vector\"").unwrap(),
        QueryMode::Vector
    );
    assert_eq!(
        serde_json::from_str::<QueryMode>("\"Hybrid\"").unwrap(),
        QueryMode::Hybrid
    );

    // The lowercase→variant manual mapping (the bin's §8 approach) works.
    fn map_mode(s: &str) -> QueryMode {
        match s {
            "flat" => QueryMode::Flat,
            "graph" => QueryMode::Graph,
            "vector" => QueryMode::Vector,
            "hybrid" => QueryMode::Hybrid,
            _ => panic!("unknown mode {s}"),
        }
    }
    assert_eq!(map_mode("flat"), QueryMode::Flat);
    assert_eq!(map_mode("graph"), QueryMode::Graph);
    assert_eq!(map_mode("vector"), QueryMode::Vector);
    assert_eq!(map_mode("hybrid"), QueryMode::Hybrid);
}
