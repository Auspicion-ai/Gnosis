//! §7.2 P2 **U3** — status honesty (the `EngineSubsystems` capability semantics)
//! — BLIND-GREENS SET (`tests/blind_u3_status_honesty_greens.rs`).
//!
//! Blind-test-writer derived from the DOCUMENTATION ONLY:
//! `docs/specs/p2-gnosis-server.md` (§5.8 the flag capability semantics + the
//! DEGRADED false claim + the frozen `EngineSubsystems` + the additive signal
//! scoped to `HealthReport`; §6 the READY lifecycle; §9.5.2 U3's five typed rows
//! — the capability predicate table, the pinned `boot_wiring(...)` seam, the F16
//! read-time derivation, adjudication notes 1–2; §9.5.3 the execution plan's
//! F11 lib seam; §9.5.4 the per-row coverage notes) and
//! `docs/specs/engine-wire-contract.md` (§9 `HealthReport` + the `health`
//! projection rules, §9.1 the capability semantics / FROZEN six-bool
//! `EngineSubsystems` / "U3 adds no `HealthReport` field" / no new status
//! surface, §12 V-8/V-8.1/V-8.2, §13 the `health` fail-states), plus
//! `docs/decisions.md`'s `SUBSYSTEM-FLAG-CAPABILITY-SEMANTICS`. **No `src/` file
//! was read for expectations** — only public API signatures (for compilation)
//! and the docs.
//!
//! Surfaces: the live `gnosis-server` bin (`CARGO_BIN_EXE_gnosis-server`) for the
//! HTTP rows, the engine lib's public API for the pure rows, and the **pinned**
//! `boot_wiring(provider: BootProvider, snapshot: &DerivedIndexes)
//! -> (EngineState, EngineSubsystems)` seam for the boot rows.
//!
//! **Layer discipline (docs: `p2` §9.5.2 `P-IM-9` + its adjudication note 1).**
//! The provider-REACHABLE *live server* outcome (`Ready` + `embedding:true` from
//! `GET /engine/status`) needs a **controlled provider** and is the live battery's
//! named row `R-L2` — never a node test's claim. Every live row here therefore
//! boots the bin with `GNOSIS_SERVER_OLLAMA_URL`/`GNOSIS_SERVER_OLLAMA_MODEL`
//! **removed**, so the not-`Ready` precondition is deterministic rather than
//! ambient; the `Reachable` outcome is asserted at lib level through the pinned
//! seam only.
//!
//! **Two-layer reading (§9.5.2 F16).** The flags are a **read-time projection**
//! inside `get_engine_status` of (a) the current snapshot's `vectors.is_some()`,
//! (b) the provider seam `embedding_provider().is_some()`, and (c) the three
//! always-true capability predicates. No scenario writes a mask as an expectation
//! source: `set_subsystems` appears **only** as `U3-11`'s adversarial write
//! (F11/F16 forbid the read-back assertion as proof of derivation).

use std::collections::HashMap;
use std::future::Future;
use std::net::TcpListener;
use std::pin::Pin;
use std::process::{Child, Command};
use std::sync::Arc;
use std::time::Duration;

use gnosis::wire::envelope::{current_schema_version, ID_FORMAT_OPAQUE_STRING_V1};
use gnosis::wire::status::health;
use gnosis::{
    boot_wiring, BootProvider, DerivedIndexes, DocumentId, EmbeddingProvider, EngineState,
    EngineStatus, EngineSubsystems, FieldType, NodeId, RagStore, Store, StoreError, VectorIndex,
};

const SERVER_BIN: &str = env!("CARGO_BIN_EXE_gnosis-server");

/// V-8.2's fixed DEGRADED `lastError` string — **unchanged** by U1/U3 ("it is the
/// store's fixed message … rewording it is not authorized", F2 §12).
const V8_2_LAST_ERROR: &str = "a non-core subsystem (embedding/reranker) is unavailable";

/// The frozen `subsystems` key set (§9.1/§4.6.1: `store`/`graph`/`lexical`/
/// `vector`/`embedding`/`reranker` — **exactly six** `bool`s; the type is FROZEN
/// and MUST NOT gain, lose or re-type a field).
const FLAG_KEYS: [&str; 6] = [
    "store",
    "graph",
    "lexical",
    "vector",
    "embedding",
    "reranker",
];

/// The frozen top-level `HealthReport` key set (F2 §9; §9.1's F12: "U3 lands
/// **NO** additive `HealthReport` field").
const REPORT_KEYS: [&str; 6] = [
    "schemaVersion",
    "idFormat",
    "state",
    "version",
    "subsystems",
    "lastError",
];

// ---------------------------------------------------------------------------
// Fixtures (§9.5.2's capability predicates; F2 §9.1's flag table).
// ---------------------------------------------------------------------------

/// A query-surface provider stub: it exists to make
/// `Store::embedding_provider().is_some()` true — the **capability predicate**
/// §9.5.2 pins for `embedding` ("a query-surface `Arc<dyn EmbeddingProvider>` is
/// wired for this store"). No scenario asserts an embedding value, so the stub
/// returns a deterministic vector and a constant probe result.
#[derive(Debug)]
struct StubProvider;

