//! §7.2 P2 **U5 — the boot vector-index build** — integration / conformance
//! layer (`tests/u5_boot_vector_index_conformance.rs`).
//!
//! TestWriter-derived **from the spec alone**
//! (`docs/specs/p2-gnosis-server.md` §9.5.5, plus its U5 annotations in §5.8,
//! §5.9, §6, §9.5.2/§9.5.4, §11, §12, and the U5 notes in
//! `docs/specs/engine-wire-contract.md` §9.1/§12). The behavior contract, the
//! six entry-question answers (contract tables (1)–(6)), the state table, the
//! failure table, the corpus-seeding clause and the move table are the source of
//! every assertion below.
//!
//! ## Surface under test (§9.5.5's "What U5 adds", surface #1)
//!
//! ```text
//! async fn build_boot_vector_index(
//!     store: &Store,
//!     provider: Option<&Arc<dyn crate::store::EmbeddingProvider>>,
//! ) -> Result<Option<VectorIndex>, crate::store::StoreError>
//! ```
//!
//! A **total, three-valued** function: `Ok(None)` = *no index is to be built*
//! (no provider supplied), `Ok(Some(vi))` = *the index to install* (`vi.entries`
//! may be empty), `Err(StoreError::EmbeddingUnavailable)` = *a provider was
//! present but an `embed` call failed*.
//!
//! ## State machine (this file's coverage map)
//!
//! Contract table (1) pins the three boot inputs; the state table pins seven
//! numbered states. Their coverage here:
//!
//! | state table # | state | test |
//! | --- | --- | --- |
//! | 1 | happy — `Reachable(p)`, populated store | `u5_state_1_reachable_populated_store_builds_full_index` |
//! | 2 | happy — `Reachable(p)`, empty store | `u5_state_2_reachable_empty_store_builds_an_empty_index` |
//! | 3 | happy — non-embeddable nodes only | `u5_state_3_value_missing_contributes_nothing_empty_value_contributes_one` |
//! | 4 | fail — mid-build embedding error | `u5_state_4_mid_build_error_installs_no_index_and_degrades`, `u5_failure_branch_call_order_is_the_pinned_order` |
//! | 5 | fail — `Absent` / `Unreachable` | `u5_state_5_absent_and_unreachable_boots_attempt_no_build` |
//! | 6 | fail — unbuilt index asked to serve (READY + `vectors: None`) | `u5_state_6_ready_store_without_index_still_reports_fs14_vector_index_unavailable` |
//! | 7 | fail — built index, no wired provider | `u5_state_7_built_index_with_no_provider_is_fs13_embedding_unavailable` |
//! | contract table (1) | the three `BootProvider` rows (build / no build / no build) | `u5_contract_1_boot_input_table_flags`, `u5_state_5_absent_and_unreachable_boots_attempt_no_build` |
//! | contract table (2) | corpus scope, text, key shape, call discipline, empty store | `u5_state_1_…`, `u5_state_3_…`, `u5_contract_2_corpus_rule_duplicate_text_two_nodes` |
//! | contract table (3) | full field only, dimension not pinned | `u5_state_1_…`, `u5_contract_3_full_field_only_and_dimension_verbatim` |
//! | contract table (4) | cost / failure / wire consequence | `u5_state_4_…`, `u5_failure_branch_call_order_is_the_pinned_order`, `u5_state_5_…` |
//! | contract table (5) | no new knob, hermetic in-memory provider | every test injects a deterministic in-memory provider (never a live one) |
//! | contract table (6) | U4/journal/durability untouched | `u5_contract_6_build_appends_no_journal_entry_and_moves_no_epoch` |
//!
//! ## RED-stage (U5 — code OWED, §9.5.5's verification status)
//!
//! `gnosis::build_boot_vector_index` does not exist yet, so this file FAILS TO
//! COMPILE (the missing-symbol red set), exactly as §9.5.5 requires: the unit's
//! code is owed and the suite is expected to be red at the end of this stage.
//!
//! ## Hermeticity
//!
//! No test reaches a live provider or the network, and none reads an ambient
//! environment variable (§9.5.5's entry question 5): every provider is a
//! deterministic in-memory `EmbeddingProvider` injected through the lib seam.

use gnosis::{
    boot_wiring, server_status, BootProvider, CreateDocumentRequest, DerivedIndexes, Document,
    DocumentId, Edge, EdgeKind, EmbeddingProvider, EngineState, EngineStatus, FieldType, Graph,
    Node, NodeId, NodeKind, RagQueryOptions, RagResult, RagStore, Store, StoreError,
    UpdateDocumentRequest, VectorIndex,
};
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

// The surface under test — ABSENT today (the missing-symbol red set).
use gnosis::build_boot_vector_index;

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// What the injected provider's `embed` does (§9.5.5's contract table (6) and
/// `P-TP-5`'s adversarial shapes; every mode is deterministic).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum EmbedMode {
    /// A constant-length vector: the plain happy path.
    Ok,
    /// The returned length follows the text (dimension drift between calls).
    DimensionDrift,
    /// A `NaN` element.
    Nan,
    /// An empty `Vec<f32>`.
    EmptyVec,
    /// `is_available()` claims `true` while every `embed` errors (the lie).
    Lie,
    /// Fail on the `k`-th call (`1`-based); later calls succeed.
    FailAt(usize),
    /// Every call errors.
    AlwaysErr,
}

/// A deterministic in-memory `EmbeddingProvider` (§9.5.5 surface #6) that
/// **counts its calls** (the call-discipline witness) and records the texts it
/// was asked to embed (so "no call for a `value:None` node" and "one call per
/// embeddable node" are both readable).
struct U5Provider {
    mode: EmbedMode,
    calls: AtomicUsize,
    texts: Mutex<Vec<String>>,
    available_probes: AtomicUsize,
}

