//! §4.2 Knowledge-graph integration tests — written **RED-first** at the TDD gate.
//!
//! Every state and fail-state below is derived from the canonical behavior
//! contract alone (`docs/specs/gnosis.md` §4.2.1–§4.2.9, §4.4.2, §4.4.3) plus the
//! concurrency contract (`docs/research/gnosis-data-structures-concurrency-plan.md`:
//! SHARDED-RWLOCK-STORE / ARC-SHARED-ENGINE).
//!
//! ## The suite is RED
//!
//! Every §4.2 operation on `Store` has an `unimplemented!()` stub body, so every
//! assertion below fails at runtime until the Implementer lands the real graph
//! logic. The §4.1 document-store suite (`tests/store_integration.rs`, 50/50)
//! is intentionally untouched and stays green.
//!
//! ## TestWriter reading of the frozen §4.1 shapes
//!
//! The §4.1 `Node`/`Edge` model is **frozen** (the Implementer must not add
//! fields to them — that would break the §4.1 green suite's literal
//! constructions). Consequences, decided up-front so the Implementer matches
//! exactly:
//! - **Triple → `relation` edge.** A triple is owned by the graph as an `Edge`
//!   of kind `Relation` with the edge's `source` = triple subject and `target` =
//!   triple object (§4.2.7.2). The `relationType` string is carried by the
//!   returned `Triple` value; the structural `relation` edge identity is what the
//!   graph owns and adjacency exposes.
//! - **Community → `community` node + `member` edges.** A declared community is
//!   stored in the authoritative graph as a `Node` of kind `Community` whose
//!   members are connected via `Edge`s of kind `Member` (§4.2.8.3). Tests assert
//!   observable behavior through `declareCommunity`/`getCommunity`/
//!   `listCommunities`/`updateCommunitySummary` and pin the `Community` node kind
//!   and `Member` edge kind at the type level.
//! - **Fact citations.** The frozen `Node` cannot carry a per-fact `citations`
//!   set (§4.3.1), so facts with citations are established via `createFact` — the
//!   spec-documented manual override (§4.2.8.1/§4.3.1) — which `mergeFacts`
//!   (§4.2.9.1) reads.

use std::sync::Arc;

use gnosis::{
    CommunityId, CreateDocumentRequest, DeclareCommunityOptions, DocumentId, Edge, EdgeKind,
    GetTriplesFilter, Graph, MergeFactsOptions, Node, NodeId, NodeKind, QueryTriplesOptions,
    RagStore, ReferenceState, ResolveEntitiesOptions, ResolveOptions, Store, StoreError,
    TripleDirection, TriplePattern, UpdateDocumentRequest, WikiId,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn wiki(s: &str) -> WikiId {
    WikiId(s.to_string())
}

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

fn fact_node(doc: &DocumentId, node: &str, key: &str, value: &str) -> Node {
    Node {
        document_id: doc.clone(),
        node_id: nid(node),
        kind: NodeKind::Fact,
        value: Some(value.to_string()),
        fact_key: Some(key.to_string()),
        target: None,
    }
}

/// A `reference` node pointing at `target`.
fn reference_node(doc: &DocumentId, node: &str, target: (DocumentId, NodeId)) -> Node {
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

/// Create an empty document (rev 0, DRAFT) in `wiki_id`.
async fn new_doc(store: &Store, w: &WikiId, title: &str) -> gnosis::Document {
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

/// Apply a graph to `doc` from its **current** revision. Every graph here
/// carries exactly one `doc-head` and one `doc-end` edge, so `update_document`'s
/// validation (§4.1) accepts it. Reading the current revision up-front keeps the
/// §4.1.4 optimistic-concurrency guard happy even if a prior §4.2 op (e.g.
/// `addTriple`) bumped the revision.
async fn apply_graph(store: &Store, doc: &gnosis::Document, graph: Graph) {
    let cur = store.get_document(&doc.document_id).await.unwrap();
    store
        .update_document(
            &doc.document_id,
            UpdateDocumentRequest {
                base_revision: cur.revision,
                graph,
                title: None,
                tags: None,
            },
        )
        .await
        .unwrap();
}

/// A "known" graph: content nodes n1, n2 and reference ref1 with a
/// `doc-head`/`doc-child`/`doc-end`/`link` edge set (§4.2.5 test fixture).
fn adjacency_graph(doc: &DocumentId) -> Graph {
    let n1 = content_node(doc, "n1", "hello");
    let n2 = content_node(doc, "n2", "world");
    let ref1 = reference_node(doc, "ref1", (did("other-doc"), nid("tgt")));
    Graph {
        nodes: vec![n1.clone(), n2.clone(), ref1.clone()],
        edges: vec![
            edge(
                EdgeKind::DocHead,
                (doc.clone(), nid("ROOT")),
                (doc.clone(), n1.node_id.clone()),
                None,
            ),
            edge(
                EdgeKind::DocChild,
                (doc.clone(), n1.node_id.clone()),
                (doc.clone(), n2.node_id.clone()),
                None,
            ),
            edge(
                EdgeKind::DocEnd,
                (doc.clone(), n2.node_id.clone()),
                (doc.clone(), nid("END")),
                None,
            ),
            edge(
                EdgeKind::Link,
                (doc.clone(), ref1.node_id.clone()),
                (did("other-doc"), nid("tgt")),
                Some(ReferenceState::Resolved),
            ),
        ],
    }
}

/// Content nodes n1..n4 (no connecting edges) — a valid Provident graph the
/// `addTriple`/`getTriples` surface can hang `relation` edges on.
fn content_only_graph(doc: &DocumentId, names: &[&str]) -> Graph {
    let nodes: Vec<Node> = names.iter().map(|n| content_node(doc, n, "v")).collect();
    Graph {
        nodes: nodes.clone(),
        edges: vec![
            edge(
                EdgeKind::DocHead,
                (doc.clone(), nid("ROOT")),
                (doc.clone(), nodes[0].node_id.clone()),
                None,
            ),
            edge(
                EdgeKind::DocEnd,
                (doc.clone(), nodes[0].node_id.clone()),
                (doc.clone(), nid("END")),
                None,
            ),
        ],
    }
}

/// A topological chain `r_top → r_mid → f_leaf` plus a side reference
/// `r_side → f_leaf`, so dependents share `f_leaf` (§4.2.6 fixture).
fn ref_chain_graph(doc: &DocumentId) -> Graph {
    let f_leaf = fact_node(doc, "f_leaf", "f_leaf", "leaf-val");
    let r_mid = reference_node(doc, "r_mid", (doc.clone(), nid("f_leaf")));
    let r_top = reference_node(doc, "r_top", (doc.clone(), nid("r_mid")));
    let r_side = reference_node(doc, "r_side", (doc.clone(), nid("f_leaf")));
    Graph {
        nodes: vec![f_leaf.clone(), r_mid.clone(), r_top.clone(), r_side.clone()],
        edges: vec![
            edge(
                EdgeKind::DocHead,
                (doc.clone(), nid("ROOT")),
                (doc.clone(), f_leaf.node_id.clone()),
                None,
            ),
            edge(
                EdgeKind::DocEnd,
                (doc.clone(), f_leaf.node_id.clone()),
                (doc.clone(), nid("END")),
                None,
            ),
            edge(
                EdgeKind::Link,
                (doc.clone(), r_mid.node_id.clone()),
                (doc.clone(), nid("f_leaf")),
                Some(ReferenceState::Resolved),
            ),
            edge(
                EdgeKind::Link,
                (doc.clone(), r_top.node_id.clone()),
                (doc.clone(), nid("r_mid")),
                Some(ReferenceState::Resolved),
            ),
            edge(
                EdgeKind::Link,
                (doc.clone(), r_side.node_id.clone()),
                (doc.clone(), nid("f_leaf")),
                Some(ReferenceState::Resolved),
            ),
        ],
    }
}

/// A `reference` cycle `r_a → r_b → r_a` (§4.2.6 CycleDetected fixture).
fn ref_cycle_graph(doc: &DocumentId) -> Graph {
    let n0 = content_node(doc, "n0", "root");
    let r_a = reference_node(doc, "r_a", (doc.clone(), nid("r_b")));
    let r_b = reference_node(doc, "r_b", (doc.clone(), nid("r_a")));
    Graph {
        nodes: vec![n0.clone(), r_a.clone(), r_b.clone()],
        edges: vec![
            edge(
                EdgeKind::DocHead,
                (doc.clone(), nid("ROOT")),
                (doc.clone(), n0.node_id.clone()),
                None,
            ),
            edge(
                EdgeKind::DocEnd,
                (doc.clone(), n0.node_id.clone()),
                (doc.clone(), nid("END")),
                None,
            ),
            edge(
                EdgeKind::Link,
                (doc.clone(), r_a.node_id.clone()),
                (doc.clone(), nid("r_b")),
                Some(ReferenceState::Resolved),
            ),
            edge(
                EdgeKind::Link,
                (doc.clone(), r_b.node_id.clone()),
                (doc.clone(), nid("r_a")),
                Some(ReferenceState::Resolved),
            ),
        ],
    }
}

/// A long chain `r1→f_leaf, r2→r1, r3→r2, r4→r3` (§4.2.6 HopLimitExceeded fixture).
fn ref_hop_overflow_graph(doc: &DocumentId) -> Graph {
    let f_leaf = fact_node(doc, "f_leaf", "f_leaf", "leaf");
    let mut nodes = vec![f_leaf.clone()];
    let mut edges: Vec<Edge> = vec![
        edge(
            EdgeKind::DocHead,
            (doc.clone(), nid("ROOT")),
            (doc.clone(), f_leaf.node_id.clone()),
            None,
        ),
        edge(
            EdgeKind::DocEnd,
            (doc.clone(), f_leaf.node_id.clone()),
            (doc.clone(), nid("END")),
            None,
        ),
    ];
    // r1 -> f_leaf
    nodes.push(reference_node(doc, "r1", (doc.clone(), nid("f_leaf"))));
    edges.push(edge(
        EdgeKind::Link,
        (doc.clone(), nid("r1")),
        (doc.clone(), nid("f_leaf")),
        Some(ReferenceState::Resolved),
    ));
    for (i, prev) in ["r1", "r2", "r3"].into_iter().enumerate() {
        let cur = ["r2", "r3", "r4"][i];
        nodes.push(reference_node(doc, cur, (doc.clone(), nid(prev))));
        edges.push(edge(
            EdgeKind::Link,
            (doc.clone(), nid(cur)),
            (doc.clone(), nid(prev)),
            Some(ReferenceState::Resolved),
        ));
    }
    Graph { nodes, edges }
}

// ---------------------------------------------------------------------------
// §4.2.5 — Adjacency methods
// ---------------------------------------------------------------------------

/// edgesFrom returns every edge whose source is `(documentId, nodeId)`.
#[tokio::test]
async fn edges_from_returns_outgoing_edges() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "adjacency").await;
    apply_graph(&store, &doc, adjacency_graph(&doc.document_id)).await;

    let from_n1 = store
        .edges_from(&doc.document_id, &nid("n1"))
        .await
        .unwrap();
    assert_eq!(from_n1.len(), 1, "n1 has exactly the doc-child edge out");
    assert_eq!(from_n1[0].kind, EdgeKind::DocChild);
    assert_eq!(from_n1[0].source, (doc.document_id.clone(), nid("n1")));

    let from_ref1 = store
        .edges_from(&doc.document_id, &nid("ref1"))
        .await
        .unwrap();
    assert_eq!(from_ref1.len(), 1);
    assert_eq!(from_ref1[0].kind, EdgeKind::Link);
}

/// edgesTo returns every edge whose target is `(documentId, nodeId)`.
#[tokio::test]
async fn edges_to_returns_incoming_edges() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "adjacency").await;
    apply_graph(&store, &doc, adjacency_graph(&doc.document_id)).await;

    let to_n1 = store.edges_to(&doc.document_id, &nid("n1")).await.unwrap();
    assert_eq!(to_n1.len(), 1);
    assert_eq!(to_n1[0].kind, EdgeKind::DocHead);
    assert_eq!(to_n1[0].target, (doc.document_id.clone(), nid("n1")));

    let to_n2 = store.edges_to(&doc.document_id, &nid("n2")).await.unwrap();
    assert_eq!(to_n2.len(), 1);
    assert_eq!(to_n2[0].kind, EdgeKind::DocChild);
}

