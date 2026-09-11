//! §7.2 P1a — Document-CRUD wire-contract conformance suite
//! (`tests/crud_wire_conformance.rs`).
//!
//! TestWriter-derived from `docs/specs/p1a-document-crud-wire.md` (the P1a
//! behavior contract) ALONE. Covers every contract state and fail-state: §4.2/§4.3
//! wire shapes for all 11 methods, §6.1 `StoreError` exhaustiveness, §6.2 the
//! request-decode outcome table (400/422, NOT 502), §7 the endpoint-path
//! constants (11 rows, bijective), §8 the RBAC `caller` shape, §9 the golden
//! vectors V-10..V-14 byte-for-byte, and §10 the valid/happy + fail states per
//! function (incl. all `CrudResponseError` routes and all `CrudValidationFailure`
//! variants).
//!
//! **RED-stage.** The P1a `src/wire/crud.rs` module does NOT exist yet — the
//! `gnosis::wire::crud::{…}` surface is absent, so this suite FAILS TO COMPILE
//! (the missing-symbol red set). The Implementer lands the least `src/wire/`
//! code to green.

use gnosis::envelope::{
    current_schema_version, Envelope, CURRENT_SCHEMA_VERSION, ID_FORMAT_OPAQUE_STRING_V1,
};
use gnosis::error::code_table;
use gnosis::wire::crud::{
    encode_crud_error, encode_crud_request, encode_crud_response, decode_crud_request,
    decode_crud_response, validate_crud_result, CrudMethod, CrudRequestArgs, CrudResponseError,
    CrudResult, CrudValidationFailure, ENGINE_ENDPOINTS, ENDPOINT_ARCHIVE_DOCUMENT,
    ENDPOINT_CREATE_DOCUMENT, ENDPOINT_CREATE_WIKI, ENDPOINT_DELETE_DOCUMENT,
    ENDPOINT_GET_DOCUMENT, ENDPOINT_GET_WIKI, ENDPOINT_LIST_DOCUMENTS, ENDPOINT_LIST_WIKIS,
    ENDPOINT_PUBLISH_DOCUMENT, ENDPOINT_UNPUBLISH_DOCUMENT, ENDPOINT_UPDATE_DOCUMENT,
};
use gnosis::wire::decode::DecodeError;
use gnosis::{
    CreateDocumentRequest, DocState, Document, DocumentId, DocumentList, DocumentSummary, Graph,
    ListDocumentsFilter, StoreError, UpdateDocumentRequest, Wiki, WikiId,
};

// ---------------------------------------------------------------------------
// Fixtures / helpers
// ---------------------------------------------------------------------------

fn did(id: &str) -> DocumentId {
    DocumentId(id.to_string())
}
fn wid(id: &str) -> WikiId {
    WikiId(id.to_string())
}

/// The §4.2 `"method"` camelCase value for each `CrudMethod` variant.
fn method_str(m: &CrudMethod) -> &'static str {
    use CrudMethod::*;
    match m {
        CreateDocument => "createDocument",
        GetDocument => "getDocument",
        UpdateDocument => "updateDocument",
        DeleteDocument => "deleteDocument",
        PublishDocument => "publishDocument",
        UnpublishDocument => "unpublishDocument",
        ArchiveDocument => "archiveDocument",
        ListDocuments => "listDocuments",
        CreateWiki => "createWiki",
        GetWiki => "getWiki",
        ListWikis => "listWikis",
    }
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

