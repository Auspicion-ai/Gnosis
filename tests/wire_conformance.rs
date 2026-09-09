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

/// Every one of the 21 variants maps to the exact §5 wire-code string.
#[test]
fn wire_code_matches_contract_table_exhaustive() {
    for (e, expected) in all_store_errors()
        .into_iter()
        .zip(WIRE_CODES.iter().map(|(_, c)| *c))
    {
        assert_eq!(
            e.wire_code(),
            expected,
            "wire_code({e:?}) must equal the §5 pinned code"
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
/// surfaces `last_error: Some(...)` exactly as the input.
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
    assert!(!h.subsystems.embedding);
    assert!(!h.subsystems.reranker, "reranker stays true");
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

/// V-8 — health reports (canonical camelCase HealthReport JSON).
#[test]
fn v8_health_reports_exact() {
    let ready = EngineStatus {
        state: EngineState::Ready,
        version: "…".to_string(),
        subsystems: all_true_subsystems(),
        last_error: None,
    };
    let ready_json =
        serde_json::to_string(&status::health(&ready)).expect("HealthReport is Serialize");
    assert_eq!(
        ready_json,
        r#"{"schemaVersion":1,"idFormat":"opaque-string-v1","state":"Ready","version":"…","subsystems":{"store":true,"graph":true,"lexical":true,"vector":true,"embedding":true,"reranker":true},"lastError":null}"#,
        "V-8 ready report"
    );

    let degraded = EngineStatus {
        state: EngineState::Degraded,
        version: "…".to_string(),
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
    let degraded_json =
        serde_json::to_string(&status::health(&degraded)).expect("HealthReport is Serialize");
    assert_eq!(
        degraded_json,
        r#"{"schemaVersion":1,"idFormat":"opaque-string-v1","state":"Degraded","version":"…","subsystems":{"store":true,"graph":true,"lexical":true,"vector":true,"embedding":false,"reranker":true},"lastError":"a non-core subsystem (embedding/reranker) is unavailable"}"#,
        "V-8 degraded report"
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