impl EmbeddingProvider for StubProvider {
    // The trait's own signature returns a boxed future, so the shape is the
    // contract's, not this stub's choice.
    #[allow(clippy::manual_async_fn)]
    fn embed(
        &self,
        text: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<f32>, StoreError>> + Send + '_>> {
        // `text` is consumed before the future is built, so the returned future
        // borrows only `self` (the trait's own `'_`).
        let dim = text.len() as f32;
        Box::pin(async move { Ok(vec![dim, 0.0, 0.0]) })
    }

    #[allow(clippy::manual_async_fn)]
    fn is_available(&self) -> Pin<Box<dyn Future<Output = bool> + Send + '_>> {
        Box::pin(async { true })
    }
}

fn provider() -> Arc<dyn EmbeddingProvider> {
    Arc::new(StubProvider)
}

/// One `boot_wiring` case: the label, the `BootProvider` handed to the seam, the
/// provider the boot's own wiring would install (`Some` only in the `Reachable`
/// case — §9.5.2 `P-IM-9`), and the state the seam must return.
type BootCase = (
    &'static str,
    BootProvider,
    Option<Arc<dyn EmbeddingProvider>>,
    EngineState,
);

/// The snapshot the U5-time boot index build will produce (§9.5.2's `vector`
/// predicate: "the **current derived snapshot** carries a vector index"). An
/// **empty** `VectorIndex` is the pinned `Some(…)` boundary: the capability is
/// the index being *wired*, not its entry count.
fn snapshot_with_empty_vector_index() -> DerivedIndexes {
    DerivedIndexes {
        lexical: None,
        vectors: Some(VectorIndex::default()),
        epoch: 0,
    }
}

/// The same index with one entry, so the "entry count is irrelevant" half is
/// asserted against a non-empty index too.
fn snapshot_with_populated_vector_index() -> DerivedIndexes {
    let mut entries: HashMap<(DocumentId, NodeId, FieldType), Vec<f32>> = HashMap::new();
    entries.insert(
        (
            DocumentId("d1".to_string()),
            NodeId("n1".to_string()),
            FieldType::Full,
        ),
        vec![0.5, 0.5],
    );
    DerivedIndexes {
        lexical: None,
        vectors: Some(VectorIndex { entries }),
        epoch: 1,
    }
}

/// The honest U3-time vectors (§9.5.2's `P-IM-9` observable + V-8.1's U3-stage
/// variant / V-8.2).
fn core_true_with(vector: bool, embedding: bool) -> EngineSubsystems {
    EngineSubsystems {
        store: true,
        graph: true,
        lexical: true,
        vector,
        embedding,
        reranker: false,
    }
}

/// The provider-absent honest vector (V-8.2, and §9.5.2's `P-IM-9`: "a
/// provider-absent boot reports `{store:true, graph:true, lexical:true,
/// vector:false, embedding:false, reranker:false}` — **never**
/// `embedding:true`").
fn provider_absent_expected() -> EngineSubsystems {
    core_true_with(false, false)
}

/// §9.5.2's capability predicates, applied to a store's own observable state —
/// the right-hand side the derivation must equal, built **from the store**, never
/// from a written mask.
async fn expected_from_store(s: &Store) -> EngineSubsystems {
    core_true_with(
        s.snapshot().vectors.is_some(),
        s.embedding_provider().is_some(),
    )
}

async fn assert_flags_match_predicates(s: &Store, label: &str) -> EngineSubsystems {
    let got = s.get_engine_status().await.subsystems;
    assert_eq!(
        got,
        expected_from_store(s).await,
        "U3: {label}: flags must equal the capability predicates \
         (store/graph/lexical true, reranker false, vector == snapshot().vectors.is_some(), \
         embedding == embedding_provider().is_some())"
    );
    got
}

// ---------------------------------------------------------------------------
// Live-server harness (docs: p2 §4 loopback bind, §6 READY lifecycle).
// ---------------------------------------------------------------------------

fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    listener.local_addr().expect("local addr").port()
}