impl U5Provider {
    fn new(mode: EmbedMode) -> Arc<Self> {
        Arc::new(U5Provider {
            mode,
            calls: AtomicUsize::new(0),
            texts: Mutex::new(Vec::new()),
            available_probes: AtomicUsize::new(0),
        })
    }

    fn call_count(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }

    fn embedded_texts(&self) -> Vec<String> {
        self.texts.lock().unwrap().clone()
    }

    fn availability_probes(&self) -> usize {
        self.available_probes.load(Ordering::SeqCst)
    }

    /// The `Arc<dyn EmbeddingProvider>` the build's seam takes, from the SAME
    /// counter the test reads (an unsizing coercion of the identical allocation).
    fn as_provider(self: &Arc<Self>) -> Arc<dyn EmbeddingProvider> {
        self.clone()
    }

    /// The vector `embed` would return for `text` in the non-failing modes.
    fn vector_for(&self, text: &str) -> Vec<f32> {
        match self.mode {
            EmbedMode::Ok => vec![0.5, 0.25],
            EmbedMode::DimensionDrift => vec![0.5; 2 + (text.len() % 5)],
            EmbedMode::Nan => vec![f32::NAN, 1.0],
            EmbedMode::EmptyVec => Vec::new(),
            _ => vec![0.5, 0.25],
        }
    }

    /// Does the `k`-th (`1`-based) call error in this mode?
    fn call_fails(&self, k: usize) -> bool {
        match self.mode {
            EmbedMode::Lie | EmbedMode::AlwaysErr => true,
            EmbedMode::FailAt(n) => k == n,
            _ => false,
        }
    }
}

impl EmbeddingProvider for U5Provider {
    fn embed(
        &self,
        text: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<f32>, StoreError>> + Send + '_>> {
        let k = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
        self.texts.lock().unwrap().push(text.to_string());
        let fails = self.call_fails(k);
        let v = self.vector_for(text);
        Box::pin(async move {
            if fails {
                Err(StoreError::EmbeddingUnavailable)
            } else {
                Ok(v)
            }
        })
    }

    fn is_available(&self) -> Pin<Box<dyn Future<Output = bool> + Send + '_>> {
        self.available_probes.fetch_add(1, Ordering::SeqCst);
        // The `Lie` mode is the adversarially-honest `true`.
        let honest = !matches!(self.mode, EmbedMode::Lie | EmbedMode::AlwaysErr);
        Box::pin(async move { honest })
    }
}

/// One seeded corpus (§9.5.5's "corpus seeding" clause: the public `RagStore`
/// surface only — a wiki, documents, then one `update_document` per document
/// carrying the nodes; never a private field, a serde round-trip or a
/// hand-built `Document`).
#[derive(Clone, Default)]
struct Corpus {
    /// `(documentId, node's value)` pairs, in seeding order.
    nodes: Vec<(DocumentId, Option<String>)>,
}

impl Corpus {
    /// The embeddable corpus: nodes whose `value` is `Some(_)` (§9.5.5's
    /// contract table (2); `Some("")` counts, `None` does not) — the `n` of the
    /// count witnesses.
    fn embeddable(&self) -> Vec<&(DocumentId, Option<String>)> {
        self.nodes.iter().filter(|(_, v)| v.is_some()).collect()
    }

    fn embeddable_count(&self) -> usize {
        self.embeddable().len()
    }

    /// The expected key set — exactly the corpus's embeddable
    /// `(documentId, nodeId, FieldType::Full)` triples (the register's key-set
    /// witness; the seeders never mint two nodes sharing a `NodeId`, which
    /// §9.5.5's corpus-key-uniqueness row excludes from the domain).
    fn expected_keys(&self) -> Vec<(DocumentId, NodeId, FieldType)> {
        self.nodes
            .iter()
            .enumerate()
            .filter(|(_, (_, v))| v.is_some())
            .map(|(i, (d, _))| (d.clone(), NodeId(node_id(i)), FieldType::Full))
            .collect()
    }
}

/// Node ids are derived from the seeding index, so they are unique within each
/// document **and** across documents (the corpus-key-uniqueness rule).
fn node_id(i: usize) -> String {
    format!("n{i}")
}

/// Seed `corpus["docs"]` documents (each with its own node slice) through the
/// public `RagStore` surface and return the seeded corpus.
///
/// `docs` is a list of node-value lists: `None` is a non-embeddable node and
/// `Some(text)` an embeddable one.
async fn seed_store(docs: &[Vec<Option<String>>]) -> (Store, Corpus) {
    let store = Store::new();
    let wiki = store.create_wiki("w").await.expect("create_wiki").wiki_id;
    let mut corpus = Corpus::default();
    let mut next_node = 0usize;
    for values in docs {
        let doc: Document = store
            .create_document(
                &wiki,
                CreateDocumentRequest {
                    title: "t".to_string(),
                    tags: None,
                    author: None,
                },
            )
            .await
            .expect("create_document");
        let document_id = doc.document_id.clone();
        let mut nodes: Vec<Node> = Vec::new();
        let mut first: Option<NodeId> = None;
        for value in values {
            let nid = NodeId(node_id(next_node));
            next_node += 1;
            corpus.nodes.push((document_id.clone(), value.clone()));
            nodes.push(Node {
                document_id: document_id.clone(),
                node_id: nid.clone(),
                kind: NodeKind::Content,
                value: value.clone(),
                fact_key: None,
                target: None,
            });
            if first.is_none() {
                first = Some(nid);
            }
        }
        // `valid_provident_graph` requires exactly one DocHead and one DocEnd
        // edge (§9.5.5's corpus-seeding clause) — a node-only graph is rejected
        // with `ValidationError`, and no U5 assertion may come from its own
        // seeding.
        let head = first.expect("a seeded document carries at least one node");
        let graph = Graph {
            nodes,
            edges: vec![
                Edge {
                    source: (document_id.clone(), NodeId("ROOT".to_string())),
                    target: (document_id.clone(), head.clone()),
                    kind: EdgeKind::DocHead,
                    state: None,
                    cross_wiki: false,
                    relation_type: None,
                },
                Edge {
                    source: (document_id.clone(), head.clone()),
                    target: (document_id.clone(), NodeId("END".to_string())),
                    kind: EdgeKind::DocEnd,
                    state: None,
                    cross_wiki: false,
                    relation_type: None,
                },
            ],
        };
        let cur = store
            .get_document(&document_id)
            .await
            .expect("get_document");
        store
            .update_document(
                &document_id,
                UpdateDocumentRequest {
                    base_revision: cur.revision,
                    graph,
                    title: None,
                    tags: None,
                },
            )
            .await
            .expect("update_document");
    }
    (store, corpus)
}

