//! §7.2 P1a — Document-CRUD wire contract — BLIND-GREENS set
//! (`tests/blind_p1a_crud_wire_greens.rs`).
//!
//! Authored by the **blind_test_writer** from `docs/specs/p1a-document-crud-wire.md`
//! (§4 wire shapes, §6 error codec + NEW-2 request-decode outcome, §7 endpoint
//! paths, §8 RBAC caller, §9 golden vectors V-10..V-14, §10 valid/happy + fail
//! states) **ALONE**. The implementation (`src/wire/crud.rs` and all `src/`) was
//! NOT read. Each `#[test]` is one greens scenario (S1..S19) asserting the spec's
//! expected behavior in observable terms (exact wire JSON, exact error outcome,
//! exact status). A failing assertion is a finding (doc/spec drift or an
//! un-hardened regression).

use gnosis::envelope::{
    current_schema_version, Envelope, CURRENT_SCHEMA_VERSION, ID_FORMAT_OPAQUE_STRING_V1,
};
use gnosis::error::code_table;
use gnosis::wire::crud::{
    decode_crud_request, decode_crud_response, encode_crud_error, encode_crud_request,
    encode_crud_response, validate_crud_result, CrudMethod, CrudRequestArgs, CrudResponseError,
    CrudResult, CrudValidationFailure, ENDPOINT_ARCHIVE_DOCUMENT, ENDPOINT_CREATE_DOCUMENT,
    ENDPOINT_CREATE_WIKI, ENDPOINT_DELETE_DOCUMENT, ENDPOINT_GET_DOCUMENT, ENDPOINT_GET_WIKI,
    ENDPOINT_LIST_DOCUMENTS, ENDPOINT_LIST_WIKIS, ENDPOINT_PUBLISH_DOCUMENT,
    ENDPOINT_UNPUBLISH_DOCUMENT, ENDPOINT_UPDATE_DOCUMENT, ENGINE_ENDPOINTS,
};
use gnosis::wire::decode::DecodeError;
use gnosis::{
    CreateDocumentRequest, DocState, Document, DocumentId, DocumentList, DocumentSummary, Graph,
    ListDocumentsFilter, StoreError, UpdateDocumentRequest, Wiki, WikiId,
};

// ---------------------------------------------------------------------------
// Fixtures / helpers (built from the spec's §4.2/§4.3 shapes)
// ---------------------------------------------------------------------------

fn did(id: &str) -> DocumentId {
    DocumentId(id.to_string())
}
fn wid(id: &str) -> WikiId {
    WikiId(id.to_string())
}

fn empty_graph() -> Graph {
    Graph {
        nodes: vec![],
        edges: vec![],
    }
}

/// The §4.2 camelCase `"method"` value for each `CrudMethod` variant.
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

fn read_only_methods() -> Vec<CrudMethod> {
    use CrudMethod::*;
    vec![GetDocument, ListDocuments, GetWiki, ListWikis]
}

/// A representative well-formed `args` for each method (§4.2).
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

/// A representative `Document` (revision 0, Draft, empty graph) — §4.3.
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

/// A representative well-formed `CrudResult` for each method (§4.3).
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
// S1 — §4.1/§4.2 request envelope: versioned envelope + camelCase "method" + "args"
// ---------------------------------------------------------------------------

#[test]
fn s1_request_envelope_all_11_methods() {
    for m in all_methods() {
        let env = encode_crud_request(m, args_for(&m));
        assert_eq!(
            env.schema_version, CURRENT_SCHEMA_VERSION,
            "schema_version for {m:?}"
        );
        assert_eq!(
            env.id_format, ID_FORMAT_OPAQUE_STRING_V1,
            "id_format for {m:?}"
        );
        let payload = env.payload.as_object().expect("payload must be an object");
        assert_eq!(
            payload.get("method").and_then(|v| v.as_str()),
            Some(method_str(&m)),
            "method discriminator for {m:?}"
        );
        assert!(payload.contains_key("args"), "args present for {m:?}");
    }
}

