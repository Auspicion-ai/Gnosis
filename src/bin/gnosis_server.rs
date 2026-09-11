//! §7.2 P2 — `gnosis-server` — the thin HTTP/SSE transport host.
//!
//! Binds loopback-only to `127.0.0.1:<port>` (a non-loopback bind is a
//! fail-state; the shell owns bind/auth/TLS policy). Hosts the retrieval trio +
//! health (`POST /rag/query`, `GET /rag/stream` SSE, `GET /engine/status`) AND
//! the 11 document-CRUD REST endpoints (paths consumed verbatim from P1a
//! `ENGINE_ENDPOINTS`). Renders the §11 HTTP-status map server-side (plus the
//! NEW-2 request-decode outcome 400/422, never 502) and threads the RBAC
//! `caller` through the request decode to the engine.
//! Contract: `docs/specs/p2-gnosis-server.md`.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use futures::StreamExt;

use gnosis::store::Store;
use gnosis::wire::crud::{
    decode_crud_request, encode_crud_error, encode_crud_response, CrudMethod, CrudRequestArgs,
    CrudResult,
};
use gnosis::wire::decode::DecodeError;
use gnosis::wire::envelope::Envelope;
use gnosis::wire::sse::encode_event;
use gnosis::wire::status::health;
use gnosis::{
    request_decode_status, server_status, DerivedIndexes, EmbeddingProvider, EngineState,
    QueryMode, RagQueryOptions, RagStore, StoreError,
};

type AppState = Arc<Store>;

/// Parse the `--port <port>` CLI arg (default 8080). The bind address is fixed
/// to loopback `127.0.0.1` (not configurable to a non-loopback value).
fn parse_port() -> u16 {
    let args: Vec<String> = std::env::args().collect();
    if let Some(i) = args.iter().position(|a| a == "--port") {
        if let Some(p) = args.get(i + 1) {
            if let Ok(p) = p.parse() {
                return p;
            }
        }
    }
    8080
}

/// Render a request-decode `DecodeError` as its transport status (400/422,
/// never 502) with a plain body.
fn decode_error_response(e: &DecodeError) -> Response {
    let status = request_decode_status(e).unwrap_or(400);
    (
        StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_REQUEST),
        "request decode failed",
    )
        .into_response()
}

/// Dispatch a decoded CRUD request to the engine and render the response
/// envelope (or the §11-mapped error). The `caller` is threaded through the
/// request decode (the decode layer validates its presence on mutating
/// methods); engine-side RBAC enforcement is a PACKAGE item, so the engine
/// methods are called without a `caller` param.
async fn dispatch(store: Arc<Store>, method: CrudMethod, args: CrudRequestArgs) -> Response {
    use CrudMethod::*;
    let result: Result<CrudResult, StoreError> = match (method, args) {
        (CreateDocument, CrudRequestArgs::CreateDocument { wiki_id, body, .. }) => store
            .create_document(&wiki_id, body)
            .await
            .map(CrudResult::Document),
        (GetDocument, CrudRequestArgs::GetDocument { document_id }) => store
            .get_document(&document_id)
            .await
            .map(CrudResult::Document),
        (UpdateDocument, CrudRequestArgs::UpdateDocument { document_id, body, .. }) => store
            .update_document(&document_id, body)
            .await
            .map(CrudResult::Document),
        (DeleteDocument, CrudRequestArgs::DeleteDocument { document_id, .. }) => store
            .delete_document(&document_id)
            .await
            .map(|_| CrudResult::DeleteDocument),
        (PublishDocument, CrudRequestArgs::PublishDocument { document_id, .. }) => store
            .publish_document(&document_id)
            .await
            .map(CrudResult::Document),
        (UnpublishDocument, CrudRequestArgs::UnpublishDocument { document_id, .. }) => store
            .unpublish_document(&document_id)
            .await
            .map(CrudResult::Document),
        (ArchiveDocument, CrudRequestArgs::ArchiveDocument { document_id, .. }) => store
            .archive_document(&document_id)
            .await
            .map(CrudResult::Document),
        (ListDocuments, CrudRequestArgs::ListDocuments { wiki_id, body }) => store
            .list_documents(&wiki_id, &body)
            .await
            .map(CrudResult::DocumentList),
        (CreateWiki, CrudRequestArgs::CreateWiki { name, .. }) => store
            .create_wiki(&name)
            .await
            .map(CrudResult::Wiki),
        (GetWiki, CrudRequestArgs::GetWiki { wiki_id }) => {
            store.get_wiki(&wiki_id).await.map(CrudResult::Wiki)
        }
        (ListWikis, CrudRequestArgs::ListWikis) => {
            store.list_wikis().await.map(CrudResult::WikiList)
        }
        _ => return (StatusCode::BAD_REQUEST, "method/args mismatch").into_response(),
    };
    match result {
        Ok(r) => {
            let env = encode_crud_response(method, r);
            (StatusCode::OK, Json(env)).into_response()
        }
        Err(e) => {
            let (status, _) = server_status(&e).unwrap_or((500, ""));
            let env = encode_crud_error(method, &e);
            (
                StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(env),
            )
                .into_response()
        }
    }
}