/// The sorted key sets of a built index, for set comparison (a `HashMap`'s
/// iteration order is never asserted — §9.5.5's iteration-order row pins
/// determinism only, "no insertion-order claim").
fn key_set(vi: &VectorIndex) -> Vec<(DocumentId, NodeId, FieldType)> {
    let mut keys: Vec<(DocumentId, NodeId, FieldType)> = vi.entries.keys().cloned().collect();
    sort_keys(&mut keys);
    keys
}

/// `FieldType` is not `Ord` (the frozen store type derives only `Eq`/`Hash`), so
/// the set comparison sorts by the ids and the field's label — never by an
/// ordering the contract does not provide.
fn sort_keys(keys: &mut [(DocumentId, NodeId, FieldType)]) {
    keys.sort_by(|a, b| {
        (&a.0 .0, &a.1 .0, field_label(&a.2)).cmp(&(&b.0 .0, &b.1 .0, field_label(&b.2)))
    });
}

fn field_label(f: &FieldType) -> &str {
    match f {
        FieldType::Full => "Full",
        FieldType::Binary => "Binary",
        FieldType::Other(s) => s.as_str(),
    }
}

/// The expected keys, sorted the same way.
fn expected_key_set(corpus: &Corpus) -> Vec<(DocumentId, NodeId, FieldType)> {
    let mut keys = corpus.expected_keys();
    sort_keys(&mut keys);
    keys
}

/// The §9.5.5 corpus used by the integration states: one document with an
/// embeddable node, a `Some("")` node, a `value:None` node and a second
/// embeddable node — i.e. 3 embeddable of 4.
fn mixed_corpus() -> Vec<Vec<Option<String>>> {
    vec![vec![
        Some("alpha".to_string()),
        Some(String::new()),
        None,
        Some("beta".to_string()),
    ]]
}

/// The boot's pinned **failure** outcome, in the order contract table (4) pins
/// (REMAND-1): `swap_snapshot(unchanged index-free snapshot)` →
/// `set_embedding_provider(pr)` (the probe DID succeed) →
/// `set_engine_state(boot_wiring(Unreachable, &snap).0)` with the returned flag
/// vector **discarded**.
fn apply_boot_failure(store: &Store, provider: Arc<dyn EmbeddingProvider>, snap: &DerivedIndexes) {
    store.swap_snapshot(snap.clone());
    store.set_embedding_provider(provider);
    let (state, _flags) = boot_wiring(BootProvider::Unreachable, snap);
    store.set_engine_state(state);
}

/// The fixed `Degraded` `last_error` string the store owns (§9.5.5: never
/// reworded by U5).
const DEGRADED_LAST_ERROR: &str = "a non-core subsystem (embedding/reranker) is unavailable";

/// Apply the pinned boot wiring for a `BootProvider` and read the derived
/// status (§9.5.5's `P-IM-14` wiring path: the returned `EngineState` applied,
/// the snapshot swapped, the provider wired only in the `Reachable` case, never
/// `set_subsystems`).
async fn wired_status(provider: &BootProvider, snap: DerivedIndexes) -> EngineStatus {
    let (state, _flags) = boot_wiring(
        match provider {
            BootProvider::Absent => BootProvider::Absent,
            BootProvider::Unreachable => BootProvider::Unreachable,
            BootProvider::Reachable(p) => BootProvider::Reachable(p.clone()),
        },
        &snap,
    );
    let store = Store::new();
    store.swap_snapshot(snap);
    if let BootProvider::Reachable(p) = provider {
        store.set_embedding_provider(p.clone());
    }
    store.set_engine_state(state);
    store.get_engine_status().await
}

/// The store's own `last_error` for the pinned failure outcome (read, never
/// asserted against a literal the test invented).
fn degraded_status_shape_ok(status: &EngineStatus) -> bool {
    status.state == EngineState::Degraded
        && !status.subsystems.vector
        && status.subsystems.embedding
        && !status.subsystems.reranker
        && status.subsystems.store
        && status.subsystems.graph
        && status.subsystems.lexical
        && status.last_error.as_deref() == Some(DEGRADED_LAST_ERROR)
}

// ---------------------------------------------------------------------------
// State 1 — happy: `Reachable(p)` over a populated store (§9.5.5's row 1)
// ---------------------------------------------------------------------------