// ---------------------------------------------------------------------------
// S2 — §4.2 per-method args wire JSON (camelCase top-level, body verbatim)
// ---------------------------------------------------------------------------

#[test]
fn s2_per_method_args_wire_json() {
    // createDocument
    let env = encode_crud_request(
        CrudMethod::CreateDocument,
        args_for(&CrudMethod::CreateDocument),
    );
    assert_eq!(
        env.payload.get("args"),
        Some(&serde_json::json!({
            "caller": "user:alice", "wikiId": "w1",
            "body": { "title": "Getting Started", "tags": ["guide"], "author": "alice" }
        })),
        "createDocument args"
    );
    // getDocument
    let env = encode_crud_request(CrudMethod::GetDocument, args_for(&CrudMethod::GetDocument));
    assert_eq!(
        env.payload.get("args"),
        Some(&serde_json::json!({ "documentId": "d1" })),
        "getDocument args"
    );
    // updateDocument
    let env = encode_crud_request(
        CrudMethod::UpdateDocument,
        args_for(&CrudMethod::UpdateDocument),
    );
    assert_eq!(
        env.payload.get("args"),
        Some(&serde_json::json!({
            "caller": "user:alice", "documentId": "d1",
            "body": { "base_revision": 0, "graph": { "nodes": [], "edges": [] },
                      "title": "Getting Started", "tags": ["guide"] }
        })),
        "updateDocument args"
    );
    // deleteDocument
    let env = encode_crud_request(
        CrudMethod::DeleteDocument,
        args_for(&CrudMethod::DeleteDocument),
    );
    assert_eq!(
        env.payload.get("args"),
        Some(&serde_json::json!({ "caller": "user:alice", "documentId": "d1" })),
        "deleteDocument args"
    );
    // publishDocument
    let env = encode_crud_request(
        CrudMethod::PublishDocument,
        args_for(&CrudMethod::PublishDocument),
    );
    assert_eq!(
        env.payload.get("args"),
        Some(&serde_json::json!({ "caller": "user:alice", "documentId": "d1" })),
        "publishDocument args"
    );
    // unpublishDocument
    let env = encode_crud_request(
        CrudMethod::UnpublishDocument,
        args_for(&CrudMethod::UnpublishDocument),
    );
    assert_eq!(
        env.payload.get("args"),
        Some(&serde_json::json!({ "caller": "user:alice", "documentId": "d1" })),
        "unpublishDocument args"
    );
    // archiveDocument
    let env = encode_crud_request(
        CrudMethod::ArchiveDocument,
        args_for(&CrudMethod::ArchiveDocument),
    );
    assert_eq!(
        env.payload.get("args"),
        Some(&serde_json::json!({ "caller": "user:alice", "documentId": "d1" })),
        "archiveDocument args"
    );
    // listDocuments
    let env = encode_crud_request(
        CrudMethod::ListDocuments,
        args_for(&CrudMethod::ListDocuments),
    );
    assert_eq!(
        env.payload.get("args"),
        Some(&serde_json::json!({
            "wikiId": "w1",
            "body": { "state": "Draft", "tag": null, "page": 1, "page_size": 20 }
        })),
        "listDocuments args"
    );
    // createWiki
    let env = encode_crud_request(CrudMethod::CreateWiki, args_for(&CrudMethod::CreateWiki));
    assert_eq!(
        env.payload.get("args"),
        Some(&serde_json::json!({ "caller": "user:alice", "name": "My Wiki" })),
        "createWiki args"
    );
    // getWiki
    let env = encode_crud_request(CrudMethod::GetWiki, args_for(&CrudMethod::GetWiki));
    assert_eq!(
        env.payload.get("args"),
        Some(&serde_json::json!({ "wikiId": "w1" })),
        "getWiki args"
    );
    // listWikis
    let env = encode_crud_request(CrudMethod::ListWikis, args_for(&CrudMethod::ListWikis));
    assert_eq!(
        env.payload.get("args"),
        Some(&serde_json::json!({})),
        "listWikis args"
    );
}

