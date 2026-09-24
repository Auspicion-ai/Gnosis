//! §7.2 P2 **U2** — the `POST /rag/query` contract — BLIND-GREENS SET
//! (`tests/blind_u2_query_post_greens.rs`).
//!
//! Blind-test-writer derived from the DOCUMENTATION ONLY:
//! `docs/specs/p2-gnosis-server.md` (§5.2 routing, §5.3 the request/response
//! contract + the 14-key table + the two-layer absent/wrongly-typed reading +
//! the canonical §4.5.2 `filters` mapping, §5.4 the single case-insensitive
//! token rule + the POST/SSE split + the SSE 400 frame, §5.5 the structured
//! transport decode-error body, §10 the per-endpoint fail-states + precedence,
//! §9.5.1 the U2 property rows, §9.5.4 the coverage notes) and
//! `docs/specs/engine-wire-contract.md` (§4.4 SSE framing, §4.5 the `ragQuery`
//! wire, §7.1 the transport decode body, §12 V-15/V-15.1, §13 the per-fn
//! valid/fail states), with the P1a envelope precedent
//! (`docs/specs/p1a-document-crud-wire.md` §4/§6.2) for the envelope/transport
//! vocabulary only. **No `src/` file was read for expectations and no existing
//! test file was read for expectations** — only public signatures (for
//! compilation) and the docs.
//!
//! Surfaces: the live `gnosis-server` bin (`CARGO_BIN_EXE_gnosis-server`) for
//! the HTTP/SSE legs, and the engine lib's public API for the pure decode-level
//! legs. Every scenario is named `u2_<clause>_<claim>` and each assertion is
//! traced to a contract clause.
//!
//! **Two-layer reading (§5.3's "THE TWO-LAYER READING OF THE `absent ⇒` COLUMN"
//! bullet).** The decode-level legs assert layer 1 (the decoded
//! `RagQueryOptions.<field> == None`; a well-formed vocabulary string is the
//! only case that yields `Some(token)`), and the live legs assert the wire
//! outcome (layer 2's effective result). No scenario asserts a decoder-supplied
//! default value.

use std::net::TcpListener;
use std::process::{Child, Command};
use std::time::Duration;

use gnosis::wire::decode::DecodeError;
use gnosis::wire::envelope::Envelope;
use gnosis::wire::error::from_wire;
use gnosis::wire::query::{
    decode_query_request, encode_result_checked, request_decode_code, QueryDecodeError, QueryPath,
    SseParams,
};
use gnosis::{
    request_decode_status, server_status, CompressionMode, DocumentId, EdgeKind, ExpandMode,
    MultiQueryOptions, NodeId, NodeKind, QueryAuditFilters, QueryMode, RagQueryOptions, RagResult,
    RagResultItem, RagTrace, ReferenceState, Source, StoreError, TraceDescriptor,
};

const SERVER_BIN: &str = env!("CARGO_BIN_EXE_gnosis-server");
const CT_JSON: &str = "application/json";

// ---------------------------------------------------------------------------
// Envelope / payload helpers (docs: engine-wire-contract §4.1, §4.5; p2 §5.3).
// ---------------------------------------------------------------------------

/// The canonical request envelope (§4.1): `{"schemaVersion":1,
/// "idFormat":"opaque-string-v1","payload":…}`.
fn envelope(payload: serde_json::Value) -> Envelope {
    let mut env =
        Envelope::from_json(r#"{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{}}"#)
            .expect("the canonical envelope literal must parse");
    env.payload = payload;
    env
}

/// §5.3 — the shared decoder on the POST path with no SSE params.
fn decode_post(payload: serde_json::Value) -> Result<(String, RagQueryOptions), QueryDecodeError> {
    decode_query_request(QueryPath::Post, &envelope(payload), SseParams::default())
}

/// §5.3 — assert an `Ok` POST decode and hand back the options.
fn decode_ok(payload: serde_json::Value) -> (String, RagQueryOptions) {
    decode_post(payload.clone())
        .unwrap_or_else(|e| panic!("§5.3: {payload} must decode `Ok`, got {e:?}"))
}

fn validation_err(e: &QueryDecodeError) -> &StoreError {
    match e {
        QueryDecodeError::Validation(s) => s,
        QueryDecodeError::Transport(d) => {
            panic!("§5.3/§5.4: expected `Err(Validation(ValidationError(_)))`, got transport {d:?}")
        }
    }
}

// ---------------------------------------------------------------------------
// Live-server harness (docs: p2 §4 loopback bind, §6 READY lifecycle).
// ---------------------------------------------------------------------------

fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    listener.local_addr().expect("local addr").port()
}

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

/// Kill the live server child on drop (kill + wait ⇒ no zombie, no leaked
/// listener).
struct ServerGuard(Child);
impl Drop for ServerGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Spawn a fresh live server and hand back `(guard, base)` — the caller binds
/// the guard for the whole scenario, so the child is killed and reaped when the
/// scenario ends.
async fn live_server() -> (ServerGuard, String) {
    let (child, base) = spawn_server().await;
    (ServerGuard(child), base)
}
/// The engine's READY state, read from the single status surface (p2 §6/§10:
/// `GET /engine/status` is always 200).
async fn engine_is_ready(base: &str) -> bool {
    let resp = reqwest::get(format!("{base}/engine/status"))
        .await
        .expect("GET /engine/status");
    let body: serde_json::Value = resp.json().await.expect("status JSON");
    body.get("state").and_then(|v| v.as_str()) == Some("Ready")
}

async fn post_query(base: &str, body: serde_json::Value) -> (u16, String) {
    let resp = reqwest::Client::new()
        .post(format!("{base}/rag/query"))
        .json(&body)
        .send()
        .await
        .expect("POST /rag/query");
    let status = resp.status().as_u16();
    let text = resp.text().await.expect("body");
    (status, text)
}

async fn get_stream(base: &str, query: &str) -> (u16, String) {
    let resp = reqwest::Client::new()
        .get(format!("{base}/rag/stream?{query}"))
        .send()
        .await
        .expect("GET /rag/stream");
    let status = resp.status().as_u16();
    let text = resp.text().await.expect("body");
    (status, text)
}

/// The five transport codes of §5.5 — a client must never read one of these as
/// a §11/`StoreError` failure.
fn transport_codes() -> [&'static str; 5] {
    [
        "invalid_json",
        "invalid_envelope",
        "unsupported_schema_version",
        "unknown_id_format",
        "unknown_method",
    ]
}

/// §5.5's body shape: `{"code":…,"message":…}`, exactly two keys, **not**
/// envelope-wrapped (the envelope is what failed to decode).
fn assert_transport_decode_body(text: &str) -> String {
    let v: serde_json::Value = serde_json::from_str(text)
        .unwrap_or_else(|_| panic!("§5.5: the decode-error body must be JSON, got {text:?}"));
    let obj = v
        .as_object()
        .unwrap_or_else(|| panic!("§5.5: the decode-error body must be an object, got {text:?}"));
    let mut keys: Vec<&String> = obj.keys().collect();
    keys.sort();
    assert_eq!(
        keys,
        vec!["code", "message"],
        "§5.5: the body is the minimal non-envelope-wrapped `{{\"code\",\"message\"}}` pair, got {text:?}"
    );
    assert!(
        obj["code"].is_string(),
        "§5.5: `code` must be a string, got {text:?}"
    );
    assert!(
        obj["message"].is_string(),
        "§5.5: `message` must be a string (it MAY be empty), got {text:?}"
    );
    // §5.5's byte-pinned ordering of the minimal pair.
    let re_ok = format!(
        "{{\"code\":\"{}\",\"message\":\"",
        regex_escape(obj["code"].as_str().unwrap())
    );
    assert!(
        text.starts_with(&re_ok),
        "§5.5: the body must start `{{\"code\":\"<code>\",\"message\":\"`, got {text:?}"
    );
    obj["code"].as_str().unwrap().to_string()
}

