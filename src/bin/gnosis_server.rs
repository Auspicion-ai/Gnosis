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
    http::{header, StatusCode},
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
    boot_wiring, build_boot_vector_index, decode_query_request, request_decode_status,
    server_status, BootProvider, DerivedIndexes, EmbeddingProvider, QueryDecodeError, QueryPath,
    RagStore, SseParams, StoreError,
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
/// never 502) with the §5.5 JSON body: `{"code","message"}`, no envelope
/// wrapper, `Content-Type: application/json` exactly (F15). The ONE shared
/// renderer for every route that can fail to decode a request (CRUD + query).
fn decode_error_response(e: &DecodeError) -> Response {
    let status = request_decode_status(e).unwrap_or(400);
    let body = serde_json::json!({
        "code": gnosis::request_decode_code(e).unwrap_or("invalid_json"),
        "message": gnosis::request_decode_message(e),
    })
    .to_string();
    (
        StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_REQUEST),
        [(header::CONTENT_TYPE, "application/json")],
        body,
    )
        .into_response()
}

/// Render the shared query decoder's outcome: a transport failure goes through
/// the one shared decode-error renderer; a validation failure is the §11-mapped
/// `ValidationError` (400 `validation_error`) as an error envelope (§5.4).
fn query_decode_error_response(e: QueryDecodeError) -> Response {
    match e {
        QueryDecodeError::Transport(e) => decode_error_response(&e),
        QueryDecodeError::Validation(e) => {
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

/// §5.4 — the SSE half of the decode-error rendering: the pre-stream `mode`
/// token failure is HTTP **400** (the §11 `validation_error` status) with the
/// single F2 §4.4 `error` frame as the body — never an `error` frame under a
/// 200, and never the POST shape. A transport failure has no SSE frame form, so
/// it falls back to the shared JSON renderer.
fn sse_decode_error_response(e: QueryDecodeError) -> Response {
    match e {
        QueryDecodeError::Transport(e) => decode_error_response(&e),
        QueryDecodeError::Validation(e) => {
            let (status, code) = server_status(&e).unwrap_or((500, ""));
            let frame = encode_event(&gnosis::store::RagChunk::Error(e));
            debug_assert_eq!(code, "validation_error");
            (
                StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                [(header::CONTENT_TYPE, "text/event-stream")],
                frame,
            )
                .into_response()
        }
    }
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
        (
            UpdateDocument,
            CrudRequestArgs::UpdateDocument {
                document_id, body, ..
            },
        ) => store
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
        (CreateWiki, CrudRequestArgs::CreateWiki { name, .. }) => {
            store.create_wiki(&name).await.map(CrudResult::Wiki)
        }
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
/// The request options are read through the ONE shared query decoder (§5.3).
async fn rag_query_handler(State(store): State<AppState>, body: String) -> Response {
    let env = match Envelope::from_json(&body) {
        Ok(env) => env,
        Err(e) => return decode_error_response(&e),
    };
    let (query, options) = match decode_query_request(QueryPath::Post, &env, SseParams::default()) {
        Ok(x) => x,
        Err(e) => return query_decode_error_response(e),
    };
    match store.rag_query(&query, &options).await {
        // The checked encoder never emits a body the result validator rejects
        // (§5.3); a rejected result renders as `EngineError` → 502 (FS-9).
        Ok(res) => match gnosis::encode_result_checked(&res) {
            Ok(env) => (StatusCode::OK, Json(env)).into_response(),
            Err(e) => {
                let (status, _) = server_status(&e).unwrap_or((500, ""));
                let env = gnosis::wire::codecs::encode_error(&e);
                (
                    StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_GATEWAY),
                    Json(env),
                )
                    .into_response()
            }
        },
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
    // The SSE surface reads exactly `query`/`topK`/`mode` (§5.4's N1); the token
    // is handed to the shared decoder as a param, never pre-resolved here.
    let sse_params = SseParams {
        query: params.get("query").map(String::as_str),
        top_k: params.get("topK").map(String::as_str),
        mode: params.get("mode").map(String::as_str),
    };
    let (query, options) = match decode_query_request(
        QueryPath::Sse,
        &Envelope::with_payload(serde_json::Value::Object(serde_json::Map::new())),
        sse_params,
    ) {
        Ok(x) => x,
        Err(e) => return sse_decode_error_response(e),
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
            let frame = encode_event(&gnosis::store::RagChunk::Error(e));
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
    // §6 boot sequence + §9.5.2 (U3) + §9.5.5 (U5): probe the provider, build the
    // `BootProvider`, then apply ONLY the `EngineState` `boot_wiring` returns,
    // plus the boot's own wiring — the snapshot swap and, only in the
    // `Reachable` case, `set_embedding_provider`. The returned flag vector is
    // the DERIVED projection (an assertion surface), so it is NOT applied: the
    // boot writes **no** flag mask, and `GET /engine/status` derives the honest
    // vector from what was actually wired (no fabricated READY, and no
    // `embedding:true` when no provider is wired).
    let boot_snapshot = DerivedIndexes::default();
    let (provider, wired): (BootProvider, Option<Arc<dyn EmbeddingProvider>>) =
        match OllamaProvider::from_env() {
            None => (BootProvider::Absent, None),
            Some(probe) => {
                if probe.is_available().await {
                    let provider: Arc<dyn EmbeddingProvider> = Arc::new(probe);
                    (BootProvider::Reachable(provider.clone()), Some(provider))
                } else {
                    (BootProvider::Unreachable, None)
                }
            }
        };
    // §9.5.5 (U5) — the boot index build. Built for a `Reachable` provider only
    // (an `Absent`/`Unreachable` boot passes no provider, so **no** `embed` call
    // is attempted and the index-free snapshot is kept unchanged), over the whole
    // store's corpus, and composed into the snapshot from the returned
    // `Option<VectorIndex>` alone — the rest of the boot snapshot is untouched:
    //   * `Ok(Some(vi))` ⇒ an index-bearing snapshot (an empty index included);
    //   * `Ok(None)`     ⇒ `vectors: None` (no provider: nothing to build);
    //   * `Err(_)`       ⇒ the failed build's `Err` is discarded with the whole
    //     local index; `built.ok().flatten()` leaves `vectors: None`.
    // The provider was probed successfully, so it IS wired even on the failure
    // branch — the one behavioural difference from the `Unreachable` branch.
    let built = build_boot_vector_index(&store, wired.as_ref()).await;
    let build_ok = built.is_ok();
    let snapshot = DerivedIndexes {
        vectors: built.ok().flatten(),
        ..boot_snapshot
    };
    // A failed `Reachable` build degrades through the EXISTING
    // `boot_wiring(BootProvider::Unreachable, &snapshot)`-shaped derived pair —
    // no new branch, no new state — while the `Reachable` branch keeps `Ready`.
    // The returned flag vector is discarded (the store derives its own at read
    // time), so the failure branch writes **no** mask either.
    let (state, _derived_flags) = match provider {
        BootProvider::Reachable(reachable) if build_ok => {
            boot_wiring(BootProvider::Reachable(reachable), &snapshot)
        }
        BootProvider::Reachable(_) => boot_wiring(BootProvider::Unreachable, &snapshot),
        BootProvider::Absent => boot_wiring(BootProvider::Absent, &snapshot),
        BootProvider::Unreachable => boot_wiring(BootProvider::Unreachable, &snapshot),
    };
    store.swap_snapshot(snapshot);
    if let Some(provider) = wired {
        store.set_embedding_provider(provider);
    }
    store.set_engine_state(state);
    let app = router(store);
    let addr = format!("127.0.0.1:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("bind loopback 127.0.0.1");
    axum::serve(listener, app).await.expect("serve");
}
