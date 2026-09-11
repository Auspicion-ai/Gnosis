//! §7.2 P2 — the pure server-side mapping fns (the §11 status rendering, the
//! NEW-2 request-decode outcome, and the 14-row routing table).
//!
//! These are pure synchronous fns over values — no tokio runtime, no engine
//! instance — so the TestWriter's conformance + property suites reach them as
//! `gnosis::server_status`, `gnosis::request_decode_status` and
//! `gnosis::route_bijection`. The `gnosis-server` bin consumes the same fns for
//! its live transport rendering.
//! Contract: `docs/specs/p2-gnosis-server.md` §7.1 / §7.2 / §5.2.

use crate::store::StoreError;
use crate::wire::decode::DecodeError;

/// The server-side §11 mapping of a `StoreError` to its HTTP status + wire code
/// (§7.1). Total over the §11 map (21 rows); the wire code is always
/// `e.wire_code()` (the server never invents a code). `None` for a variant with
/// no §11 row (there is none among the 21).
pub fn server_status(e: &StoreError) -> Option<(u16, &'static str)> {
    use StoreError::*;
    let status = match e {
        DocumentNotFound => 404,
        WikiNotFound => 404,
        ValidationError(_) => 400,
        ConflictError => 409,
        DocumentInUse => 409,
        InvalidState => 409,
        UnresolvedReference => 422,
        EngineUnavailable => 503,
        EngineError => 502,
        TraceUnavailable => 502,
        HopLimitExceeded => 422,
        CycleDetected => 409,
        EmbeddingUnavailable => 503,
        VectorIndexUnavailable => 503,
        LexicalIndexUnavailable => 503,
        RerankerUnavailable => 503,
        CompressionFailed => 500,
        HyDEGenerationFailed => 500,
        MultiQueryExpansionFailed => 500,
        CommunityNotFound => 404,
        SubTaskDagFailed => 500,
    };
    Some((status, e.wire_code()))
}

/// The NEW-2 transport-level request-decode outcome (§7.2): a malformed CRUD
/// request / unknown method is a client 4xx (400/422), **never** a
/// `StoreError`-mapped 502. Total over the 5 request-decode `DecodeError`
/// variants; the other `DecodeError` variants are response/SSE-side, not
/// request-decode outcomes, so they map to `None`.
pub fn request_decode_status(e: &DecodeError) -> Option<u16> {
    use DecodeError::*;
    let status = match e {
        InvalidJson(_) => 400,
        InvalidEnvelope(_) => 400,
        UnknownMethod(_) => 422,
        UnsupportedSchemaVersion(_) => 400,
        UnknownIdFormat(_) => 400,
        _ => return None,
    };
    Some(status)
}

/// The 14-row (path, handler) routing table — a bijection (§5.2): the 11 CRUD
/// paths equal the P1a `ENGINE_ENDPOINTS` paths verbatim, plus the 3 retrieval
/// trio paths. Each path maps to exactly one handler and each handler is
/// reachable by exactly one path.
pub fn route_bijection() -> &'static [(&'static str, &'static str)] {
    &[
        // 11 CRUD paths (verbatim from P1a `ENGINE_ENDPOINTS`).
        ("POST /documents", "create_document"),
        ("GET /documents/:id", "get_document"),
        ("POST /documents/:id/update", "update_document"),
        ("DELETE /documents/:id", "delete_document"),
        ("POST /documents/:id/publish", "publish_document"),
        ("POST /documents/:id/unpublish", "unpublish_document"),
        ("POST /documents/:id/archive", "archive_document"),
        ("GET /documents", "list_documents"),
        ("POST /wikis", "create_wiki"),
        ("GET /wikis/:id", "get_wiki"),
        ("GET /wikis", "list_wikis"),
        // 3 retrieval-trio paths.
        ("POST /rag/query", "rag_query"),
        ("GET /rag/stream", "rag_stream"),
        ("GET /engine/status", "engine_status"),
    ]
}
