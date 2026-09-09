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
    NodeKind, QueryAuditFilters, QueryMode, RagQueryOptions, RagResult, RagStore, Store,
    StoreError, UpdateDocumentRequest, VectorIndex, WikiId,
};

// ---------------------------------------------------------------------------
// Helpers (mirroring the other suites)
// ---------------------------------------------------------------------------

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

/// Create a real document in `w` with a single content node and return its
/// real `(documentId, nodeId)`. Vector tests MUST seed the vector index under
/// real documents that belong to the queried wiki — synthetic ids that map to no
/// document would break once `vector_search` is wiki-scoped (MEDIUM-5).
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

/// A **text-sensitive** deterministic provider that proves HyDE actually routes
/// the HyDE-*hypothetical* text through `embed` (de-vacuation of the weak
/// constant-vector mock, §4.5.3 re-audit LOW-F). It returns **distinct known
/// vectors** for the HyDE hypothetical (`"{query} relevant passage"`, the
/// engine's §4.5.3 deterministic stand-in) vs the plain query text. Same input →
/// same vector (deterministic). This makes `hyde:true` vs `hyde:false`
/// distinguishable: only `hyde:true` embeds the hypothetical text and therefore
/// surfaces the node whose vector equals the hypothetical's vector.
struct HydeSensitiveProvider;