// ---------------------------------------------------------------------------
// S3 — §4.3 response envelope: camelCase "method" + "result"; deleteDocument void
// ---------------------------------------------------------------------------

#[test]
fn s3_response_envelope_all_11_methods() {
    for m in all_methods() {
        let env = encode_crud_response(m, result_for(&m));
        let payload = env.payload.as_object().expect("payload must be an object");
        assert_eq!(
            payload.get("method").and_then(|v| v.as_str()),
            Some(method_str(&m)),
            "method discriminator for {m:?}"
        );
        assert!(payload.contains_key("result"), "result present for {m:?}");
    }
    // deleteDocument void → "result":null
    let env = encode_crud_response(CrudMethod::DeleteDocument, CrudResult::DeleteDocument);
    assert_eq!(
        env.payload.get("result"),
        Some(&serde_json::Value::Null),
        "deleteDocument void result must be null"
    );
}

// ---------------------------------------------------------------------------
// S4 — §4.3 result serde bodies (snake_case, PascalCase enums, null for None)
// ---------------------------------------------------------------------------

#[test]
fn s4_result_serde_bodies() {
    // Document body
    let mut d = draft_doc();
    d.author = None;
    let env = encode_crud_response(CrudMethod::GetDocument, CrudResult::Document(d));
    assert_eq!(
        env.payload.get("result"),
        Some(&serde_json::json!({
            "document_id": "d1", "wiki_id": "w1", "revision": 0, "state": "Draft",
            "graph": { "nodes": [], "edges": [] }, "title": "Getting Started",
            "created_at": "2026-09-09T00:00:00Z", "updated_at": "2026-09-09T00:00:00Z",
            "tags": ["guide"], "author": null
        })),
        "Document result body"
    );
    // DocumentList body
    let env = encode_crud_response(
        CrudMethod::ListDocuments,
        result_for(&CrudMethod::ListDocuments),
    );
    let result = env.payload.get("result").expect("result present");
    assert_eq!(result.get("total"), Some(&serde_json::json!(1)));
    assert_eq!(result.get("page"), Some(&serde_json::json!(1)));
    assert_eq!(result.get("page_size"), Some(&serde_json::json!(20)));
    assert!(result.get("items").and_then(|v| v.as_array()).is_some());
    // Wiki body
    let env = encode_crud_response(CrudMethod::GetWiki, result_for(&CrudMethod::GetWiki));
    assert_eq!(
        env.payload.get("result"),
        Some(&serde_json::json!({ "wiki_id": "w1", "name": "My Wiki" })),
        "Wiki result body"
    );
    // WikiList body
    let env = encode_crud_response(CrudMethod::ListWikis, result_for(&CrudMethod::ListWikis));
    assert_eq!(
        env.payload.get("result"),
        Some(&serde_json::json!([{ "wiki_id": "w1", "name": "My Wiki" }])),
        "WikiList result body"
    );
}

// ---------------------------------------------------------------------------
// S5 — §4.4 error envelope: {"code","message"} under "error"; ConflictError → "conflict"
// ---------------------------------------------------------------------------

#[test]
fn s5_error_envelope_shape() {
    let env = encode_crud_error(CrudMethod::UpdateDocument, &StoreError::ConflictError);
    let payload = env.payload.as_object().unwrap();
    assert_eq!(
        payload.get("method").and_then(|v| v.as_str()),
        Some("updateDocument")
    );
    let err = payload.get("error").and_then(|v| v.as_object()).unwrap();
    assert_eq!(err.get("code").and_then(|v| v.as_str()), Some("conflict"));
    assert!(err.contains_key("message"));
    assert_eq!(err.len(), 2, "error object has exactly code + message");
}

// ---------------------------------------------------------------------------
// S6 — §6.1 exhaustiveness: 7 CRUD-reachable StoreError variants ⊆ 21 §11 rows
// ---------------------------------------------------------------------------

