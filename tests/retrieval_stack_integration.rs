//! §4.5.3 Retrieval-stack integration tests — written **RED-first** at the TDD
//! gate for the §4.5 unit.
//!
//! Every state and fail-state below is derived from the canonical behavior
//! contract alone (`docs/specs/gnosis.md` §4.5.3, §4.5.3a, §4.6.1, and §6) plus
//! the concurrency contract (IMMUTABLE-DERIVED-SNAPSHOT — the BM25 + multi-field
//! vector indexes live in the immutable `DerivedIndexes` snapshot, keyed by
//! `(documentId, nodeId, fieldType)`).
//!
//! ## RED set (this suite)
//!
//! `bm25_search`, `vector_search` and `rag_query` are COMPILING STUBS
//! (`unimplemented!()` → panic), so every behavior test here is RED. The
//! immutable-snapshot seeding (`swap_snapshot`) is the provided concurrency
//! vehicle and is not itself red — it is the test setup.
//!
//! ## TestWriter contract decisions (for the Implementer to match exactly)
//!
//! - **Embedding provider** is a test-injectable abstraction. The suite
//!   implements two `EmbeddingProvider`s: a deterministic in-memory mock (for
//!   exact/cosine assertions) and a **wiremock-mocked HTTP provider** (no live
//!   Ollama call — `MockServer` answers `POST /embed`). The `EmbeddingProvider`
//!   trait methods return boxed futures for dyn-compatibility.
//! - **Coarse-to-fine (§4.5.3a.3)** — binary-first-pass (Hamming on the
//!   `binary` field → candidate pool) then full cosine on `full`, `binaryFirstPass`
//!   opt-in; the `binary` index not built **degrades to the `full`-field search`
//!   (not an error)**; an invalid (`0`) `binaryCandidatePool` → `ValidationError`.
//! - **Compression on failure degrades gracefully to uncompressed** — the
//!   compressor's own failure is NOT a query failure (only a total failure that
//!   cannot degrade → `CompressionFailed`). The suite asserts the graceful
//!   no-failure of a compression-requested query.
//! - **Reranking** is implicit in the stack (the spec's `ragQuery` options have
//!   no rerank toggle); `RerankerUnavailable` (FS-16) surfaces when the reranker
//!   model is unavailable during the automatic two-stage rerank. Flagged as a
//!   boundary fail-state (no injectable seam on `RagStore` yet).

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use gnosis::{
    CompressionMode, CreateDocumentRequest, DerivedIndexes, Document, DocumentId, Edge, EdgeKind,
    EmbeddingProvider, FieldType, FirstPassOptions, Graph, MultiQueryOptions, Node, NodeId,
    NodeKind, QueryAuditFilters, QueryMode, RagQueryOptions, RagStore, Store, StoreError,
    UpdateDocumentRequest, VectorIndex, WikiId,
};

// ---------------------------------------------------------------------------
// Helpers (mirroring the other suites)
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
        fact_key: Some(node.to_string()),
        target: None,
    }
}

fn edge(kind: EdgeKind, from: (DocumentId, NodeId), to: (DocumentId, NodeId)) -> Edge {
    Edge {
        source: from,
        target: to,
        kind,
        state: None,
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
        ),
        edge(
            EdgeKind::DocEnd,
            (d.clone(), first),
            (d.clone(), nid("END")),
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

/// Seed a seed vector snapshot (`full`, optionally `binary`) into the store.
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

// ---------------------------------------------------------------------------
// Embedding-provider mocks
// ---------------------------------------------------------------------------

/// A deterministic in-memory provider: `embed` returns a fixed vector; the
/// availability is fixed. Tests override the response through these two
/// constructors.
struct MockProvider {
    available: bool,
    emit_error: bool,
}

impl MockProvider {
    fn ok() -> Self {
        MockProvider {
            available: true,
            emit_error: false,
        }
    }
    fn unreachable() -> Self {
        MockProvider {
            available: false,
            emit_error: true,
        }
    }
}

impl EmbeddingProvider for MockProvider {
    fn embed(
        &self,
        _text: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<f32>, StoreError>> + Send + '_>> {
        let emit = self.emit_error;
        Box::pin(async move {
            if emit {
                Err(StoreError::EmbeddingUnavailable)
            } else {
                Ok(vec![1.0, 0.0, 0.0])
            }
        })
    }
    fn is_available(&self) -> Pin<Box<dyn Future<Output = bool> + Send + '_>> {
        let a = self.available;
        Box::pin(async move { a })
    }
}

/// A **wiremock-mocked HTTP embedding provider** (no live Ollama call): `embed`
/// POSTs `/embed` to the wiremock `MockServer`. A 2xx carries the vector; a
/// non-2xx / transport error → `EmbeddingUnavailable`.
struct WiremockEmbeddingProvider {
    base: String,
    http: reqwest::Client,
}

impl EmbeddingProvider for WiremockEmbeddingProvider {
    fn embed(
        &self,
        text: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<f32>, StoreError>> + Send + '_>> {
        let url = format!("{}/embed", self.base);
        let http = self.http.clone();
        let body = serde_json::json!({ "text": text });
        Box::pin(async move {
            let resp = http
                .post(&url)
                .json(&body)
                .send()
                .await
                .map_err(|_| StoreError::EmbeddingUnavailable)?;
            if !resp.status().is_success() {
                return Err(StoreError::EmbeddingUnavailable);
            }
            resp.json::<Vec<f32>>()
                .await
                .map_err(|_| StoreError::EmbeddingUnavailable)
        })
    }
    fn is_available(&self) -> Pin<Box<dyn Future<Output = bool> + Send + '_>> {
        Box::pin(async { true })
    }
}

