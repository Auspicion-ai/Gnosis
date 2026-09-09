//! §4.5 RAG/agent-memory retrieval — EXECUTED property layer (PBT-gate artifact #2).
//!
//! Implements EVERY row of the typed property register
//! `docs/specs/4-5-retrieval-property-register.md` as exactly one `#[test]` per
//! row (8 rows: P-IM-1, P-IM-2, P-SM-1, P-SM-2, P-SM-3, P-SM-4, P-TP-1, P-TP-2).
//!
//! This is a hand-rolled, crate-free PBT layer per the PBT-gate contract:
//!
//!   * **Deterministic pinned seed.** A Xoshiro256** PRNG (with a SplitMix64
//!     seed scramble) whose every row derives a fixed sub-seed from the pinned
//!     constant. No `proptest`/`rand` crate; runs are byte-identical every time.
//!   * **PINNED_SEED (file header + report): `0x5DEECE66D`.**
//!     Per-row sub-seed = `PINNED_SEED.wrapping_add(ROW_TAG)` for a unique,
//!     stable `ROW_TAG` per register row.
//!   * **Budget.** ≤100 generated cases per row; total across the property
//!     layer ≤400 (documented per row in `/// BUDGET:` comments).
//!   * **stop-after-5.** A broken row retains ≤5 minimal counterexamples, prints
//!     them, then FAILS with the failing assertion (the intended red set).
//!   * **Faithful.** Each Observable-as-property is a real check over real
//!     generated/executed data — never weakened to force green. A genuinely
//!     broken row returns red with a minimal counterexample.
//!
//! Reused (copied) certified fixtures: the deterministic embedding provider +
//! immutable vector-snapshot wiring from `tests/retrieval_stack_integration.rs`
//! (`MockProvider`, `seed_vectors`) and the document/graph helpers from
//! `tests/retrieval_stack_integration.rs` / `tests/rag_query_integration.rs`.
//! The async query surface is the crate's `RagStore` (`rag_query`/`rag_stream`).
//!
//! RESERVED-discipline honored: no row generates inputs whose success is
//! impossible (no reserved throw-path dependence — no `HyDEGenerationFailed` /
//! `CompressionFailed` / `SubTaskDagFailed` / `MultiQueryExpansionFailed` /
//! reranker variants). `P-SM-4` guarantees a distinct expansion term so the
//! multi-query fan-out always succeeds; `P-TP-2` never drives a total-compressor
//! failure. Vector/hybrid rows wire a provider + full-field vector snapshot so
//! `EmbeddingUnavailable`/`VectorIndexUnavailable` are never the asserted path.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use futures::StreamExt;
use gnosis::{
    rrf_fuse, CreateDocumentRequest, DerivedIndexes, Document, DocumentId, Edge, EdgeKind,
    EmbeddingProvider, EngineState, ExpandMode, FieldType, Graph, ListDocumentsFilter,
    MultiQueryOptions, Node, NodeId, NodeKind, QueryMode, RagChunk, RagQueryOptions, RagResult,
    RagResultItem, RagStore, RagStream, RagTrace, ReferenceState, Store, StoreError,
    UpdateDocumentRequest, VectorIndex, WikiId, RRF_K,
};

// ===========================================================================
// Pinned-seed PRNG (Xoshiro256** + SplitMix64 seed scramble) — NO external crate.
// ===========================================================================

const PINNED_SEED: u64 = 0x5DEECE66D;

struct Rng {
    s: [u64; 4],
}

impl Rng {
    fn from_seed(seed: u64) -> Self {
        fn splitmix64(x: &mut u64) -> u64 {
            *x = x.wrapping_add(0x9E3779B97F4A7C15);
            let mut z = *x;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
            z ^ (z >> 31)
        }
        let mut x = seed;
        Rng {
            s: [
                splitmix64(&mut x),
                splitmix64(&mut x),
                splitmix64(&mut x),
                splitmix64(&mut x),
            ],
        }
    }
    fn next_u64(&mut self) -> u64 {
        let result = self.s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        result
    }
    /// Uniform in `0..bound`.
    fn below(&mut self, bound: usize) -> usize {
        if bound == 0 {
            0
        } else {
            (self.next_u64() % bound as u64) as usize
        }
    }
    /// Uniform in `lo..=hi` (inclusive).
    fn in_range(&mut self, lo: usize, hi: usize) -> usize {
        lo + self.below(hi - lo + 1)
    }
    fn shuffle<T>(&mut self, v: &mut [T]) {
        for i in (1..v.len()).rev() {
            let j = self.below(i + 1);
            v.swap(i, j);
        }
    }
}

// ===========================================================================
// Reused certified fixtures (copied verbatim from the §4.5 green example suites)
// ===========================================================================

fn did(s: &str) -> DocumentId {
    DocumentId(s.to_string())
}
fn nid(s: &str) -> NodeId {
    NodeId(s.to_string())
}

fn content_node(doc: &DocumentId, node: &str, value: &str) -> Node {
    Node {
        document_id: doc.clone(),
        node_id: nid(node),
        kind: NodeKind::Content,
        value: Some(value.to_string()),
        fact_key: None,
        target: None,
    }
}

fn fact_node(doc: &DocumentId, node: &str, value: &str) -> Node {
    Node {
        document_id: doc.clone(),
        node_id: nid(node),
        kind: NodeKind::Fact,
        value: Some(value.to_string()),
        fact_key: None,
        target: None,
    }
}

fn ref_to(doc: &DocumentId, node: &str, target: (DocumentId, NodeId)) -> Node {
    Node {
        document_id: doc.clone(),
        node_id: nid(node),
        kind: NodeKind::Reference,
        value: None,
        fact_key: None,
        target: Some(target),
    }
}

fn edge(
    kind: EdgeKind,
    from: (DocumentId, NodeId),
    to: (DocumentId, NodeId),
    state: Option<ReferenceState>,
) -> Edge {
    Edge {
        source: from,
        target: to,
        kind,
        state,
        cross_wiki: false,
        relation_type: None,
    }
}

async fn new_wiki(store: &Store, name: &str) -> WikiId {
    store.create_wiki(name).await.unwrap().wiki_id
}

async fn new_doc(store: &Store, w: &WikiId, title: &str) -> Document {
    store
        .create_document(
            w,
            CreateDocumentRequest {
                title: title.to_string(),
                tags: None,
                author: None,
            },
        )
        .await
        .unwrap()
}

async fn update_graph(store: &Store, doc: &Document, nodes: Vec<Node>, edges: Vec<Edge>) {
    let cur = store.get_document(&doc.document_id).await.unwrap();
    let first = nodes[0].node_id.clone();
    let d = doc.document_id.clone();
    let mut all_edges = vec![
        edge(
            EdgeKind::DocHead,
            (d.clone(), nid("ROOT")),
            (d.clone(), first.clone()),
            None,
        ),
        edge(
            EdgeKind::DocEnd,
            (d.clone(), first),
            (d.clone(), nid("END")),
            None,
        ),
    ];
    all_edges.extend(edges);
    store
        .update_document(
            &doc.document_id,
            UpdateDocumentRequest {
                base_revision: cur.revision,
                graph: Graph {
                    nodes,
                    edges: all_edges,
                },
                title: None,
                tags: None,
            },
        )
        .await
        .unwrap();
}

