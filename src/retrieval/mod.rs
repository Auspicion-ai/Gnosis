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
/// `(documentId, nodeId)` ascending, truncated to `top_k`.
pub fn rrf_fuse(lists: &[Vec<(DocumentId, NodeId)>], top_k: usize) -> Vec<(DocumentId, NodeId)> {
    // Accumulate RRF(d) = Σ 1/(k + rank) over every list, 1-based rank = position+1.
    let mut scores: HashMap<(DocumentId, NodeId), f64> = HashMap::new();
    for list in lists {
        for (idx, item) in list.iter().enumerate() {
            let rank = (idx as f64) + 1.0;
            *scores.entry(item.clone()).or_insert(0.0) += 1.0 / (RRF_K + rank);
        }
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