/// edgesByKind filters outgoing edges by kind.
#[tokio::test]
async fn edges_by_kind_filters_by_kind() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "adjacency").await;
    apply_graph(&store, &doc, adjacency_graph(&doc.document_id)).await;

    let child = store
        .edges_by_kind(&doc.document_id, &nid("n1"), EdgeKind::DocChild)
        .await
        .unwrap();
    assert_eq!(child.len(), 1);
    assert_eq!(child[0].kind, EdgeKind::DocChild);

    let head = store
        .edges_by_kind(&doc.document_id, &nid("n1"), EdgeKind::DocHead)
        .await
        .unwrap();
    assert!(head.is_empty(), "no DocHead edge originates at n1");
}

/// edgesForDocument returns all edges in the document.
#[tokio::test]
async fn edges_for_document_returns_all_edges() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "adjacency").await;
    apply_graph(&store, &doc, adjacency_graph(&doc.document_id)).await;

    let all = store.edges_for_document(&doc.document_id).await.unwrap();
    assert_eq!(all.len(), 4, "doc-head + doc-child + doc-end + link");
    assert!(all.iter().all(|e| e.source.0 == doc.document_id));
}

/// docHeadForDocument returns the head node (the target of the `doc-head` edge).
#[tokio::test]
async fn doc_head_for_document_returns_head_node() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "adjacency").await;
    apply_graph(&store, &doc, adjacency_graph(&doc.document_id)).await;

    let head = store.doc_head_for_document(&doc.document_id).await.unwrap();
    assert_eq!(head.node_id, nid("n1"));
    assert_eq!(head.value.as_deref(), Some("hello"));
    assert_eq!(head.kind, NodeKind::Content);
}

/// edgesFrom on an unknown `documentId` → `DocumentNotFound` (FS-1).
#[tokio::test]
async fn edges_from_unknown_document_is_document_not_found() {
    let store = Arc::new(Store::new());
    let err = store
        .edges_from(&did("ghost"), &nid("n1"))
        .await
        .unwrap_err();
    assert_eq!(err, StoreError::DocumentNotFound, "FS-1");
}

/// edgesFrom with a `nodeId` that is not a node in the document →
/// `ValidationError` (FS-3).
#[tokio::test]
async fn edges_from_invalid_node_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "adjacency").await;
    apply_graph(&store, &doc, adjacency_graph(&doc.document_id)).await;
    let err = store
        .edges_from(&doc.document_id, &nid("GHOST"))
        .await
        .unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "FS-3 invalid nodeId"
    );
}

/// edgesTo on an unknown `documentId` → `DocumentNotFound` (FS-1).
#[tokio::test]
async fn edges_to_unknown_document_is_document_not_found() {
    let store = Arc::new(Store::new());
    let err = store.edges_to(&did("ghost"), &nid("n1")).await.unwrap_err();
    assert_eq!(err, StoreError::DocumentNotFound, "FS-1");
}

/// edgesTo with an invalid `nodeId` → `ValidationError` (FS-3).
#[tokio::test]
async fn edges_to_invalid_node_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "adjacency").await;
    apply_graph(&store, &doc, adjacency_graph(&doc.document_id)).await;
    let err = store
        .edges_to(&doc.document_id, &nid("GHOST"))
        .await
        .unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "FS-3 invalid nodeId"
    );
}

/// edgesByKind with an invalid `nodeId` → `ValidationError` (FS-3).
#[tokio::test]
async fn edges_by_kind_invalid_node_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "adjacency").await;
    apply_graph(&store, &doc, adjacency_graph(&doc.document_id)).await;
    let err = store
        .edges_by_kind(&doc.document_id, &nid("GHOST"), EdgeKind::DocChild)
        .await
        .unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "FS-3 invalid nodeId"
    );
}

/// edgesForDocument on an unknown `documentId` → `DocumentNotFound` (FS-1).
#[tokio::test]
async fn edges_for_document_unknown_document_is_document_not_found() {
    let store = Arc::new(Store::new());
    let err = store.edges_for_document(&did("ghost")).await.unwrap_err();
    assert_eq!(err, StoreError::DocumentNotFound, "FS-1");
}

/// docHeadForDocument on an unknown `documentId` → `DocumentNotFound` (FS-1).
#[tokio::test]
async fn doc_head_unknown_document_is_document_not_found() {
    let store = Arc::new(Store::new());
    let err = store
        .doc_head_for_document(&did("ghost"))
        .await
        .unwrap_err();
    assert_eq!(err, StoreError::DocumentNotFound, "FS-1");
}

/// docHeadForDocument on a document with **no** `doc-head` edge (a freshly
/// created doc has an empty graph) → `InvalidState` (malformed graph).
#[tokio::test]
async fn doc_head_without_doc_head_edge_is_invalid_state() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "empty").await; // created with an empty graph, no doc-head
    let err = store
        .doc_head_for_document(&doc.document_id)
        .await
        .unwrap_err();
    assert_eq!(err, StoreError::InvalidState, "FS-6 no doc-head edge");
}

// ---------------------------------------------------------------------------
// §4.2.6 — Topological `reference`→`fact` resolution
// ---------------------------------------------------------------------------