fn regex_escape(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            '"' | '\\' => vec!['\\', c],
            _ => vec![c],
        })
        .collect()
}

/// Assert the §5.5 transport body for one live POST: exact status, exact
/// `Content-Type`, the expected code, and never a §11 code.
async fn assert_transport_body(
    base: &str,
    body: serde_json::Value,
    want_status: u16,
    want_code: &str,
    label: &str,
) {
    let resp = reqwest::Client::new()
        .post(format!("{base}/rag/query"))
        .json(&body)
        .send()
        .await
        .expect("POST /rag/query");
    let status = resp.status().as_u16();
    let ctype = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap_or("<non-utf8>").to_string())
        .unwrap_or_else(|| "<absent>".to_string());
    let text = resp.text().await.expect("body");
    assert_eq!(
        status, want_status,
        "{label}: §5.5 status, got body {text:?}"
    );
    assert_eq!(
        ctype, CT_JSON,
        "{label}: §5.5/F15 — `Content-Type` must be byte-exact `application/json` (no charset)"
    );
    let code = assert_transport_decode_body(&text);
    assert_eq!(code, want_code, "{label}: §5.5 transport code for {text:?}");
    assert!(
        transport_codes().contains(&code.as_str()),
        "{label}: the rendered code is transport-level (§5.5)"
    );
}

/// §5.3/§11 — the error envelope of a §11-mapped failure:
/// `{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"code","message"}}`.
fn assert_error_envelope(text: &str, want_code: &str, label: &str) -> serde_json::Value {
    let env = Envelope::from_json(text).unwrap_or_else(|e| {
        panic!("{label}: the response must be an envelope (§5.3), got {text:?} ({e:?})")
    });
    assert_eq!(env.schema_version, 1, "{label}: envelope schemaVersion");
    assert_eq!(
        env.id_format, "opaque-string-v1",
        "{label}: envelope idFormat"
    );
    let payload = env
        .payload
        .as_object()
        .unwrap_or_else(|| panic!("{label}: payload must be an object, got {text:?}"));
    let code = payload
        .get("code")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("{label}: the §11 error payload carries `code`, got {text:?}"));
    assert_eq!(code, want_code, "{label}: §11 wire code");
    assert!(
        payload.get("message").and_then(|v| v.as_str()).is_some(),
        "{label}: the §11 error payload carries `message`, got {text:?}"
    );
    assert!(
        !transport_codes().contains(&code),
        "{label}: an error-envelope code must be a §11 code, never a transport code (got {code})"
    );
    env.payload.clone()
}

/// §5.3 — a validation-class POST failure is 400 `validation_error`.
async fn assert_post_validation_400(base: &str, body: serde_json::Value, label: &str) {
    let (status, text) = post_query(base, body).await;
    assert_eq!(
        status, 400,
        "{label}: §5.3/§10 — expected 400, body {text:?}"
    );
    assert_error_envelope(&text, "validation_error", label);
}

/// The SSE frame splitter (§4.4: `event: <type>` / `data: <json>` / blank line).
fn sse_frames(body: &str) -> Vec<(String, String)> {
    let mut frames = Vec::new();
    for block in body.split("\n\n") {
        let block = block.trim_matches(['\r', '\n']);
        if block.is_empty() {
            continue;
        }
        let mut event = None;
        let mut data = None;
        for line in block.lines() {
            let line = line.trim_end_matches('\r');
            if let Some(rest) = line.strip_prefix("event:") {
                event = Some(rest.trim().to_string());
            } else if let Some(rest) = line.strip_prefix("data:") {
                data = Some(rest.trim().to_string());
            }
        }
        frames.push((
            event.unwrap_or_else(|| panic!("§4.4: frame without `event:` in {body:?}")),
            data.unwrap_or_else(|| panic!("§4.4: frame without `data:` in {body:?}")),
        ));
    }
    frames
}

// ===========================================================================
// (a) §5.3 — a well-formed envelope POST (`mode`/`topK`/`filters`) ⇒ 200 with
// the bare `RagResult` payload (never the SSE `{"type":"result",…}` wrapper).
// ===========================================================================

/// §5.3/§4.5 — the success body shape: the §4.1 envelope whose payload is the
/// bare serde `RagResult` body (`query`/`results`/`engine`/`citations`/`trace`/
/// `blocked_by`, snake_case, no `type` discriminator).
fn minimal_rag_result() -> RagResult {
    RagResult {
        query: "q".to_string(),
        results: vec![RagResultItem {
            document_id: DocumentId("d1".to_string()),
            node_id: NodeId("n1".to_string()),
            score: 0.9,
            snippet: "s".to_string(),
            source: Source::Local,
            parent: None,
            stale: None,
        }],
        engine: "gnosis".to_string(),
        citations: vec![(DocumentId("d1".to_string()), NodeId("n1".to_string()))],
        trace: RagTrace::Flat(TraceDescriptor {
            mode: QueryMode::Flat,
            engine: "gnosis".to_string(),
            top_k: 10,
            source: Source::Local,
        }),
        blocked_by: None,
    }
}

#[test]
fn u2_post_success_payload_is_the_bare_rag_result() {
    // §5.3 "Response body — the bare `RagResult` (NOT the SSE chunk wrapper)";
    // engine-wire-contract §4.5 + §12 V-18: the response envelope's payload is
    // exactly the serde `RagResult` body; `{"type":"result","result":…}` is
    // SSE-only and "would produce a body the result validator rejects".
    let env = encode_result_checked(&minimal_rag_result()).expect("a well-formed result encodes");
    assert_eq!(env.schema_version, 1, "§4.1 envelope schemaVersion");
    assert_eq!(env.id_format, "opaque-string-v1", "§4.1 envelope idFormat");
    let payload = env.payload.as_object().expect("payload is an object");
    for key in [
        "query",
        "results",
        "engine",
        "citations",
        "trace",
        "blocked_by",
    ] {
        assert!(
            payload.contains_key(key),
            "§5.3: the bare `RagResult` payload carries `{key}` (got {:#?})",
            env.payload
        );
    }
    assert!(
        !payload.contains_key("type") && !payload.contains_key("result"),
        "§5.3/§4.5: the POST payload must NOT be the SSE `{{\"type\":\"result\",\"result\":…}}` wrapper, got {:#?}",
        env.payload
    );
    assert_eq!(
        payload["engine"], "gnosis",
        "§4.2/§5.3: the frozen `RagResult` body serializes `engine` verbatim"
    );
    assert!(
        payload["trace"].is_object(),
        "§4.2: `trace` is externally tagged (`{{\"Flat\":{{…}}}}`), got {:#?}",
        payload["trace"]
    );
    assert!(
        payload["blocked_by"].is_null(),
        "§4.2: absent `blocked_by` is present-and-`null`, got {:#?}",
        payload["blocked_by"]
    );
}