/// The generic CRUD handler: parse the request envelope, decode the method +
/// args (threading the `caller`), and dispatch to the engine.
async fn crud_handler(State(store): State<AppState>, body: String) -> Response {
    let env = match Envelope::from_json(&body) {
        Ok(env) => env,
        Err(e) => return decode_error_response(&e),
    };
    let (method, args) = match decode_crud_request(&env) {
        Ok(x) => x,
        Err(e) => return decode_error_response(&e),
    };
    dispatch(store, method, args).await
}

/// `POST /rag/query` — request envelope → `RagResult` response envelope (or the
/// §11-mapped error envelope; a not-READY engine → `EngineUnavailable` → 503).
async fn rag_query_handler(State(store): State<AppState>, body: String) -> Response {
    let env = match Envelope::from_json(&body) {
        Ok(env) => env,
        Err(e) => return decode_error_response(&e),
    };
    let query = env
        .payload
        .get("query")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    match store.rag_query(&query, &RagQueryOptions::default()).await {
        Ok(res) => {
            let env = gnosis::wire::codecs::encode_result(&res);
            (StatusCode::OK, Json(env)).into_response()
        }
        Err(e) => {
            let (status, _) = server_status(&e).unwrap_or((500, ""));
            let env = gnosis::wire::codecs::encode_error(&e);
            (
                StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(env),
            )
                .into_response()
        }
    }
}

/// `GET /rag/stream` — SSE frames (`result`/`done`/`error`). Parses the query
/// params and streams the real `rag_stream` output. A not-READY engine emits an
/// `error` SSE frame.
async fn rag_stream_handler(
    State(store): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let query = params.get("query").cloned().unwrap_or_default();
    let options = RagQueryOptions {
        top_k: params.get("topK").and_then(|v| v.parse().ok()),
        mode: params.get("mode").and_then(|m| match m.as_str() {
            "flat" => Some(QueryMode::Flat),
            "graph" => Some(QueryMode::Graph),
            "vector" => Some(QueryMode::Vector),
            "hybrid" => Some(QueryMode::Hybrid),
            _ => None,
        }),
        ..Default::default()
    };
    match store.rag_stream(&query, &options).await {
        Ok(stream) => {
            let body = Body::from_stream(stream.map(|chunk| {
                Ok::<_, std::convert::Infallible>(axum::body::Bytes::from(encode_event(&chunk)))
            }));
            (StatusCode::OK, body).into_response()
        }
        Err(e) => {
            let (status, _) = server_status(&e).unwrap_or((500, ""));
            let frame = format!(
                "event: error\ndata: {{\"type\":\"error\",\"code\":\"{}\",\"message\":\"{}\"}}\n\n",
                e.wire_code(),
                e
            );
            (
                StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                frame,
            )
                .into_response()
        }
    }
}

/// `GET /engine/status` — the `HealthReport` JSON (always 200).
async fn engine_status_handler(State(store): State<AppState>) -> Response {
    let status = store.get_engine_status().await;
    let report = health(&status);
    (StatusCode::OK, Json(report)).into_response()
}

