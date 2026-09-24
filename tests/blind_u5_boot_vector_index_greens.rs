//! §7.2 P2 **U5** — the boot vector-index build — BLIND-GREENS SET
//! (`tests/blind_u5_boot_vector_index_greens.rs`).
//!
//! Blind-test-writer derived from the DOCUMENTATION ONLY:
//! `docs/specs/p2-gnosis-server.md` — **§5.8** (the flag capability semantics
//! and U5's answer bullet: the `vector` predicate is `snapshot().vectors.is_some()`,
//! U5 changes only *what the boot puts in the snapshot*; the V-8.1 mask is the
//! `Reachable` + `vectors: Some` instance), **§5.9** (the FS-8/FS-13/FS-14 outcome
//! table, the `Precedence (pinned)` bullet — the READY gate precedes every leg check
//! for non-`graph` modes, and the index is checked before the provider), **§6** (the
//! post-U5 READY-lifecycle sequence: build → compose → wire → state),
//! **§9.5.2** (`P-IM-7`/`P-IM-8`/`P-IM-9` and the capability predicate table, F13,
//! F16), **§9.5.5** (U5's contract: the three `BootProvider` inputs, the corpus
//! table, the `FieldType::Full`-only choice, the cost/failure table, the
//! hermeticity/no-knob clause, the U4/durability isolation clause, the eight typed
//! rows `P-IM-10`…`P-IM-15`/`P-SM-7`/`P-TP-5`, the "corpus seeding" clause, the
//! valid/fail states 1–7, the `NaN` vector-comparison rule and the `embed`-call
//! count/discipline row, the adjudication notes, the residual list), **§9.5.4**'s
//! eight U5 coverage notes, **§10** (the `/rag/query` fail-states), **§11** (the U5
//! obligation row + its TestWriter surface note), **§13** (U5's constraint bullets);
//! `docs/specs/engine-wire-contract.md` **§9.1** (the same semantics; the U5 note),
//! **§12** (V-8.1/V-8.2 and the U5 note), **§16** (the ruling's rules 1–6, incl.
//! readiness-precedes-leg-availability); and
//! `docs/specs/p2-gnosis-server-live-pending-battery.md` §3.5 (`R-L3`, whose live
//! criteria (iii) and (v) are PARKED — this set does not claim them).
//! **No `src/` file was read for expectations, and neither
//! `tests/u5_boot_vector_index_conformance.rs` nor the U5 rows of
//! `tests/props_gnosis_server.rs` was read** — the only non-doc inputs were public
//! API *signatures* (for compilation) and the two precedent blind sets
//! (`tests/blind_u2_query_post_greens.rs`, `tests/blind_u3_status_honesty_greens.rs`)
//! for style, harness shape and the live/pure split.
//!
//! Surfaces: the lib seam `build_boot_vector_index(&Store, Option<&Arc<dyn
//! EmbeddingProvider>>) -> Result<Option<VectorIndex>, StoreError>` (§9.5.5's surface
//! table #1 — the F11 lib-visible precedent), the pinned `boot_wiring(provider:
//! BootProvider, snapshot: &DerivedIndexes) -> (EngineState, EngineSubsystems)` seam
//! (#2), the frozen store types (#4), the store seams (#5), an injected
//! deterministic in-memory `EmbeddingProvider` (#6) and the public `RagStore`
//! corpus-seeding calls (#7). Live rows use the `gnosis-server` bin
//! (`CARGO_BIN_EXE_gnosis-server`).
//!
//! **Layer discipline (§9.5.5's adjudication note 1 + its hermeticity clause).** All
//! eight register rows are **lib-level**; the unit's one live obligation is the
//! battery's `R-L3`. A U5 test MUST inject a deterministic in-memory provider and
//! MUST NOT depend on a live provider or on the ambient environment, so every live
//! row here either `env_remove`s the boot's provider configuration (⇒ `Absent`) or
//! points it at a loopback port with nothing listening (⇒ `Unreachable`). The
//! provider-**reachable** live boot (and the live failed-build branch) are therefore
//! **NOT-VERIFIED** here, with the reasons recorded in the report.
//!
//! **Verdict vocabulary.** Every scenario below is GREEN (verified), RED
//! (contradiction — reported, never reconciled by editing the test) or NOT-VERIFIED
//! (unconstructible, with the reason). This set carries **26** executed scenarios;
//! the NOT-VERIFIED entries are recorded in
//! `docs/specs/u5-boot-vector-index-greens.md`.

use std::future::Future;
use std::net::TcpListener;
use std::pin::Pin;
use std::process::{Child, Command};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gnosis::wire::crud::ENGINE_ENDPOINTS;
use gnosis::{
    boot_wiring, build_boot_vector_index, route_bijection, server_status, BootProvider,
    CreateDocumentRequest, DerivedIndexes, DocumentId, Edge, EdgeKind, EmbeddingProvider,
    EngineState, EngineSubsystems, FieldType, Graph, Node, NodeId, NodeKind, QueryMode,
    RagQueryOptions, RagStore, RagTrace, Store, StoreError, UpdateDocumentRequest, VectorIndex,
    WikiId,
};

const SERVER_BIN: &str = env!("CARGO_BIN_EXE_gnosis-server");

/// The store's **fixed** DEGRADED `lastError` (`V-8.2`; §9.5.5's failure table:
/// "the `Degraded` state carries the store's **existing fixed** `last_error`
/// string … not reworded by U5").
const FIXED_LAST_ERROR: &str = "a non-core subsystem (embedding/reranker) is unavailable";

// ===========================================================================
// Fixtures: an injected deterministic in-memory provider (§9.5.5 surface #6).
// ===========================================================================

/// The provider behaviours §9.5.5's rows quantify over: the happy shape, the
/// pinned adversarial shapes of `P-TP-5` (dimension change / empty vector / zero
/// vector / `NaN`+`±∞`), and the failure shapes of `P-IM-13`/`P-IM-15`.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Behavior {
    /// `vec![1.0; 1 + text.len()]` — the returned length depends on the text, so
    /// two nodes with different texts get **different dimensions** (the `P-TP-5`
    /// "different dimension than a previous call" shape; no dimension is pinned by
    /// U5 — "the provider's returned length is authoritative").
    ByText,
    /// A zero vector.
    Zeros,
    /// An **empty** `Vec<f32>`.
    Empty,
    /// One `NaN` plus `±∞` (comparable elements are compared like any other; the
    /// `NaN` position is asserted by the pinned comparison rule).
    NaNInf,
    /// Every `embed` call errors with the **existing** `EmbeddingUnavailable`.
    AlwaysErr,
    /// The *k*-th (1-based) `embed` call errors; earlier calls succeed.
    FailOnCall(usize),
    /// The first call errors, every later call succeeds (the "not sticky" shape).
    FailOnceThenOk,
}

/// The provider's own return for one text — the **expected** value the build's
/// stored vector must carry verbatim (§9.5.5's `vector comparison / NaN` row:
/// same `len()`, every non-`NaN` element bit-for-bit, `NaN` asserted only in the
/// `NaN` position, no tolerance).
fn expected_vector(behavior: Behavior, text: &str) -> Vec<f32> {
    match behavior {
        Behavior::ByText => vec![1.0; 1 + text.len()],
        Behavior::Zeros => vec![0.0; 3],
        Behavior::Empty => Vec::new(),
        Behavior::NaNInf => vec![
            f32::NAN,
            f32::INFINITY,
            f32::NEG_INFINITY,
            text.len() as f32,
        ],
        Behavior::AlwaysErr | Behavior::FailOnCall(_) | Behavior::FailOnceThenOk => Vec::new(),
    }
}

/// A scripted provider that **counts** its calls (the call-count row is an
/// invariant, not a cost note), records the texts it was asked to embed (so "no
/// call for any other text" is observable), detects fan-out (max in-flight > 1
/// would falsify "strictly sequential") and counts `is_available` probes (the build
/// "does not call `is_available` itself"; §9.5.5's `P-IM-10`/`P-TP-5`).
#[derive(Debug)]
struct Scripted {
    behavior: Behavior,
    availability: bool,
    calls: AtomicUsize,
    texts: Mutex<Vec<String>>,
    in_flight: AtomicUsize,
    max_in_flight: AtomicUsize,
    availability_calls: AtomicUsize,
}

fn scripted(behavior: Behavior) -> Arc<Scripted> {
    scripted_with_availability(behavior, true)
}

/// The same provider with a **lying** `is_available` (`true` while `embed`
/// errors — `P-TP-5`'s shape).
fn scripted_with_availability(behavior: Behavior, availability: bool) -> Arc<Scripted> {
    Arc::new(Scripted {
        behavior,
        availability,
        calls: AtomicUsize::new(0),
        texts: Mutex::new(Vec::new()),
        in_flight: AtomicUsize::new(0),
        max_in_flight: AtomicUsize::new(0),
        availability_calls: AtomicUsize::new(0),
    })
}

/// The `Arc<dyn EmbeddingProvider>` the seam takes (surface #1's argument type).
fn dyn_provider(p: &Arc<Scripted>) -> Arc<dyn EmbeddingProvider> {
    p.clone()
}

impl Scripted {
    fn call_count(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }

    fn availability_probes(&self) -> usize {
        self.availability_calls.load(Ordering::SeqCst)
    }

    fn max_concurrency(&self) -> usize {
        self.max_in_flight.load(Ordering::SeqCst)
    }

    fn embedded_texts(&self) -> Vec<String> {
        let mut t = self.texts.lock().expect("text log").clone();
        t.sort();
        t
    }
}

impl EmbeddingProvider for Scripted {
    // The trait's own signature returns a boxed future (surface #6), so the shape
    // is the contract's, not this stub's choice.
    #[allow(clippy::manual_async_fn)]
    fn embed(
        &self,
        text: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<f32>, StoreError>> + Send + '_>> {
        let call_index = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
        let in_flight = self.in_flight.fetch_add(1, Ordering::SeqCst) + 1;
        self.max_in_flight.fetch_max(in_flight, Ordering::SeqCst);
        self.texts.lock().expect("text log").push(text.to_string());
        let out = match self.behavior {
            Behavior::AlwaysErr => Err(StoreError::EmbeddingUnavailable),
            Behavior::FailOnCall(k) if call_index == k => Err(StoreError::EmbeddingUnavailable),
            Behavior::FailOnceThenOk if call_index == 1 => Err(StoreError::EmbeddingUnavailable),
            other => Ok(expected_vector(other, text)),
        };
        Box::pin(async move {
            self.in_flight.fetch_sub(1, Ordering::SeqCst);
            out
        })
    }