/// A linear chain resolves in dependency order (target facts before dependents)
/// and each resolved node's value reflects its target's value.
#[tokio::test]
async fn resolve_references_walks_in_dependency_order() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "chain").await;
    apply_graph(&store, &doc, ref_chain_graph(&doc.document_id)).await;

    let opts = ResolveOptions {
        wiki_id: w.clone(),
        max_hops: 2, // r_top→r_mid→f_leaf is a 2-hop chain; pinned 1..=5 (FS-3, §4.5.2)
    };
    let resolved = store
        .resolve_references(&[(doc.document_id.clone(), nid("r_top"))], &opts)
        .await
        .unwrap();
    assert_eq!(resolved.len(), 3, "f_leaf, r_mid, r_top");
    let order = |n: &str| -> u64 {
        resolved
            .iter()
            .find(|rf| rf.node.1 == nid(n))
            .unwrap_or_else(|| panic!("{n} was not resolved"))
            .resolution_order
    };
    assert!(
        order("f_leaf") < order("r_mid"),
        "f_leaf (a dependency) resolves before r_mid"
    );
    assert!(
        order("r_mid") < order("r_top"),
        "r_mid (a dependency) resolves before r_top"
    );
    // Every resolved node resolves to the leaf fact's value.
    assert!(resolved.iter().all(|rf| rf.value == "leaf-val"));
}

/// Each resolved fact is reused across all its dependents — even when two roots
/// share a downstream fact, that fact appears exactly once in the walk.
#[tokio::test]
async fn resolve_references_reuses_facts_across_dependents() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "chain").await;
    apply_graph(&store, &doc, ref_chain_graph(&doc.document_id)).await;

    let opts = ResolveOptions {
        wiki_id: w.clone(),
        max_hops: 2, // longest chain r_top→r_mid→f_leaf is 2 hops (r_side→f_leaf is 1); pinned 1..=5 (FS-3, §4.5.2)
    };
    // Both r_top (via r_mid) and r_side depend on f_leaf.
    let resolved = store
        .resolve_references(
            &[
                (doc.document_id.clone(), nid("r_top")),
                (doc.document_id.clone(), nid("r_side")),
            ],
            &opts,
        )
        .await
        .unwrap();
    assert_eq!(resolved.len(), 4, "f_leaf, r_mid, r_top, r_side");
    let leaf_count = resolved
        .iter()
        .filter(|rf| rf.node == (doc.document_id.clone(), nid("f_leaf")))
        .count();
    assert_eq!(
        leaf_count, 1,
        "shared f_leaf is resolved once, reused across dependents (§4.2.6)"
    );
}

/// A `reference` cycle → `CycleDetected` (FS-12).
#[tokio::test]
async fn resolve_references_cycle_is_cycle_detected() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "cycle").await;
    apply_graph(&store, &doc, ref_cycle_graph(&doc.document_id)).await;

    let opts = ResolveOptions {
        wiki_id: w.clone(),
        max_hops: 2, // cycle r_a→r_b→r_a is closed at hop 2; pinned 1..=5 (FS-3, §4.5.2)
    };
    let err = store
        .resolve_references(&[(doc.document_id.clone(), nid("r_a"))], &opts)
        .await
        .unwrap_err();
    assert_eq!(err, StoreError::CycleDetected, "FS-12 §4.2.6");
}

/// A bounded resolution whose chain exceeds `maxHops` → `HopLimitExceeded`
/// (FS-11).
#[tokio::test]
async fn resolve_references_over_hop_cap_is_hop_limit_exceeded() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "hop").await;
    apply_graph(&store, &doc, ref_hop_overflow_graph(&doc.document_id)).await;

    let opts = ResolveOptions {
        wiki_id: w.clone(),
        max_hops: 2, // the r4…f_leaf chain needs 4 reference hops
    };
    let err = store
        .resolve_references(&[(doc.document_id.clone(), nid("r4"))], &opts)
        .await
        .unwrap_err();
    assert_eq!(err, StoreError::HopLimitExceeded, "FS-11 §4.2.6");
}

// ---------------------------------------------------------------------------
// §4.2.7 — Triples (subject-relation-object)
// ---------------------------------------------------------------------------

fn triple_opts(w: &WikiId) -> GetTriplesFilter {
    GetTriplesFilter {
        direction: TripleDirection::Both,
        relation_type: None,
        wiki_id: w.clone(),
    }
}

fn query_opts(w: &WikiId, limit: Option<u64>) -> QueryTriplesOptions {
    QueryTriplesOptions {
        wiki_id: w.clone(),
        limit,
    }
}

/// A triple is owned by the graph as a `relation` edge: after `addTriple`, the
/// returned `Triple` carries the subject/relation/object, and the document's
/// graph exposes the `Relation` edge (single source of truth, §4.2.7.2).
#[tokio::test]
async fn add_triple_stores_relation_edge_in_document() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "triples").await;
    apply_graph(
        &store,
        &doc,
        content_only_graph(&doc.document_id, &["n1", "n2"]),
    )
    .await;

    let triple = store
        .add_triple(
            &(doc.document_id.clone(), nid("n1")),
            "depends_on",
            &(doc.document_id.clone(), nid("n2")),
            &w,
        )
        .await
        .unwrap();
    assert_eq!(triple.subject, (doc.document_id.clone(), nid("n1")));
    assert_eq!(triple.relation, "depends_on");
    assert_eq!(triple.object, (doc.document_id.clone(), nid("n2")));
    assert_eq!(triple.relation_type, "depends_on");
    assert!(!triple.created_at.is_empty(), "createdAt must be set");

    // It is a `relation` edge owned by the graph (§4.2.7.2), retrievable via
    // adjacency — not an external index.
    let edges = store.edges_for_document(&doc.document_id).await.unwrap();
    let rels: Vec<&Edge> = edges
        .iter()
        .filter(|e| e.kind == EdgeKind::Relation)
        .collect();
    assert_eq!(
        rels.len(),
        1,
        "triple stored as exactly one `relation` edge"
    );
    assert_eq!(rels[0].source, (doc.document_id.clone(), nid("n1")));
    assert_eq!(rels[0].target, (doc.document_id.clone(), nid("n2")));
}

/// `getTriples` with direction `out` returns triples where the node is subject.
#[tokio::test]
async fn get_triples_out_direction_returns_subjected_triples() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "triples").await;
    apply_graph(
        &store,
        &doc,
        content_only_graph(&doc.document_id, &["n1", "n2", "n3"]),
    )
    .await;
    store
        .add_triple(
            &(doc.document_id.clone(), nid("n1")),
            "depends_on",
            &(doc.document_id.clone(), nid("n2")),
            &w,
        )
        .await
        .unwrap();
    store
        .add_triple(
            &(doc.document_id.clone(), nid("n2")),
            "part_of",
            &(doc.document_id.clone(), nid("n3")),
            &w,
        )
        .await
        .unwrap();

    let out = store
        .get_triples(
            &(doc.document_id.clone(), nid("n2")),
            &GetTriplesFilter {
                direction: TripleDirection::Out,
                ..triple_opts(&w)
            },
        )
        .await
        .unwrap();
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].relation, "part_of");
    assert_eq!(out[0].object, (doc.document_id.clone(), nid("n3")));
}

/// `getTriples` with direction `in` returns triples where the node is object.
#[tokio::test]
async fn get_triples_in_direction_returns_objected_triples() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "triples").await;
    apply_graph(
        &store,
        &doc,
        content_only_graph(&doc.document_id, &["n1", "n2", "n3"]),
    )
    .await;
    store
        .add_triple(
            &(doc.document_id.clone(), nid("n1")),
            "depends_on",
            &(doc.document_id.clone(), nid("n2")),
            &w,
        )
        .await
        .unwrap();
    store
        .add_triple(
            &(doc.document_id.clone(), nid("n2")),
            "part_of",
            &(doc.document_id.clone(), nid("n3")),
            &w,
        )
        .await
        .unwrap();

    let incoming = store
        .get_triples(
            &(doc.document_id.clone(), nid("n2")),
            &GetTriplesFilter {
                direction: TripleDirection::In,
                ..triple_opts(&w)
            },
        )
        .await
        .unwrap();
    assert_eq!(incoming.len(), 1);
    assert_eq!(incoming[0].relation, "depends_on");
    assert_eq!(incoming[0].subject, (doc.document_id.clone(), nid("n1")));
}

/// `getTriples` with direction `both` returns all triples touching the node.
#[tokio::test]
async fn get_triples_both_direction_returns_all_touching() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "triples").await;
    apply_graph(
        &store,
        &doc,
        content_only_graph(&doc.document_id, &["n1", "n2", "n3", "n4"]),
    )
    .await;
    for (s, r, o) in [
        ("n1", "depends_on", "n2"),
        ("n2", "part_of", "n3"),
        ("n4", "depends_on", "n2"),
    ] {
        store
            .add_triple(
                &(doc.document_id.clone(), nid(s)),
                r,
                &(doc.document_id.clone(), nid(o)),
                &w,
            )
            .await
            .unwrap();
    }
    let all = store
        .get_triples(&(doc.document_id.clone(), nid("n2")), &triple_opts(&w))
        .await
        .unwrap();
    assert_eq!(all.len(), 3, "n1→n2, n2→n3, n4→n2");
}

