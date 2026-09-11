//! §7.2 P2 — `gnosis-server` binary crate — server conformance suite
//! (`tests/gnosis_server_conformance.rs`).
//!
//! TestWriter-derived from `docs/specs/p2-gnosis-server.md` (the P2 behavior
//! contract) ALONE. Covers every contract state and fail-state:
//!   - §7.1 the server-side §11 status mapping (the 7 CRUD-reachable
//!     `StoreError` variants → status + wire code; the retrieval-trio variants);
//!   - §7.2 the NEW-2 request-decode outcome (the 5 `DecodeError` variants →
//!     400/422, never 502);
//!   - §5.2 the endpoint routing (the 14 paths, bijective; the 11 CRUD paths
//!     equal the P1a `ENGINE_ENDPOINTS` verbatim);
//!   - §8 the RBAC `caller` threading (present on the 7 mutating, absent on the
//!     4 read-only);
//!   - §10 the valid/happy + fail states per endpoint (the §11-mapped status of
//!     each endpoint's documented fail-state `StoreError`).
//!
//! **RED-stage.** The P2 `gnosis-server` bin does NOT exist yet — the pure
//! server fns `gnosis::server_status`, `gnosis::request_decode_status` and
//! `gnosis::route_bijection` are ABSENT, so this suite FAILS TO COMPILE (the
//! missing-symbol red set). The Implementer lands the least server code to
//! green, exposing these three pure fns (over the §11 map / the request-decode
//! outcome / the 14-row routing table) plus the `caller` threading.

use gnosis::wire::crud::{
    encode_crud_request, CrudMethod, CrudRequestArgs, ENGINE_ENDPOINTS,
};
use gnosis::wire::decode::DecodeError;
use gnosis::{
    CreateDocumentRequest, DocState, DocumentId, Graph, ListDocumentsFilter, StoreError,
    UpdateDocumentRequest, WikiId,
};

// ---------------------------------------------------------------------------
// The pure server fns under test (P2 §3 / §11 API notes). These do NOT exist
// yet — the missing-symbol red set.
// ---------------------------------------------------------------------------
//
// `server_status(e) -> Option<(status, code)>` — the server-side §11 mapping of
// a `StoreError` to its HTTP status + wire code. `None` for a variant with no
// §11 row (there is none among the 21; the fn is total over the §11 map).
// `request_decode_status(e) -> Option<status>` — the NEW-2 transport-level
// request-decode outcome (400/422, never 502) for a `DecodeError`.
// `route_bijection() -> &'static [(&'static str, &'static str)]` — the 14-row
// (path, handler) routing table (11 CRUD + 3 retrieval), a bijection.
use gnosis::{request_decode_status, route_bijection, server_status};

// ---------------------------------------------------------------------------
// Fixtures / helpers
// ---------------------------------------------------------------------------

fn did(id: &str) -> DocumentId {
    DocumentId(id.to_string())
}
fn wid(id: &str) -> WikiId {
    WikiId(id.to_string())
}

/// The 7 CRUD-reachable `StoreError` variants (§7.1 / P1a §6.1).
fn crud_reachable_errors() -> Vec<StoreError> {
    vec![
        StoreError::DocumentNotFound,
        StoreError::WikiNotFound,
        StoreError::ValidationError("x".to_string()),
        StoreError::ConflictError,
        StoreError::DocumentInUse,
        StoreError::InvalidState,
        StoreError::UnresolvedReference,
    ]
}