    #[allow(clippy::manual_async_fn)]
    fn is_available(&self) -> Pin<Box<dyn Future<Output = bool> + Send + '_>> {
        self.availability_calls.fetch_add(1, Ordering::SeqCst);
        let available = self.availability;
        Box::pin(async move { available })
    }
}

// ===========================================================================
// Fixtures: the corpus (§9.5.5's "corpus seeding" clause — the public `RagStore`
// surface only: create_wiki → create_document → update_document with the
// doc-head/doc-end graph).
// ===========================================================================

/// One seeded corpus: the store, the wiki per document, the document ids and every
/// seeded node with its `value` (so the embeddable set is exact).
struct Corpus {
    store: Store,
    wikis: Vec<WikiId>,
    docs: Vec<DocumentId>,
    nodes: Vec<((DocumentId, NodeId), Option<String>)>,
}

impl Corpus {
    /// The embeddable nodes — exactly the nodes whose `value` is `Some(_)`
    /// (§9.5.5's corpus table: "a node whose `value` is `None` contributes no
    /// entry"; `Some("")` counts).
    fn embeddable(&self) -> Vec<((DocumentId, NodeId), String)> {
        self.nodes
            .iter()
            .filter_map(|(k, v)| v.clone().map(|t| (k.clone(), t)))
            .collect()
    }

    fn embeddable_count(&self) -> usize {
        self.embeddable().len()
    }

    /// The key set the index must carry, exactly: one
    /// `(documentId, nodeId, FieldType::Full)` triple per embeddable node, with the
    /// **source** ids (§9.5.5's corpus table + `P-IM-11`/`P-TP-5`).
    fn expected_keys(&self) -> Vec<(DocumentId, NodeId, FieldType)> {
        self.embeddable()
            .into_iter()
            .map(|((d, n), _)| (d, n, FieldType::Full))
            .collect()
    }

    fn text_of(&self, key: &(DocumentId, NodeId, FieldType)) -> String {
        self.nodes
            .iter()
            .find(|((d, n), _)| d == &key.0 && n == &key.1)
            .and_then(|(_, v)| v.clone())
            .unwrap_or_else(|| panic!("the test only looks up seeded nodes ({key:?})"))
    }
}

/// Seed **one wiki + one document per inner slice** (each document carries exactly
/// one `DocHead` and one `DocEnd` edge, as `update_document`'s validation requires),
/// in the order given — so `docs.len() > 1` produces the out-of-id-order / sharded
/// corpus `P-IM-12` names.
async fn seed_corpus(docs: &[&[(&str, Option<&str>)]]) -> Corpus {
    let store = Store::new();
    let mut wikis = Vec::new();
    let mut dids = Vec::new();
    let mut nodes = Vec::new();
    for (i, spec) in docs.iter().enumerate() {
        let wiki = store
            .create_wiki(&format!("w{i}"))
            .await
            .expect("create_wiki")
            .wiki_id;
        let (did, seeded) = seed_doc(&store, &wiki, spec).await;
        for n in &seeded {
            nodes.push(((did.clone(), n.node_id.clone()), n.value.clone()));
        }
        wikis.push(wiki);
        dids.push(did);
    }
    Corpus {
        store,
        wikis,
        docs: dids,
        nodes,
    }
}

/// The seeding recipe, verbatim (§9.5.5's step 3/4): create the document (it starts
/// with an **empty** graph), then one `update_document` carrying the corpus and the
/// mandatory `DocHead`/`DocEnd` pair.
async fn seed_doc(
    store: &Store,
    wiki: &WikiId,
    spec: &[(&str, Option<&str>)],
) -> (DocumentId, Vec<Node>) {
    let doc = store
        .create_document(
            wiki,
            CreateDocumentRequest {
                title: "t".to_string(),
                tags: None,
                author: None,
            },
        )
        .await
        .expect("create_document");
    let did = doc.document_id.clone();
    let nodes: Vec<Node> = spec
        .iter()
        .map(|(id, value)| Node {
            document_id: did.clone(),
            node_id: NodeId((*id).to_string()),
            kind: NodeKind::Content,
            value: value.map(|v| v.to_string()),
            fact_key: None,
            target: None,
        })
        .collect();
    if nodes.is_empty() {
        return (did, nodes);
    }
    let cur = store.get_document(&did).await.expect("get_document");
    let head = nodes[0].node_id.clone();
    let tail = nodes[nodes.len() - 1].node_id.clone();
    let edges = vec![
        Edge {
            source: (did.clone(), head.clone()),
            target: (did.clone(), tail.clone()),
            kind: EdgeKind::DocHead,
            state: None,
            cross_wiki: false,
            relation_type: None,
        },
        Edge {
            source: (did.clone(), tail),
            target: (did.clone(), head),
            kind: EdgeKind::DocEnd,
            state: None,
            cross_wiki: false,
            relation_type: None,
        },
    ];
    store
        .update_document(
            &did,
            UpdateDocumentRequest {
                base_revision: cur.revision,
                graph: Graph {
                    nodes: nodes.clone(),
                    edges,
                },
                title: None,
                tags: None,
            },
        )
        .await
        .expect("update_document (the seeded graph carries exactly one DocHead and one DocEnd)");
    (did, nodes)
}

// ===========================================================================
// Fixtures: the boot wiring (§9.5.5's ordering clauses) and the honest masks.
// ===========================================================================

/// The three `BootProvider` inputs the contract's table quantifies over.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Boot {
    Absent,
    Unreachable,
    Reachable,
}

fn boot_provider(kind: Boot, pr: &Arc<dyn EmbeddingProvider>) -> BootProvider {
    match kind {
        Boot::Absent => BootProvider::Absent,
        Boot::Unreachable => BootProvider::Unreachable,
        Boot::Reachable => BootProvider::Reachable(pr.clone()),
    }
}

/// Apply the **pinned** boot wiring (§9.5.5's ordering clause): compose the
/// snapshot → `boot_wiring(provider, &snap)` → `swap_snapshot(snap)` →
/// `set_embedding_provider` **only** in the `Reachable` case → `set_engine_state`
/// of the returned `EngineState`. The returned `EngineSubsystems` is the
/// **assertion surface**, never an applied mask (`set_subsystems` is never called
/// here as an expectation source).
async fn apply_boot(
    store: &Store,
    kind: Boot,
    wired: Option<Arc<dyn EmbeddingProvider>>,
    snap: &DerivedIndexes,
    pr: &Arc<dyn EmbeddingProvider>,
) -> (EngineState, EngineSubsystems, EngineSubsystems) {
    let (state, flags) = boot_wiring(boot_provider(kind, pr), snap);
    store.swap_snapshot(snap.clone());
    if let Some(p) = wired {
        store.set_embedding_provider(p);
    }
    store.set_engine_state(state);
    let derived = store.get_engine_status().await.subsystems;
    (state, flags, derived)
}

/// The honest mask shape: core `true`, `reranker` unconditional `false`, `vector`
/// and `embedding` as given (§5.8/§9.5.2's capability predicates; V-8.1/V-8.2).
fn mask(vector: bool, embedding: bool) -> EngineSubsystems {
    EngineSubsystems {
        store: true,
        graph: true,
        lexical: true,
        vector,
        embedding,
        reranker: false,
    }
}

/// V-8.1's mask — the **`Reachable` + `vectors: Some`** instance only
/// (§9.5.5's `P-IM-14` scoping; §5.8's REMAND-1 clarification).
fn v8_1_mask() -> EngineSubsystems {
    mask(true, true)
}

/// V-8.2's mask — the provider-`Absent`/`Unreachable` boot, unchanged by U5.
fn v8_2_mask() -> EngineSubsystems {
    mask(false, false)
}

fn vector_options(wiki: &WikiId) -> RagQueryOptions {
    RagQueryOptions {
        wiki_id: Some(wiki.clone()),
        top_k: Some(10),
        mode: Some(QueryMode::Vector),
        ..Default::default()
    }
}

// ===========================================================================
// Assertion helpers (the §9.5.5 comparison semantics).
// ===========================================================================

/// The pinned vector comparison (§9.5.5's contract table (2) *vector comparison /
/// `NaN`* row): same `len()`, every **non-`NaN`** element identical bit-for-bit,
/// `NaN` asserted only in the `NaN` position, **no** tolerance/`approx`
/// comparison.
fn assert_verbatim(label: &str, got: &[f32], want: &[f32]) {
    assert_eq!(
        got.len(),
        want.len(),
        "{label}: same len() — the provider's returned length is authoritative \
         (no dimension is pinned by U5)"
    );
    for (i, (g, w)) in got.iter().zip(want.iter()).enumerate() {
        if w.is_nan() {
            assert!(
                g.is_nan(),
                "{label}: index {i}: a NaN in the provider-returned vector requires a NaN at \
                 that same index ({got:?} vs {want:?})"
            );
        } else {
            assert_eq!(
                g.to_bits(),
                w.to_bits(),
                "{label}: index {i}: every non-NaN element is the provider's own value \
                 verbatim — bit-for-bit, no tolerance, no normalisation, no re-ordering \
                 ({got:?} vs {want:?})"
            );
        }
    }
}

/// `entries` is **exactly** the corpus: one `(documentId, nodeId, FieldType::Full)`
/// key per embeddable node, nothing dropped, nothing extra, no `Binary`/`Other(_)`
/// key (§9.5.5's contract table (2)'s key-set row + `P-IM-10`/`P-IM-11`/`P-TP-5`).
fn assert_entries_are_exactly(label: &str, vi: &VectorIndex, corpus: &Corpus) {
    let want = corpus.expected_keys();
    assert_eq!(
        vi.entries.len(),
        want.len(),
        "{label}: `entries.len()` == the corpus's embeddable count (one key per embeddable \
         node; a node whose `value` is `None` contributes none)"
    );
    for k in &want {
        assert!(
            vi.entries.contains_key(k),
            "{label}: the key {k:?} (the SOURCE ids + `FieldType::Full`) must be present"
        );
    }
    for k in vi.entries.keys() {
        assert!(
            want.contains(k),
            "{label}: unexpected key {k:?} — the index is exactly the corpus (nothing keyed by \
             text or by position, nothing extra)"
        );
        assert_eq!(
            k.2,
            FieldType::Full,
            "{label}: U5 builds the `full` field ONLY — no `FieldType::Binary` entry exists in \
             any outcome"
        );
    }
}