// ---------------------------------------------------------------------------
// §4.5.3 Lexical BM25 leg
// ---------------------------------------------------------------------------

/// State: `bm25_search` returns exact-match hits over `factKey`/`title`/`tags`/
/// node text. A query for a stored factKey finds it.
#[tokio::test]
async fn bm25_search_returns_exact_matches_over_fact_key_and_text() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "The Quantum Engine").await;
    update_graph(
        &store,
        &doc,
        vec![content_node(
            &doc.document_id,
            "factkey-n1",
            "quantum coherence term",
        )],
        vec![],
    )
    .await;

    let hits = store.bm25_search(&w, "quantum", 5).await.unwrap();
    assert!(!hits.is_empty(), "a lexical hit for the exact term");
    assert!(hits.iter().any(|r| r.node_id == nid("factkey-n1")));
}

/// Fail-state FS-15: `bm25_search` when the BM25 lexical index is not built →
/// `LexicalIndexUnavailable`.
#[tokio::test]
async fn bm25_search_lexical_index_not_built_is_unavailable() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    // Fresh store → no lexical snapshot built.
    let err = store.bm25_search(&w, "any", 5).await.unwrap_err();
    assert_eq!(err, StoreError::LexicalIndexUnavailable);
}

// ---------------------------------------------------------------------------
// §4.5.3 Vector leg + §4.5.3a multi-field coarse-to-fine
// ---------------------------------------------------------------------------

/// State: `vector_search` embeds the query via the provider and returns top-k
/// candidates from the vector snapshot ordered by cosine similarity.
#[tokio::test]
async fn vector_search_returns_top_k_by_cosine_similarity() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    seed_vectors(
        &store,
        vec![
            ((did("d1"), nid("n1")), vec![1.0, 0.0]),
            ((did("d2"), nid("n1")), vec![0.5, 0.5]),
        ],
    );
    let fp = FirstPassOptions {
        binary_first_pass: false,
        binary_candidate_pool: 0,
    };
    let hits = store
        .vector_search(&w, "query", 5, &MockProvider::ok(), &fp)
        .await
        .unwrap();
    assert_eq!(hits.len(), 2);
    // Most similar first.
    assert_eq!(hits[0].document_id, did("d1"));
}

/// Fail-state FS-13: a vector leg whose embedding provider is unreachable →
/// `EmbeddingUnavailable`.
#[tokio::test]
async fn vector_search_embedding_provider_unreachable_is_unavailable() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    seed_vectors(&store, vec![((did("d1"), nid("n1")), vec![1.0, 0.0])]);
    let fp = FirstPassOptions {
        binary_first_pass: false,
        binary_candidate_pool: 0,
    };
    let err = store
        .vector_search(&w, "q", 5, &MockProvider::unreachable(), &fp)
        .await
        .unwrap_err();
    assert_eq!(err, StoreError::EmbeddingUnavailable);
}

/// Fail-state FS-14: a vector leg whose vector index is not built →
/// `VectorIndexUnavailable`.
#[tokio::test]
async fn vector_search_vector_index_not_built_is_unavailable() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    // Fresh snapshot → no vector index.
    let fp = FirstPassOptions {
        binary_first_pass: false,
        binary_candidate_pool: 0,
    };
    let err = store
        .vector_search(&w, "q", 5, &MockProvider::ok(), &fp)
        .await
        .unwrap_err();
    assert_eq!(err, StoreError::VectorIndexUnavailable);
}

