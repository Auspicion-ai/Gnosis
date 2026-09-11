//! §7.2 P2 — `gnosis-server` binary crate — BLIND-GREENS SET
//! (`tests/blind_p2_gnosis_server_greens.rs`).
//!
//! Blind-test-writer derived from `docs/specs/p2-gnosis-server.md` ONLY (the P2
//! behavior contract). No implementation (`src/bin/gnosis_server.rs`,
//! `src/server.rs`, or any `src/` file) was read. Each `#[test]` asserts one
//! greens scenario (S1..S15) in observable terms: the exact endpoint, the exact
//! status, the exact response envelope.
//!
//! Pure surface: `gnosis::server_status`, `gnosis::request_decode_status`,
//! `gnosis::route_bijection`. Live surface: the `gnosis-server` bin (e2e legs).

use std::net::TcpListener;
use std::process::{Child, Command};
use std::time::Duration;

use gnosis::wire::crud::{
    decode_crud_request, decode_crud_response, encode_crud_request, CrudMethod, CrudRequestArgs,
    CrudResult, ENGINE_ENDPOINTS,
};
use gnosis::wire::decode::DecodeError;
use gnosis::wire::envelope::Envelope;
use gnosis::{
    request_decode_status, route_bijection, server_status, CreateDocumentRequest, DocState,
    DocumentId, ListDocumentsFilter, StoreError, UpdateDocumentRequest, WikiId,
};

// ---------------------------------------------------------------------------
// Fixtures / helpers (spec §5.2, §7.1, §7.2, §8, §10).
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

/// The 7 mutating methods (require `caller`, §8).
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

/// The 4 read-only methods (no `caller`, §8).
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
                graph: gnosis::Graph {
                    nodes: vec![],
                    edges: vec![],
                },
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

// ---------------------------------------------------------------------------
// S1 — §4 loopback bind: the server binds to 127.0.0.1 and serves.
// ---------------------------------------------------------------------------

const SERVER_BIN: &str = env!("CARGO_BIN_EXE_gnosis-server");

fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    listener.local_addr().expect("local addr").port()
}