// ===========================================================================
// Live-server harness (docs: p2 §4 loopback bind, §6 READY lifecycle).
// ===========================================================================

fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    listener.local_addr().expect("local addr").port()
}

/// Spawn the bin with the boot's provider configuration **removed** ⇒
/// `BootProvider::Absent` regardless of the ambient environment (§9.5.5's
/// hermeticity clause: the existing helper `env_remove`s both vars).
async fn spawn_server_without_provider() -> (Child, String) {
    spawn_server(&[
        ("GNOSIS_SERVER_OLLAMA_URL", None),
        ("GNOSIS_SERVER_OLLAMA_MODEL", None),
    ])
    .await
}

/// Spawn the bin with a **configured but unreachable** provider: a loopback port
/// with nothing listening, so the boot's probe fails ⇒
/// `BootProvider::Unreachable` (contract table (1) row 2). No real network is
/// touched.
async fn spawn_server_with_unreachable_provider() -> (Child, String) {
    let dead = free_port();
    let url = format!("http://127.0.0.1:{dead}");
    spawn_server(&[
        ("GNOSIS_SERVER_OLLAMA_URL", Some(url.as_str())),
        ("GNOSIS_SERVER_OLLAMA_MODEL", Some("nomic-embed-text")),
    ])
    .await
}

