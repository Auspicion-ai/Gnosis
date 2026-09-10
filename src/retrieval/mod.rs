//! RAG/agent-memory retrieval (§4.5) — the query modes (`flat`/`graph`/`vector`/
//! `hybrid`), the retrieval stack (lexical BM25, cross-encoder reranking,
//! multi-query, compression, HyDE, sub-task DAG), multiple vector fields, and
//! the agent-memory surface.
//!
//! This module hosts the **pure, deterministic retrieval-stack helpers** the §4.6
//! query surface routes through (and that a TestWriter can assert directly):
//! the Reciprocal Rank Fusion merge (§4.5.3, pinned and EXACT). The `RagStore`
//! trait's §4.5 seam (`rag_query`/`rag_stream`/`bm25_search`/`vector_search`)
//! lives in `store`.

use crate::store::{DocumentId, NodeId};
use std::cmp::Ordering;
use std::collections::HashMap;

// §7.8 F6 — RAG evaluation harness (the four pure deterministic metric fns).
pub mod eval;

// §7.8 F6 — re-export the metric surface so `gnosis::contextual_precision` etc.
// resolve (the paths the TestWriter calls).
pub use self::eval::{contextual_precision, contextual_recall, mrr_at_k, ndcg_at_k};

/// §4.5.3 — the RRF constant (`k = 60`). Pinned by the spec; a TestWriter
/// derives the exact merged ordering from input lists using this value.
pub const RRF_K: f64 = 60.0;

/// §4.5.3 — Reciprocal Rank Fusion: `RRF(d) = Σ_{r ∈ R} 1 / (k + rank_r(d))`
/// with `k = RRF_K`. `lists` is the set of ranked lists `R` (each list's items in
/// descending relevance, i.e. `rank` is the 1-based position). The merged result
/// is the top `top_k` items by descending RRF score; **ties are broken
/// deterministically by `(documentId, nodeId)` ascending**. This is the EXACT
/// merge rule — a TestWriter derives the merged ordering from the input lists.
///
/// ## GREEN
/// Implemented to the pinned EXACT rule (§4.5.3): each item's score is the sum
/// over lists of `1/(k + rank)`, merged descending by score with ties broken by
/// `(documentId, nodeId)` ascending, truncated to `top_k`, **and made
/// outer-order independent**: FP addition is commutative but NOT associative, so
/// accumulating a key's score in outer-list order can leave two mathematically
/// equal scores (exact ties) 1 ULP apart, flipping the id-ascending tie-break.
/// To make the score a pure function of each key's multiset of contributions, we
/// collect the contributions per key and sum **each key's own contributions in a
/// canonical (ascending-sorted) order**, so two keys with the same multiset
/// compute byte-identical floats and exact ties are genuinely detected.
pub fn rrf_fuse(lists: &[Vec<(DocumentId, NodeId)>], top_k: usize) -> Vec<(DocumentId, NodeId)> {
    // Collect each key's individual RRF contributions 1/(k + rank), 1-based rank
    // = position + 1, so the final sum is a pure function of its contribution
    // multiset (independent of which outer lists/ranks produced them).
    let mut contribs: HashMap<(DocumentId, NodeId), Vec<f64>> = HashMap::new();
    for list in lists {
        for (idx, item) in list.iter().enumerate() {
            let rank = (idx as f64) + 1.0;
            contribs
                .entry(item.clone())
                .or_default()
                .push(1.0 / (RRF_K + rank));
        }
    }
    // Sum each key's own contributions in a canonical ascending-sorted order, so
    // identical multisets compute byte-identical floats (outer-order independent).
    let mut scores: HashMap<(DocumentId, NodeId), f64> = HashMap::new();
    for (key, mut cs) in contribs {
        cs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        scores.insert(key, cs.iter().fold(0.0, |acc, c| acc + c));
    }
    // Descending by score; ties broken by (documentId, nodeId) ascending.
    let mut ranked: Vec<((DocumentId, NodeId), f64)> = scores.into_iter().collect();
    ranked.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.0 .0.cmp(&b.0 .0))
            .then_with(|| a.0 .1.cmp(&b.0 .1))
    });
    ranked.truncate(top_k);
    ranked.into_iter().map(|(item, _)| item).collect()
}
