//! §7.2 F2 — `StoreError` → stable wire-code string + reverse lookup.
//! Contract: `docs/specs/engine-wire-contract.md` §5 (the 21-variant table).

use crate::store::StoreError;

/// One row of the §5 wire-code table (for docs / tests).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WireCodeRow {
    pub code: &'static str,
    pub variant_name: &'static str,
    pub display: &'static str,
}

impl StoreError {
    /// The stable, unique, non-empty wire-code string for this variant (§5).
    /// Pure, total, never panics, never empty, pairwise-unique over the 21
    /// variants. `ValidationError(m)` always yields the fixed `"validation_error"`.
    pub fn wire_code(&self) -> &'static str {
        use StoreError::*;
        match self {
            DocumentNotFound => "not_found",
            WikiNotFound => "wiki_not_found",
            ValidationError(_) => "validation_error",
            ConflictError => "conflict",
            DocumentInUse => "doc_in_use",
            InvalidState => "invalid_state",
            UnresolvedReference => "unresolved_reference",
            EngineUnavailable => "engine_unavailable",
            EngineError => "engine_error",
            TraceUnavailable => "trace_unavailable",
            HopLimitExceeded => "hop_limit_exceeded",
            CycleDetected => "cycle_detected",
            EmbeddingUnavailable => "embedding_unavailable",
            VectorIndexUnavailable => "vector_index_unavailable",
            LexicalIndexUnavailable => "lexical_index_unavailable",
            RerankerUnavailable => "reranker_unavailable",
            CompressionFailed => "compression_failed",
            HyDEGenerationFailed => "hyde_generation_failed",
            MultiQueryExpansionFailed => "multi_query_expansion_failed",
            CommunityNotFound => "community_not_found",
            SubTaskDagFailed => "sub_task_dag_failed",
        }
    }
}

/// Maps a canonical wire code → the unit variant; for `"validation_error"`
/// requires/uses `message`; `None` for any unknown/foreign code (§5 reverse
/// lookup). Bijective for the 20 unit variants and for `validation_error` paired
/// with a message.
pub fn from_wire(code: &str, message: Option<&str>) -> Option<StoreError> {
    use StoreError::*;
    Some(match code {
        "not_found" => DocumentNotFound,
        "wiki_not_found" => WikiNotFound,
        // A `validation_error` without a `message` falls through `?` to `None`,
        // which the codec surfaces as `UnknownCode("validation_error")`; there
        // is no dedicated `DecodeError` for a code with a missing required
        // message, so that fall-through (a `None`) is the intended shape.
        "validation_error" => ValidationError(message?.to_string()),
        "conflict" => ConflictError,
        "doc_in_use" => DocumentInUse,
        "invalid_state" => InvalidState,
        "unresolved_reference" => UnresolvedReference,
        "engine_unavailable" => EngineUnavailable,
        "engine_error" => EngineError,
        "trace_unavailable" => TraceUnavailable,
        "hop_limit_exceeded" => HopLimitExceeded,
        "cycle_detected" => CycleDetected,
        "embedding_unavailable" => EmbeddingUnavailable,
        "vector_index_unavailable" => VectorIndexUnavailable,
        "lexical_index_unavailable" => LexicalIndexUnavailable,
        "reranker_unavailable" => RerankerUnavailable,
        "compression_failed" => CompressionFailed,
        "hyde_generation_failed" => HyDEGenerationFailed,
        "multi_query_expansion_failed" => MultiQueryExpansionFailed,
        "community_not_found" => CommunityNotFound,
        "sub_task_dag_failed" => SubTaskDagFailed,
        _ => return None,
    })
}

/// The §5 table — the 21 (code, variant_name, display) rows. `display` mirrors
/// the store enum's `Display` text (the `message` the wire carries for a
/// `ValidationError` is dynamic, so its row uses the fixed `"validation failed"`
/// label).
pub fn code_table() -> &'static [WireCodeRow] {
    &[
        WireCodeRow {
            code: "not_found",
            variant_name: "DocumentNotFound",
            display: "document not found",
        },
        WireCodeRow {
            code: "wiki_not_found",
            variant_name: "WikiNotFound",
            display: "wiki not found",
        },
        WireCodeRow {
            code: "validation_error",
            variant_name: "ValidationError",
            // Fixed §5 label, NOT the wire `message`: the carried detail `m`
            // travels separately on the wire (`encode_error`/`encode_chunk`
            // emit `format!("{}", e)` = `m`), so this row's display is static
            // and must not be confused with the dynamic message.
            display: "validation failed",
        },
        WireCodeRow {
            code: "conflict",
            variant_name: "ConflictError",
            display: "optimistic-concurrency conflict: stale base revision",
        },
        WireCodeRow {
            code: "doc_in_use",
            variant_name: "DocumentInUse",
            display: "document is in use (referenced by another document)",
        },
        WireCodeRow {
            code: "invalid_state",
            variant_name: "InvalidState",
            display: "illegal state transition",
        },
        WireCodeRow {
            code: "unresolved_reference",
            variant_name: "UnresolvedReference",
            display: "document contains unresolved references",
        },
        WireCodeRow {
            code: "engine_unavailable",
            variant_name: "EngineUnavailable",
            display: "engine is not ready",
        },
        WireCodeRow {
            code: "engine_error",
            variant_name: "EngineError",
            display: "engine returned a malformed result",
        },
        WireCodeRow {
            code: "trace_unavailable",
            variant_name: "TraceUnavailable",
            display: "result carries no provenance trace",
        },
        WireCodeRow {
            code: "hop_limit_exceeded",
            variant_name: "HopLimitExceeded",
            display: "reference→fact resolution exceeded its hop cap",
        },
        WireCodeRow {
            code: "cycle_detected",
            variant_name: "CycleDetected",
            display: "reference→fact resolution detected a cycle",
        },
        WireCodeRow {
            code: "embedding_unavailable",
            variant_name: "EmbeddingUnavailable",
            display: "embedding provider unreachable",
        },
        WireCodeRow {
            code: "vector_index_unavailable",
            variant_name: "VectorIndexUnavailable",
            display: "vector index is not built",
        },
        WireCodeRow {
            code: "lexical_index_unavailable",
            variant_name: "LexicalIndexUnavailable",
            display: "lexical BM25 index is not built",
        },
        WireCodeRow {
            code: "reranker_unavailable",
            variant_name: "RerankerUnavailable",
            display: "reranker model unavailable",
        },
        WireCodeRow {
            code: "compression_failed",
            variant_name: "CompressionFailed",
            display: "compression failed and could not degrade",
        },
        WireCodeRow {
            code: "hyde_generation_failed",
            variant_name: "HyDEGenerationFailed",
            display: "HyDE hypothetical-doc generation failed",
        },
        WireCodeRow {
            code: "multi_query_expansion_failed",
            variant_name: "MultiQueryExpansionFailed",
            display: "multi-query expansion failed",
        },
        WireCodeRow {
            code: "community_not_found",
            variant_name: "CommunityNotFound",
            display: "community not found",
        },
        WireCodeRow {
            code: "sub_task_dag_failed",
            variant_name: "SubTaskDagFailed",
            display: "sub-task DAG decomposition failed",
        },
    ]
}