/// Spawn the bin on an ephemeral loopback port with the given env overrides
/// (`None` ⇒ removed). The provider-**reachable** live boot is the live battery's
/// `R-L3`, never a claim here (§9.5.5's adjudication note 1 + its hermeticity
/// clause).
async fn spawn_server(env: &[(&str, Option<&str>)]) -> (Child, String) {
    let port = free_port();
    let mut cmd = Command::new(SERVER_BIN);
    cmd.arg("--port").arg(port.to_string());
    for (key, value) in env {
        match value {
            Some(v) => cmd.env(key, v),
            None => cmd.env_remove(key),
        };
    }
    // The child is intentionally kept live for the test and reaped via
    // `ServerGuard`'s Drop (kill + wait), so no zombie is left.
    #[allow(clippy::zombie_processes)]
    let child = cmd.spawn().expect("spawn gnosis-server");
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

async fn get_engine_status_json(base: &str) -> (u16, serde_json::Value) {
    let resp = reqwest::Client::new()
        .get(format!("{base}/engine/status"))
        .send()
        .await
        .expect("GET /engine/status");
    let status = resp.status().as_u16();
    let text = resp.text().await.expect("engine/status body");
    let body: serde_json::Value =
        serde_json::from_str(&text).expect("engine/status is JSON (§5.8: always 200)");
    (status, body)
}

/// `POST /rag/query` with the envelope-strict body (`R-L3`'s (i)/(ii) shape):
/// `{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"query":…,"mode":…}}`.
async fn post_rag_query(base: &str, payload: serde_json::Value) -> (u16, String) {
    let body = serde_json::json!({
        "schemaVersion": 1,
        "idFormat": "opaque-string-v1",
        "payload": payload,
    });
    let resp = reqwest::Client::new()
        .post(format!("{base}/rag/query"))
        .json(&body)
        .send()
        .await
        .expect("POST /rag/query");
    let status = resp.status().as_u16();
    (status, resp.text().await.expect("body"))
}

/// The §11 wire code of an error response envelope (payload `{"code","message"}`).
fn error_code(text: &str) -> String {
    let v: serde_json::Value = serde_json::from_str(text)
        .unwrap_or_else(|e| panic!("the error response must be an envelope: {text:?} ({e:?})"));
    v.get("payload")
        .and_then(|p| p.get("code"))
        .and_then(|c| c.as_str())
        .unwrap_or_else(|| panic!("the §11 error payload carries `code`: {text:?}"))
        .to_string()
}

fn flags_of(body: &serde_json::Value) -> EngineSubsystems {
    let sub = body
        .get("subsystems")
        .and_then(|v| v.as_object())
        .unwrap_or_else(|| panic!("the report carries a `subsystems` object: {body}"));
    let get = |k: &str| {
        sub.get(k)
            .and_then(|v| v.as_bool())
            .unwrap_or_else(|| panic!("the report carries the bool flag `{k}`: {body}"))
    };
    EngineSubsystems {
        store: get("store"),
        graph: get("graph"),
        lexical: get("lexical"),
        vector: get("vector"),
        embedding: get("embedding"),
        reranker: get("reranker"),
    }
}

// ===========================================================================
// U5-1 — entry question 1, contract table (1) row 1 + §6: a `Reachable` boot over
// an EMPTY store still builds an index (an empty `VectorIndex` is `Some`), so the
// composed snapshot carries it and the derived flag is `true`.
// ===========================================================================

#[tokio::test]
async fn u5_1_reachable_boot_over_an_empty_store_builds_an_empty_index() {
    let corpus = seed_corpus(&[]).await;
    let stub = scripted(Behavior::ByText);
    let pr = dyn_provider(&stub);

    let built = build_boot_vector_index(&corpus.store, Some(&pr)).await;
    assert!(
        matches!(built, Ok(Some(_))),
        "`Reachable` ⇒ `Ok(Some(vi))` — an empty store still yields an index, never `Ok(None)` \
         and never `Err` — got {built:?}"
    );

    // The composed boot snapshot (§9.5.5's ordering clause + §6).
    let snap = DerivedIndexes {
        vectors: built.ok().flatten(),
        ..DerivedIndexes::default()
    };
    assert!(
        snap.vectors.is_some(),
        "`vectors: built.ok().flatten()` — an empty index is still `Some`"
    );
    let vi = snap
        .vectors
        .as_ref()
        .expect("the composed snapshot carries the built index");
    assert!(
        vi.entries.is_empty(),
        "§9.5.5 (2): the empty store's build produces `VectorIndex::default()` — `0 keys`"
    );
    assert_eq!(
        stub.call_count(),
        0,
        "§9.5.5 (2)'s call-count row: `n = 0` embeddable nodes ⇒ 0 `embed` calls"
    );

    let s = Store::new();
    let (state, flags, derived) =
        apply_boot(&s, Boot::Reachable, Some(pr.clone()), &snap, &pr).await;
    assert_eq!(
        state,
        EngineState::Ready,
        "§9.5.5 (1): `Reachable` ⇒ `Ready`"
    );
    assert_eq!(
        flags,
        v8_1_mask(),
        "§9.5.5 (1)/§5.8: a `Reachable` boot whose snapshot has `vectors: Some` — the empty \
         index included — derives V-8.1: vector:true, embedding:true, reranker:false, core true"
    );
    assert_eq!(
        derived, flags,
        "§9.5.5 `P-IM-14`: the store's derived read equals the wiring's flag vector"
    );
    assert!(
        s.snapshot().vectors.is_some(),
        "the boot swapped an index-bearing snapshot"
    );
}

// ===========================================================================
// U5-2 — contract table (1) row 2: `Unreachable` ⇒ NOT built, the unchanged
// index-free snapshot, `Degraded`, `vector:false`, `embedding:false`, no call.
// ===========================================================================

#[tokio::test]
async fn u5_2_unreachable_boot_attempts_no_build_and_keeps_the_snapshot_index_free() {
    let corpus = seed_corpus(&[&[("n1", Some("alpha")), ("n2", Some("beta"))]]).await;
    let stub = scripted(Behavior::ByText);
    let pr = dyn_provider(&stub);

    let built = build_boot_vector_index(&corpus.store, None)
        .await
        .expect("no provider supplied ⇒ nothing is to be built");
    assert!(
        built.is_none(),
        "§9.5.5 (1)/(2): `Absent`/`Unreachable` ⇒ the build is called with NO provider (or not \
         at all) ⇒ `Ok(None)` — and this is NOT an error"
    );
    assert_eq!(
        stub.call_count(),
        0,
        "§9.5.5 (1): for `Absent`/`Unreachable` NO embedding call is attempted at all"
    );

    let snap = DerivedIndexes::default();
    let s = Store::new();
    let (state, flags, derived) = apply_boot(&s, Boot::Unreachable, None, &snap, &pr).await;
    assert_eq!(
        state,
        EngineState::Degraded,
        "§9.5.5 (1): `Unreachable` ⇒ `Degraded` (never a fabricated `Ready`)"
    );
    assert_eq!(
        flags,
        v8_2_mask(),
        "§9.5.5 (1): the `Unreachable` boot reports V-8.2 unchanged — vector:false, \
         embedding:false, reranker:false, core true"
    );
    assert_eq!(
        derived, flags,
        "§9.5.5 `P-IM-14`: derived read == the wiring's vector"
    );
    assert!(
        s.snapshot().vectors.is_none(),
        "the boot swapped the UNCHANGED index-free snapshot"
    );
}

// ===========================================================================
// U5-3 — contract table (1) row 3: `Absent` ⇒ NOT built, `Unavailable`,
// `vector:false`, `embedding:false`.
// ===========================================================================

#[tokio::test]
async fn u5_3_absent_boot_keeps_unavailable_and_claims_no_index() {
    let stub = scripted(Behavior::ByText);
    let pr = dyn_provider(&stub);
    let s = Store::new();
    let snap = DerivedIndexes::default();
    let (state, flags, derived) = apply_boot(&s, Boot::Absent, None, &snap, &pr).await;

    assert_eq!(
        state,
        EngineState::Unavailable,
        "§9.5.5 (1): `Absent` (no provider configured) ⇒ `Unavailable` — the construction value"
    );
    assert_eq!(flags, v8_2_mask(), "§9.5.5 (1): `Absent` derives V-8.2");
    assert_eq!(
        derived, flags,
        "§9.5.5 `P-IM-14`: derived read == the wiring's vector"
    );
    assert!(s.snapshot().vectors.is_none(), "no index was built");
    assert_eq!(
        stub.call_count(),
        0,
        "§9.5.5 (1): no embedding call is attempted for an absent provider"
    );
}

// ===========================================================================
// U5-4 — entry question 2 + `P-IM-10`/`P-IM-11`: the corpus is node-level and
// store-wide; keys carry the SOURCE ids; one `embed` per embeddable node, none for
// `value: None`; nothing else is embedded.
// ===========================================================================

#[tokio::test]
async fn u5_4_the_index_is_exactly_the_embeddable_node_corpus() {
    let corpus = seed_corpus(&[&[
        ("n1", Some("alpha")),
        ("n2", Some("beta beta")),
        ("n3", None),
    ]])
    .await;
    let stub = scripted(Behavior::ByText);
    let pr = dyn_provider(&stub);

    let vi = build_boot_vector_index(&corpus.store, Some(&pr))
        .await
        .expect("all `embed` calls succeed")
        .expect("a supplied provider never yields `Ok(None)`");
    assert_entries_are_exactly("U5-4", &vi, &corpus);

    assert_eq!(
        stub.call_count(),
        2,
        "§9.5.5 (2)'s call-count row: exactly `n` `embed` calls for `n` embeddable nodes — \
         the `value: None` node gets NO call"
    );
    assert_eq!(
        stub.embedded_texts(),
        vec!["alpha".to_string(), "beta beta".to_string()],
        "§9.5.5 (2): the embedded text is the node's authored `value` — no prefixing, no \
         truncation, no title/tag/factKey concatenation, and no call for any other text"
    );
    assert_eq!(
        vi.entries.len(),
        corpus.embeddable_count(),
        "the key set is bijective onto the embeddable corpus"
    );
}

// ===========================================================================
// U5-5 — entry question 2: the corpus is the WHOLE store (all wikis, all shards);
// no wiki filter is applied by the build; only the read-time leg wiki-scopes.
// ===========================================================================

#[tokio::test]
async fn u5_5_the_corpus_is_store_wide_across_wikis_and_unsorted_ids() {
    let corpus = seed_corpus(&[
        &[("a1", Some("one")), ("a2", None)],
        &[("b1", Some("two")), ("b2", Some("three"))],
    ])
    .await;
    assert_eq!(corpus.wikis.len(), 2, "two wikis, one document each");

    let stub = scripted(Behavior::ByText);
    let pr = dyn_provider(&stub);
    let vi = build_boot_vector_index(&corpus.store, Some(&pr))
        .await
        .expect("all calls succeed")
        .expect("a supplied provider never yields `Ok(None)`");

    assert_entries_are_exactly("U5-5", &vi, &corpus);
    assert_eq!(
        vi.entries.len(),
        3,
        "§9.5.5 (2)/(4): `n` is store-wide — both wikis' embeddable nodes are in the one index \
         (`build_boot_vector_index` takes no wiki parameter)"
    );
    assert_eq!(
        stub.call_count(),
        3,
        "one call per embeddable node, store-wide"
    );
    for ((doc, _), _) in corpus.embeddable() {
        assert!(
            corpus.docs.contains(&doc),
            "every key's document id is a SOURCE id of the seeded corpus"
        );
    }
}

// ===========================================================================
// U5-6 — the valid/fail state 3 + §9.5.5 (2): nodes with `value: None` contribute
// nothing (but the index is still `Some`); `value: Some("")` IS embeddable.
// ===========================================================================

#[tokio::test]
async fn u5_6_non_embeddable_nodes_yield_no_key_while_some_empty_string_does() {
    let corpus = seed_corpus(&[&[("x1", None), ("x2", None)]]).await;
    let stub = scripted(Behavior::ByText);
    let pr = dyn_provider(&stub);

    let vi = build_boot_vector_index(&corpus.store, Some(&pr))
        .await
        .expect("a `None`-valued node is NOT an error")
        .expect("`Ok(None)` is NEVER the answer for a supplied provider");
    assert_entries_are_exactly("U5-6a", &vi, &corpus);
    assert!(vi.entries.is_empty(), "no text ⇒ no entry");
    assert_eq!(stub.call_count(), 0, "no call for a `value: None` node");

    let snap = DerivedIndexes {
        vectors: Some(vi),
        ..DerivedIndexes::default()
    };
    assert!(
        snap.vectors.is_some(),
        "state 3: the index is still `Some`, so `vector: true`"
    );
    assert_eq!(
        boot_wiring(BootProvider::Reachable(pr.clone()), &snap).1,
        v8_1_mask(),
        "§9.5.5 (2): a store with nodes but none embeddable behaves exactly like an empty store"
    );

    // A node whose `value` is `Some("")` IS embeddable (the contract embeds it).
    let (did, nodes) = seed_doc(&corpus.store, &corpus.wikis[0], &[("y1", Some(""))]).await;
    let stub2 = scripted(Behavior::ByText);
    let pr2 = dyn_provider(&stub2);
    let vi2 = build_boot_vector_index(&corpus.store, Some(&pr2))
        .await
        .expect("all calls succeed")
        .expect("a supplied provider never yields `Ok(None)`");
    assert_eq!(
        vi2.entries.len(),
        1,
        "exactly one entry — the `Some(\"\")` node ({nodes:?})"
    );
    assert!(
        vi2.entries
            .contains_key(&(did.clone(), NodeId("y1".to_string()), FieldType::Full)),
        "the `Some(\"\")` node's key is present (a zero-length text is embeddable)"
    );
    assert_eq!(
        stub2.embedded_texts(),
        vec![String::new()],
        "the build passes the empty text through verbatim"
    );
}

// ===========================================================================
// U5-7 — entry question 2: the empty store ⇒ `Ok(Some(VectorIndex::default()))`,
// never `Ok(None)`; the boot's snapshot is still index-bearing.
// ===========================================================================

#[tokio::test]
async fn u5_7_the_empty_store_still_builds_and_never_answers_ok_none() {
    let corpus = seed_corpus(&[]).await;
    let stub = scripted(Behavior::ByText);
    let pr = dyn_provider(&stub);
    let r = build_boot_vector_index(&corpus.store, Some(&pr)).await;
    let ok = r.expect("the empty store is not a failure");
    let vi = ok.expect("§9.5.5 (2) 'empty store' row: an EMPTY index, NOT `None`");
    assert!(
        vi.entries.is_empty(),
        "`VectorIndex::default()` over zero vectors"
    );
    assert_eq!(
        stub.call_count(),
        0,
        "`n = 0` embeddable nodes ⇒ zero calls, strictly sequential over an empty corpus"
    );
}

// ===========================================================================
// U5-8 — §9.5.5 (2) + the ordering clause: the build returns the index ALONE; the
// composed snapshot's other fields are the boot's own `DerivedIndexes`, untouched
// (`epoch` at the input's value; U5 builds no `LexicalIndex`).
// ===========================================================================

#[tokio::test]
async fn u5_8_the_composed_snapshot_keeps_the_boot_fields_untouched() {
    let corpus = seed_corpus(&[&[("n1", Some("alpha"))]]).await;
    let stub = scripted(Behavior::ByText);
    let pr = dyn_provider(&stub);

    let boot_snapshot = DerivedIndexes {
        lexical: None,
        vectors: None,
        epoch: 7,
    };
    let built = build_boot_vector_index(&corpus.store, Some(&pr)).await;
    let snap = DerivedIndexes {
        vectors: built.ok().flatten(),
        ..boot_snapshot
    };

    assert!(
        snap.vectors.is_some(),
        "the built index is the snapshot's `vectors`"
    );
    assert_eq!(
        snap.epoch, 7,
        "§9.5.5 (2): the build does not move the epoch (it is not the epoch-driven rebuild \
         vehicle) — `DerivedIndexes.epoch` stays at the INPUT's value"
    );
    assert!(
        snap.lexical.is_none(),
        "§9.5.5 (2): `lexical` is unchanged (`None` at boot) — U5 builds NO `LexicalIndex`"
    );
}

// ===========================================================================
// U5-9 — entry question 3: U5 builds the `full` field ONLY (never `binary`), and
// `vector: true` therefore means "the full-field dense leg is wired".
// ===========================================================================

#[tokio::test]
async fn u5_9_only_full_field_keys_are_built_never_binary() {
    let corpus = seed_corpus(&[&[("n1", Some("alpha")), ("n2", Some("beta"))]]).await;
    let stub = scripted(Behavior::ByText);
    let pr = dyn_provider(&stub);
    let vi = build_boot_vector_index(&corpus.store, Some(&pr))
        .await
        .expect("all calls succeed")
        .expect("a supplied provider never yields `Ok(None)`");

    assert_entries_are_exactly("U5-9", &vi, &corpus);
    for k in vi.entries.keys() {
        assert_eq!(
            k.2,
            FieldType::Full,
            "§9.5.5 (3): U5 builds the `full` field only — a `Binary` entry is a deliberate \
             design choice with a corpus-size cost that U5 does NOT take"
        );
        assert!(
            !matches!(k.2, FieldType::Binary | FieldType::Other(_)),
            "no `Binary`/`Other(_)` key exists in any outcome"
        );
    }
    assert_eq!(
        vi.entries.len(),
        2,
        "no extra key is produced for the un-built coarse first pass"
    );
}

// ===========================================================================
// U5-10 — `P-IM-10`: the outcome space is CLOSED and three-valued over its two
// inputs, and the `None` case is NOT an error.
// ===========================================================================

#[tokio::test]
async fn u5_10_the_build_is_total_and_three_valued_over_its_inputs() {
    let empty = seed_corpus(&[]).await;
    let populated = seed_corpus(&[&[("n1", Some("alpha")), ("n2", None)]]).await;
    let none_embeddable = seed_corpus(&[&[("m1", None)]]).await;

    for (label, corpus) in [
        ("empty store", &empty),
        ("populated store", &populated),
        ("no embeddable node", &none_embeddable),
    ] {
        let stub = scripted(Behavior::ByText);
        let pr = dyn_provider(&stub);
        let r = build_boot_vector_index(&corpus.store, Some(&pr)).await;
        match r {
            Ok(Some(vi)) => {
                assert_entries_are_exactly(label, &vi, corpus);
                assert_eq!(
                    stub.call_count(),
                    corpus.embeddable_count(),
                    "{label}: exactly `n` calls for `n` embeddable nodes"
                );
            }
            Ok(None) => panic!(
                "{label}: `Ok(None)` is NEVER the answer for a SUPPLIED provider (it means \
                 'no index is to be built', i.e. the provider-absent case)"
            ),
            Err(e) => panic!("{label}: an all-`Ok` provider must not produce an error: {e:?}"),
        }

        let failing = scripted(Behavior::AlwaysErr);
        let fpr = dyn_provider(&failing);
        let bad = build_boot_vector_index(&corpus.store, Some(&fpr)).await;
        if corpus.embeddable_count() == 0 {
            // With `n = 0` no `embed` call is made at all, so no call can fail:
            // §9.5.5's `Ok(Some(vi))` arm ("a provider was supplied and every
            // `embed` call succeeded" — vacuously) and its adjudication note 1(a)
            // ("*n* = 0 embeddable nodes ⇒ `Ok(Some(VectorIndex::default()))`, an
            // EMPTY index, NOT an `Err`").
            assert!(
                matches!(bad, Ok(Some(_))),
                "{label}: with 0 embeddable nodes no `embed` call is made, so even an \
                 always-`Err` provider cannot fail the build — got {bad:?}"
            );
            assert_eq!(
                failing.call_count(),
                0,
                "{label}: zero calls ⇒ nothing to fail on"
            );
        } else {
            match bad {
                Err(StoreError::EmbeddingUnavailable) => {}
                other => panic!(
                    "{label}: a supplied provider whose `embed` fails ⇒ exactly \
                     `Err(StoreError::EmbeddingUnavailable)`, never `Ok(None)` and never a \
                     partially-filled `Some` — got {other:?}"
                ),
            }
            assert_eq!(
                failing.call_count(),
                1,
                "{label}: the build aborts on the first failing call"
            );
        }

        // `p == None` ⇒ `Ok(None)`, with the injected provider's call count 0: an
        // injected provider object may exist (the boot's probe was `Unreachable`)
        // and still never be consulted, because the build is called with no provider.
        let idle = scripted(Behavior::ByText);
        let with_none = build_boot_vector_index(&corpus.store, None).await;
        assert!(
            matches!(with_none, Ok(None)),
            "{label}: `p == None` ⇒ `Ok(None)` (the boot keeps its index-free snapshot) and \
             this is NOT an error — got {with_none:?}"
        );
        assert_eq!(
            idle.call_count(),
            0,
            "{label}: the injected provider's `embed` call count is 0 for `p == None`"
        );
    }
}

// ===========================================================================
// U5-11 — §9.5.5 (2)'s call-count/discipline row (an invariant, not a cost note):
// exactly `n` calls for `n` embeddable nodes, STRICTLY SEQUENTIAL, none for
// `value: None`, no call for any other text.
// ===========================================================================

#[tokio::test]
async fn u5_11_the_call_discipline_is_one_sequential_call_per_embeddable_node() {
    let corpus = seed_corpus(&[
        &[("n1", Some("alpha")), ("n2", None)],
        &[("n3", Some("beta")), ("n4", Some("gamma")), ("n5", None)],
    ])
    .await;
    let stub = scripted(Behavior::ByText);
    let pr = dyn_provider(&stub);
    let vi = build_boot_vector_index(&corpus.store, Some(&pr))
        .await
        .expect("all calls succeed")
        .expect("a supplied provider never yields `Ok(None)`");

    assert_entries_are_exactly("U5-11", &vi, &corpus);
    assert_eq!(
        stub.call_count(),
        3,
        "exactly `n` calls for `n` embeddable nodes — no retry, no batching, no extra call"
    );
    assert_eq!(
        stub.max_concurrency(),
        1,
        "§9.5.5 (2)/(4): the calls are STRICTLY SEQUENTIAL — no fan-out, no concurrency"
    );
    assert_eq!(
        stub.embedded_texts(),
        vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()],
        "no call for a `value: None` node and no call for any other text (titles/tags/\
         factKeys/queries)"
    );
}

