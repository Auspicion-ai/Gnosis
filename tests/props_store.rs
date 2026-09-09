//! §4.1 Document-store **property-based-testing (PBT) gate — executed layer**.
//!
//! Implements **every** register row of `docs/specs/4-1-store-property-register.md`
//! (8 rows: P-IM-1/2, P-SM-1/2/3, P-TP-1/2/3) as **one `#[test]` per row**, each
//! backed by a tiny **deterministic property harness** (`Xoshiro256**` seeded by
//! `SplitMix64`). No third-party PBT crate is introduced; `dependencies` are
//! unchanged. Property register artifact **#2** of the §4.1 unit.
//!
//! ## Pinned deterministic seed
//!
//! Every row derives its `Rng` from a **single fixed master seed** (below) mixed
//! with a per-row tag, so the whole layer is reproducible end to end:
//!
//! ```text
//! const SEED: u64 = 0x9E37_79B9_7F4A_7C15;  // SplitMix golden-ratio constant
//! ```
//!
//! Per-row sub-seed = `splitmix64(SEED ^ row_tag)` so rows do not emit identical
//! sequences. The row tags are `P-IM-1 … P-TP-3`.
//!
//! ## Budget (register gate: ≤100 cases/row, ≤400 total across the layer)
//!
//! | Row     | Budget | Row     | Budget |
//! |---------|--------|---------|--------|
//! | P-IM-1  | 50     | P-TP-1  | 60     |
//! | P-IM-2  | 20     | P-TP-2  | 60     |
//! | P-SM-1  | 40     | P-TP-3  | 40     |
//! | P-SM-2  | 40     | P-SM-3  | 50     |
//!
//! Sum = **360** generated cases (≤ 400). stop-after-5 holds per row: any broken
//! row retains at most 5 distinct minimal counterexamples and stops.
//!
//! Every `DocumentId`/`WikiId` is created through the public API against a
//! **pre-existing wiki**, so a success is unambiguously the invariant holding
//! (never masked by `WikiNotFound`).

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use gnosis::{
    CreateDocumentRequest, DocState, DocumentId, Edge, EdgeKind, Graph, ListDocumentsFilter, Node,
    NodeId, NodeKind, RagStore, ReferenceState, Store, StoreError, UpdateDocumentRequest, WikiId,
};
use tokio::sync::Barrier;

// ---------------------------------------------------------------------------
// Deterministic PRNG: SplitMix64-seeded Xoshiro256** (hand-rolled, no crate).
// ---------------------------------------------------------------------------

/// Master deterministic seed for the whole property layer (recorded in header).
const SEED: u64 = 0x9E37_79B9_7F4A_7C15;

/// Per-row budget caps (sum = 360 ≤ 400).
const B_IM1: u32 = 50;
const B_IM2: u32 = 20;
const B_SM1: u32 = 40;
const B_SM2: u32 = 40;
const B_SM3: u32 = 50;
const B_TP1: u32 = 60;
const B_TP2: u32 = 60;
const B_TP3: u32 = 40;

/// SplitMix64 (also serves as the Xoshiro seed generator).
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Xoshiro256** — deterministic, dependency-free.
struct Rng {
    s: [u64; 4],
}

impl Rng {
    fn seeded(seed: u64) -> Self {
        let mut sm = seed;
        Rng {
            s: [
                splitmix64(&mut sm),
                splitmix64(&mut sm),
                splitmix64(&mut sm),
                splitmix64(&mut sm),
            ],
        }
    }

    fn rotl(x: u64, k: u32) -> u64 {
        x.rotate_left(k)
    }

    fn next(&mut self) -> u64 {
        let result = Self::rotl(self.s[1].wrapping_mul(5), 7).wrapping_mul(9);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[1] ^= t;
        self.s[2] = Self::rotl(self.s[2], 16);
        result
    }

    /// Uniform `[0, n)` (n == 0 ⇒ 0).
    fn below(&mut self, n: u64) -> u64 {
        if n == 0 {
            0
        } else {
            self.next() % n
        }
    }

    fn pick<T: Copy>(&mut self, items: &[T]) -> T {
        items[self.below(items.len() as u64) as usize]
    }

    fn yes(&mut self) -> bool {
        self.next() & 1 == 1
    }
}

/// Per-row deterministic sub-seed from the master seed + a row tag.
fn row_seed(tag: u64) -> u64 {
    splitmix64(&mut (SEED ^ tag))
}

// ---------------------------------------------------------------------------
// Sanctioned fixtures (copied verbatim from `tests/store_integration.rs`).
// ---------------------------------------------------------------------------

fn doc_id(id: &str) -> DocumentId {
    DocumentId(id.to_string())
}

/// A minimal **valid** Provident graph: one content node + exactly one
/// `doc-head` and one `doc-end` edge (§4.2.2).
fn valid_graph() -> Graph {
    let n1 = Node {
        document_id: doc_id("d1"),
        node_id: NodeId("n1".into()),
        kind: NodeKind::Content,
        value: Some("hello".into()),
        fact_key: None,
        target: None,
    };
    Graph {
        nodes: vec![n1.clone()],
        edges: vec![
            Edge {
                source: (doc_id("d1"), NodeId("ROOT".into())),
                target: (doc_id("d1"), n1.node_id.clone()),
                kind: EdgeKind::DocHead,
                state: None,
                cross_wiki: false,
                relation_type: None,
            },
            Edge {
                source: (doc_id("d1"), n1.node_id.clone()),
                target: (doc_id("d1"), NodeId("END".into())),
                kind: EdgeKind::DocEnd,
                state: None,
                cross_wiki: false,
                relation_type: None,
            },
        ],
    }
}

/// **Audit fixture-mismatch fix (#5).** A minimal **valid** Provident graph
/// §4.2.2) stamped for the **real target document id** — fixes the sanctioned
/// `valid_graph()` which stamps node/edge `document_id` as `"d1"` while being
/// applied to a real `doc-N` id. Graph-bearing probes use this (or
/// `graph_with_link_to`) so node ids match the real document.
fn valid_graph_for(id: &DocumentId) -> Graph {
    valid_graph_for_value(id, "hello")
}

/// Like `valid_graph_for` but with a caller-chosen content-node **value**, so two
/// graph bodies applied to the same document id are distinguishable. Used by the
/// graph-payload race probe (winner's node value must persist, loser's must not).
fn valid_graph_for_value(id: &DocumentId, value: &str) -> Graph {
    let n1 = Node {
        document_id: id.clone(),
        node_id: NodeId("n1".into()),
        kind: NodeKind::Content,
        value: Some(value.into()),
        fact_key: None,
        target: None,
    };
    Graph {
        nodes: vec![n1.clone()],
        edges: vec![
            Edge {
                source: (id.clone(), NodeId("ROOT".into())),
                target: (id.clone(), n1.node_id.clone()),
                kind: EdgeKind::DocHead,
                state: None,
                cross_wiki: false,
                relation_type: None,
            },
            Edge {
                source: (id.clone(), n1.node_id.clone()),
                target: (id.clone(), NodeId("END".into())),
                kind: EdgeKind::DocEnd,
                state: None,
                cross_wiki: false,
                relation_type: None,
            },
        ],
    }
}