/// `getTriples` filtered by `relationType` returns only matching triples.
#[tokio::test]
async fn get_triples_filters_by_relation_type() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "triples").await;
    apply_graph(
        &store,
        &doc,
        content_only_graph(&doc.document_id, &["n1", "n2"]),
    )
    .await;
    store
        .add_triple(
            &(doc.document_id.clone(), nid("n1")),
            "depends_on",
            &(doc.document_id.clone(), nid("n2")),
            &w,
        )
        .await
        .unwrap();
    store
        .add_triple(
            &(doc.document_id.clone(), nid("n1")),
            "causes",
            &(doc.document_id.clone(), nid("n2")),
            &w,
        )
        .await
        .unwrap();

    let filtered = store
        .get_triples(
            &(doc.document_id.clone(), nid("n1")),
            &GetTriplesFilter {
                relation_type: Some("depends_on".to_string()),
                ..triple_opts(&w)
            },
        )
        .await
        .unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].relation, "depends_on");
}

/// `queryTriples` treats any omitted component as a wildcard.
#[tokio::test]
async fn query_triples_relation_wildcard_returns_all_matching() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "triples").await;
    apply_graph(
        &store,
        &doc,
        content_only_graph(&doc.document_id, &["n1", "n2", "n3", "n4"]),
    )
    .await;
    for (s, r, o) in [
        ("n1", "depends_on", "n2"),
        ("n2", "part_of", "n3"),
        ("n4", "depends_on", "n2"),
    ] {
        store
            .add_triple(
                &(doc.document_id.clone(), nid(s)),
                r,
                &(doc.document_id.clone(), nid(o)),
                &w,
            )
            .await
            .unwrap();
    }

    // relation = "depends_on", subject/object wildcards.
    let result = store
        .query_triples(
            &TriplePattern {
                subject: None,
                relation: Some("depends_on".to_string()),
                object: None,
            },
            &query_opts(&w, Some(10)),
        )
        .await
        .unwrap();
    assert_eq!(result.len(), 2, "n1→depends_on→n2 and n4→depends_on→n2");
    assert!(result.iter().all(|t| t.relation == "depends_on"));
}

/// `queryTriples` subject wildcard: subject fixed, relation/object wildcards.
#[tokio::test]
async fn query_triples_subject_wildcard_returns_outgoing() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "triples").await;
    apply_graph(
        &store,
        &doc,
        content_only_graph(&doc.document_id, &["n1", "n2"]),
    )
    .await;
    store
        .add_triple(
            &(doc.document_id.clone(), nid("n1")),
            "depends_on",
            &(doc.document_id.clone(), nid("n2")),
            &w,
        )
        .await
        .unwrap();
    store
        .add_triple(
            &(doc.document_id.clone(), nid("n2")),
            "part_of",
            &(doc.document_id.clone(), nid("n1")),
            &w,
        )
        .await
        .unwrap();

    let result = store
        .query_triples(
            &TriplePattern {
                subject: Some((doc.document_id.clone(), nid("n1"))),
                relation: None,
                object: None,
            },
            &query_opts(&w, None),
        )
        .await
        .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].relation, "depends_on");
}

/// `addTriple` with an empty relation → `ValidationError` (FS-3).
#[tokio::test]
async fn add_triple_empty_relation_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "triples").await;
    apply_graph(
        &store,
        &doc,
        content_only_graph(&doc.document_id, &["n1", "n2"]),
    )
    .await;
    let err = store
        .add_triple(
            &(doc.document_id.clone(), nid("n1")),
            "",
            &(doc.document_id.clone(), nid("n2")),
            &w,
        )
        .await
        .unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "FS-3 empty relation"
    );
}

/// `addTriple` where the subject document is unknown → `DocumentNotFound` (FS-1).
#[tokio::test]
async fn add_triple_unknown_subject_document_is_document_not_found() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "triples").await;
    apply_graph(
        &store,
        &doc,
        content_only_graph(&doc.document_id, &["n1", "n2"]),
    )
    .await;
    let err = store
        .add_triple(
            &(did("ghost"), nid("n1")),
            "depends_on",
            &(doc.document_id.clone(), nid("n2")),
            &w,
        )
        .await
        .unwrap_err();
    assert_eq!(err, StoreError::DocumentNotFound, "FS-1");
}

/// `addTriple` where the subject/object node is not in the document →
/// `ValidationError` (FS-3).
#[tokio::test]
async fn add_triple_invalid_node_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "triples").await;
    apply_graph(
        &store,
        &doc,
        content_only_graph(&doc.document_id, &["n1", "n2"]),
    )
    .await;
    let err = store
        .add_triple(
            &(doc.document_id.clone(), nid("GHOST")),
            "depends_on",
            &(doc.document_id.clone(), nid("n2")),
            &w,
        )
        .await
        .unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "FS-3 invalid node"
    );
}

/// `getTriples` with an invalid node → `ValidationError` (FS-3).
#[tokio::test]
async fn get_triples_invalid_node_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "triples").await;
    apply_graph(&store, &doc, content_only_graph(&doc.document_id, &["n1"])).await;
    let err = store
        .get_triples(&(doc.document_id.clone(), nid("GHOST")), &triple_opts(&w))
        .await
        .unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "FS-3 invalid node"
    );
}

/// `getTriples` with an empty `relationType` → `ValidationError` (FS-3).
#[tokio::test]
async fn get_triples_empty_relation_type_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "triples").await;
    apply_graph(&store, &doc, content_only_graph(&doc.document_id, &["n1"])).await;
    let err = store
        .get_triples(
            &(doc.document_id.clone(), nid("n1")),
            &GetTriplesFilter {
                relation_type: Some(String::new()),
                ..triple_opts(&w)
            },
        )
        .await
        .unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "FS-3 empty relationType"
    );
}

/// `queryTriples` on an unknown wiki → `WikiNotFound` (FS-2).
#[tokio::test]
async fn query_triples_unknown_wiki_is_wiki_not_found() {
    let store = Arc::new(Store::new());
    let err = store
        .query_triples(&TriplePattern::default(), &query_opts(&wiki("ghost"), None))
        .await
        .unwrap_err();
    assert_eq!(err, StoreError::WikiNotFound, "FS-2");
}

/// `queryTriples` with `limit < 1` → `ValidationError` (FS-3).
#[tokio::test]
async fn query_triples_limit_zero_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let err = store
        .query_triples(&TriplePattern::default(), &query_opts(&w, Some(0)))
        .await
        .unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "FS-3 limit<1"
    );
}

/// `queryTriples` with `limit > 100` → `ValidationError` (FS-3).
#[tokio::test]
async fn query_triples_limit_over_one_hundred_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let err = store
        .query_triples(&TriplePattern::default(), &query_opts(&w, Some(101)))
        .await
        .unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "FS-3 limit>100"
    );
}

/// A triple whose subject node is removed from the document is **cascaded away**
/// (§4.2.7.5, consistent with §4.4.5 reference-integrity).
#[tokio::test]
async fn triples_cascade_when_subject_node_is_removed() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "cascade").await;
    apply_graph(
        &store,
        &doc,
        content_only_graph(&doc.document_id, &["n1", "n2"]),
    )
    .await;
    store
        .add_triple(
            &(doc.document_id.clone(), nid("n1")),
            "depends_on",
            &(doc.document_id.clone(), nid("n2")),
            &w,
        )
        .await
        .unwrap();
    let present = store
        .get_triples(&(doc.document_id.clone(), nid("n1")), &triple_opts(&w))
        .await
        .unwrap();
    assert_eq!(present.len(), 1, "triple exists before node removal");

    // Update the graph to drop node n2 (its removal cascades the triple away).
    apply_graph(&store, &doc, content_only_graph(&doc.document_id, &["n1"])).await;

    let after = store
        .get_triples(&(doc.document_id.clone(), nid("n1")), &triple_opts(&w))
        .await
        .unwrap();
    assert!(
        after
            .iter()
            .all(|t| t.object != (doc.document_id.clone(), nid("n2"))),
        "the n1→depends_on→n2 triple is cascaded away with its object node (§4.2.7.5)"
    );
}

/// A triple whose object node is removed cascades the same way.
#[tokio::test]
async fn triples_cascade_when_object_node_is_removed() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "cascade").await;
    apply_graph(
        &store,
        &doc,
        content_only_graph(&doc.document_id, &["n1", "n2"]),
    )
    .await;
    store
        .add_triple(
            &(doc.document_id.clone(), nid("n1")),
            "depends_on",
            &(doc.document_id.clone(), nid("n2")),
            &w,
        )
        .await
        .unwrap();
    store
        .add_triple(
            &(doc.document_id.clone(), nid("n1")),
            "causes",
            &(doc.document_id.clone(), nid("n2")),
            &w,
        )
        .await
        .unwrap();

    // Drop n2 — both triples that use n2 as object must be cascaded away.
    apply_graph(&store, &doc, content_only_graph(&doc.document_id, &["n1"])).await;
    let after = store
        .get_triples(&(doc.document_id.clone(), nid("n1")), &triple_opts(&w))
        .await
        .unwrap();
    assert!(
        after.is_empty(),
        "all triples whose object node was removed are cascaded away (§4.2.7.5)"
    );
}

// ---------------------------------------------------------------------------
// §4.2.8 — Manual override: communities
// ---------------------------------------------------------------------------

fn declare_opts(w: &WikiId, summary: &str) -> DeclareCommunityOptions {
    DeclareCommunityOptions {
        summary: summary.to_string(),
        wiki_id: w.clone(),
    }
}