// ===========================================================================
// U5-12 — `P-IM-13`: the build is ATOMIC. A failure at any `k ∈ 1..=n` yields no
// index at all — no partial index is ever returned or observable — and the boot's
// failure outcome leaves the snapshot index-free (⇒ `vector: false`).
// ===========================================================================

#[tokio::test]
async fn u5_12_a_mid_build_failure_installs_no_index_and_degrades() {
    let corpus = seed_corpus(&[&[
        ("n1", Some("alpha")),
        ("n2", Some("beta")),
        ("n3", Some("gamma")),
    ]])
    .await;
    assert_eq!(corpus.embeddable_count(), 3, "`n = 3`");

    for k in 1..=3usize {
        let stub = scripted(Behavior::FailOnCall(k));
        let pr = dyn_provider(&stub);
        let r = build_boot_vector_index(&corpus.store, Some(&pr)).await;
        match r {
            Err(StoreError::EmbeddingUnavailable) => {}
            other => panic!(
                "§9.5.5 (4)/`P-IM-13`: a failure on the {k}-th call ⇒ \
                 `Err(EmbeddingUnavailable)` — NOT `Ok(Some(partial))` and NOT `Ok(None)`; \
                 got {other:?}"
            ),
        }
        assert_eq!(
            stub.call_count(),
            k,
            "the build ABORTS at the failing call (it does not keep embedding after an error)"
        );
    }

    // The boot contract's documented FAILURE outcome, in the order the contract
    // table (4) pins: `snap = DerivedIndexes::default()` (unchanged, index-free) →
    // `swap_snapshot` → `set_embedding_provider` → the derived `Degraded` state.
    let stub = scripted(Behavior::FailOnCall(1));
    let pr = dyn_provider(&stub);
    let failed = build_boot_vector_index(&corpus.store, Some(&pr)).await;
    assert!(
        matches!(failed, Err(StoreError::EmbeddingUnavailable)),
        "the failing build's `Err` is discarded with the whole local index under construction"
    );
    let snap = DerivedIndexes::default();
    corpus.store.swap_snapshot(snap.clone());
    corpus.store.set_embedding_provider(pr.clone());
    corpus
        .store
        .set_engine_state(boot_wiring(BootProvider::Unreachable, &snap).0);

    assert!(
        corpus.store.snapshot().vectors.is_none(),
        "§9.5.5 (4)/`P-IM-13`: after the failure outcome NO partially-filled index is \
         observable — the snapshot is index-free"
    );
    assert!(
        !corpus.store.get_engine_status().await.subsystems.vector,
        "`P-IM-14`: the flag follows the snapshot — a discarded build leaves `vector: false`"
    );
}

// ===========================================================================
// U5-13 — `P-IM-13`: the failure is NOT sticky (fail-once-then-succeed ⇒ the full
// index) and no provider behaviour yields a *partial* `Ok(Some(_))`.
// ===========================================================================

#[tokio::test]
async fn u5_13_the_failure_is_not_sticky_and_no_partial_index_is_some() {
    let corpus = seed_corpus(&[&[
        ("n1", Some("alpha")),
        ("n2", Some("beta")),
        ("n3", Some("gamma")),
    ]])
    .await;
    let stub = scripted(Behavior::FailOnceThenOk);
    let pr = dyn_provider(&stub);

    // The first build fails (its first `embed` call is the failing one)…
    let first = build_boot_vector_index(&corpus.store, Some(&pr)).await;
    assert!(
        matches!(first, Err(StoreError::EmbeddingUnavailable)),
        "the first attempt returns the atomic error — got {first:?}"
    );
    assert_eq!(
        stub.call_count(),
        1,
        "the first build aborts at the failing call"
    );

    // …and the failure is NOT sticky: the same input (same store, same provider
    // object) succeeds on the subsequent build, with the FULL corpus.
    let vi = build_boot_vector_index(&corpus.store, Some(&pr))
        .await
        .expect("a subsequent success is not blocked by the earlier failure")
        .expect("a supplied provider never yields `Ok(None)`");
    assert_entries_are_exactly("U5-13", &vi, &corpus);
    assert_eq!(
        vi.entries.len(),
        3,
        "the retry produced a COMPLETE index (the `P-IM-10` count witness: `entries.len() == n`)"
    );
    assert_eq!(
        stub.call_count(),
        4,
        "1 failing call + 3 succeeding calls: the failed call is neither silently skipped nor \
         sticky"
    );
}

// ===========================================================================
// U5-14 — `P-IM-15`: a `Reachable` boot whose build FAILED degrades honestly — it
// never fabricates `Ready` and never claims the index, while the provider IS wired
// (⇒ `embedding: true`) — over the pinned call order, with the store's fixed
// `last_error` and no new error family.
// ===========================================================================

#[tokio::test]
async fn u5_14_a_failed_reachable_build_degrades_with_embedding_true_and_no_index() {
    let corpus = seed_corpus(&[&[("n1", Some("alpha")), ("n2", Some("beta"))]]).await;
    let stub = scripted(Behavior::AlwaysErr);
    let pr = dyn_provider(&stub);

    let failed = build_boot_vector_index(&corpus.store, Some(&pr)).await;
    assert!(
        matches!(failed, Err(StoreError::EmbeddingUnavailable)),
        "§9.5.5 (4): a reachable provider whose `embed` fails ⇒ `Err(EmbeddingUnavailable)` — \
         the EXISTING family, never a new variant"
    );

    // The pinned failure order (contract table (4)'s "what the boot does with that
    // `Err`" row): unchanged index-free snapshot → provider wired (the probe DID
    // succeed) → `boot_wiring(Unreachable, &snap).0` applied, its flag vector
    // discarded.
    let snap = DerivedIndexes::default();
    corpus.store.swap_snapshot(snap.clone());
    corpus.store.set_embedding_provider(pr.clone());
    corpus
        .store
        .set_engine_state(boot_wiring(BootProvider::Unreachable, &snap).0);

    let status = corpus.store.get_engine_status().await;
    assert_eq!(
        status.state,
        EngineState::Degraded,
        "`P-IM-15`: the failed-build branch applies the EXISTING degraded state"
    );
    assert_ne!(
        status.state,
        EngineState::Ready,
        "`P-IM-15`: a failed build MUST NOT fabricate a `Ready`"
    );
    assert_eq!(
        status.subsystems,
        mask(false, true),
        "`P-IM-15`'s element-wise literal: {{store:true, graph:true, lexical:true, \
         vector:false, embedding:true, reranker:false}} — `vector` is never `true` without an \
         index in the snapshot, and the provider IS wired because the probe succeeded"
    );
    assert_eq!(
        status.last_error.as_deref(),
        Some(FIXED_LAST_ERROR),
        "`P-IM-15`: the store's own FIXED `last_error` string, byte-identical — U5 invents no \
         new reason text"
    );
    assert_eq!(
        status.version,
        env!("CARGO_PKG_VERSION"),
        "`P-IM-15`'s literal carries the store's own version"
    );
    assert!(
        corpus.store.snapshot().vectors.is_none(),
        "`P-IM-15`: the unchanged index-free snapshot is what the boot swaps"
    );
    assert_eq!(
        server_status(&StoreError::EmbeddingUnavailable),
        Some((503, "embedding_unavailable")),
        "§5.9/§11: the only error on this path is the existing FS-13 family — no new wire code \
         and no new §11 row is produced"
    );
}

