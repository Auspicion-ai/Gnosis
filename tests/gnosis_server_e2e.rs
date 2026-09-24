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
///
/// **Ambient-provider spawner (H4).** This spawner INHERITS the environment, so
/// on a machine whose `GNOSIS_SERVER_OLLAMA_URL` points at a reachable Ollama
/// the boot takes `BootProvider::Reachable` and the engine comes up `Ready`. That
/// is the right spawner for the tests whose subject is **not** the readiness
/// precondition (the CRUD round-trips and the status surface, which are green in
/// either state). A test that needs the not-READY precondition MUST use
/// `spawn_server_without_provider()` instead, so the precondition is
/// deterministic rather than ambient (the provider-REACHABLE live outcome is the
/// live battery's named row `R-L2`).
async fn spawn_server() -> (Child, String) {
    let port = free_port();
    // The child is intentionally kept live for the test and reaped via
    // `ServerGuard`'s Drop (kill + wait), so no zombie is left.
    #[allow(clippy::zombie_processes)]
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
///
/// **H4 (hermeticity).** Spawned with the boot's provider configuration REMOVED
/// (`spawn_server_without_provider`), so the boot takes `BootProvider::Absent` ⇒
/// `Unavailable` and the not-READY precondition is deterministic **even on a
/// machine with Ollama running at the configured URL** (the inherited-env
/// spawner would boot `Ready` there and make this assertion ambient-dependent).
#[tokio::test]
async fn rag_query_not_ready_returns_503() {
    let (child, base) = spawn_server_without_provider().await;
    let _guard = ServerGuard(child);
    let client = reqwest::Client::new();

    let env = encode_crud_request(
        CrudMethod::GetDocument,
        CrudRequestArgs::GetDocument {
            document_id: did("d1"),
        },
    );
    // A provider-absent server's engine is not READY, so a rag_query must be 503.
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
    assert_eq!(
        server_status(&StoreError::EngineUnavailable).unwrap().0,
        503
    );
    assert_eq!(server_status(&StoreError::EngineError).unwrap().0, 502);
    assert_ne!(
        server_status(&StoreError::EngineUnavailable).unwrap().0,
        server_status(&StoreError::EngineError).unwrap().0
    );
    // The request-decode outcome is a client 4xx, never a 502.
    assert_ne!(
        request_decode_status(&gnosis::DecodeError::UnknownMethod("x".to_string())).unwrap(),
        502
    );
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

// ---------------------------------------------------------------------------
// §5.3/§5.4/§5.5 (U2) — the query-POST contract over the live transport.
// ---------------------------------------------------------------------------
//
// The register's named live assertions:
//   - `rag_query_unrecognized_mode_is_400_e2e` (`P-TP-2`'s (α), F10) — the
//     handler reads the WIRE `mode` through the shared decoder, so an
//     unrecognized token is 400 `validation_error` where the pre-U2 handler
//     ignored the field and returned 503;
//   - the envelope-strictness cases (a bare body stays rejected; a wrong
//     `schemaVersion`/`idFormat` on the query path ⇒ the pinned transport
//     code/status), incl. the non-object-payload `invalid_envelope` (F9);
//   - the `Content-Type: application/json` header (exactly, no `charset`) for
//     every transport decode body (F15);
//   - the `P-SM-4` (i)/(ii)/(iii) SSE wiring assertions.

/// The transport decode-error body of a response: `(content-type, code, status)`.
async fn decode_error_body(resp: reqwest::Response) -> (String, String, u16) {
    let status = resp.status().as_u16();
    let ctype = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("<absent>")
        .to_string();
    let text = resp.text().await.expect("decode-error body");
    let code = serde_json::from_str::<serde_json::Value>(&text)
        .ok()
        .and_then(|v| v.get("code").and_then(|c| c.as_str()).map(str::to_string))
        .unwrap_or_else(|| format!("<no code> body={text:?}"));
    (ctype, code, status)
}

/// §5.5 (F15) — every transport decode-error body is `application/json`
/// EXACTLY (no `charset`), never the pre-U1 `text/plain; charset=utf-8`.
fn assert_exact_json_content_type(ctype: &str, label: &str) {
    assert_eq!(
        ctype, "application/json",
        "{label}: Content-Type must be exactly `application/json` (no charset)"
    );
}

/// `P-TP-2`'s (α) — a POST with an unrecognized `mode` on a NOT-READY engine is
/// **400 `validation_error`** (the wire token is read by the shared decoder;
/// the pre-U2 handler ignored `mode` and returned 503).
#[tokio::test]
async fn rag_query_unrecognized_mode_is_400_e2e() {
    let (child, base) = spawn_server().await;
    let _guard = ServerGuard(child);
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "schemaVersion": 1,
        "idFormat": "opaque-string-v1",
        "payload": { "query": "x", "mode": "bm25" }
    });
    let resp = client
        .post(format!("{base}/rag/query"))
        .json(&body)
        .send()
        .await
        .expect("POST /rag/query mode=bm25");
    assert_eq!(
        resp.status().as_u16(),
        400,
        "an unrecognized `mode` token must be ValidationError (400), never 503/422"
    );
    let text = resp.text().await.expect("body");
    let json: serde_json::Value = serde_json::from_str(&text).expect("error envelope is JSON");
    let code = json
        .get("payload")
        .and_then(|p| p.get("code"))
        .and_then(|c| c.as_str())
        .unwrap_or("<absent>");
    assert_eq!(
        code, "validation_error",
        "the mode token failure renders the §11 `validation_error` code"
    );
}