/// Valid state 1. Provider reachable, populated store: the build returns
/// `Ok(Some(vi))` whose entries are **exactly** the corpus's embeddable
/// `(documentId, nodeId, FieldType::Full)` triples; the boot composes
/// `snap.vectors = Some(vi)`; `boot_wiring(Reachable, &snap).1.vector == true`;
/// the subsystems read `{store,graph,lexical,vector,embedding:true,
/// reranker:false}` in state `Ready`.
///
/// Fail-states excluded from this test: mid-build provider error (state 4),
/// absent/unreachable provider (state 5), unbuilt index (state 6).
#[tokio::test]
async fn u5_state_1_reachable_populated_store_builds_full_index() {
    let (store, corpus) = seed_store(&mixed_corpus()).await;
    assert_eq!(
        corpus.embeddable_count(),
        3,
        "the seeded corpus is 3 embeddable nodes of 4 (one `value:None`)"
    );
    let provider = U5Provider::new(EmbedMode::Ok);

    let built = build_boot_vector_index(&store, Some(&provider.as_provider()))
        .await
        .expect("a reachable provider with all-`Ok` embeds is `Ok(Some(vi))`");
    let vi = built.expect("a supplied provider never yields `Ok(None)`");
    assert_eq!(
        key_set(&vi),
        expected_key_set(&corpus),
        "the index's keys are EXACTLY the corpus's embeddable (doc, node, Full) triples"
    );
    assert_eq!(
        provider.call_count(),
        corpus.embeddable_count(),
        "exactly one `embed` call per embeddable node (the call-count rule)"
    );
    // The call discipline, both halves: the embedded texts are EXACTLY the
    // corpus's embeddable node texts (so no `value:None` node's absent text and
    // no title/tag text is ever embedded).
    let mut want_texts: Vec<String> = corpus.nodes.iter().filter_map(|(_, v)| v.clone()).collect();
    want_texts.sort();
    let mut got_texts = provider.embedded_texts();
    got_texts.sort();
    assert_eq!(
        got_texts, want_texts,
        "one `embed` call per embeddable node, carrying that node's own text"
    );

    // The boot composes the snapshot from the returned index alone, then applies
    // the pinned wiring.
    let snap = DerivedIndexes {
        vectors: Some(vi),
        ..DerivedIndexes::default()
    };
    let (state, flags) = boot_wiring(BootProvider::Reachable(provider.as_provider()), &snap);
    assert_eq!(state, EngineState::Ready, "a reachable boot is `Ready`");
    assert!(
        flags.vector,
        "`vector` is `true` iff the snapshot has an index"
    );

    let status = wired_status(&BootProvider::Reachable(provider.as_provider()), snap).await;
    assert_eq!(status.state, EngineState::Ready);
    assert_eq!(
        status.subsystems,
        gnosis::EngineSubsystems {
            store: true,
            graph: true,
            lexical: true,
            vector: true,
            embedding: true,
            reranker: false,
        },
        "the post-U5 READY mask of §9.5.5's contract table (1)"
    );
}

// ---------------------------------------------------------------------------
// State 2 — happy: `Reachable(p)` over an EMPTY store (§9.5.5's row 2)
// ---------------------------------------------------------------------------

/// Valid state 2. An empty store with a reachable provider still builds:
/// `Ok(Some(VectorIndex::default()))` — `entries` empty, **not** `Ok(None)` —
/// so `snap.vectors.is_some()` ⇒ `vector: true` and `mode=vector` serves an
/// empty result rather than an error.
#[tokio::test]
async fn u5_state_2_reachable_empty_store_builds_an_empty_index() {
    let (store, corpus) = seed_store(&[]).await;
    let provider = U5Provider::new(EmbedMode::Ok);

    let built = build_boot_vector_index(&store, Some(&provider.as_provider()))
        .await
        .expect("an empty corpus is still `Ok(Some(empty index))`");
    let vi = built.expect(
        "`Ok(None)` is the no-provider answer — never the answer for a supplied provider, \
         not even over an empty corpus",
    );
    assert!(
        vi.entries.is_empty(),
        "an empty corpus yields zero keys (an empty index IS an index)"
    );
    assert_eq!(
        provider.call_count(),
        0,
        "nothing to embed ⇒ no `embed` call"
    );
    assert_eq!(corpus.embeddable_count(), 0);

    let snap = DerivedIndexes {
        vectors: Some(vi),
        ..DerivedIndexes::default()
    };
    let status = wired_status(
        &BootProvider::Reachable(provider.as_provider()),
        snap.clone(),
    )
    .await;
    assert!(
        status.subsystems.vector,
        "`Some(EMPTY VectorIndex)` ⇒ `vector:true` (the flag is the wiring, not the entry count)"
    );
    assert_eq!(status.state, EngineState::Ready);

    // The same pinned wiring APPLIED to this store (the returned `EngineState`
    // set, the built snapshot swapped, the provider wired — never a mask write),
    // so the query below runs against the post-boot state.
    let (state, _flags) = boot_wiring(BootProvider::Reachable(provider.as_provider()), &snap);
    store.swap_snapshot(snap);
    store.set_embedding_provider(provider.as_provider());
    store.set_engine_state(state);

    // …and the empty index serves an empty result, never an error.
    let result: RagResult = store
        .rag_query(
            "alpha",
            &RagQueryOptions {
                top_k: Some(5),
                mode: Some(gnosis::QueryMode::Vector),
                ..Default::default()
            },
        )
        .await
        .expect("an empty index serves an empty result");
    assert!(result.results.is_empty(), "no indexed node ⇒ empty results");
}

// ---------------------------------------------------------------------------
// State 3 — happy: non-embeddable nodes only (§9.5.5's row 3)
// ---------------------------------------------------------------------------

/// Valid state 3. `value:None` nodes contribute no key while a `Some("")` node
/// **does** contribute one (the contract embeds it); the outcome is still
/// `Some` — `Ok(None)` is never the answer for a supplied provider.
#[tokio::test]
async fn u5_state_3_value_missing_contributes_nothing_empty_value_contributes_one() {
    let (store, corpus) = seed_store(&[vec![
        None,
        Some(String::new()),
        None,
        Some("only-this-one".to_string()),
    ]])
    .await;
    assert_eq!(corpus.embeddable_count(), 2);
    let provider = U5Provider::new(EmbedMode::Ok);

    let vi = build_boot_vector_index(&store, Some(&provider.as_provider()))
        .await
        .expect("a supplied provider with all-`Ok` embeds is `Ok(Some(vi))`")
        .expect("never `Ok(None)` for a supplied provider");
    assert_eq!(
        key_set(&vi),
        expected_key_set(&corpus),
        "the `Some(\"\")` node has an entry and neither `value:None` node does"
    );
    assert_eq!(
        provider.call_count(),
        2,
        "two embeddable nodes ⇒ exactly two `embed` calls (no call for a `value:None` node)"
    );
    let texts = provider.embedded_texts();
    assert!(
        texts.contains(&String::new()),
        "the `Some(\"\")` node IS embedded — its text is the empty string"
    );
}