async fn spawn_server() -> (Child, String) {
    let port = free_port();
    let child = Command::new(SERVER_BIN)
        .arg("--port")
        .arg(port.to_string())
        .spawn()
        .expect("spawn gnosis-server");
    let base = format!("http://127.0.0.1:{port}");
    let client = reqwest::Client::new();
    for _ in 0..100 {
        if let Ok(resp) = client
            .get(format!("{base}/engine/status"))
            .timeout(Duration::from_millis(200))
            .send()
            .await
        {
            if resp.status().is_success() {
                return (child, base);
            }
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("gnosis-server did not become ready on {base}");
}

struct ServerGuard(Child);
impl Drop for ServerGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// S1 — the server binds to 127.0.0.1 and serves GET /engine/status (200).
#[tokio::test]
async fn s1_loopback_bind_serves() {
    let (child, base) = spawn_server().await;
    let _guard = ServerGuard(child);
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{base}/engine/status"))
        .send()
        .await
        .expect("GET /engine/status");
    assert_eq!(resp.status().as_u16(), 200, "S1: loopback server must serve");
}

// ---------------------------------------------------------------------------
// S2 — §5.2 the routing table is a bijection of exactly 14 rows.
// ---------------------------------------------------------------------------

#[test]
fn s2_route_bijection_14_rows() {
    let table = route_bijection();
    assert_eq!(table.len(), 14, "S2: exactly 14 rows (11 CRUD + 3 retrieval)");
    let mut paths: Vec<&str> = Vec::new();
    let mut handlers: Vec<&str> = Vec::new();
    for (path, handler) in table {
        assert!(!path.is_empty(), "S2: path non-empty");
        assert!(!handler.is_empty(), "S2: handler non-empty");
        assert!(!paths.contains(path), "S2: path {path} duplicated");
        assert!(!handlers.contains(handler), "S2: handler {handler} duplicated");
        paths.push(path);
        handlers.push(handler);
    }
}

// ---------------------------------------------------------------------------
// S3 — §5.2 the 11 CRUD paths equal the P1a ENGINE_ENDPOINTS verbatim.
// ---------------------------------------------------------------------------

#[test]
fn s3_crud_paths_equal_engine_endpoints() {
    let table = route_bijection();
    let paths: Vec<&str> = table.iter().map(|(p, _)| *p).collect();
    let engine_paths: Vec<&str> = ENGINE_ENDPOINTS.iter().map(|(p, _)| *p).collect();
    let crud_paths: Vec<&str> = paths
        .iter()
        .copied()
        .filter(|p| engine_paths.contains(p))
        .collect();
    assert_eq!(crud_paths.len(), 11, "S3: exactly 11 CRUD paths");
    for p in &engine_paths {
        assert!(crud_paths.contains(p), "S3: CRUD path {p} must be verbatim");
    }
    for p in ["POST /rag/query", "GET /rag/stream", "GET /engine/status"] {
        assert!(paths.contains(&p), "S3: retrieval path {p} must be present");
    }
}

// ---------------------------------------------------------------------------
// S4 — §6 READY lifecycle: GET /engine/status returns a HealthReport (200).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn s4_engine_status_health_report() {
    let (child, base) = spawn_server().await;
    let _guard = ServerGuard(child);
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{base}/engine/status"))
        .send()
        .await
        .expect("GET /engine/status");
    assert_eq!(resp.status().as_u16(), 200, "S4: engine/status always 200");
    let text = resp.text().await.expect("status body");
    let report: serde_json::Value = serde_json::from_str(&text).expect("status is JSON");
    assert!(report.get("state").is_some(), "S4: HealthReport carries state");
    assert!(
        report.get("subsystems").is_some(),
        "S4: HealthReport carries subsystems"
    );
}

// ---------------------------------------------------------------------------
// S5 — §7.1 the 7 CRUD-reachable StoreError variants → exact status + code.
// ---------------------------------------------------------------------------

#[test]
fn s5_crud_reachable_status_mapping() {
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
            .unwrap_or_else(|| panic!("S5: server_status({e:?}) must be Some"));
        assert_eq!(got.0, status, "S5: status for {e:?}");
        assert_eq!(got.1, code, "S5: wire code for {e:?}");
    }
}

// ---------------------------------------------------------------------------
// S6 — §7.1 the retrieval-trio reachable variants → exact §11 status.
// ---------------------------------------------------------------------------

#[test]
fn s6_retrieval_trio_status_mapping() {
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
            .unwrap_or_else(|| panic!("S6: server_status({e:?}) must be Some"));
        assert_eq!(got.0, status, "S6: status for {e:?}");
    }
}

// ---------------------------------------------------------------------------
// S7 — §7.2 NEW-2: the 5 DecodeError variants → 400/422, never 502.
// ---------------------------------------------------------------------------

#[test]
fn s7_request_decode_status_mapping() {
    let expected: Vec<(DecodeError, u16)> = vec![
        (DecodeError::InvalidJson("x".to_string()), 400),
        (DecodeError::InvalidEnvelope("x".to_string()), 400),
        (DecodeError::UnknownMethod("bogus".to_string()), 422),
        (DecodeError::UnsupportedSchemaVersion(99), 400),
        (DecodeError::UnknownIdFormat("uuid-v4".to_string()), 400),
    ];
    for (e, status) in expected {
        let got = request_decode_status(&e)
            .unwrap_or_else(|| panic!("S7: request_decode_status({e:?}) must be Some"));
        assert_eq!(got, status, "S7: status for {e:?}");
        assert_ne!(got, 502, "S7: request-decode status never 502");
    }
}

// ---------------------------------------------------------------------------
// S8 — §8 RBAC caller threading: mutating carry caller, read-only do not.
// ---------------------------------------------------------------------------