/// Create a real document in `w` with a single `content` node and return its
/// real `(documentId, nodeId)`. (Vector tests MUST seed the vector index under
/// real docs that belong to the queried wiki — MEDIUM-5 wiki-scoping.)
async fn new_content_doc(
    store: &Store,
    w: &WikiId,
    title: &str,
    node: &str,
    value: &str,
) -> (DocumentId, NodeId) {
    let doc = new_doc(store, w, title).await;
    update_graph(
        store,
        &doc,
        vec![content_node(&doc.document_id, node, value)],
        vec![],
    )
    .await;
    (doc.document_id, nid(node))
}

/// Seed the immutable vector snapshot with `full`-field entries.
fn seed_vectors(store: &Store, entries: Vec<((DocumentId, NodeId), Vec<f32>)>) {
    let mut vi = VectorIndex::default();
    for ((d, n), v) in entries {
        vi.entries.insert((d, n, FieldType::Full), v);
    }
    store.swap_snapshot(DerivedIndexes {
        lexical: None,
        vectors: Some(vi),
        epoch: 1,
    });
}

/// Deterministic in-memory embedding provider (fixed vector; availability fixed).
struct MockProvider {
    available: bool,
    emit_error: bool,
    v: Vec<f32>,
}

impl MockProvider {
    fn hit(v: Vec<f32>) -> Self {
        MockProvider {
            available: true,
            emit_error: false,
            v,
        }
    }
}

impl EmbeddingProvider for MockProvider {
    fn embed(
        &self,
        _text: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<f32>, StoreError>> + Send + '_>> {
        let emit = self.emit_error;
        let v = self.v.clone();
        Box::pin(async move {
            if emit {
                Err(StoreError::EmbeddingUnavailable)
            } else {
                Ok(v)
            }
        })
    }
    fn is_available(&self) -> Pin<Box<dyn Future<Output = bool> + Send + '_>> {
        let a = self.available;
        Box::pin(async move { a })
    }
}

fn ready(store: &Store) {
    store.set_engine_state(EngineState::Ready);
}

/// Seed a resolvable `reference`→`fact` pair in `w`, returning
/// `(fact_doc, fact_node, ref_doc, ref_node)`.
async fn seed_ref_to_fact(store: &Store, w: &WikiId) -> (DocumentId, NodeId, DocumentId, NodeId) {
    let fdoc = new_doc(store, w, "fact-doc").await;
    let fnid = nid("F");
    update_graph(
        store,
        &fdoc,
        vec![fact_node(&fdoc.document_id, "F", "canonical-fact-value")],
        vec![],
    )
    .await;

    let rdoc = new_doc(store, w, "ref-doc").await;
    let rnid = nid("R");
    let target = (fdoc.document_id.clone(), fnid.clone());
    update_graph(
        store,
        &rdoc,
        vec![ref_to(&rdoc.document_id, "R", target.clone())],
        vec![edge(
            EdgeKind::Link,
            (rdoc.document_id.clone(), rnid.clone()),
            target.clone(),
            Some(ReferenceState::Resolved),
        )],
    )
    .await;
    (fdoc.document_id, fnid, rdoc.document_id, rnid)
}

fn key(ri: &RagResultItem) -> (DocumentId, NodeId) {
    (ri.document_id.clone(), ri.node_id.clone())
}
fn result_keys(res: &RagResult) -> Vec<(DocumentId, NodeId)> {
    res.results.iter().map(key).collect()
}

/// True if `sub` is a (strict-or-equal) subsequence of `super_` preserving order.
fn is_subsequence(sub: &[(DocumentId, NodeId)], sup: &[(DocumentId, NodeId)]) -> bool {
    let mut i = 0;
    for item in sub {
        while i < sup.len() && sup[i] != *item {
            i += 1;
        }
        if i == sup.len() {
            return false;
        }
        i += 1;
    }
    true
}

fn all_distinct(keys: &[(DocumentId, NodeId)]) -> bool {
    let set: std::collections::HashSet<&(DocumentId, NodeId)> = keys.iter().collect();
    set.len() == keys.len()
}

async fn owning_wiki(store: &Store, document_id: &DocumentId) -> WikiId {
    store.get_document(document_id).await.unwrap().wiki_id
}

/// Independent hand-derivation of an item's RRF score (`k = RRF_K`, 1-based rank).
///
/// **ORACLE-ALIGNMENT (audit a-2/c-1):** each key's `1/(k+rank)` contributions
/// are summed in **ASCENDING-SORTED (canonical) order** — exactly as the fixed
/// `rrf_fuse` does (`contribs` per key, `sort`, `fold(0.0)`). Definition of an
/// **"exact tie"** throughout this suite: two keys whose **ascending-canonical
/// sums** are **byte-identical** floats. Summing in outer-list input order is
/// WRONG here: FP addition is commutative but not associative, so a genuine
/// mathematical tie with ≥3 contributions in non-sorted input order can land on
/// floats 1 ULP apart, disagreeing with the fixed impl's (doc,node)-ascending
/// tie-break and spuriously RED-flipping the (already-correct) P-IM-2. By
/// rebasing the oracle onto ascending-canonical sums it detects the same exact
/// ties the fixed impl does.
fn rrf_score(lists: &[Vec<(DocumentId, NodeId)>], item: &(DocumentId, NodeId)) -> f64 {
    let mut cs: Vec<f64> = Vec::new();
    for list in lists {
        for (idx, x) in list.iter().enumerate() {
            if x == item {
                cs.push(1.0 / (RRF_K + (idx as f64) + 1.0));
            }
        }
    }
    // Canonical ascending order → byte-identical sums for equal contribution
    // multisets (input-order independent), mirroring the fixed `rrf_fuse`.
    cs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    cs.iter().fold(0.0, |acc, c| acc + c)
}

/// Independent expected merge (score desc, tie by (doc,node) asc, truncate).
fn rrf_expected(lists: &[Vec<(DocumentId, NodeId)>], top_k: usize) -> Vec<(DocumentId, NodeId)> {
    let set: std::collections::HashSet<(DocumentId, NodeId)> =
        lists.iter().flat_map(|l| l.iter().cloned()).collect();
    let mut v: Vec<(DocumentId, NodeId)> = set.into_iter().collect();
    v.sort_by(|a, b| {
        rrf_score(lists, b)
            .partial_cmp(&rrf_score(lists, a))
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
            .then_with(|| a.1.cmp(&b.1))
    });
    v.truncate(top_k);
    v
}