/// §5.4 — the boundary token states on POST: `""` and whitespace are
/// present-but-unrecognized ⇒ 400; every ASCII casing of a member is accepted.
///
/// **H4 (hermeticity).** Spawned with the provider configuration removed, so the
/// accepted-casing leg is deterministic: it reaches the not-READY gate ⇒ **503**
/// (`EngineUnavailable`) on a provider-absent boot, never an ambient-provider
/// `200`.
#[tokio::test]
async fn rag_query_mode_token_boundaries_e2e() {
    let (child, base) = spawn_server_without_provider().await;
    let _guard = ServerGuard(child);
    let client = reqwest::Client::new();

    for token in ["", " ", "flat ", "flat\n", " hybrid"] {
        let body = serde_json::json!({
            "schemaVersion": 1,
            "idFormat": "opaque-string-v1",
            "payload": { "query": "x", "mode": token }
        });
        let resp = client
            .post(format!("{base}/rag/query"))
            .json(&body)
            .send()
            .await
            .expect("POST /rag/query");
        assert_eq!(
            resp.status().as_u16(),
            400,
            "mode {token:?} is present-but-unrecognized ⇒ 400, never a silent default"
        );
    }

    // A valid casing variant is NOT a 400 — it resolves to the typed enum and
    // reaches the not-READY gate ⇒ 503 on the provider-absent boot (H4: the
    // spawner removes the provider configuration, so this leg is deterministic).
    let body = serde_json::json!({
        "schemaVersion": 1,
        "idFormat": "opaque-string-v1",
        "payload": { "query": "x", "mode": "HYBRID" }
    });
    let resp = client
        .post(format!("{base}/rag/query"))
        .json(&body)
        .send()
        .await
        .expect("POST /rag/query mode=HYBRID");
    assert_ne!(
        resp.status().as_u16(),
        400,
        "an ASCII-casing member token must not be a token failure"
    );
}