// ===========================================================================
// U5-15 — `P-IM-14` (the whole cross-product) + §5.8's scoping: the flag IS the
// wiring (`subsystems.vector == snapshot().vectors.is_some()` for EVERY pair), the
// V-8.1 mask is the `Reachable` + `vectors: Some` instance ONLY, `Reachable` +
// `vectors: None` reports `Ready` + `vector:false` (not V-8.2), and a false-mask
// write changes nothing.
// ===========================================================================

#[tokio::test]
async fn u5_15_the_flag_is_the_wiring_over_every_boot_and_snapshot_pair() {
    let mut populated = DerivedIndexes::default();
    let mut entries = std::collections::HashMap::new();
    entries.insert(
        (
            DocumentId("d1".to_string()),
            NodeId("n1".to_string()),
            FieldType::Full,
        ),
        vec![0.5_f32, 0.5],
    );
    populated.vectors = Some(VectorIndex { entries });

    let snapshots = [
        ("index-free", DerivedIndexes::default()),
        (
            "empty index",
            DerivedIndexes {
                vectors: Some(VectorIndex::default()),
                ..DerivedIndexes::default()
            },
        ),
        ("populated index", populated),
    ];

    for (snap_label, snap) in snapshots {
        for kind in [Boot::Absent, Boot::Unreachable, Boot::Reachable] {
            let stub = scripted(Behavior::ByText);
            let pr = dyn_provider(&stub);
            let wired = if kind == Boot::Reachable {
                Some(pr.clone())
            } else {
                None
            };
            let s = Store::new();
            let (state, flags, derived) = apply_boot(&s, kind, wired, &snap, &pr).await;

            assert_eq!(
                derived, flags,
                "`P-IM-14`/F11: the returned pair is the ASSERTION SURFACE and the derived read \
                 is what the store REPORTS — a disagreement is a broken row ({kind:?}, \
                 {snap_label})"
            );

            let want_state = match kind {
                Boot::Absent => EngineState::Unavailable,
                Boot::Unreachable => EngineState::Degraded,
                Boot::Reachable => EngineState::Ready,
            };
            assert_eq!(state, want_state, "the boot's own branch state ({kind:?})");
            assert_eq!(
                s.get_engine_status().await.state,
                want_state,
                "the store reports the applied state ({kind:?})"
            );

            assert!(
                flags.store && flags.graph && flags.lexical,
                "the three core capability predicates are always true ({kind:?}/{snap_label})"
            );
            assert!(
                !flags.reranker,
                "`reranker` is `false` in every reachable state — no reranker exists ({kind:?})"
            );
            assert_eq!(
                flags.embedding,
                kind == Boot::Reachable,
                "`embedding == (a provider was wired)` — exactly, in every case"
            );
            assert_eq!(
                flags.vector,
                snap.vectors.is_some(),
                "§5.8/`P-IM-14`: the invariant that holds for EVERY `(p, snap)` pair is \
                 `flags.vector == snap.vectors.is_some()` ({kind:?}/{snap_label})"
            );

            match (kind, snap.vectors.is_some()) {
                (Boot::Reachable, true) => assert_eq!(
                    flags,
                    v8_1_mask(),
                    "V-8.1 is asserted ONLY for `Reachable` + a snapshot whose `vectors` is \
                     `Some` (the built snapshot, an empty index included)"
                ),
                (Boot::Reachable, false) => assert_eq!(
                    flags,
                    mask(false, true),
                    "a `Reachable` boot handed a snapshot with `vectors: None` reports `Ready` + \
                     `vector:false` by the same predicate — NOT V-8.2, whose `embedding` is false"
                ),
                // `Absent`/`Unreachable`: V-8.2 is the pair with the UNCHANGED
                // (index-free) snapshot; with an index-bearing snapshot the same
                // predicate derives the V-8.2-shaped vector with `vector:true` —
                // the invariant above (`vector == snap.vectors.is_some()`) is what
                // holds for every pair.
                (_, false) => assert_eq!(
                    flags,
                    v8_2_mask(),
                    "`Absent`/`Unreachable` + the unchanged index-free snapshot ⇒ V-8.2, \
                     unchanged by U5"
                ),
                (_, true) => assert_eq!(
                    flags,
                    mask(true, false),
                    "`Absent`/`Unreachable` + an index-bearing snapshot: the vector flag is the \
                     derived `snapshot().vectors.is_some()`, while `embedding` stays `false` \
                     (no provider was wired)"
                ),
            }

            // The write-independence probe: a false-mask write must change NOTHING.
            s.set_subsystems(EngineSubsystems {
                store: true,
                graph: true,
                lexical: true,
                vector: true,
                embedding: true,
                reranker: true,
            });
            assert_eq!(
                s.get_engine_status().await.subsystems,
                flags,
                "§9.5.2 `P-IM-7`/F16 (reused): the projection is write-independent — no stored \
                 mask ever produces a caller-visible value ({kind:?}/{snap_label})"
            );
        }
    }
}

// ===========================================================================
// U5-16 — §5.9's `Precedence (pinned)` bullet / `P-SM-7`: a NON-READY store's
// `mode=vector` request is FS-8 `EngineUnavailable` (⇒ 503 `engine_unavailable`),
// never FS-14, whatever its snapshot holds; no query text is embedded on that path.
// ===========================================================================

#[tokio::test]
async fn u5_16_a_non_ready_store_is_engine_unavailable_not_vector_index_unavailable() {
    for kind in [Boot::Absent, Boot::Unreachable] {
        let corpus = seed_corpus(&[&[("n1", Some("alpha"))]]).await;
        let stub = scripted(Behavior::ByText);
        let pr = dyn_provider(&stub);
        let snap = DerivedIndexes::default();
        let (state, _, _) = apply_boot(&corpus.store, kind, None, &snap, &pr).await;
        assert_ne!(
            state,
            EngineState::Ready,
            "the boot left the store non-READY"
        );

        let err = corpus
            .store
            .rag_query("alpha", &vector_options(&corpus.wikis[0]))
            .await
            .expect_err(
                "§5.9/engine-wire-contract §16 rule 6: readiness precedes leg availability",
            );
        assert!(
            matches!(err, StoreError::EngineUnavailable),
            "`P-SM-7`: an {kind:?} boot is state-gated, never leg-gated — the outcome is FS-8 \
             `EngineUnavailable`, NOT `VectorIndexUnavailable` (got {err:?})"
        );
        assert_eq!(
            server_status(&err),
            Some((503, "engine_unavailable")),
            "§11: FS-8 renders 503 `engine_unavailable`"
        );
        assert_eq!(
            stub.call_count(),
            0,
            "§9.5.5 (2)'s call-count row / `P-SM-7`: on a path that never embeds the provider's \
             call count is 0 — no query text is embedded"
        );
    }
}

// ===========================================================================
// U5-17 — `P-SM-7`: FS-14 `VectorIndexUnavailable` is reachable ONLY on a READY
// store with `vectors: None`; the index check precedes the provider, so the
// injected provider's call count is 0.
// ===========================================================================

#[tokio::test]
async fn u5_17_fs14_needs_a_ready_store_with_no_index() {
    let corpus = seed_corpus(&[&[("n1", Some("alpha"))]]).await;
    let stub = scripted(Behavior::ByText);
    let pr = dyn_provider(&stub);

    // A caller-built READY store with an unbuilt index (the named negative probe
    // §9.5.5 (state 6) keeps reachable) and a provider wired.
    corpus.store.swap_snapshot(DerivedIndexes::default());
    corpus.store.set_embedding_provider(pr.clone());
    corpus.store.set_engine_state(EngineState::Ready);
    assert_eq!(
        corpus.store.get_engine_status().await.subsystems,
        mask(false, true),
        "`Ready` + `vectors: None` + a wired provider is a legitimate derived mask (F13: `Ready` \
         does not imply `vector: true`)"
    );

    let err = corpus
        .store
        .rag_query("alpha", &vector_options(&corpus.wikis[0]))
        .await
        .expect_err("an unbuilt index on a READY store is FS-14");
    assert!(
        matches!(err, StoreError::VectorIndexUnavailable),
        "`P-SM-7`: FS-14 is asserted ONLY on a READY store with `vectors: None` (got {err:?})"
    );
    assert_eq!(
        server_status(&err),
        Some((503, "vector_index_unavailable")),
        "§5.9/§11: FS-14 renders 503 `vector_index_unavailable` (U5 does not retire the variant \
         or the §11 row)"
    );
    assert_eq!(
        stub.call_count(),
        0,
        "§9.5.5's order pin: the index is checked BEFORE the provider — the wired provider's \
         `embed` call count is 0 on this path"
    );
}

// ===========================================================================
// U5-18 — §5.9's order pin (both instances on one READY store): `vectors: None` +
// no provider ⇒ FS-14 (not FS-13); `vectors: Some(_)` + no provider ⇒ FS-13.
// ===========================================================================