/// `declareCommunity` stores a `community` node + `member` edges with the
/// provided summary; `getCommunity` retrieves the identical declaration.
#[tokio::test]
async fn declare_community_create_and_get_round_trip() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "comm").await;
    apply_graph(
        &store,
        &doc,
        content_only_graph(&doc.document_id, &["n1", "n2"]),
    )
    .await;

    let members = vec![
        (doc.document_id.clone(), nid("n1")),
        (doc.document_id.clone(), nid("n2")),
    ];
    let declared = store
        .declare_community(&members, &declare_opts(&w, "a community of two"))
        .await
        .unwrap();
    assert!(
        !declared.community_id.0.is_empty(),
        "engine assigns a communityId"
    );
    assert_eq!(declared.members, members);
    assert_eq!(declared.summary, "a community of two");
    assert_eq!(declared.wiki_id, w);

    let fetched = store.get_community(&declared.community_id).await.unwrap();
    assert_eq!(fetched.members, members);
    assert_eq!(fetched.summary, "a community of two");
    assert_eq!(fetched.wiki_id, w);
}

/// `listCommunities` returns every community declared in the wiki.
#[tokio::test]
async fn list_communities_returns_declared() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "comm").await;
    apply_graph(
        &store,
        &doc,
        content_only_graph(&doc.document_id, &["n1", "n2", "n3"]),
    )
    .await;

    let c1 = store
        .declare_community(
            &[(doc.document_id.clone(), nid("n1"))],
            &declare_opts(&w, "one"),
        )
        .await
        .unwrap();
    let c2 = store
        .declare_community(
            &[(doc.document_id.clone(), nid("n2"))],
            &declare_opts(&w, "two"),
        )
        .await
        .unwrap();

    let listed = store.list_communities(&w).await.unwrap();
    let ids: Vec<CommunityId> = listed.iter().map(|c| c.community_id.clone()).collect();
    assert!(ids.contains(&c1.community_id));
    assert!(ids.contains(&c2.community_id));
}

/// `updateCommunitySummary` updates a declared community's summary.
#[tokio::test]
async fn update_community_summary_changes_summary() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "comm").await;
    apply_graph(&store, &doc, content_only_graph(&doc.document_id, &["n1"])).await;

    let declared = store
        .declare_community(
            &[(doc.document_id.clone(), nid("n1"))],
            &declare_opts(&w, "old"),
        )
        .await
        .unwrap();
    let updated = store
        .update_community_summary(&declared.community_id, "new summary")
        .await
        .unwrap();
    assert_eq!(updated.summary, "new summary");
    let fetched = store.get_community(&declared.community_id).await.unwrap();
    assert_eq!(fetched.summary, "new summary");
}

/// Manual override is authoritative: the declared summary is NEVER overwritten
/// by an automatic computation. Since automatic derivation is parked (§4.5.5),
/// the manual value must persist even across a member-changing operation.
#[tokio::test]
async fn community_manual_summary_is_authoritative_and_persists() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "comm").await;
    apply_graph(
        &store,
        &doc,
        content_only_graph(&doc.document_id, &["n1", "n2"]),
    )
    .await;

    let declared = store
        .declare_community(
            &[(doc.document_id.clone(), nid("n1"))],
            &declare_opts(&w, "manual summary"),
        )
        .await
        .unwrap();

    // A member-touching change (a triple whose subject is a member). Automatic
    // detection is parked, so the manual declaration + summary are authoritative
    // and persist (§4.2.8.3/§4.2.8.4).
    store
        .add_triple(
            &(doc.document_id.clone(), nid("n1")),
            "depends_on",
            &(doc.document_id.clone(), nid("n2")),
            &w,
        )
        .await
        .unwrap();

    let after = store.get_community(&declared.community_id).await.unwrap();
    assert_eq!(
        after.summary, "manual summary",
        "manual declaration is authoritative and never overwritten (§4.2.8.3/§4.2.8.4)"
    );
}

/// `declareCommunity` with an empty node set → `ValidationError` (FS-3).
#[tokio::test]
async fn declare_community_empty_members_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let err = store
        .declare_community(&[], &declare_opts(&w, "summary"))
        .await
        .unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "FS-3 empty node set"
    );
}

/// `declareCommunity` with an empty summary → `ValidationError` (FS-3).
#[tokio::test]
async fn declare_community_empty_summary_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let err = store
        .declare_community(&[(did("d"), nid("n1"))], &declare_opts(&w, ""))
        .await
        .unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "FS-3 empty summary"
    );
}

/// `getCommunity` on an unknown `communityId` → `CommunityNotFound` (FS-25).
#[tokio::test]
async fn get_community_unknown_is_community_not_found() {
    let store = Arc::new(Store::new());
    let err = store
        .get_community(&CommunityId("ghost".into()))
        .await
        .unwrap_err();
    assert_eq!(err, StoreError::CommunityNotFound, "FS-25 §4.2.8.5");
}

/// `updateCommunitySummary` on an unknown `communityId` → `CommunityNotFound`
/// (FS-25).
#[tokio::test]
async fn update_community_summary_unknown_is_community_not_found() {
    let store = Arc::new(Store::new());
    let err = store
        .update_community_summary(&CommunityId("ghost".into()), "x")
        .await
        .unwrap_err();
    assert_eq!(err, StoreError::CommunityNotFound, "FS-25 §4.2.8.5");
}

/// `listCommunities` on an unknown wiki → `WikiNotFound` (FS-2), consistent with
/// the store-wide wiki-scoping contract.
#[tokio::test]
async fn list_communities_unknown_wiki_is_wiki_not_found() {
    let store = Arc::new(Store::new());
    let err = store.list_communities(&wiki("ghost")).await.unwrap_err();
    assert_eq!(err, StoreError::WikiNotFound, "FS-2");
}

// ---------------------------------------------------------------------------
// §4.2.8.5 / §4.4.2 — setReferenceState
// ---------------------------------------------------------------------------

/// A graph with a `link` reference edge `ref1 → (other, tgt)`.
fn reference_state_graph(doc: &DocumentId) -> Graph {
    let n1 = content_node(doc, "n1", "hello");
    let ref1 = reference_node(doc, "ref1", (did("other"), nid("tgt")));
    Graph {
        nodes: vec![n1.clone(), ref1.clone()],
        edges: vec![
            edge(
                EdgeKind::DocHead,
                (doc.clone(), nid("ROOT")),
                (doc.clone(), n1.node_id.clone()),
                None,
            ),
            edge(
                EdgeKind::DocEnd,
                (doc.clone(), n1.node_id.clone()),
                (doc.clone(), nid("END")),
                None,
            ),
            edge(
                EdgeKind::Link,
                (doc.clone(), ref1.node_id.clone()),
                (did("other"), nid("tgt")),
                Some(ReferenceState::Resolved),
            ),
        ],
    }
}

/// `setReferenceState` marks the reference edge `FRESH`/`STALE`/`RESOLVED`/
/// `BROKEN` and the resulting document graph reflects it.
#[tokio::test]
async fn set_reference_state_marks_each_valid_state() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "ref-state").await;
    apply_graph(&store, &doc, reference_state_graph(&doc.document_id)).await;

    for (state_str, expected) in [
        ("FRESH", ReferenceState::Fresh),
        ("STALE", ReferenceState::Stale),
        ("RESOLVED", ReferenceState::Resolved),
        ("BROKEN", ReferenceState::Broken),
    ] {
        let edge = store
            .set_reference_state(&doc.document_id, &nid("ref1"), &nid("tgt"), state_str)
            .await
            .unwrap();
        assert_eq!(
            edge.state,
            Some(expected),
            "{state_str} must be applied to the reference edge"
        );
        let persisted = store
            .edges_by_kind(&doc.document_id, &nid("ref1"), EdgeKind::Link)
            .await
            .unwrap();
        assert_eq!(
            persisted[0].state,
            Some(expected),
            "the reference edge state persists in the document's graph"
        );
    }
}

/// `setReferenceState` with an invalid state string → `ValidationError` (FS-3).
#[tokio::test]
async fn set_reference_state_invalid_state_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "ref-state").await;
    apply_graph(&store, &doc, reference_state_graph(&doc.document_id)).await;
    let err = store
        .set_reference_state(&doc.document_id, &nid("ref1"), &nid("tgt"), "INVALID")
        .await
        .unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "FS-3 invalid state"
    );
}

// ---------------------------------------------------------------------------
// §4.2.9 — Entity resolution / fact merging
// ---------------------------------------------------------------------------

fn entity_graph(doc: &DocumentId) -> Graph {
    let e1 = content_node(doc, "e1", "Astrographer");
    let e2 = content_node(doc, "e2", "Astrographer");
    let e3 = content_node(doc, "e3", "Gnosis");
    Graph {
        nodes: vec![e1.clone(), e2.clone(), e3.clone()],
        edges: vec![
            edge(
                EdgeKind::DocHead,
                (doc.clone(), nid("ROOT")),
                (doc.clone(), e1.node_id.clone()),
                None,
            ),
            edge(
                EdgeKind::DocEnd,
                (doc.clone(), e1.node_id.clone()),
                (doc.clone(), nid("END")),
                None,
            ),
        ],
    }
}

