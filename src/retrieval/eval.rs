//! §7.8 F6 — RAG evaluation harness: the four pure deterministic metric fns.
//!
//! Contract: `docs/specs/6-f6-eval-spec.md` (§3 signatures, §4 pinned
//! conventions, §5 semantics, §6 hand-computable reference values).
//!
//! **RED-STAGE STUB.** The four metric fns are declared with the EXACT contract
//! signatures but their bodies are `todo!()` placeholders. The Implementer
//! replaces the bodies to go green. The TestWriter must NOT implement the real
//! logic here.

use crate::store::{DocumentId, NodeId, RagResultItem};
use std::collections::HashSet;

/// §5.1 — contextual precision. Pure, total, returns `f64` in `[0, 1]`.
pub fn contextual_precision(
    _results: &[RagResultItem],
    _relevant: &HashSet<(DocumentId, NodeId)>,
    _k: usize,
) -> f64 {
    todo!("F6 contextual_precision not implemented (RED stub)")
}

/// §5.2 — contextual recall. Pure, total, returns `f64` in `[0, 1]`.
pub fn contextual_recall(
    _results: &[RagResultItem],
    _relevant: &HashSet<(DocumentId, NodeId)>,
    _k: usize,
) -> f64 {
    todo!("F6 contextual_recall not implemented (RED stub)")
}

/// §5.3 — nDCG@k. Pure, total, returns `f64` in `[0, 1]`.
pub fn ndcg_at_k(
    _results: &[RagResultItem],
    _relevant: &HashSet<(DocumentId, NodeId)>,
    _k: usize,
) -> f64 {
    todo!("F6 ndcg_at_k not implemented (RED stub)")
}

/// §5.4 — MRR@k. Pure, total, returns `f64` in `[0, 1]`.
pub fn mrr_at_k(
    _results: &[RagResultItem],
    _relevant: &HashSet<(DocumentId, NodeId)>,
    _k: usize,
) -> f64 {
    todo!("F6 mrr_at_k not implemented (RED stub)")
}