/// Spawn the bin with the boot's provider configuration **removed**, so the boot
/// takes the provider-ABSENT branch regardless of the ambient environment and the
/// not-`Ready` precondition is deterministic (§9.5.2's boot-outcome table: the
/// absent branch wires none and makes no `embedding` claim). The provider-
/// REACHABLE live outcome is the live battery's row `R-L2`, never a claim here.
async fn spawn_server_without_provider() -> (Child, String) {
    let port = free_port();
    // The child is intentionally kept live for the test and reaped via
    // `ServerGuard`'s Drop (kill + wait), so no zombie is left.
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

/// Kill the live server child on drop (kill + wait ⇒ no zombie, no leaked
/// listener).
struct ServerGuard(Child);
impl Drop for ServerGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// A fresh provider-absent live server: `(guard, base)`.
async fn live_server_without_provider() -> (ServerGuard, String) {
    let (child, base) = spawn_server_without_provider().await;
    (ServerGuard(child), base)
}

/// `GET /engine/status` → `(status_code, parsed_body)`. §5.8: the endpoint is the
/// single status surface and is **always 200**.
async fn get_engine_status_json(base: &str) -> (u16, serde_json::Value) {
    let resp = reqwest::Client::new()
        .get(format!("{base}/engine/status"))
        .send()
        .await
        .expect("GET /engine/status");
    let status = resp.status().as_u16();
    let text = resp.text().await.expect("engine/status body");
    let body: serde_json::Value = serde_json::from_str(&text).expect("engine/status is JSON");
    (status, body)
}

/// The six `subsystems` flags of a wire report, asserted to be **exactly** the
/// frozen key set with `bool` values (F2 §9.1's F7: the type is FROZEN and MUST
/// NOT gain, lose or re-type a field), handed back as a key→bool map.
fn assert_six_bool_flags(body: &serde_json::Value) -> HashMap<String, bool> {
    let subsystems = body
        .get("subsystems")
        .and_then(|v| v.as_object())
        .unwrap_or_else(|| panic!("U3: HealthReport must carry a `subsystems` object, got {body}"));
    let mut keys: Vec<&str> = subsystems.keys().map(|k| k.as_str()).collect();
    keys.sort_unstable();
    let mut want: Vec<&str> = FLAG_KEYS.to_vec();
    want.sort_unstable();
    assert_eq!(
        keys, want,
        "U3: `subsystems` must carry EXACTLY the six frozen keys \
         (store/graph/lexical/vector/embedding/reranker) — no additive field"
    );
    let mut flags: HashMap<String, bool> = HashMap::new();
    for k in FLAG_KEYS {
        let v = subsystems
            .get(k)
            .unwrap_or_else(|| panic!("U3: missing frozen flag `{k}`"));
        let b = v
            .as_bool()
            .unwrap_or_else(|| panic!("U3: flag `{k}` must be a bool, got {v}"));
        flags.insert(k.to_string(), b);
    }
    flags
}

/// The top level of a wire report, asserted to be **exactly** the frozen six keys
/// (F2 §9's `HealthReport` shape; §9.1's F12: U3 adds no field).
fn assert_frozen_report_keys(body: &serde_json::Value) {
    let obj = body
        .as_object()
        .unwrap_or_else(|| panic!("U3: HealthReport must be a JSON object, got {body}"));
    let mut keys: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
    keys.sort_unstable();
    let mut want: Vec<&str> = REPORT_KEYS.to_vec();
    want.sort_unstable();
    assert_eq!(
        keys, want,
        "U3: the HealthReport top level is exactly \
         {{schemaVersion,idFormat,state,version,subsystems,lastError}} — U3 adds NO field"
    );
}

// ---------------------------------------------------------------------------
// U3-1 — §5.8: GET /engine/status is the single status surface, always 200, and
// the report carries exactly the six frozen flags.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn u3_1_engine_status_is_200_with_exactly_the_six_flag_keys() {
    let (_guard, base) = live_server_without_provider().await;
    let (status, body) = get_engine_status_json(&base).await;

    assert_eq!(
        status, 200,
        "U3-1 (§5.8/§6): `GET /engine/status` is the single status surface and is ALWAYS 200"
    );
    assert_frozen_report_keys(&body);
    let flags = assert_six_bool_flags(&body);
    assert_eq!(
        flags.len(),
        6,
        "U3-1: exactly six subsystem flags, all booleans"
    );

    assert_eq!(
        body.get("schemaVersion").and_then(|v| v.as_u64()),
        Some(u64::from(current_schema_version())),
        "U3-1 (F2 §9): the report's canonical envelope carries the current schemaVersion"
    );
    assert_eq!(
        body.get("idFormat").and_then(|v| v.as_str()),
        Some(ID_FORMAT_OPAQUE_STRING_V1),
        "U3-1 (F2 §9): the report's canonical envelope carries idFormat \
         \"opaque-string-v1\""
    );
    let state = body
        .get("state")
        .and_then(|v| v.as_str())
        .expect("U3-1 (F2 §9): the report carries a PascalCase `state`");
    assert!(
        ["Ready", "Starting", "Degraded", "Unavailable"].contains(&state),
        "U3-1 (F2 §9): `state` is the real serde output of the reused EngineState enum \
         (PascalCase Ready/Starting/Degraded/Unavailable), got {state:?}"
    );
    assert!(
        body.get("lastError")
            .map(|v| v.is_null() || v.is_string())
            .unwrap_or(false),
        "U3-1 (F2 §9): `lastError` is `null` or a string"
    );
}

// ---------------------------------------------------------------------------
// U3-2 — §6 + §9.5.2 P-IM-8/P-IM-9: the provider-absent boot makes no false
// claim and never fabricates READY.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn u3_2_provider_absent_boot_is_not_ready_and_claims_no_dead_capability() {
    let (_guard, base) = live_server_without_provider().await;
    let (status, body) = get_engine_status_json(&base).await;
    assert_eq!(status, 200, "U3-2: the status surface stays 200");
    assert_frozen_report_keys(&body);
    let flags = assert_six_bool_flags(&body);

    let state = body
        .get("state")
        .and_then(|v| v.as_str())
        .expect("U3-2: `state` present");
    assert_ne!(
        state, "Ready",
        "U3-2 (§6): a boot whose provider is absent must NOT fabricate READY; \
         §9.5.2's adjudication note 2 pins `Ready` iff the provider was reachable, \
         else ∈ {{Degraded, Unavailable}}"
    );

    assert_eq!(
        flags.get("embedding"),
        Some(&false),
        "U3-2 (§5.8/§9.5.2 P-IM-8): the DEGRADED/absent boot must not claim \
         `embedding:true` — the flag follows the wired provider"
    );
    assert_eq!(
        flags.get("vector"),
        Some(&false),
        "U3-2 (§9.5.2 P-IM-7): `vector` is false while the snapshot carries no vector \
         index (the boot path leaves `vectors: None` until U5)"
    );
    assert_eq!(
        flags.get("reranker"),
        Some(&false),
        "U3-2 (§9.5.2 P-IM-7): `reranker` is false in EVERY reachable state \
         (no reranker implementation exists)"
    );
    for core in ["store", "graph", "lexical"] {
        assert_eq!(
            flags.get(core),
            Some(&true),
            "U3-2 (§9.5.2 P-IM-7): core flag `{core}` is honest and always true"
        );
    }
    assert_eq!(
        flags,
        {
            let mut m = HashMap::new();
            for (k, v) in [
                ("store", true),
                ("graph", true),
                ("lexical", true),
                ("vector", false),
                ("embedding", false),
                ("reranker", false),
            ] {
                m.insert(k.to_string(), v);
            }
            m
        },
        "U3-2: the provider-absent boot's honest vector is {{store:true, graph:true, \
         lexical:true, vector:false, embedding:false, reranker:false}}"
    );
}