/// `resolveEntities` makes the provided `canonicalId` the canonical entity and
/// turns every other entity into an alias / merge target pointing at it.
#[tokio::test]
async fn resolve_entities_merges_and_aliases_to_canonical() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "entities").await;
    apply_graph(&store, &doc, entity_graph(&doc.document_id)).await;

    let ids = vec![
        (doc.document_id.clone(), nid("e1")),
        (doc.document_id.clone(), nid("e2")),
        (doc.document_id.clone(), nid("e3")),
    ];
    let canonical = (doc.document_id.clone(), nid("e1"));
    let result = store
        .resolve_entities(
            &ids,
            &ResolveEntitiesOptions {
                canonical_id: Some(canonical.clone()),
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap();

    assert_eq!(result.canonical_id, canonical);
    // e2 and e3 were merged into e1.
    let merged_froms: Vec<_> = result.merged.iter().map(|p| p.from.clone()).collect();
    assert_eq!(merged_froms.len(), 2);
    assert!(merged_froms.contains(&(doc.document_id.clone(), nid("e2"))));
    assert!(merged_froms.contains(&(doc.document_id.clone(), nid("e3"))));
    assert!(result.merged.iter().all(|p| p.to == canonical));
    // e2 and e3 are aliases pointing at e1.
    let alias_nodes: Vec<_> = result.aliases.iter().map(|a| a.alias.clone()).collect();
    assert_eq!(alias_nodes.len(), 2);
    assert!(alias_nodes.contains(&(doc.document_id.clone(), nid("e2"))));
    assert!(alias_nodes.contains(&(doc.document_id.clone(), nid("e3"))));
    assert!(result.aliases.iter().all(|a| a.canonical == canonical));
}

/// Without a canonical id, the engine picks one canonical from the set and
/// aliases the rest to it.
#[tokio::test]
async fn resolve_entities_without_canonical_picks_one() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "entities").await;
    apply_graph(&store, &doc, entity_graph(&doc.document_id)).await;

    let ids = vec![
        (doc.document_id.clone(), nid("e1")),
        (doc.document_id.clone(), nid("e2")),
    ];
    let result = store
        .resolve_entities(
            &ids,
            &ResolveEntitiesOptions {
                canonical_id: None,
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap();
    assert!(
        ids.contains(&result.canonical_id),
        "the chosen canonical is one of the input entities"
    );
    assert_eq!(result.aliases.len(), 1);
    assert_eq!(result.aliases[0].canonical, result.canonical_id);
}

/// `resolveEntities` with an empty entity set → `ValidationError` (FS-3).
#[tokio::test]
async fn resolve_entities_empty_entities_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let err = store
        .resolve_entities(
            &[],
            &ResolveEntitiesOptions {
                canonical_id: None,
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "FS-3 empty entityIds"
    );
}

/// `resolveEntities` with a `canonicalId` that is not among the entity ids →
/// `ValidationError` (FS-3).
#[tokio::test]
async fn resolve_entities_invalid_canonical_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "entities").await;
    apply_graph(&store, &doc, entity_graph(&doc.document_id)).await;
    let err = store
        .resolve_entities(
            &[(doc.document_id.clone(), nid("e1"))],
            &ResolveEntitiesOptions {
                canonical_id: Some((doc.document_id.clone(), nid("e2"))),
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "FS-3 canonicalId not among entityIds"
    );
}

/// `resolveEntities` where an entity's document is unknown → `DocumentNotFound`
/// (FS-1).
#[tokio::test]
async fn resolve_entities_unknown_document_is_document_not_found() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let err = store
        .resolve_entities(
            &[(did("ghost"), nid("e1"))],
            &ResolveEntitiesOptions {
                canonical_id: None,
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap_err();
    assert_eq!(err, StoreError::DocumentNotFound, "FS-1");
}

/// `mergeFacts` merges duplicate fact keys: the canonical fact keeps the
/// canonical value and the **union** of the merged facts' citations (§4.2.9.1,
/// §4.3.2).
#[tokio::test]
async fn merge_facts_unions_citations_and_keeps_canonical_value() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "facts").await;
    apply_graph(
        &store,
        &doc,
        content_only_graph(&doc.document_id, &["n1", "n2", "n3"]),
    )
    .await;

    // Two spellings of the same fact ("licence" vs "license"), both "MIT".
    store
        .create_fact(
            &w,
            &doc.document_id,
            "licence",
            "MIT",
            &[(doc.document_id.clone(), nid("n1"))],
        )
        .await
        .unwrap();
    store
        .create_fact(
            &w,
            &doc.document_id,
            "license",
            "MIT",
            &[
                (doc.document_id.clone(), nid("n2")),
                (doc.document_id.clone(), nid("n3")),
            ],
        )
        .await
        .unwrap();

    let merged = store
        .merge_facts(
            &["licence".to_string(), "license".to_string()],
            &MergeFactsOptions {
                canonical_key: "license".to_string(),
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap();
    assert_eq!(merged.fact_key, "license");
    assert_eq!(merged.value, "MIT", "canonical value preserved");
    assert_eq!(merged.citations.len(), 3, "union of citations");
    for c in [
        (doc.document_id.clone(), nid("n1")),
        (doc.document_id.clone(), nid("n2")),
        (doc.document_id.clone(), nid("n3")),
    ] {
        assert!(
            merged.citations.contains(&c),
            "merged citation {c:?} present"
        );
    }
}

/// `mergeFacts` with conflicting values and no resolution → `ConflictError`
/// (FS-4).
#[tokio::test]
async fn merge_facts_conflicting_values_is_conflict_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "facts").await;
    apply_graph(&store, &doc, content_only_graph(&doc.document_id, &["n1"])).await;

    store
        .create_fact(
            &w,
            &doc.document_id,
            "a",
            "MIT",
            &[(doc.document_id.clone(), nid("n1"))],
        )
        .await
        .unwrap();
    store
        .create_fact(
            &w,
            &doc.document_id,
            "b",
            "Apache",
            &[(doc.document_id.clone(), nid("n1"))],
        )
        .await
        .unwrap();

    let err = store
        .merge_facts(
            &["a".to_string(), "b".to_string()],
            &MergeFactsOptions {
                canonical_key: "a".to_string(),
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap_err();
    assert_eq!(
        err,
        StoreError::ConflictError,
        "FS-4 conflicting values, no resolution"
    );
}

/// `mergeFacts` with an empty `factKeys` set → `ValidationError` (FS-3).
#[tokio::test]
async fn merge_facts_empty_keys_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let err = store
        .merge_facts(
            &[],
            &MergeFactsOptions {
                canonical_key: "x".to_string(),
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "FS-3 empty factKeys"
    );
}

/// `mergeFacts` with a canonical key that is not among the merged facts →
/// `ValidationError` (FS-3).
#[tokio::test]
async fn merge_facts_invalid_canonical_key_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "facts").await;
    apply_graph(&store, &doc, content_only_graph(&doc.document_id, &["n1"])).await;
    store
        .create_fact(
            &w,
            &doc.document_id,
            "a",
            "v",
            &[(doc.document_id.clone(), nid("n1"))],
        )
        .await
        .unwrap();
    let err = store
        .merge_facts(
            &["a".to_string()],
            &MergeFactsOptions {
                canonical_key: "not-a-key".to_string(),
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "FS-3 invalid canonicalKey"
    );
}

// ---------------------------------------------------------------------------
// Concurrency contract (SHARDED-RWLOCK-STORE / ARC-SHARED-ENGINE)
// ---------------------------------------------------------------------------

/// Concurrent graph readers: many tokio tasks read the adjacency of a shared
/// `Arc<Store>` simultaneously and all see the same graph. Runs on a
/// **multi-thread** runtime and every reader waits on a shared `Barrier` so the
/// reads genuinely overlap across worker threads (§4.2.5 adjacency surfaces are
/// `Arc<Store>`-reentrant like the §4.1 store).
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_adjacency_readers_see_same_graph() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "adjacency").await;
    apply_graph(&store, &doc, adjacency_graph(&doc.document_id)).await;
    let id = doc.document_id.clone();

    const N: usize = 8;
    let barrier = Arc::new(tokio::sync::Barrier::new(N));
    let handles: Vec<_> = (0..N)
        .map(|_| {
            let s = Arc::clone(&store);
            let id = id.clone();
            let b = Arc::clone(&barrier);
            tokio::spawn(async move {
                b.wait().await;
                let all = s
                    .edges_for_document(&id)
                    .await
                    .expect("reader saw the graph");
                let head = s
                    .doc_head_for_document(&id)
                    .await
                    .expect("reader saw the head");
                (all.len(), head.node_id)
            })
        })
        .collect();

    for h in handles {
        let (len, node) = h.await.expect("graph-reader task did not panic");
        assert_eq!(len, 4, "all concurrent readers see the same edge count");
        assert_eq!(
            node,
            nid("n1"),
            "all concurrent readers see the same head node"
        );
    }
}