#[test]
fn s8_caller_threading() {
    // Mutating: the encoded envelope carries a string caller on args.
    for m in mutating_methods() {
        let env = encode_crud_request(m.clone(), args_for(&m));
        let args = env
            .payload
            .get("args")
            .and_then(|v| v.as_object())
            .expect("args object");
        assert!(
            args.contains_key("caller"),
            "S8: mutating {m:?} must carry caller"
        );
        assert!(
            args.get("caller").and_then(|v| v.as_str()).is_some(),
            "S8: caller must be a string for {m:?}"
        );
    }
    // Read-only: the encoded envelope carries no caller on args.
    for m in read_only_methods() {
        let env = encode_crud_request(m.clone(), args_for(&m));
        let args = env
            .payload
            .get("args")
            .and_then(|v| v.as_object())
            .expect("args object");
        assert!(
            !args.contains_key("caller"),
            "S8: read-only {m:?} must NOT carry caller"
        );
    }
    // Decode layer: a mutating request with the caller removed is rejected (400).
    for m in mutating_methods() {
        let mut env = encode_crud_request(m.clone(), args_for(&m));
        if let Some(args_obj) = env.payload.get_mut("args").and_then(|v| v.as_object_mut()) {
            args_obj.remove("caller");
        }
        assert!(
            decode_crud_request(&env).is_err(),
            "S8: mutating {m:?} with no caller must be a request-decode error (400)"
        );
    }
    // Decode layer: a read-only request with an added caller is tolerated.
    for m in [CrudMethod::GetDocument, CrudMethod::ListDocuments, CrudMethod::GetWiki] {
        let mut env = encode_crud_request(m.clone(), args_for(&m));
        if let Some(args_obj) = env.payload.get_mut("args").and_then(|v| v.as_object_mut()) {
            args_obj.insert("caller".to_string(), serde_json::json!("user:alice"));
        }
        assert!(
            decode_crud_request(&env).is_ok(),
            "S8: read-only {m:?} with a caller must be tolerated"
        );
    }
}

// ---------------------------------------------------------------------------
// S9 — §9 e2e: rag_query while not READY → EngineUnavailable → 503.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn s9_rag_query_not_ready_503() {
    let (child, base) = spawn_server().await;
    let _guard = ServerGuard(child);
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "schemaVersion": 1,
        "idFormat": "opaque-string-v1",
        "payload": { "query": "hello", "args": {} }
    });
    let resp = client
        .post(format!("{base}/rag/query"))
        .json(&body)
        .send()
        .await
        .expect("POST /rag/query");
    assert_eq!(
        resp.status().as_u16(),
        503,
        "S9: rag_query on a not-READY engine must be EngineUnavailable (503)"
    );
}

// ---------------------------------------------------------------------------
// S10 — §9 e2e: createDocument → Document response envelope (200).
// ---------------------------------------------------------------------------

async fn create_wiki(client: &reqwest::Client, base: &str) -> String {
    let env = encode_crud_request(
        CrudMethod::CreateWiki,
        CrudRequestArgs::CreateWiki {
            caller: "user:alice".to_string(),
            name: "My Wiki".to_string(),
        },
    );
    let resp = client
        .post(format!("{base}/wikis"))
        .json(&env)
        .send()
        .await
        .expect("POST /wikis");
    assert_eq!(resp.status().as_u16(), 200, "createWiki must be 200");
    let text = resp.text().await.expect("createWiki body");
    let resp_env = Envelope::from_json(&text).expect("createWiki response envelope");
    match decode_crud_response(&resp_env).expect("createWiki decodes") {
        CrudResult::Wiki(w) => w.wiki_id.0,
        other => panic!("expected Wiki, got {other:?}"),
    }
}

#[tokio::test]
async fn s10_create_document_roundtrip() {
    let (child, base) = spawn_server().await;
    let _guard = ServerGuard(child);
    let client = reqwest::Client::new();
    let wiki_id = create_wiki(&client, &base).await;
    let env = encode_crud_request(
        CrudMethod::CreateDocument,
        CrudRequestArgs::CreateDocument {
            caller: "user:alice".to_string(),
            wiki_id: wid(&wiki_id),
            body: CreateDocumentRequest {
                title: "Getting Started".to_string(),
                tags: Some(vec!["guide".to_string()]),
                author: Some("alice".to_string()),
            },
        },
    );
    let resp = client
        .post(format!("{base}/documents"))
        .json(&env)
        .send()
        .await
        .expect("POST /documents");
    assert_eq!(resp.status().as_u16(), 200, "S10: createDocument must be 200");
    let text = resp.text().await.expect("create body");
    let resp_env = Envelope::from_json(&text).expect("create response envelope");
    let decoded = decode_crud_response(&resp_env).expect("create decodes");
    assert!(
        matches!(decoded, CrudResult::Document(_)),
        "S10: createDocument must return a Document response envelope"
    );
}