/// §5.3 — the query POST path is envelope-strict: a bare (non-envelope) body is
/// rejected with a transport code (400), and so are a wrong `schemaVersion`
/// (`unsupported_schema_version`) and a wrong `idFormat` (`unknown_id_format`).
#[tokio::test]
async fn rag_query_envelope_strict_e2e() {
    let (child, base) = spawn_server().await;
    let _guard = ServerGuard(child);
    let client = reqwest::Client::new();

    // (a) a bare body — the GR-1 back-compat tolerance is REFUSED.
    let resp = client
        .post(format!("{base}/rag/query"))
        .json(&serde_json::json!({"query": "x"}))
        .send()
        .await
        .expect("POST /rag/query bare");
    let (ctype, code, status) = decode_error_body(resp).await;
    assert_eq!(status, 400, "a bare body is rejected (400)");
    assert_eq!(
        code, "invalid_envelope",
        "a bare body is a transport `invalid_envelope`, never a §11 code"
    );
    assert_exact_json_content_type(&ctype, "bare body");

    // (b) an unknown schemaVersion ⇒ 400 unsupported_schema_version.
    let resp = client
        .post(format!("{base}/rag/query"))
        .json(&serde_json::json!({
            "schemaVersion": 99,
            "idFormat": "opaque-string-v1",
            "payload": {"query": "x"}
        }))
        .send()
        .await
        .expect("POST /rag/query schemaVersion=99");
    let (ctype, code, status) = decode_error_body(resp).await;
    assert_eq!(status, 400, "an unknown schemaVersion is 400");
    assert_eq!(code, "unsupported_schema_version");
    assert_exact_json_content_type(&ctype, "schemaVersion 99");

    // (c) an unknown idFormat ⇒ 400 unknown_id_format.
    let resp = client
        .post(format!("{base}/rag/query"))
        .json(&serde_json::json!({
            "schemaVersion": 1,
            "idFormat": "uuid-v4",
            "payload": {"query": "x"}
        }))
        .send()
        .await
        .expect("POST /rag/query idFormat=uuid-v4");
    let (ctype, code, status) = decode_error_body(resp).await;
    assert_eq!(status, 400, "an unknown idFormat is 400");
    assert_eq!(code, "unknown_id_format");
    assert_exact_json_content_type(&ctype, "idFormat uuid-v4");

    // (d) a non-object payload ⇒ 400 invalid_envelope ALONE (F9).
    let resp = client
        .post(format!("{base}/rag/query"))
        .json(&serde_json::json!({
            "schemaVersion": 1,
            "idFormat": "opaque-string-v1",
            "payload": 5
        }))
        .send()
        .await
        .expect("POST /rag/query payload=5");
    let (ctype, code, status) = decode_error_body(resp).await;
    assert_eq!(status, 400, "a non-object payload is 400");
    assert_eq!(
        code, "invalid_envelope",
        "a non-object payload is `invalid_envelope`, never `invalid_json`/`validation_error`"
    );
    assert_exact_json_content_type(&ctype, "non-object payload");
}

/// V-15 / V-15.1 rendered at the transport: the byte-exact body of each pinned
/// transport decode error, produced by the ONE shared renderer both handlers
/// call (CRUD + query), with `Content-Type: application/json` exactly.
#[tokio::test]
async fn v15_transport_decode_error_body_exact_e2e() {
    let (child, base) = spawn_server().await;
    let _guard = ServerGuard(child);
    let client = reqwest::Client::new();

    // V-15 (1/2): the CRUD path, `UnknownMethod("bogus")` ⇒ 422 + exact bytes.
    let resp = client
        .post(format!("{base}/documents"))
        .json(&serde_json::json!({
            "schemaVersion": 1,
            "idFormat": "opaque-string-v1",
            "payload": {"method": "bogus", "args": {}}
        }))
        .send()
        .await
        .expect("POST /documents");
    let status = resp.status().as_u16();
    let ctype = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("<absent>")
        .to_string();
    let body = resp.text().await.expect("body");
    assert_eq!(status, 422, "V-15 unknown_method status");
    assert_exact_json_content_type(&ctype, "V-15 unknown_method");
    assert_eq!(
        body, r#"{"code":"unknown_method","message":"bogus"}"#,
        "V-15 unknown_method body must be byte-exact"
    );

    // V-15 (2/2): the QUERY path, `UnsupportedSchemaVersion(99)` ⇒ 400 + bytes.
    let resp = client
        .post(format!("{base}/rag/query"))
        .json(&serde_json::json!({
            "schemaVersion": 99,
            "idFormat": "opaque-string-v1",
            "payload": {"query": "x"}
        }))
        .send()
        .await
        .expect("POST /rag/query");
    let status = resp.status().as_u16();
    let ctype = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("<absent>")
        .to_string();
    let body = resp.text().await.expect("body");
    assert_eq!(status, 400, "V-15 unsupported_schema_version status");
    assert_exact_json_content_type(&ctype, "V-15 unsupported_schema_version");
    assert_eq!(
        body, r#"{"code":"unsupported_schema_version","message":"unsupported schemaVersion: 99"}"#,
        "V-15 unsupported_schema_version body must be byte-exact"
    );

    // V-15.1: the query path, a non-object payload ⇒ 400 `invalid_envelope`.
    let resp = client
        .post(format!("{base}/rag/query"))
        .json(&serde_json::json!({
            "schemaVersion": 1,
            "idFormat": "opaque-string-v1",
            "payload": 5
        }))
        .send()
        .await
        .expect("POST /rag/query");
    let status = resp.status().as_u16();
    let ctype = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("<absent>")
        .to_string();
    let body = resp.text().await.expect("body");
    assert_eq!(status, 400, "V-15.1 status");
    assert_exact_json_content_type(&ctype, "V-15.1");
    assert_eq!(
        body, r#"{"code":"invalid_envelope","message":"envelope payload must be a JSON object"}"#,
        "V-15.1 non-object-payload body must be byte-exact"
    );
}

