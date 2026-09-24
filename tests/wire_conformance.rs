//! §7.2 F2 — Engine wire contract conformance suite (`tests/wire_conformance.rs`).
//!
//! TestWriter-derived from `docs/specs/engine-wire-contract.md` (the F2 behavior
//! contract) ALONE. Covers every contract state and fail-state: the §5 21-variant
//! `StoreError::wire_code` table, §6 codecs round-trips, §7 decode-then-validate,
//! §8 single-event SSE, §9 health, §10/§4.1 envelope, and the §12 golden vectors
//! V-1..V-9 byte-exact.
//!
//! **RED-stage.** This suite compiles against the stubs in `src/wire/` (whose
//! bodies are `todo!()`/`Err` placeholders) and FAILS at runtime — the failing
//! red set. The Implementer replaces the stub bodies to go green.
//!
//! **HTTP-status map is reference-only.** §11 of the contract (the wired
//! `StoreError` → HTTP-status table, incl. mandated `ConflictError` = 409) is a
//! **documentation table for the Astrographrer shell unit** — F2 ships no HTTP
//! code and no status rendering (§2). It is NOT exercised here; there is no
//! status map in scope.

use gnosis::codecs;
use gnosis::decode;
use gnosis::decode::{DecodeError, ValidationFailure};
use gnosis::envelope::{
    current_schema_version, Envelope, CURRENT_SCHEMA_VERSION, ID_FORMAT_OPAQUE_STRING_V1,
};
use gnosis::error::{code_table, from_wire};
use gnosis::sse;
use gnosis::status;
use gnosis::{
    BlockedBy, DocumentId, EdgeKind, EngineState, EngineStatus, EngineSubsystems, GraphTraceStep,
    HybridTrace, NodeId, QueryMode, RagChunk, RagResult, RagResultItem, RagTrace, ReferenceState,
    Source, StoreError, TraceDescriptor,
};
// §9.5.2 U3 additions (probe 6: the `boot_wiring`-vs-derived coupling).
use gnosis::{
    boot_wiring, BootProvider, DerivedIndexes, EmbeddingProvider, FieldType, RagStore, Store,
    VectorIndex,
};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Fixtures / helpers
// ---------------------------------------------------------------------------

/// Enumerate every `StoreError` variant (`ValidationError` with a fixed message).
fn all_store_errors() -> Vec<StoreError> {
    vec![
        StoreError::DocumentNotFound,
        StoreError::WikiNotFound,
        StoreError::ValidationError("empty query".to_string()),
        StoreError::ConflictError,
        StoreError::DocumentInUse,
        StoreError::InvalidState,
        StoreError::UnresolvedReference,
        StoreError::CommunityNotFound,
        StoreError::CycleDetected,
        StoreError::HopLimitExceeded,
        StoreError::EngineUnavailable,
        StoreError::EngineError,
        StoreError::TraceUnavailable,
        StoreError::EmbeddingUnavailable,
        StoreError::VectorIndexUnavailable,
        StoreError::LexicalIndexUnavailable,
        StoreError::RerankerUnavailable,
        StoreError::CompressionFailed,
        StoreError::HyDEGenerationFailed,
        StoreError::MultiQueryExpansionFailed,
        StoreError::SubTaskDagFailed,
    ]
}

/// The 20 unit variants (all except `ValidationError`).
fn unit_store_errors() -> Vec<StoreError> {
    all_store_errors()
        .into_iter()
        .filter(|e| !matches!(e, StoreError::ValidationError(_)))
        .collect()
}

/// The §5 wire-code table as the contract pins it: (variant, exact code).
const WIRE_CODES: &[(&str, &str)] = &[
    ("DocumentNotFound", "not_found"),
    ("WikiNotFound", "wiki_not_found"),
    ("ValidationError", "validation_error"),
    ("ConflictError", "conflict"),
    ("DocumentInUse", "doc_in_use"),
    ("InvalidState", "invalid_state"),
    ("UnresolvedReference", "unresolved_reference"),
    ("EngineUnavailable", "engine_unavailable"),
    ("EngineError", "engine_error"),
    ("TraceUnavailable", "trace_unavailable"),
    ("HopLimitExceeded", "hop_limit_exceeded"),
    ("CycleDetected", "cycle_detected"),
    ("EmbeddingUnavailable", "embedding_unavailable"),
    ("VectorIndexUnavailable", "vector_index_unavailable"),
    ("LexicalIndexUnavailable", "lexical_index_unavailable"),
    ("RerankerUnavailable", "reranker_unavailable"),
    ("CompressionFailed", "compression_failed"),
    ("HyDEGenerationFailed", "hyde_generation_failed"),
    ("MultiQueryExpansionFailed", "multi_query_expansion_failed"),
    ("CommunityNotFound", "community_not_found"),
    ("SubTaskDagFailed", "sub_task_dag_failed"),
];

fn did(id: &str) -> DocumentId {
    DocumentId(id.to_string())
}
fn nid(id: &str) -> NodeId {
    NodeId(id.to_string())
}

fn flat_trace() -> RagTrace {
    RagTrace::Flat(TraceDescriptor {
        mode: QueryMode::Flat,
        engine: "gnosis".to_string(),
        top_k: 10,
        source: Source::Local,
    })
}

fn hybrid_trace() -> RagTrace {
    RagTrace::Hybrid(HybridTrace {
        mode: QueryMode::Hybrid,
        engine: "gnosis".to_string(),
        legs: vec![
            "graph".to_string(),
            "vector".to_string(),
            "lexical".to_string(),
        ],
        top_k: 10,
        source: Source::Local,
    })
}

fn item(doc: &str, node: &str, score: f64) -> RagResultItem {
    RagResultItem {
        document_id: did(doc),
        node_id: nid(node),
        score,
        snippet: "s".to_string(),
        source: Source::Local,
        parent: None,
        stale: None,
    }
}

/// The §12 V-5 minimal flat `RagResult` (`query:"q"`, one item, `engine:"gnosis"`,
/// flat trace).
fn v5_result() -> RagResult {
    RagResult {
        query: "q".to_string(),
        results: vec![item("d1", "n1", 0.9)],
        engine: "gnosis".to_string(),
        citations: vec![(did("d1"), nid("n1"))],
        trace: flat_trace(),
        blocked_by: None,
    }
}

/// A well-formed `RagResult` for every trace mode (and both `blocked_by` shapes).
fn wellformed_results() -> Vec<RagResult> {
    use QueryMode::{Flat, Hybrid, Vector};
    let flat_res = RagResult {
        query: "q".to_string(),
        results: vec![item("d1", "n1", 0.5)],
        engine: "gnosis".to_string(),
        citations: vec![(did("d1"), nid("n1"))],
        trace: RagTrace::Flat(TraceDescriptor {
            mode: Flat,
            engine: "gnosis".to_string(),
            top_k: 1,
            source: Source::Local,
        }),
        blocked_by: None,
    };
    let vector_res = RagResult {
        query: "v".to_string(),
        results: vec![item("d2", "n2", 1.0)],
        engine: "gnosis".to_string(),
        citations: vec![],
        trace: RagTrace::Vector(TraceDescriptor {
            mode: Vector,
            engine: "gnosis".to_string(),
            top_k: 50,
            source: Source::Zodiac,
        }),
        blocked_by: None,
    };
    let step = GraphTraceStep {
        from: (did("d1"), nid("n1")),
        to: (did("d2"), nid("n2")),
        edge: EdgeKind::Link,
        state: ReferenceState::Resolved,
    };
    let graph_res = RagResult {
        query: "g".to_string(),
        results: vec![],
        engine: "gnosis".to_string(),
        citations: vec![],
        trace: RagTrace::Graph(vec![step]),
        blocked_by: Some(vec![BlockedBy {
            document_id: did("d1"),
            node_id: nid("n1"),
            state: ReferenceState::Broken,
        }]),
    };
    let hybrid_res = RagResult {
        query: "h".to_string(),
        results: vec![item("d3", "n3", 0.0)],
        engine: "gnosis".to_string(),
        citations: vec![(did("d3"), nid("n3"))],
        trace: RagTrace::Hybrid(HybridTrace {
            mode: Hybrid,
            engine: "gnosis".to_string(),
            legs: vec![
                "graph".to_string(),
                "vector".to_string(),
                "lexical".to_string(),
            ],
            top_k: 10,
            source: Source::Local,
        }),
        blocked_by: None,
    };
    vec![flat_res, vector_res, graph_res, hybrid_res]
}

// ---------------------------------------------------------------------------
// §5 — StoreError::wire_code (exhaustive over 21) + code_table
// ---------------------------------------------------------------------------