// ---------------------------------------------------------------------------
// State 4 — fail: mid-build embedding error (§9.5.5's row 4 / failure table)
// ---------------------------------------------------------------------------

/// Fail-state 4 (with the boundary *k* = 1 and *k* = *n*). A provider whose
/// *k*-th `embed` returns `Err` makes the build return
/// `Err(StoreError::EmbeddingUnavailable)` — never `Ok(Some(partial))` and never
/// `Ok(None)` — and after the boot's documented failure outcome the store holds
/// the **unchanged** index-free snapshot, reports `Degraded`/`vector:false`/
/// `embedding:true`/`reranker:false` and the store's own fixed `last_error`;
/// `mode=vector` then takes the FS-8 pre-READY gate.
#[tokio::test]
async fn u5_state_4_mid_build_error_installs_no_index_and_degrades() {
    let (store, corpus) = seed_store(&mixed_corpus()).await;
    let n = corpus.embeddable_count();
    assert!(
        n >= 2,
        "the boundary cases need at least two embeddable nodes"
    );

    for k in [1usize, n] {
        let provider = U5Provider::new(EmbedMode::FailAt(k));
        let err = build_boot_vector_index(&store, Some(&provider.as_provider()))
            .await
            .expect_err("a failed `embed` call makes the build `Err`");
        assert_eq!(
            err,
            StoreError::EmbeddingUnavailable,
            "the only error is the EXISTING `EmbeddingUnavailable` (k={k})"
        );
        assert_eq!(
            provider.call_count(),
            k,
            "the build stops at the k-th failing call and retries nothing (k={k})"
        );

        // The boot's documented failure outcome (contract table (4), the pinned
        // order): unchanged index-free snapshot + the provider WIRED + the
        // derived `Unreachable` state.
        apply_boot_failure(&store, provider.as_provider(), &DerivedIndexes::default());
        assert!(
            store.snapshot().vectors.is_none(),
            "no partial index is installed — the snapshot stays index-free (k={k})"
        );
        let status = store.get_engine_status().await;
        assert!(
            degraded_status_shape_ok(&status),
            "the failed build's status must be the pinned Degraded pair, got {status:?} (k={k})"
        );
        assert_ne!(
            status.state,
            EngineState::Ready,
            "never a fabricated `Ready`"
        );

        // The wire consequence: the `Degraded` state is non-READY, so the READY
        // gate fires before the leg check (FS-8, not FS-14).
        let err = store
            .rag_query(
                "alpha",
                &RagQueryOptions {
                    top_k: Some(5),
                    mode: Some(gnosis::QueryMode::Vector),
                    ..Default::default()
                },
            )
            .await
            .expect_err("a non-READY store short-circuits every non-`graph` mode");
        assert_eq!(
            err,
            StoreError::EngineUnavailable,
            "the failed-build branch is Degraded ⇒ FS-8 (k={k})"
        );
        let (status_code, code) = server_status(&err).expect("FS-8 has a §11 row");
        assert_eq!(status_code, 503);
        assert_eq!(code, "engine_unavailable");
    }
}

/// The failure branch's **call order** (pinned verbatim by REMAND-1): the same
/// steps in the same order as the `Unreachable` branch, with the provider wired
/// because the probe succeeded. Observed as the difference between the two
/// branches: `embedding:true` for the failed build, `false` for `Unreachable`.
#[tokio::test]
async fn u5_failure_branch_call_order_is_the_pinned_order() {
    let (store, corpus) = seed_store(&mixed_corpus()).await;
    assert!(corpus.embeddable_count() > 0);
    let provider = U5Provider::new(EmbedMode::AlwaysErr);
    assert!(
        build_boot_vector_index(&store, Some(&provider.as_provider()))
            .await
            .is_err(),
        "the always-`Err` provider makes the build fail"
    );

    // The pinned outcome, applied in the pinned order.
    apply_boot_failure(&store, provider.as_provider(), &DerivedIndexes::default());
    let failed = store.get_engine_status().await;
    assert!(degraded_status_shape_ok(&failed), "got {failed:?}");

    // The `Unreachable` branch writes the same steps but wires NO provider.
    let unreachable = wired_status(&BootProvider::Unreachable, DerivedIndexes::default()).await;
    assert_eq!(
        unreachable.state,
        EngineState::Degraded,
        "both branches derive the `Unreachable` branch's state"
    );
    assert!(!unreachable.subsystems.embedding);
    assert!(
        failed.subsystems.embedding,
        "a failing boot MUST NOT skip the provider wiring (its one behavioural difference)"
    );
    assert_eq!(
        failed.subsystems,
        gnosis::EngineSubsystems {
            embedding: true,
            ..unreachable.subsystems.clone()
        },
        "the two branches differ in exactly the `embedding` flag"
    );
    assert_eq!(failed.last_error, unreachable.last_error);

    // The returned flag vector of that `boot_wiring` call is DISCARDED (the
    // store derives its own at read time): an independent read equals the store's.
    let (_, discarded) = boot_wiring(BootProvider::Unreachable, &DerivedIndexes::default());
    assert_eq!(
        discarded.vector, failed.subsystems.vector,
        "the flag vector is not a mask the boot applies; the store's read is the observable"
    );
}

// ---------------------------------------------------------------------------
// State 5 — fail: no provider / unreachable provider (§9.5.5's row 5)
// ---------------------------------------------------------------------------