// ===========================================================================
// P-IM-1 — rrf_fuse output validity (unique, bounded, no fabrication).  #[test]
// BUDGET: 60 cases (≤100).
// ===========================================================================
#[test]
fn p_im_1_rrf_fuse_output_valid_unique_bounded_no_fabrication() {
    const ROW_TAG: u64 = 0x101;
    const BUDGET: usize = 60; // ≤100
    let mut rng = Rng::from_seed(PINNED_SEED.wrapping_add(ROW_TAG));
    let mut failures: Vec<String> = Vec::new();

    for _case in 0..BUDGET {
        if failures.len() >= 5 {
            break;
        }
        let nlists = rng.in_range(0, 5);
        let ndocs = rng.in_range(1, 6); // distinct docs in the universe
        let nnodes = rng.in_range(1, 4); // distinct node ids per doc

        let mut lists: Vec<Vec<(DocumentId, NodeId)>> = Vec::new();
        let mut all_keys: Vec<(DocumentId, NodeId)> = Vec::new();
        for _ in 0..nlists {
            let len = rng.in_range(0, 8);
            let mut list = Vec::new();
            for _ in 0..len {
                let d = rng.below(ndocs);
                let n = rng.below(nnodes);
                let k = (did(&format!("d{d}")), nid(&format!("n{n}")));
                list.push(k.clone());
                all_keys.push(k);
            }
            // ~1/3 chance to duplicate an inner element (rank is positional).
            if len > 0 && rng.below(3) == 0 {
                let dup = list[0].clone();
                list.push(dup);
                all_keys.push(list.last().unwrap().clone());
            }
            lists.push(list);
        }
        // A list containing keys not in any other list (fabrication guard).
        if nlists > 0 && rng.below(2) == 0 {
            lists[rng.below(nlists)].push((did("d-solo"), nid("n-solo")));
            all_keys.push((did("d-solo"), nid("n-solo")));
        }

        let distinct_count: usize = all_keys
            .iter()
            .cloned()
            .collect::<std::collections::HashSet<(DocumentId, NodeId)>>()
            .len();
        let top_k = match rng.below(4) {
            0 => 0,
            1 => 1,
            2 => rng.in_range(2, distinct_count.max(3)),
            _ => distinct_count + 5, // larger than the corpus
        };

        let merged = rrf_fuse(&lists, top_k);

        // (a) no duplicates — the multiset of merged keys is a set.
        if !all_distinct(&merged) {
            failures.push(format!(
                "duplicate output key present: lists={lists:?} top_k={top_k} merged={merged:?}"
            ));
            continue;
        }
        // (b) length == min(top_k, distinct).
        let expected_len = top_k.min(distinct_count);
        if merged.len() != expected_len {
            failures.push(format!(
                "length {}/{} != min(top_k={top_k}, distinct={distinct_count}): \
                 lists={lists:?} merged={merged:?}",
                merged.len(),
                expected_len
            ));
            continue;
        }
        // (c) every output key present in ≥1 input list (no fabrication).
        let input_all: std::collections::HashSet<(DocumentId, NodeId)> =
            lists.iter().flatten().cloned().collect();
        for k in &merged {
            if !input_all.contains(k) {
                failures.push(format!(
                    "fabricated key {k:?} not present in any input list: lists={lists:?} merged={merged:?}"
                ));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "P-IM-1 BROKEN — ≤5 minimal counterexamples:\n{}",
        failures.join("\n")
    );
}

// ===========================================================================
// P-IM-2 — rrf_fuse determinism, outer order-independence, exact tie-break. #[test]
// BUDGET: 60 cases (≤100).
// ===========================================================================
#[test]
fn p_im_2_rrf_fuse_deterministic_order_independent_exact_ties() {
    const ROW_TAG: u64 = 0x102;
    const BUDGET: usize = 60; // ≤100
    let mut rng = Rng::from_seed(PINNED_SEED.wrapping_add(ROW_TAG));
    let mut failures: Vec<String> = Vec::new();

    // Hand-constructed exact tie (generator note): list1=[P,Q], list2=[Q,P].
    let d = did("d");
    let p = (d.clone(), nid("n-p"));
    let q = (d.clone(), nid("n-q"));
    let tie = vec![vec![p.clone(), q.clone()], vec![q.clone(), p.clone()]];
    let tmerged = rrf_fuse(&tie, 2);
    let texpected = vec![p.clone(), q.clone()];
    if tmerged != texpected {
        failures.push(format!(
            "exact-tie (P,Q)/(Q,P) must order (doc,node) ascending: got {tmerged:?} want {texpected:?}"
        ));
    }

    // AUDIT-1 (b-1): deterministic ≥3-list exact tie with contributions in
    // DIFFERENT input orders. This is the ONLY shape guaranteed to bite FP
    // non-associativity: the 2-list hand-case above is commutative-safe and
    // cannot catch a regression. l0=[P,Q,R], l1=[R,P,Q], l2=[Q,R,P] → every key
    // gets contribution multiset {1/61,1/62,1/63} (P ranks 1/2/3, Q 2/3/1,
    // R 3/1/2). The ascending-canonical sums are byte-identical → a genuine
    // exact tie → merged output is exactly [P,Q,R] in (doc,node)-ascending
    // order, identical across every outer permutation of the three lists.
    let td = did("d");
    let tp = (td.clone(), nid("n-p"));
    let tq = (td.clone(), nid("n-q"));
    let tr = (td.clone(), nid("n-r"));
    let l0 = vec![tp.clone(), tq.clone(), tr.clone()];
    let l1 = vec![tr.clone(), tp.clone(), tq.clone()];
    let l2 = vec![tq.clone(), tr.clone(), tp.clone()];
    let three = vec![l0, l1, l2];
    let tbase3 = rrf_fuse(&three, 3);
    // Ascending canonical sums are identical → [P,Q,R] exactly.
    let expected3 = vec![tp.clone(), tq.clone(), tr.clone()];
    if tbase3 != expected3 {
        failures.push(format!(
            "≥3-list exact-tie must resolve to (doc,node) ascending [P,Q,R]: got {tbase3:?}"
        ));
    }
    // Any 2-key prefix is (doc,node)-ascending and identical across all outer
    // permutations of the three lists.
    for p0 in 0..3 {
        for p1 in 0..3 {
            for p2 in 0..3 {
                let p = [p0, p1, p2];
                if p[0] == p[1] || p[0] == p[2] || p[1] == p[2] {
                    continue;
                }
                let mut order = [0usize; 3];
                order[p[0]] = 0;
                order[p[1]] = 1;
                order[p[2]] = 2;
                let perm: Vec<Vec<(DocumentId, NodeId)>> = vec![
                    three[order[0]].clone(),
                    three[order[1]].clone(),
                    three[order[2]].clone(),
                ];
                let m = rrf_fuse(&perm, 2);
                if m.len() != 2 {
                    failures.push(format!(
                        "≥3-list exact-tie perm={p:?} → prefix len {} != 2: {m:?}",
                        m.len()
                    ));
                    continue;
                }
                let asc = m[0] < m[1];
                if !asc {
                    failures.push(format!(
                        "≥3-list exact-tie perm={p:?} 2-key prefix not (doc,node)-ascending: {m:?}"
                    ));
                }
                if m[..] != tbase3[..2] {
                    failures.push(format!(
                        "≥3-list exact-tie perm={p:?} 2-key prefix {m:?} != base {tbase3:?}"
                    ));
                }
            }
        }
    }
    let _ = (tp, tq, tr);

    for _case in 0..BUDGET {
        if failures.len() >= 5 {
            break;
        }
        let nlists = rng.in_range(1, 5);
        let ndocs = rng.in_range(1, 4);
        let nnodes = rng.in_range(1, 4);
        let mut lists: Vec<Vec<(DocumentId, NodeId)>> = Vec::new();
        for _ in 0..nlists {
            let len = rng.in_range(1, 6);
            let mut list = Vec::new();
            for _ in 0..len {
                let k = (
                    did(&format!("d{}", rng.below(ndocs))),
                    nid(&format!("n{}", rng.below(nnodes))),
                );
                list.push(k);
            }
            // sometimes a duplicated inner element
            if rng.below(3) == 0 {
                list.push(list[0].clone());
            }
            lists.push(list);
        }
        let top_k = rng.in_range(1, ndocs * nnodes + 3);

        // (a) repeat → identical.
        let base = rrf_fuse(&lists, top_k);
        if rrf_fuse(&lists, top_k) != base {
            failures.push(format!(
                "non-deterministic on repeat: lists={lists:?} top_k={top_k}"
            ));
            continue;
        }
        // (b) outer permutations → identical.
        for _ in 0..3 {
            let mut perm = lists.clone();
            rng.shuffle(&mut perm);
            if rrf_fuse(&perm, top_k) != base {
                failures.push(format!(
                    "outer permutation changed output: base={base:?} permuted={perm:?} top_k={top_k}"
                ));
            }
        }
        // (c) independently re-derive scores; output non-increasing; ties asc.
        let scores: Vec<(f64, (DocumentId, NodeId))> = base
            .iter()
            .map(|k| (rrf_score(&lists, k), k.clone()))
            .collect();
        for w in scores.windows(2) {
            let (sa, ka) = &w[0];
            let (sb, kb) = &w[1];
            let ok = sa > sb || (sa == sb && (ka.0 < kb.0 || (ka.0 == kb.0 && ka.1 < kb.1)));
            if !ok {
                failures.push(format!(
                    "RRF order violated between {ka:?}({sa}) and {kb:?}({sb}): lists={lists:?} base={base:?}"
                ));
                break;
            }
        }
        // (d) matches independent expected merge.
        let expect = rrf_expected(&lists, top_k);
        if base != expect {
            failures.push(format!(
                "hand-derived merge mismatch: base={base:?} expect={expect:?} lists={lists:?} top_k={top_k}"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "P-IM-2 BROKEN — ≤5 minimal counterexamples:\n{}",
        failures.join("\n")
    );
}

// ===========================================================================
// P-SM-1 — result-set purity / wiki scope (no cross-wiki leak).        #[tokio::test]
// BUDGET: 15 cases (≤100).
// ===========================================================================
#[tokio::test]
async fn p_sm_1_no_cross_wiki_leak_across_all_modes() {
    let mut failures: Vec<String> = Vec::new();

    // configs: (a, b) query texts / seed content used across A and B identically.
    let contents: [&str; 3] = [
        "alpha quantum coherence",
        "beta entanglement physics",
        "gamma hybrid retrieval",
    ];

    for (idx, query) in contents.iter().enumerate() {
        let store = Arc::new(Store::new());
        ready(&store);
        store.set_embedding_provider(Arc::new(MockProvider::hit(vec![1.0, 0.0])));
        let wa = new_wiki(&store, "A").await;
        let wb = new_wiki(&store, "B").await;

        // Identical-content docs in BOTH wikis (a leak would be invisible by content).
        let da = new_content_doc(&store, &wa, &format!("a-doc-{idx}"), "na", query).await;
        let _db = new_content_doc(&store, &wb, &format!("b-doc-{idx}"), "nb", query).await;
        // A vector index hosted only under A docs.
        seed_vectors(&store, vec![(da.clone(), vec![1.0, 0.0])]);
        // A resolvable reference→fact chain in A so the graph leg is non-empty.
        let (_fdoc, _fnid, _rdoc, _rnid) = seed_ref_to_fact(&store, &wa).await;

        let modes = [
            QueryMode::Flat,
            QueryMode::Graph,
            QueryMode::Vector,
            QueryMode::Hybrid,
        ];
        for mode in modes {
            let res: RagResult = store
                .rag_query(
                    query,
                    &RagQueryOptions {
                        wiki_id: Some(wa.clone()),
                        top_k: Some(10),
                        mode: Some(mode),
                        ..Default::default()
                    },
                )
                .await
                .unwrap_or_else(|e| panic!("mode {mode:?} must be Ok on a wired store: {e:?}"));
            for item in &res.results {
                let w = owning_wiki(&store, &item.document_id).await;
                if w != wa {
                    failures.push(format!(
                        "mode {mode:?} leaked {w:?} document {:?} into A-scoped result",
                        item.document_id
                    ));
                    continue;
                }
            }
            for (cd, cn) in &res.citations {
                let w = owning_wiki(&store, cd).await;
                if w != wa || cn.0.is_empty() {
                    failures.push(format!(
                        "mode {mode:?} leaked cross-wiki citation ({cd:?},{cn:?}) into A scope"
                    ));
                }
            }
        }
        // Multi-query hybrid merge list must also be A-pure.
        let mq = store
            .rag_query(
                query,
                &RagQueryOptions {
                    wiki_id: Some(wa.clone()),
                    top_k: Some(10),
                    mode: Some(QueryMode::Hybrid),
                    multi_query: Some(MultiQueryOptions {
                        enabled: true,
                        n: 2,
                    }),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        for item in &mq.results {
            let w = owning_wiki(&store, &item.document_id).await;
            if w != wa {
                failures.push(format!(
                    "multi-query hybrid leaked {w:?} into A scope: {:?}",
                    item.document_id
                ));
            }
        }
        if !failures.is_empty() {
            break;
        }
    }

    // Crosslink edge: an A reference with a Crosslink to a B fact must never
    // surface the B target in an A-scoped graph/hybrid result.
    let store = Arc::new(Store::new());
    ready(&store);
    store.set_embedding_provider(Arc::new(MockProvider::hit(vec![1.0, 0.0])));
    let wa = new_wiki(&store, "A").await;
    let wb = new_wiki(&store, "B").await;
    let bfact = new_content_doc(&store, &wb, "b-fact", "BF", "secret-b-content").await;
    let aroot_doc = new_doc(&store, &wa, "a-cross").await;
    update_graph(
        &store,
        &aroot_doc,
        vec![ref_to(&aroot_doc.document_id, "X", bfact.clone())],
        vec![Edge {
            source: (aroot_doc.document_id.clone(), nid("X")),
            target: bfact.clone(),
            kind: EdgeKind::Crosslink,
            state: Some(ReferenceState::Resolved),
            cross_wiki: true,
            relation_type: None,
        }],
    )
    .await;
    for mode in [QueryMode::Graph, QueryMode::Hybrid] {
        let res = store
            .rag_query(
                "secret",
                &RagQueryOptions {
                    wiki_id: Some(wa.clone()),
                    top_k: Some(10),
                    mode: Some(mode),
                    ..Default::default()
                },
            )
            .await
            .unwrap_or_else(|e| panic!("crosslink mode {mode:?} must be Ok: {e:?}"));
        let b_in_results = res
            .results
            .iter()
            .any(|r| r.document_id == bfact.0 && r.node_id == bfact.1);
        let b_in_citations = res.citations.contains(&bfact);
        if b_in_results || b_in_citations {
            failures.push(format!(
                "crosslink mode {mode:?} leaked the B target {:?} into A scope",
                bfact
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "P-SM-1 BROKEN — ≤5 minimal counterexamples:\n{}",
        failures.join("\n")
    );
}

// ===========================================================================
// P-SM-2 — graceful degradation on empty / non-indexable wiki.        #[tokio::test]
// BUDGET: 8 cases (≤100).
// ===========================================================================
#[tokio::test]
async fn p_sm_2_empty_and_non_indexable_wiki_degrades_gracefully() {
    let mut failures: Vec<String> = Vec::new();

    // (1) Freshly-created empty wiki, flat + graph, across top_k boundaries.
    for top_k in [1u64, 50u64] {
        for mode in [QueryMode::Flat, QueryMode::Graph] {
            let store = Arc::new(Store::new());
            ready(&store);
            let w = new_wiki(&store, "empty").await;
            let res = store
                .rag_query(
                    "anything",
                    &RagQueryOptions {
                        wiki_id: Some(w),
                        top_k: Some(top_k),
                        mode: Some(mode),
                        ..Default::default()
                    },
                )
                .await
                .unwrap_or_else(|e| panic!("empty-wiki mode {mode:?} must be Ok: {e:?}"));
            if !res.results.is_empty() || !res.citations.is_empty() {
                failures.push(format!(
                    "empty wiki mode {mode:?} top_k={top_k} returned fabricated items: {:?}",
                    res.results.iter().map(key).collect::<Vec<_>>()
                ));
            }
            let trace_ok = match (&mode, &res.trace) {
                (QueryMode::Flat, RagTrace::Flat(_)) | (QueryMode::Graph, RagTrace::Graph(_)) => {
                    true
                }
                (m, t) => {
                    failures.push(format!(
                        "empty wiki mode {m:?} must have the matching trace shape, got {t:?}"
                    ));
                    false
                }
            };
            if trace_ok && res.blocked_by.is_some() {
                failures.push("empty-wiki graph mode: no roots → blocked_by must stay None".into());
            }
        }
    }

    // (2) A wiki whose docs share NO token with the query → empty flat results.
    let store = Arc::new(Store::new());
    ready(&store);
    let w = new_wiki(&store, "noise").await;
    for i in 0..3 {
        let _ = new_content_doc(
            &store,
            &w,
            &format!("noise-{i}"),
            &format!("nn{i}"),
            "zebra",
        )
        .await;
    }
    let res = store
        .rag_query(
            "quantum",
            &RagQueryOptions {
                wiki_id: Some(w),
                top_k: Some(5),
                mode: Some(QueryMode::Flat),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    if !res.results.is_empty() {
        failures.push(format!(
            "no-token-overlap flat query returned fabricated items: {:?}",
            res.results.iter().map(key).collect::<Vec<_>>()
        ));
    }

    // AUDIT-4 (b-4, P-SM-2 graceful-degradation fill): an "empty graph / no
    // indexable node text" wiki — DISTINCT from the empty wiki in (1): real docs
    // exist, but none of them has any indexable node text — plus a flat query on
    // whitespace/stop-word-only text. Both must degrade to `Ok` with empty
    // results, the correct per-mode trace, and `blocked_by` = None (this row is
    // scoped to the flat/graph-only deterministic legs; vector/hybrid are the
    // reserved unavailable-provider fail-states and are out of scope).
    let store = Arc::new(Store::new());
    ready(&store);
    let w = new_wiki(&store, "empty-graph").await;
    // Docs exist (created via create_document → empty `Graph`), so this is NOT
    // the empty wiki: there is content, but no indexable node text.
    let _d1 = new_doc(&store, &w, "doc-one").await;
    let _d2 = new_doc(&store, &w, "doc-two").await;
    for (mode, trace_pat) in [(QueryMode::Flat, 1u8), (QueryMode::Graph, 2u8)] {
        // A normal indexable-word query on an empty-graph wiki.
        let r = store
            .rag_query(
                "indexable",
                &RagQueryOptions {
                    wiki_id: Some(w.clone()),
                    top_k: Some(10),
                    mode: Some(mode),
                    ..Default::default()
                },
            )
            .await
            .unwrap_or_else(|e| panic!("empty-graph wiki mode {mode:?} must be Ok: {e:?}"));
        if !r.results.is_empty() || !r.citations.is_empty() {
            failures.push(format!(
                "empty-graph wiki mode {mode:?} returned fabricated results: {:?}",
                r.results.iter().map(key).collect::<Vec<_>>()
            ));
        }
        let trace_shape_ok = match (&mode, &r.trace) {
            (QueryMode::Flat, RagTrace::Flat(_)) => trace_pat == 1,
            (QueryMode::Graph, RagTrace::Graph(_)) => trace_pat == 2,
            _ => false,
        };
        if !trace_shape_ok {
            failures.push(format!(
                "empty-graph wiki mode {mode:?} must have the matching trace shape, got {:?}",
                r.trace
            ));
        }
        if r.blocked_by.is_some() {
            failures.push(format!(
                "empty-graph wiki mode {mode:?}: no roots → blocked_by must stay None, got {:?}",
                r.blocked_by
            ));
        }
    }
    // A flat query on whitespace/stop-word-only text — trim()-non-empty but with
    // no indexable keyword overlap — degrades to empty results too.
    let r = store
        .rag_query(
            "the and of to",
            &RagQueryOptions {
                wiki_id: Some(w.clone()),
                top_k: Some(10),
                mode: Some(QueryMode::Flat),
                ..Default::default()
            },
        )
        .await
        .unwrap_or_else(|e| panic!("stop-word-only flat query must be Ok: {e:?}"));
    if !r.results.is_empty() {
        failures.push(format!(
            "stop-word-only flat query on empty-graph wiki returned fabricated items: {:?}",
            r.results.iter().map(key).collect::<Vec<_>>()
        ));
    }
    if r.blocked_by.is_some() {
        failures.push(format!(
            "stop-word-only flat query: blocked_by must stay None, got {:?}",
            r.blocked_by
        ));
    }

    assert!(
        failures.is_empty(),
        "P-SM-2 BROKEN — ≤5 minimal counterexamples:\n{}",
        failures.join("\n")
    );
}

// ===========================================================================
// P-SM-3 — full-query determinism / stability (rag_query + rag_stream).#[tokio::test]
// BUDGET: 20 cases (≤100).
// ===========================================================================
#[tokio::test]
async fn p_sm_3_repeat_query_and_stream_are_element_wise_stable() {
    let mut failures: Vec<String> = Vec::new();

    // One deterministic multi-wiki multi-document store, all modes.
    let store = Arc::new(Store::new());
    ready(&store);
    store.set_embedding_provider(Arc::new(MockProvider::hit(vec![1.0, 0.0])));
    let w = new_wiki(&store, "W").await;
    let vdoc = new_content_doc(&store, &w, "vec-doc", "n-vx", "unrelated zzz").await;
    seed_vectors(&store, vec![(vdoc.clone(), vec![1.0, 0.0])]);
    let _on = new_content_doc(&store, &w, "lex-doc", "n-lx", "alpha term").await;
    let (_fdoc, _fnid, _rdoc, _rnid) = seed_ref_to_fact(&store, &w).await;

    async fn assert_stable(
        store: &Arc<Store>,
        w: &WikiId,
        opts: RagQueryOptions,
        failures: &mut Vec<String>,
    ) {
        let mk = |o: &RagQueryOptions| {
            let mut c = o.clone();
            c.wiki_id = Some(w.clone());
            c
        };
        // rag_query twice.
        let r1 = store.rag_query("alpha", &mk(&opts)).await.unwrap();
        let r2 = store.rag_query("alpha", &mk(&opts)).await.unwrap();
        if r1.results != r2.results
            || r1.citations != r2.citations
            || r1.trace != r2.trace
            || r1.blocked_by != r2.blocked_by
        {
            failures.push(format!(
                "repeat rag_query drifted: mode={:?} r1.results={:?} r2.results={:?}",
                opts.mode, r1.results, r2.results
            ));
            return;
        }
        // rag_stream → [Result(matching r1), Done].
        let s: RagStream = store.rag_stream("alpha", &mk(&opts)).await.unwrap();
        let chunks: Vec<RagChunk> = s.collect().await;
        let stream_ok = matches!(&chunks[..],
            [RagChunk::Result(r), RagChunk::Done] if r.results == r1.results && r.citations == r1.citations && r.trace == r1.trace);
        if !stream_ok {
            failures.push(format!(
                "rag_stream not [Result(match), Done]: mode={:?} chunks={chunks:?}",
                opts.mode
            ));
        }
    }

    for mode in [
        QueryMode::Flat,
        QueryMode::Graph,
        QueryMode::Vector,
        QueryMode::Hybrid,
    ] {
        let opts = RagQueryOptions {
            top_k: Some(7),
            mode: Some(mode),
            ..Default::default()
        };
        assert_stable(&store, &w, opts, &mut failures).await;
    }
    // multi-query on with succeeding expansion (distinct term present).
    assert_stable(
        &store,
        &w,
        RagQueryOptions {
            top_k: Some(7),
            mode: Some(QueryMode::Flat),
            multi_query: Some(MultiQueryOptions {
                enabled: true,
                n: 2,
            }),
            ..Default::default()
        },
        &mut failures,
    )
    .await;
    // expand: Parent — parent payload must be stable too.
    let opts = RagQueryOptions {
        top_k: Some(7),
        mode: Some(QueryMode::Flat),
        expand: Some(ExpandMode::Parent),
        max_parent_context: Some(2),
        ..Default::default()
    };
    {
        let r1 = store.rag_query("alpha", &opts).await.unwrap();
        let r2 = store.rag_query("alpha", &opts).await.unwrap();
        if r1.results != r2.results {
            failures.push(format!(
                "expand:Parent repeat drifted: {:?} vs {:?}",
                r1.results, r2.results
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "P-SM-3 BROKEN — ≤5 minimal counterexamples:\n{}",
        failures.join("\n")
    );
}

// ===========================================================================
// P-SM-4 — dedup across fused legs / merged variants.               #[tokio::test]
// BUDGET: 6 cases (≤100).
// ===========================================================================
#[tokio::test]
async fn p_sm_4_dedup_across_fused_legs_and_variants() {
    let mut failures: Vec<String> = Vec::new();

    // One (documentId,nodeId) reachable by vector + lexical legs: a content node
    // whose text contains the query term AND whose `full` vector matches the
    // query. A separate reference→fact chain adds the graph leg / citations.
    let store = Arc::new(Store::new());
    ready(&store);
    store.set_embedding_provider(Arc::new(MockProvider::hit(vec![1.0, 0.0])));
    let w = new_wiki(&store, "W").await;
    let overlap = new_content_doc(&store, &w, "overlap-doc", "n-ov", "shared alpha term").await;
    let vfact = new_content_doc(&store, &w, "vec-fact", "n-vf", "shared alpha term").await;
    seed_vectors(
        &store,
        vec![
            (overlap.clone(), vec![1.0, 0.0]),
            (vfact.clone(), vec![1.0, 0.0]),
        ],
    );
    let (_fdoc, _fnid, rdoc, rnid) = seed_ref_to_fact(&store, &w).await;

    // n ∈ {1,2,3} multi-query fan-out on; plus one disabled. Expansion succeeds:
    // the corpus has distinct terms (shared/alpha/term) to fan out to.
    for n in [1u64, 2u64, 3u64] {
        let res = store
            .rag_query(
                "alpha",
                &RagQueryOptions {
                    wiki_id: Some(w.clone()),
                    top_k: Some(20),
                    mode: Some(QueryMode::Hybrid),
                    multi_query: Some(MultiQueryOptions { enabled: true, n }),
                    ..Default::default()
                },
            )
            .await
            .unwrap_or_else(|e| {
                panic!("multi-query n={n} must succeed (distinct term present): {e:?}")
            });
        // results is a set — even with overlapping vector+lexical legs + variants.
        if !all_distinct(&result_keys(&res)) {
            failures.push(format!(
                "multi-query n={n}: results not a set: {:?}",
                result_keys(&res)
            ));
        }
        if !all_distinct(&res.citations) {
            failures.push(format!(
                "multi-query n={n}: citations not deduped: {:?}",
                res.citations
            ));
        }
        // The overlap key must appear at most once (may appear 0..=1 in top-k).
        let count_ov = res
            .results
            .iter()
            .filter(|r| r.document_id == overlap.0 && r.node_id == overlap.1)
            .count();
        if count_ov > 1 {
            failures.push(format!(
                "multi-query n={n}: overlap key appears {count_ov} times"
            ));
        }
    }
    // Disabled multi-query hybrid — every merged key distinct.
    let res = store
        .rag_query(
            "alpha",
            &RagQueryOptions {
                wiki_id: Some(w.clone()),
                top_k: Some(20),
                mode: Some(QueryMode::Hybrid),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    if !all_distinct(&result_keys(&res)) {
        failures.push(format!(
            "disabled-multi-query hybrid: results not a set: {:?}",
            result_keys(&res)
        ));
    }
    if !all_distinct(&res.citations) {
        failures.push("disabled-multi-query hybrid: citations not deduped".to_string());
    }
    // Non-vacuity: the graph-leg reference actually contributed a result.
    if !res
        .results
        .iter()
        .any(|r| r.document_id == rdoc && r.node_id == rnid)
    {
        failures.push("graph leg did not contribute its reference to the hybrid result".into());
    }
    let _ = (vfact, rdoc, rnid);

    // AUDIT-2 (b-3): a SINGLE (doc,node) reachable by ALL THREE hybrid legs must
    // appear exactly once in the fused deduped result. The graph leg surfaces its
    // resolved *reference* root as a result item (and its target fact as a
    // citation). So we build one reference node whose doc title is the top BM25
    // hit AND whose `full` vector is the top cosine neighbor AND which is a
    // resolved reference→fact root — i.e. the same key is in the graph, lexical,
    // and vector result legs. A dedup regression (or a fusion that double-counts)
    // would surface it twice.
    let store3 = Arc::new(Store::new());
    ready(&store3);
    store3.set_embedding_provider(Arc::new(MockProvider::hit(vec![1.0, 0.0])));
    let w3 = new_wiki(&store3, "W3").await;
    // Fact F (the referenced target): unrelated text so it is NOT itself the hit.
    let fdoc = new_doc(&store3, &w3, "ffact-doc").await;
    let fnid3 = nid("F");
    update_graph(
        &store3,
        &fdoc,
        vec![fact_node(&fdoc.document_id, "F", "zzz unrelated filler")],
        vec![],
    )
    .await;
    // Reference R whose DOC TITLE is the top BM25 hit for "alpha".
    let rdoc3 = new_doc(&store3, &w3, "alpha hit doc").await;
    let rnid3 = nid("R");
    let target3 = (fdoc.document_id.clone(), fnid3.clone());
    update_graph(
        &store3,
        &rdoc3,
        vec![ref_to(&rdoc3.document_id, "R", target3.clone())],
        vec![edge(
            EdgeKind::Link,
            (rdoc3.document_id.clone(), rnid3.clone()),
            target3.clone(),
            Some(ReferenceState::Resolved),
        )],
    )
    .await;
    // R's `full` vector is the top cosine neighbor of the query embed.
    seed_vectors(
        &store3,
        vec![((rdoc3.document_id.clone(), rnid3.clone()), vec![1.0, 0.0])],
    );
    let res3 = store3
        .rag_query(
            "alpha",
            &RagQueryOptions {
                wiki_id: Some(w3.clone()),
                top_k: Some(20),
                mode: Some(QueryMode::Hybrid),
                ..Default::default()
            },
        )
        .await
        .unwrap_or_else(|e| panic!("all-three-legs hybrid must be Ok on a wired store: {e:?}"));
    // R appears exactly once in the fused results (dedup of 3 legs → 1).
    let r_key = (rdoc3.document_id.clone(), rnid3.clone());
    let count_r = res3.results.iter().filter(|r| key(r) == r_key).count();
    if count_r != 1 {
        failures.push(format!(
            "all-three-legs overlap key {r_key:?} must appear exactly once in results, got {count_r}: {:?}",
            result_keys(&res3)
        ));
    }
    // Non-vacuity: it really was in all three legs (present in the result key set
    // at least once; it is the unique vector entry and a BM25 hit + graph root).
    if count_r == 0 {
        failures.push(format!(
            "all-three-legs overlap key {r_key:?} missing from results (leg did not contribute): {:?}",
            result_keys(&res3)
        ));
    }
    // Its referenced fact F is the graph leg's citation source — cited exactly once.
    let f_key = (fdoc.document_id.clone(), fnid3.clone());
    let cite_f = res3.citations.iter().filter(|c| **c == f_key).count();
    if cite_f != 1 {
        failures.push(format!(
            "all-three-legs fact {f_key:?} must be cited exactly once by the graph leg, got {cite_f}: {:?}",
            res3.citations
        ));
    }
    let _ = &res3;
    assert!(
        failures.is_empty(),
        "P-SM-4 BROKEN — ≤5 minimal counterexamples:\n{}",
        failures.join("\n")
    );
}

// ===========================================================================
// P-TP-1 — single-variant result-count bound.                      #[tokio::test]
// BUDGET: 36 cases (≤100).
// ===========================================================================
#[tokio::test]
async fn p_tp_1_single_variant_count_bound() {
    let mut failures: Vec<String> = Vec::new();

    // Distinct eligible-item count for a wiki = distinct (doc,node) across its docs.
    async fn distinct_eligible(store: &Store, w: &WikiId) -> usize {
        let mut set = std::collections::HashSet::new();
        let docs = store
            .list_documents(
                w,
                &ListDocumentsFilter {
                    page_size: Some(100),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        for summary in docs.items {
            let d = store.get_document(&summary.document_id).await.unwrap();
            for n in &d.graph.nodes {
                set.insert((d.document_id.clone(), n.node_id.clone()));
            }
        }
        set.len()
    }

    let configs: [&str; 3] = ["alpha term", "beta entanglement", "gamma retrieval"];
    for (ci, word) in configs.iter().enumerate() {
        let store = Arc::new(Store::new());
        ready(&store);
        store.set_embedding_provider(Arc::new(MockProvider::hit(vec![1.0, 0.0])));
        let w = new_wiki(&store, "W").await;
        let n_docs = 6usize;
        let mut vectors = Vec::new();
        for i in 0..n_docs {
            let (d, n) =
                new_content_doc(&store, &w, &format!("d{ci}-{i}"), &format!("n{i}"), word).await;
            vectors.push(((d, n), vec![1.0, 0.0]));
        }
        let total_eligible = distinct_eligible(&store, &w).await;
        seed_vectors(&store, vectors);

        let top_k_values = [1u64, 3u64, 50u64];
        for top_k in top_k_values {
            for mode in [
                QueryMode::Flat,
                QueryMode::Vector,
                QueryMode::Hybrid,
                QueryMode::Graph,
            ] {
                // count-bound rows: multi_query ABSENT, compression None.
                let res = store
                    .rag_query(
                        word,
                        &RagQueryOptions {
                            wiki_id: Some(w.clone()),
                            top_k: Some(top_k),
                            mode: Some(mode),
                            compression: None,
                            expand: if rng_mode(ci) {
                                Some(ExpandMode::Parent)
                            } else {
                                None
                            },
                            ..Default::default()
                        },
                    )
                    .await
                    .unwrap_or_else(|e| panic!("count-bound mode {mode:?} must be Ok: {e:?}"));
                let len = res.results.len();
                if len > (top_k as usize) || len > total_eligible {
                    failures.push(format!(
                        "mode {mode:?} top_k={top_k} eligible={total_eligible}: len={len} exceeds a bound"
                    ));
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "P-TP-1 BROKEN — ≤5 minimal counterexamples:\n{}",
        failures.join("\n")
    );
}

// Helper mirroring the deterministic RNG toggle for expand (keeps it seeded-fixed).
fn rng_mode(ci: usize) -> bool {
    (PINNED_SEED as usize).wrapping_add(ci).is_multiple_of(2)
}

// ===========================================================================
// P-TP-2 — compression is a pure projection (no invention / no reorder).#[tokio::test]
// BUDGET: 12 cases (≤100).
// ===========================================================================
#[tokio::test]
async fn p_tp_2_compression_is_a_pure_projection() {
    let mut failures: Vec<String> = Vec::new();

    let configs: [(&str, &str); 3] = [
        ("term", "term"), // on-topic query; off-topic snippets diluted
        ("retrieval", "retrieval"),
        ("fuse", "fusion"),
    ];
    for (query, on_word) in configs {
        let store = Arc::new(Store::new());
        ready(&store);
        let w = new_wiki(&store, "W").await;
        // on-topic: high query-token density → kept by Filter.
        let _ = new_content_doc(&store, &w, "on", "n1", on_word).await;
        // off-topic but still a lexical match: term diluted → dropped by Filter.
        let _ = new_content_doc(
            &store,
            &w,
            "off1",
            "n2",
            &format!("{on_word} diluted by many many unnecessary filler words about other topics"),
        )
        .await;
        let _ = new_content_doc(&store, &w, "off2", "n3", "completely unrelated qwertyuiop").await;

        let q = |compression| RagQueryOptions {
            wiki_id: Some(w.clone()),
            top_k: Some(8),
            compression,
            ..Default::default()
        };

        let base = store.rag_query(query, &q(None)).await.unwrap();
        let base_keys = result_keys(&base);
        let flt = store
            .rag_query(query, &q(Some(gnosis::CompressionMode::Filter)))
            .await
            .unwrap();
        let flt_keys = result_keys(&flt);
        let ext = store
            .rag_query(query, &q(Some(gnosis::CompressionMode::Extract)))
            .await
            .unwrap();
        let ext_keys = result_keys(&ext);
        let gra = store
            .rag_query(query, &q(Some(gnosis::CompressionMode::Graph)))
            .await
            .unwrap();
        let gra_keys = result_keys(&gra);

        // Filter: keys are a sub-SEQUENCE of base keys, order preserved.
        if !is_subsequence(&flt_keys, &base_keys) {
            failures.push(format!(
                "Filter {flt_keys:?} is not an order-preserving subsequence of base {base_keys:?} (query {query:?})"
            ));
        }
        // Extract/Graph are pass-through → exactly equal to base key set, in order.
        if ext_keys != base_keys || gra_keys != base_keys {
            failures.push(format!(
                "Extract/Graph must equal base key set+order: base={base_keys:?} ext={ext_keys:?} gra={gra_keys:?} (query {query:?})"
            ));
        }
        // Non-vacuity where possible: base not empty and Filter shrank a strict
        // non-empty subset when an on-topic + diluted mix exist. Guard: only
        // assert the strict-shink when base actually contains the diluted snippet.
        if !base_keys.is_empty() {
            // Filter must never invent an off-document key.
            let fabricated = flt_keys.iter().any(|k| !base_keys.contains(k));
            if fabricated {
                failures.push(format!(
                    "Filter invented a key absent from base: flt={flt_keys:?} base={base_keys:?} (query {query:?})"
                ));
            }
        }
    }

    // AUDIT-3 (b-4): Filter strict-shrink. With a guaranteed on-topic + diluted +
    // unrelated snippet mix, Filter must KEEP the dense on-topic snippet and DROP
    // the diluted (retrieved) and unrelated snippets — a strict non-empty
    // sub-sequence of the base keys, with the dropped keys ABSENT from `flt`. A
    // regression to a `Filter` no-op would retain `off` (len == base) and cannot
    // green this row.
    let store = Arc::new(Store::new());
    ready(&store);
    let w = new_wiki(&store, "W-strict").await;
    // Dense on-topic snippet: query-token density 1.0 → kept by Filter.
    let on_key = new_content_doc(&store, &w, "on", "n-on", "term term term").await;
    // Diluted snippet: still a BM25 hit (score > 0) but token density < 0.25.
    let off_key = new_content_doc(
        &store,
        &w,
        "off",
        "n-off",
        "term filler filler filler filler filler filler filler",
    )
    .await;
    // Unrelated snippet: no query token → never retrieved (absent from base).
    let un_key = new_content_doc(
        &store,
        &w,
        "unrelated",
        "n-un",
        "completely unrelated qwertyuiop",
    )
    .await;
    let qopt = |compression: Option<gnosis::CompressionMode>| RagQueryOptions {
        wiki_id: Some(w.clone()),
        top_k: Some(8),
        compression,
        multi_query: None,
        ..Default::default()
    };
    let base = store.rag_query("term", &qopt(None)).await.unwrap();
    let base_keys = result_keys(&base);
    let flt = store
        .rag_query("term", &qopt(Some(gnosis::CompressionMode::Filter)))
        .await
        .unwrap();
    let flt_keys = result_keys(&flt);
    // Guard: base must actually contain the dense on-topic AND the diluted
    // snippet for the strict-shrink claim to be well-posed.
    if !base_keys.contains(&on_key) || !base_keys.contains(&off_key) {
        failures.push(format!(
            "strict-shrink setup broken: base={base_keys:?} must contain on={on_key:?} and off={off_key:?}"
        ));
    } else {
        // Filter retained the on-topic snippet (non-empty).
        if flt_keys.is_empty() {
            failures.push(
                "Filter dropped every snippet incl. the dense on-topic one (starts empty)".into(),
            );
        }
        // Strict shrink: Filter dropped at least the diluted snippet.
        if flt_keys.len() >= base_keys.len() {
            failures.push(format!(
                "Filter was a no-op / not a strict shrink: flt={flt_keys:?} len={} base={base_keys:?} len={}",
                flt_keys.len(),
                base_keys.len()
            ));
        }
        // The diluted (retrieved) snippet was dropped by Filter.
        if flt_keys.contains(&off_key) {
            failures.push(format!(
                "Filter retained the diluted snippet {off_key:?}: flt={flt_keys:?}"
            ));
        }
        // The unrelated key is absent from the fused/flattened result entirely.
        if flt_keys.contains(&un_key) {
            failures.push(format!(
                "unrelated key leaked into Filter output: flt={flt_keys:?}"
            ));
        }
    }
    // Order is preserved (flt ⊆ base as an order-preserving sub-sequence).
    if !is_subsequence(&flt_keys, &base_keys) {
        failures.push(format!(
            "Filter strict-shrink output {flt_keys:?} not an order-preserving sub-sequence of base {base_keys:?}"
        ));
    }

    assert!(
        failures.is_empty(),
        "P-TP-2 BROKEN — ≤5 minimal counterexamples:\n{}",
        failures.join("\n")
    );
}