/// The PascalCase variant name of a `StoreError` value, so `wire_code` is
/// checked against the §5 table **by name** rather than positionally. The enum
/// (`src/store/mod.rs`) is declared in fail-state order (after `UnresolvedReference`
/// comes `CommunityNotFound`, `CycleDetected`, `HopLimitExceeded`, `EngineUnavailable`, …)
/// while the §5 table lists them in its own order (after `UnresolvedReference` comes
/// `EngineUnavailable`, `EngineError`, `TraceUnavailable`, … `CommunityNotFound` near
/// the end). `wire_code` is a per-variant map, so pairing the two lists positionally
/// would pair wrong variants with wrong codes; looking up by name is order-independent.
fn store_error_variant_name(e: &StoreError) -> &'static str {
    match e {
        StoreError::DocumentNotFound => "DocumentNotFound",
        StoreError::WikiNotFound => "WikiNotFound",
        StoreError::ValidationError(_) => "ValidationError",
        StoreError::ConflictError => "ConflictError",
        StoreError::DocumentInUse => "DocumentInUse",
        StoreError::InvalidState => "InvalidState",
        StoreError::UnresolvedReference => "UnresolvedReference",
        StoreError::CommunityNotFound => "CommunityNotFound",
        StoreError::CycleDetected => "CycleDetected",
        StoreError::HopLimitExceeded => "HopLimitExceeded",
        StoreError::EngineUnavailable => "EngineUnavailable",
        StoreError::EngineError => "EngineError",
        StoreError::TraceUnavailable => "TraceUnavailable",
        StoreError::EmbeddingUnavailable => "EmbeddingUnavailable",
        StoreError::VectorIndexUnavailable => "VectorIndexUnavailable",
        StoreError::LexicalIndexUnavailable => "LexicalIndexUnavailable",
        StoreError::RerankerUnavailable => "RerankerUnavailable",
        StoreError::CompressionFailed => "CompressionFailed",
        StoreError::HyDEGenerationFailed => "HyDEGenerationFailed",
        StoreError::MultiQueryExpansionFailed => "MultiQueryExpansionFailed",
        StoreError::SubTaskDagFailed => "SubTaskDagFailed",
    }
}

/// Every one of the 21 variants maps to the exact §5 wire-code string (looked up
/// by variant name, so §5-table order vs enum declaration order cannot mismatch).
#[test]
fn wire_code_matches_contract_table_exhaustive() {
    for e in all_store_errors() {
        let name = store_error_variant_name(&e);
        let expected = WIRE_CODES
            .iter()
            .find(|(n, _)| *n == name)
            .unwrap_or_else(|| panic!("§5 table has no row for variant {name}"))
            .1;
        assert_eq!(
            e.wire_code(),
            expected,
            "wire_code({e:?}) must equal the §5 pinned code for {name}"
        );
    }
}

/// All 21 codes are non-empty, pairwise-unique, and ASCII.
#[test]
fn wire_code_all_nonempty_unique_ascii() {
    let codes: Vec<&str> = all_store_errors().iter().map(|e| e.wire_code()).collect();
    assert_eq!(codes.len(), 21, "exactly 21 variants");
    let mut seen = std::collections::HashSet::new();
    for c in &codes {
        assert!(!c.is_empty(), "code must be non-empty");
        assert!(c.is_ascii(), "code must be ASCII");
        assert!(seen.insert(*c), "code {c:?} must be pairwise-unique");
    }
}

/// `wire_code` is a pure, stable function: repeated calls return the same code;
/// a `ValidationError(m)` has the fixed `"validation_error"` regardless of `m`.
#[test]
fn wire_code_is_stable_and_validation_error_fixed() {
    let e = StoreError::ConflictError;
    assert_eq!(e.wire_code(), e.wire_code(), "stable on repeat");
    for m in ["a", "b", "漢字", ""] {
        assert_eq!(
            StoreError::ValidationError(m.to_string()).wire_code(),
            "validation_error",
            "wire code fixed regardless of carried message"
        );
    }
    assert_eq!(
        StoreError::ValidationError("x".into()).wire_code(),
        StoreError::ValidationError("y".into()).wire_code(),
        "equal variants yield equal codes"
    );
}

/// `code_table()` returns all 21 rows matching the §5 (code, display) mapping,
/// with the codes unique / non-empty.
#[test]
fn code_table_exhaustive_and_matches_contract_codes() {
    let table = code_table();
    assert_eq!(table.len(), 21, "exactly 21 rows");
    let mut seen = std::collections::HashSet::new();
    for row in table {
        assert!(!row.code.is_empty());
        assert!(!row.variant_name.is_empty());
        assert!(seen.insert(row.code), "code {} duplicated", row.code);
    }
    // Every §5 code appears in the table.
    for (_, c) in WIRE_CODES {
        assert!(
            table.iter().any(|r| r.code == *c),
            "§5 code {c} missing from code_table"
        );
    }
}

// ---------------------------------------------------------------------------
// §5 — StoreError::from_wire reverse lookup
// ---------------------------------------------------------------------------

/// Round-trips every unit variant via `from_wire(wire_code, None)`.
#[test]
fn from_wire_roundtrips_all_unit_variants() {
    for e in unit_store_errors() {
        let code = e.wire_code();
        assert_eq!(
            from_wire(code, None),
            Some(e.clone()),
            "from_wire({code}) must reconstruct {e:?}"
        );
    }
}

/// `from_wire("validation_error", Some(m)) == Some(ValidationError(m))`.
#[test]
fn from_wire_validation_error_preserves_message() {
    let cases = ["empty query", "漢字", ""];
    for m in cases {
        assert_eq!(
            from_wire("validation_error", Some(m)),
            Some(StoreError::ValidationError(m.to_string())),
            "message {m:?} must survive"
        );
    }
}

/// Unknown / foreign / blank codes → `None`.
#[test]
fn from_wire_unknown_code_returns_none() {
    assert_eq!(from_wire("bogus", None), None);
    assert_eq!(from_wire("", None), None);
    assert_eq!(from_wire("not_found ", None), None); // not trimmed
    assert_eq!(from_wire("NOT_FOUND", None), None); // case-sensitive
    assert_eq!(from_wire("uuid-v4", Some("x")), None);
}

// ---------------------------------------------------------------------------
// §6 — codecs round-tripping (all RagChunk / StoreError / RagResult variants)
// ---------------------------------------------------------------------------

/// `decode_chunk(&encode_chunk(&RagChunk::Done)) == Ok(Done)`.
#[test]
fn decode_chunk_roundtrips_done() {
    assert_eq!(
        codecs::decode_chunk(&codecs::encode_chunk(&RagChunk::Done)),
        Ok(RagChunk::Done)
    );
}

/// `decode_error(&encode_error(&e)) == Ok(e)` for every `StoreError` variant.
#[test]
fn decode_error_roundtrips_all_21_variants() {
    for e in all_store_errors() {
        assert_eq!(
            codecs::decode_error(&codecs::encode_error(&e)),
            Ok(e.clone()),
            "error round-trip failed for {e:?}"
        );
    }
}

/// `decode_chunk(&encode_chunk(&RagChunk::Error(e))) == Ok(Error(e))` for all 21.
#[test]
fn decode_chunk_roundtrips_error_all_variants() {
    for e in all_store_errors() {
        let chunk = RagChunk::Error(e.clone());
        assert_eq!(
            codecs::decode_chunk(&codecs::encode_chunk(&chunk)),
            Ok(chunk),
            "chunk(Error) round-trip failed for {e:?}"
        );
    }
}

/// `decode_result(&encode_result(&r)) == Ok(r)` for well-formed `RagResult` of
/// all four trace shapes (incl. graph `blocked_by:Some`).
#[test]
fn decode_result_roundtrips_all_trace_shapes() {
    for r in wellformed_results() {
        assert_eq!(
            codecs::decode_result(&codecs::encode_result(&r)),
            Ok(r.clone()),
            "result round-trip failed for trace {:?}",
            std::mem::discriminant(&r.trace)
        );
    }
}

/// `decode_chunk(&encode_chunk(&RagChunk::Result(r))) == Ok(Result(r))` for the
/// well-formed corpus.
#[test]
fn decode_chunk_roundtrips_result_variants() {
    for r in wellformed_results() {
        let c = RagChunk::Result(r);
        assert_eq!(codecs::decode_chunk(&codecs::encode_chunk(&c)), Ok(c));
    }
}

// ---------------------------------------------------------------------------
// §7 — decode-then-validate
// ---------------------------------------------------------------------------

/// Malformed body (wrong field type) → `DecodeError::InvalidJson` → outcome
/// `RagChunk::Error(EngineError)` (FS-9 route, V-9).
#[test]
fn decode_malformed_body_yields_engine_error() {
    // `query` typed as a number → structurally malformed.
    let bad = serde_json::json!({
        "type": "result",
        "result": { "query": 42, "results": [], "engine": "gnosis", "citations": [], "trace": null }
    });
    let err = decode::decode_chunk_payload(&bad).expect_err("malformed body must fail");
    match err {
        DecodeError::InvalidJson(_) => {}
        other => panic!("expected InvalidJson, got {other:?}"),
    }
    assert_eq!(
        decode::outcome_of(err),
        RagChunk::Error(StoreError::EngineError),
        "FS-9 malformed body → EngineError outcome"
    );
}