#[test]
fn s6_crud_reachable_store_errors_subset_of_21_rows() {
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
// S7 — §6.2 NEW-2 request-decode outcome (400/422, NOT 502, NOT a StoreError)
// ---------------------------------------------------------------------------

#[test]
fn s7_request_decode_outcome_table() {
    // Unknown method → UnknownMethod → 422.
    let unknown = Envelope::with_payload(serde_json::json!({ "method": "bogus", "args": {} }));
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
    // schemaVersion:99 → UnsupportedSchemaVersion → 400.
    let bad_schema = Envelope {
        schema_version: 99,
        id_format: ID_FORMAT_OPAQUE_STRING_V1.to_string(),
        payload: serde_json::json!({ "method": "getDocument", "args": { "documentId": "d1" } }),
    };
    match decode_crud_request(&bad_schema) {
        Err(DecodeError::UnsupportedSchemaVersion(v)) => assert_eq!(v, 99),
        other => panic!("expected UnsupportedSchemaVersion(99), got {other:?}"),
    }
    // id_format:"uuid-v4" → UnknownIdFormat → 400.
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
    // Non-object payload → InvalidEnvelope → 400.
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
// S8 — §7 endpoint-path ownership (H4): 11 rows, pairwise distinct, bijective
// ---------------------------------------------------------------------------

#[test]
fn s8_endpoint_paths_bijective() {
    assert_eq!(ENGINE_ENDPOINTS.len(), 11, "exactly 11 rows");
    let mut paths: Vec<&str> = Vec::new();
    let mut methods: Vec<CrudMethod> = Vec::new();
    for (path, m) in ENGINE_ENDPOINTS {
        assert!(!path.is_empty(), "path non-empty");
        assert!(!paths.contains(path), "path {path} duplicated");
        assert!(!methods.contains(m), "method {m:?} duplicated");
        paths.push(path);
        methods.push(*m);
    }
    for m in all_methods() {
        assert!(
            methods.contains(&m),
            "method {m:?} missing from ENGINE_ENDPOINTS"
        );
    }
}

#[test]
fn s8_endpoint_constants_match_pinned_paths() {
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
// S9 — §8 RBAC caller shape (present on 7 mutating, absent on 4 read-only)
// ---------------------------------------------------------------------------

#[test]
fn s9_rbac_caller_shape() {
    for m in mutating_methods() {
        let env = encode_crud_request(m, args_for(&m));
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
    for m in read_only_methods() {
        let env = encode_crud_request(m, args_for(&m));
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
// S10/S11/S12 — §9 golden vectors V-10/V-11/V-12 (byte-for-byte)
// ---------------------------------------------------------------------------

const V10: &str = r#"{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"method":"createDocument","args":{"caller":"user:alice","wikiId":"w1","body":{"title":"Getting Started","tags":["guide"],"author":"alice"}}}}"#;

const V11: &str = r#"{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"method":"createDocument","result":{"document_id":"d1","wiki_id":"w1","revision":0,"state":"Draft","graph":{"nodes":[],"edges":[]},"title":"Getting Started","created_at":"2026-09-09T00:00:00Z","updated_at":"2026-09-09T00:00:00Z","tags":["guide"],"author":"alice"}}}"#;

const V12: &str = r#"{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"method":"updateDocument","error":{"code":"conflict","message":"optimistic-concurrency conflict: stale base revision"}}}"#;

#[test]
fn s10_golden_v10_create_document_request() {
    let env = encode_crud_request(
        CrudMethod::CreateDocument,
        args_for(&CrudMethod::CreateDocument),
    );
    assert_eq!(env.to_json().unwrap(), V10);
}

#[test]
fn s11_golden_v11_create_document_response() {
    let env = encode_crud_response(
        CrudMethod::CreateDocument,
        result_for(&CrudMethod::CreateDocument),
    );
    assert_eq!(env.to_json().unwrap(), V11);
}

#[test]
fn s12_golden_v12_update_document_error() {
    let env = encode_crud_error(CrudMethod::UpdateDocument, &StoreError::ConflictError);
    assert_eq!(env.to_json().unwrap(), V12);
}

// ---------------------------------------------------------------------------
// S13 — §9 V-13 round-trip identity across the four method families
// ---------------------------------------------------------------------------

#[test]
fn s13_roundtrip_identity_families() {
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
    // List family.
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
    // Wiki family.
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
    // Void family.
    let decoded = decode_crud_response(&encode_crud_response(
        DeleteDocument,
        CrudResult::DeleteDocument,
    ))
    .unwrap();
    assert_eq!(decoded, CrudResult::DeleteDocument, "void family");
}

// ---------------------------------------------------------------------------
// S14 — §9 V-14 request-decode outcome (NEW-2) mapping
// ---------------------------------------------------------------------------

#[test]
fn s14_golden_v14_request_decode_outcome() {
    let bogus = Envelope::with_payload(serde_json::json!({ "method": "bogus", "args": {} }));
    match decode_crud_request(&bogus) {
        Err(DecodeError::UnknownMethod(m)) => assert_eq!(m, "bogus"),
        other => panic!("expected UnknownMethod, got {other:?}"),
    }
    let no_args = Envelope::with_payload(serde_json::json!({ "method": "getDocument" }));
    assert!(matches!(
        decode_crud_request(&no_args),
        Err(DecodeError::InvalidEnvelope(_))
    ));
    let bad_schema = Envelope {
        schema_version: 99,
        id_format: ID_FORMAT_OPAQUE_STRING_V1.to_string(),
        payload: serde_json::json!({ "method": "getDocument", "args": { "documentId": "d1" } }),
    };
    match decode_crud_request(&bad_schema) {
        Err(DecodeError::UnsupportedSchemaVersion(v)) => assert_eq!(v, 99),
        other => panic!("expected UnsupportedSchemaVersion(99), got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// S15 — §10 valid/happy: request + response round-trip for all 11 methods
// ---------------------------------------------------------------------------

#[test]
fn s15_request_response_roundtrip_all_11() {
    for m in all_methods() {
        let args = args_for(&m);
        let decoded = decode_crud_request(&encode_crud_request(m, args.clone())).unwrap();
        assert_eq!(decoded, (m, args), "request round-trip {m:?}");
        let r = result_for(&m);
        let decoded = decode_crud_response(&encode_crud_response(m, r.clone())).unwrap();
        assert_eq!(decoded, r, "response round-trip {m:?}");
    }
}

// ---------------------------------------------------------------------------
// S16 — §10 fail-states: decode_crud_response routes
// ---------------------------------------------------------------------------

#[test]
fn s16_decode_response_fail_routes() {
    // Error envelope → Store(e).
    let env = encode_crud_error(CrudMethod::GetDocument, &StoreError::DocumentNotFound);
    match decode_crud_response(&env) {
        Err(CrudResponseError::Store(StoreError::DocumentNotFound)) => {}
        other => panic!("expected Store(DocumentNotFound), got {other:?}"),
    }
    // Method/result mismatch → Decode.
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
    // Unknown error code → Decode(UnknownCode).
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
    // createDocument revision != 0 → Validation.
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

// ---------------------------------------------------------------------------
// S17 — §10 validate_crud_result invariants
// ---------------------------------------------------------------------------

#[test]
fn s17_validate_crud_result_invariants() {
    // Happy path all 11.
    for m in all_methods() {
        let r = result_for(&m);
        assert!(
            validate_crud_result(&m, &r).is_ok(),
            "validate happy path {m:?}"
        );
    }
    // UnexpectedRevision.
    let mut d = draft_doc();
    d.revision = 1;
    match validate_crud_result(&CrudMethod::CreateDocument, &CrudResult::Document(d)) {
        Err(CrudValidationFailure::UnexpectedRevision { expected, actual }) => {
            assert_eq!(expected, 0);
            assert_eq!(actual, 1);
        }
        other => panic!("expected UnexpectedRevision, got {other:?}"),
    }
    // UnexpectedState: publish/unpublish/archive.
    let mut d = draft_doc();
    d.state = DocState::Draft;
    match validate_crud_result(&CrudMethod::PublishDocument, &CrudResult::Document(d)) {
        Err(CrudValidationFailure::UnexpectedState { expected, actual }) => {
            assert_eq!(expected, DocState::Published);
            assert_eq!(actual, DocState::Draft);
        }
        other => panic!("expected UnexpectedState, got {other:?}"),
    }
    let mut d = draft_doc();
    d.state = DocState::Published;
    match validate_crud_result(&CrudMethod::UnpublishDocument, &CrudResult::Document(d)) {
        Err(CrudValidationFailure::UnexpectedState { expected, actual }) => {
            assert_eq!(expected, DocState::Draft);
            assert_eq!(actual, DocState::Published);
        }
        other => panic!("expected UnexpectedState, got {other:?}"),
    }
    let mut d = draft_doc();
    d.state = DocState::Draft;
    match validate_crud_result(&CrudMethod::ArchiveDocument, &CrudResult::Document(d)) {
        Err(CrudValidationFailure::UnexpectedState { expected, actual }) => {
            assert_eq!(expected, DocState::Archived);
            assert_eq!(actual, DocState::Draft);
        }
        other => panic!("expected UnexpectedState, got {other:?}"),
    }
    // InvalidPagination: page==0, page_size==0, page_size>100.
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
    // UnexpectedVoid: deleteDocument with a non-void result.
    let r = CrudResult::Document(draft_doc());
    match validate_crud_result(&CrudMethod::DeleteDocument, &r) {
        Err(CrudValidationFailure::UnexpectedVoid) => {}
        other => panic!("expected UnexpectedVoid, got {other:?}"),
    }
    // No-invariant methods accept any well-formed result of the right variant.
    let mut d = draft_doc();
    d.revision = 7;
    assert!(validate_crud_result(&CrudMethod::GetDocument, &CrudResult::Document(d)).is_ok());
    assert!(validate_crud_result(
        &CrudMethod::UpdateDocument,
        &CrudResult::Document(draft_doc())
    )
    .is_ok());
    let w = CrudResult::Wiki(Wiki {
        wiki_id: wid("w1"),
        name: "My Wiki".to_string(),
    });
    assert!(validate_crud_result(&CrudMethod::CreateWiki, &w).is_ok());
    assert!(validate_crud_result(&CrudMethod::GetWiki, &w).is_ok());
    assert!(validate_crud_result(&CrudMethod::ListWikis, &CrudResult::WikiList(vec![])).is_ok());
}

// ---------------------------------------------------------------------------
// S18 — §10 cross-cutting request-decode states
// ---------------------------------------------------------------------------

#[test]
fn s18_cross_cutting_request_decode_states() {
    // Mutating request with no caller → InvalidEnvelope (400).
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
    // Read-only request with a caller field is tolerated/ignored.
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
    // listWikis with non-empty args → InvalidEnvelope.
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

// ---------------------------------------------------------------------------
// S19 — §4.5 SSE is unchanged and the CRUD codecs never touch sse.rs
// ---------------------------------------------------------------------------

#[test]
fn s19_sse_unchanged_and_crud_does_not_touch_sse() {
    use gnosis::wire::sse::SseEventType;
    let labels: Vec<&str> = [
        SseEventType::Result,
        SseEventType::Done,
        SseEventType::Error,
    ]
    .iter()
    .map(|t| match t {
        SseEventType::Result => "result",
        SseEventType::Done => "done",
        SseEventType::Error => "error",
    })
    .collect();
    assert_eq!(labels, ["result", "done", "error"]);
    for m in all_methods() {
        let req = encode_crud_request(m, args_for(&m));
        let resp = encode_crud_response(m, result_for(&m));
        let err = encode_crud_error(m, &StoreError::DocumentNotFound);
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
// §4.1 envelope constants reused from F2 unchanged (supporting assertion)
// ---------------------------------------------------------------------------

#[test]
fn envelope_constants_reused() {
    assert_eq!(current_schema_version(), 1);
    assert_eq!(CURRENT_SCHEMA_VERSION, 1);
    assert_eq!(ID_FORMAT_OPAQUE_STRING_V1, "opaque-string-v1");
}