/// §5.3 — the load-bearing tolerance: an absent `mode` and an extra `args` key
/// still reach the READY gate (503 on a not-READY server), so the pre-U2 e2e
/// POSTs stay valid.
///
/// **H4 (hermeticity).** Spawned with the provider configuration removed, so the
/// `503` leg is deterministic regardless of any ambient Ollama.
#[tokio::test]
async fn rag_query_absent_mode_and_extra_key_still_503_e2e() {
    let (child, base) = spawn_server_without_provider().await;
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
        "absent mode ⇒ Flat and an extra key is tolerated ⇒ the READY gate (503)"
    );
}

/// **T-6 — the decode-level non-merge probe at the transport seam.**
///
/// §9.5.1's F6 signature note pins that the decoder takes the SSE params as a
/// third input and "MUST NOT merge the two inputs (a POST body that carried no
/// `mode` key MUST NOT be defaulted from an SSE param list and vice versa)". The
/// live POST above cannot observe that argument (the handler passes its own
/// `SseParams::default()` on the POST path), so the pin is asserted here at the
/// `decode_query_request` seam the handler itself calls:
///
/// - a POST payload that omits `mode`/`topK` and an SSE param list that carries
///   `query`/`topK`/`mode` ⇒ **`("x", mode == None, top_k == None)`** — the SSE
///   values must NOT leak into the POST decode;
/// - the mirror case: `QueryPath::Sse` with the *same* env and params reads the
///   params (`"q"`, `Some(7)`, `Some(Graph)`) and ignores the envelope payload —
///   the two inputs are never merged in either direction.
#[test]
fn rag_query_post_sse_params_are_not_merged_decode_level_e2e() {
    use gnosis::wire::query::{decode_query_request, QueryPath, SseParams};
    use gnosis::QueryMode;

    let env = Envelope {
        schema_version: 1,
        id_format: "opaque-string-v1".to_string(),
        payload: serde_json::json!({"query": "x"}),
    };
    let params = SseParams {
        query: Some("q"),
        top_k: Some("7"),
        mode: Some("graph"),
    };

    // (a) POST: the SSE params are UNREAD (F6) — never defaulted into the body.
    let (q, o) = decode_query_request(QueryPath::Post, &env, params)
        .expect("a well-formed POST payload decodes");
    assert_eq!(q, "x", "the POST `query` half must come from the payload");
    assert_eq!(
        o.mode, None,
        "an absent POST `mode` must stay None — the SSE param must not be merged in"
    );
    assert_eq!(
        o.top_k, None,
        "an absent POST `topK` must stay None — the SSE param must not be merged in"
    );

    // (b) SSE: the params ARE the input; the (empty) payload contributes nothing.
    let (q, o) =
        decode_query_request(QueryPath::Sse, &env, params).expect("well-formed SSE params decode");
    assert_eq!(q, "q", "the SSE `query` param is the input on the SSE path");
    assert_eq!(o.mode, Some(QueryMode::Graph));
    assert_eq!(o.top_k, Some(7));
}

/// §5.5 — the transport decode body replaces the pre-U1 plain text for EVERY
/// route that can fail to decode (the CRUD path too): JSON, exact
/// `application/json`, and the pinned code/status per variant.
#[tokio::test]
async fn transport_decode_error_body_is_json_on_crud_path_e2e() {
    let (child, base) = spawn_server().await;
    let _guard = ServerGuard(child);
    let client = reqwest::Client::new();

    // An unknown method ⇒ 422 unknown_method (the V-15 422 golden on the wire).
    let resp = client
        .post(format!("{base}/documents"))
        .json(&serde_json::json!({
            "schemaVersion": 1,
            "idFormat": "opaque-string-v1",
            "payload": {"method": "bogus", "args": {}}
        }))
        .send()
        .await
        .expect("POST /documents unknown method");
    let (ctype, code, status) = decode_error_body(resp).await;
    assert_eq!(status, 422, "an unknown method is 422");
    assert_eq!(code, "unknown_method");
    assert_exact_json_content_type(&ctype, "unknown method");

    // Unparseable JSON ⇒ 400 invalid_json.
    let resp = client
        .post(format!("{base}/documents"))
        .body("{ not json")
        .header("content-type", "application/json")
        .send()
        .await
        .expect("POST /documents malformed");
    let (ctype, code, status) = decode_error_body(resp).await;
    assert_eq!(status, 400, "unparseable JSON is 400");
    assert_eq!(code, "invalid_json");
    assert_exact_json_content_type(&ctype, "unparseable json");
}

