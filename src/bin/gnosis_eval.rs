//! §7.8 F6 — the `gnosis-eval` `[[bin]]`.
//!
//! Contract: `docs/specs/6-f6-eval-spec.md` §7. Reads a labeled corpus JSON (§8
//! schema), seeds a `Store` so each case's labeled relevant nodes are retrievable
//! by its `query` under its `mode`, runs `rag_query` per case, computes the four
//! metrics, and prints a per-case + aggregate report.
//!
//! CLI: `gnosis-eval [--live] <corpus.json>`. `--live` (or `GNOSIS_EVAL_LIVE=1`)
//! selects a real embedding provider; the default is a deterministic fake
//! provider (reproducible CI). The lowercase `mode` string is mapped to the
//! `QueryMode` variant MANUALLY (never deserialized directly into `QueryMode`).

use gnosis::{
    contextual_precision, contextual_recall, mrr_at_k, ndcg_at_k, CreateDocumentRequest,
    DerivedIndexes, DocumentId, Edge, EdgeKind, EmbeddingProvider, EngineState, FieldType, Graph,
    Node, NodeId, NodeKind, QueryMode, RagQueryOptions, RagResult, RagStore, Store, StoreError,
    UpdateDocumentRequest, VectorIndex, WikiId,
};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// §8 corpus schema — a JSON array of case objects.
#[derive(Deserialize)]
struct CorpusCase {
    query: String,
    wiki_id: String,
    mode: String,
    top_k: usize,
    relevant: Vec<(String, String)>,
}

/// §8 `mode` serde note: `QueryMode` has no `#[serde(rename_all)]` (PascalCase
/// serde), so the lowercase corpus string is mapped to the variant MANUALLY.
fn map_mode(s: &str) -> QueryMode {
    match s {
        "flat" => QueryMode::Flat,
        "graph" => QueryMode::Graph,
        "vector" => QueryMode::Vector,
        "hybrid" => QueryMode::Hybrid,
        other => panic!("unknown mode {other}"),
    }
}

/// A deterministic fake embedding provider (reproducible CI, no remote call).
/// Returns a fixed-dimension vector derived from the text's bytes.
struct FakeProvider;

impl EmbeddingProvider for FakeProvider {
    fn embed(
        &self,
        text: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<f32>, StoreError>> + Send + '_>> {
        let mut v = vec![0.0f32; 8];
        for (i, b) in text.bytes().enumerate() {
            v[i % 8] += b as f32;
        }
        Box::pin(async move { Ok(v) })
    }
    fn is_available(&self) -> Pin<Box<dyn Future<Output = bool> + Send + '_>> {
        Box::pin(async move { true })
    }
}

/// A well-formed Provident graph: exactly one doc-head (ROOT → first node) and
/// one doc-end (first node → END), plus the given content nodes.
fn wellformed_graph(nodes: Vec<Node>) -> Graph {
    let first = nodes[0].node_id.clone();
    let doc = nodes[0].document_id.clone();
    let edges = vec![
        Edge {
            source: (doc.clone(), NodeId("ROOT".to_string())),
            target: (doc.clone(), first.clone()),
            kind: EdgeKind::DocHead,
            state: None,
            cross_wiki: false,
            relation_type: None,
        },
        Edge {
            source: (doc.clone(), first),
            target: (doc.clone(), NodeId("END".to_string())),
            kind: EdgeKind::DocEnd,
            state: None,
            cross_wiki: false,
            relation_type: None,
        },
    ];
    Graph { nodes, edges }
}

fn content_node(doc: &DocumentId, node: &str, value: &str) -> Node {
    Node {
        document_id: doc.clone(),
        node_id: NodeId(node.to_string()),
        kind: NodeKind::Content,
        value: Some(value.to_string()),
        fact_key: None,
        target: None,
    }
}

/// Seed a `Store` so each case's labeled relevant nodes are retrievable by its
/// `query` under its `mode`. Returns a map from corpus `(doc-id, node-id)` to the
/// store's generated `(DocumentId, NodeId)` and a map from corpus `wiki_id` to the
/// store's generated `WikiId`, so the metrics and queries can use the store's keys.
async fn seed_store(
    cases: &[CorpusCase],
    live: bool,
) -> Result<
    (
        Store,
        HashMap<(String, String), (DocumentId, NodeId)>,
        HashMap<String, WikiId>,
    ),
    String,