// ---------------------------------------------------------------------------
// S11 — §9 e2e: a ConflictError → HTTP 409.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn s11_conflict_error_409() {
    let (child, base) = spawn_server().await;
    let _guard = ServerGuard(child);
    let client = reqwest::Client::new();
    let wiki_id = create_wiki(&client, &base).await;
    let create = encode_crud_request(
        CrudMethod::CreateDocument,
        CrudRequestArgs::CreateDocument {
            caller: "user:alice".to_string(),
            wiki_id: wid(&wiki_id),
            body: CreateDocumentRequest {
                title: "Getting Started".to_string(),
                tags: Some(vec!["guide".to_string()]),
                author: Some("alice".to_string()),
            },
        },
    );
    let resp = client
        .post(format!("{base}/documents"))
        .json(&create)
        .send()
        .await
        .expect("POST /documents");
    let text = resp.text().await.expect("create body");
    let resp_env = Envelope::from_json(&text).expect("create response envelope");
    let doc = match decode_crud_response(&resp_env).expect("create decodes") {
        CrudResult::Document(d) => d,
        other => panic!("expected Document, got {other:?}"),
    };
    let doc_id = doc.document_id.0.clone();
    let node_id = gnosis::NodeId("n1".to_string());
    let doc = did(&doc_id);
    let graph = gnosis::Graph {
        nodes: vec![gnosis::Node {
            document_id: doc.clone(),
            node_id: node_id.clone(),
            kind: gnosis::NodeKind::Content,
            value: Some("Updated body".to_string()),
            fact_key: None,
            target: None,
        }],
        edges: vec![
            gnosis::Edge {
                source: (doc.clone(), gnosis::NodeId("ROOT".to_string())),
                target: (doc.clone(), node_id.clone()),
                kind: gnosis::EdgeKind::DocHead,
                state: None,
                cross_wiki: false,
                relation_type: None,
            },
            gnosis::Edge {
                source: (doc.clone(), node_id),
                target: (doc.clone(), gnosis::NodeId("END".to_string())),
                kind: gnosis::EdgeKind::DocEnd,
                state: None,
                cross_wiki: false,
                relation_type: None,
            },
        ],
    };
    let update = encode_crud_request(
        CrudMethod::UpdateDocument,
        CrudRequestArgs::UpdateDocument {
            caller: "user:alice".to_string(),
            document_id: did(&doc_id),
            body: UpdateDocumentRequest {
                base_revision: 0,
                graph,
                title: Some("Updated".to_string()),
                tags: None,
            },
        },
    );
    let first = client
        .post(format!("{base}/documents/{doc_id}/update"))
        .json(&update)
        .send()
        .await
        .expect("first update");
    assert_eq!(first.status().as_u16(), 200, "first update must be 200");
    let second = client
        .post(format!("{base}/documents/{doc_id}/update"))
        .json(&update)
        .send()
        .await
        .expect("second update");
    assert_eq!(
        second.status().as_u16(),
        409,
        "S11: stale base revision must be ConflictError (409)"
    );
}

// ---------------------------------------------------------------------------
// S12 — §9 e2e: a malformed request → HTTP 400.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn s12_malformed_request_400() {
    let (child, base) = spawn_server().await;
    let _guard = ServerGuard(child);
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{base}/documents"))
        .body("{ not json")
        .header("content-type", "application/json")
        .send()
        .await
        .expect("POST /documents malformed");
    assert_eq!(
        resp.status().as_u16(),
        400,
        "S12: malformed request body must be 400"
    );
}

// ---------------------------------------------------------------------------
// S13 — §9 e2e: an unknown method → HTTP 422.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn s13_unknown_method_422() {
    let (child, base) = spawn_server().await;
    let _guard = ServerGuard(child);
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "schemaVersion": 1,
        "idFormat": "opaque-string-v1",
        "payload": { "method": "bogus", "args": {} }
    });
    let resp = client
        .post(format!("{base}/documents"))
        .json(&body)
        .send()
        .await
        .expect("POST /documents unknown method");
    assert_eq!(
        resp.status().as_u16(),
        422,
        "S13: unknown method must be 422 (unprocessable)"
    );
}