/// Fail-state 5. `build_boot_vector_index(s, None)` is `Ok(None)` — **not** an
/// error, and with **no** `embed` call; the `Absent`/`Unreachable` boots leave
/// the store non-READY with an index-free snapshot, so `mode=vector` is FS-8
/// `EngineUnavailable` → 503 `engine_unavailable` (**NOT** FS-14).
#[tokio::test]
async fn u5_state_5_absent_and_unreachable_boots_attempt_no_build() {
    let (store, corpus) = seed_store(&mixed_corpus()).await;
    assert!(corpus.embeddable_count() > 0);

    // (a) no provider ⇒ `Ok(None)`, and the injected provider is never consulted.
    let unused = U5Provider::new(EmbedMode::Ok);
    let built = build_boot_vector_index(&store, None)
        .await
        .expect("`no provider` is NOT an error — nothing is to be built");
    assert!(
        built.is_none(),
        "`Ok(None)` iff no provider was supplied (the index-free snapshot is kept)"
    );
    let _ = &unused; // the seam is `None`; there is no provider to count

    // (b) the two no-build boot outcomes, element-wise.
    for provider in [BootProvider::Absent, BootProvider::Unreachable] {
        let snap = DerivedIndexes::default();
        let status = wired_status(&provider, DerivedIndexes::default()).await;
        assert!(
            snap.vectors.is_none() && !status.subsystems.vector,
            "no build ⇒ the unchanged snapshot has no index and `vector:false`"
        );
        assert!(
            status.state != EngineState::Ready,
            "an `Absent`/`Unreachable` boot is never READY"
        );
        assert!(!status.subsystems.embedding);
        assert!(!status.subsystems.reranker);

        // The §11 mapping of the non-READY outcome is FS-8, not FS-14.
        let err = store
            .rag_query(
                "alpha",
                &RagQueryOptions {
                    top_k: Some(5),
                    mode: Some(gnosis::QueryMode::Vector),
                    ..Default::default()
                },
            )
            .await
            .expect_err("a non-READY store short-circuits before the leg check");
        assert_eq!(
            err,
            StoreError::EngineUnavailable,
            "the FS-14 error is NOT what a non-READY store produces"
        );
        assert_eq!(
            server_status(&err).expect("FS-8 is in the §11 map"),
            (503, "engine_unavailable")
        );
    }

    // (c) with a provider supplied the same store DOES build (the contrast that
    // makes the no-build half meaningful).
    let live = U5Provider::new(EmbedMode::Ok);
    assert!(build_boot_vector_index(&store, Some(&live.as_provider()))
        .await
        .expect("a supplied provider builds")
        .is_some());
}

// ---------------------------------------------------------------------------
// State 6 — fail: an unbuilt index asked to serve → FS-14 (§9.5.5's row 6)
// ---------------------------------------------------------------------------

/// Fail-state 6. On a **READY** store with `vectors: None` (a caller-built
/// store) `mode=vector` is still FS-14 `VectorIndexUnavailable` → 503
/// `vector_index_unavailable`: U5 does not retire the variant or its §11 row.
#[tokio::test]
async fn u5_state_6_ready_store_without_index_still_reports_fs14_vector_index_unavailable() {
    let (store, _corpus) = seed_store(&mixed_corpus()).await;
    store.swap_snapshot(DerivedIndexes::default());
    store.set_embedding_provider(U5Provider::new(EmbedMode::Ok).as_provider());
    store.set_engine_state(EngineState::Ready);

    assert!(store.snapshot().vectors.is_none());
    assert!(!store.get_engine_status().await.subsystems.vector);

    let err = store
        .rag_query(
            "alpha",
            &RagQueryOptions {
                top_k: Some(5),
                mode: Some(gnosis::QueryMode::Vector),
                ..Default::default()
            },
        )
        .await
        .expect_err("a READY store with no index is FS-14");
    assert_eq!(err, StoreError::VectorIndexUnavailable);
    assert_eq!(
        server_status(&err).expect("FS-14 is in the §11 map"),
        (503, "vector_index_unavailable")
    );
}

// ---------------------------------------------------------------------------
// State 7 — fail: built index, provider gone at query time → FS-13
// ---------------------------------------------------------------------------

/// Fail-state 7. `vectors: Some(_)` with **no** wired provider is FS-13
/// `EmbeddingUnavailable` → 503: within `mode=vector` the index is checked
/// **before** the provider, and the flag stays the wired capability (§9.5.5's
/// adjudication note 5 — post-boot provider loss is the OPEN `docs/defects.md`
/// row P-8, not U5's).
#[tokio::test]
async fn u5_state_7_built_index_with_no_provider_is_fs13_embedding_unavailable() {
    let (store, corpus) = seed_store(&mixed_corpus()).await;
    let provider = U5Provider::new(EmbedMode::Ok);
    let vi = build_boot_vector_index(&store, Some(&provider.as_provider()))
        .await
        .expect("all-`Ok` embeds")
        .expect("a supplied provider yields `Some`");
    assert_eq!(vi.entries.len(), corpus.embeddable_count());
    store.swap_snapshot(DerivedIndexes {
        vectors: Some(vi),
        ..DerivedIndexes::default()
    });
    // No provider is wired (the query-time provider is gone/unreachable).
    store.set_engine_state(EngineState::Ready);
    let status = store.get_engine_status().await;
    assert!(status.subsystems.vector, "the index IS in the snapshot");
    assert!(
        !status.subsystems.embedding,
        "no provider is wired ⇒ `embedding:false` (the flag is the capability)"
    );

    let err = store
        .rag_query(
            "alpha",
            &RagQueryOptions {
                top_k: Some(5),
                mode: Some(gnosis::QueryMode::Vector),
                ..Default::default()
            },
        )
        .await
        .expect_err("a built index with no provider is FS-13");
    assert_eq!(err, StoreError::EmbeddingUnavailable);
    assert_eq!(
        server_status(&err).expect("FS-13 is in the §11 map"),
        (503, "embedding_unavailable")
    );
}

// ---------------------------------------------------------------------------
// Contract table (1) — the three `BootProvider` inputs
// ---------------------------------------------------------------------------