#[tokio::test]
async fn u2_post_well_formed_envelope_is_accepted_by_the_wire() {
    // §5.3 — a well-formed envelope with `mode`/`topK`/`filters` is *accepted*:
    // the outcome is either the 200 bare-`RagResult` contract or the engine's
    // READY gate (503), never a transport/validation 400. Precedence (§5.3):
    // transport decode → token resolver → `validate_rag_options` → READY gate.
    let (_guard, base) = live_server().await;
    let body = serde_json::json!({
        "schemaVersion": 1,
        "idFormat": "opaque-string-v1",
        "payload": {
            "query": "hello",
            "mode": "flat",
            "topK": 10,
            "filters": {"nodeKind": "fact"}
        }
    });
    let (status, text) = post_query(&base, body).await;
    // The observed outcome is recorded so the run report can state which half of
    // the §5.3 contract was exercised on this boot (the 200 half needs a READY
    // engine; `GET /engine/status` is the §6/§10 readiness surface).
    eprintln!(
        "U2-1 observed: POST /rag/query (well-formed, mode/topK/filters) ⇒ HTTP {status}, body {text}"
    );
    if status == 200 {
        let env = Envelope::from_json(&text).expect("§5.3: the success body is an envelope");
        let payload = env.payload.as_object().expect("payload object");
        assert!(
            payload.contains_key("query") && payload.contains_key("results"),
            "§5.3: the 200 payload is the bare `RagResult` body, got {text:?}"
        );
        assert!(
            !payload.contains_key("type"),
            "§5.3: the 200 payload is never the SSE wrapper, got {text:?}"
        );
    } else {
        assert_eq!(
            status, 503,
            "§5.3/§10: a well-formed request is either 200 or the READY gate (503), got {status} {text:?}"
        );
        assert_error_envelope(
            &text,
            "engine_unavailable",
            "well-formed POST on a not-READY engine",
        );
    }
}

// ===========================================================================
// (b) §5.4/§5.3 — an unrecognized `mode` ⇒ 400 `validation_error` on POST,
// including `""`, whitespace, `"bm25"` and padded members.
// ===========================================================================

#[tokio::test]
async fn u2_post_unrecognized_mode_is_400_validation_error() {
    // §5.4 (the `mode` table's last row + "never a silent default"),
    // §5.3's fail-state table, §9.5.4's `P-SM-4` boundary corpus.
    let (_guard, base) = live_server().await;
    for token in [
        "",
        " ",
        "\t",
        "bm25",
        "flat ",
        "flat\n",
        " hybrid",
        "flat\tflat",
        "FlatX",
        "vectors",
        "none",
        "parent",
    ] {
        let body = serde_json::json!({
            "schemaVersion": 1,
            "idFormat": "opaque-string-v1",
            "payload": {"query": "x", "mode": token}
        });
        assert_post_validation_400(&base, body, &format!("§5.4 unrecognized mode {token:?}")).await;
    }
}

#[tokio::test]
async fn u2_post_mode_token_error_precedes_the_range_and_ready_checks() {
    // §5.3's precedence: (2) the wire token resolver precedes (3)
    // `validate_rag_options` and (4) the READY gate, so an unrecognized `mode`
    // is 400 even alongside an out-of-range `topK` — and 400 (not 503) on a
    // not-READY engine.
    let (_guard, base) = live_server().await;
    let body = serde_json::json!({
        "schemaVersion": 1,
        "idFormat": "opaque-string-v1",
        "payload": {"query": "x", "mode": "bm25", "topK": 0}
    });
    assert_post_validation_400(
        &base,
        body,
        "§5.3 precedence: mode token before range/READY",
    )
    .await;
}

// ===========================================================================
// (c) §5.4 — every ASCII casing of a member token ⇒ non-400.
// ===========================================================================

#[tokio::test]
async fn u2_post_every_member_token_casing_is_non_400() {
    // §5.4 — `flat|graph|vector|hybrid` matched ASCII case-insensitively;
    // §9.5.1's `P-SM-4` wiring assertion (ii) is the SSE mirror of this claim.
    let (_guard, base) = live_server().await;
    for token in [
        "flat", "Flat", "FLAT", "fLaT", "graph", "Graph", "GRAPH", "gRaPh", "vector", "Vector",
        "VECTOR", "vEcToR", "hybrid", "Hybrid", "HYBRID", "hYbRiD",
    ] {
        let body = serde_json::json!({
            "schemaVersion": 1,
            "idFormat": "opaque-string-v1",
            "payload": {"query": "x", "mode": token}
        });
        let (status, text) = post_query(&base, body).await;
        assert_ne!(
            status, 400,
            "§5.4: member token {token:?} is recognized in any ASCII casing, got 400 {text:?}"
        );
        assert!(
            status == 200 || status == 503,
            "§5.4: a recognized mode is either served (200) or READY-gated (503), got {status} {text:?}"
        );
    }
}

#[tokio::test]
async fn u2_post_expand_and_compression_member_tokens_are_non_400() {
    // §5.3's "Value tokens (casing)" bullet: `expand` (`none|parent`) and
    // `compression` (`none|filter|extract|graph`) follow the same casing policy
    // on the POST payload; their unrecognized-token 400s are asserted below.
    let (_guard, base) = live_server().await;
    for payload in [
        serde_json::json!({"query": "x", "expand": "parent"}),
        serde_json::json!({"query": "x", "expand": "PARENT"}),
        serde_json::json!({"query": "x", "expand": "none"}),
        serde_json::json!({"query": "x", "expand": "None"}),
        serde_json::json!({"query": "x", "compression": "graph"}),
        serde_json::json!({"query": "x", "compression": "GRAPH"}),
        serde_json::json!({"query": "x", "compression": "filter"}),
        serde_json::json!({"query": "x", "compression": "extract"}),
        serde_json::json!({"query": "x", "compression": "ExTrAcT"}),
    ] {
        let body = serde_json::json!({
            "schemaVersion": 1,
            "idFormat": "opaque-string-v1",
            "payload": payload.clone()
        });
        let (status, text) = post_query(&base, body).await;
        assert_ne!(
            status, 400,
            "§5.3: a member expand/compression token is accepted, got 400 for {payload} — {text:?}"
        );
    }
}

#[test]
fn u2_post_unrecognized_expand_and_compression_tokens_are_validation_err() {
    // §5.3's "Value tokens" bullet: "an unrecognized token ⇒ 400
    // `validation_error`, never a silent default", for all three token families
    // on the POST payload (§5.4).
    for token in ["", " ", "bogus", "parent ", "parent\n"] {
        let e = decode_post(serde_json::json!({"query": "x", "expand": token}))
            .expect_err("§5.3: an unrecognized `expand` token must be `Err`");
        assert!(
            matches!(validation_err(&e), StoreError::ValidationError(_)),
            "§5.3/§5.4: expand {token:?} ⇒ Err(Validation(ValidationError(_))), got {e:?}"
        );
    }
    for token in ["", " ", "nope", "graph ", "graphs"] {
        let e = decode_post(serde_json::json!({"query": "x", "compression": token}))
            .expect_err("§5.3: an unrecognized `compression` token must be `Err`");
        assert!(
            matches!(validation_err(&e), StoreError::ValidationError(_)),
            "§5.3/§5.4: compression {token:?} ⇒ Err(Validation(ValidationError(_))), got {e:?}"
        );
    }
}