fn router(store: Arc<Store>) -> Router {
    Router::new()
        .route("/rag/query", post(rag_query_handler))
        .route("/rag/stream", get(rag_stream_handler))
        .route("/engine/status", get(engine_status_handler))
        .route("/documents", post(crud_handler).get(crud_handler))
        .route("/documents/:id", get(crud_handler).delete(crud_handler))
        .route("/documents/:id/update", post(crud_handler))
        .route("/documents/:id/publish", post(crud_handler))
        .route("/documents/:id/unpublish", post(crud_handler))
        .route("/documents/:id/archive", post(crud_handler))
        .route("/wikis", post(crud_handler).get(crud_handler))
        .route("/wikis/:id", get(crud_handler))
        .with_state(store)
}

/// §4.5.3 — the embedding provider the server wires at boot (a local Ollama
/// HTTP provider, mirroring the `gnosis-eval` `LiveProvider`). Configured via
/// `GNOSIS_SERVER_OLLAMA_URL` (default `http://localhost:11434`) and
/// `GNOSIS_SERVER_OLLAMA_MODEL` (default `nomic-embed-text`).
struct OllamaProvider {
    base_url: String,
    model: String,
    client: reqwest::Client,
}

impl OllamaProvider {
    /// `Some` only when `GNOSIS_SERVER_OLLAMA_URL` is set (the server does NOT
    /// fabricate a provider — without the env var no provider is wired).
    fn from_env() -> Option<Self> {
        let base_url = std::env::var("GNOSIS_SERVER_OLLAMA_URL").ok()?;
        let model = std::env::var("GNOSIS_SERVER_OLLAMA_MODEL")
            .unwrap_or_else(|_| "nomic-embed-text".to_string());
        Some(Self {
            base_url,
            model,
            client: reqwest::Client::new(),
        })
    }
}

impl EmbeddingProvider for OllamaProvider {
    fn embed(
        &self,
        text: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<f32>, StoreError>> + Send + '_>> {
        let url = format!("{}/api/embed", self.base_url);
        let model = self.model.clone();
        let client = self.client.clone();
        let text = text.to_string();
        Box::pin(async move {
            let resp = client
                .post(&url)
                .json(&serde_json::json!({ "model": model, "input": text }))
                .send()
                .await
                .map_err(|_| StoreError::EmbeddingUnavailable)?;
            let body: serde_json::Value = resp
                .json()
                .await
                .map_err(|_| StoreError::EmbeddingUnavailable)?;
            let emb = body
                .get("embeddings")
                .and_then(|v| v.as_array())
                .and_then(|a| a.first())
                .and_then(|v| v.as_array())
                .ok_or(StoreError::EmbeddingUnavailable)?
                .iter()
                .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                .collect();
            Ok(emb)
        })
    }
    fn is_available(&self) -> Pin<Box<dyn Future<Output = bool> + Send + '_>> {
        let url = format!("{}/api/tags", self.base_url);
        let client = self.client.clone();
        Box::pin(async move {
            client
                .get(&url)
                .send()
                .await
                .map(|r| r.status().is_success())
                .unwrap_or(false)
        })
    }
}

#[tokio::main]
async fn main() {
    let port = parse_port();
    let store = Arc::new(Store::new());
    // §6 boot sequence: build the derived-indexes snapshot, wire the embedding
    // provider, and transition to READY only when a provider is available. The
    // server does NOT fabricate READY — without a wired provider the engine
    // stays DEGRADED/UNAVAILABLE and `GET /engine/status` reflects that.
    store.swap_snapshot(DerivedIndexes::default());
    if let Some(provider) = OllamaProvider::from_env() {
        if provider.is_available().await {
            store.set_embedding_provider(Arc::new(provider));
            store.set_engine_state(EngineState::Ready);
        } else {
            store.set_engine_state(EngineState::Degraded);
        }
    }
    let app = router(store);
    let addr = format!("127.0.0.1:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("bind loopback 127.0.0.1");
    axum::serve(listener, app).await.expect("serve");
}