// ---------------------------------------------------------------------------
// GRAPH-OWNS-RELATION-AND-MERGE — adversarial regression suite
//
// These RED-first tests pin the defects the adversarial review of §4.2 surfaced
// (docs/decisions.md GRAPH-OWNS-RELATION-AND-MERGE). Each is RED against today's
// implementation and is flipped green by a specific Implementer fix (noted in
// the doc comment). None of the §4.1 suite semantics changed; the only
// `Edge`-literal change here is the new frozen `relation_type` field.
// ---------------------------------------------------------------------------

/// CRITICAL #4 — the graph owns the triple's relationship: after `addTriple`,
/// `edgesForDocument` on the subject's document returns the `Relation` edge
/// whose `relation_type` equals the added relation string (not `None`). Today
/// the frozen `Edge` had no relation payload, so `relation_type` is `None`.
/// Implementer fix: `addTriple`'s `Relation` edge carries `Some(relation)`.
#[tokio::test]
async fn triple_relation_type_is_recoverable_from_graph_edge() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "triples").await;
    apply_graph(
        &store,
        &doc,
        content_only_graph(&doc.document_id, &["n1", "n2"]),
    )
    .await;

    store
        .add_triple(
            &(doc.document_id.clone(), nid("n1")),
            "depends_on",
            &(doc.document_id.clone(), nid("n2")),
            &w,
        )
        .await
        .unwrap();

    let edges = store.edges_for_document(&doc.document_id).await.unwrap();
    let rel = edges
        .iter()
        .find(|e| e.kind == EdgeKind::Relation)
        .unwrap_or_else(|| panic!("expected a Relation edge after addTriple"));
    assert_eq!(rel.source, (doc.document_id.clone(), nid("n1")));
    assert_eq!(rel.target, (doc.document_id.clone(), nid("n2")));
    assert_eq!(
        rel.relation_type.as_deref(),
        Some("depends_on"),
        "CRITICAL #4: the graph edge must carry the triple's relationType, not None"
    );
}

/// CRITICAL #3 — real cascade, no duplicate records: removing the subject node
/// via `updateDocument` prunes the `Relation` edge, `getTriples` for those
/// endpoints is empty, and re-adding the same triple yields exactly one result.
/// Today `triple_store` keeps a zombie record that resurfaces once the node is
/// restored, and a re-add duplicates it. Implementer fix: node deletion prunes
/// the relation edges (true cascade) so no zombie record can resurface.
#[tokio::test]
async fn triple_cascade_removes_relation_edge_and_does_not_duplicate_on_readd() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "triples").await;
    apply_graph(
        &store,
        &doc,
        content_only_graph(&doc.document_id, &["n1", "n2"]),
    )
    .await;

    store
        .add_triple(
            &(doc.document_id.clone(), nid("n1")),
            "depends_on",
            &(doc.document_id.clone(), nid("n2")),
            &w,
        )
        .await
        .unwrap();

    // Remove the subject node by replacing the graph without it.
    apply_graph(&store, &doc, content_only_graph(&doc.document_id, &["n2"])).await;

    let edges = store.edges_for_document(&doc.document_id).await.unwrap();
    assert!(
        !edges.iter().any(|e| e.kind == EdgeKind::Relation),
        "CRITICAL #3: removing the subject node must prune its Relation edge"
    );
    let gone = store
        .get_triples(&(doc.document_id.clone(), nid("n1")), &triple_opts(&w))
        .await
        .unwrap();
    assert!(
        gone.is_empty(),
        "no triple may survive removal of its endpoints"
    );

    // Restore the subject node and re-add the same triple.
    apply_graph(
        &store,
        &doc,
        content_only_graph(&doc.document_id, &["n1", "n2"]),
    )
    .await;
    store
        .add_triple(
            &(doc.document_id.clone(), nid("n1")),
            "depends_on",
            &(doc.document_id.clone(), nid("n2")),
            &w,
        )
        .await
        .unwrap();

    let triples = store
        .get_triples(&(doc.document_id.clone(), nid("n1")), &triple_opts(&w))
        .await
        .unwrap();
    assert_eq!(
        triples.len(),
        1,
        "CRITICAL #3: re-adding the same triple must yield exactly one result (no duplicate)"
    );
}

/// CRITICAL #1 — `mergeFacts` PERSISTS the merged fact: the canonical fact's
/// `citations` is the union and its `updatedAt` reflects the merge — not just the
/// returned descriptor. Verified by reading it back from the canonical fact store
/// via the read-only `getFact` accessor the TestWriter added for this regression.
/// Today `merge_facts` computes a merged `Fact` and returns it without writing
/// anything; `getFact` is an `unimplemented!()` stub (RED).
/// Implementer fix: `mergeFacts` writes the merged fact (union citations +
/// refreshed `updatedAt`) into the fact store, and the Implementer fills
/// `getFact` (a plain read of the fact store).
#[tokio::test]
async fn merge_facts_persists_merged_fact_to_store() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "facts").await;
    apply_graph(
        &store,
        &doc,
        content_only_graph(&doc.document_id, &["n1", "n2", "n3"]),
    )
    .await;

    store
        .create_fact(
            &w,
            &doc.document_id,
            "licence",
            "MIT",
            &[(doc.document_id.clone(), nid("n1"))],
        )
        .await
        .unwrap();
    store
        .create_fact(
            &w,
            &doc.document_id,
            "license",
            "MIT",
            &[
                (doc.document_id.clone(), nid("n2")),
                (doc.document_id.clone(), nid("n3")),
            ],
        )
        .await
        .unwrap();

    let merged = store
        .merge_facts(
            &["licence".to_string(), "license".to_string()],
            &MergeFactsOptions {
                canonical_key: "license".to_string(),
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap();
    assert_eq!(merged.citations.len(), 3);

    // The merge must be durable: read the canonical fact back from the store.
    let persisted = store.get_fact(&w, "license").await.expect(
        "CRITICAL #1: the merged fact must be retrievable after merge (getFact is stubbed RED)",
    );
    assert_eq!(persisted.fact_key, "license");
    assert_eq!(persisted.value, "MIT");
    assert_eq!(
        persisted.citations.len(),
        3,
        "union citations must be persisted"
    );
    for c in [
        (doc.document_id.clone(), nid("n1")),
        (doc.document_id.clone(), nid("n2")),
        (doc.document_id.clone(), nid("n3")),
    ] {
        assert!(
            persisted.citations.contains(&c),
            "citation {c:?} must be present in the persisted merge"
        );
    }
    assert_eq!(
        persisted.updated_at, merged.updated_at,
        "CRITICAL #1: the persisted updatedAt must reflect the merge"
    );
}

/// CRITICAL #2 — `resolveEntities` writes a durable alias→canonical mapping. A
/// later read of that mapping returns `alias → canonical`. Today it returns a
/// descriptor and persists nothing; the read-only `entityAliasCanonical`
/// accessor the TestWriter added is an `unimplemented!()` stub (RED).
/// Implementer fix: `resolveEntities` records the alias→canonical mapping
/// (journaled); the Implementer fills `entityAliasCanonical` as a read of it.
#[tokio::test]
async fn resolve_entities_records_durable_alias_to_canonical_mapping() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "entities").await;
    apply_graph(&store, &doc, entity_graph(&doc.document_id)).await;

    let ids = vec![
        (doc.document_id.clone(), nid("e1")),
        (doc.document_id.clone(), nid("e2")),
    ];
    let canonical = (doc.document_id.clone(), nid("e1"));
    store
        .resolve_entities(
            &ids,
            &ResolveEntitiesOptions {
                canonical_id: Some(canonical.clone()),
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap();

    let resolved = store
        .entity_alias_canonical(&(doc.document_id.clone(), nid("e2")))
        .await
        .expect("CRITICAL #2: resolveEntities must persist an alias→canonical mapping (accessor stubbed RED)");
    assert_eq!(
        resolved,
        Some(canonical),
        "CRITICAL #2: e2 must resolve durably to canonical e1"
    );
}

/// HIGH #5 — `setReferenceState` mutates ONLY the matching reference edge's
/// state and preserves the document's remaining edges, nodes and revision
/// history (no whole-document clobber). Deterministic pin; today's whole-graph
/// write happens to preserve content, so this may be green today — it is the
/// regression guard that the optimistic-concurrency fix keeps non-destructive.
#[tokio::test]
async fn set_reference_state_preserves_other_edges_and_nodes() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "ref-state").await;
    apply_graph(&store, &doc, reference_state_graph(&doc.document_id)).await;
    let before = store.get_document(&doc.document_id).await.unwrap();
    let before_rev = before.revision;

    let updated = store
        .set_reference_state(&doc.document_id, &nid("ref1"), &nid("tgt"), "BROKEN")
        .await
        .unwrap();
    assert_eq!(updated.state, Some(ReferenceState::Broken));

    let after = store.get_document(&doc.document_id).await.unwrap();
    // All nodes are preserved.
    let mut node_ids: Vec<NodeId> = after
        .graph
        .nodes
        .iter()
        .map(|n| n.node_id.clone())
        .collect();
    node_ids.sort_by_key(|n| n.0.clone());
    assert_eq!(
        node_ids,
        vec![nid("n1"), nid("ref1")],
        "all nodes preserved by setReferenceState"
    );
    // Edge kinds/count are preserved; only the reference edge's state changed.
    let mut kinds: Vec<EdgeKind> = after.graph.edges.iter().map(|e| e.kind).collect();
    let mut expected: Vec<EdgeKind> = vec![EdgeKind::DocHead, EdgeKind::DocEnd, EdgeKind::Link];
    kinds.sort_by_key(|k| *k as u8);
    expected.sort_by_key(|k| *k as u8);
    assert_eq!(
        kinds, expected,
        "all edge kinds preserved by setReferenceState"
    );
    let link = after
        .graph
        .edges
        .iter()
        .find(|e| e.kind == EdgeKind::Link)
        .expect("link edge still present");
    assert_eq!(link.state, Some(ReferenceState::Broken));
    for e in after
        .graph
        .edges
        .iter()
        .filter(|e| e.kind != EdgeKind::Link)
    {
        assert_eq!(
            e.state, None,
            "HIGH #5: non-reference edge state must be untouched"
        );
    }
    assert_eq!(
        before_rev + 1,
        after.revision,
        "revision history advances by exactly one"
    );
}