impl EmbeddingProvider for HydeSensitiveProvider {
    fn embed(
        &self,
        text: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<f32>, StoreError>> + Send + '_>> {
        // The §4.5.3 HyDE stand-in the engine embeds under `hyde: true`.
        const HYPOTHETICAL: &str = "hypothetical relevant passage";
        let v: Vec<f32> = if text == HYPOTHETICAL {
            vec![1.0, 0.0]
        } else {
            vec![0.0, 1.0]
        };
        Box::pin(async move { Ok(v) })
    }
    fn is_available(&self) -> Pin<Box<dyn Future<Output = bool> + Send + '_>> {
        Box::pin(async { true })
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
/// candidates from the vector snapshot ordered by cosine similarity. The vectors
/// are seeded under REAL docs that belong to the queried wiki (MEDIUM-5).
#[tokio::test]
async fn vector_search_returns_top_k_by_cosine_similarity() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let (d1, n1) = new_content_doc(&store, &w, "d1", "a1", "alpha").await;
    let (d2, n2) = new_content_doc(&store, &w, "d2", "b1", "beta").await;
    seed_vectors(
        &store,
        vec![
            ((d1.clone(), n1.clone()), vec![1.0, 0.0]),
            ((d2, n2), vec![0.5, 0.5]),
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
    assert_eq!(hits[0].document_id, d1);
    assert_eq!(hits[0].node_id, n1);
}

/// Fail-state FS-13: a vector leg whose embedding provider is unreachable →
/// `EmbeddingUnavailable`. (Vector seeded under a real doc in the queried wiki.)
#[tokio::test]
async fn vector_search_embedding_provider_unreachable_is_unavailable() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let (d1, n1) = new_content_doc(&store, &w, "d1", "n1", "alpha").await;
    seed_vectors(&store, vec![((d1, n1), vec![1.0, 0.0])]);
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
    // A real doc so the wiki "has documents", but NO vector index is seeded.
    let _ = new_content_doc(&store, &w, "d0", "n0", "alpha").await;
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
    // Seed a multi-field vector index: full + binary for two real docs in `w`.
    let (d1, n1) = new_content_doc(&store, &w, "d1", "a1", "alpha").await;
    let (d2, n2) = new_content_doc(&store, &w, "d2", "b1", "beta").await;
    let mut vi = VectorIndex::default();
    vi.entries
        .insert((d1.clone(), n1.clone(), FieldType::Full), vec![1.0, 0.0]);
    vi.entries
        .insert((d1.clone(), n1.clone(), FieldType::Binary), vec![1.0, 0.0]);
    vi.entries
        .insert((d2.clone(), n2.clone(), FieldType::Full), vec![0.0, 1.0]);
    vi.entries
        .insert((d2.clone(), n2, FieldType::Binary), vec![0.0, 1.0]);
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
    assert_eq!(hits[0].document_id, d1);
}

/// State (§4.5.3a.4): a `binaryFirstPass` when the `binary` field/index is not
/// built **degrades to the `full`-field search** (not an error).
#[tokio::test]
async fn vector_search_binary_first_pass_degrades_to_full_when_binary_unbuilt() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    // Only `full` fields are built (binary is optional/additive) — seeded under
    // a real doc in `w`.
    let (d1, n1) = new_content_doc(&store, &w, "d1", "a1", "alpha").await;
    seed_vectors(&store, vec![((d1, n1), vec![1.0, 0.0])]);
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

/// State (§4.5.3a.3 / LOW-G): `binaryFirstPass: true` with
/// `binaryCandidatePool: None` must apply the spec's default **10× topK** as the
/// coarse first-pass candidate-pool cap. Seed more than 10×topK candidates whose
/// binary ranking places the true full-cosine best hit BEYOND the 10×topK
/// boundary; the default cap must bound the pool to 10×topK, narrowing that best
/// hit OUT before the second-pass full cosine.
/// RED today: `rag_query` maps `binaryCandidatePool: None` to `0`, and a `0`
/// candidate pool is treated as UNCAPPED (all candidates pass to the full
/// cosine), so the best full hit flows through — the spec default is not applied.
#[tokio::test]
async fn binary_candidate_pool_none_default_caps_pool_at_tenx_topk() {
    use gnosis::RagResult;

    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let top_k = 2u64;
    // DEFAULT_CAP (20) = 10× topK (topK = 2). Seed DEFAULT_CAP candidates ranked
    // best-by-binary (Hamming 0) — they exactly fill the 10×topK = 20 pool and
    // exhaust its budget. WORSE (2) further Hamming-1 candidates bring the total
    // to 22 (> 10×topK) and rank the true best at binary position 21 — BEYOND the
    // 20-wide pool boundary.
    const DEFAULT_CAP: usize = 20;
    const WORSE: usize = 2;

    let mut vi = VectorIndex::default();
    for i in 0..DEFAULT_CAP {
        let (d, n) = new_content_doc(
            &store,
            &w,
            &format!("b-{i}"),
            &format!("nb-{i}"),
            "unrelated",
        )
        .await;
        vi.entries
            .insert((d.clone(), n.clone(), FieldType::Full), vec![0.0, 1.0]);
        // Hamming 0 vs the binary code of the query vector [1,0,0] → top-ranked,
        // filling the 10×topK pool with the coarser (non-best) candidates.
        vi.entries
            .insert((d.clone(), n.clone(), FieldType::Binary), vec![1.0, 0.0]);
    }
    // The TRUE full-cosine best hit (its `full` vector IS the query vector), but
    // its binary code (Hamming 1) ranks it 21st — beyond the default 10×topK cap.
    let mut best_hit: Option<(DocumentId, NodeId)> = None;
    for i in 0..WORSE {
        let (d, n) = new_content_doc(
            &store,
            &w,
            &format!("w-{i}"),
            &format!("nw-{i}"),
            "unrelated",
        )
        .await;
        if i == 0 {
            vi.entries
                .insert((d.clone(), n.clone(), FieldType::Full), vec![1.0, 0.0]);
            best_hit = Some((d.clone(), n.clone()));
        } else {
            vi.entries
                .insert((d.clone(), n.clone(), FieldType::Full), vec![0.0, 1.0]);
        }
        // Hamming 1 vs the query binary code → outranked, beyond the 10×topK cap.
        vi.entries
            .insert((d.clone(), n.clone(), FieldType::Binary), vec![1.0, 1.0]);
    }
    let (best_doc, best_node) = best_hit.expect("seeded the full-cosine best");
    store.swap_snapshot(DerivedIndexes {
        lexical: None,
        vectors: Some(vi),
        epoch: 1,
    });
    store.set_embedding_provider(Arc::new(MockProvider::ok()));
    store.set_engine_state(gnosis::EngineState::Ready);

    let contains_best = |res: &RagResult| {
        res.results
            .iter()
            .any(|r| r.document_id == best_doc && r.node_id == best_node)
    };
    let opts = |binary_first_pass: bool, pool: Option<u64>| RagQueryOptions {
        wiki_id: Some(w.clone()),
        top_k: Some(top_k),
        mode: Some(QueryMode::Vector),
        binary_first_pass: Some(binary_first_pass),
        binary_candidate_pool: pool,
        ..Default::default()
    };

    // CONTROL: no coarse pass → the flat full cosine over ALL candidates surfaces
    // the true best hit, so its absence elsewhere is specifically the cap.
    let flat = store
        .rag_query("qq-unmatched", &opts(false, None))
        .await
        .unwrap();
    assert!(
        contains_best(&flat),
        "sanity: without a binary first pass the full-cosine best IS a reachable hit"
    );
    // CONTROL: an explicitly uncapped pool likewise keeps it.
    let uncapped = store
        .rag_query("qq-unmatched", &opts(true, Some(u64::MAX)))
        .await
        .unwrap();
    assert!(
        contains_best(&uncapped),
        "sanity: an explicitly uncapped candidate pool keeps the best hit"
    );

    // THE ASSERTION: `binaryCandidatePool: None` must cap the binary pool at
    // 10×topK, EXCLUDING the beyond-cap best hit.
    let capped = store
        .rag_query("qq-unmatched", &opts(true, None))
        .await
        .unwrap();
    assert!(
        !contains_best(&capped),
        "binaryCandidatePool: None must apply the spec default 10×topK cap and bound the \
         candidate pool — the best full hit lies beyond the cap and must be excluded \
         (it is currently treated as an uncapped pool)"
    );
}

/// Fail-state (§4.5.3a.4 / FS-3): an invalid `binaryCandidatePool` (`0`) →
/// `ValidationError`.
#[tokio::test]
async fn vector_search_binary_candidate_pool_zero_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let (d1, n1) = new_content_doc(&store, &w, "d1", "a1", "alpha").await;
    seed_vectors(&store, vec![((d1, n1), vec![1.0, 0.0])]);
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
    let (d1, n1) = new_content_doc(&store, &w, "d1", "a1", "alpha").await;
    seed_vectors(&store, vec![((d1, n1), vec![1.0, 0.0])]);
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
    let (d1, n1) = new_content_doc(&store, &w, "d1", "a1", "alpha").await;
    seed_vectors(&store, vec![((d1, n1), vec![1.0, 0.0])]);
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

/// State (MEDIUM-5, §4.5.3): `vector_search` is **wiki-scoped** — vectors
/// hosted under documents that belong to wiki A are NOT returned when querying
/// wiki B (no cross-wiki leakage).
/// RED today: `vector_search` ignores `_wiki_id`, so a wiki-B query returns the
/// wiki-A vectors.
#[tokio::test]
async fn vector_search_does_not_leak_vectors_across_wikis() {
    let store = Arc::new(Store::new());
    let wa = new_wiki(&store, "A").await;
    let wb = new_wiki(&store, "B").await;
    // A real doc in wiki A with a seeded vector; wiki B owns no vectors at all.
    let (da, na) = new_content_doc(&store, &wa, "d-a", "n1", "alpha").await;
    let _ = new_content_doc(&store, &wb, "d-b", "n1", "alpha").await;
    seed_vectors(&store, vec![((da, na), vec![1.0, 0.0])]);

    let fp = FirstPassOptions {
        binary_first_pass: false,
        binary_candidate_pool: 0,
    };
    let hits = store
        .vector_search(&wb, "q", 5, &MockProvider::ok(), &fp)
        .await
        .unwrap();
    assert!(
        hits.is_empty(),
        "wiki B has no own vectors — a wiki-scoped vector_search must not leak wiki A's vectors"
    );
}

/// State (HIGH-4, §4.5.3): `multiQuery: {enabled, n}` must actually fan the
/// query out and merge by stable `(documentId, nodeId)` identity — it must NOT be
/// silently ignored. Over a corpus where single-shot recall misses docs that
/// query-variant fan-out reaches, the enabled result's distinct-identity set is
/// a STRICT SUPERSET of the disabled single-shot result.
/// RED today: `rag_query` parses `multi_query` only for validation and never
/// fans out, so the enabled and disabled results are identical (not a strict
/// superset).
#[tokio::test]
async fn multi_query_fan_out_is_not_ignored() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    // Three related docs: single-shot recall for "quantum" only reaches d1; a
    // 3-variant fan-out (e.g. quantum / entanglement / coherence) reaches all.
    for (t, v) in [
        ("d1", "quantum entanglement"),
        ("d2", "entanglement coherence"),
        ("d3", "coherence physics"),
    ] {
        let _ = new_content_doc(&store, &w, t, &format!("n-{t}"), v).await;
    }
    store.set_engine_state(gnosis::EngineState::Ready);
    let ids = |res: &RagResult| -> std::collections::HashSet<(DocumentId, NodeId)> {
        res.results
            .iter()
            .map(|r| (r.document_id.clone(), r.node_id.clone()))
            .collect()
    };