#[test]
fn u2_post_mode_resolver_is_total_and_single_sourced() {
    // §5.4 "one rule, both readings" + §9.5.1's `P-SM-4`: the POST decode and
    // the SSE decode classify the same `mode` input identically (same `Ok`
    // enum, same `Err` family).
    for token in ["flat", "FLAT", "Graph", "vEcToR", "HYBRID"] {
        let (_, post_opts) = decode_ok(serde_json::json!({"query": "x", "mode": token}));
        let sse = decode_query_request(
            QueryPath::Sse,
            &envelope(serde_json::json!({})),
            SseParams {
                query: Some("x"),
                top_k: None,
                mode: Some(token),
            },
        )
        .unwrap_or_else(|e| panic!("§5.4: SSE mode {token:?} must decode, got {e:?}"))
        .1;
        assert_eq!(
            post_opts.mode, sse.mode,
            "§5.4/§9.5.1 P-SM-4: POST and SSE resolve {token:?} identically"
        );
        assert!(
            post_opts.mode.is_some(),
            "§5.3 layer 1: a well-formed vocabulary string yields `Some(token)`"
        );
    }
    for token in ["", " ", "bm25", "flat "] {
        let post = decode_post(serde_json::json!({"query": "x", "mode": token}));
        let sse = decode_query_request(
            QueryPath::Sse,
            &envelope(serde_json::json!({})),
            SseParams {
                query: Some("x"),
                top_k: None,
                mode: Some(token),
            },
        );
        assert!(
            post.is_err() && sse.is_err(),
            "§5.4/§9.5.1 P-SM-4: {token:?} must be `Err` on BOTH paths (POST {post:?}, SSE {sse:?})"
        );
        assert!(
            matches!(post.unwrap_err(), QueryDecodeError::Validation(_))
                && matches!(sse.unwrap_err(), QueryDecodeError::Validation(_)),
            "§5.4: the token failure is a `Validation` arm — never a decode-level 422"
        );
    }
}

// ===========================================================================
// (d) §5.3/§5.5 — envelope strictness: a bare/non-envelope body, a wrong
// `schemaVersion`, a wrong `idFormat`, a non-object payload ⇒ the pinned
// transport codes/statuses with the JSON body and exact `Content-Type`.
// ===========================================================================

#[tokio::test]
async fn u2_post_transport_decode_bodies_are_exact() {
    // §5.5's code table + the two worked examples; §12 V-15/V-15.1 bind the
    // status, the header and the `code`+`message` keyset.
    let (_guard, base) = live_server().await;

    // A wrong `schemaVersion` (checked FIRST) ⇒ unsupported_schema_version/400.
    for v in [0u32, 2, 99, u32::MAX] {
        let body = serde_json::json!({
            "schemaVersion": v,
            "idFormat": "opaque-string-v1",
            "payload": {"query": "x"}
        });
        assert_transport_body(
            &base,
            body,
            400,
            "unsupported_schema_version",
            &format!("§5.5 schemaVersion {v}"),
        )
        .await;
    }
    // §5.5's pinned message for `UnsupportedSchemaVersion(99)`.
    {
        let body = serde_json::json!({
            "schemaVersion": 99, "idFormat": "opaque-string-v1", "payload": {"query": "x"}
        });
        let (status, text) = post_query(&base, body).await;
        assert_eq!(status, 400);
        assert!(
            text.contains("\"message\":\"unsupported schemaVersion: 99\""),
            "§5.5: `UnsupportedSchemaVersion(v)` ⇒ \"unsupported schemaVersion: {{v}}\", got {text:?}"
        );
    }

    // A wrong `idFormat` ⇒ unknown_id_format/400.
    for f in ["uuid-v4", "", "OPAQUE-STRING-V1", "opaque-string-v2"] {
        let body = serde_json::json!({
            "schemaVersion": 1,
            "idFormat": f,
            "payload": {"query": "x"}
        });
        assert_transport_body(
            &base,
            body,
            400,
            "unknown_id_format",
            &format!("§5.5 idFormat {f:?}"),
        )
        .await;
    }

    // `schemaVersion` precedes `idFormat`: both unknown ⇒ version first.
    assert_transport_body(
        &base,
        serde_json::json!({"schemaVersion": 7, "idFormat": "uuid-v4", "payload": {"query": "x"}}),
        400,
        "unsupported_schema_version",
        "§5.3/§5.5 precedence: schemaVersion before idFormat",
    )
    .await;

    // A non-object `payload` ⇒ invalid_envelope/400 (V-15.1: `InvalidJson` is
    // unreachable on this path).
    for payload in [
        serde_json::json!(5),
        serde_json::Value::Null,
        serde_json::json!("hi"),
        serde_json::json!([]),
        serde_json::json!(true),
    ] {
        let body = serde_json::json!({
            "schemaVersion": 1, "idFormat": "opaque-string-v1", "payload": payload
        });
        assert_transport_body(
            &base,
            body,
            400,
            "invalid_envelope",
            &format!("§5.3/V-15.1 non-object payload {payload}"),
        )
        .await;
    }

    // A bare (non-envelope) body: valid JSON, no envelope shape ⇒ transport
    // 400 (§4.5: "a bare (non-envelope) body is **rejected**").
    assert_transport_body(
        &base,
        serde_json::json!({"query": "x"}),
        400,
        "invalid_envelope",
        "§5.3 bare non-envelope body",
    )
    .await;

    // Unparseable JSON ⇒ 400 with a transport code whose `message` is the
    // carried string (§5.5: TestWriters MUST NOT assert serde's own text).
    {
        let resp = reqwest::Client::new()
            .post(format!("{base}/rag/query"))
            .header(reqwest::header::CONTENT_TYPE, CT_JSON)
            .body("{\"schemaVersion\":1,")
            .send()
            .await
            .expect("POST /rag/query");
        let status = resp.status().as_u16();
        let ctype = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .map(|v| v.to_str().unwrap().to_string())
            .unwrap_or_default();
        let text = resp.text().await.expect("body");
        assert_eq!(
            status, 400,
            "§5.3/§5.5: malformed JSON ⇒ transport 400, got {text:?}"
        );
        assert_eq!(ctype, CT_JSON, "§5.5/F15: byte-exact `application/json`");
        let code = assert_transport_decode_body(&text);
        assert_eq!(
            code, "invalid_json",
            "§5.5: unparseable JSON ⇒ invalid_json"
        );
    }

    // `unknown_method` (422) is CRUD-only and unreachable on this surface: a
    // `"method"` payload key is just a tolerated extra key (§5.3).
    let (status, text) = post_query(
        &base,
        serde_json::json!({
            "schemaVersion": 1, "idFormat": "opaque-string-v1",
            "payload": {"query": "x", "method": "bogus"}
        }),
    )
    .await;
    assert_ne!(
        status, 422,
        "§5.3: `unknown_method`/422 is unreachable on the query surface, got {text:?}"
    );
    assert_ne!(
        status, 400,
        "§5.3: a tolerated `method` key is not a 400, got {text:?}"
    );
}