// ---------------------------------------------------------------------------
// S14 — §9 e2e: the remaining CRUD round-trips.
// ---------------------------------------------------------------------------

async fn create_doc(client: &reqwest::Client, base: &str) -> String {
    let wiki_id = create_wiki(client, base).await;
    let env = encode_crud_request(
        CrudMethod::CreateDocument,
        CrudRequestArgs::CreateDocument {
            caller: "user:alice".to_string(),
            wiki_id: wid(&wiki_id),
            body: CreateDocumentRequest {
                title: "Getting Started".to_string(),
                tags: Some(vec!["guide".to_string()]),
                author: Some("alice".to_string()),
            },
        },
    );
    let resp = client
        .post(format!("{base}/documents"))
        .json(&env)
        .send()
        .await
        .expect("POST /documents");
    assert_eq!(resp.status().as_u16(), 200, "createDocument must be 200");
    let text = resp.text().await.expect("create body");
    let resp_env = Envelope::from_json(&text).expect("create response envelope");
    match decode_crud_response(&resp_env).expect("create decodes") {
        CrudResult::Document(d) => d.document_id.0,
        other => panic!("expected Document, got {other:?}"),
    }
}

#[tokio::test]
async fn s14_remaining_crud_roundtrips() {
    let (child, base) = spawn_server().await;
    let _guard = ServerGuard(child);
    let client = reqwest::Client::new();

    // getDocument
    let doc_id = create_doc(&client, &base).await;
    let env = encode_crud_request(
        CrudMethod::GetDocument,
        CrudRequestArgs::GetDocument {
            document_id: did(&doc_id),
        },
    );
    let resp = client
        .get(format!("{base}/documents/{doc_id}"))
        .json(&env)
        .send()
        .await
        .expect("GET /documents/:id");
    assert_eq!(resp.status().as_u16(), 200, "S14: getDocument must be 200");
    let text = resp.text().await.expect("get body");
    let resp_env = Envelope::from_json(&text).expect("get response envelope");
    assert!(
        matches!(decode_crud_response(&resp_env).expect("get decodes"), CrudResult::Document(_)),
        "S14: getDocument must return a Document response envelope"
    );

    // deleteDocument
    let doc_id = create_doc(&client, &base).await;
    let env = encode_crud_request(
        CrudMethod::DeleteDocument,
        CrudRequestArgs::DeleteDocument {
            caller: "user:alice".to_string(),
            document_id: did(&doc_id),
        },
    );
    let resp = client
        .delete(format!("{base}/documents/{doc_id}"))
        .json(&env)
        .send()
        .await
        .expect("DELETE /documents/:id");
    assert_eq!(resp.status().as_u16(), 200, "S14: deleteDocument must be 200");
    let text = resp.text().await.expect("delete body");
    let resp_env = Envelope::from_json(&text).expect("delete response envelope");
    assert!(
        matches!(decode_crud_response(&resp_env).expect("delete decodes"), CrudResult::DeleteDocument),
        "S14: deleteDocument must return a void response envelope"
    );

    // publishDocument
    let doc_id = create_doc(&client, &base).await;
    let env = encode_crud_request(
        CrudMethod::PublishDocument,
        CrudRequestArgs::PublishDocument {
            caller: "user:alice".to_string(),
            document_id: did(&doc_id),
        },
    );
    let resp = client
        .post(format!("{base}/documents/{doc_id}/publish"))
        .json(&env)
        .send()
        .await
        .expect("POST /documents/:id/publish");
    assert_eq!(resp.status().as_u16(), 200, "S14: publishDocument must be 200");
    let text = resp.text().await.expect("publish body");
    let resp_env = Envelope::from_json(&text).expect("publish response envelope");
    match decode_crud_response(&resp_env).expect("publish decodes") {
        CrudResult::Document(d) => assert_eq!(d.state, DocState::Published),
        other => panic!("expected Document, got {other:?}"),
    }

    // unpublishDocument (publish first)
    let doc_id = create_doc(&client, &base).await;
    let pub_env = encode_crud_request(
        CrudMethod::PublishDocument,
        CrudRequestArgs::PublishDocument {
            caller: "user:alice".to_string(),
            document_id: did(&doc_id),
        },
    );
    let pub_resp = client
        .post(format!("{base}/documents/{doc_id}/publish"))
        .json(&pub_env)
        .send()
        .await
        .expect("publish");
    assert_eq!(pub_resp.status().as_u16(), 200, "publish must be 200");
    let env = encode_crud_request(
        CrudMethod::UnpublishDocument,
        CrudRequestArgs::UnpublishDocument {
            caller: "user:alice".to_string(),
            document_id: did(&doc_id),
        },
    );
    let resp = client
        .post(format!("{base}/documents/{doc_id}/unpublish"))
        .json(&env)
        .send()
        .await
        .expect("POST /documents/:id/unpublish");
    assert_eq!(resp.status().as_u16(), 200, "S14: unpublishDocument must be 200");
    let text = resp.text().await.expect("unpublish body");
    let resp_env = Envelope::from_json(&text).expect("unpublish response envelope");
    match decode_crud_response(&resp_env).expect("unpublish decodes") {
        CrudResult::Document(d) => assert_eq!(d.state, DocState::Draft),
        other => panic!("expected Document, got {other:?}"),
    }

    // archiveDocument
    let doc_id = create_doc(&client, &base).await;
    let env = encode_crud_request(
        CrudMethod::ArchiveDocument,
        CrudRequestArgs::ArchiveDocument {
            caller: "user:alice".to_string(),
            document_id: did(&doc_id),
        },
    );
    let resp = client
        .post(format!("{base}/documents/{doc_id}/archive"))
        .json(&env)
        .send()
        .await
        .expect("POST /documents/:id/archive");
    assert_eq!(resp.status().as_u16(), 200, "S14: archiveDocument must be 200");
    let text = resp.text().await.expect("archive body");
    let resp_env = Envelope::from_json(&text).expect("archive response envelope");
    match decode_crud_response(&resp_env).expect("archive decodes") {
        CrudResult::Document(d) => assert_eq!(d.state, DocState::Archived),
        other => panic!("expected Document, got {other:?}"),
    }

    // listDocuments
    let wiki_id = create_wiki(&client, &base).await;
    let env = encode_crud_request(
        CrudMethod::ListDocuments,
        CrudRequestArgs::ListDocuments {
            wiki_id: wid(&wiki_id),
            body: ListDocumentsFilter {
                state: None,
                tag: None,
                page: Some(1),
                page_size: Some(20),
            },
        },
    );
    let resp = client
        .get(format!("{base}/documents"))
        .json(&env)
        .send()
        .await
        .expect("GET /documents");
    assert_eq!(resp.status().as_u16(), 200, "S14: listDocuments must be 200");
    let text = resp.text().await.expect("list body");
    let resp_env = Envelope::from_json(&text).expect("list response envelope");
    assert!(
        matches!(decode_crud_response(&resp_env).expect("list decodes"), CrudResult::DocumentList(_)),
        "S14: listDocuments must return a DocumentList response envelope"
    );

    // createWiki
    let env = encode_crud_request(
        CrudMethod::CreateWiki,
        CrudRequestArgs::CreateWiki {
            caller: "user:alice".to_string(),
            name: "My Wiki".to_string(),
        },
    );
    let resp = client
        .post(format!("{base}/wikis"))
        .json(&env)
        .send()
        .await
        .expect("POST /wikis");
    assert_eq!(resp.status().as_u16(), 200, "S14: createWiki must be 200");
    let text = resp.text().await.expect("createWiki body");
    let resp_env = Envelope::from_json(&text).expect("createWiki response envelope");
    assert!(
        matches!(decode_crud_response(&resp_env).expect("createWiki decodes"), CrudResult::Wiki(_)),
        "S14: createWiki must return a Wiki response envelope"
    );

    // getWiki
    let wiki_id = create_wiki(&client, &base).await;
    let env = encode_crud_request(
        CrudMethod::GetWiki,
        CrudRequestArgs::GetWiki {
            wiki_id: wid(&wiki_id),
        },
    );
    let resp = client
        .get(format!("{base}/wikis/{wiki_id}"))
        .json(&env)
        .send()
        .await
        .expect("GET /wikis/:id");
    assert_eq!(resp.status().as_u16(), 200, "S14: getWiki must be 200");
    let text = resp.text().await.expect("getWiki body");
    let resp_env = Envelope::from_json(&text).expect("getWiki response envelope");
    assert!(
        matches!(decode_crud_response(&resp_env).expect("getWiki decodes"), CrudResult::Wiki(_)),
        "S14: getWiki must return a Wiki response envelope"
    );

    // listWikis
    let env = encode_crud_request(CrudMethod::ListWikis, CrudRequestArgs::ListWikis);
    let resp = client
        .get(format!("{base}/wikis"))
        .json(&env)
        .send()
        .await
        .expect("GET /wikis");
    assert_eq!(resp.status().as_u16(), 200, "S14: listWikis must be 200");
    let text = resp.text().await.expect("listWikis body");
    let resp_env = Envelope::from_json(&text).expect("listWikis response envelope");
    assert!(
        matches!(decode_crud_response(&resp_env).expect("listWikis decodes"), CrudResult::WikiList(_)),
        "S14: listWikis must return a Vec<Wiki> response envelope"
    );
}

