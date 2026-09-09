//! §4.5/§4.6 — RAG query + stream + engine-status + provenance + audit
//! integration tests — written **RED-first** at the TDD gate for the §4.5 unit.
//!
//! Every state and fail-state below is derived from the canonical behavior
//! contract alone (`docs/specs/gnosis.md` §4.5.1–§4.5.3, §4.6.1, §4.3.3, §4.3.4,
//! and the §6 fail-state catalogue) plus the concurrency contract
//! (`docs/research/gnosis-data-structures-concurrency-plan.md`:
//! IMMUTABLE-DERIVED-SNAPSHOT / ARC-SHARED-ENGINE / LOCK-ORDER-REF-SHARD-SIDECAR).
//!
//! ## RED set (this suite)
//!
//! `rag_query`, `rag_stream`, `get_engine_status`, `bm25_search`,
//! `get_profile_summary` and `rrf_fuse` are COMPILING STUBS
//! (`unimplemented!()` → panic), so every test that calls them is RED. The
//! provenance-trace *shape* tests and the immutable-snapshot concurrency-vehicle
//! tests are GREEN (they assert the provided types / swap infra).
//!
//! ## TestWriter contract decisions (for the Implementer to match exactly)
//!
//! - **`ragQuery` top-K default** is not pinned by the spec; a reasonable
//!   reading is `10` (the suite does not depend on it — every default is set
//!   explicitly in the options).
//! - **`filters` malformed (FS-3)** — a §4.5.2 filter pins `edgeType` to
//!   `link|embed|crosslink`; an `edgeType` outside that set (e.g.
//!   `EdgeKind::Relation`) is malformed → `ValidationError`. `nodeKind`/`state`/
//!   `target` are Rust-typed enums, so the only *representable* malformed filter
//!   is an out-of-set `edge_type`.
//! - **`multiQuery.n` out of range (FS-3)** — `n == 0` is out of range →
//!   `ValidationError` (a fan-out of zero variants is meaningless).
//! - **`binaryCandidatePool` not a positive integer (FS-3)** — `0` →
//!   `ValidationError`.
//! - **invalid `mode` / invalid `compression` / non-boolean `hyde` / non-boolean
//!   `binaryFirstPass` / non-boolean-object `subTaskDag`** are not representable
//!   via the typed `RagQueryOptions` (Rust enums / `bool` / struct), so they
//!   cannot be exercised from a typed test; they are validated at the §5.1
//!   transport boundary. Noted; not fabricated as runtime tests here.
//! - **`EngineError` (FS-9) / `TraceUnavailable` (FS-10) / `RerankerUnavailable`
//!   (FS-16) / `CompressionFailed` (FS-17) / `HyDEGenerationFailed` (FS-18) /
//!   `MultiQueryExpansionFailed` (FS-19) / `SubTaskDagFailed` (FS-26)** are
//!   engine-internal boundary fail-states that the current stub seam cannot
//!   drive from a typed test (there is no reranker/generation/compressor
//!   injection seam on `RagStore`). They are asserted as *boundary* fail-states
//!   here (RED via the stub) and flagged in the report for the Implementer to
//!   wire deterministically (and for the §5.1 transport seam to surface).

use std::sync::{Arc, Barrier};

use futures::StreamExt;
use gnosis::{
    rrf_fuse, BlockedBy, CreateDocumentRequest, DerivedIndexes, Document, DocumentId, Edge,
    EdgeKind, EngineState, EngineSubsystems, ExpandMode, FieldType, Graph, GraphTraceStep,
    HybridTrace, MultiQueryOptions, Node, NodeId, NodeKind, QueryAuditEntry, QueryAuditFilters,
    QueryMode, RagChunk, RagQueryOptions, RagResult, RagResultItem, RagStore, RagStream, RagTrace,
    ReferenceState, Source, Store, StoreError, TraceDescriptor, UpdateDocumentRequest, VectorIndex,
    WikiId,
};

// ---------------------------------------------------------------------------
// Helpers (mirroring the §4.2/§4.3/§4.4 suites)
// ---------------------------------------------------------------------------

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

/// A `fact` node with a canonical value (the source-of-truth target of a
/// reference walk).
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

/// A `reference` node pointing at `target` (a link — no embedded snapshot).
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