#[test]
fn u2_post_decode_level_transport_outcomes_are_pinned() {
    // The pure mirror of the live transport legs: §5.3's envelope precedence
    // and §9.5.1's `P-IM-4` disjoint outcome sets (transport `Err` vs `Ok`).
    let base_env = envelope(serde_json::json!({"query": "x"}));

    for v in [0u32, 2, 99, u32::MAX] {
        let mut env = base_env.clone();
        env.schema_version = v;
        match decode_query_request(QueryPath::Post, &env, SseParams::default()) {
            Err(QueryDecodeError::Transport(DecodeError::UnsupportedSchemaVersion(got))) => {
                assert_eq!(
                    got, v,
                    "§5.5: `UnsupportedSchemaVersion({{v}})` carries the value"
                )
            }
            other => panic!(
                "§5.3: schemaVersion {v} ⇒ Transport(UnsupportedSchemaVersion), got {other:?}"
            ),
        }
        assert_eq!(
            request_decode_status(&DecodeError::UnsupportedSchemaVersion(v)),
            Some(400),
            "§5.5: the transport status is 400 (never 502)"
        );
    }

    for f in ["uuid-v4", "", "OPAQUE-STRING-V1"] {
        let mut env = base_env.clone();
        env.id_format = f.to_string();
        assert!(
            matches!(
                decode_query_request(QueryPath::Post, &env, SseParams::default()),
                Err(QueryDecodeError::Transport(DecodeError::UnknownIdFormat(_)))
            ),
            "§5.3: idFormat {f:?} ⇒ Transport(UnknownIdFormat)"
        );
    }

    // Precedence: both unknown ⇒ the version error.
    let mut env = base_env.clone();
    env.schema_version = 7;
    env.id_format = "uuid-v4".to_string();
    assert!(
        matches!(
            decode_query_request(QueryPath::Post, &env, SseParams::default()),
            Err(QueryDecodeError::Transport(
                DecodeError::UnsupportedSchemaVersion(7)
            ))
        ),
        "§5.3/§9.5.1 P-IM-4: schemaVersion is checked before idFormat"
    );

    // Non-object payloads ⇒ exactly one transport code (`InvalidEnvelope`);
    // `InvalidJson` is unreachable here (V-15.1).
    for payload in [
        serde_json::json!(5),
        serde_json::Value::Null,
        serde_json::json!("hi"),
        serde_json::json!([]),
        serde_json::json!(true),
    ] {
        let env = envelope(payload.clone());
        assert!(
            matches!(
                decode_query_request(QueryPath::Post, &env, SseParams::default()),
                Err(QueryDecodeError::Transport(DecodeError::InvalidEnvelope(_)))
            ),
            "§5.3/V-15.1: non-object payload {payload} ⇒ Transport(InvalidEnvelope(_))"
        );
    }

    // Every request-decode transport outcome has a status (never `None`).
    for e in [
        DecodeError::InvalidJson("x".to_string()),
        DecodeError::InvalidEnvelope("x".to_string()),
        DecodeError::UnsupportedSchemaVersion(1),
        DecodeError::UnknownIdFormat("x".to_string()),
        DecodeError::UnknownMethod("x".to_string()),
    ] {
        assert_eq!(
            request_decode_status(&e),
            Some(match e {
                DecodeError::UnknownMethod(_) => 422,
                _ => 400,
            }),
            "§5.5/§9.5.1 P-TP-3: the transport status is defined on the 5-variant domain"
        );
        assert!(
            request_decode_code(&e).is_some(),
            "§5.5/§9.5.1 P-TP-3: the transport code is defined on the same domain"
        );
    }
}

// ===========================================================================
// (e) §5.4 — the SSE route: `?mode=bm25` ⇒ HTTP 400 + exactly one
// `event: error` frame carrying `validation_error`; `?mode=HYBRID` ⇒ non-400;
// `?expand=`/`?compression=` ⇒ non-400 (those params are not read on SSE).
// ===========================================================================

#[tokio::test]
async fn u2_sse_unrecognized_mode_is_400_with_one_error_frame() {
    // §5.4's SSE fail-state rendering (F4: "Both halves are pinned (status
    // *and* body)"), §10's `GET /rag/stream` row, §9.5.1's `P-SM-4` wiring
    // assertion (i), F2 §4.4's single-event framing.
    let (_guard, base) = live_server().await;
    for token in ["bm25", "", "%20"] {
        let (status, body) = get_stream(&base, &format!("query=x&mode={token}")).await;
        assert_eq!(
            status, 400,
            "§5.4 F4: an unrecognized `mode` on SSE is HTTP 400 (never a 200 with an error frame), got body {body:?}"
        );
        let frames = sse_frames(&body);
        assert_eq!(
            frames.len(),
            1,
            "§5.4: exactly one SSE frame as the response body, got {frames:?} (raw {body:?})"
        );
        let (event, data) = &frames[0];
        assert_eq!(
            event, "error",
            "§4.4/§5.4: the single frame is `event: error`"
        );
        let json: serde_json::Value = serde_json::from_str(data)
            .unwrap_or_else(|_| panic!("§4.2: the `data:` line must be JSON, got {data:?}"));
        assert_eq!(
            json["type"], "error",
            "§4.4: `event:` must equal the data `type` field, got {data:?}"
        );
        assert_eq!(
            json["code"], "validation_error",
            "§5.4: the error frame carries the §11 code `validation_error`, got {data:?}"
        );
        assert!(
            json.get("message").map(|m| m.is_string()).unwrap_or(false),
            "§4.2: the error frame carries a `message`, got {data:?}"
        );
    }
}

#[tokio::test]
async fn u2_sse_member_token_casings_are_non_400() {
    // §5.4 — `?mode=HYBRID` (the `P-SM-4` wiring assertion (ii)) proves the
    // handler routed the param through the shared resolver rather than a local
    // case-sensitive `match`.
    let (_guard, base) = live_server().await;
    for token in ["HYBRID", "Hybrid", "hYbRiD", "flat", "Graph", "VECTOR"] {
        let (status, body) = get_stream(&base, &format!("query=x&mode={token}")).await;
        assert_ne!(
            status, 400,
            "§5.4: `?mode={token}` is a recognized token in any ASCII casing, got 400 {body:?}"
        );
    }
}

#[tokio::test]
async fn u2_sse_expand_and_compression_params_are_not_read() {
    // §5.4 N1 + §9.5.1's `P-SM-4` wiring assertion (iii) + §10's `GET
    // /rag/stream` row: the pre-U4 SSE surface reads exactly `query`/`topK`/
    // `mode`; `?expand=`/`?compression=` are ignored ⇒ never a 400 and never an
    // `error` frame.
    let (_guard, base) = live_server().await;
    for params in [
        "query=x&expand=parent&compression=nope",
        "query=x&expand=bogus",
        "query=x&compression=bogus",
        "query=x&expand=&compression=",
        "query=x&wikiId=w1&maxHops=2&hyde=true",
    ] {
        let (status, body) = get_stream(&base, params).await;
        assert_ne!(
            status, 400,
            "§5.4 N1: an unread SSE param is ignored ⇒ never a 400, got {status} for `?{params}` — {body:?}"
        );
        assert!(
            !body.contains("\"code\":\"validation_error\""),
            "§5.4 N1: no `validation_error` frame for an unread param, got {body:?}"
        );
    }
}

// ===========================================================================
// (f) §5.3 — envelope strictness does not break the documented tolerance:
// unknown payload keys (incl. `args`/`method`/`requester`), absent
// `mode`/`topK` ⇒ the documented defaults.
// ===========================================================================

#[tokio::test]
async fn u2_post_unknown_keys_and_absent_options_are_tolerated() {
    // §5.3's unknown-key tolerance ("load-bearing": the existing e2e POSTs
    // carry an extra `args` key) and §5.4's "Absent `mode` ⇒ `Flat` ... they
    // still reach the READY gate and still return 503 while the engine is not
    // READY".
    let (_guard, base) = live_server().await;
    let ready = engine_is_ready(&base).await;
    for payload in [
        serde_json::json!({"query": "x", "args": {}}),
        serde_json::json!({"query": "x", "args": {"anything": 1}, "method": "bogus"}),
        serde_json::json!({"query": "x", "requester": "user:alice"}),
        serde_json::json!({"query": "x", "nodeKind": "content", "extra": [1, 2]}),
        serde_json::json!({"query": "x"}),
    ] {
        // The documented defaults are absent ⇒ `Flat` / `topK = 10`
        // (layer 2) — i.e. no `mode` and no `topK` key at all.
        let body = serde_json::json!({
            "schemaVersion": 1, "idFormat": "opaque-string-v1", "payload": payload
        });
        let (status, text) = post_query(&base, body).await;
        assert_ne!(
            status, 400,
            "§5.3: unknown keys and absent options are tolerated, got 400 for {payload} — {text:?}"
        );
        assert_ne!(
            status, 422,
            "§5.3: no 422 on the query surface, got {text:?}"
        );
        if ready {
            assert_eq!(
                status, 200,
                "§5.3: on a READY engine the request is served, got {text:?}"
            );
        } else {
            assert_eq!(
                status, 503,
                "§5.4: absent `mode` ⇒ `Flat` still reaches the READY gate ⇒ 503, got {status} {text:?}"
            );
            assert_error_envelope(
                &text,
                "engine_unavailable",
                "absent mode/topK on a not-READY engine",
            );
        }
    }
}

