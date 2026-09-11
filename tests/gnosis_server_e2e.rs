//! §7.2 P2 — `gnosis-server` binary crate — end-to-end transport test
//! (`tests/gnosis_server_e2e.rs`).
//!
//! TestWriter-derived from `docs/specs/p2-gnosis-server.md` §9. Runs against the
//! **real server** (a tokio runtime + the bound `127.0.0.1:<port>`), exercising
//! the `EngineUnavailable`/`EngineError` split over a live transport plus the
//! document-CRUD round-trips:
//!   - a `createDocument` request envelope → a `Document` response envelope;
//!   - a `ConflictError` → HTTP 409;
//!   - a malformed request → HTTP 400;
//!   - an unknown method → HTTP 422;
//!   - a `rag_query` while the engine is not READY → `EngineUnavailable` → 503;
//!   - round-trips for the remaining CRUD methods (`getDocument`/`deleteDocument`/
//!     `publish`/`unpublish`/`archive`/`listDocuments`/`createWiki`/`getWiki`/
//!     `listWikis`).
//!
//! The server does NOT auto-create a wiki on `createDocument` (the `WikiNotFound`
//! fail-state is reachable), so each document round-trip pre-creates its wiki via
//! `POST /wikis` first.

use std::net::TcpListener;
use std::process::{Child, Command};
use std::time::Duration;

use gnosis::wire::crud::{
    decode_crud_response, encode_crud_request, CrudMethod, CrudRequestArgs, CrudResult,
};
use gnosis::wire::envelope::Envelope;
use gnosis::{
    request_decode_status, server_status, CreateDocumentRequest, DocState, DocumentId,
    ListDocumentsFilter, StoreError, UpdateDocumentRequest, WikiId,
};

// ---------------------------------------------------------------------------
// Server harness: spawn the real `gnosis-server` binary on a free loopback port.
// ---------------------------------------------------------------------------

/// The compiled `gnosis-server` binary (Cargo sets this env var for integration
/// tests once the `[[bin]]` is declared). Compile-fail until the bin exists.
const SERVER_BIN: &str = env!("CARGO_BIN_EXE_gnosis-server");

/// Reserve a free loopback port, then release it so the server can bind it.
fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    listener.local_addr().expect("local addr").port()
}