/// State (§4.5.3a.3): coarse-to-fine — `binaryFirstPass` uses the `binary` field
/// to select a candidate pool then full cosine on `full`. The candidate pool is
/// `binaryCandidatePool` (default 10× topK).
#[tokio::test]
async fn vector_search_binary_first_pass_narrows_pool_then_full_cosine() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    // Seed a multi-field vector index: full + binary for two nodes.
    let mut vi = VectorIndex::default();
    vi.entries
        .insert((did("d1"), nid("n1"), FieldType::Full), vec![1.0, 0.0]);
    vi.entries
        .insert((did("d1"), nid("n1"), FieldType::Binary), vec![1.0, 0.0]);
    vi.entries
        .insert((did("d2"), nid("n1"), FieldType::Full), vec![0.0, 1.0]);
    vi.entries
        .insert((did("d2"), nid("n1"), FieldType::Binary), vec![0.0, 1.0]);
    store.swap_snapshot(DerivedIndexes {
        lexical: None,
        vectors: Some(vi),
        epoch: 1,
    });

    let fp = FirstPassOptions {
        binary_first_pass: true,
        binary_candidate_pool: 10,
    };
    let hits = store
        .vector_search(&w, "q", 2, &MockProvider::ok(), &fp)
        .await
        .unwrap();
    assert_eq!(hits.len(), 2);
    // The full-field cosine orders d1 (most similar to [1,0,0]) first.
    assert_eq!(hits[0].document_id, did("d1"));
}

/// State (§4.5.3a.4): a `binaryFirstPass` when the `binary` field/index is not
/// built **degrades to the `full`-field search** (not an error).
#[tokio::test]
async fn vector_search_binary_first_pass_degrades_to_full_when_binary_unbuilt() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    // Only `full` fields are built (binary is optional/additive).
    seed_vectors(&store, vec![((did("d1"), nid("n1")), vec![1.0, 0.0])]);
    let fp = FirstPassOptions {
        binary_first_pass: true,
        binary_candidate_pool: 10,
    };
    let hits = store
        .vector_search(&w, "q", 5, &MockProvider::ok(), &fp)
        .await
        .unwrap();
    assert!(
        !hits.is_empty(),
        "degraded to the full-field search, not an error"
    );
}

/// Fail-state (§4.5.3a.4 / FS-3): an invalid `binaryCandidatePool` (`0`) →
/// `ValidationError`.
#[tokio::test]
async fn vector_search_binary_candidate_pool_zero_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    seed_vectors(&store, vec![((did("d1"), nid("n1")), vec![1.0, 0.0])]);
    let fp = FirstPassOptions {
        binary_first_pass: true,
        binary_candidate_pool: 0,
    };
    let err = store
        .vector_search(&w, "q", 5, &MockProvider::ok(), &fp)
        .await
        .unwrap_err();
    assert!(matches!(err, StoreError::ValidationError(_)));
}

/// A wiremock-mocked HTTP embedding provider answers `POST /embed`: the vector
/// leg honors the seam (no live Ollama call) — 2xx → candidates.
#[tokio::test]
async fn wiremock_http_embedding_provider_serves_the_vector_seam() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    server
        .register(
            Mock::given(method("POST"))
                .and(path("/embed"))
                .respond_with(ResponseTemplate::new(200).set_body_json(vec![1.0f32, 0.0])),
        )
        .await;
    let provider = WiremockEmbeddingProvider {
        base: server.uri(),
        http: reqwest::Client::new(),
    };

    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    seed_vectors(&store, vec![((did("d1"), nid("n1")), vec![1.0, 0.0])]);
    let fp = FirstPassOptions {
        binary_first_pass: false,
        binary_candidate_pool: 0,
    };
    // A 2xx embed → the provider returns a vector; a query over the mock HTTP
    // leg yields the candidates.
    let hits = store
        .vector_search(&w, "q", 5, &provider, &fp)
        .await
        .unwrap();
    assert!(!hits.is_empty());
}

/// Fail-state FS-13 via the wiremock seam: a non-2xx embed → `EmbeddingUnavailable`
/// (the wiremock server has no stub → 404). No live Ollama call is made.
#[tokio::test]
async fn wiremock_http_embedding_provider_404_is_embedding_unavailable() {
    let server = wiremock::MockServer::start().await; // no stub → 404 for POST /embed
    let provider = WiremockEmbeddingProvider {
        base: server.uri(),
        http: reqwest::Client::new(),
    };
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    seed_vectors(&store, vec![((did("d1"), nid("n1")), vec![1.0, 0.0])]);
    let fp = FirstPassOptions {
        binary_first_pass: false,
        binary_candidate_pool: 0,
    };
    let err = store
        .vector_search(&w, "q", 5, &provider, &fp)
        .await
        .unwrap_err();
    assert_eq!(err, StoreError::EmbeddingUnavailable);
}