#[test]
fn u2_post_wrongly_typed_values_decode_as_absent_layer1() {
    // §5.3's per-key wrongly-typed table + the two-layer bullet: for every
    // optional key except `query`/`filters`, a present-with-wrong-type value
    // decodes to the store `Option`'s `None` (layer 1) and is NOT an `Err`.
    // `query` must stay well-formed and every ranged option in range, so this
    // scenario stays inside `P-IM-5`'s bounded domain.
    let (_, opts) = decode_ok(serde_json::json!({
        "query": "x",
        "mode": 5,
        "topK": "10",
        "wikiId": 5,
        "maxHops": null,
        "expand": 5,
        "maxParentContext": "5",
        "multiQuery": 5,
        "compression": null,
        "hyde": "yes",
        "binaryFirstPass": 1,
        "binaryCandidatePool": "10",
        "subTaskDag": 5
    }));
    assert_eq!(
        opts.mode, None,
        "§5.3: `mode:5` ⇒ `mode == None` (never `Some(Flat)`)"
    );
    assert_eq!(
        opts.top_k, None,
        "§5.3: `topK:\"10\"` ⇒ `top_k == None` (never `Some(10)`)"
    );
    assert_eq!(opts.wiki_id, None, "§5.3: `wikiId:5` ⇒ `wiki_id == None`");
    assert_eq!(
        opts.max_hops, None,
        "§5.3: `maxHops:null` ⇒ `max_hops == None`"
    );
    assert_eq!(opts.expand, None, "§5.3: `expand:5` ⇒ `expand == None`");
    assert_eq!(
        opts.max_parent_context, None,
        "§5.3: `maxParentContext:\"5\"` ⇒ `None`"
    );
    assert_eq!(
        opts.multi_query, None,
        "§5.3: `multiQuery:5` (non-object) ⇒ `None`"
    );
    assert_eq!(
        opts.compression, None,
        "§5.3: `compression:null` ⇒ `compression == None`"
    );
    assert_eq!(opts.hyde, None, "§5.3: `hyde:\"yes\"` ⇒ `hyde == None`");
    assert_eq!(
        opts.binary_first_pass, None,
        "§5.3: `binaryFirstPass:1` ⇒ `None`"
    );
    assert_eq!(
        opts.binary_candidate_pool, None,
        "§5.3: `binaryCandidatePool:\"10\"` ⇒ `None`"
    );
    assert_eq!(opts.sub_task_dag, None, "§5.3: `subTaskDag:5` ⇒ `None`");
    assert_eq!(
        opts.requester, None,
        "§5.3/§9.5.1 P-TP-2: `requester` is never part of the options"
    );

    // The object-member misuse corpus (§9.5.4's `P-IM-5` note) is the same
    // absent ⇒ default reading; its layer-1 field identity is asserted by the
    // dedicated scenario below (`u2_post_object_member_misuse_is_absent_layer1`).

    // `multiQuery.n`'s pinned default: `{enabled:true}` with `n` *omitted* ⇒ 3
    // (the member-level default applies only *inside a well-formed object*).
    let (_, opts) = decode_ok(serde_json::json!({"query": "x", "multiQuery": {"enabled": true}}));
    assert_eq!(
        opts.multi_query,
        Some(MultiQueryOptions {
            enabled: true,
            n: 3
        }),
        "§5.3: `{{multiQuery:{{enabled:true}}}}` ⇒ `n = 3`"
    );
    // CORRECTED (2026-09-17, post-clarification; credited to this set's own
    // `U2-13` finding): a *present* documented member with the wrong type is a
    // wrongly-typed documented member, so the WHOLE option is absent — never a
    // `Some` carrying a member-level default. §5.3's object-valued-option clause
    // + its `multiQuery` cell: "a present non-u64 `n` is a wrongly-typed
    // documented member, so `{"enabled":true,"n":"3"}` ⇒ `None`". This assertion
    // previously took the pre-clarification member-level reading (the `unwrap_or`
    // reading `U2-13` disproved) and expected `Some({enabled:true, n:3})`; it is
    // an alignment to a *clarified* rule, not a greens failure smoothed over.
    let (_, opts) = decode_ok(serde_json::json!({
        "query": "x", "multiQuery": {"enabled": true, "n": "3"}
    }));
    assert_eq!(
        opts.multi_query, None,
        "§5.3: `enabled:true` with a wrongly-typed *documented* `n` ⇒ the whole option is absent (`None`)"
    );

    // Well-typed controls ⇒ `Some(mapped)` (the only `Some` producer is a
    // well-formed value).
    let (q, opts) = decode_ok(serde_json::json!({
        "query": "verbatim",
        "mode": "Flat",
        "topK": 42,
        "wikiId": "w1",
        "expand": "Parent",
        "compression": "Filter",
        "hyde": true,
        "binaryFirstPass": true,
        "subTaskDag": {"enabled": true}
    }));
    assert_eq!(
        q, "verbatim",
        "§5.3/§9.5.1 P-TP-2: `query` is copied verbatim"
    );
    assert_eq!(
        opts.mode,
        Some(QueryMode::Flat),
        "§5.3: a member string ⇒ `Some(token)`"
    );
    assert_eq!(
        opts.top_k,
        Some(42),
        "§5.3: `topK:42` ⇒ `top_k == Some(42)`"
    );
    assert_eq!(
        opts.wiki_id,
        Some(gnosis::WikiId("w1".to_string())),
        "§5.3: `wikiId`"
    );
    assert_eq!(opts.expand, Some(ExpandMode::Parent), "§5.3: `expand`");
    assert_eq!(
        opts.compression,
        Some(CompressionMode::Filter),
        "§5.3: `compression`"
    );
    assert_eq!(opts.hyde, Some(true), "§5.3: `hyde`");
    assert_eq!(
        opts.binary_first_pass,
        Some(true),
        "§5.3: `binaryFirstPass`"
    );
    assert!(
        opts.sub_task_dag.is_some(),
        "§5.3: `subTaskDag` decodes (its *consumption* is out of this contract's scope — §5.3 reachability)"
    );

    // An absent `query` is the documented layer-1 exception: `""` (the engine's
    // FS-3 400 is layer 2), while a present non-string `query` is a
    // decoder-level `Err(Validation)`.
    let (q, _) = decode_ok(serde_json::json!({}));
    assert_eq!(
        q, "",
        "§5.3: an absent `query` decodes to `\"\"`, not an `Err`"
    );
    for bad in [
        serde_json::json!(5),
        serde_json::Value::Null,
        serde_json::json!({}),
        serde_json::json!([]),
    ] {
        let e = decode_post(serde_json::json!({"query": bad}))
            .expect_err("§5.3: a present non-string `query` is a decoder-level Err");
        assert!(
            matches!(validation_err(&e), StoreError::ValidationError(_)),
            "§5.3: non-string query {bad} ⇒ Err(Validation(ValidationError(_))), got {e:?}"
        );
    }
}

#[test]
fn u2_post_object_member_misuse_is_absent_layer1() {
    // §5.3's per-key wrongly-typed table (`multiQuery` / `subTaskDag` cells) and
    // §9.5.4's `P-IM-5` note: "an **object** with a non-boolean `enabled` ⇒
    // absent ⇒ disabled", read at layer 1 (the documented default is the
    // engine's query-time reading, and "the decoder NEVER substitutes a typed
    // default value … and writes none into the options").
    let (_, opts) = decode_ok(serde_json::json!({
        "query": "x",
        "multiQuery": {"enabled": "yes"},
        "subTaskDag": {"enabled": "yes"}
    }));
    assert_eq!(
        opts.sub_task_dag, None,
        "§5.3: a non-boolean `enabled` ⇒ disabled ⇒ `sub_task_dag == None` (layer 1)"
    );
    assert_eq!(
        opts.multi_query, None,
        "§5.3: a non-boolean `enabled` ⇒ that member absent ⇒ `multi_query == None` (layer 1)"
    );
}