/// Spawn the server on a free port and wait until `GET /engine/status` answers.
async fn spawn_server() -> (Child, String) {
    let port = free_port();
    let child = Command::new(SERVER_BIN)
        .arg("--port")
        .arg(port.to_string())
        .spawn()
        .expect("spawn gnosis-server");
    let base = format!("http://127.0.0.1:{port}");
    // Poll until the server is up (bounded).
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

/// Kill the server child on drop.
struct ServerGuard(Child);
impl Drop for ServerGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn did(id: &str) -> DocumentId {
    DocumentId(id.to_string())
}
fn wid(id: &str) -> WikiId {
    WikiId(id.to_string())
}

/// A well-formed `createDocument` request envelope for the given wiki.
fn create_document_envelope(wiki_id: &str) -> Envelope {
    encode_crud_request(
        CrudMethod::CreateDocument,
        CrudRequestArgs::CreateDocument {
            caller: "user:alice".to_string(),
            wiki_id: wid(wiki_id),
            body: CreateDocumentRequest {
                title: "Getting Started".to_string(),
                tags: Some(vec!["guide".to_string()]),
                author: Some("alice".to_string()),
            },
        },
    )
}

/// Pre-create a wiki via `POST /wikis` and return its id.
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

/// Pre-create a wiki + a document and return the document id.
async fn create_doc(client: &reqwest::Client, base: &str) -> String {
    let wiki_id = create_wiki(client, base).await;
    create_doc_in_wiki(client, base, &wiki_id).await
}

/// Create a document in an existing wiki and return its id.
async fn create_doc_in_wiki(client: &reqwest::Client, base: &str, wiki_id: &str) -> String {
    let env = encode_crud_request(
        CrudMethod::CreateDocument,
        CrudRequestArgs::CreateDocument {
            caller: "user:alice".to_string(),
            wiki_id: wid(wiki_id),
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

// ---------------------------------------------------------------------------
// §9 — the `EngineUnavailable`/`EngineError` split over a live transport.
// ---------------------------------------------------------------------------

/// §9 — a `rag_query` while the engine is not READY → `EngineUnavailable` → 503
/// over the live transport (FS-8).
#[tokio::test]
async fn rag_query_not_ready_returns_503() {
    let (child, base) = spawn_server().await;
    let _guard = ServerGuard(child);
    let client = reqwest::Client::new();

    let env = encode_crud_request(
        CrudMethod::GetDocument,
        CrudRequestArgs::GetDocument {
            document_id: did("d1"),
        },
    );
    // A fresh server's engine is not READY, so a rag_query must be 503.
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
        "rag_query on a not-READY engine must be EngineUnavailable (503)"
    );
    let _ = env; // the request envelope shape is exercised by the CRUD tests
}

/// §9 — the `EngineUnavailable`/`EngineError` split is distinct over the wire:
/// `EngineUnavailable`→503 and `EngineError`→502 are different statuses, and the
/// live not-READY path renders 503 (never 502).
#[test]
fn engine_error_502_distinct_from_503() {
    // Pure mapping: the two fail-states are distinct statuses.
    assert_eq!(server_status(&StoreError::EngineUnavailable).unwrap().0, 503);
    assert_eq!(server_status(&StoreError::EngineError).unwrap().0, 502);
    assert_ne!(
        server_status(&StoreError::EngineUnavailable).unwrap().0,
        server_status(&StoreError::EngineError).unwrap().0
    );
    // The request-decode outcome is a client 4xx, never a 502.
    assert_ne!(request_decode_status(&gnosis::DecodeError::UnknownMethod("x".to_string())).unwrap(), 502);
}

// ---------------------------------------------------------------------------
// §9 — the document-CRUD round-trips over the live transport.
// ---------------------------------------------------------------------------

/// §9 — a `createDocument` request envelope → a `Document` response envelope.
#[tokio::test]
async fn create_document_roundtrip() {
    let (child, base) = spawn_server().await;
    let _guard = ServerGuard(child);
    let client = reqwest::Client::new();

    // The server does NOT auto-create the wiki — pre-create it first.
    let wiki_id = create_wiki(&client, &base).await;
    let env = create_document_envelope(&wiki_id);
    let resp = client
        .post(format!("{base}/documents"))
        .json(&env)
        .send()
        .await
        .expect("POST /documents");
    assert_eq!(resp.status().as_u16(), 200, "createDocument must be 200");
    let text = resp.text().await.expect("response body");
    let resp_env = Envelope::from_json(&text).expect("response is an envelope");
    let decoded = decode_crud_response(&resp_env).expect("response decodes");
    assert!(
        matches!(decoded, gnosis::wire::crud::CrudResult::Document(_)),
        "createDocument must return a Document response envelope"
    );
}

/// §9 — a `ConflictError` → HTTP 409 over the live transport.
#[tokio::test]
async fn conflict_error_returns_409() {
    let (child, base) = spawn_server().await;
    let _guard = ServerGuard(child);
    let client = reqwest::Client::new();

    // Pre-create the wiki, then create a document (revision 0).
    let wiki_id = create_wiki(&client, &base).await;
    let create = create_document_envelope(&wiki_id);
    let resp = client
        .post(format!("{base}/documents"))
        .json(&create)
        .send()
        .await
        .expect("POST /documents");
    let text = resp.text().await.expect("create body");
    let resp_env = Envelope::from_json(&text).expect("create response envelope");
    let doc = match decode_crud_response(&resp_env).expect("create decodes") {
        gnosis::wire::crud::CrudResult::Document(d) => d,
        other => panic!("expected Document, got {other:?}"),
    };
    let doc_id = doc.document_id.0.clone();

    // Update with a stale base revision (0) twice: the second is a conflict.
    // The graph must be a valid Provident graph (exactly one doc-head and one
    // doc-end edge, per `Store::valid_provident_graph` at src/store/mod.rs:5429).
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

    // Same base_revision 0 again → stale → ConflictError → 409.
    let second = client
        .post(format!("{base}/documents/{doc_id}/update"))
        .json(&update)
        .send()
        .await
        .expect("second update");
    assert_eq!(
        second.status().as_u16(),
        409,
        "stale base revision must be ConflictError (409)"
    );
}

/// §9 — a `getDocument` request envelope → a `Document` response envelope.
#[tokio::test]
async fn get_document_roundtrip() {
    let (child, base) = spawn_server().await;
    let _guard = ServerGuard(child);
    let client = reqwest::Client::new();

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
    assert_eq!(resp.status().as_u16(), 200, "getDocument must be 200");
    let text = resp.text().await.expect("get body");
    let resp_env = Envelope::from_json(&text).expect("get response envelope");
    let decoded = decode_crud_response(&resp_env).expect("get decodes");
    assert!(
        matches!(decoded, CrudResult::Document(_)),
        "getDocument must return a Document response envelope"
    );
}

/// §9 — a `deleteDocument` request envelope → a void (`null`) response.
#[tokio::test]
async fn delete_document_roundtrip() {
    let (child, base) = spawn_server().await;
    let _guard = ServerGuard(child);
    let client = reqwest::Client::new();

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
    assert_eq!(resp.status().as_u16(), 200, "deleteDocument must be 200");
    let text = resp.text().await.expect("delete body");
    let resp_env = Envelope::from_json(&text).expect("delete response envelope");
    let decoded = decode_crud_response(&resp_env).expect("delete decodes");
    assert!(
        matches!(decoded, CrudResult::DeleteDocument),
        "deleteDocument must return a void response envelope"
    );
}

/// §9 — a `publishDocument` request envelope → a `Document` (Published) response.
#[tokio::test]
async fn publish_document_roundtrip() {
    let (child, base) = spawn_server().await;
    let _guard = ServerGuard(child);
    let client = reqwest::Client::new();

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
    assert_eq!(resp.status().as_u16(), 200, "publishDocument must be 200");
    let text = resp.text().await.expect("publish body");
    let resp_env = Envelope::from_json(&text).expect("publish response envelope");
    match decode_crud_response(&resp_env).expect("publish decodes") {
        CrudResult::Document(d) => assert_eq!(d.state, DocState::Published),
        other => panic!("expected Document, got {other:?}"),
    }
}

/// §9 — an `unpublishDocument` request envelope → a `Document` (Draft) response.
#[tokio::test]
async fn unpublish_document_roundtrip() {
    let (child, base) = spawn_server().await;
    let _guard = ServerGuard(child);
    let client = reqwest::Client::new();

    let doc_id = create_doc(&client, &base).await;
    // Publish first (unpublish requires a Published document).
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
    assert_eq!(resp.status().as_u16(), 200, "unpublishDocument must be 200");
    let text = resp.text().await.expect("unpublish body");
    let resp_env = Envelope::from_json(&text).expect("unpublish response envelope");
    match decode_crud_response(&resp_env).expect("unpublish decodes") {
        CrudResult::Document(d) => assert_eq!(d.state, DocState::Draft),
        other => panic!("expected Document, got {other:?}"),
    }
}

/// §9 — an `archiveDocument` request envelope → a `Document` (Archived) response.
#[tokio::test]
async fn archive_document_roundtrip() {
    let (child, base) = spawn_server().await;
    let _guard = ServerGuard(child);
    let client = reqwest::Client::new();

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
    assert_eq!(resp.status().as_u16(), 200, "archiveDocument must be 200");
    let text = resp.text().await.expect("archive body");
    let resp_env = Envelope::from_json(&text).expect("archive response envelope");
    match decode_crud_response(&resp_env).expect("archive decodes") {
        CrudResult::Document(d) => assert_eq!(d.state, DocState::Archived),
        other => panic!("expected Document, got {other:?}"),
    }
}

/// §9 — a `listDocuments` request envelope → a `DocumentList` response.
#[tokio::test]
async fn list_documents_roundtrip() {
    let (child, base) = spawn_server().await;
    let _guard = ServerGuard(child);
    let client = reqwest::Client::new();

    let wiki_id = create_wiki(&client, &base).await;
    let _doc_id = create_doc_in_wiki(&client, &base, &wiki_id).await;
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
    assert_eq!(resp.status().as_u16(), 200, "listDocuments must be 200");
    let text = resp.text().await.expect("list body");
    let resp_env = Envelope::from_json(&text).expect("list response envelope");
    let decoded = decode_crud_response(&resp_env).expect("list decodes");
    assert!(
        matches!(decoded, CrudResult::DocumentList(_)),
        "listDocuments must return a DocumentList response envelope"
    );
}

/// §9 — a `createWiki` request envelope → a `Wiki` response envelope.
#[tokio::test]
async fn create_wiki_roundtrip() {
    let (child, base) = spawn_server().await;
    let _guard = ServerGuard(child);
    let client = reqwest::Client::new();

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
    let decoded = decode_crud_response(&resp_env).expect("createWiki decodes");
    assert!(
        matches!(decoded, CrudResult::Wiki(_)),
        "createWiki must return a Wiki response envelope"
    );
}

/// §9 — a `getWiki` request envelope → a `Wiki` response envelope.
#[tokio::test]
async fn get_wiki_roundtrip() {
    let (child, base) = spawn_server().await;
    let _guard = ServerGuard(child);
    let client = reqwest::Client::new();

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
    assert_eq!(resp.status().as_u16(), 200, "getWiki must be 200");
    let text = resp.text().await.expect("getWiki body");
    let resp_env = Envelope::from_json(&text).expect("getWiki response envelope");
    let decoded = decode_crud_response(&resp_env).expect("getWiki decodes");
    assert!(
        matches!(decoded, CrudResult::Wiki(_)),
        "getWiki must return a Wiki response envelope"
    );
}

/// §9 — a `listWikis` request envelope → a `Vec<Wiki>` response envelope.
#[tokio::test]
async fn list_wikis_roundtrip() {
    let (child, base) = spawn_server().await;
    let _guard = ServerGuard(child);
    let client = reqwest::Client::new();

    let _wiki_id = create_wiki(&client, &base).await;
    let env = encode_crud_request(CrudMethod::ListWikis, CrudRequestArgs::ListWikis);
    let resp = client
        .get(format!("{base}/wikis"))
        .json(&env)
        .send()
        .await
        .expect("GET /wikis");
    assert_eq!(resp.status().as_u16(), 200, "listWikis must be 200");
    let text = resp.text().await.expect("listWikis body");
    let resp_env = Envelope::from_json(&text).expect("listWikis response envelope");
    let decoded = decode_crud_response(&resp_env).expect("listWikis decodes");
    assert!(
        matches!(decoded, CrudResult::WikiList(_)),
        "listWikis must return a Vec<Wiki> response envelope"
    );
}

// ---------------------------------------------------------------------------
// §9 — the request-decode fail-states over the live transport.
// ---------------------------------------------------------------------------

/// §9 — a malformed request → HTTP 400 over the live transport.
#[tokio::test]
async fn malformed_request_returns_400() {
    let (child, base) = spawn_server().await;
    let _guard = ServerGuard(child);
    let client = reqwest::Client::new();

    // Unparseable JSON body → InvalidJson → 400.
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
        "malformed request body must be 400"
    );
}

/// §9 — an unknown method → HTTP 422 over the live transport.
#[tokio::test]
async fn unknown_method_returns_422() {
    let (child, base) = spawn_server().await;
    let _guard = ServerGuard(child);
    let client = reqwest::Client::new();

    // Well-formed JSON with an unrecognized "method" → UnknownMethod → 422.
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
        "unknown method must be 422 (unprocessable)"
    );
}

/// §9 — `GET /engine/status` returns a `HealthReport` JSON (always 200).
#[tokio::test]
async fn engine_status_returns_health_report() {
    let (child, base) = spawn_server().await;
    let _guard = ServerGuard(child);
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base}/engine/status"))
        .send()
        .await
        .expect("GET /engine/status");
    assert_eq!(resp.status().as_u16(), 200, "engine/status is always 200");
    let text = resp.text().await.expect("status body");
    let report: serde_json::Value = serde_json::from_str(&text).expect("status is JSON");
    assert!(
        report.get("state").is_some(),
        "HealthReport must carry a state"
    );
    assert!(
        report.get("subsystems").is_some(),
        "HealthReport must carry subsystems"
    );
}