/// `P-SM-4` wiring (i) — `?mode=bm25` on `/rag/stream` ⇒ **HTTP 400** with the
/// single `error` frame carrying `validation_error` (the status AND the body are
/// pinned: never an `error` frame under a 200).
#[tokio::test]
async fn rag_stream_unrecognized_mode_is_400_with_error_frame_e2e() {
    let (child, base) = spawn_server().await;
    let _guard = ServerGuard(child);
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base}/rag/stream?query=x&mode=bm25"))
        .send()
        .await
        .expect("GET /rag/stream mode=bm25");
    assert_eq!(
        resp.status().as_u16(),
        400,
        "an unrecognized SSE `mode` is HTTP 400, not a 200 carrying an error frame"
    );
    let text = resp.text().await.expect("stream body");
    assert!(
        text.contains(r#""type":"error""#),
        "the 400 body is the single error frame, got {text:?}"
    );
    assert!(
        text.contains(r#""code":"validation_error""#),
        "the frame carries the §11 `validation_error` code, got {text:?}"
    );
    assert!(
        text.ends_with("\n\n") && text.starts_with("event: error\ndata: "),
        "the frame uses the F2 §4.4 single-event framing, got {text:?}"
    );
}

/// `P-SM-4` wiring (ii) — `?mode=HYBRID` ⇒ NON-400 (the casing variant is a
/// member token; the pre-U2 case-sensitive match would have silently defaulted).
#[tokio::test]
async fn rag_stream_valid_mode_casing_is_not_400_e2e() {
    let (child, base) = spawn_server().await;
    let _guard = ServerGuard(child);
    let client = reqwest::Client::new();

    for token in ["HYBRID", "Graph", "fLaT", "vector"] {
        let resp = client
            .get(format!("{base}/rag/stream?query=x&mode={token}"))
            .send()
            .await
            .expect("GET /rag/stream");
        assert_ne!(
            resp.status().as_u16(),
            400,
            "mode={token} is an ASCII-casing member token ⇒ non-400"
        );
    }
}

/// `P-SM-4` wiring (iii) — `?expand=`/`?compression=` do NOT exist on the SSE
/// surface (§5.4's N1): they are ignored ⇒ the option's default ⇒ NEVER a 400
/// and never an `error` frame.
#[tokio::test]
async fn rag_stream_expand_compression_params_are_ignored_e2e() {
    let (child, base) = spawn_server().await;
    let _guard = ServerGuard(child);
    let client = reqwest::Client::new();

    for q in [
        "expand=parent",
        "compression=nope",
        "expand=",
        "compression=",
    ] {
        let resp = client
            .get(format!("{base}/rag/stream?query=x&{q}"))
            .send()
            .await
            .expect("GET /rag/stream");
        assert_ne!(
            resp.status().as_u16(),
            400,
            "`?{q}` is an unknown SSE param ⇒ ignored ⇒ non-400 (the params do not exist)"
        );
    }
}

/// §5.4 — the SSE surface reads exactly `query`/`topK`/`mode`: an unparseable
/// `topK` is tolerated (⇒ default 10) and any other param is ignored, never 400.
#[tokio::test]
async fn rag_stream_tolerates_bad_topk_and_unknown_params_e2e() {
    let (child, base) = spawn_server().await;
    let _guard = ServerGuard(child);
    let client = reqwest::Client::new();

    for q in [
        "topK=not-a-number",
        "topK=",
        "wikiId=w1",
        "filters=%7B%7D",
        "maxHops=2",
    ] {
        let resp = client
            .get(format!("{base}/rag/stream?query=x&{q}"))
            .send()
            .await
            .expect("GET /rag/stream");
        assert_ne!(
            resp.status().as_u16(),
            400,
            "`?{q}` must be tolerated (unparseable ⇒ default / unknown ⇒ ignored)"
        );
    }
}

// ---------------------------------------------------------------------------
// §5.8 / §9.5.2 (U3) — the status-honesty contract over the live transport.
// ---------------------------------------------------------------------------
//
// The register's named live assertion for U3 is
// **`engine_status_reports_no_false_embedding_claim`** (§9.5.2's adjudication
// note 1 / §9.5.4's `P-IM-9` coverage note): the provider-**absent** boot is
// transport-assertable today, and it must NOT report `embedding: true`. The
// provider-**reachable** live-server outcome (`Ready` + `embedding:true` from
// `GET /engine/status`) needs a **controlled provider** (an Ollama-compatible
// endpoint answering `/api/tags`) and is asserted by the **live-scenario
// battery's named row `R-L2`**, never here — this suite deliberately does not
// require a live provider.

/// Spawn the server with the boot's provider configuration **removed**, so the
/// boot takes the provider-ABSENT branch (`BootProvider::Absent`) regardless of
/// the ambient environment. The boot with no provider configured wires none and
/// leaves the construction state `EngineState::Unavailable` (§9.5.2's boot
/// outcome table), so the honest flag vector is
/// `{store:true, graph:true, lexical:true, vector:false, embedding:false,
/// reranker:false}`.
async fn spawn_server_without_provider() -> (Child, String) {
    let port = free_port();
    #[allow(clippy::zombie_processes)]
    let child = Command::new(SERVER_BIN)
        .arg("--port")
        .arg(port.to_string())
        .env_remove("GNOSIS_SERVER_OLLAMA_URL")
        .env_remove("GNOSIS_SERVER_OLLAMA_MODEL")
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

/// §5.8 / §9.5.2 (U3) — `engine_status_reports_no_false_embedding_claim`: a
/// provider-absent boot reports the HONEST flag vector — **never**
/// `embedding: true` (the false claim §5.8 calls out for the DEGRADED boot and
/// which today's hard-coded flags emit for every state).
///
/// Pinned assertions (all derived from §5.8's capability semantics + §9.5.2's
/// boot outcome table, not from the implementation):
///   * `GET /engine/status` stays the single status surface and answers **200**
///     (always-200 row, §5.8/§10);
///   * `subsystems` carries exactly the six frozen keys (F7);
///   * with no provider wired: `embedding == false`;
///   * `store`/`graph`/`lexical` are `true` under capability semantics (the
///     lexical leg is a live shard scan — no prebuilt index);
///   * `reranker == false` in every reachable state (no implementation exists);
///   * `vector == false` until U5's boot index build lands (the boot swaps in
///     `DerivedIndexes::default()`), the honest U3-time value;
///   * the engine state is NOT a fabricated `Ready` (the absent-provider branch
///     leaves `Unavailable` — §9.5.2's note 2).
#[tokio::test]
async fn engine_status_reports_no_false_embedding_claim() {
    let (child, base) = spawn_server_without_provider().await;
    let _guard = ServerGuard(child);
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base}/engine/status"))
        .send()
        .await
        .expect("GET /engine/status");
    assert_eq!(resp.status().as_u16(), 200, "engine/status is always 200");
    let report: serde_json::Value = resp.json().await.expect("status body is JSON");

    let flags = report
        .get("subsystems")
        .and_then(|v| v.as_object())
        .expect("HealthReport carries a subsystems object");
    let mut keys: Vec<&str> = flags.keys().map(|k| k.as_str()).collect();
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
        "the six-bool EngineSubsystems shape is frozen (none added/lost/re-typed)"
    );
    for (k, v) in flags.iter() {
        assert!(v.is_boolean(), "flag {k:?} must be a JSON boolean");
    }

    // The honesty pin: a boot that wired no provider makes NO embedding claim.
    assert_eq!(
        flags["embedding"],
        serde_json::Value::Bool(false),
        "a provider-absent boot must not claim embedding:true (§5.8's false-claim defect)"
    );
    // The capability-honest core legs.
    for k in ["store", "graph", "lexical"] {
        assert_eq!(
            flags[k],
            serde_json::Value::Bool(true),
            "{k} is honest under capability semantics"
        );
    }
    // No reranker implementation exists anywhere in src/ ⇒ false, always.
    assert_eq!(
        flags["reranker"],
        serde_json::Value::Bool(false),
        "reranker must be false in every reachable state (no implementation exists)"
    );
    // Post-U5: this boot's provider is ABSENT (the spawn helper strips both
    // provider env vars) ⇒ no index is built ⇒ `vector:false` by construction.
    assert_eq!(
        flags["vector"],
        serde_json::Value::Bool(false),
        "vector must be false when no provider is configured (no build is attempted)"
    );
    // The state axis is separate (F13) — but it must never be a fabricated Ready.
    assert_ne!(
        report.get("state").and_then(|v| v.as_str()),
        Some("Ready"),
        "a boot with no reachable provider must not fabricate Ready"
    );
}