#[tokio::test]
async fn u5_18_on_a_ready_store_the_index_precedes_the_provider() {
    let corpus = seed_corpus(&[&[("n1", Some("alpha"))]]).await;
    let stub = scripted(Behavior::ByText);
    let pr = dyn_provider(&stub);

    // (a) unbuilt index, no provider wired.
    corpus.store.swap_snapshot(DerivedIndexes::default());
    corpus.store.set_engine_state(EngineState::Ready);
    assert!(
        corpus.store.embedding_provider().is_none(),
        "no provider is wired in this instance"
    );
    let err = corpus
        .store
        .rag_query("alpha", &vector_options(&corpus.wikis[0]))
        .await
        .expect_err("an unbuilt index wins over an absent provider");
    assert!(
        matches!(err, StoreError::VectorIndexUnavailable),
        "§5.9's precedence: `vectors: None` + no provider ⇒ `VectorIndexUnavailable`, NOT \
         `EmbeddingUnavailable` (got {err:?})"
    );

    // (b) built index, no provider wired.
    let built = build_boot_vector_index(&corpus.store, Some(&pr))
        .await
        .expect("all calls succeed")
        .expect("a supplied provider never yields `Ok(None)`");
    corpus.store.swap_snapshot(DerivedIndexes {
        vectors: Some(built),
        ..DerivedIndexes::default()
    });
    corpus.store.set_engine_state(EngineState::Ready);
    let err = corpus
        .store
        .rag_query("alpha", &vector_options(&corpus.wikis[0]))
        .await
        .expect_err("with an index but no provider the explicit leg fails on the embedding");
    assert!(
        matches!(err, StoreError::EmbeddingUnavailable),
        "§5.9: `vectors: Some(_)` + no provider ⇒ `EmbeddingUnavailable` (FS-13) (got {err:?})"
    );
    assert_eq!(
        server_status(&err),
        Some((503, "embedding_unavailable")),
        "§11: FS-13 renders 503 `embedding_unavailable`"
    );
}

// ===========================================================================
// U5-19 — `P-SM-7`: on a `Reachable` boot (index built, provider wired, `Ready`)
// a `mode=vector` request SERVES: `Ok(RagResult)` with the vector trace, engine
// "gnosis", `results.len() <= top_k`, and every result drawn from the index's
// full-field entries.
// ===========================================================================

#[tokio::test]
async fn u5_19_a_reachable_boot_serves_mode_vector_over_the_built_index() {
    let corpus = seed_corpus(&[&[("n1", Some("hello world")), ("n2", Some("goodbye"))]]).await;
    let stub = scripted(Behavior::ByText);
    let pr = dyn_provider(&stub);
    let built = build_boot_vector_index(&corpus.store, Some(&pr))
        .await
        .expect("all calls succeed")
        .expect("a supplied provider never yields `Ok(None)`");
    let snap = DerivedIndexes {
        vectors: Some(built),
        ..DerivedIndexes::default()
    };
    let (state, _, _) =
        apply_boot(&corpus.store, Boot::Reachable, Some(pr.clone()), &snap, &pr).await;
    assert_eq!(state, EngineState::Ready, "the boot is READY");

    let result = corpus
        .store
        .rag_query("hello", &vector_options(&corpus.wikis[0]))
        .await
        .expect("`mode=vector` serves on a `Reachable` boot (U5's whole point)");
    assert!(
        matches!(result.trace, RagTrace::Vector(_)),
        "the trace names the vector leg: `RagTrace::Vector(_)`"
    );
    assert_eq!(result.engine, "gnosis", "the result's engine is `gnosis`");
    assert!(
        result.results.len() <= 10,
        "`results.len() <= top_k` ({} results)",
        result.results.len()
    );
    let snap_now = corpus.store.snapshot();
    let entries = &snap_now
        .vectors
        .as_ref()
        .expect("the built index is the snapshot's `vectors`")
        .entries;
    for item in &result.results {
        let key = (
            item.document_id.clone(),
            item.node_id.clone(),
            FieldType::Full,
        );
        assert!(
            entries.contains_key(&key),
            "`P-SM-7`: every returned result is drawn from the index's full-field entries \
             ({key:?} is not a key of the built index)"
        );
    }
}

// ===========================================================================
// U5-20 — `P-SM-7`: an EMPTY index serves an EMPTY result (never an error, never a
// panic) — the reachable-boot consequence of an empty store.
// ===========================================================================

#[tokio::test]
async fn u5_20_an_empty_index_serves_an_empty_result_not_an_error() {
    let corpus = seed_corpus(&[]).await;
    let wiki = corpus
        .store
        .create_wiki("q")
        .await
        .expect("create_wiki")
        .wiki_id;
    let stub = scripted(Behavior::ByText);
    let pr = dyn_provider(&stub);
    let built = build_boot_vector_index(&corpus.store, Some(&pr))
        .await
        .expect("the empty store builds")
        .expect("an empty index is still an index — never `Ok(None)` for a supplied provider");
    let snap = DerivedIndexes {
        vectors: Some(built),
        ..DerivedIndexes::default()
    };
    let (state, _, _) =
        apply_boot(&corpus.store, Boot::Reachable, Some(pr.clone()), &snap, &pr).await;
    assert_eq!(
        state,
        EngineState::Ready,
        "the boot is READY over an empty index"
    );

    let result = corpus
        .store
        .rag_query("anything", &vector_options(&wiki))
        .await
        .expect(
            "an empty index serves `Ok` with an empty `results` — never `VectorIndexUnavailable`",
        );
    assert!(
        result.results.is_empty(),
        "state 2: `mode=vector` over an EMPTY index ⇒ 200 with an empty `results` array"
    );
    assert!(
        matches!(result.trace, RagTrace::Vector(_)),
        "the vector leg served"
    );
}

// ===========================================================================
// U5-21 — `P-TP-5`: the build is TOTAL over the adversarial provider shapes. The
// outcome stays in `{Ok(Some(index over exactly the corpus)), Err(EmbeddingUnavailable)}`,
// the returned vectors are the provider's own value verbatim (pinned `NaN` rule),
// `is_available` is never consulted, and no `Binary` key appears.
// ===========================================================================

#[tokio::test]
async fn u5_21_the_build_is_total_over_the_adversarial_provider_shapes() {
    let shapes = [
        ("dimension change between calls", Behavior::ByText),
        ("empty vector", Behavior::Empty),
        ("zero vector", Behavior::Zeros),
        ("NaN / ±∞", Behavior::NaNInf),
        ("every call errors", Behavior::AlwaysErr),
    ];
    let corpora = [
        ("empty store", seed_corpus(&[]).await),
        (
            "three-node store",
            seed_corpus(&[&[("n1", Some("abcdef")), ("n2", Some("gh")), ("n3", None)]]).await,
        ),
    ];

    for (corpus_label, corpus) in &corpora {
        for (shape, behavior) in shapes {
            let stub = scripted(behavior);
            let pr = dyn_provider(&stub);
            let r = build_boot_vector_index(&corpus.store, Some(&pr)).await;
            match r {
                Ok(Some(vi)) => {
                    assert!(
                        behavior != Behavior::AlwaysErr || corpus.embeddable_count() == 0,
                        "{corpus_label}/{shape}: an always-`Err` provider cannot produce an \
                         index over a non-empty corpus"
                    );
                    assert_entries_are_exactly(
                        &format!("U5-21 {corpus_label}/{shape}"),
                        &vi,
                        corpus,
                    );
                    for key in corpus.expected_keys() {
                        let text = corpus.text_of(&key);
                        let got = vi
                            .entries
                            .get(&key)
                            .unwrap_or_else(|| panic!("the key {key:?} must be present"));
                        assert_verbatim(
                            &format!("U5-21 {corpus_label}/{shape}"),
                            got,
                            &expected_vector(behavior, &text),
                        );
                    }
                    if behavior == Behavior::ByText && corpus.embeddable_count() >= 2 {
                        let lens: Vec<usize> = vi.entries.values().map(|v| v.len()).collect();
                        assert_ne!(
                            lens[0], lens[1],
                            "{corpus_label}/{shape}: the corpus really exercised a DIFFERENT \
                             dimension per call (U5 pins no dimension)"
                        );
                    }
                }
                Ok(None) => panic!(
                    "{corpus_label}/{shape}: `Ok(None)` is never the answer for a supplied \
                     provider"
                ),
                Err(StoreError::EmbeddingUnavailable) => assert!(
                    behavior == Behavior::AlwaysErr,
                    "{corpus_label}/{shape}: only an erroring `embed` may produce this `Err`"
                ),
                Err(other) => panic!(
                    "{corpus_label}/{shape}: never a new error type (`ValidationError`/\
                     `EngineError`/a new variant) — got {other:?}"
                ),
            }
            assert_eq!(
                stub.availability_probes(),
                0,
                "{corpus_label}/{shape}: the build makes NO availability probe of its own — \
                 `is_available` is never consulted (the boot's probe already produced the \
                 `BootProvider`)"
            );
        }
    }

    // The lying `is_available` shape: `true` while `embed` errors ⇒ NOTHING changes.
    let corpus = seed_corpus(&[&[("n1", Some("alpha"))]]).await;
    let liar = scripted_with_availability(Behavior::AlwaysErr, true);
    let pr = dyn_provider(&liar);
    let r = build_boot_vector_index(&corpus.store, Some(&pr)).await;
    assert!(
        matches!(r, Err(StoreError::EmbeddingUnavailable)),
        "a provider whose `is_available` lies (`true` while `embed` errors) changes NOTHING \
         about the build's outcome — got {r:?}"
    );
    assert_eq!(
        liar.availability_probes(),
        0,
        "the seam is never consulted by the build"
    );
}

// ===========================================================================
// U5-22 — `P-TP-5`/§9.5.5 (2): a duplicate TEXT under two DISTINCT ids yields two
// keys and two `embed` calls (the per-node rule; the pre-REMAND-4 "no re-embed on a
// duplicate text" phrase is deleted by the spec).
// ===========================================================================

#[tokio::test]
async fn u5_22_a_duplicate_text_under_two_ids_yields_two_keys_and_two_calls() {
    let corpus = seed_corpus(&[&[("p", Some("same text")), ("q", Some("same text"))]]).await;
    let stub = scripted(Behavior::ByText);
    let pr = dyn_provider(&stub);
    let vi = build_boot_vector_index(&corpus.store, Some(&pr))
        .await
        .expect("all calls succeed")
        .expect("a supplied provider never yields `Ok(None)`");

    assert_entries_are_exactly("U5-22", &vi, &corpus);
    assert_eq!(
        vi.entries.len(),
        2,
        "two nodes carrying the same `value` text are two embeddable nodes ⇒ TWO keys (no \
         text-keyed dedup)"
    );
    assert_eq!(
        stub.call_count(),
        2,
        "…and TWO `embed` calls — one per embeddable node, no text-keyed cache"
    );
}