> {
    let store = Store::new();
    store.set_engine_state(EngineState::Ready);
    if live {
        // Real provider: not wired here (a real provider is selected by the
        // operator's environment); fall back to the deterministic fake so the
        // tool always runs.
        store.set_embedding_provider(Arc::new(FakeProvider));
    } else {
        store.set_embedding_provider(Arc::new(FakeProvider));
    }

    let mut key_map: HashMap<(String, String), (DocumentId, NodeId)> = HashMap::new();
    let mut wiki_map: HashMap<String, WikiId> = HashMap::new();
    let mut vector_entries: Vec<((DocumentId, NodeId), Vec<f32>)> = Vec::new();

    for case in cases {
        let wiki_id = match wiki_map.get(&case.wiki_id) {
            Some(w) => w.clone(),
            None => {
                let w = store
                    .create_wiki(&case.wiki_id)
                    .await
                    .map_err(|e| format!("create_wiki failed: {e}"))?
                    .wiki_id;
                wiki_map.insert(case.wiki_id.clone(), w.clone());
                w
            }
        };
        for (doc_id, node_id) in &case.relevant {
            if key_map.contains_key(&(doc_id.clone(), node_id.clone())) {
                continue;
            }
            // Create a document whose content node carries the query text so the
            // lexical (flat) leg can surface it.
            let doc = store
                .create_document(
                    &wiki_id,
                    CreateDocumentRequest {
                        title: format!("doc {doc_id}"),
                        tags: None,
                        author: None,
                    },
                )
                .await
                .map_err(|e| format!("create_document failed: {e}"))?;
            let node = content_node(&doc.document_id, node_id, &case.query);
            let graph = wellformed_graph(vec![node]);
            let cur = store
                .get_document(&doc.document_id)
                .await
                .map_err(|e| format!("get_document failed: {e}"))?;
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
                .map_err(|e| format!("update_document failed: {e}"))?;
            key_map.insert(
                (doc_id.clone(), node_id.clone()),
                (doc.document_id.clone(), NodeId(node_id.clone())),
            );
            // Seed a vector entry for vector/hybrid modes.
            vector_entries.push((
                (doc.document_id.clone(), NodeId(node_id.clone())),
                vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            ));
        }
    }

    // Seed the vector index so vector/hybrid cases have a dense leg.
    if !vector_entries.is_empty() {
        let mut vi = VectorIndex::default();
        for ((d, n), v) in vector_entries {
            vi.entries.insert((d, n, FieldType::Full), v);
        }
        store.swap_snapshot(DerivedIndexes {
            lexical: None,
            vectors: Some(vi),
            epoch: 1,
        });
    }

    Ok((store, key_map, wiki_map))
}

/// Compute the four metrics over a `RagResult` and the case's relevant set
/// (translated to the store's keys), with `k = top_k`.
fn compute_metrics(
    result: &RagResult,
    case: &CorpusCase,
    key_map: &HashMap<(String, String), (DocumentId, NodeId)>,
) -> (f64, f64, f64, f64) {
    let relevant: HashSet<(DocumentId, NodeId)> = case
        .relevant
        .iter()
        .filter_map(|k| key_map.get(k).cloned())
        .collect();
    let k = case.top_k;
    (
        contextual_precision(&result.results, &relevant, k),
        contextual_recall(&result.results, &relevant, k),
        ndcg_at_k(&result.results, &relevant, k),
        mrr_at_k(&result.results, &relevant, k),
    )
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut live = std::env::var("GNOSIS_EVAL_LIVE")
        .map(|v| v == "1")
        .unwrap_or(false);
    let mut corpus_path: Option<String> = None;
    for arg in &args[1..] {
        if arg == "--live" {
            live = true;
        } else if corpus_path.is_none() {
            corpus_path = Some(arg.clone());
        } else {
            eprintln!("error: unexpected argument {arg}");
            std::process::exit(2);
        }
    }
    let corpus_path = match corpus_path {
        Some(p) => p,
        None => {
            eprintln!("usage: gnosis-eval [--live] <corpus.json>");
            std::process::exit(2);
        }
    };

    let raw = match std::fs::read_to_string(&corpus_path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: cannot read corpus {corpus_path}: {e}");
            std::process::exit(1);
        }
    };
    let cases: Vec<CorpusCase> = match serde_json::from_str(&raw) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: corpus parse failed: {e}");
            std::process::exit(1);
        }
    };

    let rt = tokio::runtime::Runtime::new().expect("failed to start tokio runtime");
    let (store, key_map, wiki_map) = match rt.block_on(seed_store(&cases, live)) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: store seeding failed: {e}");
            std::process::exit(1);
        }
    };

    println!("=== gnosis-eval report ===");
    let mut agg = [0.0f64; 4];
    let mut n = 0usize;
    for (idx, case) in cases.iter().enumerate() {
        let mode = map_mode(&case.mode);
        let wiki_id = wiki_map
            .get(&case.wiki_id)
            .cloned()
            .unwrap_or_else(|| WikiId(case.wiki_id.clone()));
        let options = RagQueryOptions {
            wiki_id: Some(wiki_id),
            top_k: Some(case.top_k as u64),
            mode: Some(mode),
            ..Default::default()
        };
        match rt.block_on(store.rag_query(&case.query, &options)) {
            Ok(result) => {
                let (cp, cr, ndcg, mrr) = compute_metrics(&result, case, &key_map);
                println!(
                    "case {}: query=\"{}\", wiki=\"{}\", mode={}, top_k={}",
                    idx + 1,
                    case.query,
                    case.wiki_id,
                    case.mode,
                    case.top_k
                );
                println!("  contextual_precision = {cp:.4}");
                println!("  contextual_recall    = {cr:.4}");
                println!("  ndcg_at_k            = {ndcg:.4}");
                println!("  mrr_at_k             = {mrr:.4}");
                agg[0] += cp;
                agg[1] += cr;
                agg[2] += ndcg;
                agg[3] += mrr;
                n += 1;
            }
            Err(e) => {
                println!("case {}: ERROR {e}", idx + 1);
            }
        }
    }
    if n > 0 {
        println!("aggregate (mean over {n} cases):");
        println!("  contextual_precision = {:.4}", agg[0] / n as f64);
        println!("  contextual_recall    = {:.4}", agg[1] / n as f64);
        println!("  ndcg_at_k            = {:.4}", agg[2] / n as f64);
        println!("  mrr_at_k             = {:.4}", agg[3] / n as f64);
    }
}