/// A valid Provident graph for `self_id` carrying one `link` **reference** edge
/// to a node in `target_id`'s document (node ids stamped for the real docs).
/// Makes `target_id` "in use" for the §4.4.5 delete gate, and gives
/// `set_reference_state` (§4.2.8.5) a reference edge to annotate.
fn graph_with_link_to(self_id: &DocumentId, target_id: &DocumentId) -> Graph {
    let n1 = Node {
        document_id: self_id.clone(),
        node_id: NodeId("n1".into()),
        kind: NodeKind::Content,
        value: Some("hello".into()),
        fact_key: None,
        target: None,
    };
    let ref1 = Node {
        document_id: self_id.clone(),
        node_id: NodeId("ref1".into()),
        kind: NodeKind::Reference,
        value: None,
        fact_key: None,
        target: Some((target_id.clone(), NodeId("n1".into()))),
    };
    Graph {
        nodes: vec![n1.clone(), ref1.clone()],
        edges: vec![
            Edge {
                source: (self_id.clone(), NodeId("ROOT".into())),
                target: (self_id.clone(), n1.node_id.clone()),
                kind: EdgeKind::DocHead,
                state: None,
                cross_wiki: false,
                relation_type: None,
            },
            Edge {
                source: (self_id.clone(), n1.node_id.clone()),
                target: (self_id.clone(), NodeId("END".into())),
                kind: EdgeKind::DocEnd,
                state: None,
                cross_wiki: false,
                relation_type: None,
            },
            Edge {
                source: (self_id.clone(), ref1.node_id.clone()),
                target: (target_id.clone(), NodeId("n1".into())),
                kind: EdgeKind::Link,
                state: Some(ReferenceState::Resolved),
                cross_wiki: false,
                relation_type: None,
            },
        ],
    }
}

/// Recurring request/response builders scoped to §4.1 request shapes.
fn create_req(title: &str, tags: Option<Vec<String>>) -> CreateDocumentRequest {
    CreateDocumentRequest {
        title: title.to_string(),
        tags,
        author: None,
    }
}

fn create_req_author(
    title: &str,
    tags: Option<Vec<String>>,
    author: &str,
) -> CreateDocumentRequest {
    CreateDocumentRequest {
        title: title.to_string(),
        tags,
        author: Some(author.to_string()),
    }
}

fn graph_only_update(base: u64) -> UpdateDocumentRequest {
    UpdateDocumentRequest {
        base_revision: base,
        graph: valid_graph(),
        title: None,
        tags: None,
    }
}

fn title_update(title: &str, base: u64) -> UpdateDocumentRequest {
    UpdateDocumentRequest {
        base_revision: base,
        graph: valid_graph(),
        title: Some(title.to_string()),
        tags: None,
    }
}

fn tags_update(tags: &[&str], base: u64) -> UpdateDocumentRequest {
    UpdateDocumentRequest {
        base_revision: base,
        graph: valid_graph(),
        title: None,
        tags: Some(tags.iter().map(|s| s.to_string()).collect()),
    }
}

/// Create a wiki (pre-exists so doc ops on it are unambiguously success).
async fn new_wiki(store: &Store, name: &str) -> WikiId {
    store.create_wiki(name).await.unwrap().wiki_id
}

/// Create one document on `w` via the public API.
async fn create_doc(store: &Store, w: &WikiId, req: CreateDocumentRequest) -> gnosis::Document {
    store
        .create_document(w, req)
        .await
        .expect("create must succeed against a pre-existing wiki")
}

// ---------------------------------------------------------------------------
// P-IM-1 (IM) — strat:create-roundtrip-id — identity stability + zero reuse.
// ---------------------------------------------------------------------------
//
// States generated: same-wiki creates (ids distinct), delete-a-middle then
// re-create (new id differs), get interleaved with unrelated updates,
// empty-wiki first create, multiple wikis (an id never resolves to a doc in a
// different wiki than it was created in).
#[tokio::test]
async fn p_im_1_create_roundtrip_id() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "P-IM-1-wiki").await;
    let w2 = new_wiki(&store, "P-IM-1-wiki-2").await;
    let mut rng = Rng::seeded(row_seed(0x494D31)); // "P-IM-1"

    let mut created: Vec<DocumentId> = Vec::new();
    let mut deleted: Vec<DocumentId> = Vec::new();
    let mut all: HashSet<DocumentId> = HashSet::new();
    let mut cases: u32 = 0;
    // Cross-wiki ids: created in w2, must always resolve to w2.
    let mut w2_ids: HashSet<DocumentId> = HashSet::new();
    let mut counter: u64 = 0;

    // Empty-wiki first create.
    let first = create_doc(&store, &w, create_req("first", Some(vec!["t".into()]))).await;
    cases += 1;
    assert!(
        all.insert(first.document_id.clone()),
        "first id must be brand new"
    );
    assert_eq!(
        store
            .get_document(&first.document_id)
            .await
            .unwrap()
            .document_id,
        first.document_id
    );
    assert_eq!(
        store
            .get_document(&first.document_id)
            .await
            .unwrap()
            .wiki_id,
        w
    );
    created.push(first.document_id.clone());

    // A few docs in the second wiki for the cross-wiki check.
    for i in 0..3u64 {
        let d = create_doc(&store, &w2, create_req(&format!("w2-{i}"), None)).await;
        cases += 1;
        w2_ids.insert(d.document_id.clone());
        assert_eq!(
            store.get_document(&d.document_id).await.unwrap().wiki_id,
            w2
        );
    }

    let rounds: u32 = 12;
    for _ in 0..rounds {
        // Create on the main wiki and round-trip.
        counter += 1;
        let d = create_doc(
            &store,
            &w,
            create_req(&format!("t-{counter}"), Some(vec!["x".into()])),
        )
        .await;
        cases += 1;
        assert!(
            all.insert(d.document_id.clone()),
            "engine-assigned DocumentId must never be reassigned (distinct within run)"
        );
        let fetched = store.get_document(&d.document_id).await.unwrap();
        assert_eq!(
            fetched.document_id, d.document_id,
            "DocumentId must resolve to the same document"
        );
        assert_eq!(
            fetched.wiki_id, w,
            "a created id must never resolve to a different wiki"
        );
        created.push(d.document_id.clone());

        // Interleave an unrelated update to the newest-but-one document.
        if created.len() > 1 {
            let some = created[created.len() - 2].clone();
            store
                .update_document(&some, graph_only_update(0))
                .await
                .unwrap();
            let still = store.get_document(&some).await.unwrap();
            assert_eq!(
                still.document_id, some,
                "live id still resolves while others are updated"
            );
        }

        // Occasionally delete an existing (not the newest) document.
        if rng.yes() && created.len() > 2 {
            let idx = rng.below((created.len() - 1) as u64) as usize; // not the newest
            let id = created.remove(idx);
            store.delete_document(&id).await.unwrap();
            cases += 1;
            assert_eq!(
                store.get_document(&id).await.unwrap_err(),
                StoreError::DocumentNotFound,
                "deleted id must be gone"
            );
            deleted.push(id.clone());
        }

        // Occasionally re-create after a delete: the new id must differ from all
        // previously deleted ids and all live ids.
        if rng.yes() && !deleted.is_empty() {
            let d2 = create_doc(&store, &w, create_req("re-created", None)).await;
            cases += 1;
            assert!(
                deleted.iter().all(|d| d != &d2.document_id),
                "a re-created document must receive an id strictly different from every previously deleted id"
            );
            assert!(
                created.iter().all(|c| c != &d2.document_id) && all.insert(d2.document_id.clone()),
                "re-created id must be brand-new and distinct from all live ids"
            );
            created.push(d2.document_id.clone());
        }
    }

    // No id ever resolves to a document in a different wiki than it was created in.
    for w2id in &w2_ids {
        let d = store.get_document(w2id).await.unwrap();
        assert_eq!(d.wiki_id, w2, "w2 id must resolve to w2, never w");
    }
    assert!(cases <= B_IM1, "B_IM1 budget exceeded: {cases}");
    println!("[P-IM-1] generated cases: {cases}");
}