#[tokio::test]
async fn u2_post_object_member_misuse_outcome_is_non_400_on_the_wire() {
    // The layer-2 / wire half of the same corpus (§5.3: a wrongly-typed value is
    // "not a decode failure and not a 400"): whatever the decoded `Option` is,
    // the wire outcome is the option's documented default, never a 400.
    let (_guard, base) = live_server().await;
    let ready = engine_is_ready(&base).await;
    for payload in [
        serde_json::json!({"query": "x", "multiQuery": {"enabled": "yes"}}),
        serde_json::json!({"query": "x", "subTaskDag": {"enabled": "yes"}}),
        serde_json::json!({"query": "x", "multiQuery": {"enabled": true, "n": "3"}}),
        serde_json::json!({"query": "x", "multiQuery": {"enabled": false, "n": "0"}}),
    ] {
        let body = serde_json::json!({
            "schemaVersion": 1, "idFormat": "opaque-string-v1", "payload": payload.clone()
        });
        let (status, text) = post_query(&base, body).await;
        assert_ne!(
            status, 400,
            "§5.3: object-member misuse {payload} is absent ⇒ default, never a 400 — got {text:?}"
        );
        assert_eq!(
            status,
            if ready { 200 } else { 503 },
            "§5.3/§5.4: the effective outcome is the documented default ({payload}), got {text:?}"
        );
    }
}

#[test]
fn u2_post_unknown_keys_are_decoded_away_but_do_not_change_options() {
    // §5.3/§9.5.1 `P-TP-2`/`P-IM-4`: extra keys change nothing, `requester` is
    // never set, and a well-formed payload decodes identically with or without
    // them.
    let plain = serde_json::json!({"query": "q", "mode": "hybrid", "topK": 10});
    let with_extras = serde_json::json!({
        "query": "q", "mode": "hybrid", "topK": 10,
        "args": {"a": 1}, "method": "bogus", "requester": "user:alice"
    });
    let (q1, o1) = decode_ok(plain);
    let (q2, o2) = decode_ok(with_extras);
    assert_eq!(q1, q2, "§5.3: extra keys do not change `query`");
    assert_eq!(o1, o2, "§5.3/§9.5.1 P-IM-4: extra keys change no option");
    assert_eq!(o2.requester, None, "§5.3: `requester` stays `None`");
}

// ===========================================================================
// (g) §5.3 — a `filters` payload in the canonical camelCase shape ⇒ the filter
// is applied (asserted where the contract makes it observable), while the
// store's internal serde shape / shape-mismatched members ⇒ `Err`.
// ===========================================================================

#[test]
fn u2_filters_canonical_mapping_is_member_by_member_and_never_all_none() {
    // §5.3's canonical §4.5.2 mapping table + N5; §9.5.1's `P-IM-6`.
    let (_, opts) = decode_ok(serde_json::json!({
        "query": "x",
        "filters": {
            "nodeKind": "fact",
            "edgeType": "embed",
            "target": {"documentId": "d1", "nodeId": "n1"},
            "state": "FRESH"
        }
    }));
    assert_eq!(
        opts.filters,
        Some(QueryAuditFilters {
            node_kind: Some(NodeKind::Fact),
            edge_type: Some(EdgeKind::Embed),
            target: Some((DocumentId("d1".to_string()), NodeId("n1".to_string()))),
            state: Some(ReferenceState::Fresh),
        }),
        "§5.3: the canonical camelCase `filters` maps member-by-member (never a serde pass-through ⇒ never all-`None`)"
    );

    // A canonical subset maps `Some`, not all-`None`.
    let (_, opts) = decode_ok(serde_json::json!({"query": "x", "filters": {"nodeKind": "fact"}}));
    assert_eq!(
        opts.filters,
        Some(QueryAuditFilters {
            node_kind: Some(NodeKind::Fact),
            edge_type: None,
            target: None,
            state: None,
        }),
        "§5.3: a canonical subset ⇒ `Some(filters)` with exactly the named members mapped"
    );

    // `state` accepts the store's own `"Fresh"`-style casing as a casing variant
    // of the canonical uppercase token; `nodeKind` is ASCII-case-insensitive.
    for (state, node_kind) in [
        ("FRESH", "content"),
        ("Fresh", "Content"),
        ("fresh", "CONTENT"),
        ("RESOLVED", "reference"),
        ("stale", "fact"),
        ("BROKEN", "fact"),
    ] {
        let (_, opts) = decode_ok(serde_json::json!({
            "query": "x", "filters": {"state": state, "nodeKind": node_kind}
        }));
        assert!(
            opts.filters.is_some(),
            "§5.3: `state: {state:?}` / `nodeKind: {node_kind:?}` are accepted casing variants"
        );
    }

    // `{}` ⇒ `Some(all-None)`: present, valid, honored (the caller's identity
    // filter); `null` ⇒ absent.
    let (_, opts) = decode_ok(serde_json::json!({"query": "x", "filters": {}}));
    assert_eq!(
        opts.filters,
        Some(QueryAuditFilters {
            node_kind: None,
            edge_type: None,
            target: None,
            state: None,
        }),
        "§5.3 N5: `filters: {{}}` ⇒ `Some(all-None)` (present/valid/honored)"
    );
    let (_, opts) = decode_ok(serde_json::json!({"query": "x", "filters": null}));
    assert_eq!(
        opts.filters, None,
        "§5.3: `filters: null` ⇒ absent (no filter)"
    );

    // A `target` with an extra member ⇒ `Ok` with the two named members mapped
    // and the extra one ignored; an unknown member NAME with a valid value ⇒
    // tolerated (contributes no option).
    let (_, opts) = decode_ok(serde_json::json!({
        "query": "x",
        "filters": {"target": {"documentId": "d1", "nodeId": "n1", "x": 1}}
    }));
    assert_eq!(
        opts.filters.as_ref().and_then(|f| f.target.clone()),
        Some((DocumentId("d1".to_string()), NodeId("n1".to_string()))),
        "§5.3/§9.5.1 P-IM-6: a `target` extra member is ignored (the tuple has no third slot)"
    );
    let (_, opts) =
        decode_ok(serde_json::json!({"query": "x", "filters": {"nodekind": "content"}}));
    assert_eq!(
        opts.filters,
        Some(QueryAuditFilters {
            node_kind: None,
            edge_type: None,
            target: None,
            state: None,
        }),
        "§5.3/§9.5.4 P-IM-6: a genuinely unknown member NAME with a valid value ⇒ `Ok(Some(all-None))`"
    );
}