// ---------------------------------------------------------------------------
// §4.5.3 Multi-query / compression / HyDE / rerank (via ragQuery — RED stub)
// ---------------------------------------------------------------------------

/// State: multi-query fan-out (`multiQuery.enabled`, `n` variants) merges by
/// stable `(documentId, nodeId)` identity (exact, not fuzzy) — no duplicates.
#[tokio::test]
async fn multi_query_merges_variants_by_stable_node_identity() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "doc").await;
    update_graph(
        &store,
        &doc,
        vec![content_node(&doc.document_id, "n1", "term")],
        vec![],
    )
    .await;
    store.set_engine_state(gnosis::EngineState::Ready);

    let res = store
        .rag_query(
            "term",
            &RagQueryOptions {
                wiki_id: Some(w),
                top_k: Some(10),
                mode: Some(QueryMode::Hybrid),
                multi_query: Some(MultiQueryOptions {
                    enabled: true,
                    n: 3,
                }),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    // Merged by stable node identity: each (documentId, nodeId) appears at most once.
    let mut seen = std::collections::HashSet::new();
    for r in &res.results {
        let key = (r.document_id.clone(), r.node_id.clone());
        assert!(
            seen.insert(key),
            "no duplicate node identity after multi-query merge"
        );
    }
}

/// State: compression `filter` mode keeps relevant context (and is not itself a
/// failure; a compressor failure degrades gracefully to uncompressed context).
#[tokio::test]
async fn compression_filter_mode_is_not_a_query_failure() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "doc").await;
    update_graph(
        &store,
        &doc,
        vec![content_node(&doc.document_id, "n1", "contextual term")],
        vec![],
    )
    .await;
    store.set_engine_state(gnosis::EngineState::Ready);

    // On compressor failure (graceful degrade) the query still succeeds.
    let res = store
        .rag_query(
            "contextual",
            &RagQueryOptions {
                wiki_id: Some(w),
                top_k: Some(5),
                compression: Some(CompressionMode::Filter),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert!(!res.results.is_empty());
}

/// State: compression `extract` mode (fact-value extraction) is Phase 2.
#[tokio::test]
async fn compression_extract_mode_runs() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    store.set_engine_state(gnosis::EngineState::Ready);
    let _ = store
        .rag_query(
            "q",
            &RagQueryOptions {
                wiki_id: Some(w),
                top_k: Some(5),
                compression: Some(CompressionMode::Extract),
                ..Default::default()
            },
        )
        .await;
}

/// State: compression `graph` mode (sub-structure compression) is Phase 3.
#[tokio::test]
async fn compression_graph_mode_runs() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    store.set_engine_state(gnosis::EngineState::Ready);
    let _ = store
        .rag_query(
            "q",
            &RagQueryOptions {
                wiki_id: Some(w),
                top_k: Some(5),
                compression: Some(CompressionMode::Graph),
                ..Default::default()
            },
        )
        .await;
}

/// State: HyDE is opt-in (`hyde: true`); a `vector`/`hybrid` query with HyDE
/// generates a hypothetical doc and routes its embedding through the vector leg.
#[tokio::test]
async fn hyde_opt_in_routes_hypothetical_through_vector_leg() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "doc").await;
    update_graph(
        &store,
        &doc,
        vec![content_node(&doc.document_id, "n1", "hypothetical")],
        vec![],
    )
    .await;
    store.set_engine_state(gnosis::EngineState::Ready);

    let res = store
        .rag_query(
            "hypothetical",
            &RagQueryOptions {
                wiki_id: Some(w),
                top_k: Some(5),
                mode: Some(QueryMode::Vector),
                hyde: Some(true),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert!(!res.results.is_empty());
}

/// State (§4.5.2 filters): a filter restricts which nodes/edges retrieval
/// considers (here, only `content` nodes via `nodeKind`).
#[tokio::test]
async fn rag_query_filters_restrict_retrieved_nodes_by_node_kind() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "doc").await;
    update_graph(
        &store,
        &doc,
        vec![content_node(&doc.document_id, "cnt", "term")],
        vec![],
    )
    .await;
    store.set_engine_state(gnosis::EngineState::Ready);

    let res = store
        .rag_query(
            "term",
            &RagQueryOptions {
                wiki_id: Some(w),
                top_k: Some(5),
                filters: Some(QueryAuditFilters {
                    node_kind: Some(NodeKind::Content),
                    edge_type: None,
                    target: None,
                    state: None,
                }),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert!(!res.results.is_empty());
}