// ---------------------------------------------------------------------------
// P-IM-2 (IM) — strat:boundaries — inclusive bounds, byte-for-byte round-trip.
// ---------------------------------------------------------------------------
//
// Boundary shapes: title len {1,199,200,201, 200-astral, 200-whitespace,
// 0(empty-reject)}, wiki name len {1,99,100,101}. Bounds by `chars().count()`.
#[tokio::test]
async fn p_im_2_boundaries() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "P-IM-2-wiki").await;
    let mut rng = Rng::seeded(row_seed(0x494D32)); // "P-IM-2"
    let mut cases: u32 = 0;

    // ---- Document title boundaries ----
    let cases_valid_title: Vec<(String, usize, usize)> = vec![
        ("t".to_string(), 1, 1),
        ("t".repeat(199), 199, 1),
        ("t".repeat(200), 200, 1),
        ("\u{1F600}".repeat(200), 200, 1), // 200 astral chars → 200 chars
        ("  ".repeat(100), 200, 1),        // whitespace-only, valid length
        ("\u{C1C1}\u{6D6D}".repeat(100), 200, 1), // multi-byte BMP astral mix → 200 chars
    ];

    for (title, expect_len, _) in cases_valid_title {
        // Deterministically shuffle the valid-title set ordering via rng.
        let _ = rng.below(3);
        assert_eq!(title.chars().count(), expect_len, "premise");
        let doc = create_doc(&store, &w, create_req(&title, None)).await;
        let fetched = store.get_document(&doc.document_id).await.unwrap();
        assert_eq!(
            fetched.title.chars().count(),
            expect_len,
            "round-trip length"
        );
        assert_eq!(fetched.title, title, "title round-trips byte-for-byte");
        cases += 1;
    }

    // Title 201 → ValidationError (inclusive upper bound 200, exclusive ≥201).
    let over = "t".repeat(201);
    let err = store
        .create_document(&w, create_req(&over, None))
        .await
        .expect_err("201-char title must be rejected");
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "P-IM-2 title>200"
    );
    cases += 1;

    // Empty title → ValidationError (§4.1.3 non-empty).
    let err = store
        .create_document(&w, create_req("", None))
        .await
        .expect_err("empty title must be rejected");
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "P-IM-2 empty title"
    );
    cases += 1;

    // Very-large (≫200) title → ValidationError.
    let huge = "t".repeat(5000);
    let err = store
        .create_document(&w, create_req(&huge, None))
        .await
        .expect_err("huge title must be rejected");
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "P-IM-2 huge title"
    );
    cases += 1;

    // ---- Wiki name boundaries ----
    let w100 = "n".repeat(100);
    assert_eq!(w100.chars().count(), 100);
    let wwiki = store.create_wiki(&w100).await.unwrap();
    assert_eq!(
        store.get_wiki(&wwiki.wiki_id).await.unwrap().name,
        w100,
        "name round-trip"
    );
    cases += 1;

    for name_len in [1usize, 99usize] {
        let name = "n".repeat(name_len);
        let wk = store.create_wiki(&name).await.unwrap();
        assert_eq!(store.get_wiki(&wk.wiki_id).await.unwrap().name, name);
        cases += 1;
    }

    let name101 = "n".repeat(101);
    let err = store
        .create_wiki(&name101)
        .await
        .expect_err("101-char name must be rejected");
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "P-IM-2 name>100"
    );
    cases += 1;

    let err = store
        .create_wiki("")
        .await
        .expect_err("empty wiki name rejected");
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "P-IM-2 empty name"
    );
    cases += 1;

    let _ = rng.yes(); // keep rng used/deterministic bookkeeping
    assert!(cases <= B_IM2, "B_IM2 budget exceeded: {cases}");
    println!("[P-IM-2] generated cases: {cases}");
}

// ---------------------------------------------------------------------------
// P-SM-1 (SM) — strat:create-update-seq — revision monotonic, steps exactly 1.
// ---------------------------------------------------------------------------
//
// States: sequence lengths {0,1,2,N}; base always exactly the last committed
// revision; alternating graph-only and title-bearing updates; a long monotonic
// ladder.
#[tokio::test]
async fn p_sm_1_create_update_seq() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "P-SM-1-wiki").await;
    let mut rng = Rng::seeded(row_seed(0x534D31)); // "P-SM-1"
    let mut cases: u32 = 0;

    // Length-0 sequence: create only, rev 0.
    let d0 = create_doc(&store, &w, create_req("base", None)).await;
    assert_eq!(d0.revision, 0);
    cases += 1;

    // Length-1: base 0 → rev 1.
    let d1 = create_doc(&store, &w, create_req("one", None)).await;
    let u1 = store
        .update_document(&d1.document_id, title_update("one-v", 0))
        .await
        .unwrap();
    assert_eq!(u1.revision, 1, "base 0 → rev 1");
    cases += 1;

    // Length-2.
    let d2 = create_doc(&store, &w, create_req("two", None)).await;
    let a = store
        .update_document(&d2.document_id, title_update("two-a", 0))
        .await
        .unwrap();
    assert_eq!(a.revision, 1);
    let b = store
        .update_document(&d2.document_id, tags_update(&["b"], 1))
        .await
        .unwrap();
    assert_eq!(b.revision, 2, "base 1 → rev 2");
    cases += 2;

    // N-length ladders (mix of graph-only / title-bearing), N up to 8.
    for ladder in 0..4u64 {
        let n = 3 + rng.below(6); // length 3..=8
        let d = create_doc(&store, &w, create_req(&format!("ladder-{ladder}"), None)).await;
        let mut prior = d.revision;
        let mut last_rev = prior;
        let mut steps: u64 = 0;
        while steps < n {
            let base = last_rev;
            let req = if rng.yes() {
                title_update(&format!("l-{ladder}-{steps}"), base)
            } else {
                graph_only_update(base)
            };
            let updated = store.update_document(&d.document_id, req).await.unwrap();
            assert_eq!(
                updated.revision,
                base + 1,
                "revision must step by exactly one per committed update (base {base})"
            );
            assert!(updated.revision > last_rev, "revision must never decrease");
            last_rev = updated.revision;
            assert_eq!(prior + 1, updated.revision);
            prior = updated.revision;
            steps += 1;
            cases += 1;
        }
        // Final fetched revision equals last returned revision (monotonic steady-state).
        let fetched = store.get_document(&d.document_id).await.unwrap();
        assert_eq!(fetched.revision, last_rev);
    }

    assert!(cases <= B_SM1, "B_SM1 budget exceeded: {cases}");
    println!("[P-SM-1] generated cases: {cases}");
}

// ---------------------------------------------------------------------------
// P-SM-2 (SM) — strat:revision-race-seqs — exactly-one-winner concurrency.
// ---------------------------------------------------------------------------
// Real contention: shared `Arc<Store>`, barrier-released on a multi-thread
// runtime. Two-way and three-way same-base races, payload-persistence of the
// winner, and a repeated race re-based on the newly committed revision.