#[test]
fn u2_filters_malformed_and_store_serde_shape_are_validation_err() {
    // §5.3's `filters` exception bullet + §9.5.4's `P-IM-6` coverage note: a
    // non-object `filters`, a wrongly-typed member, an out-of-set token, or a
    // `target` not carrying both string members ⇒ `Err(Validation)`; the
    // store's own serde shape (snake_case names / the 2-element `target`
    // array) is a SHAPE MISMATCH ⇒ `Err(Validation)`, never `Some(all-None)`.
    for filters in [
        serde_json::json!([]),
        serde_json::json!("x"),
        serde_json::json!(5),
        serde_json::json!({"nodeKind": 5}),
        serde_json::json!({"state": 5}),
        serde_json::json!({"edgeType": []}),
        serde_json::json!({"target": "d1"}),
        serde_json::json!({"target": ["d1", "n1"]}),
        serde_json::json!({"target": {"documentId": "d1"}}),
        serde_json::json!({"target": {"nodeId": "n1"}}),
        serde_json::json!({"target": {"documentId": 1, "nodeId": "n1"}}),
        serde_json::json!({"nodeKind": "community"}),
        serde_json::json!({"edgeType": "docHead"}),
        serde_json::json!({"edgeType": "member"}),
        serde_json::json!({"state": "frozen"}),
        serde_json::json!({"node_kind": "Content", "target": ["d1", "n1"]}),
    ] {
        match decode_post(serde_json::json!({"query": "x", "filters": filters.clone()})) {
            Err(e) => assert!(
                matches!(validation_err(&e), StoreError::ValidationError(_)),
                "§5.3/§9.5.4 P-IM-6: malformed filters {filters} ⇒ Err(Validation(ValidationError(_))), got {e:?}"
            ),
            Ok((_, opts)) => panic!(
                "§5.3/§9.5.4 P-IM-6: malformed filters {filters} must be `Err(Validation)`, got Ok with filters={:?}",
                opts.filters
            ),
        }
    }
}

// ===========================================================================
// (h) §5.3 — an explicitly out-of-range option value ⇒ `validation_error`.
// ===========================================================================

#[tokio::test]
async fn u2_post_out_of_range_options_are_400_validation_error() {
    // §5.3's fail-state table: `topK` not in 1..=50, `maxHops` not in 1..=5,
    // `multiQuery.enabled` with `n == 0`, `binaryCandidatePool: 0` with
    // `binaryFirstPass: true`, and a malformed `filters` ⇒ 400
    // `validation_error` (FS-3) — the engine's own `validate_rag_options`.
    let (_guard, base) = live_server().await;
    for payload in [
        serde_json::json!({"query": "x", "topK": 0}),
        serde_json::json!({"query": "x", "topK": 51}),
        serde_json::json!({"query": "x", "maxHops": 0}),
        serde_json::json!({"query": "x", "maxHops": 6}),
        serde_json::json!({"query": "x", "multiQuery": {"enabled": true, "n": 0}}),
        serde_json::json!({"query": "x", "binaryFirstPass": true, "binaryCandidatePool": 0}),
        serde_json::json!({"query": "x", "filters": {"nodeKind": "community"}}),
        serde_json::json!({"query": "x", "filters": "nope"}),
    ] {
        let body = serde_json::json!({
            "schemaVersion": 1, "idFormat": "opaque-string-v1", "payload": payload.clone()
        });
        assert_post_validation_400(&base, body, &format!("§5.3 out-of-range {payload}")).await;
    }
}

#[tokio::test]
async fn u2_post_empty_query_is_400_validation_error() {
    // §5.3: an absent `query` decodes to `""` ⇒ FS-3
    // `ValidationError("query must be non-empty")` ⇒ 400 `validation_error`;
    // the present-non-string half is the decoder's own `Err` (also 400).
    let (_guard, base) = live_server().await;
    for payload in [
        serde_json::json!({}),
        serde_json::json!({"query": ""}),
        serde_json::json!({"query": 5}),
        serde_json::json!({"query": null}),
    ] {
        let body = serde_json::json!({
            "schemaVersion": 1, "idFormat": "opaque-string-v1", "payload": payload.clone()
        });
        assert_post_validation_400(
            &base,
            body,
            &format!("§5.3 empty/non-string query {payload}"),
        )
        .await;
    }
}

// ===========================================================================
// §5.5 — the transport code mapping: total on the 5-variant domain, disjoint
// from the §11 map, and never 502.
// ===========================================================================

#[test]
fn u2_transport_codes_are_total_disjoint_and_never_502() {
    // §5.5's code table + its "Disjointness (testable)" rule + §9.5.1's
    // `P-TP-3`: for each of the five, `StoreError::from_wire(code, …) == None`.
    let cases = [
        (
            DecodeError::InvalidJson("x".to_string()),
            "invalid_json",
            400,
        ),
        (
            DecodeError::InvalidEnvelope("x".to_string()),
            "invalid_envelope",
            400,
        ),
        (
            DecodeError::UnsupportedSchemaVersion(99),
            "unsupported_schema_version",
            400,
        ),
        (
            DecodeError::UnknownIdFormat("uuid-v4".to_string()),
            "unknown_id_format",
            400,
        ),
        (
            DecodeError::UnknownMethod("bogus".to_string()),
            "unknown_method",
            422,
        ),
    ];
    for (e, code, status) in cases {
        assert_eq!(request_decode_code(&e), Some(code), "§5.5: transport code");
        assert_eq!(
            request_decode_status(&e),
            Some(status),
            "§5.5: transport status (never 502)"
        );
        assert_ne!(status, 502, "§5.5: a transport status is never 502");
        assert!(
            from_wire(code, Some("m")).is_none(),
            "§5.5: {code} must be absent from the §11 map"
        );
    }

    // Out-of-domain variants ⇒ both fns are `None` (defined together).
    for e in [
        DecodeError::UnknownType("cursor".to_string()),
        DecodeError::MissingTrace,
        DecodeError::UnknownCode("nope".to_string()),
        DecodeError::EventTypeMismatch {
            event: "result".to_string(),
            data_type: "done".to_string(),
        },
    ] {
        assert_eq!(
            request_decode_code(&e),
            None,
            "§5.5: `None` outside the 5-variant domain"
        );
        assert_eq!(
            request_decode_status(&e),
            None,
            "§5.5: the status is defined exactly where the code is"
        );
    }

    // A `StoreError` keeps its own §11 `wire_code()`, never a transport code.
    for e in [
        StoreError::ValidationError("x".to_string()),
        StoreError::EngineUnavailable,
        StoreError::EngineError,
    ] {
        assert!(
            !transport_codes().contains(&e.wire_code()),
            "§5.5: §11 code {} must not be a transport code",
            e.wire_code()
        );
    }

    // The §11 status of the two encoder-rule codes is pinned (§5.3/FS-9).
    assert_eq!(
        server_status(&StoreError::EngineError),
        Some((502, "engine_error")),
        "§5.3/§9.5.1 P-TP-4: a validator-rejected body renders 502 `engine_error`"
    );
}

// ===========================================================================
// §5.3 — the checked encoder: a validator-rejected `RagResult` is an `Err`,
// never a 200 carrying an un-decodable body.
// ===========================================================================

#[test]
fn u2_encode_result_checked_never_emits_a_rejected_body() {
    // §5.3's "The encoder must never emit a body the result validator rejects"
    // + §9.5.1's `P-TP-4`: the two representable rejected classes are an
    // `engine != "gnosis"` (including `""`) and a `blocked_by` without a graph
    // trace.
    assert!(
        encode_result_checked(&minimal_rag_result()).is_ok(),
        "§5.3: a well-formed `RagResult` encodes"
    );
    for engine in ["", "not-gnosis"] {
        let mut bad = minimal_rag_result();
        bad.engine = engine.to_string();
        assert_eq!(
            encode_result_checked(&bad),
            Err(StoreError::EngineError),
            "§5.3/§9.5.1 P-TP-4: `engine == {engine:?}` ⇒ Err(EngineError)"
        );
    }
    let mut bad = minimal_rag_result();
    bad.blocked_by = Some(vec![]);
    assert_eq!(
        encode_result_checked(&bad),
        Err(StoreError::EngineError),
        "§5.3/§9.5.1 P-TP-4: a `blocked_by` without a graph trace ⇒ Err(EngineError)"
    );
}