/// Contract table (1), element-wise: `Reachable` ⇒ built + `Ready` +
/// `vector:true` + `embedding:true`; `Unreachable` ⇒ not built + `Degraded` +
/// `vector:false` + `embedding:false`; `Absent` ⇒ not built + `Unavailable` +
/// `vector:false` + `embedding:false`.
#[tokio::test]
async fn u5_contract_1_boot_input_table_flags() {
    let provider = U5Provider::new(EmbedMode::Ok);

    // `Reachable` over a populated store: BUILT even though nothing forces it.
    let (store, corpus) = seed_store(&mixed_corpus()).await;
    assert!(corpus.embeddable_count() > 0);
    let vi = build_boot_vector_index(&store, Some(&provider.as_provider()))
        .await
        .expect("all-`Ok` embeds")
        .expect("a supplied provider yields `Some`");
    let built_snap = DerivedIndexes {
        vectors: Some(vi),
        ..DerivedIndexes::default()
    };
    let reachable =
        wired_status(&BootProvider::Reachable(provider.as_provider()), built_snap).await;
    assert_eq!(reachable.state, EngineState::Ready);
    assert!(reachable.subsystems.vector && reachable.subsystems.embedding);

    // `Unreachable`: no build, no embedding call attempted.
    let unreachable_provider = U5Provider::new(EmbedMode::Ok);
    let unreachable = wired_status(&BootProvider::Unreachable, DerivedIndexes::default()).await;
    assert_eq!(unreachable.state, EngineState::Degraded);
    assert!(!unreachable.subsystems.vector && !unreachable.subsystems.embedding);
    assert_eq!(
        unreachable_provider.call_count(),
        0,
        "the `Unreachable` boot attempts no embedding call at all"
    );

    // `Absent`: the construction value (`Unavailable`) and the V-8.2 mask.
    let absent = wired_status(&BootProvider::Absent, DerivedIndexes::default()).await;
    assert_eq!(absent.state, EngineState::Unavailable);
    assert_eq!(
        absent.subsystems,
        gnosis::EngineSubsystems {
            store: true,
            graph: true,
            lexical: true,
            vector: false,
            embedding: false,
            reranker: false,
        },
        "the `Absent` boot keeps the pre-U5 V-8.2 mask unchanged"
    );
}

// ---------------------------------------------------------------------------
// Contract table (2) — the corpus rules
// ---------------------------------------------------------------------------

/// Contract table (2): "two nodes carrying the same `value` text are two
/// embeddable nodes ⇒ they get **two** `embed` calls and **two** keys"
/// (REMAND-4 NOTE 5) — the text-keyed-dedup negative probe.
#[tokio::test]
async fn u5_contract_2_corpus_rule_duplicate_text_two_nodes() {
    let (store, corpus) = seed_store(&[vec![
        Some("identical text".to_string()),
        Some("identical text".to_string()),
    ]])
    .await;
    assert_eq!(corpus.embeddable_count(), 2);
    let provider = U5Provider::new(EmbedMode::Ok);

    let vi = build_boot_vector_index(&store, Some(&provider.as_provider()))
        .await
        .expect("all-`Ok` embeds")
        .expect("a supplied provider yields `Some`");
    assert_eq!(
        vi.entries.len(),
        2,
        "a duplicate text under two distinct ids yields TWO keys (no text-keyed dedup)"
    );
    assert_eq!(key_set(&vi), expected_key_set(&corpus));
    assert_eq!(
        provider.call_count(),
        2,
        "two embeddable nodes ⇒ two `embed` calls (the per-node call rule)"
    );
}

/// Contract table (2): the corpus is the **whole store, all wikis** — every node
/// of every document in every shard; the index is node-keyed, not document- or
/// wiki-keyed. `build_boot_vector_index` takes no wiki parameter.
#[tokio::test]
async fn u5_contract_2_corpus_is_store_wide_all_documents() {
    let (store, corpus) = seed_store(&[
        vec![Some("doc-a-1".to_string()), Some("doc-a-2".to_string())],
        vec![Some("doc-b-1".to_string()), None],
        vec![Some(String::new())],
    ])
    .await;
    assert_eq!(corpus.embeddable_count(), 4, "2 + 1 + 1 embeddable nodes");
    let provider = U5Provider::new(EmbedMode::Ok);

    let vi = build_boot_vector_index(&store, Some(&provider.as_provider()))
        .await
        .expect("all-`Ok` embeds")
        .expect("a supplied provider yields `Some`");
    assert_eq!(
        vi.entries.len(),
        corpus.embeddable_count(),
        "entries.len() == the store-wide embeddable count"
    );
    assert_eq!(key_set(&vi), expected_key_set(&corpus));
    let distinct_docs: std::collections::HashSet<&DocumentId> =
        vi.entries.keys().map(|k| &k.0).collect();
    assert_eq!(
        distinct_docs.len(),
        3,
        "all three documents (all shards) contribute their embeddable nodes"
    );
    assert_eq!(provider.call_count(), corpus.embeddable_count());
}

// ---------------------------------------------------------------------------
// Contract table (3) — full field only, dimension not pinned
// ---------------------------------------------------------------------------

/// Contract table (3): U5 builds **`Full` only** (no `Binary`, no `Other`), and
/// the returned `Vec<f32>` is the provider's own value **verbatim** — no
/// normalisation, no truncation, no re-ordering (the dimension is NOT pinned).
#[tokio::test]
async fn u5_contract_3_full_field_only_and_dimension_verbatim() {
    let (store, corpus) = seed_store(&[vec![
        Some("short".to_string()),
        Some("a-much-longer-text-value".to_string()),
    ]])
    .await;
    // A dimension-drifting provider: the returned lengths differ per text, so a
    // normalising/truncating implementation is caught.
    let provider = U5Provider::new(EmbedMode::DimensionDrift);

    let vi = build_boot_vector_index(&store, Some(&provider.as_provider()))
        .await
        .expect("a drifting dimension is not a failure (the dimension is not pinned)")
        .expect("a supplied provider yields `Some`");
    assert_eq!(key_set(&vi), expected_key_set(&corpus));
    for (key, v) in vi.entries.iter() {
        assert_eq!(
            key.2,
            FieldType::Full,
            "no entry key is `Binary`/`Other` — U5 builds the full field only"
        );
        let want = provider.vector_for(text_for(&corpus, key));
        assert_eq!(
            v, &want,
            "the stored vector is the provider's own value verbatim (same length, same elements)"
        );
    }
    assert!(
        !vi.entries
            .keys()
            .any(|k| matches!(k.2, FieldType::Binary | FieldType::Other(_))),
        "the build never emits a `Binary` field entry"
    );
}