async fn run_race(
    store: &Arc<Store>,
    id: &DocumentId,
    base: u64,
    payloads: Vec<String>,
) -> (u32, u32, u32, Vec<Result<gnosis::Document, StoreError>>) {
    let n = payloads.len();
    let barrier = Arc::new(Barrier::new(n));
    let mut handles = Vec::new();
    for title in payloads {
        let store = store.clone();
        let barrier = barrier.clone();
        let id = id.clone();
        handles.push(tokio::spawn(async move {
            barrier.wait().await; // genuinely contend
            store
                .update_document(
                    &id,
                    UpdateDocumentRequest {
                        base_revision: base,
                        graph: valid_graph(),
                        title: Some(title),
                        tags: None,
                    },
                )
                .await
        }));
    }
    let mut results = Vec::new();
    let (mut ok, mut conflict, mut other) = (0u32, 0u32, 0u32);
    for h in handles {
        let r = h.await.expect("no task panic");
        match &r {
            Ok(_) => ok += 1,
            Err(StoreError::ConflictError) => conflict += 1,
            Err(_) => other += 1,
        }
        results.push(r);
    }
    (ok, conflict, other, results)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn p_sm_2_revision_race_seqs() {
    let store: Arc<Store> = Arc::new(Store::new());
    let w = new_wiki(&store, "P-SM-2-wiki").await;
    let mut rng = Rng::seeded(row_seed(0x534D32)); // "P-SM-2"
    let mut cases: u32 = 0;

    // Race on a fresh rev-0 doc.
    for round in 0..4u64 {
        let d = create_doc(&store, &w, create_req(&format!("race0-{round}"), None)).await;
        let base = d.revision; // 0
        let payloads = vec![format!("A-{round}"), format!("B-{round}")];
        let (ok, conflict, other, results) = run_race(&store, &d.document_id, base, payloads).await;
        assert_eq!(ok, 1, "exactly one winner");
        assert_eq!(conflict, 1, "exactly one ConflictError");
        assert_eq!(other, 0, "no other error / panic");
        let winner = results
            .iter()
            .find_map(|r| r.as_ref().ok())
            .expect("winner exists");
        assert_eq!(winner.revision, base + 1);
        let final_doc = store.get_document(&d.document_id).await.unwrap();
        assert_eq!(
            final_doc.revision,
            base + 1,
            "stored revision advances by exactly one"
        );
        assert_eq!(
            final_doc.title, winner.title,
            "the winner's payload is the one that persists"
        );
        cases += 1;
    }

    // Three-way same-base race: exactly one winner, two ConflictError.
    let d = create_doc(&store, &w, create_req("three-way", None)).await;
    let base = d.revision;
    let (ok, conflict, other, results) = run_race(
        &store,
        &d.document_id,
        base,
        vec!["X".into(), "Y".into(), "Z".into()],
    )
    .await;
    assert_eq!(ok, 1, "three-way: exactly one winner");
    assert_eq!(conflict, 2, "three-way: two ConflictError");
    assert_eq!(other, 0);
    let winner = results
        .iter()
        .find_map(|r| r.as_ref().ok())
        .expect("winner");
    assert_eq!(winner.revision, base + 1);
    let final_doc = store.get_document(&d.document_id).await.unwrap();
    assert_eq!(final_doc.revision, base + 1);
    assert_eq!(final_doc.title, winner.title);
    cases += 1;

    // Repeated race re-based on the newly committed revision: after race1 at
    // base R, re-read rev R+1 and race again at that base.
    let d = create_doc(&store, &w, create_req("rebased", None)).await;
    let base0 = d.revision;
    for k in 0..3u64 {
        let (ok, conflict, other, _results) = run_race(
            &store,
            &d.document_id,
            base0 + k,
            vec![format!("r{k}a"), format!("r{k}b")],
        )
        .await;
        assert_eq!(ok, 1);
        assert_eq!(conflict, 1);
        assert_eq!(other, 0);
        let after = store.get_document(&d.document_id).await.unwrap();
        assert_eq!(
            after.revision,
            base0 + k + 1,
            "each race advances by exactly one"
        );
        cases += 1;
    }
    let _ = rng.yes(); // keep deterministic bookkeeping

    assert!(cases <= B_SM2, "B_SM2 budget exceeded: {cases}");
    println!("[P-SM-2] generated races: {cases}");
}

// ---------------------------------------------------------------------------
// P-SM-3 (SM) — strat:state-machine-seq — §4.1.1 transition legality + sink.
// ---------------------------------------------------------------------------
// Full 3×3 `can_transition_to` matrix vs the legal graph; DRAFT→PUBLISHED→DRAFT→ARCHIVED;
// DRAFT→ARCHIVED terminal; PUBLISHED→ARCHIVED; repeated ops after ARCHIVED no-op.

fn legal_transition(a: DocState, b: DocState) -> bool {
    match a {
        DocState::Draft => matches!(b, DocState::Published | DocState::Archived),
        DocState::Published => matches!(b, DocState::Draft | DocState::Archived),
        DocState::Archived => false,
    }
}

#[tokio::test]
async fn p_sm_3_state_machine_seq() {
    let store: Arc<Store> = Arc::new(Store::new());
    let w = new_wiki(&store, "P-SM-3-wiki").await;
    let mut rng = Rng::seeded(row_seed(0x534D33)); // "P-SM-3"
    let mut cases: u32 = 0;

    // Full 3×3 matrix — `can_transition_to` encodes exactly the legal graph.
    for a in [DocState::Draft, DocState::Published, DocState::Archived] {
        for b in [DocState::Draft, DocState::Published, DocState::Archived] {
            assert_eq!(
                a.can_transition_to(b),
                legal_transition(a, b),
                "can_transition_to({a:?},{b:?}) must match the legal graph"
            );
            cases += 1;
        }
    }

    // Helper: drive a fresh document along a legal prefix, checking each step.
    async fn drive(
        store: &Arc<Store>,
        w: &WikiId,
        ops: &[DocState], // the sequence of target states to attempt in order
    ) -> (DocState, u32, Vec<String>) {
        let d = create_doc(store, w, create_req("sm", None)).await;
        let mut state = DocState::Draft;
        let mut cases: u32 = 0;
        let mut cexes: Vec<String> = Vec::new();
        for &target in ops {
            let (res, next) = match target {
                DocState::Published => (
                    store.publish_document(&d.document_id).await,
                    DocState::Published,
                ),
                DocState::Draft => (
                    store.unpublish_document(&d.document_id).await,
                    DocState::Draft,
                ),
                DocState::Archived => (
                    store.archive_document(&d.document_id).await,
                    DocState::Archived,
                ),
            };
            let legal = state.can_transition_to(next);
            match res {
                Ok(committed) => {
                    if !legal {
                        cexes.push(format!("illegal committed {state:?}->{next:?}"));
                    }
                    assert!(
                        legal,
                        "a committed transition must be legal: {state:?}->{next:?}"
                    );
                    assert_eq!(committed.state, next);
                    assert_eq!(
                        store.get_document(&d.document_id).await.unwrap().state,
                        next
                    );
                    state = next;
                }
                Err(e) => {
                    // In the sink, every subsequent op must fail and commit nothing.
                    if state == DocState::Archived {
                        assert!(
                            matches!(e, StoreError::InvalidState),
                            "sink must reject with InvalidState"
                        );
                        assert_eq!(
                            store.get_document(&d.document_id).await.unwrap().state,
                            DocState::Archived
                        );
                    } else {
                        assert!(
                            !legal,
                            "an out-of-register error for a legal transition: {e:?}"
                        );
                        if let Some(msg) = cex_msg(&e, state, next) {
                            cexes.push(msg);
                        }
                    }
                }
            }
            cases += 1;
            if state == DocState::Archived {
                break; // terminal sink reached.
            }
        }
        (state, cases, cexes)
    }

    // DRAFT→PUBLISHED→DRAFT→ARCHIVED (all legal).
    let (st, c, cex) = drive(
        &store,
        &w,
        &[DocState::Published, DocState::Draft, DocState::Archived],
    )
    .await;
    assert_eq!(st, DocState::Archived, "reached terminal sink");
    assert!(cex.is_empty(), "unexpected cexes: {cex:?}");
    cases += c;

    // DRAFT→ARCHIVED (terminal; further ops no-op).
    let d = create_doc(&store, &w, create_req("sink", None)).await;
    let r = store.archive_document(&d.document_id).await.unwrap();
    assert_eq!(r.state, DocState::Archived);
    cases += 1;
    for op in [0u8, 1, 2] {
        let res = match op {
            0 => store.publish_document(&d.document_id).await,
            1 => store.unpublish_document(&d.document_id).await,
            _ => store.archive_document(&d.document_id).await,
        };
        assert!(
            matches!(res, Err(StoreError::InvalidState)),
            "sink must reject all ops"
        );
        assert_eq!(
            store.get_document(&d.document_id).await.unwrap().state,
            DocState::Archived
        );
        cases += 1;
    }

    // PUBLISHED→ARCHIVED.
    let (st, c, cex) = drive(&store, &w, &[DocState::Published, DocState::Archived]).await;
    assert_eq!(st, DocState::Archived);
    assert!(cex.is_empty(), "unexpected cexes: {cex:?}");
    cases += c;

    // Random legal walks of length 1..=5 (each step a legal transition).
    for _ in 0..4u64 {
        let n = 1 + rng.below(5);
        let mut ops: Vec<DocState> = Vec::new();
        let mut state = DocState::Draft;
        for _ in 0..n {
            let choices: Vec<DocState> = match state {
                DocState::Draft => vec![DocState::Published, DocState::Archived],
                DocState::Published => vec![DocState::Draft, DocState::Archived],
                DocState::Archived => vec![],
            };
            if choices.is_empty() {
                break;
            }
            let target = rng.pick(&choices);
            ops.push(target);
            state = target;
            if state == DocState::Archived {
                break;
            }
        }
        let (st, c, cex) = drive(&store, &w, &ops).await;
        let _ = st;
        assert!(cex.is_empty(), "unexpected cexes: {cex:?}");
        cases += c;
    }

    assert!(cases <= B_SM3, "B_SM3 budget exceeded: {cases}");
    println!("[P-SM-3] generated cases: {cases}");
}

/// Minimal human-readable counterexample text for a rejected transition.
fn cex_msg(e: &StoreError, from: DocState, to: DocState) -> Option<String> {
    Some(format!(
        "expected Ok but got {e:?} for legal {from:?}->{to:?}"
    ))
}

// ---------------------------------------------------------------------------
// P-TP-1 (TP) — strat:pagination-pages — lossless, exhaustive, duplicate-free.
// ---------------------------------------------------------------------------
// Walks pages at fixed page_size over a static wiki; set coverage, no dups,
// stable `total`. Also empty wiki, page beyond last, page_size ∈ {1, 100},
// and a state/tag *filtered* page set.

async fn collect_pages(
    store: &Arc<Store>,
    w: &WikiId,
    filter: &ListDocumentsFilter,
    page_size: u64,
    total_expected: u64,
) -> (Vec<DocumentId>, u32) {
    let mut seen: HashSet<DocumentId> = HashSet::new();
    let mut list = Vec::new();
    let mut cases: u32 = 0;
    let pages = if total_expected == 0 {
        1
    } else {
        total_expected.div_ceil(page_size)
    };
    for page in 1..=pages {
        let mut f = filter.clone();
        f.page = Some(page);
        f.page_size = Some(page_size);
        let res = store.list_documents(w, &f).await.unwrap();
        assert_eq!(res.total, total_expected, "total identical on every page");
        assert_eq!(res.page, page);
        assert_eq!(res.page_size, page_size);
        for item in &res.items {
            assert!(
                seen.insert(item.document_id.clone()),
                "no id may appear twice across pages"
            );
            assert_ne!(res.items.len() as u64, 0);
        }
        list.extend(res.items.into_iter().map(|s| s.document_id));
        cases += 1;
    }
    (list, cases)
}

#[tokio::test]
async fn p_tp_1_pagination_pages() {
    let store: Arc<Store> = Arc::new(Store::new());
    let w = new_wiki(&store, "P-TP-1-wiki").await;
    let mut rng = Rng::seeded(row_seed(0x545031)); // "P-TP-1"
    let mut cases: u32 = 0;

    // Empty wiki: total 0, page 1 empty.
    let res = store
        .list_documents(
            &w,
            &ListDocumentsFilter {
                page: Some(1),
                page_size: Some(20),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(res.items.len(), 0);
    assert_eq!(res.total, 0);
    cases += 1;

    // Build a set S of docs in one wiki. Half tagged "grp-a", half "grp-b".
    let mut by_tag_a: HashSet<DocumentId> = HashSet::new();
    let mut all_ids: Vec<DocumentId> = Vec::new();
    let total_whole: u64 = 9;
    for i in 0..total_whole {
        let tag = if i % 2 == 0 { "grp-a" } else { "grp-b" };
        let d = create_doc(
            &store,
            &w,
            create_req(&format!("doc-{i}"), Some(vec![tag.into()])),
        )
        .await;
        all_ids.push(d.document_id.clone());
        if tag == "grp-a" {
            by_tag_a.insert(d.document_id.clone());
        }
        cases += 1;
    }

    let _ = rng.yes();

    // page_size ∈ {1, 2, mid(4), ≥ |S|(20), 100-boundary} → whole set.
    for ps in [1u64, 2, 4, 20, 100] {
        let (ids, c) =
            collect_pages(&store, &w, &ListDocumentsFilter::default(), ps, total_whole).await;
        cases += c;
        let collected: HashSet<&DocumentId> = ids.iter().collect();
        assert_eq!(
            collected.len(),
            total_whole as usize,
            "no omissions, no duplicates at ps={ps}"
        );
        assert!(
            all_ids.iter().all(|i| collected.contains(i)),
            "every generated id present at ps={ps}"
        );
        assert_eq!(
            ids.len() as u64,
            total_whole,
            "exactly |S| items across pages at ps={ps}"
        );
    }

    // Filtered set (tag=="grp-a"): generated set S is the filtered subset.
    let page_size_a: u64 = 2;
    let expected_a = by_tag_a.len() as u64;
    let filter_a = ListDocumentsFilter {
        tag: Some("grp-a".into()),
        ..Default::default()
    };
    let (ids_a, c) = collect_pages(&store, &w, &filter_a, page_size_a, expected_a).await;
    cases += c;
    let collected_a: HashSet<&DocumentId> = ids_a.iter().collect();
    assert_eq!(
        collected_a.len(),
        expected_a as usize,
        "filtered set complete + unique"
    );
    assert!(
        by_tag_a.iter().all(|i| collected_a.contains(i)),
        "filtered set matches generated subset"
    );

    // State-filtered set: all docs are Draft here.
    let filter_d = ListDocumentsFilter {
        state: Some(DocState::Draft),
        ..Default::default()
    };
    let (ids_d, c) = collect_pages(&store, &w, &filter_d, 3, total_whole).await;
    cases += c;
    assert_eq!(
        ids_d.len() as u64,
        total_whole,
        "state filter matches full set"
    );

    // Combined filters (state + tag), page_size 1.
    let filter_both = ListDocumentsFilter {
        state: Some(DocState::Draft),
        tag: Some("grp-a".into()),
        ..Default::default()
    };
    let (ids_b, c) = collect_pages(&store, &w, &filter_both, 1, expected_a).await;
    cases += c;
    assert_eq!(ids_b.len() as u64, expected_a);

    // page beyond the last page → empty items, correct total.
    let f_beyond = ListDocumentsFilter {
        page: Some(1000),
        page_size: Some(4),
        ..Default::default()
    };
    let res = store.list_documents(&w, &f_beyond).await.unwrap();
    assert!(res.items.is_empty(), "page beyond last must be empty");
    assert_eq!(
        res.total, total_whole,
        "total correct even past the last page"
    );
    assert_eq!(res.page, 1000);
    cases += 1;

    assert!(cases <= B_TP1, "B_TP1 budget exceeded: {cases}");
    println!("[P-TP-1] generated cases: {cases}");
}

// ---------------------------------------------------------------------------
// P-TP-2 (TP) — strat:metadata-preserve-update — unsupplied fields preserved.
// ---------------------------------------------------------------------------
// Graph-only updates preserve title/tags/author; title-only and tags-only update
// exactly their field; author preserved across *every* update.

#[tokio::test]
async fn p_tp_2_metadata_preserve_update() {
    let store: Arc<Store> = Arc::new(Store::new());
    let w = new_wiki(&store, "P-TP-2-wiki").await;
    let mut rng = Rng::seeded(row_seed(0x545032)); // "P-TP-2"
    let mut cases: u32 = 0;
    let author = "alice";
    let init_title = "initial";
    let d = create_doc(
        &store,
        &w,
        create_req_author(init_title, Some(vec!["t0".into(), "t1".into()]), author),
    )
    .await;
    assert_eq!(d.author.as_deref(), Some(author));
    assert_eq!(d.tags, vec!["t0".to_string(), "t1".to_string()]);
    assert_eq!(d.title, init_title);
    cases += 1;

    // Track last committed field values.
    let mut last_title = init_title.to_string();
    let mut last_tags: Vec<String> = vec!["t0".into(), "t1".into()];

    // Graph-only updates right after create: everything preserved.
    for k in 0..3u64 {
        let base = k;
        let upd = store
            .update_document(&d.document_id, graph_only_update(base))
            .await
            .unwrap();
        assert_eq!(upd.title, last_title, "graph-only keeps title");
        assert_eq!(upd.tags, last_tags, "graph-only keeps tags");
        assert_eq!(
            upd.author.as_deref(),
            Some(author),
            "graph-only keeps author"
        );
        let fetched = store.get_document(&d.document_id).await.unwrap();
        assert_eq!(fetched.title, last_title);
        assert_eq!(fetched.tags, last_tags);
        assert_eq!(fetched.author.as_deref(), Some(author));
        cases += 1;
    }

    // Alternating title-only / tags-only / graph-only over a ladder.
    for (rev, step) in (3u64..).zip(0..6u64) {
        let req = match step % 3 {
            0 => {
                // title-only: title replaced, tags+author preserved.
                last_title = format!("title-n{step}");
                title_update(&last_title, rev)
            }
            1 => {
                // tags-only: tags replaced, title+author preserved.
                last_tags = vec![format!("tg-{step}")];
                tags_update(&[&last_tags[0]], rev)
            }
            _ => {
                // graph-only once more after partial updates: everything stays.
                graph_only_update(rev)
            }
        };
        let upd = store.update_document(&d.document_id, req).await.unwrap();
        assert_eq!(
            upd.title, last_title,
            "title matches last committed value at step {step}"
        );
        assert_eq!(
            upd.tags, last_tags,
            "tags match last committed value at step {step}"
        );
        assert_eq!(
            upd.author.as_deref(),
            Some(author),
            "author preserved across every update"
        );
        let fetched = store.get_document(&d.document_id).await.unwrap();
        assert_eq!(fetched.title, last_title);
        assert_eq!(fetched.tags, last_tags);
        assert_eq!(fetched.author.as_deref(), Some(author));
        let _ = rng.yes();
        cases += 1;
    }

    assert!(cases <= B_TP2, "B_TP2 budget exceeded: {cases}");
    println!("[P-TP-2] generated cases: {cases}");

    // Suppress unused-var lint on rng (kept for determinism harness symmetry).
    let _ = rng.below(2);
}

// ---------------------------------------------------------------------------
// P-TP-3 (TP) — strat:delete-recreate — delete-then-get + brand-new re-creation.
// ---------------------------------------------------------------------------
// After delete: DocumentNotFound, absent from every page; re-create yields a
// distinct id, rev 0, DRAFT, listable. Delete of a never-created id has no
// side effect. Repeated create→delete→create keeps ids strictly distinct.

#[tokio::test]
async fn p_tp_3_delete_recreate() {
    let store: Arc<Store> = Arc::new(Store::new());
    let w = new_wiki(&store, "P-TP-3-wiki").await;
    let mut rng = Rng::seeded(row_seed(0x545033)); // "P-TP-3"
    let mut cases: u32 = 0;

    // Delete the only doc in the wiki, then re-create (fresh distinct doc).
    let d = create_doc(&store, &w, create_req("only", None)).await;
    let deleted_id = d.document_id.clone();
    store.delete_document(&deleted_id).await.unwrap();
    cases += 1;
    assert_eq!(
        store.get_document(&deleted_id).await.unwrap_err(),
        StoreError::DocumentNotFound,
        "delete-then-get → DocumentNotFound"
    );

    let re = create_doc(&store, &w, create_req("re-created", None)).await;
    cases += 1;
    assert_ne!(
        re.document_id, deleted_id,
        "re-created id must differ from the deleted id"
    );
    assert_eq!(re.revision, 0, "re-created document starts at revision 0");
    assert_eq!(
        re.state,
        DocState::Draft,
        "re-created document starts DRAFT"
    );
    let fetched = store.get_document(&re.document_id).await.unwrap();
    assert_ne!(
        fetched.document_id, deleted_id,
        "re-created doc is independent"
    );
    assert_eq!(fetched.document_id, re.document_id);
    assert_eq!(fetched.revision, 0);

    // delete-then-get on a never-created id → DocumentNotFound, no side effect.
    let ghost = doc_id("never-created");
    assert_eq!(
        store.delete_document(&ghost).await.unwrap_err(),
        StoreError::DocumentNotFound
    );
    assert_eq!(
        store.get_document(&ghost).await.unwrap_err(),
        StoreError::DocumentNotFound
    );
    cases += 1;

    // Delete one of several, then list: deleted absent, others present.
    let mut keep_ids: Vec<DocumentId> = Vec::new();
    for i in 0..5u64 {
        let doc = create_doc(&store, &w, create_req(&format!("multi-{i}"), None)).await;
        keep_ids.push(doc.document_id.clone());
        cases += 1;
    }
    let doomed = keep_ids.remove(2);
    store.delete_document(&doomed).await.unwrap();
    cases += 1;
    let seen_by_tag: HashMap<String, u64> = HashMap::new();
    let _ = seen_by_tag;
    let mut seen_pages: HashSet<DocumentId> = HashSet::new();
    let list = store
        .list_documents(
            &w,
            &ListDocumentsFilter {
                page: Some(1),
                page_size: Some(100),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    for s in &list.items {
        assert_ne!(s.document_id, doomed, "deleted doc absent from list");
        seen_pages.insert(s.document_id.clone());
    }
    assert!(
        keep_ids.iter().all(|i| seen_pages.contains(i)),
        "survivors still listable"
    );
    assert!(!seen_pages.contains(&doomed));

    // Repeated create→delete→create cycles: ids strictly distinct across the run.
    let mut all_ids: HashSet<DocumentId> = HashSet::new();
    let mut prev_deleted: Vec<DocumentId> = Vec::new();
    let cycles: u32 = 6;
    for i in 0..cycles {
        let doc = create_doc(&store, &w, create_req(&format!("cycle-{i}"), None)).await;
        assert!(
            all_ids.insert(doc.document_id.clone()),
            "cycle id must be globally distinct"
        );
        assert!(
            prev_deleted.iter().all(|p| p != &doc.document_id),
            "re-created id != every prior deleted id"
        );
        // Optionally delete in some cycles.
        if rng.yes() {
            let id = doc.document_id.clone();
            store.delete_document(&id).await.unwrap();
            prev_deleted.push(id);
            cases += 1;
        }
        cases += 1;
    }

    assert!(cases <= B_TP3, "B_TP3 budget exceeded: {cases}");
    println!("[P-TP-3] generated cases: {cases}");
}

// ---------------------------------------------------------------------------
// §4.1 READ-ONLY PBT AUDIT — deterministic robustness probes (non-PRN)
// ---------------------------------------------------------------------------
// The following probe blocks were requested verbatim by the read-only audit of
// the §4.1 register. They are **deterministic explicit assertion blocks**, not
// PRNG-driven generated cases: they consume no PRNG draws and are therefore
// **outside** the ≤400 generated-case budget (mirroring the facts layer's
// boundary blocks). Each probe PASSES against the current GREEN implementation;
// where one documents a documented exception to a register row rather than a
// failure, it pins the ACTUAL behavior with an explicit comment.

/// Shared barrier race that drives a two-way (or n-way) same-base race where
/// each contender carries its OWN graph body (not just a different title), so
/// the graph-payload race probe can observe *which* graph persisted.
#[allow(clippy::type_complexity)]
async fn run_graph_race(
    store: &Arc<Store>,
    id: &DocumentId,
    base: u64,
    contenders: Vec<(String, Graph)>,
) -> (u32, u32, u32, Vec<Result<gnosis::Document, StoreError>>) {
    let n = contenders.len();
    let barrier = Arc::new(Barrier::new(n));
    let mut handles = Vec::new();
    for (title, graph) in contenders {
        let store = store.clone();
        let barrier = barrier.clone();
        let id = id.clone();
        handles.push(tokio::spawn(async move {
            barrier.wait().await; // genuinely contend
            store
                .update_document(
                    &id,
                    UpdateDocumentRequest {
                        base_revision: base,
                        graph,
                        title: Some(title),
                        tags: None,
                    },
                )
                .await
        }));
    }
    let mut results = Vec::new();
    let (mut ok, mut conflict, mut other) = (0u32, 0u32, 0u32);
    for h in handles {
        let r = h.await.expect("no task panic");
        match &r {
            Ok(_) => ok += 1,
            Err(StoreError::ConflictError) => conflict += 1,
            Err(_) => other += 1,
        }
        results.push(r);
    }
    (ok, conflict, other, results)
}

/// **Audit probe #1 (P-SM-2 two-sided + graph-payload race).** A same-base race
/// where the two contenders carry DIFFERENT `Graph` bodies (distinct node
/// values, built with `valid_graph_for_value` — not a title-only difference).
/// Asserts BOTH sides of the winner/loser condition explicitly *and* that the
/// winner's graph nodes persisted while the loser's graph did not leak.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn audit_probe_psm2_race_two_sided_graph_payload() {
    let store: Arc<Store> = Arc::new(Store::new());
    let w = new_wiki(&store, "probe-sm2-wiki").await;
    let d = create_doc(&store, &w, create_req("race-base", None)).await;
    let id = d.document_id.clone();
    let base = d.revision;

    // Both contenders target the SAME real document id; their graph bodies are
    // distinguishable by node value (graph-payload race, §generator P-SM-2:
    // "contenders with different payloads — the winner's payload must be the
    // one that persists").
    let contender_a = ("t-a".to_string(), valid_graph_for_value(&id, "graph-alpha"));
    let contender_b = ("t-b".to_string(), valid_graph_for_value(&id, "graph-beta"));
    let (ok, conflict, other, results) =
        run_graph_race(&store, &id, base, vec![contender_a, contender_b]).await;

    assert_eq!(ok, 1, "exactly one winner");
    assert_eq!(conflict, 1, "exactly one ConflictError");
    assert_eq!(other, 0, "no other error / panic");

    let winner = results
        .iter()
        .find_map(|r| r.as_ref().ok())
        .expect("a winner exists");
    // The loser committed nothing (ConflictError) so it has no resolved Document;
    // its *requested* title is the other contender's title, which must not persist.
    let (loser_title, winner_val, loser_val) = if winner.title == "t-a" {
        ("t-b", "graph-alpha", "graph-beta")
    } else {
        ("t-a", "graph-beta", "graph-alpha")
    };
    let final_doc = store.get_document(&id).await.unwrap();

    // Two-sided winner/loser, EXPLICIT (not inferred from the revision bump).
    assert_eq!(
        final_doc.title, winner.title,
        "winner's title must be the one that persists"
    );
    assert_ne!(
        final_doc.title, loser_title,
        "loser's title must NOT persist"
    );

    // Graph-payload race: the winner's graph nodes persisted; the loser's did not.
    assert!(
        final_doc
            .graph
            .nodes
            .iter()
            .any(|n| n.value.as_deref() == Some(winner_val)),
        "winner's graph nodes must persist in the committed document"
    );
    assert!(
        !final_doc
            .graph
            .nodes
            .iter()
            .any(|n| n.value.as_deref() == Some(loser_val)),
        "loser's graph must NOT leak into the committed document"
    );
    assert_eq!(
        final_doc.revision,
        base + 1,
        "stored revision advances by exactly one"
    );
}

/// **Audit probe #2 (P-SM-1 annotation-reconcile boundary — DOCUMENTED, not a
/// failure).**
///
/// Part A: the `set_reference_state` reconcile path. A `set_reference_state`
/// bump carries no structural/content change (§4.2.8.5), so when an `update`
/// carries a base stale ONLY because of such annotation-only revisions the
/// store RECONCILES rather than returning `ConflictError`, and the returned
/// revision reflects the reconcile jump (`current+1`, i.e. > `base+1`). This is
/// the **documented exception** to P-SM-1's "steps by exactly one" (see the
/// upcoming register re-scope of P-SM-1).
///
/// Part B: a genuinely **future** base (`base = current+1`) on a fresh document
/// with no annotation history is an unreconcilable stale base → `ConflictError`.
/// (A future base on an annotation-touched doc would be swept into the
/// empty-range reconcile — verified against the current impl — so Part B is run
/// on a separate annotation-free doc to pin the genuine `ConflictError`.)
#[tokio::test]
async fn audit_probe_psm1_reference_state_reconcile_boundary() {
    let store: Arc<Store> = Arc::new(Store::new());
    let w = new_wiki(&store, "probe-sm1-wiki").await;

    // ---- Part A: documented reconcile jump.
    let target = create_doc(&store, &w, create_req("reconcile-target", None)).await;
    let d = create_doc(&store, &w, create_req("reconcile-doc", None)).await; // rev 0
                                                                             // First commit a graph carrying a `link` reference edge so
                                                                             // `set_reference_state` has an edge to annotate. base 0 → rev 1.
    store
        .update_document(
            &d.document_id,
            UpdateDocumentRequest {
                base_revision: 0,
                graph: graph_with_link_to(&d.document_id, &target.document_id),
                title: None,
                tags: None,
            },
        )
        .await
        .unwrap();
    // `set_reference_state` (annotation-only) → rev 2. The link edge's
    // source node id is `ref1`, target node id is `n1`.
    store
        .set_reference_state(
            &d.document_id,
            &NodeId("ref1".into()),
            &NodeId("n1".into()),
            "FRESH",
        )
        .await
        .unwrap();
    // The caller holds base=1 (the last NON-annotation revision it saw); the
    // current revision is 2. The stale base is caused solely by the annotation
    // bump, so update RECONCILES instead of `ConflictError`.
    let updated = store
        .update_document(
            &d.document_id,
            UpdateDocumentRequest {
                base_revision: 1,
                graph: valid_graph_for(&d.document_id),
                title: Some("reconciled".into()),
                tags: None,
            },
        )
        .await
        .unwrap_or_else(|e| {
            panic!("a base stale solely due to set_reference_state must reconcile, got {e:?}")
        });
    // The reconcile jump: returned revision = current+1 = 3, which is > base+1 = 2
    // (the annotation-only rev 2 is skipped in the "step by exactly one" sense).
    assert_eq!(updated.revision, 3, "reconcile returns current+1");
    assert!(
        updated.revision > 1 + 1,
        "reconcile jump exceeds base+1 (documented exception to steps-by-exactly-one)"
    );
    let fetched = store.get_document(&d.document_id).await.unwrap();
    assert_eq!(fetched.revision, 3);
    assert_eq!(
        fetched.title, "reconciled",
        "the caller's structural edit landed"
    );

    // ---- Part B: future base → ConflictError (§4.1.4), annotation-free doc.
    let d2 = create_doc(&store, &w, create_req("future-base", None)).await; // rev 0, no annotations
    let err = store
        .update_document(
            &d2.document_id,
            UpdateDocumentRequest {
                base_revision: d2.revision + 1, // base = current+1 (future base)
                graph: valid_graph_for(&d2.document_id),
                title: Some("nope".into()),
                tags: None,
            },
        )
        .await
        .unwrap_err();
    assert_eq!(
        err,
        StoreError::ConflictError,
        "a future base (base = current+1) is ConflictError (§4.1.4)"
    );
}

/// **Audit probe #3 (P-TP-3 delete-DocumentInUse + multi-page).** Deleting a doc
/// that is the `link` **target** of another doc → `DocumentInUse`, and (the
/// non-success delete branch) `get_document` still resolves the survivor. After
/// clearing the referrer, the delete succeeds; then delete-then-recreate and
/// walk ALL pages at `page_size ∈ {1, 2, 100}`, asserting the deleted id is
/// absent from every page.
#[tokio::test]
async fn audit_probe_ptp3_delete_in_use_and_multi_page_walk() {
    let store: Arc<Store> = Arc::new(Store::new());
    let w = new_wiki(&store, "probe-tp3-wiki").await;

    let tgt = create_doc(&store, &w, create_req("target", None)).await;
    let tgt_id = tgt.document_id;
    let refr = create_doc(&store, &w, create_req("referrer", None)).await;
    // `referrer` holds a `link` edge whose target is in `target` → §4.4.5 gate.
    store
        .update_document(
            &refr.document_id,
            UpdateDocumentRequest {
                base_revision: 0,
                graph: graph_with_link_to(&refr.document_id, &tgt_id),
                title: None,
                tags: None,
            },
        )
        .await
        .unwrap();
    let err = store.delete_document(&tgt_id).await.unwrap_err();
    assert_eq!(
        err,
        StoreError::DocumentInUse,
        "deleting a link target is DocumentInUse (§4.4.5)"
    );
    // Non-success delete branch: a rejected delete leaves the doc resolvable.
    assert!(
        store.get_document(&tgt_id).await.is_ok(),
        "a rejected delete must leave the document resolvable"
    );

    // Clear the referrer's link (rev 1 → rev 2), making the target deletable.
    store
        .update_document(
            &refr.document_id,
            UpdateDocumentRequest {
                base_revision: 1,
                graph: valid_graph_for(&refr.document_id),
                title: None,
                tags: None,
            },
        )
        .await
        .unwrap();
    store.delete_document(&tgt_id).await.unwrap();
    assert_eq!(
        store.get_document(&tgt_id).await.unwrap_err(),
        StoreError::DocumentNotFound
    );

    // delete-then-recreate: a brand-new doc is created after the delete.
    let recreated = create_doc(&store, &w, create_req("recreated", None)).await;

    // Walk ALL pages at every page_size and assert the deleted id never appears.
    for ps in [1u64, 2, 100] {
        for page in 1..=1000u64 {
            let res = store
                .list_documents(
                    &w,
                    &ListDocumentsFilter {
                        page: Some(page),
                        page_size: Some(ps),
                        ..Default::default()
                    },
                )
                .await
                .unwrap();
            if res.items.is_empty() {
                break; // past the last page → all pages walked
            }
            assert!(
                res.items.iter().all(|s| s.document_id != tgt_id),
                "deleted id must be absent from every page at page_size={ps}"
            );
        }
    }

    // The re-created doc is independently listable.
    let present = store
        .list_documents(
            &w,
            &ListDocumentsFilter {
                page: Some(1),
                page_size: Some(100),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert!(
        present
            .items
            .iter()
            .any(|s| s.document_id == recreated.document_id),
        "the re-created document must be independently listable"
    );
}

/// **Audit probe #4 (P-TP-2 title-201-on-update + tags-clear).**
///
/// (a) `update_document(title = Some(201-char))` — PIN the current behavior: the
/// store does NOT re-validate title length on the update path (validation is
/// §4.1.3 create-only; `update_document` applies `request.title` verbatim via
/// `unwrap_or_else(|| current.title.clone())`). So a 201-char title through
/// `update_document` is ACCEPTED and persisted byte-for-byte. The register
/// re-scope may later move title validation to the update path; this block pins
/// today's actual behavior deterministically so a naive "validate everywhere"
/// change cannot silently regress the round-trip.
///
/// (b) `update_document(tags = Some(vec![]))` — the explicit-empty `unwrap_or_else`
/// branch clears the tags (an explicitly-supplied empty vec REPLACES the prior
/// set, unlike `None` which preserves it).
#[tokio::test]
async fn audit_probe_ptp2_title_201_on_update_and_tags_clear() {
    let store: Arc<Store> = Arc::new(Store::new());
    let w = new_wiki(&store, "probe-tp2-wiki").await;
    let d = create_doc(
        &store,
        &w,
        create_req_author("init", Some(vec!["t0".into(), "t1".into()]), "alice"),
    )
    .await;

    // (a) title = Some(201-char) on update — pinned current behavior.
    let long201 = "T".repeat(201);
    assert_eq!(long201.chars().count(), 201, "premise");
    let upd = store
        .update_document(
            &d.document_id,
            UpdateDocumentRequest {
                base_revision: 0,
                graph: valid_graph_for(&d.document_id),
                title: Some(long201.clone()),
                tags: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(upd.revision, 1);
    assert_eq!(
        upd.title, long201,
        "a 201-char title is accepted on update (store does not re-validate; pinned)"
    );
    let fetched = store.get_document(&d.document_id).await.unwrap();
    assert_eq!(
        fetched.title, long201,
        "the stored title is the 201-char string, round-tripped byte-for-byte"
    );

    // (b) tags = Some(vec![]) on update clears the tags (explicit-empty branch).
    let upd2 = store
        .update_document(
            &d.document_id,
            UpdateDocumentRequest {
                base_revision: 1,
                graph: valid_graph_for(&d.document_id),
                title: None,
                tags: Some(vec![]),
            },
        )
        .await
        .unwrap();
    assert_eq!(upd2.revision, 2);
    assert!(
        upd2.tags.is_empty(),
        "explicit empty tags clear the tag set"
    );
    let fetched2 = store.get_document(&d.document_id).await.unwrap();
    assert!(
        fetched2.tags.is_empty(),
        "stored tags are empty after the clear"
    );
    assert_eq!(
        fetched2.title, long201,
        "title preserved while tags cleared"
    );
    assert_eq!(
        fetched2.author.as_deref(),
        Some("alice"),
        "author preserved across every update"
    );
}