// ---------------------------------------------------------------------------
// S15 — §10 per-endpoint fail-state status mapping.
// ---------------------------------------------------------------------------

#[test]
fn s15_per_endpoint_fail_state_status() {
    // POST /documents: WikiNotFound→404, ValidationError→400.
    assert_eq!(server_status(&StoreError::WikiNotFound).unwrap().0, 404);
    assert_eq!(server_status(&StoreError::ValidationError("x".to_string())).unwrap().0, 400);
    // GET /documents/:id: DocumentNotFound→404.
    assert_eq!(server_status(&StoreError::DocumentNotFound).unwrap().0, 404);
    // POST /documents/:id/update: 404/400/409.
    assert_eq!(server_status(&StoreError::DocumentNotFound).unwrap().0, 404);
    assert_eq!(server_status(&StoreError::ValidationError("x".to_string())).unwrap().0, 400);
    assert_eq!(server_status(&StoreError::ConflictError).unwrap().0, 409);
    // DELETE /documents/:id: 404/409.
    assert_eq!(server_status(&StoreError::DocumentNotFound).unwrap().0, 404);
    assert_eq!(server_status(&StoreError::DocumentInUse).unwrap().0, 409);
    // POST /documents/:id/publish: 404/422.
    assert_eq!(server_status(&StoreError::DocumentNotFound).unwrap().0, 404);
    assert_eq!(server_status(&StoreError::UnresolvedReference).unwrap().0, 422);
    // POST /documents/:id/unpublish: 404/409.
    assert_eq!(server_status(&StoreError::DocumentNotFound).unwrap().0, 404);
    assert_eq!(server_status(&StoreError::InvalidState).unwrap().0, 409);
    // POST /documents/:id/archive: 404/409.
    assert_eq!(server_status(&StoreError::DocumentNotFound).unwrap().0, 404);
    assert_eq!(server_status(&StoreError::InvalidState).unwrap().0, 409);
    // GET /documents: 404/400.
    assert_eq!(server_status(&StoreError::WikiNotFound).unwrap().0, 404);
    assert_eq!(server_status(&StoreError::ValidationError("x".to_string())).unwrap().0, 400);
    // POST /wikis: 400.
    assert_eq!(server_status(&StoreError::ValidationError("x".to_string())).unwrap().0, 400);
    // GET /wikis/:id: 404.
    assert_eq!(server_status(&StoreError::WikiNotFound).unwrap().0, 404);
    // POST /rag/query: 503/502/502.
    assert_eq!(server_status(&StoreError::EngineUnavailable).unwrap().0, 503);
    assert_eq!(server_status(&StoreError::EngineError).unwrap().0, 502);
    assert_eq!(server_status(&StoreError::TraceUnavailable).unwrap().0, 502);
}