// ---------------------------------------------------------------------------
// U3-3 — §9.5.2 P-IM-7: `vector` is the index being WIRED, incl. Some(EMPTY).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn u3_3_vector_flag_is_the_wired_index_including_some_empty() {
    let s = Store::new();

    let fresh = assert_flags_match_predicates(&s, "fresh store").await;
    assert_eq!(
        fresh,
        provider_absent_expected(),
        "U3-3: a fresh store (vectors:None, no provider) reports the honest \
         provider-absent vector"
    );

    s.swap_snapshot(DerivedIndexes::default());
    let explicit_none = assert_flags_match_predicates(&s, "default snapshot").await;
    assert!(
        !explicit_none.vector,
        "U3-3 (§9.5.2 P-IM-7): `vector` is false exactly while the current snapshot \
         has no vector index"
    );

    // The pinned boundary: an EMPTY VectorIndex still means the capability is
    // wired (the predicate is `snapshot().vectors.is_some()`, not an entry count).
    s.swap_snapshot(snapshot_with_empty_vector_index());
    let empty_index = assert_flags_match_predicates(&s, "Some(empty VectorIndex)").await;
    assert!(
        empty_index.vector,
        "U3-3 (§9.5.2 P-IM-7): `Some(empty VectorIndex)` ⇒ `vector:true` — the \
         capability is the index being WIRED, not its entry count"
    );

    s.swap_snapshot(snapshot_with_populated_vector_index());
    let populated = assert_flags_match_predicates(&s, "populated VectorIndex").await;
    assert!(
        populated.vector,
        "U3-3 (§9.5.2 P-IM-7): a populated index is `vector:true` too"
    );

    for (label, flags) in [
        ("fresh", &fresh),
        ("explicit None", &explicit_none),
        ("empty index", &empty_index),
        ("populated index", &populated),
    ] {
        assert!(
            flags.store && flags.graph && flags.lexical,
            "U3-3: core store/graph/lexical stay true ({label})"
        );
        assert!(
            !flags.reranker,
            "U3-3: `reranker` stays false in every reachable state ({label})"
        );
    }
}