/// HIGH #8 / §4.2.6 / FS-3 — `maxHops` is bounded to 1..=5: `0` is rejected
/// up-front with `ValidationError`. RED today: hops are unbounded (a `0` cap
/// simply resolves a leaf root without error).
/// Implementer fix: `resolve_references` validates `max_hops` ∈ 1..=5 first.
#[tokio::test]
async fn resolve_references_zero_max_hops_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "chain").await;
    apply_graph(&store, &doc, ref_chain_graph(&doc.document_id)).await;
    let opts = ResolveOptions {
        wiki_id: w.clone(),
        max_hops: 0,
    };
    let err = store
        .resolve_references(&[(doc.document_id.clone(), nid("f_leaf"))], &opts)
        .await
        .unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "FS-3: maxHops must be >= 1 (RED: unbounded today)"
    );
}

/// HIGH #8 / §4.2.6 / FS-3 — `maxHops: 6` is rejected (`spec pins 1..=5`). RED
/// today: a 6-hop cap on a short chain resolves without error.
/// Implementer fix: `resolve_references` validates `max_hops` ∈ 1..=5 first.
#[tokio::test]
async fn resolve_references_over_five_max_hops_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "chain").await;
    apply_graph(&store, &doc, ref_chain_graph(&doc.document_id)).await;
    let opts = ResolveOptions {
        wiki_id: w.clone(),
        max_hops: 6,
    };
    let err = store
        .resolve_references(&[(doc.document_id.clone(), nid("r_top"))], &opts)
        .await
        .unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "FS-3: maxHops must be at most 5 (RED: unbounded today)"
    );
}

/// HIGH #8 / FS-2 — an unknown `wikiId` → `WikiNotFound`. RED today: the wiki is
/// ignored and a resolvable root resolves successfully. Implementer fix:
/// `resolve_references` scopes/validates the wiki like the other §4.2 ops.
#[tokio::test]
async fn resolve_references_unknown_wiki_is_wiki_not_found() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "chain").await;
    apply_graph(&store, &doc, ref_chain_graph(&doc.document_id)).await;
    let opts = ResolveOptions {
        wiki_id: wiki("ghost"),
        max_hops: 2,
    };
    let err = store
        .resolve_references(&[(doc.document_id.clone(), nid("r_top"))], &opts)
        .await
        .unwrap_err();
    assert_eq!(err, StoreError::WikiNotFound, "FS-2: unknown wiki");
}

/// MEDIUM #10 / FS-2 — `addTriple` on an unknown wiki → `WikiNotFound`. RED
/// today: `add_triple` ignores the wiki entirely. Implementer fix: validate the
/// wiki up-front.
#[tokio::test]
async fn add_triple_unknown_wiki_is_wiki_not_found() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "triples").await;
    apply_graph(
        &store,
        &doc,
        content_only_graph(&doc.document_id, &["n1", "n2"]),
    )
    .await;
    let err = store
        .add_triple(
            &(doc.document_id.clone(), nid("n1")),
            "depends_on",
            &(doc.document_id.clone(), nid("n2")),
            &wiki("ghost"),
        )
        .await
        .unwrap_err();
    assert_eq!(
        err,
        StoreError::WikiNotFound,
        "FS-2: addTriple on an unknown wiki"
    );
}

/// MEDIUM #10 / FS-2 — `getTriples` on an unknown wiki → `WikiNotFound` (not an
/// empty slice). RED today: `get_triples` filters to the unknown wiki and
/// returns `Ok([])`. Implementer fix: validate the wiki up-front.
#[tokio::test]
async fn get_triples_unknown_wiki_is_wiki_not_found() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "triples").await;
    apply_graph(&store, &doc, content_only_graph(&doc.document_id, &["n1"])).await;
    let options = GetTriplesFilter {
        direction: TripleDirection::Both,
        relation_type: None,
        wiki_id: wiki("ghost"),
    };
    let err = store
        .get_triples(&(doc.document_id.clone(), nid("n1")), &options)
        .await
        .unwrap_err();
    assert_eq!(
        err,
        StoreError::WikiNotFound,
        "FS-2: getTriples on an unknown wiki (today returns an empty slice)"
    );
}

/// MEDIUM synthesis / HIGH #5 — two tasks concurrently mutate the SAME document:
/// one `setReferenceState`, one `updateDocument`, launched together behind a
/// `Barrier(2)` on a multi-threaded runtime. Task B captures its base revision
/// **before** the barrier, then (like a slightly-later real-world commit) applies
/// its structural update after task A's state write has landed — so task B is
/// carrying a base made stale by a concurrent edit. Neither task may panic nor
/// lose the other's edit. Today this is the deterministically **lost** interleaving:
/// `setReferenceState`'s whole-document write advances the revision, so the
/// concurrent `updateDocument` with its pre-captured stale base is rejected with
/// `ConflictError` and its `n3` edit never lands. Implementer fix:
/// `setReferenceState` mutates only the matched edge's state under
/// `reference_lock` with an optimistic-concurrency base compare, so both
/// concurrent edits land without data loss.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_set_reference_state_and_update_document_do_not_lose_updates() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "concurrent").await;
    apply_graph(&store, &doc, reference_state_graph(&doc.document_id)).await;
    let id = doc.document_id.clone();

    // Task B builds the updated graph: the current graph plus a content node n3
    // and its edge — a real structural change that must survive A's state change.
    let new_graph = {
        let cur = store.get_document(&id).await.unwrap();
        let mut g = cur.graph.clone();
        g.nodes.push(content_node(&id, "n3", "three"));
        g.edges.push(edge(
            EdgeKind::DocChild,
            (id.clone(), nid("n1")),
            (id.clone(), nid("n3")),
            None,
        ));
        g
    };
    // The base revision task B applies from — captured BEFORE the concurrent
    // state write, so it is stale by the time B lands (a later real-world commit).
    let stale_base = store.get_document(&id).await.unwrap().revision;

    let barrier = Arc::new(tokio::sync::Barrier::new(2));
    let b1 = Arc::clone(&barrier);
    let b2 = Arc::clone(&barrier);

    let s1 = Arc::clone(&store);
    let id1 = id.clone();
    let t1 = tokio::spawn(async move {
        b1.wait().await;
        s1.set_reference_state(&id1, &nid("ref1"), &nid("tgt"), "FRESH")
            .await
    });

    let s2 = Arc::clone(&store);
    let id2 = id.clone();
    let graph2 = new_graph;
    let base2 = stale_base;
    let t2 = tokio::spawn(async move {
        b2.wait().await;
        // A short pause makes the "concurrent edit that lands slightly later"
        // interleaving deterministic: task A's state write commits first, so B's
        // pre-captured base is stale and today B's edit is lost.
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        s2.update_document(
            &id2,
            UpdateDocumentRequest {
                base_revision: base2,
                graph: graph2,
                title: None,
                tags: None,
            },
        )
        .await
    });

    let r1 = t1
        .await
        .expect("concurrent setReferenceState task did not panic");
    let r2 = t2
        .await
        .expect("concurrent updateDocument task did not panic");
    r1.expect("HIGH #5: concurrent setReferenceState must not fail");
    r2.expect("HIGH #5: concurrent updateDocument must not fail or lose its edit");

    // Both concurrent edits must survive in the final document.
    let final_doc = store.get_document(&id).await.unwrap();
    assert!(
        final_doc.graph.nodes.iter().any(|n| n.node_id == nid("n3")),
        "the structural update (n3) must survive the concurrent setReferenceState"
    );
    let ref_edge = final_doc
        .graph
        .edges
        .iter()
        .find(|e| e.kind == EdgeKind::Link)
        .expect("link edge still present");
    assert_eq!(
        ref_edge.state,
        Some(ReferenceState::Fresh),
        "the reference-state change must survive the concurrent updateDocument"
    );
}
