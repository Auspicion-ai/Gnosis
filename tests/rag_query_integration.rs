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

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Barrier};

use futures::StreamExt;
use gnosis::{
    rrf_fuse, BlockedBy, CreateDocumentRequest, DerivedIndexes, Document, DocumentId, Edge,
    EdgeKind, EmbeddingProvider, EngineState, ExpandMode, FieldType, Graph, GraphTraceStep,
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

/// A deterministic query-surface embedding provider (fixed vector; availability
/// override). Mirrors the §4.5.3 retrieval-stack mock so the `ragQuery` vector/
/// hybrid/hyde legs' provider seam is exercised from the query surface.
struct StubProvider {
    available: bool,
    emit_error: bool,
    v: Vec<f32>,
}

impl StubProvider {
    fn hit(v: Vec<f32>) -> Self {
        StubProvider {
            available: true,
            emit_error: false,
            v,
        }
    }
    fn unreachable() -> Self {
        StubProvider {
            available: false,
            emit_error: true,
            v: Vec::new(),
        }
    }
}

impl EmbeddingProvider for StubProvider {
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

/// Seed the immutable vector snapshot with `full`-field entries (the read-side
/// host the §4.5 vector legs consume).
fn seed_vector_index(store: &Store, entries: Vec<((DocumentId, NodeId), Vec<f32>)>) {
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

/// DOC-REVIEW QUICK-PIN (docs/pending.md · FS-19): `MultiQueryExpansionFailed`
/// is reachable and asserted. A `multiQuery:{enabled:true, n>=2}` fan-out over
/// a wiki whose `term_popularity` yields **no distinct indexable term** (an
/// empty wiki → `term_popularity` has nothing to add) → `MultiQueryExpansionFailed`
/// (§4.5.3/§4.6.1). This is the one engine-internal fail-state that a typed
/// test can drive with no seam.
#[tokio::test]
async fn rag_query_multi_query_empty_wiki_is_expansion_failed() {
    let store = Arc::new(Store::new());
    // An empty wiki: created but carrying no documents, so `term_popularity`
    // over `wiki_id: Some(w)` yields zero distinct candidate terms to fan out to.
    let w = new_wiki(&store, "empty").await;
    ready(&store);
    let err = store
        .rag_query(
            "some query",
            &RagQueryOptions {
                wiki_id: Some(w),
                mode: Some(QueryMode::Flat),
                multi_query: Some(MultiQueryOptions {
                    enabled: true,
                    n: 2,
                }),
                ..Default::default()
            },
        )
        .await
        .unwrap_err();
    assert_eq!(
        err,
        StoreError::MultiQueryExpansionFailed,
        "multi-query fan-out with no distinct term to add must fail (FS-19)"
    );
}

/// DOC-REVIEW QUICK-PIN (docs/pending.md · §4.6.1): the stream surface routes
/// the same `MultiQueryExpansionFailed` fail-state to an `Error` chunk. `rag_stream`
/// calls `rag_query` and maps its `Err` to `RagChunk::Error`, then `Done`.
#[tokio::test]
async fn rag_stream_multi_query_empty_wiki_emits_error_chunk() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "empty").await;
    ready(&store);
    let stream = store
        .rag_stream(
            "some query",
            &RagQueryOptions {
                wiki_id: Some(w),
                mode: Some(QueryMode::Flat),
                multi_query: Some(MultiQueryOptions {
                    enabled: true,
                    n: 2,
                }),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let chunks: Vec<RagChunk> = stream.collect().await;
    assert!(
        chunks
            .iter()
            .any(|c| matches!(c, RagChunk::Error(StoreError::MultiQueryExpansionFailed))),
        "stream must emit the MultiQueryExpansionFailed Error chunk (§4.6.1)"
    );
    assert_eq!(
        chunks.last(),
        Some(&RagChunk::Done),
        "stream closes after the error chunk"
    );
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

/// State: `vector` mode returns the hits from the **VECTOR** leg (NOT a lexical
/// fallback). A seeded vector-doc whose content does not lexically match the
/// query is surfaced only via the provider-injected dense leg (§4.5.3 / HIGH-1).
///
/// RED today: `rag_query`'s `vector_query` ignores the injected provider + the
/// seeded vector index and returns the lexical (empty) list, so the vector-leg
/// hit below is absent.
#[tokio::test]
async fn rag_query_vector_mode_returns_vector_leg_hits() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    // A doc whose content does NOT contain the query term — plain lexical
    // retrieval cannot surface it; only the vector leg (via the seeded index +
    // injected provider) can.
    let doc = new_doc(&store, &w, "vec-source").await;
    apply_graph(
        &store,
        &doc,
        wellformed_graph(
            vec![content_node(&doc.document_id, "v1", "unrelated-topic")],
            vec![],
        ),
    )
    .await;
    seed_vector_index(
        &store,
        vec![((doc.document_id.clone(), nid("v1")), vec![1.0, 0.0])],
    );
    store.set_embedding_provider(Arc::new(StubProvider::hit(vec![1.0, 0.0])));
    ready(&store);

    let res = store
        .rag_query(
            "gamma",
            &RagQueryOptions {
                wiki_id: Some(w),
                top_k: Some(5),
                mode: Some(QueryMode::Vector),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    // trace shape (§4.3.3).
    match res.trace {
        RagTrace::Vector(trace) => {
            assert_eq!(trace.mode, QueryMode::Vector);
            assert_eq!(trace.engine, "gnosis");
            assert_eq!(trace.source, Source::Local);
        }
        other => panic!("expected vector trace, got {other:?}"),
    }
    // The results MUST come from the vector leg: the top vector-leg hit is the
    // seeded (unrelated, non-lexical) doc — a lexical fallback returns nothing.
    assert!(
        res.results
            .iter()
            .any(|r| r.document_id == doc.document_id && r.node_id == nid("v1")),
        "vector-leg hit must be returned from the dense leg"
    );
}

/// Fail-state FS-13 reachable from the **`ragQuery` surface**: vector mode with a
/// real vector index built but an **unreachable embedding provider** →
/// `EmbeddingUnavailable` (§4.6.1 / HIGH-3). RED today: `rag_query`'s
/// `vector_query` ignores the provider seam and returns an `Ok` lexical result.
#[tokio::test]
async fn rag_query_vector_mode_unavailable_provider_is_embedding_unavailable() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    // A real doc in the wiki whose vector index IS built — only the provider
    // being unreachable can fail the query (not the index).
    let doc = new_doc(&store, &w, "vec-source").await;
    apply_graph(
        &store,
        &doc,
        wellformed_graph(vec![content_node(&doc.document_id, "v1", "alpha")], vec![]),
    )
    .await;
    seed_vector_index(
        &store,
        vec![((doc.document_id.clone(), nid("v1")), vec![1.0, 0.0])],
    );
    store.set_embedding_provider(Arc::new(StubProvider::unreachable()));
    ready(&store);

    let err = store
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
        .unwrap_err();
    assert_eq!(err, StoreError::EmbeddingUnavailable);
}

/// Fail-state FS-14 reachable from the **`ragQuery` surface**: vector mode when
/// the vector index is **not built** → `VectorIndexUnavailable` (§4.6.1 / HIGH-3).
/// RED today: `rag_query`'s `vector_query` never consults the vector index and
/// returns an `Ok` (possibly empty) lexical result instead of the fail-state.
#[tokio::test]
async fn rag_query_vector_mode_index_not_built_is_vector_index_unavailable() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "vec-source").await;
    apply_graph(
        &store,
        &doc,
        wellformed_graph(vec![content_node(&doc.document_id, "v1", "alpha")], vec![]),
    )
    .await;
    // No vector index is seeded (snapshot vectors == None).
    store.set_embedding_provider(Arc::new(StubProvider::hit(vec![1.0, 0.0])));
    ready(&store);

    let err = store
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
        .unwrap_err();
    assert_eq!(err, StoreError::VectorIndexUnavailable);
}

/// State: `hybrid` mode merges the graph + vector + lexical legs by RRF, and the
/// merged result reflects **three distinct non-degenerate legs** (§4.5.3 / HIGH-2).
/// A lexical hit, a vector-only hit, and a graph-only hit that all differ must
/// ALL appear — a result that only reflects the lexical list must fail.
///
/// RED today: `hybrid_query` collapses the vector leg to the lexical list and
/// the graph leg to empty, so only the lexical hit appears.
#[tokio::test]
async fn rag_query_hybrid_mode_merges_three_distinct_legs() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;

    // LEXICAL leg hit — content contains the query term.
    let ldoc = new_doc(&store, &w, "lex-doc").await;
    apply_graph(
        &store,
        &ldoc,
        wellformed_graph(
            vec![content_node(&ldoc.document_id, "n-lx", "alpha")],
            vec![],
        ),
    )
    .await;

    // VECTOR leg hit — content does NOT match the query; only the seeded vector
    // index + injected provider reach it.
    let vdoc = new_doc(&store, &w, "vec-doc").await;
    apply_graph(
        &store,
        &vdoc,
        wellformed_graph(vec![content_node(&vdoc.document_id, "n-vx", "zzz")], vec![]),
    )
    .await;
    seed_vector_index(
        &store,
        vec![((vdoc.document_id.clone(), nid("n-vx")), vec![1.0, 0.0])],
    );

    // GRAPH leg hit — a reference node resolving to a fact (a §4.5.2 walk root).
    let (_fdoc, _fnid, rdoc, rnid) = seed_ref_to_fact(&store, &w).await;

    store.set_embedding_provider(Arc::new(StubProvider::hit(vec![1.0, 0.0])));
    ready(&store);

    let res = store
        .rag_query(
            "alpha",
            &RagQueryOptions {
                wiki_id: Some(w),
                top_k: Some(10),
                mode: Some(QueryMode::Hybrid),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    // trace legs (§4.3.3): ['graph','vector','lexical'].
    match res.trace {
        RagTrace::Hybrid(trace) => {
            assert_eq!(trace.mode, QueryMode::Hybrid);
            assert_eq!(trace.legs, vec!["graph", "vector", "lexical"]);
            assert_eq!(trace.top_k, 10);
        }
        other => panic!("expected hybrid trace, got {other:?}"),
    }
    // Non-degeneracy: EVERY distinct leg contributed at least one result.
    assert!(
        res.results
            .iter()
            .any(|r| r.document_id == ldoc.document_id && r.node_id == nid("n-lx")),
        "lexical leg must contribute"
    );
    assert!(
        res.results
            .iter()
            .any(|r| r.document_id == vdoc.document_id && r.node_id == nid("n-vx")),
        "vector leg must contribute (a distinct, non-degenerate leg)"
    );
    assert!(
        res.results
            .iter()
            .any(|r| r.document_id == rdoc && r.node_id == rnid),
        "graph leg must contribute"
    );
}

// ---------------------------------------------------------------------------
// §4.5.2 Multi-hop traversal (mode:'graph')
// ---------------------------------------------------------------------------/// State: `graph` mode resolves through the `reference`→`fact` graph (wrapping
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

/// State (MEDIUM-7, §4.5.2/§4.4.2 liveness): a reference edge whose
/// target is **missing** must be derived as blocked by liveness (surfaced as an
/// empty result with `blockedBy`) — NOT fabricated as `Resolved` and walked into
/// a missing node (which would error).
///
/// Status: **GREEN**. The store already derives reference state from liveness at
/// ingest — `update_document` stamps a missing-target link as `BROKEN` — so the
/// `graph_walk` never actually receives a `state: None` edge to a missing target
/// (its `unwrap_or(Resolved)` fallback is unreachable through the public API).
/// This is a genuine regression guard: it asserts the required end-to-end
/// observable behavior (empty result + blockedBy, no phantom walk / no error).
#[tokio::test]
async fn rag_query_graph_mode_derives_blocked_from_missing_target() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    // A reference whose edge carries `state: None` and targets a missing node.
    let d = new_doc(&store, &w, "dangling").await;
    let missing = (did("ghost-doc"), nid("ghost"));
    let rnid = nid("r");
    apply_graph(
        &store,
        &d,
        wellformed_graph(
            vec![ref_to(&d.document_id, "r", missing.clone())],
            vec![edge(
                EdgeKind::Link,
                (d.document_id.clone(), rnid.clone()),
                missing,
                None, // state: None — state must be derived from liveness
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
        .expect("a missing target derived from liveness is a valid blocked state, not an error");
    assert!(
        res.results.is_empty(),
        "no target resolved → an empty (blocked) result, not a walk into a phantom"
    );
    assert!(res.citations.is_empty());
    let blocked = res
        .blocked_by
        .expect("blockedBy present for the blocked empty result");
    assert!(
        blocked
            .iter()
            .any(|b: &BlockedBy| b.document_id == d.document_id && b.node_id == rnid),
        "the liveness-blocked reference is listed in blockedBy"
    );
}

/// State (MEDIUM-10, §4.5.2): `blockedBy` is only valid on an **empty** result
/// set. When some roots resolve and others are blocked, the result must NOT
/// carry `blockedBy`.
/// RED today: `graph_query` sets `blockedBy = Some(...)` whenever any root was
/// blocked, even alongside resolved results.
#[tokio::test]
async fn rag_query_graph_mode_blocked_by_only_on_empty_results() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    // Root #1 resolves (reference → fact); root #2 is BROKEN/blocked.
    let (_fdoc, _fnid, rdoc, rnid) = seed_ref_to_fact(&store, &w).await;
    let bdoc = new_doc(&store, &w, "broken").await;
    let missing = (did("ghost-doc"), nid("ghost"));
    let bnode = nid("b");
    apply_graph(
        &store,
        &bdoc,
        wellformed_graph(
            vec![ref_to(&bdoc.document_id, "b", missing.clone())],
            vec![edge(
                EdgeKind::Link,
                (bdoc.document_id.clone(), bnode.clone()),
                missing,
                Some(ReferenceState::Broken),
            )],
        ),
    )
    .await;
    ready(&store);

    let res = store
        .rag_query(
            "resolve",
            &RagQueryOptions {
                wiki_id: Some(w),
                mode: Some(QueryMode::Graph),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert!(
        res.results
            .iter()
            .any(|r| r.document_id == rdoc && r.node_id == rnid),
        "the resolvable reference root resolves"
    );
    assert!(
        res.blocked_by.is_none(),
        "blockedBy is only valid on empty results — a partial/non-empty result must not carry it"
    );
}

/// Determinism regression (§4.5.2/§4.5.3 re-audit, MEDIUM-A). `graph_query` and
/// the hybrid graph leg collect their walk roots by iterating a `HashMap`
/// (`docs.values()`), which is **not** order-guaranteed. When several
/// graph roots all resolve (so every root ties at the same relevance), the
/// ordering (and, at a `top_k` boundary, even the *set*) of the returned hits
/// must still be deterministic and tie-break by `(documentId, nodeId)`
/// ascending (§4.5.3 RRF pin; `mode: graph` per §4.5.2 deterministic walk).
///
/// State under test: 5 distinct `reference`→`fact` chains, all resolving, with
/// `top_k = 3` (so the `top_k` truncation happens *between* the 5 tied roots and
/// the boundary is exercised). We make the assertion in two ways:
///   (a) **cross-instance determinism** — build the identical corpus across
///       NUM_STORES independently-created `Store` instances (each fresh
///       `Store::new()` spins up a NEW HashMap `RandomState`, so each instance
///       simulates a separate run/process with a different unordered iteration
///       order). Every instance must return the identical
///       `(documentId, nodeId)`-ascending top-3 for both `hybrid` and `graph`.
///       A single instance that reorders (either a wrong SET at the top_k
///       boundary, or the right set in the wrong order) is the nondeterminism
///       bug the re-audit flagged.
///   (b) **within-instance run-to-run stability** — repeat the query 30× on the
///       first instance and assert every run returns the identical ordering.
///
/// The only contributing leg is the graph leg (no vector index seeded, lexical
/// query term matches nothing), so the returned hits are precisely the graph-leg
/// roots and the tie-break rule is what a deterministic merge must enforce.
#[tokio::test]
async fn graph_hybrid_ordering_is_deterministic() {
    const CHAINS: usize = 48; // many roots → several reference-root docs share a shard (the unordered-HashMap hazard)
    const TOP_K: usize = 4; // few relative to CHAINS → the top_k boundary cuts the tie deep inside a shared shard
    const NUM_STORES: usize = 40; // many fresh RandomState seeds → exercises the unordered HashMap order

    // Seed `CHAINS` distinct resolving `reference`→`fact` chains into `store`.
    // Each chain's reference root resolves (a Resolved link to a fact), so all
    // roots tie in the graph leg.
    async fn seed_chains(store: &Arc<Store>, w: &WikiId) -> Vec<(DocumentId, NodeId)> {
        let mut roots: Vec<(DocumentId, NodeId)> = Vec::new();
        for _ in 0..CHAINS {
            let (_fdoc, _fnid, rdoc, rnid) = seed_ref_to_fact(store, w).await;
            roots.push((rdoc, rnid));
        }
        roots
    }

    for store_idx in 0..NUM_STORES {
        let store = Arc::new(Store::new());
        let w = new_wiki(&store, "w").await;
        let roots = seed_chains(&store, &w).await;
        ready(&store);

        // §4.5.3 / §4.5.2 deterministic expectation: ties broken by
        // `(documentId, nodeId)` ascending, truncated to top_k.
        let mut expected: Vec<(DocumentId, NodeId)> = roots;
        expected.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
        expected.truncate(TOP_K);

        for mode in [QueryMode::Hybrid, QueryMode::Graph] {
            // (a) cross-instance assertion: THIS store instance must produce the
            // deterministic ascending order.
            let res = store
                .rag_query(
                    "determinism-check-zz-unmatched",
                    &RagQueryOptions {
                        wiki_id: Some(w.clone()),
                        top_k: Some(TOP_K as u64),
                        mode: Some(mode),
                        ..Default::default()
                    },
                )
                .await
                .unwrap();
            let order: Vec<(DocumentId, NodeId)> = res
                .results
                .iter()
                .map(|r| (r.document_id.clone(), r.node_id.clone()))
                .collect();
            assert_eq!(
                order, expected,
                "{mode:?} store #{store_idx}: the result ordering must tie-break by \
                 (documentId, nodeId) ascending regardless of HashMap insertion order — \
                 a fresh store instance reordered it (nondeterminism)"
            );

            // (b) within-instance stability: 30 identical runs must not vary.
            let mut seen: Option<Vec<(DocumentId, NodeId)>> = None;
            for _ in 0..30 {
                let rerun = store
                    .rag_query(
                        "determinism-check-zz-unmatched",
                        &RagQueryOptions {
                            wiki_id: Some(w.clone()),
                            top_k: Some(TOP_K as u64),
                            mode: Some(mode),
                            ..Default::default()
                        },
                    )
                    .await
                    .unwrap();
                let rerun_order: Vec<(DocumentId, NodeId)> = rerun
                    .results
                    .iter()
                    .map(|r| (r.document_id.clone(), r.node_id.clone()))
                    .collect();
                match &seen {
                    Some(prev) => assert_eq!(
                        &rerun_order, prev,
                        "{mode:?} store #{store_idx}: the ordering must be identical across \
                         every run (graph-leg roots are collected from an unordered HashMap)"
                    ),
                    None => seen = Some(rerun_order),
                }
            }
        }
    }
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
/// generator does not trust stale content. Non-vacuous: a reference node with a
/// REAL `Stale` embed edge (not an edgeless node) is retrieved and a parent is
/// actually produced carrying `stale: true`.
#[tokio::test]
async fn rag_query_stale_embed_parent_carries_stale_flag() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    // A reference node with a REAL Stale embed edge to a fact.
    let fdoc = new_doc(&store, &w, "fact-doc").await;
    apply_graph(
        &store,
        &fdoc,
        wellformed_graph(vec![fact_node(&fdoc.document_id, "F", "v")], vec![]),
    )
    .await;
    let rdoc = new_doc(&store, &w, "stale-embed").await;
    let target = (fdoc.document_id.clone(), nid("F"));
    let rnid = nid("R");
    apply_graph(
        &store,
        &rdoc,
        wellformed_graph(
            vec![ref_to(&rdoc.document_id, "R", target.clone())],
            vec![edge(
                EdgeKind::Embed,
                (rdoc.document_id.clone(), rnid.clone()),
                target,
                Some(ReferenceState::Stale),
            )],
        ),
    )
    .await;
    ready(&store);

    let res = store
        .rag_query(
            "stale-embed",
            &RagQueryOptions {
                wiki_id: Some(w),
                top_k: Some(5),
                expand: Some(ExpandMode::Parent),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    // Assert results were actually produced (the ref doc is retrieved by its
    // title), so this is not a vacuous pass over an empty list.
    let with_parent: Vec<_> = res.results.iter().filter(|r| r.parent.is_some()).collect();
    assert!(
        !with_parent.is_empty(),
        "a parent must be produced for the retrieved reference"
    );
    // Every expanded parent for a STALE embed must carry stale: true (§4.5.2).
    for r in with_parent {
        assert!(
            r.parent.as_ref().unwrap().stale,
            "STALE embed's parent must carry stale: true"
        );
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
///
/// H2 (adversarial pass): the `set_subsystems` write that used to sit here was
/// **inert** — after U3 the six flags are a read-time projection derived inside
/// `get_engine_status` (F16), so the hook's value is never read back and the
/// assertion could not fail. The write is therefore **deleted** (it had become a
/// no-op assertion), and this test now asserts the **derived** flags for the
/// capability state it actually wired: a `Ready` engine with NO index and NO
/// provider reports `store/graph/lexical == true`, `vector == false`,
/// `embedding == false`, `reranker == false` (the honest `Ready` vector, §5.8 /
/// F2 §9.1).
#[tokio::test]
async fn get_engine_status_ready_when_all_subsystems_up() {
    let store = Arc::new(Store::new());
    store.set_engine_state(EngineState::Ready);
    let status = store.get_engine_status().await;
    assert_eq!(status.state, EngineState::Ready);
    assert!(
        status.subsystems.store && status.subsystems.graph && status.subsystems.lexical,
        "the core legs are functional in every reachable state"
    );
    assert!(
        !status.subsystems.vector,
        "no index is wired ⇒ the derived vector flag must be false (not the hook's value)"
    );
    assert!(
        !status.subsystems.embedding,
        "no provider is wired ⇒ the derived embedding flag must be false"
    );
    assert!(
        !status.subsystems.reranker,
        "no reranker exists anywhere in src/ ⇒ false in every reachable state"
    );
}

/// State: `DEGRADED` when a non-core subsystem (the embedding provider) is down;
/// core store/graph/lexical still work (§4.6.1).
///
/// H2 (adversarial pass): the previous form wrote an inert
/// `set_subsystems({embedding:false})` mask and then asserted
/// `!status.subsystems.embedding` — which after U3 holds merely because **no
/// provider is wired**, so it could not detect a regression in the
/// DEGRADED/`embedding` derivation. This test now wires a genuinely-unavailable
/// provider (a local test provider whose `is_available()` is `false`) and
/// asserts the full honest vector for a no-index snapshot: `store/graph/lexical
/// == true`, `vector == false`, `reranker == false`, **`embedding == true`**
/// (the flag is the WIRED capability, §5.8 / §9.5.2 `P-IM-8` — a wired-then-
/// unreachable provider keeps the claim), plus `last_error.is_some()` as the
/// unreachability signal. The `set_subsystems` write is dropped.
#[tokio::test]
async fn get_engine_status_degraded_when_embedding_provider_down() {
    let store = Arc::new(Store::new());
    // A genuinely-unavailable provider is WIRED (the seam is `Some`) and the
    // engine state is `Degraded`: the flag reports the wired capability, while
    // `state`/`last_error` carry the unreachability.
    store.set_embedding_provider(Arc::new(StubProvider::unreachable()));
    store.set_engine_state(EngineState::Degraded);
    let status = store.get_engine_status().await;
    assert_eq!(status.state, EngineState::Degraded);
    assert!(
        status.subsystems.store && status.subsystems.graph && status.subsystems.lexical,
        "core store/graph/lexical still work while a non-core subsystem is down"
    );
    assert!(
        !status.subsystems.vector,
        "no index is built for a fresh store ⇒ the derived vector flag is false"
    );
    assert!(
        status.subsystems.embedding,
        "a provider IS wired (unreachable ≠ unwired) ⇒ the derived embedding flag is true; \
         the unreachability is reported by state/last_error, not by a false claim"
    );
    assert!(
        !status.subsystems.reranker,
        "no reranker exists ⇒ the derived reranker flag is false"
    );
    assert!(
        status.last_error.is_some(),
        "the DEGRADED state must carry its reason in last_error"
    );
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
/// multiple concurrent **vector-mode** `ragQuery` reads (the read side) race a
/// concurrent index rebuild (the immutable-snapshot swap). Each reader must
/// actually read the immutable vector snapshot and return the seeded vector-leg
/// hit (a coherent per-reader result) — flat-over-empty reads (the old vacuous
/// form) return nothing and fail the assertion.
///
/// RED today: `rag_query(mode: vector)` ignores the injected provider + seeded
/// index and returns a lexical (empty) list, so the readers surface no
/// vector-leg hit (a panic in the reader task → a join error).
#[tokio::test(flavor = "multi_thread")]
async fn concurrent_rag_query_reads_race_a_concurrent_index_rebuild() {
    use tokio::sync::Barrier;

    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    ready(&store);
    store.set_embedding_provider(Arc::new(StubProvider::hit(vec![1.0, 0.0])));

    // Seed a real doc in the wiki whose content does NOT lexically match the
    // query, then host its vector in the immutable snapshot (the read-side).
    let doc = new_doc(&store, &w, "vec-source").await;
    apply_graph(
        &store,
        &doc,
        wellformed_graph(
            vec![content_node(&doc.document_id, "v1", "unrelated")],
            vec![],
        ),
    )
    .await;
    let mut v0 = VectorIndex::default();
    v0.entries.insert(
        (doc.document_id.clone(), nid("v1"), FieldType::Full),
        vec![1.0, 0.0],
    );
    store.swap_snapshot(DerivedIndexes {
        lexical: None,
        vectors: Some(v0),
        epoch: 1,
    });
    let vkey = (doc.document_id.clone(), nid("v1"), FieldType::Full);

    let barrier = Arc::new(Barrier::new(9)); // 8 readers + 1 rebuild writer
    let mut readers = Vec::new();
    for _ in 0..8 {
        let s = store.clone();
        let b = barrier.clone();
        let wid = w.clone();
        let target = vkey.clone();
        readers.push(tokio::spawn(async move {
            b.wait().await;
            let res = s
                .rag_query(
                    "gamma",
                    &RagQueryOptions {
                        wiki_id: Some(wid),
                        top_k: Some(5),
                        mode: Some(QueryMode::Vector),
                        ..Default::default()
                    },
                )
                .await
                .expect("concurrent vector rag_query must return Ok(RagResult)");
            // The reader MUST have read the immutable vector snapshot and return
            // the seeded vector-leg hit (a coherent per-reader result).
            assert!(
                res.results
                    .iter()
                    .any(|r| r.document_id == target.0 && r.node_id == target.1),
                "reader must return the vector-leg hit from the immutable snapshot"
            );
        }));
    }
    // The concurrent index rebuild: swap a fresh snapshot on the epoch feed.
    let wstore = store.clone();
    let wb = barrier.clone();
    let rebuild = tokio::spawn(async move {
        wb.wait().await;
        for epoch in 1..=10u64 {
            let mut v1 = VectorIndex::default();
            v1.entries.insert(vkey.clone(), vec![1.0, 0.0]);
            wstore.swap_snapshot(DerivedIndexes {
                lexical: None,
                vectors: Some(v1),
                epoch,
            });
        }
    });

    // The 8 readers + 1 rebuild self-synchronize on the barrier, then the
    // readers issue genuinely-concurrent vector-mode `ragQuery` reads against
    // the immutable snapshot while the rebuild swipes the index. Both the
    // rebuild (independent) and the reader joins are collected; a reader that
    // read no vector-leg hit panics inside its task, surfacing as a join error.
    let _ = rebuild.await;
    for r in readers {
        r.await
            .expect("concurrent reader task must not crash on join");
    }
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