// ---------------------------------------------------------------------------
// U3-4 — §9.5.2 P-IM-7/P-IM-8: the four EngineStates × provider present/absent.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn u3_4_flags_track_the_provider_seam_across_the_four_states() {
    let states = [
        EngineState::Ready,
        EngineState::Starting,
        EngineState::Degraded,
        EngineState::Unavailable,
    ];
    for state in states {
        for wired in [false, true] {
            let s = Store::new();
            if wired {
                s.set_embedding_provider(provider());
            }
            s.set_engine_state(state);

            let status = s.get_engine_status().await;
            assert_eq!(
                status.state, state,
                "U3-4: the read surfaces the store's own engine state ({state:?})"
            );
            let flags = assert_flags_match_predicates(
                &s,
                &format!("state {state:?}, provider_wired={wired}"),
            )
            .await;

            assert_eq!(
                flags.embedding,
                s.embedding_provider().is_some(),
                "U3-4 (§9.5.2 P-IM-8): `embedding == s.embedding_provider().is_some()` — \
                 the flag tracks the PROVIDER SEAM, not the state \
                 (state {state:?}, provider_wired={wired})"
            );
            assert_eq!(
                flags.vector,
                s.snapshot().vectors.is_some(),
                "U3-4 (§9.5.2 P-IM-7): `vector == s.snapshot().vectors.is_some()` \
                 (state {state:?}, provider_wired={wired})"
            );
            assert!(
                flags.store && flags.graph && flags.lexical,
                "U3-4 (§9.5.2 P-IM-7): core flags are always true \
                 (state {state:?}, provider_wired={wired})"
            );
            assert!(
                !flags.reranker,
                "U3-4 (§9.5.2 P-IM-7): `reranker` is false in every reachable state \
                 (state {state:?}, provider_wired={wired})"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// U3-5 — §9.5.2 P-IM-8: a WIRED-but-unreachable provider keeps embedding:true
// while state/last_error carry the degradation.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn u3_5_wired_provider_keeps_embedding_true_with_the_degradation_in_state() {
    let s = Store::new();
    s.set_embedding_provider(provider());
    s.set_engine_state(EngineState::Degraded);
    s.swap_snapshot(DerivedIndexes::default());

    let status = s.get_engine_status().await;
    assert_eq!(
        status.state,
        EngineState::Degraded,
        "U3-5: the state axis carries the degradation"
    );
    assert!(
        status.subsystems.embedding,
        "U3-5 (§9.5.2 P-IM-8): the flag is about the WIRED capability, not live \
         provider reachability — a wired-then-unreachable provider keeps \
         `embedding:true`"
    );
    assert_eq!(
        status.last_error.as_deref(),
        Some(V8_2_LAST_ERROR),
        "U3-5 (§9.5.2 P-IM-8 / V-8.2): the honest signal for a wired-then-unreachable \
         provider is the `state`/`last_error` pair — the DEGRADED report carries the \
         store's fixed message"
    );
    let flags = assert_flags_match_predicates(&s, "Degraded + wired provider").await;
    assert_eq!(
        flags,
        core_true_with(false, true),
        "U3-5 (§9.5.2 P-IM-8): the honest vector for `{{Degraded, provider:Some, \
         no index}}` is {{store:true, graph:true, lexical:true, vector:false, \
         embedding:true, reranker:false}} — the flag tracks the seam, not the state"
    );

    // The other half of the pin: with NO provider wired, the DEGRADED state still
    // makes no claim — `Degraded` never implies `embedding:true` (§9.5.2 P-IM-8's
    // pinned `{Degraded, provider:None}` instance = V-8.2).
    let degraded = Store::new();
    degraded.set_engine_state(EngineState::Degraded);
    let status = degraded.get_engine_status().await;
    assert_eq!(
        status.state,
        EngineState::Degraded,
        "U3-5: the second store is DEGRADED"
    );
    assert_eq!(
        status.last_error.as_deref(),
        Some(V8_2_LAST_ERROR),
        "U3-5 (V-8.2): the DEGRADED `lastError` string is the store's fixed message"
    );
    assert!(
        !status.subsystems.embedding,
        "U3-5 (§9.5.2 P-IM-8): `Degraded` NEVER implies `embedding:true` when no \
         provider is wired — the honest V-8.2 mask"
    );
    assert!(
        !status.subsystems.vector && !status.subsystems.reranker,
        "U3-5 (§9.5.2 P-IM-8): the `{{Degraded, None}}` instance additionally \
         asserts `vector == false` and `reranker == false`"
    );
    assert_eq!(
        status.subsystems,
        provider_absent_expected(),
        "U3-5 (V-8.2): the `{{Degraded, provider:None}}` flag vector is exactly \
         {{store:true, graph:true, lexical:true, vector:false, embedding:false, \
         reranker:false}}"
    );
}

// ---------------------------------------------------------------------------
// U3-6 — §9.5.2 P-IM-7: `reranker:false` in every reachable state.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn u3_6_reranker_is_false_in_every_reachable_state() {
    let states = [
        EngineState::Ready,
        EngineState::Starting,
        EngineState::Degraded,
        EngineState::Unavailable,
    ];
    let snapshots = [
        ("no index", DerivedIndexes::default()),
        ("empty index", snapshot_with_empty_vector_index()),
    ];
    for state in states {
        for wired in [false, true] {
            for (snap_label, snap) in snapshots.clone() {
                let s = Store::new();
                if wired {
                    s.set_embedding_provider(provider());
                }
                s.set_engine_state(state);
                s.swap_snapshot(snap);
                let flags = assert_flags_match_predicates(
                    &s,
                    &format!("{state:?}/wired={wired}/{snap_label}"),
                )
                .await;
                assert!(
                    !flags.reranker,
                    "U3-6 (§9.5.2 P-IM-7 / V-8.1): `reranker:false` is \
                     UNCONDITIONAL in every reachable state (no reranker \
                     implementation exists) — violated at {state:?}/wired={wired}/{snap_label}"
                );
            }
        }
    }

    // The projection keeps it false too (F2 §9's verbatim mapping).
    let status = EngineStatus {
        state: EngineState::Ready,
        version: "v".to_string(),
        subsystems: core_true_with(true, true),
        last_error: None,
    };
    assert!(
        !health(&status).subsystems.reranker,
        "U3-6: the `health` projection must not invent a reranker capability"
    );
}

// ---------------------------------------------------------------------------
// U3-7 — §9.5.2 P-IM-9: boot_wiring's three BootProvider outcomes.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn u3_7_boot_wiring_three_outcomes_are_exact() {
    let snapshots = [
        ("traceless", DerivedIndexes::default()),
        ("vector-bearing", snapshot_with_empty_vector_index()),
    ];
    for (snap_label, snap) in snapshots {
        let cases: [(&str, BootProvider, EngineState, bool); 3] = [
            (
                "Absent",
                BootProvider::Absent,
                EngineState::Unavailable,
                false,
            ),
            (
                "Unreachable",
                BootProvider::Unreachable,
                EngineState::Degraded,
                false,
            ),
            (
                "Reachable",
                BootProvider::Reachable(provider()),
                EngineState::Ready,
                true,
            ),
        ];
        for (label, provider, want_state, want_embedding) in cases {
            let (state, vector) = boot_wiring(provider, &snap);
            assert_eq!(
                state, want_state,
                "U3-7 (§9.5.2 P-IM-9): `{label}` ⇒ state {want_state:?} \
                 (Absent ⇒ Unavailable; Unreachable ⇒ Degraded; Reachable ⇒ Ready) \
                 — snapshot {snap_label}"
            );
            assert_eq!(
                vector,
                core_true_with(snap.vectors.is_some(), want_embedding),
                "U3-7 (§9.5.2 P-IM-9): `{label}` ⇒ the derived vector of WHAT WAS \
                 WIRED (store/graph/lexical true, reranker false, \
                 vector == snapshot.vectors.is_some(), embedding == (a provider was \
                 wired)) — snapshot {snap_label}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// U3-8 — §9.5.2 P-IM-9: the seam's returned vector equals the store's own
// derived read after the PINNED wiring path (never a set_subsystems write).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn u3_8_boot_wiring_vector_equals_the_stores_derived_read() {
    let snapshots = [
        ("traceless", DerivedIndexes::default()),
        ("vector-bearing", snapshot_with_empty_vector_index()),
    ];
    for (snap_label, snap) in snapshots {
        // BootProvider is not `Copy`, so the three cases are built per iteration
        // and the Reachable case hands its Arc to the seam AND the store.
        let cases: Vec<BootCase> = vec![
            (
                "Absent",
                BootProvider::Absent,
                None,
                EngineState::Unavailable,
            ),
            (
                "Unreachable",
                BootProvider::Unreachable,
                None,
                EngineState::Degraded,
            ),
            (
                "Reachable",
                BootProvider::Reachable(provider()),
                Some(provider()),
                EngineState::Ready,
            ),
        ];
        for (label, provider, wired_provider, want_state) in cases {
            let (state, returned_vector) = boot_wiring(provider, &snap);
            assert_eq!(
                state, want_state,
                "U3-8: the returned EngineState is what the boot applies ({label})"
            );

            // The PINNED wiring path (§9.5.2 P-IM-9 / BLOCKING item 4 of the
            // second remand): apply ONLY the returned state, plus the boot's own
            // wiring — the snapshot swap, and the provider seam only in the
            // Reachable case. NEVER `set_subsystems`.
            let s = Store::new();
            s.set_engine_state(state);
            s.swap_snapshot(snap.clone());
            if let Some(ref p) = wired_provider {
                s.set_embedding_provider(Arc::clone(p));
            }

            let derived = s.get_engine_status().await;
            assert_eq!(
                derived.state, state,
                "U3-8: the store reports the applied state ({label})"
            );
            assert_eq!(
                derived.subsystems, returned_vector,
                "U3-8 (§9.5.2 P-IM-9): the pair is what the wiring DEMONSTRATES and \
                 the derived read is what the store REPORTS — a disagreement is a \
                 broken row ({label}, snapshot {snap_label})"
            );
            assert_eq!(
                returned_vector,
                core_true_with(snap.vectors.is_some(), wired_provider.is_some()),
                "U3-8: the returned vector is the derived capability vector of what \
                 this boot wired ({label}, snapshot {snap_label})"
            );
        }
    }

    // The U3-time Reachable vector, spelled out (V-8.1's U3-stage variant).
    let (state, vector) = boot_wiring(
        BootProvider::Reachable(provider()),
        &DerivedIndexes::default(),
    );
    assert_eq!(state, EngineState::Ready, "U3-8: Reachable ⇒ Ready");
    assert_eq!(
        vector,
        core_true_with(false, true),
        "U3-8 (§9.5.2 P-IM-9): `Reachable` ⇒ exactly {{store:true, graph:true, \
         lexical:true, vector:false, embedding:true, reranker:false}} at U3-time"
    );
    let (state, vector) = boot_wiring(BootProvider::Absent, &DerivedIndexes::default());
    assert_eq!(
        state,
        EngineState::Unavailable,
        "U3-8: Absent ⇒ Unavailable"
    );
    assert_eq!(
        vector,
        provider_absent_expected(),
        "U3-8 (§9.5.2 P-IM-9): Absent/Unreachable ⇒ exactly {{…, embedding:false, \
         vector:false, reranker:false}}"
    );
}

// ---------------------------------------------------------------------------
// U3-9 — §9.5.2 P-SM-5: the status read is deterministic and side-effect-free.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn u3_9_repeated_status_read_is_side_effect_free() {
    let states = [
        EngineState::Ready,
        EngineState::Starting,
        EngineState::Degraded,
        EngineState::Unavailable,
    ];
    for state in states {
        for wired in [false, true] {
            let s = Store::new();
            if wired {
                s.set_embedding_provider(provider());
            }
            s.set_engine_state(state);
            s.swap_snapshot(snapshot_with_empty_vector_index());

            let snap_before = s.snapshot();
            let vectors_before = snap_before.vectors.is_some();
            let epoch_before = s.epoch();
            let journal_before = s.journal_len();

            let a = s.get_engine_status().await;
            let b = s.get_engine_status().await;

            assert_eq!(
                a, b,
                "U3-9 (§9.5.2 P-SM-5): repeated reads with no intervening mutation \
                 return element-wise identical EngineStatus values \
                 (state {state:?}, wired={wired})"
            );
            assert_eq!(a.state, b.state, "U3-9: state identical");
            assert_eq!(a.version, b.version, "U3-9: version identical");
            assert_eq!(a.subsystems, b.subsystems, "U3-9: every flag identical");
            assert_eq!(a.last_error, b.last_error, "U3-9: last_error identical");

            let snap_after = s.snapshot();
            assert_eq!(
                s.epoch(),
                epoch_before,
                "U3-9 (§9.5.2 P-SM-5): the epoch does not advance across a status read \
                 (state {state:?}, wired={wired})"
            );
            assert_eq!(
                s.journal_len(),
                journal_before,
                "U3-9 (§9.5.2 P-SM-5): the journal does not grow across a status read \
                 (state {state:?}, wired={wired})"
            );
            assert!(
                Arc::ptr_eq(&snap_before, &snap_after),
                "U3-9 (§9.5.2 P-SM-5): the derived snapshot's Arc identity is \
                 unchanged by a status read (state {state:?}, wired={wired})"
            );
            assert_eq!(
                snap_after.vectors.is_some(),
                vectors_before,
                "U3-9 (§9.5.2 P-SM-5): the snapshot's `vectors` presence is unchanged"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// U3-10 — F2 §9/§9.1: the HealthReport is a faithful, frozen-shape projection
// with NO additive field.
// ---------------------------------------------------------------------------

#[test]
fn u3_10_health_report_is_frozen_shape_and_verbatim_projection() {
    let masks = [
        (
            "all-false",
            EngineState::Unavailable,
            EngineSubsystems {
                store: false,
                graph: false,
                lexical: false,
                vector: false,
                embedding: false,
                reranker: false,
            },
        ),
        (
            "honest absent",
            EngineState::Unavailable,
            provider_absent_expected(),
        ),
        (
            "Degraded absent",
            EngineState::Degraded,
            provider_absent_expected(),
        ),
        (
            "Degraded wired",
            EngineState::Degraded,
            core_true_with(false, true),
        ),
        (
            "Ready wired",
            EngineState::Ready,
            core_true_with(true, true),
        ),
        (
            "Starting",
            EngineState::Starting,
            core_true_with(false, false),
        ),
    ];
    for (label, state, subsystems) in masks {
        for last_error in [None, Some(String::new()), Some(V8_2_LAST_ERROR.to_string())] {
            let status = EngineStatus {
                state,
                version: "v-test".to_string(),
                subsystems: subsystems.clone(),
                last_error: last_error.clone(),
            };
            let report = health(&status);

            assert_eq!(
                report.state, status.state,
                "U3-10: state mirrored ({label})"
            );
            assert_eq!(
                report.version, status.version,
                "U3-10 (F2 §9): version mirrored verbatim ({label})"
            );
            assert_eq!(
                report.subsystems, status.subsystems,
                "U3-10 (F2 §9): each of the six flags mirrored ({label})"
            );
            assert_eq!(
                report.last_error.is_some(),
                status.last_error.is_some(),
                "U3-10 (F2 §9): `last_error` is `Some` exactly when the input's is \
                 `Some` ({label}, input {last_error:?})"
            );
            assert_eq!(
                report.last_error, status.last_error,
                "U3-10: the `last_error` value is mirrored, never invented ({label})"
            );
            assert_eq!(
                report.schema_version,
                current_schema_version(),
                "U3-10 (F2 §9): the envelope constants are the pinned current \
                 version, never derived from the input ({label})"
            );
            assert_eq!(
                report.id_format, ID_FORMAT_OPAQUE_STRING_V1,
                "U3-10 (F2 §9): idFormat is the pinned opaque-string-v1 ({label})"
            );

            let json = serde_json::to_value(&report).expect("HealthReport serializes");
            let mut keys: Vec<&str> = json
                .as_object()
                .expect("report is an object")
                .keys()
                .map(|k| k.as_str())
                .collect();
            keys.sort_unstable();
            let mut want: Vec<&str> = REPORT_KEYS.to_vec();
            want.sort_unstable();
            assert_eq!(
                keys, want,
                "U3-10 (F2 §9/§9.1 F12): the wire report's top level is EXACTLY the six \
                 frozen keys — U3 lands NO additive HealthReport field ({label})"
            );
            let subs = json
                .get("subsystems")
                .and_then(|v| v.as_object())
                .expect("subsystems object");
            let mut sub_keys: Vec<&str> = subs.keys().map(|k| k.as_str()).collect();
            sub_keys.sort_unstable();
            let mut want_sub: Vec<&str> = FLAG_KEYS.to_vec();
            want_sub.sort_unstable();
            assert_eq!(
                sub_keys, want_sub,
                "U3-10 (F2 §9.1): `EngineSubsystems` is FROZEN — exactly six bools, \
                 no field gained, lost or re-typed ({label})"
            );
            for k in FLAG_KEYS {
                assert!(
                    subs.get(k).map(|v| v.is_boolean()).unwrap_or(false),
                    "U3-10: flag `{k}` must remain a bool on the wire ({label})"
                );
            }
        }
    }

    // The contradictory input is mirrored verbatim, never "fixed" (§9.5.4's
    // P-SM-6 adversarial case).
    let contradictory = EngineStatus {
        state: EngineState::Degraded,
        version: "v".to_string(),
        subsystems: provider_absent_expected(),
        last_error: None,
    };
    assert_eq!(
        health(&contradictory).last_error,
        None,
        "U3-10 (§9.5.4 P-SM-6): a contradictory `{{Degraded, last_error:None}}` input \
         is mirrored verbatim — `health` invents nothing"
    );

    // Determinism: equal input ⇒ equal report (F2 §9).
    let a = health(&contradictory);
    let b = health(&contradictory);
    assert_eq!(
        a, b,
        "U3-10 (F2 §9): equal EngineStatus ⇒ equal HealthReport"
    );
}

// ---------------------------------------------------------------------------
// U3-11 — §9.5.2 P-IM-7 / F16: the projection is write-independent (the
// injected false mask must not flip any flag).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn u3_11_injected_mask_does_not_produce_any_flag() {
    let s = Store::new();
    s.set_engine_state(EngineState::Degraded);
    s.swap_snapshot(DerivedIndexes::default());

    let before = s.get_engine_status().await.subsystems;
    assert_eq!(
        before,
        provider_absent_expected(),
        "U3-11: the pre-write derived read is the honest provider-absent vector"
    );

    // The adversarial write: an all-true mask through the (test-hook) mutator.
    s.set_subsystems(EngineSubsystems {
        store: true,
        graph: true,
        lexical: true,
        vector: true,
        embedding: true,
        reranker: true,
    });

    let after = s.get_engine_status().await.subsystems;
    assert_eq!(
        after, before,
        "U3-11 (§9.5.2 P-IM-7 / F16): the derivation is the READ-TIME projection \
         inside `get_engine_status` — no caller-visible value depends on a written \
         mask; an injected all-true mask must not flip ANY flag"
    );
    assert!(
        !after.vector && !after.embedding && !after.reranker,
        "U3-11: in particular the false claims must not appear after the mask write"
    );
    assert_eq!(
        after,
        expected_from_store(&s).await,
        "U3-11: the flags still equal the capability predicates"
    );
}

// ---------------------------------------------------------------------------
// U3-12 — §5.8/§6 + P-SM-5 (live half): a repeated status read is stable.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn u3_12_repeated_live_status_read_is_identical() {
    let (_guard, base) = live_server_without_provider().await;
    let (first_status, first) = get_engine_status_json(&base).await;
    let (second_status, second) = get_engine_status_json(&base).await;

    assert_eq!(first_status, 200, "U3-12: the first read is 200");
    assert_eq!(second_status, 200, "U3-12: the repeated read is 200");
    for key in ["state", "version", "subsystems", "lastError"] {
        assert_eq!(
            first.get(key),
            second.get(key),
            "U3-12 (§9.5.2 P-SM-5): `{key}` is identical across repeated status reads"
        );
    }
    assert_eq!(
        first, second,
        "U3-12 (§9.5.2 P-SM-5): the whole report is identical across repeated reads \
         — the read is a deterministic, side-effect-free projection"
    );
    assert_frozen_report_keys(&second);
    assert_six_bool_flags(&second);
}

// ---------------------------------------------------------------------------
// U3-13 — V-8.1 (U3-stage variant) / V-8.2: the honest U3-time vectors.
// ---------------------------------------------------------------------------

#[test]
fn u3_13_honest_u3_time_vectors_match_v8_1_stage_and_v8_2() {
    // V-8.1's U3-stage variant: `{store:true, graph:true, lexical:true,
    // vector:false, embedding:true, reranker:false}` (the index is not built
    // until U5).
    let ready_u3_stage = EngineStatus {
        state: EngineState::Ready,
        version: "v".to_string(),
        subsystems: core_true_with(false, true),
        last_error: None,
    };
    let report = health(&ready_u3_stage);
    assert_eq!(
        report.state,
        EngineState::Ready,
        "U3-13: V-8.1's state is Ready"
    );
    assert_eq!(
        report.subsystems,
        core_true_with(false, true),
        "U3-13 (V-8.1, U3-stage variant): {{store:true, graph:true, lexical:true, \
         vector:false, embedding:true, reranker:false}}"
    );
    assert_eq!(report.last_error, None, "U3-13: V-8.1's lastError is null");

    let json = serde_json::to_value(&report).expect("serializes");
    assert_eq!(
        json.get("lastError"),
        Some(&serde_json::Value::Null),
        "U3-13 (V-8.1): `lastError` is null (camelCase top level)"
    );
    let subs = json.get("subsystems").and_then(|v| v.as_object()).unwrap();
    assert_eq!(
        subs.get("vector").and_then(|v| v.as_bool()),
        Some(false),
        "U3-13 (§5.8's unit split): `vector` stays false until U5's boot index build"
    );
    assert_eq!(
        subs.get("reranker").and_then(|v| v.as_bool()),
        Some(false),
        "U3-13 (V-8.1): `reranker:false` is unconditional"
    );

    // V-8.2: the amended DEGRADED literal, with the fixed message unchanged.
    let degraded = EngineStatus {
        state: EngineState::Degraded,
        version: "v".to_string(),
        subsystems: provider_absent_expected(),
        last_error: Some(V8_2_LAST_ERROR.to_string()),
    };
    let report = health(&degraded);
    assert_eq!(
        report.subsystems,
        provider_absent_expected(),
        "U3-13 (V-8.2): {{store:true, graph:true, lexical:true, vector:false, \
         embedding:false, reranker:false}}"
    );
    assert_eq!(
        report.last_error.as_deref(),
        Some(V8_2_LAST_ERROR),
        "U3-13 (V-8.2/§9.5.2): the DEGRADED lastError string is unchanged: \
         \"{V8_2_LAST_ERROR}\""
    );
    let json = serde_json::to_value(&report).expect("serializes");
    assert_eq!(
        json.get("lastError").and_then(|v| v.as_str()),
        Some(V8_2_LAST_ERROR),
        "U3-13 (V-8.2): the emitted `lastError` value is the fixed message verbatim"
    );
}