/// Well-formed body missing `trace` → `DecodeError::MissingTrace` →
/// `RagChunk::Error(TraceUnavailable)` (FS-10 route, V-9).
#[test]
fn decode_missing_trace_yields_trace_unavailable() {
    let body = serde_json::json!({
        "type": "result",
        "result": { "query": "q", "results": [], "engine": "gnosis", "citations": [], "blocked_by": null }
    });
    match decode::decode_chunk_payload(&body) {
        Err(DecodeError::MissingTrace) => {}
        other => panic!("expected MissingTrace, got {other:?}"),
    }
    assert_eq!(
        decode::outcome_of(DecodeError::MissingTrace),
        RagChunk::Error(StoreError::TraceUnavailable),
        "FS-10 missing trace → TraceUnavailable outcome"
    );
}

/// Every non-`MissingTrace` `DecodeError` maps to the `EngineError` outcome (§7).
#[test]
fn outcome_of_all_non_missing_trace_errors_to_engine_error() {
    let errors: Vec<DecodeError> = vec![
        DecodeError::InvalidJson("bad".into()),
        DecodeError::UnknownType("bogus".into()),
        DecodeError::UnsupportedSchemaVersion(99),
        DecodeError::UnknownIdFormat("uuid-v4".into()),
        DecodeError::InvalidEnvelope("nope".into()),
        DecodeError::UnknownCode("x".into()),
        DecodeError::EventTypeMismatch {
            event: "result".into(),
            data_type: "done".into(),
        },
        DecodeError::ValidationFailed(ValidationFailure::WrongEngine("other".into())),
    ];
    for e in errors {
        assert_eq!(
            decode::outcome_of(e),
            RagChunk::Error(StoreError::EngineError),
            "non-MissingTrace decode error → EngineError"
        );
    }
}

/// A well-formed result body decodes to `Ok` and validates.
#[test]
fn decode_rag_result_and_validate_ok_for_wellformed() {
    for r in wellformed_results() {
        let json = serde_json::to_value(&r).expect("RagResult is Serialize");
        let decoded = decode::decode_rag_result(&json).expect("well-formed body decodes");
        assert_eq!(decoded, r);
        assert_eq!(decode::validate_rag_result(&decoded), Ok(()));
    }
}

/// `decode_rag_result` on a body with `engine != "gnosis"` still decodes (serde
/// accepts any string), then `validate_rag_result` rejects with `WrongEngine`.
#[test]
fn validate_wrong_engine_is_rejected() {
    let mut r = v5_result();
    r.engine = "not-gnosis".to_string();
    assert_eq!(
        decode::validate_rag_result(&r),
        Err(ValidationFailure::WrongEngine("not-gnosis".to_string()))
    );
}

/// `blocked_by: Some(..)` with a non-Graph trace is rejected
/// (`BlockedByWithoutGraphTrace`).
#[test]
fn validate_blocked_by_without_graph_trace_is_rejected() {
    let mut r = v5_result(); // Flat trace
    r.blocked_by = Some(vec![BlockedBy {
        document_id: did("d1"),
        node_id: nid("n1"),
        state: ReferenceState::Broken,
    }]);
    assert_eq!(
        decode::validate_rag_result(&r),
        Err(ValidationFailure::BlockedByWithoutGraphTrace)
    );
}

/// `blocked_by: Some(..)` with a `RagTrace::Graph` trace validates.
#[test]
fn validate_blocked_by_with_graph_trace_is_ok() {
    let step = GraphTraceStep {
        from: (did("d1"), nid("n1")),
        to: (did("d2"), nid("n2")),
        edge: EdgeKind::Link,
        state: ReferenceState::Resolved,
    };
    let mut r = RagResult {
        query: "g".to_string(),
        results: vec![],
        engine: "gnosis".to_string(),
        citations: vec![],
        trace: RagTrace::Graph(vec![step]),
        blocked_by: Some(vec![BlockedBy {
            document_id: did("d1"),
            node_id: nid("n1"),
            state: ReferenceState::Broken,
        }]),
    };
    assert_eq!(decode::validate_rag_result(&r), Ok(()));
    let _ = &mut r;
}

/// `decode_result` invariant break (engine not "gnosis") surfaces as
/// `ValidationFailed(_)` (EngineError route via `outcome_of`).
#[test]
fn decode_result_invariant_break_is_validation_failed() {
    let mut r = v5_result();
    r.engine = "other".to_string();
    let env = codecs::encode_result(&r);
    match codecs::decode_result(&env) {
        Err(DecodeError::ValidationFailed(ValidationFailure::WrongEngine(_))) => {}
        other => panic!("expected ValidationFailed(WrongEngine), got {other:?}"),
    }
}

/// `decode_result` with a body missing `trace` → `MissingTrace` (FS-10 route).
#[test]
fn decode_result_missing_trace_is_missing_trace() {
    let body = serde_json::json!({
        "query": "q", "results": [], "engine": "gnosis", "citations": [], "blocked_by": null
    });
    let env = Envelope::with_payload(body);
    match codecs::decode_result(&env) {
        Err(DecodeError::MissingTrace) => {}
        other => panic!("expected MissingTrace, got {other:?}"),
    }
}

/// Cross-wiki transparency: a result referencing a different wiki round-trips
/// byte-for-byte; the wire codec does not enforce wiki scope.
#[test]
fn decode_result_cross_wiki_transparent() {
    let mut r = v5_result();
    r.results = vec![
        item("d1", "n1", 0.9),
        item("dX", "nX", 0.8), // a second-wiki document id
    ];
    r.citations.push((did("dX"), nid("nX")));
    assert_eq!(
        codecs::decode_result(&codecs::encode_result(&r)),
        Ok(r),
        "codec is transparent to cross-wiki ids"
    );
}

// ---------------------------------------------------------------------------
// §8 — single-event SSE
// ---------------------------------------------------------------------------

/// `event_type` mirrors the chunk class.
#[test]
fn event_type_reflects_chunk_kind() {
    use gnosis::sse::SseEventType;
    assert_eq!(sse::event_type(&RagChunk::Done), SseEventType::Done);
    assert_eq!(
        sse::event_type(&RagChunk::Error(StoreError::EngineError)),
        SseEventType::Error
    );
    assert_eq!(
        sse::event_type(&RagChunk::Result(v5_result())),
        SseEventType::Result
    );
}

/// `decode_event(&encode_event(c)) == Ok(c)` for all three variant classes.
#[test]
fn decode_event_roundtrips_all_chunk_classes() {
    use sse::SseEventType::{Done, Error, Result};
    let chunks = vec![
        RagChunk::Done,
        RagChunk::Error(StoreError::ValidationError("empty query".into())),
        RagChunk::Error(StoreError::ConflictError),
        RagChunk::Result(v5_result()),
    ];
    let _ = (Done, Error, Result);
    for c in chunks {
        assert_eq!(
            sse::decode_event(&sse::encode_event(&c)),
            Ok(c.clone()),
            "SSE round-trip failed for {c:?}"
        );
    }
}

/// Malformed frame (no `event:`/`data:` lines) → `InvalidEnvelope`.
#[test]
fn decode_event_malformed_frame_is_invalid_envelope() {
    match sse::decode_event("not an sse frame") {
        Err(DecodeError::InvalidEnvelope(_)) => {}
        other => panic!("expected InvalidEnvelope, got {other:?}"),
    }
    match sse::decode_event("") {
        Err(DecodeError::InvalidEnvelope(_)) => {}
        other => panic!("expected InvalidEnvelope, got {other:?}"),
    }
}

/// Unparseable `data:` JSON → `InvalidJson`.
#[test]
fn decode_event_bad_data_json_is_invalid_json() {
    let frame = "event: done\ndata: {invalid\n\n";
    match sse::decode_event(frame) {
        Err(DecodeError::InvalidJson(_)) => {}
        other => panic!("expected InvalidJson, got {other:?}"),
    }
}