/// The retrieval-trio reachable `StoreError` variants (§7.1).
fn retrieval_trio_errors() -> Vec<StoreError> {
    vec![
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

/// The 5 request-decode `DecodeError` variants (§7.2 / P1a §6.2).
fn decode_error_variants() -> Vec<DecodeError> {
    vec![
        DecodeError::InvalidJson("bad json".to_string()),
        DecodeError::InvalidEnvelope("missing args".to_string()),
        DecodeError::UnknownMethod("bogus".to_string()),
        DecodeError::UnsupportedSchemaVersion(99),
        DecodeError::UnknownIdFormat("uuid-v4".to_string()),
    ]
}

/// The 7 mutating methods (require `caller`).
fn mutating_methods() -> Vec<CrudMethod> {
    use CrudMethod::*;
    vec![
        CreateDocument,
        UpdateDocument,
        DeleteDocument,
        PublishDocument,
        UnpublishDocument,
        ArchiveDocument,
        CreateWiki,
    ]
}

/// The 4 read-only methods (no `caller`).
fn read_only_methods() -> Vec<CrudMethod> {
    use CrudMethod::*;
    vec![GetDocument, ListDocuments, GetWiki, ListWikis]
}

/// A representative well-formed `args` for each method.
fn args_for(m: &CrudMethod) -> CrudRequestArgs {
    use CrudMethod::*;
    match m {
        CreateDocument => CrudRequestArgs::CreateDocument {
            caller: "user:alice".to_string(),
            wiki_id: wid("w1"),
            body: CreateDocumentRequest {
                title: "Getting Started".to_string(),
                tags: Some(vec!["guide".to_string()]),
                author: Some("alice".to_string()),
            },
        },
        GetDocument => CrudRequestArgs::GetDocument {
            document_id: did("d1"),
        },
        UpdateDocument => CrudRequestArgs::UpdateDocument {
            caller: "user:alice".to_string(),
            document_id: did("d1"),
            body: UpdateDocumentRequest {
                base_revision: 0,
                graph: empty_graph(),
                title: Some("Getting Started".to_string()),
                tags: Some(vec!["guide".to_string()]),
            },
        },
        DeleteDocument => CrudRequestArgs::DeleteDocument {
            caller: "user:alice".to_string(),
            document_id: did("d1"),
        },
        PublishDocument => CrudRequestArgs::PublishDocument {
            caller: "user:alice".to_string(),
            document_id: did("d1"),
        },
        UnpublishDocument => CrudRequestArgs::UnpublishDocument {
            caller: "user:alice".to_string(),
            document_id: did("d1"),
        },
        ArchiveDocument => CrudRequestArgs::ArchiveDocument {
            caller: "user:alice".to_string(),
            document_id: did("d1"),
        },
        ListDocuments => CrudRequestArgs::ListDocuments {
            wiki_id: wid("w1"),
            body: ListDocumentsFilter {
                state: Some(DocState::Draft),
                tag: None,
                page: Some(1),
                page_size: Some(20),
            },
        },
        CreateWiki => CrudRequestArgs::CreateWiki {
            caller: "user:alice".to_string(),
            name: "My Wiki".to_string(),
        },
        GetWiki => CrudRequestArgs::GetWiki { wiki_id: wid("w1") },
        ListWikis => CrudRequestArgs::ListWikis,
    }
}

fn empty_graph() -> Graph {
    Graph {
        nodes: vec![],
        edges: vec![],
    }
}

// ---------------------------------------------------------------------------
// §7.1 — the server-side §11 status mapping (the 7 CRUD-reachable variants).
// ---------------------------------------------------------------------------

/// §7.1 — each of the 7 CRUD-reachable `StoreError` variants maps to its pinned
/// §11 HTTP status + wire code.
#[test]
fn server_status_maps_all_7_crud_reachable_variants() {
    let expected: Vec<(StoreError, u16, &str)> = vec![
        (StoreError::DocumentNotFound, 404, "not_found"),
        (StoreError::WikiNotFound, 404, "wiki_not_found"),
        (StoreError::ValidationError("x".to_string()), 400, "validation_error"),
        (StoreError::ConflictError, 409, "conflict"),
        (StoreError::DocumentInUse, 409, "doc_in_use"),
        (StoreError::InvalidState, 409, "invalid_state"),
        (StoreError::UnresolvedReference, 422, "unresolved_reference"),
    ];
    for (e, status, code) in expected {
        let got = server_status(&e)
            .unwrap_or_else(|| panic!("server_status({e:?}) must be Some"));
        assert_eq!(got.0, status, "status for {e:?}");
        assert_eq!(got.1, code, "wire code for {e:?}");
    }
}

/// §7.1 — the retrieval-trio reachable variants map per the full §11 table.
#[test]
fn server_status_maps_retrieval_trio_variants() {
    let expected: Vec<(StoreError, u16)> = vec![
        (StoreError::EngineUnavailable, 503),
        (StoreError::EngineError, 502),
        (StoreError::TraceUnavailable, 502),
        (StoreError::EmbeddingUnavailable, 503),
        (StoreError::VectorIndexUnavailable, 503),
        (StoreError::LexicalIndexUnavailable, 503),
        (StoreError::RerankerUnavailable, 503),
        (StoreError::CompressionFailed, 500),
        (StoreError::HyDEGenerationFailed, 500),
        (StoreError::MultiQueryExpansionFailed, 500),
        (StoreError::SubTaskDagFailed, 500),
    ];
    for (e, status) in expected {
        let got = server_status(&e)
            .unwrap_or_else(|| panic!("server_status({e:?}) must be Some"));
        assert_eq!(got.0, status, "status for {e:?}");
    }
}

/// §7.1 — `server_status` is total over the 7 CRUD-reachable variants (no
/// CRUD-reachable variant is unmapped).
#[test]
fn server_status_is_total_over_crud_reachable() {
    for e in crud_reachable_errors() {
        assert!(
            server_status(&e).is_some(),
            "CRUD-reachable {e:?} must be mapped"
        );
    }
}

/// §7.1 — the server does NOT invent a wire code: `server_status(e).code ==
/// e.wire_code()` for every CRUD-reachable variant.
#[test]
fn server_status_wire_code_equals_storeerror_wire_code() {
    for e in crud_reachable_errors() {
        let (_, code) = server_status(&e)
            .unwrap_or_else(|| panic!("server_status({e:?}) must be Some"));
        assert_eq!(code, e.wire_code(), "wire-code fidelity for {e:?}");
    }
}

// ---------------------------------------------------------------------------
// §7.2 — the NEW-2 request-decode outcome (400/422, NOT 502).
// ---------------------------------------------------------------------------

/// §7.2 — each of the 5 request-decode `DecodeError` variants maps to its
/// pinned transport status (400/422).
#[test]
fn request_decode_status_maps_all_5_decode_variants() {
    let expected: Vec<(DecodeError, u16)> = vec![
        (DecodeError::InvalidJson("x".to_string()), 400),
        (DecodeError::InvalidEnvelope("x".to_string()), 400),
        (DecodeError::UnknownMethod("bogus".to_string()), 422),
        (DecodeError::UnsupportedSchemaVersion(99), 400),
        (DecodeError::UnknownIdFormat("uuid-v4".to_string()), 400),
    ];
    for (e, status) in expected {
        let got = request_decode_status(&e)
            .unwrap_or_else(|| panic!("request_decode_status({e:?}) must be Some"));
        assert_eq!(got, status, "status for {e:?}");
    }
}

/// §7.2 — the request-decode outcome is total over the 5 `DecodeError`
/// variants, and is NEVER 502 (a client 4xx, not a `StoreError`-mapped 502).
#[test]
fn request_decode_status_is_total_and_never_502() {
    for e in decode_error_variants() {
        let got = request_decode_status(&e)
            .unwrap_or_else(|| panic!("request_decode_status({e:?}) must be Some"));
        assert!(
            got == 400 || got == 422,
            "request-decode status for {e:?} must be 400/422, got {got}"
        );
        assert_ne!(got, 502, "request-decode status for {e:?} must never be 502");
    }
}

// ---------------------------------------------------------------------------
// §5.2 — the endpoint routing (the 14 paths, bijective).
// ---------------------------------------------------------------------------

/// §5.2 — the routing table has exactly 14 rows (11 CRUD + 3 retrieval), one
/// per endpoint.
#[test]
fn route_bijection_has_14_rows() {
    let table = route_bijection();
    assert_eq!(table.len(), 14, "the routing table has exactly 14 rows");
}

/// §5.2 — the 14 paths are pairwise distinct (a bijection: each path maps to
/// exactly one handler).
#[test]
fn route_bijection_paths_pairwise_distinct() {
    let table = route_bijection();
    let mut paths: Vec<&str> = Vec::new();
    for (path, _) in table {
        assert!(!path.is_empty(), "path must be non-empty");
        assert!(
            !paths.contains(path),
            "path {path} duplicated in the routing table"
        );
        paths.push(path);
    }
}

/// §5.2 — the 14 handlers are pairwise distinct (each handler reachable by
/// exactly one path).
#[test]
fn route_bijection_handlers_pairwise_distinct() {
    let table = route_bijection();
    let mut handlers: Vec<&str> = Vec::new();
    for (_, handler) in table {
        assert!(!handler.is_empty(), "handler must be non-empty");
        assert!(
            !handlers.contains(handler),
            "handler {handler} duplicated in the routing table"
        );
        handlers.push(handler);
    }
}

/// §5.2 — the 11 CRUD paths equal the P1a `ENGINE_ENDPOINTS` paths verbatim
/// (the server does NOT invent or re-derive the CRUD paths). A routing-table row
/// is a CRUD row iff its path is one of the 11 pinned `ENGINE_ENDPOINTS` paths.
#[test]
fn route_bijection_crud_paths_equal_engine_endpoints() {
    let table = route_bijection();
    let engine_paths: Vec<&str> = ENGINE_ENDPOINTS.iter().map(|(p, _)| *p).collect();
    let crud_paths: Vec<&str> = table
        .iter()
        .map(|(p, _)| *p)
        .filter(|p| engine_paths.contains(p))
        .collect();
    assert_eq!(
        crud_paths.len(),
        11,
        "exactly 11 CRUD paths in the routing table"
    );
    for p in &engine_paths {
        assert!(
            crud_paths.contains(p),
            "CRUD path {p} must be in the routing table verbatim"
        );
    }
    for p in &crud_paths {
        assert!(
            engine_paths.contains(p),
            "routing-table CRUD path {p} must equal a P1a ENGINE_ENDPOINTS path"
        );
    }
}

/// §5.2 — the 3 retrieval-trio paths are present in the routing table.
#[test]
fn route_bijection_includes_retrieval_trio() {
    let table = route_bijection();
    let paths: Vec<&str> = table.iter().map(|(p, _)| *p).collect();
    for p in ["POST /rag/query", "GET /rag/stream", "GET /engine/status"] {
        assert!(paths.contains(&p), "retrieval path {p} must be in the routing table");
    }
}

// ---------------------------------------------------------------------------
// §8 — the RBAC `caller` threading (present on the 7 mutating, absent on the 4
// read-only).
// ---------------------------------------------------------------------------

/// §8 — the 7 mutating request envelopes carry a `caller` field on `args` (the
/// server accepts the caller on mutating CRUD requests and threads it through).
#[test]
fn caller_present_on_7_mutating() {
    for m in mutating_methods() {
        let env = encode_crud_request(m.clone(), args_for(&m));
        let args = env
            .payload
            .get("args")
            .and_then(|v| v.as_object())
            .expect("args object");
        assert!(
            args.contains_key("caller"),
            "mutating {m:?} must carry caller"
        );
        assert!(
            args.get("caller").and_then(|v| v.as_str()).is_some(),
            "caller must be a string for {m:?}"
        );
    }
}

/// §8 — the 4 read-only request envelopes carry NO `caller` field on `args`.
#[test]
fn caller_absent_on_4_read_only() {
    for m in read_only_methods() {
        let env = encode_crud_request(m.clone(), args_for(&m));
        let args = env
            .payload
            .get("args")
            .and_then(|v| v.as_object())
            .expect("args object");
        assert!(
            !args.contains_key("caller"),
            "read-only {m:?} must NOT carry caller"
        );
    }
}

// ---------------------------------------------------------------------------
// §10 — the valid/happy + fail states per endpoint (the §11-mapped status of
// each endpoint's documented fail-state `StoreError`).
// ---------------------------------------------------------------------------

/// §10 — `POST /documents` (createDocument): `WikiNotFound`→404,
/// `ValidationError`→400.
#[test]
fn endpoint_create_document_fail_states() {
    assert_eq!(server_status(&StoreError::WikiNotFound).unwrap().0, 404);
    assert_eq!(
        server_status(&StoreError::ValidationError("x".to_string())).unwrap().0,
        400
    );
}

/// §10 — `GET /documents/:id` (getDocument): `DocumentNotFound`→404.
#[test]
fn endpoint_get_document_fail_states() {
    assert_eq!(server_status(&StoreError::DocumentNotFound).unwrap().0, 404);
}

/// §10 — `POST /documents/:id/update` (updateDocument): `DocumentNotFound`→404,
/// `ValidationError`→400, `ConflictError`→409.
#[test]
fn endpoint_update_document_fail_states() {
    assert_eq!(server_status(&StoreError::DocumentNotFound).unwrap().0, 404);
    assert_eq!(
        server_status(&StoreError::ValidationError("x".to_string())).unwrap().0,
        400
    );
    assert_eq!(server_status(&StoreError::ConflictError).unwrap().0, 409);
}

/// §10 — `DELETE /documents/:id` (deleteDocument): `DocumentNotFound`→404,
/// `DocumentInUse`→409.
#[test]
fn endpoint_delete_document_fail_states() {
    assert_eq!(server_status(&StoreError::DocumentNotFound).unwrap().0, 404);
    assert_eq!(server_status(&StoreError::DocumentInUse).unwrap().0, 409);
}

/// §10 — `POST /documents/:id/publish` (publishDocument): `DocumentNotFound`→404,
/// `UnresolvedReference`→422.
#[test]
fn endpoint_publish_document_fail_states() {
    assert_eq!(server_status(&StoreError::DocumentNotFound).unwrap().0, 404);
    assert_eq!(server_status(&StoreError::UnresolvedReference).unwrap().0, 422);
}

/// §10 — `POST /documents/:id/unpublish` (unpublishDocument):
/// `DocumentNotFound`→404, `InvalidState`→409.
#[test]
fn endpoint_unpublish_document_fail_states() {
    assert_eq!(server_status(&StoreError::DocumentNotFound).unwrap().0, 404);
    assert_eq!(server_status(&StoreError::InvalidState).unwrap().0, 409);
}

/// §10 — `POST /documents/:id/archive` (archiveDocument): `DocumentNotFound`→404,
/// `InvalidState`→409.
#[test]
fn endpoint_archive_document_fail_states() {
    assert_eq!(server_status(&StoreError::DocumentNotFound).unwrap().0, 404);
    assert_eq!(server_status(&StoreError::InvalidState).unwrap().0, 409);
}

/// §10 — `GET /documents` (listDocuments): `WikiNotFound`→404,
/// `ValidationError`→400.
#[test]
fn endpoint_list_documents_fail_states() {
    assert_eq!(server_status(&StoreError::WikiNotFound).unwrap().0, 404);
    assert_eq!(
        server_status(&StoreError::ValidationError("x".to_string())).unwrap().0,
        400
    );
}

/// §10 — `POST /wikis` (createWiki): `ValidationError`→400.
#[test]
fn endpoint_create_wiki_fail_states() {
    assert_eq!(
        server_status(&StoreError::ValidationError("x".to_string())).unwrap().0,
        400
    );
}

/// §10 — `GET /wikis/:id` (getWiki): `WikiNotFound`→404.
#[test]
fn endpoint_get_wiki_fail_states() {
    assert_eq!(server_status(&StoreError::WikiNotFound).unwrap().0, 404);
}

/// §10 — `POST /rag/query`: `EngineUnavailable`→503, `EngineError`→502,
/// `TraceUnavailable`→502.
#[test]
fn endpoint_rag_query_fail_states() {
    assert_eq!(server_status(&StoreError::EngineUnavailable).unwrap().0, 503);
    assert_eq!(server_status(&StoreError::EngineError).unwrap().0, 502);
    assert_eq!(server_status(&StoreError::TraceUnavailable).unwrap().0, 502);
}

/// §10 — the cross-cutting request-decode fail-states (NEW-2): a malformed
/// request / unknown method is a client 4xx, never a `StoreError`-mapped 502.
#[test]
fn cross_cutting_request_decode_fail_states() {
    // Malformed request envelope → 400.
    assert_eq!(
        request_decode_status(&DecodeError::InvalidJson("x".to_string())).unwrap(),
        400
    );
    assert_eq!(
        request_decode_status(&DecodeError::InvalidEnvelope("x".to_string())).unwrap(),
        400
    );
    // Unknown method → 422.
    assert_eq!(
        request_decode_status(&DecodeError::UnknownMethod("bogus".to_string())).unwrap(),
        422
    );
    // Unknown schema_version / id_format → 400.
    assert_eq!(
        request_decode_status(&DecodeError::UnsupportedSchemaVersion(99)).unwrap(),
        400
    );
    assert_eq!(
        request_decode_status(&DecodeError::UnknownIdFormat("uuid-v4".to_string())).unwrap(),
        400
    );
    // None of these is a StoreError; none is 502.
    for e in decode_error_variants() {
        assert_ne!(request_decode_status(&e).unwrap(), 502);
    }
}