    let base = store
        .rag_query(
            "quantum",
            &RagQueryOptions {
                wiki_id: Some(w.clone()),
                top_k: Some(10),
                mode: Some(QueryMode::Hybrid),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let fan = store
        .rag_query(
            "quantum",
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
    let base_ids = ids(&base);
    let fan_ids = ids(&fan);
    assert!(
        base_ids.is_subset(&fan_ids) && base_ids.len() < fan_ids.len(),
        "multi-query fan-out must merge N variant results into a strictly larger \
         distinct-identity set than the single-shot query (it is currently ignored)"
    );
}

/// State (HIGH-4, §4.5.3): compression `filter` mode is a **post-retrieval
/// binary keep/drop** — it must change the returned snippet set vs the
/// uncompressed result. It is currently silently ignored.
/// RED today: `rag_query` parses `compression` but never applies it, so the
/// Filter result is byte-for-byte identical to the None result.
#[tokio::test]
async fn compression_filter_changes_the_snippet_set() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    // d1: a tight, on-topic snippet. d2: a low-relevance context whose query
    // token is diluted by filler — a query-aware binary filter keeps d1 and
    // drops/compresses d2, so the snippet set differs from uncompressed.
    let _ = new_content_doc(&store, &w, "d1", "n1", "term").await;
    let _ = new_content_doc(
        &store,
        &w,
        "d2",
        "n2",
        "term diluted by a very long run of irrelevant filler words that a query-aware filter should drop",
    )
    .await;
    store.set_engine_state(gnosis::EngineState::Ready);
    let snips = |res: &RagResult| -> Vec<String> {
        res.results.iter().map(|r| r.snippet.clone()).collect()
    };

    let none = store
        .rag_query(
            "term",
            &RagQueryOptions {
                wiki_id: Some(w.clone()),
                top_k: Some(5),
                compression: None,
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let filtered = store
        .rag_query(
            "term",
            &RagQueryOptions {
                wiki_id: Some(w),
                top_k: Some(5),
                compression: Some(CompressionMode::Filter),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_ne!(
        snips(&none),
        snips(&filtered),
        "filter compression must change the returned snippet set (it is currently ignored)"
    );
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

/// State (HIGH-4, §4.5.3, de-vacuated LOW-F): HyDE is opt-in (`hyde: true`); a
/// `vector` query with HyDE embeds the **hypothetical** text and routes it
/// through the vector leg. Using a *text-sensitive* provider, this only passes
/// when the hypothetical text actually reached `embed` (an `embed` of the raw
/// query, or any constant-vector mock, would NOT surface the seeded hit).
///
/// GREEN today: the engine routes the hypothetical through `embed` under
/// `hyde: true` (§4.5.3 vector leg). A `hyde: false` comparison shows plain
/// vector mode does NOT return the hypothetical-seeded hit.
#[tokio::test]
async fn hyde_opt_in_routes_hypothetical_through_vector_leg() {
    use gnosis::RagResultItem;

    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    // The doc whose `full` vector equals the *hypothetical* vector — reachable
    // ONLY if the hypothetical text is embedded. (Its content does not contain
    // the query term, so plain lexical recall cannot reach it.)
    let (hypo_doc, hypo_node) = new_content_doc(&store, &w, "d-hypo", "n1", "unrelated-one").await;
    // Three docs whose `full` vector matches the *plain query* vector: under
    // `hyde:false` they fill top_k with cosine ~1, pushing the hypothetical doc
    // (cosine 0 vs the query vector) off the top-k.
    let (o1, a1) = new_content_doc(&store, &w, "d-o1", "a1", "unrelated-1").await;
    let (o2, b1) = new_content_doc(&store, &w, "d-o2", "b1", "unrelated-2").await;
    let (o3, c1) = new_content_doc(&store, &w, "d-o3", "c1", "unrelated-3").await;

    let mut vi = VectorIndex::default();
    vi.entries.insert(
        (hypo_doc.clone(), hypo_node.clone(), FieldType::Full),
        vec![1.0, 0.0],
    );
    vi.entries
        .insert((o1.clone(), a1.clone(), FieldType::Full), vec![0.0, 1.0]);
    vi.entries
        .insert((o2.clone(), b1.clone(), FieldType::Full), vec![0.0, 1.0]);
    vi.entries
        .insert((o3.clone(), c1.clone(), FieldType::Full), vec![0.0, 1.0]);
    store.swap_snapshot(DerivedIndexes {
        lexical: None,
        vectors: Some(vi),
        epoch: 1,
    });

    // Prove the provider is text-sensitive (different text → different vector);
    // otherwise the `hyde:false` exclusion below would be vacuous.
    let probe = HydeSensitiveProvider;
    assert_eq!(
        probe.embed("hypothetical").await.unwrap(),
        vec![0.0, 1.0],
        "the probe must map the plain query text to its distinct vector"
    );
    assert_eq!(
        probe.embed("hypothetical relevant passage").await.unwrap(),
        vec![1.0, 0.0],
        "the probe must map the HyDE hypothetical text to its distinct vector"
    );

    store.set_embedding_provider(Arc::new(HydeSensitiveProvider));
    store.set_engine_state(gnosis::EngineState::Ready);

    let on_hypo = |r: &RagResultItem| r.document_id == hypo_doc && r.node_id == hypo_node;

    // `hyde: true` → the hypothetical text ("hypothetical relevant passage")
    // reaches `embed`, producing the hypothetical vector → the seeded hit.
    let hyde = store
        .rag_query(
            "hypothetical",
            &RagQueryOptions {
                wiki_id: Some(w.clone()),
                top_k: Some(3),
                mode: Some(QueryMode::Vector),
                hyde: Some(true),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert!(
        hyde.results.iter().any(on_hypo),
        "HyDE must embed the hypothetical text and route it through the vector leg \
         (the hypothetical-seeded hit is absent — hyde/vector is not routing the hypothetical)"
    );

    // `hyde: false` (plain vector mode) → embeds the raw query text → the
    // hypothetical-seeded hit (cosine 0 vs the query vector) is NOT returned.
    let plain = store
        .rag_query(
            "hypothetical",
            &RagQueryOptions {
                wiki_id: Some(w),
                top_k: Some(3),
                mode: Some(QueryMode::Vector),
                hyde: Some(false),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert!(
        !plain.results.iter().any(on_hypo),
        "plain vector mode (hyde:false) must NOT return the hypothetical-seeded hit — \
         a constant-vector mock (or an ignored hyde flag) would make hyde:true and \
         hyde:false indistinguishable"
    );
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