/// The corpus text behind an index key (the test's own record of what it seeded).
fn text_for<'a>(corpus: &'a Corpus, key: &(DocumentId, NodeId, FieldType)) -> &'a str {
    corpus
        .nodes
        .iter()
        .enumerate()
        .find(|(i, (d, _))| *d == key.0 && NodeId(node_id(*i)) == key.1)
        .and_then(|(_, (_, v))| v.as_deref())
        .expect("every index key maps to a seeded `value:Some(_)` node")
}

// ---------------------------------------------------------------------------
// Contract table (6) — the journal/epoch/durability guards
// ---------------------------------------------------------------------------

/// Contract table (6): the build appends **no** journal entry, does not move
/// `epoch()`, and does not touch the snapshot (the build returns an index; only
/// the caller installs it through one `swap_snapshot`).
#[tokio::test]
async fn u5_contract_6_build_appends_no_journal_entry_and_moves_no_epoch() {
    let (store, corpus) = seed_store(&mixed_corpus()).await;
    assert!(corpus.embeddable_count() > 0);
    let epoch_before = store.epoch();
    let journal_before = store.journal_len();
    let snap_before = store.snapshot();
    let provider = U5Provider::new(EmbedMode::Ok);

    let vi = build_boot_vector_index(&store, Some(&provider.as_provider()))
        .await
        .expect("all-`Ok` embeds")
        .expect("a supplied provider yields `Some`");
    assert_eq!(vi.entries.len(), corpus.embeddable_count());

    assert_eq!(
        store.epoch(),
        epoch_before,
        "the build does not move epoch()"
    );
    assert_eq!(
        store.journal_len(),
        journal_before,
        "the build appends NO journal entry"
    );
    assert!(
        Arc::ptr_eq(&store.snapshot(), &snap_before),
        "the build does not swap the snapshot — the caller installs the one composed value"
    );
    assert!(
        store.snapshot().vectors.is_none(),
        "the store's own snapshot is untouched by a bare build"
    );
    assert!(
        store.embedding_provider().is_none(),
        "the build never wires a provider onto the store"
    );
}

// ---------------------------------------------------------------------------
// The `is_available` seam is never consulted by the build
// ---------------------------------------------------------------------------

/// §9.5.5 surface #1: the build "makes **no** availability probe of its own"
/// (the boot's probe already produced the `BootProvider`). A provider whose
/// `is_available()` lies (`true` while every `embed` errors) changes nothing
/// about the build's outcome — and in particular never turns the `Err` into an
/// `Ok`.
#[tokio::test]
async fn u5_contract_build_never_consults_is_available() {
    let (store, corpus) = seed_store(&mixed_corpus()).await;
    assert!(corpus.embeddable_count() > 0);

    let honest = U5Provider::new(EmbedMode::Ok);
    let _ = build_boot_vector_index(&store, Some(&honest.as_provider()))
        .await
        .expect("all-`Ok` embeds")
        .expect("a supplied provider yields `Some`");
    assert_eq!(
        honest.availability_probes(),
        0,
        "the build consults no availability probe of its own"
    );

    let liar = U5Provider::new(EmbedMode::Lie);
    let err = build_boot_vector_index(&store, Some(&liar.as_provider()))
        .await
        .expect_err("an `embed` failure is an error regardless of a lying `is_available`");
    assert_eq!(err, StoreError::EmbeddingUnavailable);
    assert_eq!(
        liar.availability_probes(),
        0,
        "a lying `is_available` changes nothing about the build's outcome"
    );
}

// ---------------------------------------------------------------------------
// The three-valued return shape is closed (no new error, no panic)
// ---------------------------------------------------------------------------

/// §9.5.5's "no invention" clause: the whole outcome space is closed — `Ok(None)`
/// with no provider, `Ok(Some(_))` with a provider whose calls all succeed, and
/// `Err(StoreError::EmbeddingUnavailable)` when one fails. No other variant ever
/// appears (a new error type would be an invention), and the call never panics
/// for `P-TP-5`'s adversarial provider shapes.
#[tokio::test]
async fn u5_three_valued_outcome_is_closed_for_adversarial_providers() {
    let (store, corpus) = seed_store(&mixed_corpus()).await;
    let n = corpus.embeddable_count();
    assert!(n > 0);
    assert!(
        build_boot_vector_index(&store, None)
            .await
            .unwrap()
            .is_none(),
        "`p == None` ⇒ `Ok(None)`"
    );

    for mode in [
        EmbedMode::Ok,
        EmbedMode::DimensionDrift,
        EmbedMode::Nan,
        EmbedMode::EmptyVec,
        EmbedMode::Lie,
        EmbedMode::AlwaysErr,
        EmbedMode::FailAt(1),
        EmbedMode::FailAt(n),
    ] {
        let provider = U5Provider::new(mode);
        match build_boot_vector_index(&store, Some(&provider.as_provider())).await {
            Ok(Some(vi)) => assert_eq!(
                vi.entries.len(),
                n,
                "an `Ok(Some(_))` always covers exactly the corpus ({mode:?}); \
                 a partially-filled index presented as `Ok(Some(_))` fails here"
            ),
            Ok(None) => panic!("`Ok(None)` is never the answer for a SUPPLIED provider ({mode:?})"),
            Err(e) => assert_eq!(
                e,
                StoreError::EmbeddingUnavailable,
                "the only error is the existing `EmbeddingUnavailable` ({mode:?})"
            ),
        }
    }
}