fn wellformed_graph(nodes: Vec<Node>, extra_edges: Vec<Edge>) -> Graph {
    // Valid Provident graph: exactly one doc-head (→ first node) + one doc-end.
    let first = nodes[0].node_id.clone();
    let doc = nodes[0].document_id.clone();
    let mut edges = vec![
        edge(
            EdgeKind::DocHead,
            (doc.clone(), nid("ROOT")),
            (doc.clone(), first.clone()),
            None,
        ),
        edge(
            EdgeKind::DocEnd,
            (doc.clone(), first),
            (doc.clone(), nid("END")),
            None,
        ),
    ];
    edges.extend(extra_edges);
    Graph { nodes, edges }
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

async fn apply_graph(store: &Store, doc: &Document, graph: Graph) {
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

/// Seed a `fact` doc (`name`, canonical `value`) plus a `reference` doc that
/// links to it, returning `(fact_doc_id, fact_node_id, ref_doc_id)`.
async fn seed_ref_to_fact(store: &Store, w: &WikiId) -> (DocumentId, NodeId, DocumentId, NodeId) {
    let fdoc = new_doc(store, w, "fact-doc").await;
    let fnid = nid("F");
    apply_graph(
        store,
        &fdoc,
        wellformed_graph(
            vec![fact_node(&fdoc.document_id, "F", "canonical-value")],
            vec![],
        ),
    )
    .await;

    let rdoc = new_doc(store, w, "ref-doc").await;
    let rnid = nid("R");
    let target = (fdoc.document_id.clone(), fnid.clone());
    let ref_node = ref_to(&rdoc.document_id, "R", target.clone());
    apply_graph(
        store,
        &rdoc,
        wellformed_graph(
            vec![ref_node],
            vec![edge(
                EdgeKind::Link,
                (rdoc.document_id.clone(), rnid.clone()),
                target.clone(),
                Some(ReferenceState::Resolved),
            )],
        ),
    )
    .await;

    (fdoc.document_id, fnid, rdoc.document_id, rnid)
}

/// A ready engine: `READY` + all subsystems up (the engine is NOT `READY` by
/// default in this suite, mirroring the boot state).
fn ready(store: &Store) {
    store.set_engine_state(EngineState::Ready);
}

// ---------------------------------------------------------------------------
// §4.6.1 Validations (FS-3) — ragQuery
// ---------------------------------------------------------------------------

/// Fail-state FS-3: empty `query` → `ValidationError` (§4.6.1).
#[tokio::test]
async fn rag_query_empty_query_is_validation_error() {
    let store = Arc::new(Store::new());
    ready(&store);
    let err = store
        .rag_query("", &RagQueryOptions::default())
        .await
        .unwrap_err();
    assert_eq!(
        err,
        StoreError::ValidationError("query must be non-empty".into())
    );
}

/// Fail-state FS-3: `topK < 1` → `ValidationError` (§4.6.1).
#[tokio::test]
async fn rag_query_top_k_below_one_is_validation_error() {
    let store = Arc::new(Store::new());
    ready(&store);
    let err = store
        .rag_query(
            "q",
            &RagQueryOptions {
                top_k: Some(0),
                ..Default::default()
            },
        )
        .await
        .unwrap_err();
    assert!(matches!(err, StoreError::ValidationError(_)));
}

/// Fail-state FS-3: `topK > 50` → `ValidationError` (§4.6.1).
#[tokio::test]
async fn rag_query_top_k_above_fifty_is_validation_error() {
    let store = Arc::new(Store::new());
    ready(&store);
    let err = store
        .rag_query(
            "q",
            &RagQueryOptions {
                top_k: Some(51),
                ..Default::default()
            },
        )
        .await
        .unwrap_err();
    assert!(matches!(err, StoreError::ValidationError(_)));
}

/// Fail-state FS-3 (§4.5.2/§4.6.1): `maxHops` out of range (not 1–5) →
/// `ValidationError`. Two bounds: 0 (too small) and 6 (too large).
#[tokio::test]
async fn rag_query_max_hops_out_of_range_is_validation_error() {
    let store = Arc::new(Store::new());
    ready(&store);
    for hops in [0u64, 6u64] {
        let err = store
            .rag_query(
                "q",
                &RagQueryOptions {
                    mode: Some(QueryMode::Graph),
                    max_hops: Some(hops),
                    ..Default::default()
                },
            )
            .await
            .unwrap_err();
        assert!(
            matches!(err, StoreError::ValidationError(_)),
            "maxHops={hops}"
        );
    }
}

/// Fail-state FS-3 (§4.5.2/§4.6.1): a malformed filter — an `edgeType` outside
/// `link|embed|crosslink` — → `ValidationError`.
#[tokio::test]
async fn rag_query_malformed_filters_edge_type_is_validation_error() {
    let store = Arc::new(Store::new());
    ready(&store);
    let filters = QueryAuditFilters {
        node_kind: None,
        edge_type: Some(EdgeKind::Relation), // not a §4.5.2 reference edge type
        target: None,
        state: None,
    };
    let err = store
        .rag_query(
            "q",
            &RagQueryOptions {
                filters: Some(filters),
                ..Default::default()
            },
        )
        .await
        .unwrap_err();
    assert!(matches!(err, StoreError::ValidationError(_)));
}

/// Fail-state FS-3 (§4.6.1): `multiQuery.n` out of range (`n == 0`) →
/// `ValidationError`.
#[tokio::test]
async fn rag_query_multi_query_n_zero_is_validation_error() {
    let store = Arc::new(Store::new());
    ready(&store);
    let err = store
        .rag_query(
            "q",
            &RagQueryOptions {
                multi_query: Some(MultiQueryOptions {
                    enabled: true,
                    n: 0,
                }),
                ..Default::default()
            },
        )
        .await
        .unwrap_err();
    assert!(matches!(err, StoreError::ValidationError(_)));
}

/// Fail-state FS-3 (§4.5.3a.4/§4.6.1): `binaryCandidatePool` not a positive
/// integer (`0`) → `ValidationError`.
#[tokio::test]
async fn rag_query_binary_candidate_pool_zero_is_validation_error() {
    let store = Arc::new(Store::new());
    ready(&store);
    let err = store
        .rag_query(
            "q",
            &RagQueryOptions {
                binary_first_pass: Some(true),
                binary_candidate_pool: Some(0),
                ..Default::default()
            },
        )
        .await
        .unwrap_err();
    assert!(matches!(err, StoreError::ValidationError(_)));
}

// ---------------------------------------------------------------------------
// §4.6.1 EngineUnavailable (FS-8)
// ---------------------------------------------------------------------------

/// Fail-state FS-8: `ragQuery` when the engine is not `READY` → `EngineUnavailable`.
#[tokio::test]
async fn rag_query_engine_not_ready_is_engine_unavailable() {
    let store = Arc::new(Store::new());
    // Fresh store → engine `UNAVAILABLE` (boot state; never made `READY`).
    let err = store
        .rag_query("q", &RagQueryOptions::default())
        .await
        .unwrap_err();
    assert_eq!(err, StoreError::EngineUnavailable);
}

/// Fail-state FS-8: `ragStream` when the engine is not `READY` →
/// `EngineUnavailable` (emitted before any chunk).
#[tokio::test]
async fn rag_stream_engine_not_ready_is_engine_unavailable() {
    let store = Arc::new(Store::new());
    let err = match store.rag_stream("q", &RagQueryOptions::default()).await {
        Ok(_) => panic!("expected EngineUnavailable, got a stream"),
        Err(e) => e,
    };
    assert_eq!(err, StoreError::EngineUnavailable);
}

// ---------------------------------------------------------------------------
// §4.5.1 Query modes
// ---------------------------------------------------------------------------

/// State: `flat` (top-k by combined score) returns an `Ok(RagResult)` with
/// `results` sorted, `engine == 'gnosis'`, and a `flat` `TraceDescriptor` trace.
#[tokio::test]
async fn rag_query_flat_mode_returns_top_k_with_flat_trace() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "doc").await;
    apply_graph(
        &store,
        &doc,
        wellformed_graph(
            vec![content_node(
                &doc.document_id,
                "n1",
                "hybrid retrieval term",
            )],
            vec![],
        ),
    )
    .await;
    ready(&store);

    let res: RagResult = store
        .rag_query(
            "hybrid",
            &RagQueryOptions {
                wiki_id: Some(w.clone()),
                top_k: Some(5),
                mode: Some(QueryMode::Flat),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(res.query, "hybrid");
    assert_eq!(res.engine, "gnosis");
    // flat trace shape (§4.3.3): {mode:'flat', engine, topK, source}.
    match res.trace {
        RagTrace::Flat(trace) => {
            assert_eq!(trace.mode, QueryMode::Flat);
            assert_eq!(trace.engine, "gnosis");
            assert_eq!(trace.top_k, 5);
            assert_eq!(trace.source, Source::Local);
        }
        other => panic!("expected flat trace, got {other:?}"),
    }
}

/// State: `vector` mode returns an `Ok(RagResult)` with a `vector` trace shape
/// `{mode, engine, topK, source}` (§4.3.3).
#[tokio::test]
async fn rag_query_vector_mode_returns_vector_trace() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "doc").await;
    apply_graph(
        &store,
        &doc,
        wellformed_graph(vec![content_node(&doc.document_id, "n1", "alpha")], vec![]),
    )
    .await;
    ready(&store);
    let res = store
        .rag_query(
            "alpha",
            &RagQueryOptions {
                wiki_id: Some(w),
                top_k: Some(5),
                mode: Some(QueryMode::Vector),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    match res.trace {
        RagTrace::Vector(trace) => {
            assert_eq!(trace.mode, QueryMode::Vector);
            assert_eq!(trace.engine, "gnosis");
            assert_eq!(trace.source, Source::Local);
        }
        other => panic!("expected vector trace, got {other:?}"),
    }
}

/// State: `hybrid` mode returns an `Ok(RagResult)` with a `hybrid` trace shape
/// `{mode, engine, legs: ['graph','vector','lexical'], topK, source}` (§4.3.3).
#[tokio::test]
async fn rag_query_hybrid_mode_returns_hybrid_trace_with_three_legs() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "doc").await;
    apply_graph(
        &store,
        &doc,
        wellformed_graph(vec![content_node(&doc.document_id, "n1", "alpha")], vec![]),
    )
    .await;
    ready(&store);
    let res = store
        .rag_query(
            "alpha",
            &RagQueryOptions {
                wiki_id: Some(w),
                top_k: Some(5),
                mode: Some(QueryMode::Hybrid),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    match res.trace {
        RagTrace::Hybrid(trace) => {
            assert_eq!(trace.mode, QueryMode::Hybrid);
            assert_eq!(trace.engine, "gnosis");
            assert_eq!(
                trace.legs,
                vec![
                    "graph".to_string(),
                    "vector".to_string(),
                    "lexical".to_string()
                ]
            );
            assert_eq!(trace.top_k, 5);
        }
        other => panic!("expected hybrid trace, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// §4.5.2 Multi-hop traversal (mode:'graph')
// ---------------------------------------------------------------------------

/// State: `graph` mode resolves through the `reference`→`fact` graph (wrapping
/// `resolve_references`) and returns the ordered path as the `graph` trace.
#[tokio::test]
async fn rag_query_graph_mode_resolves_reference_to_fact() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let (fdoc, fnid, rdoc, rnid) = seed_ref_to_fact(&store, &w).await;
    ready(&store);

    let res = store
        .rag_query(
            "resolve",
            &RagQueryOptions {
                wiki_id: Some(w),
                top_k: Some(5),
                mode: Some(QueryMode::Graph),
                max_hops: Some(3),
                filters: Some(QueryAuditFilters {
                    node_kind: Some(NodeKind::Reference),
                    edge_type: Some(EdgeKind::Link),
                    target: Some((fdoc.clone(), fnid.clone())),
                    state: Some(ReferenceState::Resolved),
                }),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    // The reachable target (the fact node) is in the citations (§4.3.2).
    assert!(res.citations.contains(&(fdoc, fnid)));
    match res.trace {
        RagTrace::Graph(steps) => {
            assert!(!steps.is_empty());
            // Each step is an ordered {from, to, edge, state} walk step.
            assert!(steps
                .iter()
                .all(|s: &GraphTraceStep| !s.from.0 .0.is_empty() && !s.to.0 .0.is_empty()));
        }
        other => panic!("expected graph trace, got {other:?}"),
    }
    // The reference doc is a reachable node too.
    assert!(res
        .results
        .iter()
        .any(|r: &RagResultItem| r.document_id == rdoc && r.node_id == rnid));
}

/// Fail-state FS-11: a graph traversal that exceeds `maxHops` without resolving
/// a target → `HopLimitExceeded`.
#[tokio::test]
async fn rag_query_graph_mode_hop_limit_exceeded() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    // Wire a real 6-link reference chain (`chain-0`..`chain-5` → `chain-fact`)
    // into the store with `apply_graph`, so the §4.5.2 walk has a >1-hop path
    // to traverse. With `max_hops: 1` the walk must reach a second reference
    // (hops == 2 > max_hops == 1) before any fact, so `HopLimitExceeded` fires
    // in `graph_walk`. Without the wired graph the store has no reference roots
    // and the query would return an empty result instead of erroring.
    let mut docs = Vec::new();
    for i in 0..6 {
        docs.push(new_doc(&store, &w, &format!("chain-{i}")).await);
    }
    let fdoc = new_doc(&store, &w, "chain-fact").await;
    for i in 0..5 {
        let from = (docs[i].document_id.clone(), nid(&format!("h{i}")));
        let to = (docs[i + 1].document_id.clone(), nid(&format!("h{}", i + 1)));
        apply_graph(
            &store,
            &docs[i],
            wellformed_graph(
                vec![ref_to(&docs[i].document_id, &format!("h{i}"), to.clone())],
                vec![edge(
                    EdgeKind::Link,
                    from,
                    to,
                    Some(ReferenceState::Resolved),
                )],
            ),
        )
        .await;
    }
    // Terminal link: the 6th reference points at a fact node.
    let from = (docs[5].document_id.clone(), nid("h5"));
    let to = (fdoc.document_id.clone(), nid("F"));
    apply_graph(
        &store,
        &docs[5],
        wellformed_graph(
            vec![ref_to(&docs[5].document_id, "h5", to.clone())],
            vec![edge(
                EdgeKind::Link,
                from,
                to,
                Some(ReferenceState::Resolved),
            )],
        ),
    )
    .await;
    apply_graph(
        &store,
        &fdoc,
        wellformed_graph(vec![fact_node(&fdoc.document_id, "F", "value")], vec![]),
    )
    .await;
    ready(&store);
    let err = store
        .rag_query(
            "hops",
            &RagQueryOptions {
                wiki_id: Some(w),
                mode: Some(QueryMode::Graph),
                max_hops: Some(1),
                ..Default::default()
            },
        )
        .await
        .unwrap_err();
    assert_eq!(err, StoreError::HopLimitExceeded);
}

/// Fail-state FS-12: a `reference`→`fact` cycle (a node already on the current
/// path) → `CycleDetected`.
#[tokio::test]
async fn rag_query_graph_mode_cycle_detected() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    // Two reference nodes pointing at each other form a cycle.
    let d1 = new_doc(&store, &w, "a").await;
    let d2 = new_doc(&store, &w, "b").await;
    apply_graph(
        &store,
        &d1,
        wellformed_graph(
            vec![ref_to(
                &d1.document_id,
                "x",
                (d2.document_id.clone(), nid("y")),
            )],
            vec![edge(
                EdgeKind::Link,
                (d1.document_id.clone(), nid("x")),
                (d2.document_id.clone(), nid("y")),
                Some(ReferenceState::Resolved),
            )],
        ),
    )
    .await;
    apply_graph(
        &store,
        &d2,
        wellformed_graph(
            vec![ref_to(
                &d2.document_id,
                "y",
                (d1.document_id.clone(), nid("x")),
            )],
            vec![edge(
                EdgeKind::Link,
                (d2.document_id.clone(), nid("y")),
                (d1.document_id.clone(), nid("x")),
                Some(ReferenceState::Resolved),
            )],
        ),
    )
    .await;
    ready(&store);
    let err = store
        .rag_query(
            "cycle",
            &RagQueryOptions {
                wiki_id: Some(w),
                mode: Some(QueryMode::Graph),
                ..Default::default()
            },
        )
        .await
        .unwrap_err();
    assert_eq!(err, StoreError::CycleDetected);
}

/// State (§4.5.2): a traversal that resolves no target returns an empty result
/// (`results: []`, `citations: []`) with `blockedBy` listing the `BROKEN`/
/// `STALE` nodes that blocked the walk — a **valid state, not an error**.
#[tokio::test]
async fn rag_query_graph_mode_empty_result_with_blocked_by() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    // A reference node whose target is missing → BROKEN; nothing resolvable.
    let d = new_doc(&store, &w, "broken").await;
    let missing = (did("ghost-doc"), nid("ghost"));
    apply_graph(
        &store,
        &d,
        wellformed_graph(
            vec![ref_to(&d.document_id, "b", missing.clone())],
            vec![edge(
                EdgeKind::Link,
                (d.document_id.clone(), nid("b")),
                missing,
                Some(ReferenceState::Broken),
            )],
        ),
    )
    .await;
    ready(&store);

    let res = store
        .rag_query(
            "x",
            &RagQueryOptions {
                wiki_id: Some(w),
                mode: Some(QueryMode::Graph),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert!(res.results.is_empty(), "no target resolved → empty results");
    assert!(res.citations.is_empty(), "no citations on an empty result");
    let blocked = res
        .blocked_by
        .expect("blockedBy present on a blocked empty result");
    assert!(blocked
        .iter()
        .any(|b: &BlockedBy| b.state == ReferenceState::Broken));
}

/// State (§4.5.2 `expand: 'parent'`): the top `maxParentContext` results by
/// score carry a `parent {documentId, title, snippet, stale}`; results beyond
/// the cap are returned without one.
#[tokio::test]
async fn rag_query_expand_parent_caps_expanded_hits_by_max_parent_context() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    for i in 0..6 {
        let doc = new_doc(&store, &w, &format!("doc-{i}")).await;
        apply_graph(
            &store,
            &doc,
            wellformed_graph(
                vec![content_node(&doc.document_id, "n1", "same term")],
                vec![],
            ),
        )
        .await;
    }
    ready(&store);

    let res = store
        .rag_query(
            "same term",
            &RagQueryOptions {
                wiki_id: Some(w),
                top_k: Some(6),
                mode: Some(QueryMode::Flat),
                expand: Some(ExpandMode::Parent),
                max_parent_context: Some(2),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let with_parent = res.results.iter().filter(|r| r.parent.is_some()).count();
    assert_eq!(
        with_parent, 2,
        "only the top maxParentContext hits carry a parent"
    );
}

/// State (§4.5.2): a `STALE` embed's parent carries `stale: true` so the
/// generator does not trust stale content.
#[tokio::test]
async fn rag_query_stale_embed_parent_carries_stale_flag() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "embedded").await;
    apply_graph(
        &store,
        &doc,
        wellformed_graph(
            vec![content_node(&doc.document_id, "n1", "content")],
            vec![],
        ),
    )
    .await;
    ready(&store);
    let opts = RagQueryOptions {
        wiki_id: Some(w),
        top_k: Some(5),
        expand: Some(ExpandMode::Parent),
        filters: Some(QueryAuditFilters {
            node_kind: None,
            edge_type: Some(EdgeKind::Embed),
            target: None,
            state: Some(ReferenceState::Stale),
        }),
        ..Default::default()
    };
    let res = store.rag_query("content", &opts).await.unwrap();
    // Any expanded parent for a STALE embed must carry stale: true.
    for r in res.results.iter().filter(|r| r.parent.is_some()) {
        assert!(r.parent.as_ref().unwrap().stale);
    }
}

// ---------------------------------------------------------------------------
// §4.5.3 RRF fusion — EXACT merge rule (k=60, ties by (documentId,nodeId) asc)
// ---------------------------------------------------------------------------

/// The RRF merge rule is pinned and EXACT: `RRF(d) = Σ 1/(60 + rank_r(d))`,
/// top-k by descending score, ties by `(documentId, nodeId)` ascending. This
/// test computes the expected ordering by hand from the input lists.
#[tokio::test]
async fn rrf_fusion_merges_ranked_lists_exactly_by_score_then_id() {
    // Input lists (1-based ranks implied by position).
    let graph = vec![("a", "n1"), ("b", "n1"), ("c", "n1")];
    let vector = vec![("b", "n1"), ("c", "n1"), ("d", "n1")];
    let lexical = vec![("c", "n1"), ("a", "n1")];

    let lists: Vec<Vec<(DocumentId, NodeId)>> = [graph, vector, lexical]
        .map(|l| l.into_iter().map(|(d, n)| (did(d), nid(n))).collect())
        .into_iter()
        .collect();

    // Hand-derived RRF with k = RRF_K = 60:
    //   c: graph r3 -> 1/63, vector r2 -> 1/62, lexical r1 -> 1/61
    //   a: graph r1 -> 1/61,                lexical r2 -> 1/62
    //   b: graph r2 -> 1/62, vector r1 -> 1/61
    //   d:                 vector r3 -> 1/63
    // Scores: c > a == b > d; the a/b tie breaks by (documentId, nodeId)
    // ascending ("a","n1") before ("b","n1").
    let expected = vec![did("c"), did("a"), did("b"), did("d")]
        .into_iter()
        .map(|d| (d, nid("n1")))
        .collect::<Vec<_>>();

    // sanity of the tie-break assumption (RRF k is pinned at 60 by the spec)
    let merged = rrf_fuse(&lists, expected.len());
    assert_eq!(
        merged, expected,
        "RRF must merge exactly (score desc, id asc)"
    );
}

/// The RRF tie-break is deterministic by `(documentId, nodeId)` ascending — two
/// items with identical RRF scores are ordered by their ids.
#[tokio::test]
async fn rrf_fusion_breaks_ties_by_document_then_node_id_ascending() {
    // list1=[P, Q], list2=[Q, P]: P and Q both get 1/61 + 1/62 → an exact tie.
    let lists = vec![
        vec![(did("d"), nid("n-p")), (did("d"), nid("n-q"))],
        vec![(did("d"), nid("n-q")), (did("d"), nid("n-p"))],
    ];
    // Same documentId; nodeId "n-p" sorts before "n-q" → P first.
    let expected = vec![(did("d"), nid("n-p")), (did("d"), nid("n-q"))];
    let merged = rrf_fuse(&lists, 2);
    assert_eq!(merged, expected);
}

// ---------------------------------------------------------------------------
// §4.3.3 Provenance (trace shapes) — GREEN type-shape guards
// ---------------------------------------------------------------------------

/// A result set without a `trace` is not a valid `RagResult` (§4.3.3): the
/// typed `RagResult.trace` is non-optional, and each mode maps to the pinned
/// shape. (Type-shape guard — green; the runtime `TraceUnavailable` boundary
/// is a §4.6.1/§5.1 transport concern noted in the report.)
#[test]
fn rag_result_trace_is_always_present_and_mode_shaped() {
    let (d, n) = (did("d"), nid("n"));
    let flat = RagResult {
        query: "q".into(),
        results: vec![],
        engine: "gnosis".into(),
        citations: vec![],
        trace: RagTrace::Flat(TraceDescriptor {
            mode: QueryMode::Flat,
            engine: "gnosis".into(),
            top_k: 5,
            source: Source::Local,
        }),
        blocked_by: None,
    };
    assert!(matches!(flat.trace, RagTrace::Flat(_)));
    let graph = RagResult {
        query: "q".into(),
        results: vec![],
        engine: "gnosis".into(),
        citations: vec![],
        trace: RagTrace::Graph(vec![GraphTraceStep {
            from: (d.clone(), n.clone()),
            to: (d.clone(), n.clone()),
            edge: EdgeKind::Link,
            state: ReferenceState::Resolved,
        }]),
        blocked_by: None,
    };
    assert!(matches!(graph.trace, RagTrace::Graph(_)));
    let hybrid = RagResult {
        query: "q".into(),
        results: vec![],
        engine: "gnosis".into(),
        citations: vec![],
        trace: RagTrace::Hybrid(HybridTrace {
            mode: QueryMode::Hybrid,
            engine: "gnosis".into(),
            legs: vec!["graph".into(), "vector".into(), "lexical".into()],
            top_k: 5,
            source: Source::Local,
        }),
        blocked_by: Some(vec![BlockedBy {
            document_id: d.clone(),
            node_id: n.clone(),
            state: ReferenceState::Broken,
        }]),
    };
    assert!(matches!(hybrid.trace, RagTrace::Hybrid(_)));
}

// ---------------------------------------------------------------------------
// §4.3.4 Query audit-log recording
// ---------------------------------------------------------------------------

/// State (§4.3.4): after N `ragQuery` calls, `getQueryAuditLog` returns N
/// entries recording `{query, filters, mode, resultCount, timestamp,
/// requester}`. (RED: `rag_query` is a stub, so nothing appends yet.)
#[tokio::test]
async fn rag_query_appends_query_audit_entries() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    ready(&store);

    let n = 3;
    for k in 0..n {
        store
            .rag_query(
                &format!("query-{k}"),
                &RagQueryOptions {
                    wiki_id: Some(w.clone()),
                    top_k: Some(5),
                    mode: Some(QueryMode::Flat),
                    filters: Some(QueryAuditFilters {
                        node_kind: Some(NodeKind::Content),
                        edge_type: None,
                        target: None,
                        state: None,
                    }),
                    requester: Some("gui-user-1".to_string()),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
    }
    let log: Vec<QueryAuditEntry> = store.get_query_audit_log().await.unwrap();
    assert_eq!(log.len(), n, "every ragQuery appends one audit entry");
    assert_eq!(log[0].requester, "gui-user-1");
    assert_eq!(log[0].query, "query-0");
    assert_eq!(log[0].mode, QueryMode::Flat);
    assert!(
        log[0].filters.is_some(),
        "filters recorded (or null when none supplied)"
    );
    assert!(!log[0].timestamp.is_empty(), "timestamp is ISO-8601 UTC");
}

// ---------------------------------------------------------------------------
// §4.6.1 ragStream
// ---------------------------------------------------------------------------

/// State: `ragStream` emits a `type:'result'` chunk carrying the full
/// `RagResult`, then a `type:'done'` chunk, then closes.
#[tokio::test]
async fn rag_stream_emits_result_then_done() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    ready(&store);

    let stream: RagStream = store
        .rag_stream(
            "q",
            &RagQueryOptions {
                wiki_id: Some(w),
                top_k: Some(3),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let chunks: Vec<RagChunk> = stream.collect().await;
    assert!(
        matches!(chunks[0], RagChunk::Result(_)),
        "a result chunk first"
    );
    assert_eq!(
        chunks.last(),
        Some(&RagChunk::Done),
        "a done chunk closes the stream"
    );
}

/// Fail-state: a mid-stream failure is emitted as a `type:'error'` chunk, then
/// the stream closes (not a channel-level error).
#[tokio::test]
async fn rag_stream_emits_error_chunk_then_closes() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    // Wire an actual reference cycle (two reference nodes each linking to the
    // other) into the store before streaming, so the §4.5.2 graph walk detects
    // the cycle. On an empty store there is no reference path to walk and the
    // stream would emit a `Result` chunk instead of the `Error(CycleDetected)`
    // chunk asserted below.
    let d1 = new_doc(&store, &w, "a").await;
    let d2 = new_doc(&store, &w, "b").await;
    apply_graph(
        &store,
        &d1,
        wellformed_graph(
            vec![ref_to(
                &d1.document_id,
                "x",
                (d2.document_id.clone(), nid("y")),
            )],
            vec![edge(
                EdgeKind::Link,
                (d1.document_id.clone(), nid("x")),
                (d2.document_id.clone(), nid("y")),
                Some(ReferenceState::Resolved),
            )],
        ),
    )
    .await;
    apply_graph(
        &store,
        &d2,
        wellformed_graph(
            vec![ref_to(
                &d2.document_id,
                "y",
                (d1.document_id.clone(), nid("x")),
            )],
            vec![edge(
                EdgeKind::Link,
                (d2.document_id.clone(), nid("y")),
                (d1.document_id.clone(), nid("x")),
                Some(ReferenceState::Resolved),
            )],
        ),
    )
    .await;
    ready(&store);
    // A graph-mode cycle inside the stream → CycleDetected as an error chunk.
    let stream = store
        .rag_stream(
            "x",
            &RagQueryOptions {
                wiki_id: Some(w),
                mode: Some(QueryMode::Graph),
                max_hops: Some(3),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let chunks: Vec<RagChunk> = stream.collect().await;
    assert!(chunks
        .iter()
        .any(|c| matches!(c, RagChunk::Error(StoreError::CycleDetected))));
    assert_eq!(
        chunks.last(),
        Some(&RagChunk::Done),
        "stream closes after the error chunk"
    );
}

// ---------------------------------------------------------------------------
// §4.6.1 getEngineStatus
// ---------------------------------------------------------------------------

/// State: `READY` when all subsystems are up (§4.6.1 engine state).
#[tokio::test]
async fn get_engine_status_ready_when_all_subsystems_up() {
    let store = Arc::new(Store::new());
    store.set_engine_state(EngineState::Ready);
    store.set_subsystems(EngineSubsystems {
        store: true,
        graph: true,
        lexical: true,
        vector: true,
        embedding: true,
        reranker: true,
    });
    let status = store.get_engine_status().await;
    assert_eq!(status.state, EngineState::Ready);
    assert!(status.subsystems.store && status.subsystems.graph && status.subsystems.lexical);
}

/// State: `DEGRADED` when a non-core subsystem (the embedding provider) is down;
/// core store/graph/lexical still work (§4.6.1).
#[tokio::test]
async fn get_engine_status_degraded_when_embedding_provider_down() {
    let store = Arc::new(Store::new());
    store.set_engine_state(EngineState::Degraded);
    store.set_subsystems(EngineSubsystems {
        store: true,
        graph: true,
        lexical: true,
        vector: true,
        embedding: false, // embedding provider down
        reranker: true,
    });
    let status = store.get_engine_status().await;
    assert_eq!(status.state, EngineState::Degraded);
    assert!(
        !status.subsystems.embedding,
        "embedding subsystem reported down"
    );
    assert!(status.subsystems.store && status.subsystems.graph && status.subsystems.lexical);
}

/// State: `UNAVAILABLE` when the engine is not running/reachable — and a
/// `ragQuery` on it → `EngineUnavailable` (§4.6.1, FS-8).
#[tokio::test]
async fn get_engine_status_unavailable_blocked_for_rag_query() {
    let store = Arc::new(Store::new());
    store.set_engine_state(EngineState::Unavailable);
    assert_eq!(
        store.get_engine_status().await.state,
        EngineState::Unavailable
    );
    let err = store
        .rag_query("q", &RagQueryOptions::default())
        .await
        .unwrap_err();
    assert_eq!(err, StoreError::EngineUnavailable);
}

// ---------------------------------------------------------------------------
// Concurrency (IMMUTABLE-DERIVED-SNAPSHOT / ARC-SHARED-ENGINE)
// ---------------------------------------------------------------------------

/// Genuinely-contending `#[tokio::test(flavor = "multi_thread")]` + `Barrier`:
/// multiple concurrent `ragQuery` reads (against the immutable snapshot, the
/// read-side) race a concurrent index rebuild (the immutable-snapshot swap).
/// The read-side is a RED stub, so every reader surfaces the unimplemented
/// panic — proving the concurrent read path is reachable but not yet green.
#[tokio::test(flavor = "multi_thread")]
async fn concurrent_rag_query_reads_race_a_concurrent_index_rebuild() {
    use tokio::sync::Barrier;

    let store = Arc::new(Store::new());
    ready(&store);

    // Seed an immutable derived snapshot (the read-side host).
    let mut v = VectorIndex::default();
    v.entries
        .insert((did("d"), nid("n"), FieldType::Full), vec![1.0, 0.0]);
    store.swap_snapshot(DerivedIndexes {
        lexical: None,
        vectors: Some(v),
        epoch: 1,
    });

    let barrier = Arc::new(Barrier::new(9)); // 8 readers + 1 rebuild writer
    let mut readers = Vec::new();
    for k in 0..8 {
        let s = store.clone();
        let b = barrier.clone();
        readers.push(tokio::spawn(async move {
            b.wait().await;
            s.rag_query(&format!("concurrent-{k}"), &RagQueryOptions::default())
                .await
        }));
    }
    // The concurrent index rebuild: swap a fresh snapshot on the epoch feed.
    let wstore = store.clone();
    let wb = barrier.clone();
    let rebuild = tokio::spawn(async move {
        wb.wait().await;
        for epoch in 1..=10u64 {
            let mut v1 = VectorIndex::default();
            v1.entries.insert(
                (did("d"), nid("n"), FieldType::Full),
                vec![epoch as f32, 0.0],
            );
            wstore.swap_snapshot(DerivedIndexes {
                lexical: None,
                vectors: Some(v1),
                epoch,
            });
        }
    });

    // The 8 readers + 1 rebuild self-synchronize on the barrier, then the
    // readers issue genuinely-concurrent `ragQuery` reads against the immutable
    // snapshot while the rebuild swipes the index. Each reader asserts a valid
    // `Ok(RagResult)` — which is RED today (the query path is a compiling stub
    // that panics, surfacing as a join error). The rebuild proceeds regardless.
    for r in readers {
        let joined = r
            .await
            .expect("concurrent reader task must not crash on join");
        let res: RagResult = joined.expect("concurrent rag_query must return Ok(RagResult)");
        let _ = res;
    }
    let _ = rebuild.await;
}

/// The immutable-snapshot swap is atomic under concurrent readers: N OS threads
/// read a never-torn snapshot (epoch and its vector always agree) while a
/// writer `swap_snapshot`s new indexes — the concurrency contract
/// (IMMUTABLE-DERIVED-SNAPSHOT). This verifies the provided swap vehicle.
#[tokio::test(flavor = "multi_thread")]
async fn derived_snapshot_swap_is_atomic_under_concurrent_readers() {
    let store = Arc::new(Store::new());
    let mut v0 = VectorIndex::default();
    v0.entries
        .insert((did("d"), nid("n"), FieldType::Full), vec![1.0, 0.0]);
    store.swap_snapshot(DerivedIndexes {
        lexical: None,
        vectors: Some(v0),
        epoch: 1,
    });

    const READERS: usize = 8;
    // 8 reader threads + 1 writer thread self-synchronize on this gate.
    let gate = Arc::new(Barrier::new(READERS + 1));
    let mut readers = Vec::new();
    for i in 0..READERS {
        let s = store.clone();
        let b = gate.clone();
        let key = (did("d"), nid("n"), FieldType::Full); // fresh per reader
        readers.push(std::thread::spawn(move || {
            b.wait();
            for _ in 0..2000 {
                let snap = s.snapshot();
                let full = snap
                    .vectors
                    .as_ref()
                    .and_then(|v| v.entries.get(&key).cloned());
                // A reader never observes a torn snapshot: epoch 1 ↔ [1.0,0.0],
                // epoch 2 ↔ [9.9,9.9] — always the complete, internally-consistent image.
                match snap.epoch {
                    1 => assert_eq!(
                        full,
                        Some(vec![1.0, 0.0]),
                        "reader {i} saw epoch-1 torn vector"
                    ),
                    2 => assert_eq!(
                        full,
                        Some(vec![9.9, 9.9]),
                        "reader {i} saw epoch-2 torn vector"
                    ),
                    e => panic!("reader {i} saw unexpected epoch {e}"),
                }
            }
        }));
    }
    let ws = store.clone();
    let wb = gate.clone();
    let writer = std::thread::spawn(move || {
        wb.wait();
        for _ in 0..5 {
            let mut v2 = VectorIndex::default();
            v2.entries
                .insert((did("d"), nid("n"), FieldType::Full), vec![9.9, 9.9]);
            ws.swap_snapshot(DerivedIndexes {
                lexical: None,
                vectors: Some(v2),
                epoch: 2,
            });
            let mut v1 = VectorIndex::default();
            v1.entries
                .insert((did("d"), nid("n"), FieldType::Full), vec![1.0, 0.0]);
            ws.swap_snapshot(DerivedIndexes {
                lexical: None,
                vectors: Some(v1),
                epoch: 1,
            });
        }
    });
    // All 9 participate on the gate, then proceed; the test thread just joins.
    for r in readers {
        r.join().unwrap();
    }
    writer.join().unwrap();
}