/// All 11 methods.
fn all_methods() -> Vec<CrudMethod> {
    use CrudMethod::*;
    vec![
        CreateDocument,
        GetDocument,
        UpdateDocument,
        DeleteDocument,
        PublishDocument,
        UnpublishDocument,
        ArchiveDocument,
        ListDocuments,
        CreateWiki,
        GetWiki,
        ListWikis,
    ]
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

/// A representative `Document` (revision 0, Draft, empty graph).
fn draft_doc() -> Document {
    Document {
        document_id: did("d1"),
        wiki_id: wid("w1"),
        revision: 0,
        state: DocState::Draft,
        graph: empty_graph(),
        title: "Getting Started".to_string(),
        created_at: "2026-09-09T00:00:00Z".to_string(),
        updated_at: "2026-09-09T00:00:00Z".to_string(),
        tags: vec!["guide".to_string()],
        author: Some("alice".to_string()),
    }
}

/// A representative well-formed `CrudResult` for each method.
fn result_for(m: &CrudMethod) -> CrudResult {
    use CrudMethod::*;
    match m {
        CreateDocument => CrudResult::Document(draft_doc()),
        GetDocument => CrudResult::Document(draft_doc()),
        UpdateDocument => CrudResult::Document(draft_doc()),
        DeleteDocument => CrudResult::DeleteDocument,
        PublishDocument => CrudResult::Document({
            let mut d = draft_doc();
            d.state = DocState::Published;
            d
        }),
        UnpublishDocument => CrudResult::Document(draft_doc()),
        ArchiveDocument => CrudResult::Document({
            let mut d = draft_doc();
            d.state = DocState::Archived;
            d
        }),
        ListDocuments => CrudResult::DocumentList(DocumentList {
            items: vec![DocumentSummary {
                document_id: did("d1"),
                wiki_id: wid("w1"),
                title: "Getting Started".to_string(),
                state: DocState::Draft,
                revision: 0,
                updated_at: "2026-09-09T00:00:00Z".to_string(),
            }],
            total: 1,
            page: 1,
            page_size: 20,
        }),
        CreateWiki => CrudResult::Wiki(Wiki {
            wiki_id: wid("w1"),
            name: "My Wiki".to_string(),
        }),
        GetWiki => CrudResult::Wiki(Wiki {
            wiki_id: wid("w1"),
            name: "My Wiki".to_string(),
        }),
        ListWikis => CrudResult::WikiList(vec![Wiki {
            wiki_id: wid("w1"),
            name: "My Wiki".to_string(),
        }]),
    }
}

// ---------------------------------------------------------------------------
// §4.2 / §4.3 — wire shapes for all 11 methods (request + response envelopes)
// ---------------------------------------------------------------------------

/// §4.2 — every request envelope carries the camelCase `"method"` discriminator
/// and a per-method `"args"` object.
#[test]
fn request_envelope_shapes_all_11_methods() {
    for m in all_methods() {
        let env = encode_crud_request(m.clone(), args_for(&m));
        let payload = env.payload.as_object().expect("payload must be an object");
        assert_eq!(
            payload.get("method").and_then(|v| v.as_str()),
            Some(method_str(&m)),
            "method discriminator for {m:?}"
        );
        assert!(payload.contains_key("args"), "args present for {m:?}");
        // Envelope-level fields are pinned by §4.1.
        assert_eq!(env.schema_version, CURRENT_SCHEMA_VERSION);
        assert_eq!(env.id_format, ID_FORMAT_OPAQUE_STRING_V1);
    }
}

/// §4.3 — every response envelope carries the camelCase `"method"` discriminator
/// and a `"result"` field for a well-formed result.
#[test]
fn response_envelope_shapes_all_11_methods() {
    for m in all_methods() {
        let env = encode_crud_response(m.clone(), result_for(&m));
        let payload = env.payload.as_object().expect("payload must be an object");
        assert_eq!(
            payload.get("method").and_then(|v| v.as_str()),
            Some(method_str(&m)),
            "method discriminator for {m:?}"
        );
        assert!(payload.contains_key("result"), "result present for {m:?}");
    }
}

/// §4.3 — the `deleteDocument` void result encodes to `"result":null`.
#[test]
fn delete_document_void_result_is_null() {
    let env = encode_crud_response(CrudMethod::DeleteDocument, CrudResult::DeleteDocument);
    let payload = env.payload.as_object().unwrap();
    assert_eq!(payload.get("result"), Some(&serde_json::Value::Null));
}

/// §4.3 — an error response envelope carries `{"code","message"}` under `"error"`.
#[test]
fn error_envelope_shape() {
    let env = encode_crud_error(CrudMethod::UpdateDocument, &StoreError::ConflictError);
    let payload = env.payload.as_object().unwrap();
    assert_eq!(
        payload.get("method").and_then(|v| v.as_str()),
        Some("updateDocument")
    );
    let err = payload.get("error").and_then(|v| v.as_object()).unwrap();
    assert_eq!(err.get("code").and_then(|v| v.as_str()), Some("conflict"));
    assert!(err.contains_key("message"));
}

// ---------------------------------------------------------------------------
// §6.1 — exhaustiveness: the CRUD-reachable `StoreError` variants are a subset
// of the 21 §11 rows.
// ---------------------------------------------------------------------------

/// §6.1 — the 7 CRUD-reachable `StoreError` variants each have a `wire_code()`
/// present in the §11 `code_table()` (21 rows). No new §11 row is added.
#[test]
fn crud_reachable_store_errors_are_subset_of_21_rows() {
    let crud_reachable = vec![
        StoreError::DocumentNotFound,
        StoreError::WikiNotFound,
        StoreError::ValidationError("x".to_string()),
        StoreError::ConflictError,
        StoreError::DocumentInUse,
        StoreError::InvalidState,
        StoreError::UnresolvedReference,
    ];
    let table = code_table();
    assert_eq!(table.len(), 21, "the §11 map has exactly 21 rows");
    for e in &crud_reachable {
        let code = e.wire_code();
        assert!(
            table.iter().any(|row| row.code == code),
            "CRUD-reachable {e:?} wire_code {code} must be in the §11 table"
        );
    }
}

// ---------------------------------------------------------------------------
// §6.2 — the request-decode outcome table (400/422, NOT 502).
// ---------------------------------------------------------------------------

/// §6.2 — a malformed CRUD request / unknown method is a transport-level
/// `DecodeError` (client 4xx), NEVER a `StoreError`-mapped 502.
#[test]
fn request_decode_outcome_table() {
    // Unknown method → UnknownMethod → 422.
    let unknown = Envelope::with_payload(serde_json::json!({
        "method": "bogus",
        "args": {}
    }));
    match decode_crud_request(&unknown) {
        Err(DecodeError::UnknownMethod(m)) => assert_eq!(m, "bogus"),
        other => panic!("expected UnknownMethod, got {other:?}"),
    }

    // Missing "args" → InvalidEnvelope → 400.
    let no_args = Envelope::with_payload(serde_json::json!({ "method": "getDocument" }));
    assert!(
        matches!(
            decode_crud_request(&no_args),
            Err(DecodeError::InvalidEnvelope(_))
        ),
        "missing args must be InvalidEnvelope (400)"
    );

    // Unsupported schema version → UnsupportedSchemaVersion → 400.
    let bad_schema = Envelope {
        schema_version: 99,
        id_format: ID_FORMAT_OPAQUE_STRING_V1.to_string(),
        payload: serde_json::json!({ "method": "getDocument", "args": { "documentId": "d1" } }),
    };
    match decode_crud_request(&bad_schema) {
        Err(DecodeError::UnsupportedSchemaVersion(v)) => assert_eq!(v, 99),
        other => panic!("expected UnsupportedSchemaVersion(99), got {other:?}"),
    }

    // Unknown id_format → UnknownIdFormat → 400.
    let bad_id = Envelope {
        schema_version: CURRENT_SCHEMA_VERSION,
        id_format: "uuid-v4".to_string(),
        payload: serde_json::json!({ "method": "getDocument", "args": { "documentId": "d1" } }),
    };
    assert!(
        matches!(
            decode_crud_request(&bad_id),
            Err(DecodeError::UnknownIdFormat(_))
        ),
        "unknown id_format must be UnknownIdFormat (400)"
    );

    // Malformed JSON → InvalidJson → 400.
    let malformed = Envelope::with_payload(serde_json::json!({ "method": 42 }));
    assert!(
        matches!(
            decode_crud_request(&malformed),
            Err(DecodeError::InvalidEnvelope(_))
        ),
        "non-object payload must be InvalidEnvelope (400)"
    );
}

// ---------------------------------------------------------------------------
// §7 — endpoint-path constants (11 rows, bijective).
// ---------------------------------------------------------------------------

/// §7 — the 11 pinned endpoint paths, pairwise distinct, bijective with the 11
/// methods.
#[test]
fn endpoint_paths_are_bijective() {
    assert_eq!(ENGINE_ENDPOINTS.len(), 11, "exactly 11 rows");
    let mut paths: Vec<&str> = Vec::new();
    let mut methods: Vec<CrudMethod> = Vec::new();
    for (path, m) in ENGINE_ENDPOINTS {
        assert!(!path.is_empty(), "path non-empty");
        assert!(!paths.contains(path), "path {path} duplicated");
        assert!(!methods.contains(m), "method {m:?} duplicated");
        paths.push(path);
        methods.push(m.clone());
    }
    // Every one of the 11 methods appears exactly once.
    for m in all_methods() {
        assert!(
            methods.contains(&m),
            "method {m:?} missing from ENGINE_ENDPOINTS"
        );
    }
}

/// §7 — each `ENDPOINT_*` constant equals the pinned path for its method.
#[test]
fn endpoint_constants_match_pinned_paths() {
    assert_eq!(ENDPOINT_CREATE_DOCUMENT, "POST /documents");
    assert_eq!(ENDPOINT_GET_DOCUMENT, "GET /documents/:id");
    assert_eq!(ENDPOINT_UPDATE_DOCUMENT, "POST /documents/:id/update");
    assert_eq!(ENDPOINT_DELETE_DOCUMENT, "DELETE /documents/:id");
    assert_eq!(ENDPOINT_PUBLISH_DOCUMENT, "POST /documents/:id/publish");
    assert_eq!(ENDPOINT_UNPUBLISH_DOCUMENT, "POST /documents/:id/unpublish");
    assert_eq!(ENDPOINT_ARCHIVE_DOCUMENT, "POST /documents/:id/archive");
    assert_eq!(ENDPOINT_LIST_DOCUMENTS, "GET /documents");
    assert_eq!(ENDPOINT_CREATE_WIKI, "POST /wikis");
    assert_eq!(ENDPOINT_GET_WIKI, "GET /wikis/:id");
    assert_eq!(ENDPOINT_LIST_WIKIS, "GET /wikis");
}

// ---------------------------------------------------------------------------
// §8 — RBAC `caller` shape (present on the 7 mutating, absent on the 4 read-only).
// ---------------------------------------------------------------------------

/// §8 — the 7 mutating request args carry a `caller` field; the 4 read-only do not.
#[test]
fn rbac_caller_shape() {
    for m in mutating_methods() {
        let env = encode_crud_request(m.clone(), args_for(&m));
        let args = env
            .payload
            .get("args")
            .and_then(|v| v.as_object())
            .expect("args object");
        assert!(args.contains_key("caller"), "mutating {m:?} must carry caller");
        assert!(
            args.get("caller").and_then(|v| v.as_str()).is_some(),
            "caller must be a string for {m:?}"
        );
    }
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
// §9 — golden conformance vectors V-10..V-14 (byte-for-byte).
// ---------------------------------------------------------------------------

const V10: &str = r#"{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"method":"createDocument","args":{"caller":"user:alice","wikiId":"w1","body":{"title":"Getting Started","tags":["guide"],"author":"alice"}}}}"#;

const V11: &str = r#"{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"method":"createDocument","result":{"document_id":"d1","wiki_id":"w1","revision":0,"state":"Draft","graph":{"nodes":[],"edges":[]},"title":"Getting Started","created_at":"2026-09-09T00:00:00Z","updated_at":"2026-09-09T00:00:00Z","tags":["guide"],"author":"alice"}}}"#;

const V12: &str = r#"{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"method":"updateDocument","error":{"code":"conflict","message":"optimistic-concurrency conflict: stale base revision"}}}"#;

/// V-10 — `encode_crud_request(CreateDocument, …)` byte-for-byte.
#[test]
fn golden_v10_create_document_request() {
    let env =
        encode_crud_request(CrudMethod::CreateDocument, args_for(&CrudMethod::CreateDocument));
    assert_eq!(env.to_json().unwrap(), V10);
}

/// V-11 — `encode_crud_response(CreateDocument, CrudResult::Document(…))`
/// byte-for-byte.
#[test]
fn golden_v11_create_document_response() {
    let env = encode_crud_response(
        CrudMethod::CreateDocument,
        result_for(&CrudMethod::CreateDocument),
    );
    assert_eq!(env.to_json().unwrap(), V11);
}

/// V-12 — `encode_crud_error(UpdateDocument, &StoreError::ConflictError)`
/// byte-for-byte.
#[test]
fn golden_v12_update_document_error() {
    let env = encode_crud_error(CrudMethod::UpdateDocument, &StoreError::ConflictError);
    assert_eq!(env.to_json().unwrap(), V12);
}

/// V-13 — round-trip identity samples across the document method families.
#[test]
fn golden_v13_roundtrip_identity_families() {
    use CrudMethod::*;
    // Document family: revision 0, Draft, empty graph, author:null round-trips.
    let mut doc = draft_doc();
    doc.author = None;
    for m in [
        CreateDocument,
        GetDocument,
        UpdateDocument,
        PublishDocument,
        UnpublishDocument,
        ArchiveDocument,
    ] {
        let r = match m {
            PublishDocument => CrudResult::Document({
                let mut d = doc.clone();
                d.state = DocState::Published;
                d
            }),
            ArchiveDocument => CrudResult::Document({
                let mut d = doc.clone();
                d.state = DocState::Archived;
                d
            }),
            _ => CrudResult::Document(doc.clone()),
        };
        let decoded = decode_crud_response(&encode_crud_response(m, r.clone())).unwrap();
        assert_eq!(decoded, r, "document family {m:?}");
    }

    // List family: a DocumentList with items, total, page:1, page_size:20.
    let list = CrudResult::DocumentList(DocumentList {
        items: vec![DocumentSummary {
            document_id: did("d1"),
            wiki_id: wid("w1"),
            title: "Getting Started".to_string(),
            state: DocState::Draft,
            revision: 0,
            updated_at: "2026-09-09T00:00:00Z".to_string(),
        }],
        total: 1,
        page: 1,
        page_size: 20,
    });
    let decoded = decode_crud_response(&encode_crud_response(ListDocuments, list.clone())).unwrap();
    assert_eq!(decoded, list, "list family");

    // Wiki family: a Wiki and a WikiList round-trip.
    let wiki = CrudResult::Wiki(Wiki {
        wiki_id: wid("w1"),
        name: "My Wiki".to_string(),
    });
    for m in [CreateWiki, GetWiki] {
        let decoded = decode_crud_response(&encode_crud_response(m, wiki.clone())).unwrap();
        assert_eq!(decoded, wiki, "wiki family {m:?}");
    }
    let wikis = CrudResult::WikiList(vec![Wiki {
        wiki_id: wid("w1"),
        name: "My Wiki".to_string(),
    }]);
    let decoded = decode_crud_response(&encode_crud_response(ListWikis, wikis.clone())).unwrap();
    assert_eq!(decoded, wikis, "wikilist family");

    // Void family: deleteDocument encodes to "result":null and decodes back.
    let decoded =
        decode_crud_response(&encode_crud_response(DeleteDocument, CrudResult::DeleteDocument))
            .unwrap();
    assert_eq!(decoded, CrudResult::DeleteDocument, "void family");
}

/// V-14 — the request-decode outcome (NEW-2) mapping: transport-level, NOT a
/// `StoreError`, no §11 row.
#[test]
fn golden_v14_request_decode_outcome() {
    // "method":"bogus" → UnknownMethod("bogus") → 422.
    let bogus = Envelope::with_payload(serde_json::json!({ "method": "bogus", "args": {} }));
    match decode_crud_request(&bogus) {
        Err(DecodeError::UnknownMethod(m)) => assert_eq!(m, "bogus"),
        other => panic!("expected UnknownMethod, got {other:?}"),
    }
    // missing "args" → InvalidEnvelope → 400.
    let no_args = Envelope::with_payload(serde_json::json!({ "method": "getDocument" }));
    assert!(matches!(
        decode_crud_request(&no_args),
        Err(DecodeError::InvalidEnvelope(_))
    ));
    // schemaVersion:99 → UnsupportedSchemaVersion(99) → 400.
    let bad_schema = Envelope {
        schema_version: 99,
        id_format: ID_FORMAT_OPAQUE_STRING_V1.to_string(),
        payload: serde_json::json!({ "method": "getDocument", "args": { "documentId": "d1" } }),
    };
    match decode_crud_request(&bad_schema) {
        Err(DecodeError::UnsupportedSchemaVersion(v)) => assert_eq!(v, 99),
        other => panic!("expected UnsupportedSchemaVersion(99), got {other:?}"),
    }
    // None of these is a StoreError (they are DecodeError, not StoreError-mapped 502).
}

// ---------------------------------------------------------------------------
// §10 — valid/happy + fail states per function.
// ---------------------------------------------------------------------------

/// §10 — `encode_crud_request` round-trips all 11 methods.
#[test]
fn encode_decode_request_roundtrip_all_11() {
    for m in all_methods() {
        let args = args_for(&m);
        let decoded = decode_crud_request(&encode_crud_request(m.clone(), args.clone())).unwrap();
        assert_eq!(decoded, (m, args));
    }
}

/// §10 — `decode_crud_request` fail-states: unknown method, missing args,
/// unsupported schema version, unknown id format.
#[test]
fn decode_request_fail_states() {
    // Unknown method.
    let unknown = Envelope::with_payload(serde_json::json!({ "method": "nope", "args": {} }));
    assert!(matches!(
        decode_crud_request(&unknown),
        Err(DecodeError::UnknownMethod(_))
    ));
    // Missing args.
    let no_args = Envelope::with_payload(serde_json::json!({ "method": "getDocument" }));
    assert!(matches!(
        decode_crud_request(&no_args),
        Err(DecodeError::InvalidEnvelope(_))
    ));
    // Unsupported schema version.
    let bad_schema = Envelope {
        schema_version: 99,
        id_format: ID_FORMAT_OPAQUE_STRING_V1.to_string(),
        payload: serde_json::json!({ "method": "getDocument", "args": { "documentId": "d1" } }),
    };
    assert!(matches!(
        decode_crud_request(&bad_schema),
        Err(DecodeError::UnsupportedSchemaVersion(99))
    ));
    // Unknown id format.
    let bad_id = Envelope {
        schema_version: CURRENT_SCHEMA_VERSION,
        id_format: "uuid-v4".to_string(),
        payload: serde_json::json!({ "method": "getDocument", "args": { "documentId": "d1" } }),
    };
    assert!(matches!(
        decode_crud_request(&bad_id),
        Err(DecodeError::UnknownIdFormat(_))
    ));
}

/// §10 — `decode_crud_response` happy path: well-formed result body, method
/// matches, CRUD invariants hold → `Ok(result)`.
#[test]
fn decode_response_happy_path() {
    for m in all_methods() {
        let r = result_for(&m);
        let decoded = decode_crud_response(&encode_crud_response(m.clone(), r.clone())).unwrap();
        assert_eq!(decoded, r, "happy path {m:?}");
    }
}

/// §10 — `decode_crud_response` → `CrudResponseError::Store(e)` for an error
/// envelope.
#[test]
fn decode_response_store_error_route() {
    let env = encode_crud_error(CrudMethod::GetDocument, &StoreError::DocumentNotFound);
    match decode_crud_response(&env) {
        Err(CrudResponseError::Store(StoreError::DocumentNotFound)) => {}
        other => panic!("expected Store(DocumentNotFound), got {other:?}"),
    }
}

/// §10 — `decode_crud_response` → `CrudResponseError::Decode(_)` for a malformed
/// body (method/result mismatch, unknown schema version, unknown id format).
#[test]
fn decode_response_decode_error_route() {
    // Method/result mismatch: "getWiki" with a Document result is malformed.
    let mismatch = Envelope::with_payload(serde_json::json!({
        "method": "getWiki",
        "result": { "document_id": "d1", "wiki_id": "w1", "revision": 0, "state": "Draft",
                    "graph": { "nodes": [], "edges": [] }, "title": "t",
                    "created_at": "2026-09-09T00:00:00Z", "updated_at": "2026-09-09T00:00:00Z",
                    "tags": [], "author": null }
    }));
    assert!(
        matches!(
            decode_crud_response(&mismatch),
            Err(CrudResponseError::Decode(_))
        ),
        "method/result mismatch must be Decode"
    );

    // Unknown schema version on a response → Decode.
    let bad_schema = Envelope {
        schema_version: 99,
        id_format: ID_FORMAT_OPAQUE_STRING_V1.to_string(),
        payload: serde_json::json!({ "method": "getDocument", "result": {} }),
    };
    assert!(matches!(
        decode_crud_response(&bad_schema),
        Err(CrudResponseError::Decode(_))
    ));

    // Unknown id format on a response → Decode.
    let bad_id = Envelope {
        schema_version: CURRENT_SCHEMA_VERSION,
        id_format: "uuid-v4".to_string(),
        payload: serde_json::json!({ "method": "getDocument", "result": {} }),
    };
    assert!(matches!(
        decode_crud_response(&bad_id),
        Err(CrudResponseError::Decode(_))
    ));
}

/// §10 — `decode_crud_response` → `CrudResponseError::Validation(_)` for a
/// well-formed body failing a CRUD invariant.
#[test]
fn decode_response_validation_route() {
    // createDocument with revision != 0 violates the createDocument invariant.
    let mut bad = draft_doc();
    bad.revision = 1;
    let env = encode_crud_response(CrudMethod::CreateDocument, CrudResult::Document(bad));
    assert!(
        matches!(
            decode_crud_response(&env),
            Err(CrudResponseError::Validation(_))
        ),
        "createDocument revision!=0 must be Validation"
    );
}

/// §10 — `validate_crud_result` happy path: valid result → `Ok(())`.
#[test]
fn validate_crud_result_happy_path() {
    for m in all_methods() {
        let r = result_for(&m);
        assert!(validate_crud_result(&m, &r).is_ok(), "validate happy path {m:?}");
    }
}

/// §10 — `validate_crud_result` → `UnexpectedRevision` (createDocument must
/// yield revision == 0).
#[test]
fn validate_unexpected_revision() {
    let mut d = draft_doc();
    d.revision = 1;
    match validate_crud_result(&CrudMethod::CreateDocument, &CrudResult::Document(d)) {
        Err(CrudValidationFailure::UnexpectedRevision { expected, actual }) => {
            assert_eq!(expected, 0);
            assert_eq!(actual, 1);
        }
        other => panic!("expected UnexpectedRevision, got {other:?}"),
    }
}

/// §10 — `validate_crud_result` → `UnexpectedState` (publish/unpublish/archive
/// must yield the documented state).
#[test]
fn validate_unexpected_state() {
    // publishDocument must yield Published.
    let mut d = draft_doc();
    d.state = DocState::Draft;
    match validate_crud_result(&CrudMethod::PublishDocument, &CrudResult::Document(d)) {
        Err(CrudValidationFailure::UnexpectedState { expected, actual }) => {
            assert_eq!(expected, DocState::Published);
            assert_eq!(actual, DocState::Draft);
        }
        other => panic!("expected UnexpectedState, got {other:?}"),
    }
    // unpublishDocument must yield Draft.
    let mut d = draft_doc();
    d.state = DocState::Published;
    match validate_crud_result(&CrudMethod::UnpublishDocument, &CrudResult::Document(d)) {
        Err(CrudValidationFailure::UnexpectedState { expected, actual }) => {
            assert_eq!(expected, DocState::Draft);
            assert_eq!(actual, DocState::Published);
        }
        other => panic!("expected UnexpectedState, got {other:?}"),
    }
    // archiveDocument must yield Archived.
    let mut d = draft_doc();
    d.state = DocState::Draft;
    match validate_crud_result(&CrudMethod::ArchiveDocument, &CrudResult::Document(d)) {
        Err(CrudValidationFailure::UnexpectedState { expected, actual }) => {
            assert_eq!(expected, DocState::Archived);
            assert_eq!(actual, DocState::Draft);
        }
        other => panic!("expected UnexpectedState, got {other:?}"),
    }
}

/// §10 — `validate_crud_result` → `InvalidPagination` (listDocuments must yield
/// page >= 1, 1 <= page_size <= 100).
#[test]
fn validate_invalid_pagination() {
    // page == 0.
    let list = CrudResult::DocumentList(DocumentList {
        items: vec![],
        total: 0,
        page: 0,
        page_size: 20,
    });
    match validate_crud_result(&CrudMethod::ListDocuments, &list) {
        Err(CrudValidationFailure::InvalidPagination { page, page_size }) => {
            assert_eq!(page, 0);
            assert_eq!(page_size, 20);
        }
        other => panic!("expected InvalidPagination, got {other:?}"),
    }
    // page_size == 0.
    let list = CrudResult::DocumentList(DocumentList {
        items: vec![],
        total: 0,
        page: 1,
        page_size: 0,
    });
    assert!(matches!(
        validate_crud_result(&CrudMethod::ListDocuments, &list),
        Err(CrudValidationFailure::InvalidPagination { .. })
    ));
    // page_size > 100.
    let list = CrudResult::DocumentList(DocumentList {
        items: vec![],
        total: 0,
        page: 1,
        page_size: 101,
    });
    assert!(matches!(
        validate_crud_result(&CrudMethod::ListDocuments, &list),
        Err(CrudValidationFailure::InvalidPagination { .. })
    ));
}

/// §10 — `validate_crud_result` → `UnexpectedVoid` (deleteDocument must yield
/// void).
#[test]
fn validate_unexpected_void() {
    let r = CrudResult::Document(draft_doc());
    match validate_crud_result(&CrudMethod::DeleteDocument, &r) {
        Err(CrudValidationFailure::UnexpectedVoid) => {}
        other => panic!("expected UnexpectedVoid, got {other:?}"),
    }
}

/// §10 — `validate_crud_result` happy: getDocument/updateDocument/createWiki/
/// getWiki/listWikis accept any well-formed result of the right variant.
#[test]
fn validate_no_invariant_methods() {
    // getDocument accepts any well-formed Document (even revision != 0).
    let mut d = draft_doc();
    d.revision = 7;
    assert!(validate_crud_result(&CrudMethod::GetDocument, &CrudResult::Document(d)).is_ok());
    // updateDocument accepts any well-formed Document.
    assert!(validate_crud_result(
        &CrudMethod::UpdateDocument,
        &CrudResult::Document(draft_doc())
    )
    .is_ok());
    // createWiki / getWiki accept any well-formed Wiki.
    let w = CrudResult::Wiki(Wiki {
        wiki_id: wid("w1"),
        name: "My Wiki".to_string(),
    });
    assert!(validate_crud_result(&CrudMethod::CreateWiki, &w).is_ok());
    assert!(validate_crud_result(&CrudMethod::GetWiki, &w).is_ok());
    // listWikis accepts any well-formed Vec<Wiki>.
    assert!(validate_crud_result(
        &CrudMethod::ListWikis,
        &CrudResult::WikiList(vec![])
    )
    .is_ok());
}

/// §10 — `encode_crud_request` embeds the serde-frozen body verbatim under
/// `body` (snake_case keys preserved).
#[test]
fn request_body_embedded_verbatim() {
    let env =
        encode_crud_request(CrudMethod::CreateDocument, args_for(&CrudMethod::CreateDocument));
    let body = env
        .payload
        .get("args")
        .and_then(|v| v.get("body"))
        .expect("body present");
    assert_eq!(
        body,
        &serde_json::json!({ "title": "Getting Started", "tags": ["guide"], "author": "alice" })
    );
}

/// §10 — `encode_crud_response` embeds the serde-frozen result body verbatim
/// (snake_case keys, PascalCase `state`, `null` for absent `Option`).
#[test]
fn response_result_embedded_verbatim() {
    let mut d = draft_doc();
    d.author = None;
    let env = encode_crud_response(CrudMethod::GetDocument, CrudResult::Document(d));
    let result = env.payload.get("result").expect("result present");
    assert_eq!(
        result,
        &serde_json::json!({
            "document_id": "d1", "wiki_id": "w1", "revision": 0, "state": "Draft",
            "graph": { "nodes": [], "edges": [] }, "title": "Getting Started",
            "created_at": "2026-09-09T00:00:00Z", "updated_at": "2026-09-09T00:00:00Z",
            "tags": ["guide"], "author": null
        })
    );
}

/// §10 — `current_schema_version()` and the envelope constants are reused from
/// F2 unchanged.
#[test]
fn envelope_constants_reused() {
    assert_eq!(current_schema_version(), 1);
    assert_eq!(CURRENT_SCHEMA_VERSION, 1);
    assert_eq!(ID_FORMAT_OPAQUE_STRING_V1, "opaque-string-v1");
}

// ---------------------------------------------------------------------------
// §4.5 — the SSE schema is unchanged (retrieval-only) and the CRUD codecs
// never touch `sse.rs`.
// ---------------------------------------------------------------------------

/// §4.5 — the SSE event schema stays retrieval-only: exactly the 3 F2
/// `SseEventType` variants (no CRUD event types), and no CRUD request/response/
/// error envelope carries SSE framing keys (`event`/`data`/`type`).
#[test]
fn sse_schema_unchanged_and_crud_does_not_touch_sse() {
    use gnosis::wire::sse::SseEventType;
    // The SSE event schema is unchanged: exactly the 3 F2 event types, no CRUD
    // event types. (Exhaustive match — a new variant would fail to compile.)
    let labels: Vec<&str> = [SseEventType::Result, SseEventType::Done, SseEventType::Error]
        .iter()
        .map(|t| match t {
            SseEventType::Result => "result",
            SseEventType::Done => "done",
            SseEventType::Error => "error",
        })
        .collect();
    assert_eq!(labels, ["result", "done", "error"]);

    // The CRUD codecs never touch sse.rs: no CRUD request/response/error
    // envelope carries SSE framing keys at the top level of its payload.
    for m in all_methods() {
        let req = encode_crud_request(m.clone(), args_for(&m));
        let resp = encode_crud_response(m.clone(), result_for(&m));
        let err = encode_crud_error(m.clone(), &StoreError::DocumentNotFound);
        for env in [&req, &resp, &err] {
            let payload = env.payload.as_object().expect("payload object");
            for key in ["event", "data", "type"] {
                assert!(
                    !payload.contains_key(key),
                    "CRUD envelope for {m:?} must not carry SSE framing key {key}"
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// §10 — additional fail-states (adversarial-gate host-minor findings).
// ---------------------------------------------------------------------------

/// §10 — `validate_crud_result` rejects a wrong-variant result for the
/// invariant-carrying methods (matching `DeleteDocument`'s `UnexpectedVoid`).
#[test]
fn validate_wrong_variant_rejected() {
    // createDocument must yield a Document, not a Wiki.
    let w = CrudResult::Wiki(Wiki {
        wiki_id: wid("w1"),
        name: "My Wiki".to_string(),
    });
    assert!(matches!(
        validate_crud_result(&CrudMethod::CreateDocument, &w),
        Err(CrudValidationFailure::UnexpectedVoid)
    ));
    // publishDocument must yield a Document.
    assert!(matches!(
        validate_crud_result(&CrudMethod::PublishDocument, &w),
        Err(CrudValidationFailure::UnexpectedVoid)
    ));
    // unpublishDocument must yield a Document.
    assert!(matches!(
        validate_crud_result(&CrudMethod::UnpublishDocument, &w),
        Err(CrudValidationFailure::UnexpectedVoid)
    ));
    // archiveDocument must yield a Document.
    assert!(matches!(
        validate_crud_result(&CrudMethod::ArchiveDocument, &w),
        Err(CrudValidationFailure::UnexpectedVoid)
    ));
    // listDocuments must yield a DocumentList, not a Document.
    let d = CrudResult::Document(draft_doc());
    assert!(matches!(
        validate_crud_result(&CrudMethod::ListDocuments, &d),
        Err(CrudValidationFailure::UnexpectedVoid)
    ));
}

/// §10 — a mutating request with no `caller` field → `InvalidEnvelope` (400).
#[test]
fn mutating_request_without_caller_is_invalid() {
    // createDocument is mutating and requires `caller`.
    let no_caller = Envelope::with_payload(serde_json::json!({
        "method": "createDocument",
        "args": { "wikiId": "w1", "body": { "title": "t", "tags": [], "author": null } }
    }));
    assert!(
        matches!(
            decode_crud_request(&no_caller),
            Err(DecodeError::InvalidEnvelope(_))
        ),
        "mutating request without caller must be InvalidEnvelope (400)"
    );
}

/// §10 — a read-only request with a `caller` field is tolerated/ignored.
#[test]
fn read_only_request_with_caller_is_tolerated() {
    // getDocument is read-only; a `caller` field is ignored.
    let with_caller = Envelope::with_payload(serde_json::json!({
        "method": "getDocument",
        "args": { "documentId": "d1", "caller": "user:alice" }
    }));
    match decode_crud_request(&with_caller) {
        Ok((CrudMethod::GetDocument, CrudRequestArgs::GetDocument { document_id })) => {
            assert_eq!(document_id, did("d1"));
        }
        other => panic!("read-only request with caller must decode, got {other:?}"),
    }
}

/// §10 — `listWikis` with non-empty `args` → `InvalidEnvelope`.
#[test]
fn list_wikis_with_nonempty_args_is_invalid() {
    let nonempty = Envelope::with_payload(serde_json::json!({
        "method": "listWikis",
        "args": { "wikiId": "w1" }
    }));
    assert!(
        matches!(
            decode_crud_request(&nonempty),
            Err(DecodeError::InvalidEnvelope(_))
        ),
        "listWikis with non-empty args must be InvalidEnvelope"
    );
}

/// §10 — a response error envelope with an unknown `code` →
/// `CrudResponseError::Decode(UnknownCode)`.
#[test]
fn response_error_unknown_code_is_decode() {
    let unknown = Envelope::with_payload(serde_json::json!({
        "method": "getDocument",
        "error": { "code": "bogus_code", "message": "x" }
    }));
    match decode_crud_response(&unknown) {
        Err(CrudResponseError::Decode(DecodeError::UnknownCode(c))) => {
            assert_eq!(c, "bogus_code");
        }
        other => panic!("expected Decode(UnknownCode), got {other:?}"),
    }
}