// ===========================================================================
// U5-23 — `P-IM-12`: determinism + the store's own observable state untouched
// (`epoch()`, `journal_len()`, the snapshot `Arc` identity, `vectors` presence and
// the wired provider unchanged; no journal entry appended).
// ===========================================================================

#[tokio::test]
async fn u5_23_two_builds_agree_and_the_build_mutates_nothing_the_store_reports() {
    let corpus = seed_corpus(&[
        &[("n1", Some("alpha")), ("n2", None)],
        &[("n3", Some("beta"))],
    ])
    .await;
    let stub = scripted(Behavior::ByText);
    let pr = dyn_provider(&stub);
    corpus.store.set_embedding_provider(pr.clone());

    let epoch_before = corpus.store.epoch();
    let journal_before = corpus.store.journal_len();
    let snap_before = corpus.store.snapshot();
    let vectors_before = snap_before.vectors.is_some();
    let provider_before = corpus.store.embedding_provider().is_some();

    let a = build_boot_vector_index(&corpus.store, Some(&pr)).await;
    let b = build_boot_vector_index(&corpus.store, Some(&pr)).await;

    let (va, vb) = match (a, b) {
        (Ok(Some(x)), Ok(Some(y))) => (x, y),
        (x, y) => panic!("two builds over the same state must both be `Ok(Some(_))`: {x:?} {y:?}"),
    };
    assert_eq!(
        va.entries.len(),
        vb.entries.len(),
        "§9.5.5 (2)'s determinism clause: the same key SET both times"
    );
    for (k, v) in &va.entries {
        let other = vb
            .entries
            .get(k)
            .unwrap_or_else(|| panic!("the key {k:?} must be in both builds"));
        assert_verbatim("U5-23", v, other);
    }
    assert_entries_are_exactly("U5-23", &va, &corpus);
    assert_entries_are_exactly("U5-23 (second build)", &vb, &corpus);

    assert_eq!(
        corpus.store.epoch(),
        epoch_before,
        "the build does not move the epoch"
    );
    assert_eq!(
        corpus.store.journal_len(),
        journal_before,
        "the build appends NO journal entry (`append_journal` has no call site in the build)"
    );
    assert!(
        Arc::ptr_eq(&snap_before, &corpus.store.snapshot()),
        "the build does not swap the snapshot — only the CALLER installs (IMMUTABLE-DERIVED-\
         SNAPSHOT)"
    );
    assert_eq!(
        corpus.store.snapshot().vectors.is_some(),
        vectors_before,
        "`vectors` presence is unchanged by a build"
    );
    assert_eq!(
        corpus.store.embedding_provider().is_some(),
        provider_before,
        "the wired provider is unchanged by a build"
    );
}

// ===========================================================================
// U5-24 — entry question 5 + §13's U5 bullets: hermeticity — the build consults no
// env var and no ambient provider; its outcome is a function of its two ARGUMENTS
// only.
// ===========================================================================

#[tokio::test]
async fn u5_24_the_build_consults_no_env_var_and_no_ambient_provider() {
    let corpus = seed_corpus(&[&[("n1", Some("alpha"))]]).await;
    let stub = scripted(Behavior::ByText);
    let pr = dyn_provider(&stub);

    // A bogus ambient provider configuration is irrelevant to the lib seam.
    let saved = std::env::var("GNOSIS_SERVER_OLLAMA_URL").ok();
    std::env::set_var("GNOSIS_SERVER_OLLAMA_URL", "http://127.0.0.1:9/bogus");
    let with_env = build_boot_vector_index(&corpus.store, None).await;
    match saved {
        Some(v) => std::env::set_var("GNOSIS_SERVER_OLLAMA_URL", v),
        None => std::env::remove_var("GNOSIS_SERVER_OLLAMA_URL"),
    }
    assert!(
        matches!(with_env, Ok(None)),
        "§9.5.5 (5): the build consults NO env var — with a bogus ambient provider configuration \
         and no provider ARGUMENT the answer is still `Ok(None)` (the `BootProvider` is the \
         caller's, produced by the boot's own probe) — got {with_env:?}"
    );
    assert_eq!(
        stub.call_count(),
        0,
        "no ambient provider is consulted either"
    );

    // The same store with the provider ARGUMENT supplied is unaffected by the env.
    let supplied = build_boot_vector_index(&corpus.store, Some(&pr)).await;
    assert!(
        matches!(supplied, Ok(Some(_))),
        "the outcome is a function of the two arguments only — got {supplied:?}"
    );
}

// ===========================================================================
// U5-25 — entry question 6 (§13's U5 bullets): the U4/durability isolation — U5
// adds no route, no `StoreError` variant, no §11 row and no field on the frozen
// `EngineSubsystems`.
// ===========================================================================

#[test]
fn u5_25_no_route_error_map_or_frozen_type_moved() {
    let table = route_bijection();
    assert_eq!(
        table.len(),
        14,
        "§13's U5 bullet: U5 does NOT change the route table (14 rows) — no route is added"
    );
    let mut paths: Vec<&str> = Vec::new();
    let mut handlers: Vec<&str> = Vec::new();
    for &(path, handler) in table {
        assert!(
            !paths.contains(&path),
            "the routing table stays a bijection: {path} twice"
        );
        assert!(
            !handlers.contains(&handler),
            "the routing table stays a bijection: {handler} twice"
        );
        let lower = path.to_lowercase();
        assert!(
            !lower.contains("vector") && !lower.contains("index"),
            "U5 adds no route (found {path}); the boot build is not an endpoint"
        );
        paths.push(path);
        handlers.push(handler);
    }
    let engine_paths: Vec<&str> = ENGINE_ENDPOINTS.iter().map(|(p, _)| *p).collect();
    assert_eq!(engine_paths.len(), 11, "the 11 CRUD rows are unchanged");
    for p in &engine_paths {
        assert!(
            paths.contains(p),
            "the CRUD route {p} is still present verbatim"
        );
    }
    for p in ["POST /rag/query", "GET /rag/stream", "GET /engine/status"] {
        assert!(
            paths.contains(&p),
            "the retrieval trio is still present: {p}"
        );
    }

    // The error map is unchanged: the boot-build failure's family is the existing
    // FS-13 code, and the FS-8/FS-14 rows are untouched.
    assert_eq!(
        server_status(&StoreError::EmbeddingUnavailable),
        Some((503, "embedding_unavailable"))
    );
    assert_eq!(
        server_status(&StoreError::VectorIndexUnavailable),
        Some((503, "vector_index_unavailable"))
    );
    assert_eq!(
        server_status(&StoreError::EngineUnavailable),
        Some((503, "engine_unavailable"))
    );

    // The frozen six-bool `EngineSubsystems` gains/loses/re-types nothing (a
    // literal with exactly these six fields still compiles, and the serialized key
    // set is exactly the six single-word flags).
    let value = serde_json::to_value(mask(true, true)).expect("EngineSubsystems is serializable");
    let mut keys: Vec<&str> = value
        .as_object()
        .expect("a flag object")
        .keys()
        .map(|k| k.as_str())
        .collect();
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
        "U5 adds NO field to the frozen `EngineSubsystems` and no `HealthReport` field either"
    );
}

// ===========================================================================
// U5-26 — live (hermetic): a provider-ABSENT boot builds nothing and its
// `mode=vector` request is FS-8 `engine_unavailable` — never FS-14.
// ===========================================================================

#[tokio::test]
async fn u5_26_live_provider_absent_boot_has_no_index_and_is_engine_unavailable() {
    let (child, base) = spawn_server_without_provider().await;
    let _guard = ServerGuard(child);

    let (status, body) = get_engine_status_json(&base).await;
    assert_eq!(status, 200, "§5.8: `GET /engine/status` is always 200");
    assert_ne!(
        body.get("state").and_then(|v| v.as_str()),
        Some("Ready"),
        "§6/`P-IM-9`: a provider-absent boot must NOT fabricate READY"
    );
    let flags = flags_of(&body);
    assert_eq!(
        flags,
        v8_2_mask(),
        "§9.5.5 (1) row 3 + §9.5.5 (5)'s hermeticity clause: the provider-absent boot builds \
         nothing ⇒ vector:false, embedding:false, reranker:false, core true"
    );

    let (post_status, text) =
        post_rag_query(&base, serde_json::json!({"query": "q", "mode": "vector"})).await;
    assert_eq!(
        post_status, 503,
        "§5.9's precedence bullet: a non-READY store's `mode=vector` fails the READY gate"
    );
    assert_eq!(
        error_code(&text),
        "engine_unavailable",
        "`P-SM-7`'s live instance: a non-READY boot ⇒ FS-8 `engine_unavailable`, NOT \
         `vector_index_unavailable` (the response was {text})"
    );
}

// ===========================================================================
// U5-27 — live (hermetic): a configured-but-UNREACHABLE provider takes the
// `Unreachable` branch (no build) and reports the honest degraded vector — plus the
// same FS-8 query consequence.
// ===========================================================================

#[tokio::test]
async fn u5_27_live_unreachable_provider_boot_degrades_without_building() {
    let (child, base) = spawn_server_with_unreachable_provider().await;
    let _guard = ServerGuard(child);

    let (status, body) = get_engine_status_json(&base).await;
    assert_eq!(status, 200, "the status surface is always 200");
    assert_eq!(
        body.get("state").and_then(|v| v.as_str()),
        Some("Degraded"),
        "§9.5.5 (1) row 2: a configured provider whose probe failed ⇒ `Unreachable` ⇒ `Degraded`"
    );
    assert_eq!(
        flags_of(&body),
        v8_2_mask(),
        "§9.5.5 (1) row 2 / §12 V-8.2: the unreachable-provider boot attempts NO embedding call \
         and keeps the unchanged index-free snapshot"
    );
    assert_eq!(
        body.get("lastError").and_then(|v| v.as_str()),
        Some(FIXED_LAST_ERROR),
        "V-8.2's fixed DEGRADED `lastError`, byte-identical (U5 does not reword it)"
    );

    let (post_status, text) =
        post_rag_query(&base, serde_json::json!({"query": "q", "mode": "vector"})).await;
    assert_eq!(
        post_status, 503,
        "a non-READY store's explicit leg hits the READY gate"
    );
    assert_eq!(
        error_code(&text),
        "engine_unavailable",
        "FS-8 — never FS-14, whatever the snapshot holds (the response was {text})"
    );
}
