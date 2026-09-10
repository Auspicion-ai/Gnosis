//! §7.8 F6 — RAG evaluation harness: the four pure deterministic metric fns.
//!
//! Contract: `docs/specs/6-f6-eval-spec.md` (§3 signatures, §4 pinned
//! conventions, §5 semantics, §6 hand-computable reference values).
//!
//! **GREEN.** The four metric fns implement the pinned §4/§5 semantics exactly:
//! 1-based rank convention (`index i → rank i+1`), effective top-k
//! `min(k, results.len())`, `log2(i+1)` discount denominator, the empty-set
//! conventions (empty relevant → precision 0 / recall 1 / nDCG 0 / MRR 0; empty
//! results → all 0; `k=0` → all 0; empty-relevant precedence over `k=0` for
//! recall), and the duplicate-key handling (precision/MRR per-position, recall
//! set-semantics, nDCG first-occurrence). Every fn is total, pure, never panics,
//! and returns a finite `f64` in `[0, 1]`.

use crate::store::{DocumentId, NodeId, RagResultItem};
use std::collections::HashSet;

/// §5.1 — contextual precision. Pure, total, returns `f64` in `[0, 1]`.
///
/// `CP = (1/|R|)·Σ_{i∈R} precision@i`, where `R` = the 1-based ranks of relevant
/// items in the effective top-k and `precision@i = (# relevant in top i)/i`.
/// No relevant item in the effective top → `0.0`. Duplicate relevant keys count
/// at each position.
pub fn contextual_precision(
    results: &[RagResultItem],
    relevant: &HashSet<(DocumentId, NodeId)>,
    k: usize,
) -> f64 {
    let effective_top = k.min(results.len());
    if effective_top == 0 {
        return 0.0;
    }
    // Single pass: at each relevant position (1-based rank i = index+1),
    // `relevant_count` is the number of relevant items in the top i, so
    // precision@i = relevant_count / i. `relevant_count` is also |R|.
    let mut relevant_count = 0usize;
    let mut sum = 0.0;
    for (i, item) in results.iter().take(effective_top).enumerate() {
        if relevant.contains(&(item.document_id.clone(), item.node_id.clone())) {
            relevant_count += 1;
            sum += relevant_count as f64 / (i as f64 + 1.0);
        }
    }
    if relevant_count == 0 {
        return 0.0;
    }
    sum / relevant_count as f64
}

/// §5.2 — contextual recall. Pure, total, returns `f64` in `[0, 1]`.
///
/// `CR = |relevant ∩ top_k| / |relevant|`, where `top_k` is the set of **distinct**
/// keys among the first `effective_top` items. Empty `relevant` → `1.0` (vacuous,
/// takes precedence over `k=0`). Set semantics: duplicates do not inflate recall.
pub fn contextual_recall(
    results: &[RagResultItem],
    relevant: &HashSet<(DocumentId, NodeId)>,
    k: usize,
) -> f64 {
    if relevant.is_empty() {
        return 1.0;
    }
    let effective_top = k.min(results.len());
    if effective_top == 0 {
        return 0.0;
    }
    let mut top_keys: HashSet<(DocumentId, NodeId)> = HashSet::new();
    for item in results.iter().take(effective_top) {
        top_keys.insert((item.document_id.clone(), item.node_id.clone()));
    }
    let intersection = top_keys.intersection(relevant).count();
    intersection as f64 / relevant.len() as f64
}

/// §5.3 — nDCG@k. Pure, total, returns `f64` in `[0, 1]`.
///
/// `DCG@k = Σ rel_i/log2(i+1)` (rel_i = 1 if `results[i]` relevant, **first
/// occurrence** for duplicates); `IDCG@k = Σ_{i=1..min(k,|relevant|)} 1/log2(i+1)`;
/// `nDCG = DCG/IDCG`. `IDCG=0` (empty relevant) or `k=0` → `0.0`.
pub fn ndcg_at_k(
    results: &[RagResultItem],
    relevant: &HashSet<(DocumentId, NodeId)>,
    k: usize,
) -> f64 {
    if relevant.is_empty() || k == 0 {
        return 0.0;
    }
    let effective_top = k.min(results.len());
    // DCG with first-occurrence rule. For 0-based index `i`, the 1-based rank is
    // `i+1`, so the pinned denominator is `log2((i+1)+1) = log2(i+2)`.
    let mut seen: HashSet<(DocumentId, NodeId)> = HashSet::new();
    let mut dcg = 0.0;
    for (i, item) in results.iter().take(effective_top).enumerate() {
        let key = (item.document_id.clone(), item.node_id.clone());
        if relevant.contains(&key) && !seen.contains(&key) {
            seen.insert(key);
            dcg += 1.0 / (i as f64 + 2.0).log2();
        }
    }
    // IDCG over the ideal top `min(k, |relevant|)` ranks.
    let idcg_len = k.min(relevant.len());
    let mut idcg = 0.0;
    for i in 1..=idcg_len {
        idcg += 1.0 / (i as f64 + 1.0).log2();
    }
    if idcg == 0.0 {
        return 0.0;
    }
    dcg / idcg
}

/// §5.4 — MRR@k. Pure, total, returns `f64` in `[0, 1]`.
///
/// `MRR = 1/r` where `r` is the 1-based rank of the **first** relevant item in the
/// effective top-k; `0.0` if none (or `k=0`). Duplicate relevant keys: the first
/// occurrence wins.
pub fn mrr_at_k(
    results: &[RagResultItem],
    relevant: &HashSet<(DocumentId, NodeId)>,
    k: usize,
) -> f64 {
    if k == 0 {
        return 0.0;
    }
    let effective_top = k.min(results.len());
    for (i, item) in results.iter().take(effective_top).enumerate() {
        if relevant.contains(&(item.document_id.clone(), item.node_id.clone())) {
            return 1.0 / (i as f64 + 1.0);
        }
    }
    0.0
}