/// `event:` line mismatching the data `"type"` → `EventTypeMismatch`.
#[test]
fn decode_event_type_mismatch_is_event_type_mismatch() {
    let frame = "event: done\ndata: {\"type\":\"result\",\"result\":{}}\n\n";
    match sse::decode_event(frame) {
        Err(DecodeError::EventTypeMismatch { event, data_type }) => {
            assert_eq!(event, "done");
            assert_eq!(data_type, "result");
        }
        other => panic!("expected EventTypeMismatch, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// §9 — health
// ---------------------------------------------------------------------------

fn all_true_subsystems() -> EngineSubsystems {
    EngineSubsystems {
        store: true,
        graph: true,
        lexical: true,
        vector: true,
        embedding: true,
        reranker: true,
    }
}

/// §9.1 / §12 V-8.1 (U3, **amended by U5**) — the HONEST flag vector a READY
/// engine reports under the capability semantics: `store`/`graph`/`lexical` true
/// (always), `reranker` **false** in every reachable state (no reranker exists in
/// `src/`), and `vector` **true** — the post-U5 value, because a `Reachable` boot
/// now BUILDS the boot vector index and swaps an index-bearing snapshot
/// (`docs/specs/p2-gnosis-server.md` §9.5.5's contract table (1), an EMPTY index
/// included). The scope is the boot-build contract's `Reachable` + a snapshot
/// whose `vectors` is `Some`: the rule that always holds is
/// `subsystems.vector == snapshot().vectors.is_some()` (U5's `P-IM-14`), and
/// V-8.1 is that rule's `Some` instance.
fn honest_ready_subsystems() -> EngineSubsystems {
    EngineSubsystems {
        store: true,
        graph: true,
        lexical: true,
        vector: true,
        embedding: true,
        reranker: false,
    }
}

/// §9.1 / §12 V-8.2 (U3) — the HONEST flag vector a DEGRADED engine reports for
/// the provider-**`Absent`**/**`Unreachable`** boot: no `embedding` claim, no
/// index, no reranker.
///
/// **Scope (U5's reconciliation, per `docs/specs/p2-gnosis-server.md` §9.5.5's
/// move table / `docs/specs/engine-wire-contract.md` §12's V-8.2 clause):** this
/// fixture is the `Absent`/`Unreachable` instance only. U5's failed-`Reachable`-
/// build branch is ALSO `Degraded` while it **wires** the provider — its honest
/// mask is `vector:false`, `embedding:true`, `reranker:false` with the same fixed
/// `lastError` — and that mask has no golden of its own (it is asserted at lib
/// level by U5's `P-IM-15`).
fn honest_degraded_subsystems() -> EngineSubsystems {
    EngineSubsystems {
        store: true,
        graph: true,
        lexical: true,
        vector: false,
        embedding: false,
        reranker: false,
    }
}

fn ready_status() -> EngineStatus {
    EngineStatus {
        state: EngineState::Ready,
        version: "0.1.0".to_string(),
        subsystems: all_true_subsystems(),
        last_error: None,
    }
}

/// Health is a pure, deterministic projection: equal status → equal report;
/// state/version/subsystems map verbatim; `last_error` mirrors (None here).
#[test]
fn health_deterministic_and_maps_fields() {
    let s = &ready_status();
    let h1 = status::health(s);
    let h2 = status::health(s);
    assert_eq!(h1, h2, "deterministic");
    assert_eq!(h1.schema_version, CURRENT_SCHEMA_VERSION);
    assert_eq!(h1.id_format, ID_FORMAT_OPAQUE_STRING_V1);
    assert_eq!(h1.state, EngineState::Ready);
    assert_eq!(h1.version, s.version);
    assert_eq!(h1.subsystems, s.subsystems);
    assert_eq!(h1.last_error, None, "no last_error invented");
}

/// A DEGRADED engine with an unavailable embedding subsystem + a `last_error`
/// surfaces `last_error: Some(...)` exactly as the input and projects every
/// subsystem flag faithfully (embedding stays false, reranker stays true).
#[test]
fn health_degaded_last_error_present_and_mirrors_input() {
    let s = EngineStatus {
        state: EngineState::Degraded,
        version: "0.1.0".to_string(),
        subsystems: EngineSubsystems {
            store: true,
            graph: true,
            lexical: true,
            vector: true,
            embedding: false,
            reranker: true,
        },
        last_error: Some("a non-core subsystem (embedding/reranker) is unavailable".to_string()),
    };
    let h = status::health(&s);
    assert_eq!(h.state, EngineState::Degraded);
    assert!(
        !h.subsystems.embedding,
        "embedding was false in the input and stays false (faithful projection)"
    );
    assert!(
        h.subsystems.reranker,
        "reranker was true in the input and stays true (faithful projection)"
    );
    assert_eq!(
        h.last_error,
        Some("a non-core subsystem (embedding/reranker) is unavailable".to_string())
    );
    // Faithful projection: `Some` iff input `Some`.
    assert_eq!(h.last_error.is_some(), s.last_error.is_some());
}

/// Health never invents a `last_error` for a Starting / Ready status even if the
/// caller left the field as `None`.
#[test]
fn health_never_invents_last_error() {
    for state in [
        EngineState::Ready,
        EngineState::Starting,
        EngineState::Unavailable,
    ] {
        let s = EngineStatus {
            state,
            version: "0.1.0".to_string(),
            subsystems: all_true_subsystems(),
            last_error: None,
        };
        assert_eq!(
            status::health(&s).last_error,
            None,
            "no last_error for {state:?} with None input"
        );
    }
}

// ---------------------------------------------------------------------------
// §10 / §4.1 — envelope
// ---------------------------------------------------------------------------

/// `Envelope` round-trips `schema_version`/`id_format`/`payload` (deep Value
/// equality) through `to_json`/`from_json`.
#[test]
fn envelope_to_from_json_roundtrip_preserves_fields() {
    let env = Envelope {
        schema_version: current_schema_version(),
        id_format: ID_FORMAT_OPAQUE_STRING_V1.to_string(),
        payload: serde_json::json!({"type": "done"}),
    };
    let json = env.to_json().expect("to_json cannot fail on our payload");
    let back = Envelope::from_json(&json).expect("round-trip");
    assert_eq!(
        back, env,
        "all three fields preserved (deep payload equality)"
    );
}

/// `with_payload` sets `schema_version=1` and `id_format="opaque-string-v1"`.
#[test]
fn envelope_with_payload_sets_defaults() {
    let env = Envelope::with_payload(serde_json::json!({"type":"done"}));
    assert_eq!(env.schema_version, CURRENT_SCHEMA_VERSION);
    assert_eq!(env.id_format, ID_FORMAT_OPAQUE_STRING_V1);
    assert_eq!(env.payload, serde_json::json!({"type":"done"}));
}

/// Unparseable input to `from_json` → `InvalidJson`.
#[test]
fn envelope_from_json_parse_error_is_invalid_json() {
    match Envelope::from_json("{ not json ") {
        Err(DecodeError::InvalidJson(_)) => {}
        other => panic!("expected InvalidJson, got {other:?}"),
    }
}

/// Missing / wrongly-typed top-level field to `from_json` → `InvalidEnvelope`.
#[test]
fn envelope_from_json_missing_field_is_invalid_envelope() {
    // No `payload`.
    let bad = r#"{"schemaVersion":1,"idFormat":"opaque-string-v1"}"#;
    match Envelope::from_json(bad) {
        Err(DecodeError::InvalidEnvelope(_)) => {}
        other => panic!("expected InvalidEnvelope, got {other:?}"),
    }
    // `schemaVersion` wrongly typed (string).
    let bad2 = r#"{"schemaVersion":"1","idFormat":"opaque-string-v1","payload":{}}"#;
    match Envelope::from_json(bad2) {
        Err(DecodeError::InvalidEnvelope(_)) => {}
        other => panic!("expected InvalidEnvelope, got {other:?}"),
    }
}

/// `from_json` does not itself validate schema/id_format; that is the decode
/// step's job. A raw envelope with an unknown id_format still parses.
#[test]
fn envelope_from_json_does_not_validate_id_format_or_schema() {
    let ok = Envelope::from_json(
        r#"{"schemaVersion":99,"idFormat":"uuid-v4","payload":{"type":"done"}}"#,
    );
    assert!(
        ok.is_ok(),
        "from_json only shape-checks; schema/id_format validation is the decode step"
    );
}

/// `from_json` must reject a `schemaVersion` outside the `u32` range rather than
/// truncate it: `4294967297` (2^32 + 1) must not wrap to a passable `1`, and a
/// negative `-1` must be rejected outright.
#[test]
fn envelope_from_json_rejects_oversized_and_negative_schema_version() {
    for bad in [
        r#"{"schemaVersion":4294967297,"idFormat":"opaque-string-v1","payload":{}}"#,
        r#"{"schemaVersion":-1,"idFormat":"opaque-string-v1","payload":{}}"#,
    ] {
        match Envelope::from_json(bad) {
            Err(DecodeError::InvalidEnvelope(_)) => {}
            Ok(env) => panic!(
                "schemaVersion must be rejected, got Ok(schema={})",
                env.schema_version
            ),
            other => panic!("expected InvalidEnvelope, got {other:?}"),
        }
    }
}

/// `decode_chunk_payload` accepts `RagChunk::Done` only for the exact
/// `{"type":"done"}` shape; extra keys make the done frame malformed
/// (A1) rather than silently ignored.
#[test]
fn decode_done_with_extra_keys_is_malformed() {
    let bad = serde_json::json!({ "type": "done", "garbage": true });
    match decode::decode_chunk_payload(&bad) {
        Err(DecodeError::InvalidEnvelope(_)) => {}
        other => panic!("expected InvalidEnvelope, got {other:?}"),
    }
    // The exact canonical shape still decodes to `Ok(RagChunk::Done)`.
    assert_eq!(
        decode::decode_chunk_payload(&serde_json::json!({ "type": "done" })),
        Ok(RagChunk::Done)
    );
}

/// Decoding a result envelope with an unknown `id_format` →
/// `UnknownIdFormat` → `EngineError` outcome (RFC-4122 seam, not implemented).
#[test]
fn decode_unknown_id_format_yields_unknown_id_format_and_engine_error() {
    let body = serde_json::to_value(v5_result()).expect("Serialize");
    let env = Envelope {
        schema_version: current_schema_version(),
        id_format: "uuid-v4".to_string(),
        payload: body,
    };
    match codecs::decode_result(&env) {
        Err(DecodeError::UnknownIdFormat(s)) => assert_eq!(s, "uuid-v4"),
        other => panic!("expected UnknownIdFormat, got {other:?}"),
    }
    assert_eq!(
        decode::outcome_of(DecodeError::UnknownIdFormat("uuid-v4".into())),
        RagChunk::Error(StoreError::EngineError)
    );
}

/// Decoding a result envelope with an unknown `schema_version` →
/// `UnsupportedSchemaVersion(99)` → `EngineError` outcome.
#[test]
fn decode_unknown_schema_version_yields_unsupported_and_engine_error() {
    let body = serde_json::to_value(v5_result()).expect("Serialize");
    let env = Envelope {
        schema_version: 99,
        id_format: ID_FORMAT_OPAQUE_STRING_V1.to_string(),
        payload: body,
    };
    match codecs::decode_result(&env) {
        Err(DecodeError::UnsupportedSchemaVersion(99)) => {}
        other => panic!("expected UnsupportedSchemaVersion(99), got {other:?}"),
    }
    assert_eq!(
        decode::outcome_of(DecodeError::UnsupportedSchemaVersion(99)),
        RagChunk::Error(StoreError::EngineError)
    );
}

/// The `done` chunk payload must be exactly `{"type":"done"}`; a `result`/`error`
/// chunk with an empty payload is `InvalidEnvelope`/`UnknownType`.
#[test]
fn decode_empty_payload_result_is_invalid_or_unknown_type() {
    let env = Envelope::with_payload(serde_json::Value::Null);
    match codecs::decode_chunk(&env) {
        Err(DecodeError::InvalidEnvelope(_)) | Err(DecodeError::UnknownType(_)) => {}
        other => panic!("expected InvalidEnvelope/UnknownType, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// §12 — golden conformance vectors (byte-exact)
// ---------------------------------------------------------------------------

/// V-1 — `encode_chunk(&RagChunk::Done)`.
#[test]
fn v1_encode_chunk_done_exact() {
    const V1: &str =
        r#"{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"type":"done"}}"#;
    let env = codecs::encode_chunk(&RagChunk::Done);
    assert_eq!(env.to_json().expect("to_json"), V1);
}

/// V-2 — `encode_chunk(&RagChunk::Error(ConflictError))`.
#[test]
fn v2_encode_chunk_error_conflict_exact() {
    const V2: &str = r#"{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"type":"error","code":"conflict","message":"optimistic-concurrency conflict: stale base revision"}}"#;
    let env = codecs::encode_chunk(&RagChunk::Error(StoreError::ConflictError));
    assert_eq!(env.to_json().expect("to_json"), V2);
}

/// V-3 — `encode_chunk(&RagChunk::Error(ValidationError("empty query")))`.
#[test]
fn v3_encode_chunk_error_validation_exact() {
    const V3: &str = r#"{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"type":"error","code":"validation_error","message":"empty query"}}"#;
    let env = codecs::encode_chunk(&RagChunk::Error(StoreError::ValidationError(
        "empty query".to_string(),
    )));
    assert_eq!(env.to_json().expect("to_json"), V3);
}

/// V-4 — `encode_error(&StoreError::WikiNotFound)` (standalone error codec).
#[test]
fn v4_encode_error_wiki_not_found_exact() {
    const V4: &str = r#"{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"code":"wiki_not_found","message":"wiki not found"}}"#;
    let env = codecs::encode_error(&StoreError::WikiNotFound);
    assert_eq!(env.to_json().expect("to_json"), V4);
}

/// V-5 — `encode_result(r)` for the minimal flat `RagResult`.
#[test]
fn v5_encode_result_flat_minimal_exact() {
    const V5: &str = r#"{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"query":"q","results":[{"document_id":"d1","node_id":"n1","score":0.9,"snippet":"s","source":"Local","parent":null,"stale":null}],"engine":"gnosis","citations":[["d1","n1"]],"trace":{"Flat":{"mode":"Flat","engine":"gnosis","top_k":10,"source":"Local"}},"blocked_by":null}}"#;
    let env = codecs::encode_result(&v5_result());
    assert_eq!(
        env.to_json().expect("to_json"),
        V5,
        "V-5 must match the exact frozen-body serde bytes"
    );
}

/// V-6 — the three SSE event frames (from `encode_event`).
#[test]
fn v6_sse_event_frames_exact() {
    // Done
    assert_eq!(
        sse::encode_event(&RagChunk::Done),
        "event: done\ndata: {\"type\":\"done\"}\n\n"
    );
    // Error(ValidationError("empty query"))
    assert_eq!(
        sse::encode_event(&RagChunk::Error(StoreError::ValidationError(
            "empty query".to_string()
        ))),
        "event: error\ndata: {\"type\":\"error\",\"code\":\"validation_error\",\"message\":\"empty query\"}\n\n"
    );
    // Result(r): `event: result` + data `{"type":"result","result":<V-5 body>}`.
    let result_frame = sse::encode_event(&RagChunk::Result(v5_result()));
    assert!(result_frame.starts_with("event: result\ndata: {\"type\":\"result\",\"result\":"));
    assert!(result_frame.ends_with("\n\n"), "terminal blank line");
}

/// V-7 — three representative `wire_code` samples.
#[test]
fn v7_wire_code_samples_exact() {
    assert_eq!(StoreError::DocumentNotFound.wire_code(), "not_found");
    assert_eq!(StoreError::ConflictError.wire_code(), "conflict");
    assert_eq!(
        StoreError::EngineUnavailable.wire_code(),
        "engine_unavailable"
    );
}

/// V-8 / **V-8.1** / **V-8.2** — health reports (canonical camelCase
/// `HealthReport` JSON) with the **amended (U3) literals**.
///
/// The pre-U1 fixtures asserted the all-`true` masks (`vector: true`,
/// `reranker: true`) — the literals `docs/specs/engine-wire-contract.md` §12
/// marks as **pre-U1 masks whose `vector`/`reranker` values are known-false
/// claims** under §9.1's capability semantics. The golden edit lands **in the
/// same unit as the flag change** (§5.8 / F2 §12's golden-literal discipline;
/// U3), so this test now asserts:
///
/// * **V-8.1 (READY)** — `{store, graph, lexical, vector, embedding, reranker} =
///   {true, true, true, false, true, false}`: the READY mask with `reranker`
///   unconditionally `false` and `vector` `false` (the U3-stage variant — the
///   boot index build that flips that one value is U5's, §9.5.2's `vector` note);
/// * **V-8.2 (DEGRADED)** — `{…vector:false, embedding:false, reranker:false}`
///   with the store's fixed `lastError` string unchanged (rewording it is not
///   authorized).
///
/// V-8 is a **codec** vector: `health` is a faithful projection of whatever mask
/// the fixture supplies — so the literals are exact projections of the fixtures
/// below, and this test also pins that the DEGRADED `lastError` string is
/// byte-identical to the store's (§12).
#[test]
fn v8_health_reports_exact() {
    let ready = EngineStatus {
        state: EngineState::Ready,
        version: "…".to_string(),
        subsystems: honest_ready_subsystems(),
        last_error: None,
    };
    let ready_json =
        serde_json::to_string(&status::health(&ready)).expect("HealthReport is Serialize");
    assert_eq!(
        ready_json,
        r#"{"schemaVersion":1,"idFormat":"opaque-string-v1","state":"Ready","version":"…","subsystems":{"store":true,"graph":true,"lexical":true,"vector":true,"embedding":true,"reranker":false},"lastError":null}"#,
        "V-8.1 ready report (U5 mask: a Reachable boot over a snapshot with an index ⇒ \
         vector true, reranker false)"
    );

    let degraded = EngineStatus {
        state: EngineState::Degraded,
        version: "…".to_string(),
        subsystems: honest_degraded_subsystems(),
        last_error: Some("a non-core subsystem (embedding/reranker) is unavailable".to_string()),
    };
    let degraded_json =
        serde_json::to_string(&status::health(&degraded)).expect("HealthReport is Serialize");
    assert_eq!(
        degraded_json,
        r#"{"schemaVersion":1,"idFormat":"opaque-string-v1","state":"Degraded","version":"…","subsystems":{"store":true,"graph":true,"lexical":true,"vector":false,"embedding":false,"reranker":false},"lastError":"a non-core subsystem (embedding/reranker) is unavailable"}"#,
        "V-8.2 degraded report (amended U3 mask: no embedding claim, no index, no reranker)"
    );
}

/// **Probe 6 (audit addition) — `boot_wiring_couples_to_the_derived_read`:** the
/// V-8.1/V-8.2 literals cannot drift from the PRODUCER.
///
/// The `v8_health_reports_exact` golden above projects `honest_ready_subsystems()`
/// / `honest_degraded_subsystems()` — fixtures, i.e. the TestWriter's hand-typed
/// literals. `health` is a faithful projector, so that test pins the projection
/// but says nothing about what the STORE actually derives. This test closes that
/// seam: it applies the pinned wiring path (`boot_wiring`'s returned
/// `EngineState` + its own snapshot swap, and the provider seam only in the
/// `Reachable` case — never a `set_subsystems` mask write) to a real `Store` and
/// asserts the store's own `get_engine_status().subsystems` **equals** the
/// fixture — so a U5 change to the index build (or any flag-derivation
/// regression) fails HERE, in the same file, instead of leaving the golden
/// literals disagreeing with the producer.
#[tokio::test]
async fn boot_wiring_couples_to_the_derived_read() {
    /// A deterministic in-memory provider: the seam's PRESENCE is what the
    /// `embedding` flag tracks, so only constructibility matters here.
    struct Provider {
        available: bool,
    }

    impl EmbeddingProvider for Provider {
        fn embed(
            &self,
            _text: &str,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<f32>, StoreError>> + Send + '_>> {
            let available = self.available;
            Box::pin(async move {
                if available {
                    Ok(vec![0.0, 1.0])
                } else {
                    Err(StoreError::EmbeddingUnavailable)
                }
            })
        }
        fn is_available(&self) -> Pin<Box<dyn Future<Output = bool> + Send + '_>> {
            let available = self.available;
            Box::pin(async move { available })
        }
    }

    /// A boot snapshot: traceless (`ready_vectors == false` — the
    /// `Absent`/`Unreachable` shape and every non-boot-constructed store), or
    /// index-bearing (`ready_vectors == true` — the post-U5 `Reachable` boot
    /// shape, which the amended V-8.1 fixture projects).
    fn boot_snapshot(ready_vectors: bool) -> DerivedIndexes {
        let mut snapshot = DerivedIndexes::default();
        if ready_vectors {
            let mut vi = VectorIndex::default();
            vi.entries.insert(
                (
                    DocumentId("d1".to_string()),
                    NodeId("n1".to_string()),
                    FieldType::Full,
                ),
                vec![1.0, 0.0],
            );
            snapshot.vectors = Some(vi);
        }
        snapshot
    }

    /// Apply the pinned wiring path (never a mask write) and read the store.
    async fn derived_read(
        provider: BootProvider,
        wired: Option<Arc<dyn EmbeddingProvider>>,
        snapshot: DerivedIndexes,
    ) -> EngineSubsystems {
        let (state, _flags) = boot_wiring(provider, &snapshot);
        let store = Store::new();
        store.swap_snapshot(snapshot);
        if let Some(p) = wired {
            store.set_embedding_provider(p);
        }
        store.set_engine_state(state);
        store.get_engine_status().await.subsystems
    }

    // (1) V-8.1's producer: a REACHABLE provider is wired ⇒ `Ready` with an index
    // (the post-U5 boot shape the amended fixture projects).
    let reached = derived_read(
        BootProvider::Reachable(Arc::new(Provider { available: true })),
        Some(Arc::new(Provider { available: true })),
        boot_snapshot(true),
    )
    .await;
    assert_eq!(
        reached,
        honest_ready_subsystems(),
        "V-8.1's literal must equal what the pinned wiring path actually derives"
    );
    assert!(
        reached.embedding && reached.vector && !reached.reranker,
        "the honest READY vector is the post-U5 `embedding:true` with an index and no \
         reranker: {reached:?}"
    );

    // (2) V-8.2's producer: no provider at all ⇒ no `embedding` claim.
    let absent = derived_read(BootProvider::Absent, None, boot_snapshot(false)).await;
    assert_eq!(
        absent,
        honest_degraded_subsystems(),
        "V-8.2's literal must equal what the provider-absent wiring actually derives"
    );
    // (3) …and the CONFIGURED-but-UNREACHABLE boot derives the same honest
    // vector (the boot wires no provider in that branch), so the two boot
    // outcomes of §9.5.2's table agree with one literal.
    let unreachable = derived_read(BootProvider::Unreachable, None, boot_snapshot(false)).await;
    assert_eq!(
        unreachable,
        honest_degraded_subsystems(),
        "the Unreachable boot branch must derive the same honest vector as Absent"
    );

    // (4) the post-U5 shape (an index is wired): `vector` is `true` and nothing
    // else moves — the one-value edit §12 promised for U5, now the fixture's own
    // value, so this probe pins the same shape probe (1) exercises.
    let u5 = derived_read(
        BootProvider::Reachable(Arc::new(Provider { available: true })),
        Some(Arc::new(Provider { available: true })),
        boot_snapshot(true),
    )
    .await;
    let u5_expected = honest_ready_subsystems();
    assert_eq!(
        u5, u5_expected,
        "wiring a vector index must flip `vector` alone (the U5-time V-8.1 shape)"
    );
}

/// **`health_report_shape_frozen`** (§9.5.2 `P-SM-6`'s moved-out half, F12; F2
/// §9.1's F7 rule) — the frozen six-flag shape of `EngineSubsystems` and the
/// frozen top-level key set of `HealthReport`.
///
/// This is an **acceptance/frozen-type criterion**, not a falsifiable property
/// (a generator can only sample values that already exist), which is why it
/// lives at the **conformance layer** rather than as a U3 property row. It
/// asserts, for the health projection of a status:
///
/// * the **compile-time** field accesses — all six `EngineSubsystems` fields
///   exist and are `bool` (a removed, re-typed or added *required* field fails
///   compilation, and the `false`/`true` literals below fail the type check if a
///   field is re-typed);
/// * `serde_json::to_value(&subsystems)` yields **exactly** the six keys —
///   `store`/`graph`/`lexical`/`vector`/`embedding`/`reranker` — none added, none
///   lost, each a JSON boolean (a re-typed field serializes as a non-bool);
/// * the `HealthReport` top level carries **exactly**
///   `{schemaVersion, idFormat, state, version, subsystems, lastError}` — U3 adds
///   **no** additive `HealthReport` field (F12: the F2 §9.1 additive permission
///   stays unused), so the key set is byte-pinned here as well.
#[test]
fn health_report_shape_frozen() {
    // (1) the six flags are booleans AT COMPILE TIME (a re-typed field breaks
    // this literal; a removed one breaks the struct literal).
    let s = EngineSubsystems {
        store: false,
        graph: true,
        lexical: false,
        vector: true,
        embedding: false,
        reranker: true,
    };
    let _: bool = s.store;
    let _: bool = s.graph;
    let _: bool = s.lexical;
    let _: bool = s.vector;
    let _: bool = s.embedding;
    let _: bool = s.reranker;

    // (2) the serialized flag object has EXACTLY those six keys, all booleans.
    let flags = serde_json::to_value(&s).expect("EngineSubsystems is Serialize");
    let obj = flags
        .as_object()
        .expect("the subsystems flags serialize to a JSON object");
    let mut keys: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        vec![
            "embedding",
            "graph",
            "lexical",
            "reranker",
            "store",
            "vector"
        ],
        "EngineSubsystems is FROZEN: exactly these six keys, none added/lost"
    );
    for (k, v) in obj.iter() {
        assert!(
            v.is_boolean(),
            "flag {k:?} must stay a JSON boolean (re-typed to {v})"
        );
    }

    // (3) the HealthReport top-level key set is frozen too (U3 adds no additive
    // field: the F2 §9.1 permission stays unused — F12).
    let status_in = EngineStatus {
        state: EngineState::Degraded,
        version: "0.1.0".to_string(),
        subsystems: honest_degraded_subsystems(),
        last_error: None,
    };
    let report = serde_json::to_value(status::health(&status_in)).expect("HealthReport Serialize");
    let report_obj = report.as_object().expect("HealthReport is a JSON object");
    let mut report_keys: Vec<&str> = report_obj.keys().map(|k| k.as_str()).collect();
    report_keys.sort_unstable();
    assert_eq!(
        report_keys,
        vec![
            "idFormat",
            "lastError",
            "schemaVersion",
            "state",
            "subsystems",
            "version"
        ],
        "HealthReport carries exactly its pinned camelCase top-level keys — U3 adds no field"
    );
    // The nested flags keep their single-word names (unchanged by the top-level
    // camelCase rename) and the frozen six-key shape.
    let nested = report_obj["subsystems"]
        .as_object()
        .expect("health.subsystems is a JSON object");
    let mut nested_keys: Vec<&str> = nested.keys().map(|k| k.as_str()).collect();
    nested_keys.sort_unstable();
    assert_eq!(
        nested_keys,
        vec![
            "embedding",
            "graph",
            "lexical",
            "reranker",
            "store",
            "vector"
        ],
        "the health report's subsystems flags carry exactly the six frozen keys"
    );
}

/// V-9 — decode-then-validate vector pair: missing `trace` → TraceUnavailable;
/// truncated/`null` body → EngineError.
#[test]
fn v9_decode_then_validate_routes_exact() {
    let r = v5_result();
    // (a) missing trace → MissingTrace → TraceUnavailable.
    let mut no_trace = serde_json::to_value(&r).expect("Serialize");
    no_trace.as_object_mut().unwrap().remove("trace");
    match decode::decode_rag_result(&no_trace) {
        Err(DecodeError::MissingTrace) => {}
        other => panic!("expected MissingTrace, got {other:?}"),
    }
    assert_eq!(
        decode::outcome_of(DecodeError::MissingTrace),
        RagChunk::Error(StoreError::TraceUnavailable)
    );
    // (b) truncated / null body → InvalidJson → EngineError.
    match decode::decode_rag_result(&serde_json::Value::Null) {
        Err(DecodeError::InvalidJson(_)) => {}
        other => panic!("expected InvalidJson, got {other:?}"),
    }
    assert_eq!(
        decode::outcome_of(DecodeError::InvalidJson("x".into())),
        RagChunk::Error(StoreError::EngineError)
    );
}

// ---------------------------------------------------------------------------
// Adversarial negative-generator probes (audit-recommended; assert the current
// `src/wire/` behavior per `engine-wire-contract.md` §7/§8/§9/§13). Each asserts
// a real error variant / faithful-projection on a crafted bad/contradictory input
// that the happy-path corpus does not exercise.
// ---------------------------------------------------------------------------

/// P-NEG-1 — mode-mismatched trace + `blocked_by`: a Flat and a Hybrid trace
/// carrying `blocked_by: Some(..)` is rejected by `validate_rag_result` with
/// `BlockedByWithoutGraphTrace`, surfaces as `ValidationFailed(_)` on both the
/// `decode_result` and `decode_chunk` paths, and `outcome_of` maps it to the
/// `EngineError` outcome (§7).
#[test]
fn probe_blocked_by_without_graph_across_flat_and_hybrid() {
    for trace in [flat_trace(), hybrid_trace()] {
        let mut r = v5_result();
        r.trace = trace;
        r.blocked_by = Some(vec![BlockedBy {
            document_id: did("d1"),
            node_id: nid("n1"),
            state: ReferenceState::Broken,
        }]);
        // Direct validator.
        assert_eq!(
            decode::validate_rag_result(&r),
            Err(ValidationFailure::BlockedByWithoutGraphTrace),
            "blocked_by with a non-Graph trace must be rejected"
        );
        // decode_result (envelope path) → ValidationFailed(BlockedByWithoutGraphTrace).
        assert!(
            matches!(
                codecs::decode_result(&codecs::encode_result(&r)),
                Err(DecodeError::ValidationFailed(
                    ValidationFailure::BlockedByWithoutGraphTrace
                ))
            ),
            "decode_result must surface BlockedByWithoutGraphTrace"
        );
        // decode_chunk (chunk path) → same.
        assert!(
            matches!(
                codecs::decode_chunk(&codecs::encode_chunk(&RagChunk::Result(r))),
                Err(DecodeError::ValidationFailed(
                    ValidationFailure::BlockedByWithoutGraphTrace
                ))
            ),
            "decode_chunk must surface BlockedByWithoutGraphTrace"
        );
        // outcome_of → EngineError route.
        assert_eq!(
            decode::outcome_of(DecodeError::ValidationFailed(
                ValidationFailure::BlockedByWithoutGraphTrace
            )),
            RagChunk::Error(StoreError::EngineError)
        );
    }
}

/// P-NEG-1b — trace-variant↔descriptor `mode` mismatch (a "flat result carrying
/// a HybridTrace") is NOT a pinned validation fail-state: the wire is transparent
/// to it when `blocked_by` is None, so it round-trips and validates `Ok` (the
/// current correct behavior — the contract reserves only MissingTrace /
/// WrongEngine / BlockedByWithoutGraphTrace).
#[test]
fn probe_trace_mode_mismatch_is_transparent_not_a_failure() {
    let mut r = v5_result();
    r.trace = RagTrace::Hybrid(HybridTrace {
        mode: QueryMode::Vector, // contradicts the Hybrid variant's mode — not validated
        engine: "gnosis".to_string(),
        legs: vec!["graph".to_string()],
        top_k: 1,
        source: Source::Local,
    });
    assert_eq!(decode::validate_rag_result(&r), Ok(()));
    assert_eq!(
        codecs::decode_result(&codecs::encode_result(&r)),
        Ok(r),
        "codec is transparent to trace-variant/mode mismatch"
    );
}

/// P-NEG-2 — unknown schema_version / id_format are never treated as schema-1
/// (coordinated with the implementer's A2): over-sized / negative `schemaVersion`
/// is rejected at the envelope layer, a well-formed-but-unknown `u32` schema is
/// rejected at decode (`UnsupportedSchemaVersion`), and the known non-canonical
/// `id_format:"uuid-v4"` is rejected (`UnknownIdFormat`) — each maps to the
/// `EngineError` outcome.
#[test]
fn probe_unknown_schema_version_and_id_format_are_errors() {
    // Over-sized / negative schemaVersion: rejected at from_json (A2 — no truncation).
    assert!(
        Envelope::from_json(
            r#"{"schemaVersion":4294967297,"idFormat":"opaque-string-v1","payload":{}}"#
        )
        .is_err(),
        "schemaVersion 2^32+1 must be rejected, not truncated"
    );
    assert!(
        Envelope::from_json(r#"{"schemaVersion":-1,"idFormat":"opaque-string-v1","payload":{}}"#)
            .is_err(),
        "negative schemaVersion must be rejected"
    );
    // A valid-u32-but-unknown schema (99) parses, but the decode step rejects it.
    let env99 = Envelope::from_json(
        r#"{"schemaVersion":99,"idFormat":"opaque-string-v1","payload":{"type":"done"}}"#,
    )
    .expect("99 is a valid u32 — from_json only shape-checks");
    match codecs::decode_chunk(&env99) {
        Err(DecodeError::UnsupportedSchemaVersion(99)) => {}
        other => panic!("expected UnsupportedSchemaVersion(99), got {other:?}"),
    }
    assert_eq!(
        decode::outcome_of(DecodeError::UnsupportedSchemaVersion(99)),
        RagChunk::Error(StoreError::EngineError)
    );
    // Known non-canonical id_format → UnknownIdFormat → EngineError.
    let envv4 = Envelope::from_json(
        r#"{"schemaVersion":1,"idFormat":"uuid-v4","payload":{"type":"done"}}"#,
    )
    .expect("id_format is not shape-validated at from_json");
    match codecs::decode_chunk(&envv4) {
        Err(DecodeError::UnknownIdFormat(s)) => assert_eq!(s, "uuid-v4"),
        other => panic!("expected UnknownIdFormat, got {other:?}"),
    }
    assert_eq!(
        decode::outcome_of(DecodeError::UnknownIdFormat("uuid-v4".into())),
        RagChunk::Error(StoreError::EngineError)
    );
}

/// P-NEG-3 — health is a faithful projection on contradictory input (§9): a
/// `Degraded` status that (contradictorily) has ALL subsystem flags `true` still
/// mirrors every flag and the `last_error` exactly; a `Ready` status carrying
/// `last_error: Some(..)` keeps it (never clears it).
#[test]
fn probe_health_faithful_on_contradictory_input() {
    let degraded_all_true = EngineStatus {
        state: EngineState::Degraded,
        version: "v".to_string(),
        subsystems: all_true_subsystems(), // contradictory: Degraded but every subsystem up
        last_error: Some("contradictory: degraded but every subsystem is up".to_string()),
    };
    let h = status::health(&degraded_all_true);
    assert_eq!(h.state, EngineState::Degraded);
    assert_eq!(
        h.subsystems, degraded_all_true.subsystems,
        "mirrors every flag exactly (incl. the contradictory all-true mask)"
    );
    assert_eq!(
        h.last_error,
        Some("contradictory: degraded but every subsystem is up".to_string())
    );

    let ready_with_error = EngineStatus {
        state: EngineState::Ready,
        version: "v".to_string(),
        subsystems: all_true_subsystems(),
        last_error: Some("carried across states".to_string()),
    };
    let h2 = status::health(&ready_with_error);
    assert_eq!(h2.state, EngineState::Ready);
    assert_eq!(
        h2.last_error,
        Some("carried across states".to_string()),
        "faithful projection must not clear a Some last_error on a Ready status"
    );
    assert_eq!(h2.subsystems, ready_with_error.subsystems);
}

/// P-NEG-4 — SSE `event:`/data-`type` mismatch across ALL three mismatched ordered
/// pairs, each → `EventTypeMismatch` (§8): (result, done), (done, error),
/// (error, valid-result-body).
#[test]
fn probe_sse_event_data_mismatch_all_ordered_pairs() {
    // event:result vs data type "done".
    let f1 = "event: result\ndata: {\"type\":\"done\"}\n\n";
    match sse::decode_event(f1) {
        Err(DecodeError::EventTypeMismatch { event, data_type }) => {
            assert_eq!(event, "result");
            assert_eq!(data_type, "done");
        }
        other => panic!("expected EventTypeMismatch(result/done), got {other:?}"),
    }
    // event:done vs data type "error".
    let f2 = "event: done\ndata: {\"type\":\"error\",\"code\":\"conflict\",\"message\":\"x\"}\n\n";
    match sse::decode_event(f2) {
        Err(DecodeError::EventTypeMismatch { event, data_type }) => {
            assert_eq!(event, "done");
            assert_eq!(data_type, "error");
        }
        other => panic!("expected EventTypeMismatch(done/error), got {other:?}"),
    }
    // event:error vs a valid result body (data type "result").
    let body = serde_json::to_string(&v5_result()).expect("Serialize");
    let f3 = format!("event: error\ndata: {{\"type\":\"result\",\"result\":{body}}}\n\n");
    match sse::decode_event(&f3) {
        Err(DecodeError::EventTypeMismatch { event, data_type }) => {
            assert_eq!(event, "error");
            assert_eq!(data_type, "result");
        }
        other => panic!("expected EventTypeMismatch(error/result), got {other:?}"),
    }
}

/// P-NEG-5 — error-codec edges (§5/§6): a `validation_error` codec input with NO
/// `message` field yields no variant (`from_wire` → `None`) so `decode_error` is
/// `UnknownCode`; an unknown / non-canonical code is `UnknownCode`.
#[test]
fn probe_error_codec_missing_message_and_unknown_code() {
    // validation_error with no message → from_wire → None → decode_error UnknownCode.
    assert_eq!(from_wire("validation_error", None), None);
    let env_no_msg = Envelope {
        schema_version: current_schema_version(),
        id_format: ID_FORMAT_OPAQUE_STRING_V1.to_string(),
        payload: serde_json::json!({"code": "validation_error"}),
    };
    match codecs::decode_error(&env_no_msg) {
        Err(DecodeError::UnknownCode(c)) => assert_eq!(c, "validation_error"),
        other => panic!("expected UnknownCode(validation_error), got {other:?}"),
    }
    // Unknown / non-canonical code.
    match codecs::decode_error(&Envelope::with_payload(serde_json::json!({
        "code": "bogus",
        "message": "x"
    }))) {
        Err(DecodeError::UnknownCode(c)) => assert_eq!(c, "bogus"),
        other => panic!("expected UnknownCode(bogus), got {other:?}"),
    }
}

/// P-NEG-5b — a `ValidationError` whose `message` contains the reserved-looking
/// key substrings `"code"`/`"message"`/`"data:"` survives the codec round-trip
/// byte-for-byte (field escaping is not re-read as structure).
#[test]
fn probe_validation_error_message_with_code_message_data_survives() {
    let msg = r#"{"code":1,"message":2,"note":"data: marker"}"#.to_string();
    let err = StoreError::ValidationError(msg.clone());
    assert_eq!(
        from_wire("validation_error", Some(&msg)),
        Some(err.clone()),
        "from_wire must preserve a data:-bearing message verbatim"
    );
    assert_eq!(
        codecs::decode_error(&codecs::encode_error(&err)),
        Ok(err.clone()),
        "error codec must round-trip a data:-bearing message"
    );
    assert_eq!(
        codecs::decode_chunk(&codecs::encode_chunk(&RagChunk::Error(err.clone()))),
        Ok(RagChunk::Error(err)),
        "chunk codec must round-trip a data:-bearing message"
    );
}

/// P-NEG-6 — done-chunk strictness regression (ties to implementer A1): the SSE
/// path must also reject a `{"type":"done"}` body carrying an extra field rather
/// than decode to `Ok(Done)`.
#[test]
fn probe_done_chunk_strictness_via_sse_path() {
    let extra = "event: done\ndata: {\"type\":\"done\",\"x\":1}\n\n";
    match sse::decode_event(extra) {
        Err(DecodeError::InvalidEnvelope(_)) => {}
        other => {
            panic!("expected InvalidEnvelope for extra-key done, got {other:?} — A1 must be landed")
        }
    }
    // The exact canonical done frame still decodes.
    assert_eq!(
        sse::decode_event("event: done\ndata: {\"type\":\"done\"}\n\n"),
        Ok(RagChunk::Done)
    );
}

// ---------------------------------------------------------------------------
// §12 V-15 / V-15.1 (U2) — the transport request-decode error body, byte-exact.
// ---------------------------------------------------------------------------

/// The canonical non-envelope transport decode-error body (§4.3 / §7.1):
/// exactly two keys, `code` first then `message`, no envelope wrapper.
fn v15_body(code: &str, message: &str) -> String {
    serde_json::json!({"code": code, "message": message}).to_string()
}

/// **V-15 (both bodies) + V-15.1 (the query-path body).**
///
/// The rendered body is `{"code":"<transport code>","message":"<detail>"}` with
/// `Content-Type: application/json` exactly (no `charset`) — the header and the
/// live bytes are asserted by the e2e layer against the real transport, since
/// this file has no HTTP client. Here the *bytes* are pinned AND the expected
/// bytes are derived from the mapping under test (`request_decode_code` /
/// `request_decode_message`), so the golden is red until the code/status mapping
/// lands (§9.5.1 `P-TP-3`'s F8 split). For `invalid_json` only the code, the
/// key set/order and the code's presence in the body are asserted: the carried
/// string is serde's own parse text, NOT a contract token.
#[test]
fn v15_request_decode_error_body_exact() {
    // (variant, the V-15/V-15.1 code, the transport status, the byte-exact body)
    let cases = vec![
        (
            DecodeError::UnknownMethod("bogus".to_string()),
            "unknown_method",
            422u16,
            r#"{"code":"unknown_method","message":"bogus"}"#,
        ),
        (
            DecodeError::UnknownMethod(String::new()),
            "unknown_method",
            422,
            r#"{"code":"unknown_method","message":""}"#,
        ),
        (
            DecodeError::UnsupportedSchemaVersion(99),
            "unsupported_schema_version",
            400,
            r#"{"code":"unsupported_schema_version","message":"unsupported schemaVersion: 99"}"#,
        ),
        (
            DecodeError::InvalidEnvelope("envelope payload must be a JSON object".to_string()),
            "invalid_envelope",
            400,
            r#"{"code":"invalid_envelope","message":"envelope payload must be a JSON object"}"#,
        ),
        (
            DecodeError::UnknownIdFormat("uuid-v4".to_string()),
            "unknown_id_format",
            400,
            r#"{"code":"unknown_id_format","message":"uuid-v4"}"#,
        ),
    ];

    for (e, want_code, want_status, want_body) in &cases {
        // The code/status mapping (P-TP-3's pure half) — the body is built from
        // the mapping under test, so both halves go red together.
        assert_eq!(
            gnosis::request_decode_code(e),
            Some(*want_code),
            "transport code for {e:?}"
        );
        assert_eq!(
            gnosis::request_decode_status(e),
            Some(*want_status),
            "transport status for {e:?}"
        );
        assert!(
            !want_code.is_empty(),
            "the transport code must be non-empty"
        );
        assert_ne!(*want_status, 502, "a transport status is never 502");
        let body = v15_body(
            gnosis::request_decode_code(e).expect("code"),
            &gnosis::request_decode_message(e),
        );
        assert_eq!(body, *want_body, "V-15/V-15.1 byte-exact body for {e:?}");
    }

    // V-15's second body, byte-exact against the literal (the one variant whose
    // message is guaranteed non-empty).
    assert_eq!(
        v15_body(
            "unsupported_schema_version",
            &gnosis::request_decode_message(&DecodeError::UnsupportedSchemaVersion(99))
        ),
        r#"{"code":"unsupported_schema_version","message":"unsupported schemaVersion: 99"}"#,
        "V-15 unsupported_schema_version body"
    );

    // V-15.1's body, byte-exact against the literal (F9: `invalid_envelope`
    // ALONE — `InvalidJson` is unreachable on the non-object-payload path).
    assert_eq!(
        v15_body(
            "invalid_envelope",
            &gnosis::request_decode_message(&DecodeError::InvalidEnvelope(
                "envelope payload must be a JSON object".to_string()
            ))
        ),
        r#"{"code":"invalid_envelope","message":"envelope payload must be a JSON object"}"#,
        "V-15.1 query-path non-object-payload body"
    );

    // `invalid_json`: the code + the key set/order, never the serde message text.
    let invalid_json = DecodeError::InvalidJson("bad json".to_string());
    assert_eq!(
        gnosis::request_decode_code(&invalid_json),
        Some("invalid_json")
    );
    assert_eq!(gnosis::request_decode_status(&invalid_json), Some(400));
    let body = v15_body(
        gnosis::request_decode_code(&invalid_json).expect("code"),
        &gnosis::request_decode_message(&invalid_json),
    );
    assert!(
        body.starts_with(r#"{"code":"invalid_json","message":"#),
        "body must open with the pinned key set/order: {body}"
    );
    assert!(
        body.ends_with(r#""}"#),
        "body must close with the (verbatim, possibly empty) message: {body}"
    );

    // §5.5's disjointness: no transport code is a §11 code.
    for code in [
        "invalid_json",
        "invalid_envelope",
        "unsupported_schema_version",
        "unknown_id_format",
        "unknown_method",
    ] {
        assert_eq!(
            from_wire(code, Some("m")),
            None,
            "transport code {code} must not exist in the §11 map"
        );
    }
}
