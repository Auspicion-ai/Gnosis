//! §4.2 knowledge graph — EXECUTED property layer (PBT-gate artifact #2, TestWriter).
//!
//! Implements every row of `docs/specs/4-2-graph-property-register.md` as one
//! `#[test]` per row (P-IM-1, P-IM-2, P-SM-1, P-SM-2, P-TP-1, P-TP-2, P-TP-3,
//! P-TP-4). Each row runs a tiny deterministic property harness (hand-rolled
//! **SplitMix64**, pinned seed — no external crate, no proptest) and implements
//! its Observable-as-property **faithfully** — a genuinely-broken row FAILS its
//! `#[test]` with the retained minimal counterexamples (stop-after-5).
//!
//! ---------------------------------------------------------------------------
//! PINNED SEED (recorded in the file header and report):
//!    BASE_SEED = 0x6D797A4CF0D09E5D
//! Every row derives its own instance seed as `BASE_SEED ^ ROW_TAG` (a fixed
//! per-row constant), so generation is byte-for-byte reproducible run-to-run
//! without cross-row correlation.
//! ---------------------------------------------------------------------------
//!
//! Budget: ≤100 generated cases per row; the per-row budgets sum to exactly
//! **400** total (P-IM-1=60, P-IM-2=55, P-SM-1=60, P-SM-2=45, P-TP-1=40,
//! P-TP-2=35, P-TP-3=55, P-TP-4=50). stop-after-5: a row that starts failing
//! stops as soon as it has retained ≤5 minimal counterexamples; a HELD row runs
//! its full budget.
//!
//! All register rows are tagged GREEN (invariants TRUE of the current
//! implementation per the register). The empirical run below records held/broken
//! per row. Every generator stays on the **legal** side of the §4.2 guards so a
//! row never drives a `StoreError` (fail-states are excluded from this register;
//! §4.2.7.5 cascade is exercised as a state transition, not an error).
//!
//! Fixture helpers are reused from `tests/graph_integration.rs` and the exact
//! register API notes. No `src/` file, existing test file, or
//! `docs/specs/gnosis.md` is modified — only `tests/props_graph.rs` is added.

use std::collections::HashMap;
use std::sync::Arc;

use gnosis::{
    CreateDocumentRequest, DeclareCommunityOptions, DocumentId, Edge, EdgeKind, GetTriplesFilter,
    Graph, MergeFactsOptions, Node, NodeId, NodeKind, RagStore, ReferenceState,
    ResolveEntitiesOptions, ResolveOptions, ResolvedFact, Store, Triple, TripleDirection,
    UpdateDocumentRequest, WikiId,
};

// ---------------------------------------------------------------------------
// Deterministic PRNG (SplitMix64) + fixture helpers
// ---------------------------------------------------------------------------

/// Deterministic SplitMix64 — fixed seed, no external crate.
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        SplitMix64 { state: seed }
    }
    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// Uniform in `0..n` (n > 0). Bounded-budget generator; modulo bias is fine.
    fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n.max(1)
    }
    fn pick(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            self.below(n as u64) as usize
        }
    }
    /// Uniform in `lo..=hi` inclusive (lo <= hi). Range generator.
    fn range(&mut self, lo: u64, hi: u64) -> u64 {
        lo + self.next_u64() % (hi - lo + 1)
    }
}

/// Order-insensitive multiset equality of two edge slices (Edge has no Ord/Hash).
fn same_edge_multiset(a: &[Edge], b: &[Edge]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut used = vec![false; b.len()];
    for e in a {
        let mut found = false;
        for (i, be) in b.iter().enumerate() {
            if !used[i] && be == e {
                used[i] = true;
                found = true;
                break;
            }
            if found {
                break;
            }
        }
        if !found {
            return false;
        }
    }
    true
}

/// Order-insensitive set equality of two triple slices (by full value equality).
fn same_triple_set(a: &[Triple], b: &[Triple]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut used = vec![false; b.len()];
    for ta in a {
        let mut found = false;
        for (i, tb) in b.iter().enumerate() {
            if !used[i] && tb == ta {
                used[i] = true;
                found = true;
                break;
            }
            if found {
                break;
            }
        }
        if !found {
            return false;
        }
    }
    true
}

/// Does a triple touch `nq` in the given direction?
fn touches(t: &Triple, nq: &(DocumentId, NodeId), dir: TripleDirection) -> bool {
    match dir {
        TripleDirection::Out => t.subject == *nq,
        TripleDirection::In => t.object == *nq,
        TripleDirection::Both => t.subject == *nq || t.object == *nq,
    }
}

/// A `reference`/`fact`/`embed` reference edge predicate — replicated from §4.4.2.
fn is_reference_edge(e: &Edge, source_nid: &NodeId, target_nid: &NodeId) -> bool {
    matches!(
        e.kind,
        EdgeKind::Link | EdgeKind::Embed | EdgeKind::Crosslink
    ) && e.source.1 == *source_nid
        && e.target.1 == *target_nid
}
/// Case-insensitive parse of a reference-state string (replicating §4.4.2).
fn parse_state(s: &str) -> ReferenceState {
    match s.to_ascii_uppercase().as_str() {
        "FRESH" => ReferenceState::Fresh,
        "STALE" => ReferenceState::Stale,
        "RESOLVED" => ReferenceState::Resolved,
        _ => ReferenceState::Broken,
    }
}

// Fixture helpers (reused from `tests/graph_integration.rs`).
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
async fn new_wiki(store: &Store, name: &str) -> WikiId {
    store.create_wiki(name).await.unwrap().wiki_id
}
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
/// A fresh store + wiki + a content-only document with the given node names.
async fn fresh_doc(names: &[&str]) -> (Arc<Store>, WikiId, gnosis::Document) {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = fresh_doc_in(&store, &w, names).await;
    (store, w, doc)
}
async fn fresh_doc_in(store: &Store, w: &WikiId, names: &[&str]) -> gnosis::Document {
    let doc = new_doc(store, w, "doc").await;
    apply_graph(store, &doc, content_only_graph(&doc.document_id, names)).await;
    doc
}

/// A `doc` with nodes n1..n4 + a reference node ref1 (→ n4) + doc-head/child/end
/// (and optionally a self-loop n1→n1). Used by P-SM-1.
fn adjacency_probe_graph(doc: &DocumentId, self_loop: bool) -> Graph {
    let ref1 = reference_node(doc, "ref1", (doc.clone(), nid("n4")));
    let mut edges = vec![
        edge(
            EdgeKind::DocHead,
            (doc.clone(), nid("ROOT")),
            (doc.clone(), nid("n1")),
            None,
        ),
        edge(
            EdgeKind::DocChild,
            (doc.clone(), nid("n1")),
            (doc.clone(), nid("n2")),
            None,
        ),
        edge(
            EdgeKind::DocChild,
            (doc.clone(), nid("n2")),
            (doc.clone(), nid("n3")),
            None,
        ),
        edge(
            EdgeKind::DocEnd,
            (doc.clone(), nid("n3")),
            (doc.clone(), nid("END")),
            None,
        ),
        edge(
            EdgeKind::Link,
            (doc.clone(), ref1.node_id.clone()),
            (doc.clone(), nid("n4")),
            Some(ReferenceState::Resolved),
        ),
    ];
    if self_loop {
        edges.push(edge(
            EdgeKind::DocChild,
            (doc.clone(), nid("n1")),
            (doc.clone(), nid("n1")),
            None,
        ));
    }
    Graph {
        nodes: vec![
            content_node(doc, "n1", "v1"),
            content_node(doc, "n2", "v2"),
            content_node(doc, "n3", "v3"),
            content_node(doc, "n4", "v4"),
            ref1,
        ],
        edges,
    }
}

// ---------------------------------------------------------------------------
// P-IM-1 — Triple return-shape consistency  (IM, `gen_triple`, tag [GREEN])
// ---------------------------------------------------------------------------
//
// For every generated legal `addTriple(s,r,o,w)` the returned `Triple` satisfies
// `subject == s`, `object == o`, `relation == r`, `relation_type == r`, and
// `!created_at.is_empty()`. Boundaries probed: same-doc and cross-doc endpoints,
// self-loop, single-node doc, short/long/whitespace/Unicode relations, and a
// same-endpoint/collision relation (return shape under a second distinct relation).
const ROW_IM1_TAG: u64 = 0x00_00_00_00_00_00_00_01;
const ROW_IM1_BUDGET: u64 = 60;
const RELS: &[&str] = &[
    "depends_on",
    "causes",
    "x",
    "a fairly long relation type string that exceeds typical length for coverage",
    "  padded_with_whitespace  ",
    "über_verknüpft",
    "中関係子",
];

#[tokio::test]
async fn p_im1_triple_return_shape() {
    let mut cex: Vec<String> = Vec::new();
    let mut rng = SplitMix64::new(0x6D797A4CF0D09E5D ^ ROW_IM1_TAG);

    for budget in 1..=ROW_IM1_BUDGET {
        if cex.len() >= 5 {
            break;
        }
        let (store, w, doc_a) = fresh_doc(&["n1", "n2", "n3", "n4"]).await;
        let did_a = doc_a.document_id.clone();
        // Cross-doc boundary: a second document in the same wiki.
        let doc_b = fresh_doc_in(&store, &w, &["m1", "m2"]).await;
        let did_b = doc_b.document_id.clone();

        let single_node = budget % 7 == 0;
        let s_nodes: Vec<(DocumentId, NodeId)> = if single_node {
            vec![(did_a.clone(), nid("n1"))]
        } else {
            let mut v = Vec::new();
            for name in ["n1", "n2", "n3", "n4"] {
                v.push((did_a.clone(), nid(name)));
            }
            if budget % 5 == 0 {
                v.push((did_b.clone(), nid("m1")));
            }
            v
        };
        let o_nodes: Vec<(DocumentId, NodeId)> = if single_node {
            vec![(did_a.clone(), nid("n1"))]
        } else {
            let mut v = Vec::new();
            for name in ["n1", "n2", "n3", "n4"] {
                v.push((did_a.clone(), nid(name)));
            }
            if budget % 5 == 0 {
                v.push((did_b.clone(), nid("m2")));
            }
            v
        };
        let subj = s_nodes[rng.pick(s_nodes.len())].clone();
        let obj = if budget % 6 == 0 {
            subj.clone() // self-loop
        } else {
            o_nodes[rng.pick(o_nodes.len())].clone()
        };
        let rel = RELS[rng.pick(RELS.len())];

        let t = store.add_triple(&subj, rel, &obj, &w).await;
        let triple = match t {
            Ok(v) => v,
            Err(e) => {
                if cex.len() < 5 {
                    cex.push(format!(
                        "case {budget}: add_triple({subj:?}, {rel:?}, {obj:?}) -> unexpected {e:?}"
                    ));
                }
                continue;
            }
        };
        let mut bad = |msg: String| -> Vec<String> {
            if cex.len() < 5 {
                cex.push(format!(
                    "case {budget}: triple({subj:?},{rel:?},{obj:?}) — {msg}"
                ));
            }
            cex.clone()
        };
        if triple.subject != subj {
            bad(format!(
                "subject expected {subj:?}, got {:?}",
                triple.subject
            ));
        }
        if triple.object != obj {
            bad(format!("object expected {obj:?}, got {:?}", triple.object));
        }
        if triple.relation != rel {
            bad(format!(
                "relation expected {rel:?}, got {:?}",
                triple.relation
            ));
        }
        if triple.relation_type != rel {
            bad(format!(
                "relation_type expected {rel:?}, got {:?}",
                triple.relation_type
            ));
        }
        if triple.created_at.is_empty() {
            bad("created_at is empty".into());
        }

        // Adversarial collision: a second triple on the SAME endpoints, a
        // DIFFERENT relation — returned Triple must mirror the second relation.
        if budget % 4 == 0 && !single_node {
            let rel2 = RELS[(rng.pick(RELS.len()) + 1) % RELS.len()];
            if let Ok(t2) = store.add_triple(&subj, rel2, &obj, &w).await {
                if t2.relation != rel2 || t2.relation_type != rel2 {
                    bad(format!(
                        "collision: expected relation {rel2:?}, got {}/{}",
                        t2.relation, t2.relation_type
                    ));
                }
            }
        }
    }
    assert!(
        cex.is_empty(),
        "P-IM-1 BROKEN — {} counterexample(s):\n  - {}",
        cex.len(),
        cex.join("\n  - ")
    );
}

// ---------------------------------------------------------------------------
// P-IM-2 — relationType filter honored  (IM, `gen_filtered_triples`, tag [GREEN])
// ---------------------------------------------------------------------------
//
// For the generated triple set and a queried node `n` with `relation_type =
// Some(rt)` and direction d, the returned triple set is exactly the added
// triples that touch `n` per direction d and have relation == rt (soundness +
// completeness). A filter relation_type that matches nothing returns empty (no
// error).
const ROW_IM2_TAG: u64 = 0x00_00_00_00_00_00_00_02;
const ROW_IM2_BUDGET: u64 = 55;

#[tokio::test]
async fn p_im2_relation_type_filter() {
    let mut cex: Vec<String> = Vec::new();
    let mut rng = SplitMix64::new(0x6D797A4CF0D09E5D ^ ROW_IM2_TAG);

    for budget in 1..=ROW_IM2_BUDGET {
        if cex.len() >= 5 {
            break;
        }
        let (store, w, doc) = fresh_doc(&["n1", "n2", "n3", "n4"]).await;
        let did = doc.document_id.clone();
        let nq: (DocumentId, NodeId) = (did.clone(), nid("n2"));

        let mut added: Vec<Triple> = Vec::new();
        let mut rels_on_nq: Vec<&str> = Vec::new();

        // Outgoing (subject == nq) triples.
        let out_rels = ["rel_a", "rel_b"];
        for (i, rel) in out_rels.iter().enumerate() {
            let obj = nid(if i % 2 == 0 { "n3" } else { "n4" });
            let t = store
                .add_triple(&nq, rel, &(did.clone(), obj), &w)
                .await
                .unwrap();
            added.push(t);
            if !rels_on_nq.contains(rel) {
                rels_on_nq.push(rel);
            }
        }
        // Incoming (object == nq) triples.
        let in_rels = ["rel_a", "rel_c"];
        for rel in in_rels {
            let s = nid("n1");
            let t = store
                .add_triple(&(did.clone(), s), rel, &nq, &w)
                .await
                .unwrap();
            if !added.iter().any(|x| x == &t) {
                added.push(t);
            }
            if !rels_on_nq.contains(&rel) {
                rels_on_nq.push(rel);
            }
        }
        // Unrelated triples that don't touch nq.
        store
            .add_triple(
                &(did.clone(), nid("n3")),
                "rel_c",
                &(did.clone(), nid("n4")),
                &w,
            )
            .await
            .unwrap();

        let dir = [
            TripleDirection::Out,
            TripleDirection::In,
            TripleDirection::Both,
        ][rng.pick(3)];
        // ~1/5 of the time use a filter matching zero triples on nq.
        let rt: &str = if budget % 5 == 0 {
            "zzz_no_such_type"
        } else {
            rels_on_nq[rng.pick(rels_on_nq.len())]
        };

        let opts = GetTriplesFilter {
            direction: dir,
            relation_type: Some(rt.to_string()),
            wiki_id: w.clone(),
        };
        let result = match store.get_triples(&nq, &opts).await {
            Ok(v) => v,
            Err(e) => {
                if cex.len() < 5 {
                    cex.push(format!(
                        "case {budget}: get_triples(nq, dir={dir:?}, rt={rt:?}) -> unexpected {e:?}"
                    ));
                }
                continue;
            }
        };
        let expected: Vec<Triple> = added
            .iter()
            .filter(|t| t.relation == rt && touches(t, &nq, dir))
            .cloned()
            .collect();

        // Soundness: every returned triple matches the filter and touches nq.
        for t in &result {
            if t.relation != rt && cex.len() < 5 {
                cex.push(format!(
                    "case {budget}: returned triple {:?} has relation {:?} != filter {rt:?}",
                    (format!("{:?}", t.subject), format!("{:?}", t.object)),
                    t.relation
                ));
            }
            if !touches(t, &nq, dir) && cex.len() < 5 {
                cex.push(format!(
                    "case {budget}: returned triple ({:?} -> {:?}) does not touch nq per {dir:?}",
                    t.subject, t.object
                ));
            }
        }
        // Completeness: exactly the modeled set.
        if !same_triple_set(&result, &expected) && cex.len() < 5 {
            cex.push(format!(
                "case {budget}: dir={dir:?} rt={rt:?} got {} triples, expected {}",
                result.len(),
                expected.len()
            ));
        }
    }
    assert!(
        cex.is_empty(),
        "P-IM-2 BROKEN — {} counterexample(s):\n  - {}",
        cex.len(),
        cex.join("\n  - ")
    );
}

// ---------------------------------------------------------------------------
// P-SM-1 — Adjacency consistency (+ §4.2.7.5 cascade)  (SM, `gen_adjacency`)
// ---------------------------------------------------------------------------
//
// For every node v of the document, `edges_from(v)` == exactly `{e: e.source==v}`
// and `edges_to(v)` == exactly `{e: e.target==v}` (computed from the
// authoritative `edges_for_document`). A node removed by `updateDocument`
// cascades any `Relation` edge out of BOTH adjacency surfaces (soundness, no
// residue on retained siblings).
const ROW_SM1_TAG: u64 = 0x00_00_00_00_00_00_00_03;
const ROW_SM1_BUDGET: u64 = 60;

#[tokio::test]
async fn p_sm1_adjacency_consistency_and_cascade() {
    let mut cex: Vec<String> = Vec::new();
    let mut rng = SplitMix64::new(0x6D797A4CF0D09E5D ^ ROW_SM1_TAG);

    for budget in 1..=ROW_SM1_BUDGET {
        if cex.len() >= 5 {
            break;
        }
        let (store, w, doc) = fresh_doc(&["n1", "n2", "n3", "n4"]).await;
        let did = doc.document_id.clone();
        let self_loop = rng.pick(5) == 0;
        apply_graph(&store, &doc, adjacency_probe_graph(&did, self_loop)).await;
        // Relation edges owned by the graph (GR-OWNS-RELATION-AND-MERGE).
        store
            .add_triple(
                &(did.clone(), nid("n1")),
                "depends_on",
                &(did.clone(), nid("n2")),
                &w,
            )
            .await
            .unwrap();
        store
            .add_triple(
                &(did.clone(), nid("n4")),
                "causes",
                &(did.clone(), nid("n1")),
                &w,
            )
            .await
            .unwrap();

        let full = store.edges_for_document(&did).await.unwrap();
        let cur = store.get_document(&did).await.unwrap();
        for node in &cur.graph.nodes {
            let nodeid = node.node_id.clone();
            let from = store.edges_from(&did, &nodeid).await.unwrap();
            let exp_from: Vec<Edge> = full
                .iter()
                .filter(|e| e.source == (did.clone(), nodeid.clone()))
                .cloned()
                .collect();
            if !same_edge_multiset(&from, &exp_from) && cex.len() < 5 {
                cex.push(format!(
                    "case {budget}: edges_from({nodeid:?}) len {} != expected {}",
                    from.len(),
                    exp_from.len()
                ));
            }
            let to = store.edges_to(&did, &nodeid).await.unwrap();
            let exp_to: Vec<Edge> = full
                .iter()
                .filter(|e| e.target == (did.clone(), nodeid.clone()))
                .cloned()
                .collect();
            if !same_edge_multiset(&to, &exp_to) && cex.len() < 5 {
                cex.push(format!(
                    "case {budget}: edges_to({nodeid:?}) len {} != expected {}",
                    to.len(),
                    exp_to.len()
                ));
            }
        }

        // Cascade: remove a triple endpoint node; the Relation edge must vanish
        // from both adjacency surfaces on the retained siblings (§4.2.7.5).
        if rng.pick(3) == 0 {
            apply_graph(&store, &doc, content_only_graph(&did, &["n1", "n3"])).await;
            let full2 = store.edges_for_document(&did).await.unwrap();
            if full2.iter().any(|e| e.kind == EdgeKind::Relation) && cex.len() < 5 {
                cex.push(format!(
                    "case {budget}: Relation edge survived node removal (no cascade)"
                ));
            }
            let cur2 = store.get_document(&did).await.unwrap();
            for node in &cur2.graph.nodes {
                let nodeid = node.node_id.clone();
                let from = store.edges_from(&did, &nodeid).await.unwrap();
                let exp_from: Vec<Edge> = full2
                    .iter()
                    .filter(|e| e.source == (did.clone(), nodeid.clone()))
                    .cloned()
                    .collect();
                if !same_edge_multiset(&from, &exp_from) && cex.len() < 5 {
                    cex.push(format!(
                        "case {budget}: post-removal edges_from({nodeid:?}) mismatch"
                    ));
                }
                let to = store.edges_to(&did, &nodeid).await.unwrap();
                let exp_to: Vec<Edge> = full2
                    .iter()
                    .filter(|e| e.target == (did.clone(), nodeid.clone()))
                    .cloned()
                    .collect();
                if !same_edge_multiset(&to, &exp_to) && cex.len() < 5 {
                    cex.push(format!(
                        "case {budget}: post-removal edges_to({nodeid:?}) mismatch"
                    ));
                }
                if (from.iter().any(|e| e.target.1 == nid("n2"))
                    || to.iter().any(|e| e.source.1 == nid("n2")))
                    && cex.len() < 5
                {
                    cex.push(format!(
                        "case {budget}: adjacency of {nodeid:?} still references removed node n2"
                    ));
                }
            }
        }
    }
    assert!(
        cex.is_empty(),
        "P-SM-1 BROKEN — {} counterexample(s):\n  - {}",
        cex.len(),
        cex.join("\n  - ")
    );
}

// ---------------------------------------------------------------------------
// P-SM-2 — Reference-graph acyclicity / topological order  (SM, `gen_reference_chain`)
// ---------------------------------------------------------------------------
//
// A SUCCESSFUL `resolveReferences` returns a topologically ordered,
// non-redundant list: `n == len`, the `resolution_order`s are exactly `{1..=n}`
// in the result sequence, and for every resolved `Reference` node whose `target`
// is also resolved, `order(target) < order(node)`. Generator emits only acyclic
// subgraphs (a cycle is a StoreError, not an invariant).
const ROW_SM2_TAG: u64 = 0x00_00_00_00_00_00_00_04;
const ROW_SM2_BUDGET: u64 = 45;

#[tokio::test]
async fn p_sm2_topological_resolution() {
    let mut cex: Vec<String> = Vec::new();
    let mut rng = SplitMix64::new(0x6D797A4CF0D09E5D ^ ROW_SM2_TAG);

    for budget in 1..=ROW_SM2_BUDGET {
        if cex.len() >= 5 {
            break;
        }
        let (store, w, doc) = fresh_doc(&["c_root"]).await;
        let did = doc.document_id.clone();
        let shape = rng.pick(5); // 0 leaf, 1 depth1, 2 depth2, 3 diamond, 4 content-root
        let (graph, roots, max_hops): (Graph, Vec<(DocumentId, NodeId)>, u64) = match shape {
            0 => (
                Graph {
                    nodes: vec![fact_node(&did, "f", "f", "leaf")],
                    edges: vec![
                        edge(
                            EdgeKind::DocHead,
                            (did.clone(), nid("ROOT")),
                            (did.clone(), nid("f")),
                            None,
                        ),
                        edge(
                            EdgeKind::DocEnd,
                            (did.clone(), nid("f")),
                            (did.clone(), nid("END")),
                            None,
                        ),
                    ],
                },
                vec![(did.clone(), nid("f"))],
                1,
            ),
            1 => (
                Graph {
                    nodes: vec![
                        fact_node(&did, "f", "f", "leaf"),
                        reference_node(&did, "r1", (did.clone(), nid("f"))),
                    ],
                    edges: vec![
                        edge(
                            EdgeKind::DocHead,
                            (did.clone(), nid("ROOT")),
                            (did.clone(), nid("f")),
                            None,
                        ),
                        edge(
                            EdgeKind::DocEnd,
                            (did.clone(), nid("f")),
                            (did.clone(), nid("END")),
                            None,
                        ),
                        edge(
                            EdgeKind::Link,
                            (did.clone(), nid("r1")),
                            (did.clone(), nid("f")),
                            Some(ReferenceState::Resolved),
                        ),
                    ],
                },
                vec![(did.clone(), nid("r1"))],
                1,
            ),
            2 => (
                Graph {
                    nodes: vec![
                        fact_node(&did, "f", "f", "leaf"),
                        reference_node(&did, "r1", (did.clone(), nid("f"))),
                        reference_node(&did, "r2", (did.clone(), nid("r1"))),
                    ],
                    edges: vec![
                        edge(
                            EdgeKind::DocHead,
                            (did.clone(), nid("ROOT")),
                            (did.clone(), nid("f")),
                            None,
                        ),
                        edge(
                            EdgeKind::DocEnd,
                            (did.clone(), nid("f")),
                            (did.clone(), nid("END")),
                            None,
                        ),
                        edge(
                            EdgeKind::Link,
                            (did.clone(), nid("r1")),
                            (did.clone(), nid("f")),
                            Some(ReferenceState::Resolved),
                        ),
                        edge(
                            EdgeKind::Link,
                            (did.clone(), nid("r2")),
                            (did.clone(), nid("r1")),
                            Some(ReferenceState::Resolved),
                        ),
                    ],
                },
                vec![(did.clone(), nid("r2"))],
                2,
            ),
            3 => (
                Graph {
                    nodes: vec![
                        fact_node(&did, "f", "f", "leaf"),
                        reference_node(&did, "r1", (did.clone(), nid("f"))),
                        reference_node(&did, "r2", (did.clone(), nid("r1"))),
                        reference_node(&did, "r_side", (did.clone(), nid("f"))),
                    ],
                    edges: vec![
                        edge(
                            EdgeKind::DocHead,
                            (did.clone(), nid("ROOT")),
                            (did.clone(), nid("f")),
                            None,
                        ),
                        edge(
                            EdgeKind::DocEnd,
                            (did.clone(), nid("f")),
                            (did.clone(), nid("END")),
                            None,
                        ),
                        edge(
                            EdgeKind::Link,
                            (did.clone(), nid("r1")),
                            (did.clone(), nid("f")),
                            Some(ReferenceState::Resolved),
                        ),
                        edge(
                            EdgeKind::Link,
                            (did.clone(), nid("r2")),
                            (did.clone(), nid("r1")),
                            Some(ReferenceState::Resolved),
                        ),
                        edge(
                            EdgeKind::Link,
                            (did.clone(), nid("r_side")),
                            (did.clone(), nid("f")),
                            Some(ReferenceState::Resolved),
                        ),
                    ],
                },
                vec![(did.clone(), nid("r2")), (did.clone(), nid("r_side"))],
                2,
            ),
            _ => (
                Graph {
                    nodes: vec![content_node(&did, "c_root", "rootval")],
                    edges: vec![
                        edge(
                            EdgeKind::DocHead,
                            (did.clone(), nid("ROOT")),
                            (did.clone(), nid("c_root")),
                            None,
                        ),
                        edge(
                            EdgeKind::DocEnd,
                            (did.clone(), nid("c_root")),
                            (did.clone(), nid("END")),
                            None,
                        ),
                    ],
                },
                vec![(did.clone(), nid("c_root"))],
                1,
            ),
        };
        apply_graph(&store, &doc, graph).await;

        // Occasionally probe the empty-root boundary (successful → empty result).
        let roots_use: Vec<(DocumentId, NodeId)> =
            if rng.pick(11) == 0 { Vec::new() } else { roots };
        let opts = ResolveOptions {
            wiki_id: w.clone(),
            max_hops,
        };
        let resolved = match store.resolve_references(&roots_use, &opts).await {
            Ok(v) => v,
            Err(e) => {
                if cex.len() < 5 {
                    cex.push(format!(
                        "case {budget}: acyclic resolve_references(shape {shape}) -> unexpected {e:?}"
                    ));
                }
                continue;
            }
        };

        // Non-redundancy: each resolved node appears exactly once (n == len).
        let mut seen: Vec<(DocumentId, NodeId)> = Vec::new();
        for rf in &resolved {
            if seen.contains(&rf.node) && cex.len() < 5 {
                cex.push(format!(
                    "case {budget}: node {:?} resolved more than once (redundant)",
                    rf.node
                ));
            }
            seen.push(rf.node.clone());
        }
        // resolution_order values are exactly {1..=n}.
        let n = resolved.len() as u64;
        let mut orders: Vec<u64> = resolved.iter().map(|rf| rf.resolution_order).collect();
        orders.sort_unstable();
        let expected_orders: Vec<u64> = (1..=n).collect();
        if orders != expected_orders && cex.len() < 5 {
            cex.push(format!(
                "case {budget}: resolution orders {:?} != 1..={n}",
                orders
            ));
        }
        // Dependencies resolve before dependents.
        let doc_cur = store.get_document(&did).await.unwrap();
        let order_of = |node: &(DocumentId, NodeId)| -> Option<u64> {
            resolved
                .iter()
                .find(|rf| &rf.node == node)
                .map(|rf| rf.resolution_order)
        };
        for rf in &resolved {
            let node_def = doc_cur.graph.nodes.iter().find(|n| n.node_id == rf.node.1);
            let Some(node_def) = node_def else { continue };
            if node_def.kind != NodeKind::Reference {
                continue;
            }
            let Some(target) = &node_def.target else {
                continue;
            };
            if let Some(to) = order_of(target) {
                if !(to < rf.resolution_order) && cex.len() < 5 {
                    cex.push(format!(
                            "case {budget}: dependent {:?} order {} < dependency {:?} order {} (not topological)",
                            rf.node, rf.resolution_order, target, to
                        ));
                }
            }
        }
        if let Some(root) = roots_use.first() {
            // The resolved value of a Content root is its own value.
            let node_def = doc_cur.graph.nodes.iter().find(|n| n.node_id == root.1);
            if let Some(ndef) = node_def {
                if ndef.kind == NodeKind::Content {
                    if let Some(rf) = resolved.iter().find(|rf| &rf.node == root) {
                        if rf.value != ndef.value.clone().unwrap_or_default() && cex.len() < 5 {
                            cex.push(format!(
                                "case {budget}: content root value {:?} != node value",
                                rf.value
                            ));
                        }
                    }
                }
            }
        }
    }
    assert!(
        cex.is_empty(),
        "P-SM-2 BROKEN — {} counterexample(s):\n  - {}",
        cex.len(),
        cex.join("\n  - ")
    );
}

// ---------------------------------------------------------------------------
// P-TP-1 — setReferenceState targeted no-clobber  (TP, `gen_reference_state`)
// ---------------------------------------------------------------------------
//
// Only the matched reference edge(s)' `state` changes; the document's node set and
// every other edge (identity AND state) are preserved, and the revision advances
// by exactly one. Repeated `setReferenceState` on the SAME matched edge is
// last-write-wins with every other edge untouched.
const ROW_TP1_TAG: u64 = 0x00_00_00_00_00_00_00_05;
const ROW_TP1_BUDGET: u64 = 40;
const STATES: &[&str] = &["FRESH", "STALE", "RESOLVED", "BROKEN"];

#[tokio::test]
async fn p_tp1_set_reference_state_no_clobber() {
    let mut cex: Vec<String> = Vec::new();
    let mut rng = SplitMix64::new(0x6D797A4CF0D09E5D ^ ROW_TP1_TAG);

    for budget in 1..=ROW_TP1_BUDGET {
        if cex.len() >= 5 {
            break;
        }
        let (store, _w, doc) = fresh_doc(&["n1"]).await;
        let did = doc.document_id.clone();
        let ref2_kind = if budget % 3 == 0 {
            EdgeKind::Embed
        } else {
            EdgeKind::Link
        };
        let two_out = budget % 4 == 0; // same source ref1, two different targets
        let ref1 = reference_node(&did, "ref1", (did.clone(), nid("tgt1")));
        let ref2 = reference_node(&did, "ref2", (did.clone(), nid("tgt2")));
        let mut edges = vec![
            edge(
                EdgeKind::DocHead,
                (did.clone(), nid("ROOT")),
                (did.clone(), nid("n1")),
                None,
            ),
            edge(
                EdgeKind::DocEnd,
                (did.clone(), nid("n1")),
                (did.clone(), nid("END")),
                None,
            ),
            edge(
                EdgeKind::Link,
                (did.clone(), ref1.node_id.clone()),
                (did.clone(), nid("tgt1")),
                Some(ReferenceState::Resolved),
            ),
            edge(
                ref2_kind,
                (did.clone(), ref2.node_id.clone()),
                (did.clone(), nid("tgt2")),
                Some(ReferenceState::Stale),
            ),
        ];
        if two_out {
            edges.push(edge(
                EdgeKind::Link,
                (did.clone(), ref1.node_id.clone()),
                (did.clone(), nid("tgt1b")),
                None,
            ));
        }
        apply_graph(
            &store,
            &doc,
            Graph {
                nodes: vec![content_node(&did, "n1", "v"), ref1, ref2],
                edges,
            },
        )
        .await;

        // Match ref1 -> tgt1 (a Link edge) by (source.1, target.1).
        let src = nid("ref1");
        let tgt = nid("tgt1");
        let state_str = STATES[rng.pick(STATES.len())];

        let g_before = store.get_document(&did).await.unwrap().graph;
        let rev_before = store.get_document(&did).await.unwrap().revision;
        let updated = match store.set_reference_state(&did, &src, &tgt, state_str).await {
            Ok(e) => e,
            Err(e) => {
                if cex.len() < 5 {
                    cex.push(format!(
                        "case {budget}: set_reference_state({src:?},{tgt:?},{state_str:?}) -> {e:?}"
                    ));
                }
                continue;
            }
        };
        let new_state = parse_state(state_str);
        if updated.state != Some(new_state) && cex.len() < 5 {
            cex.push(format!(
                "case {budget}: returned edge state {:?} != {new_state:?}",
                updated.state
            ));
        }
        let g_after = store.get_document(&did).await.unwrap().graph;
        let rev_after = store.get_document(&did).await.unwrap().revision;

        // Nodes preserved.
        if g_after.nodes != g_before.nodes && cex.len() < 5 {
            cex.push(format!(
                "case {budget}: node set changed by setReferenceState (clobber)"
            ));
        }
        // Only the matched reference edge(s) change state; everything else is
        // byte-for-byte preserved.
        let matched = |e: &Edge| is_reference_edge(e, &src, &tgt);
        let expected_edges: Vec<Edge> = g_before
            .edges
            .iter()
            .map(|e| {
                if matched(e) {
                    Edge {
                        state: Some(new_state),
                        ..e.clone()
                    }
                } else {
                    e.clone()
                }
            })
            .collect();
        let got_edges = &g_after.edges;
        if !same_edge_multiset(got_edges, &expected_edges) && cex.len() < 5 {
            cex.push(format!(
                "case {budget}: reference-state mutation changed more than the matched edge(s)"
            ));
        }
        // Revision advances by exactly one.
        if rev_after != rev_before + 1 && cex.len() < 5 {
            cex.push(format!(
                "case {budget}: revision {rev_before} -> {rev_after} (expected +1)"
            ));
        }

        // Repeated setReferenceState on the SAME edge: last write wins, other
        // edges untouched, revision advances by exactly one again.
        if budget % 3 == 0 {
            let g_before2 = store.get_document(&did).await.unwrap().graph;
            let rev_before2 = store.get_document(&did).await.unwrap().revision;
            let state_str2 = STATES[(rng.pick(STATES.len()) + 1) % STATES.len()];
            let new_state2 = parse_state(state_str2);
            if store
                .set_reference_state(&did, &src, &tgt, state_str2)
                .await
                .is_ok()
            {
                let g_after2 = store.get_document(&did).await.unwrap().graph;
                let rev_after2 = store.get_document(&did).await.unwrap().revision;
                if rev_after2 != rev_before2 + 1 && cex.len() < 5 {
                    cex.push(format!(
                            "case {budget}: second revision {rev_before2} -> {rev_after2} (expected +1)"
                        ));
                }
                let matched2 = |e: &Edge| is_reference_edge(e, &src, &tgt);
                let expected2: Vec<Edge> = g_before2
                    .edges
                    .iter()
                    .map(|e| {
                        if matched2(e) {
                            Edge {
                                state: Some(new_state2),
                                ..e.clone()
                            }
                        } else {
                            e.clone()
                        }
                    })
                    .collect();
                if !same_edge_multiset(&g_after2.edges, &expected2) && cex.len() < 5 {
                    cex.push(format!(
                        "case {budget}: repeated setReferenceState clobbered unrelated edges"
                    ));
                }
            }
        }
    }
    assert!(
        cex.is_empty(),
        "P-TP-1 BROKEN — {} counterexample(s):\n  - {}",
        cex.len(),
        cex.join("\n  - ")
    );
}

// ---------------------------------------------------------------------------
// P-TP-2 — resolveEntities idempotent + convergent  (TP, `gen_entities`)
// ---------------------------------------------------------------------------
//
// Re-applying the SAME `(entity_ids, canonical_id)` leaves the durable
// alias→canonical mapping unchanged: a resolved alias never flips canonical
// under repetition, and the two `ResolutionResult`s are equal.
const ROW_TP2_TAG: u64 = 0x00_00_00_00_00_00_00_06;
const ROW_TP2_BUDGET: u64 = 35;

#[tokio::test]
async fn p_tp2_resolve_entities_idempotent() {
    let mut cex: Vec<String> = Vec::new();
    let mut rng = SplitMix64::new(0x6D797A4CF0D09E5D ^ ROW_TP2_TAG);

    for budget in 1..=ROW_TP2_BUDGET {
        if cex.len() >= 5 {
            break;
        }
        // e1..e4 host the entity set under test; e5/e6 are FRESH nodes reserved
        // for the disjoint "unrelated" resolution so it can never re-alias a
        // recorded alias (an overlapping re-resolution is a separate legal
        // operation, not the invariant under test).
        let (store, w, doc) = fresh_doc(&["e1", "e2", "e3", "e4", "e5", "e6"]).await;
        let did = doc.document_id.clone();
        let all = [
            (did.clone(), nid("e1")),
            (did.clone(), nid("e2")),
            (did.clone(), nid("e3")),
            (did.clone(), nid("e4")),
        ];
        let n_entities = 1 + rng.range(1, 3) as usize; // 2..=4
        let idx0 = rng.pick(all.len());
        let mut ids: Vec<(DocumentId, NodeId)> = Vec::new();
        let mut k = idx0;
        while ids.len() < n_entities.min(all.len()) {
            if !ids.contains(&all[k]) {
                ids.push(all[k].clone());
            }
            k = (k + 1) % all.len();
        }
        ids.sort_by_key(|(_, n)| n.0.clone());
        // Explicit canonical ~3/4 of the time (sometimes not first in ids);
        // canonical_id: None otherwise (deterministic first-entity choice).
        let explicit = budget % 4 != 0;
        let canonical: (DocumentId, NodeId) = if explicit {
            ids[rng.pick(ids.len())].clone()
        } else {
            ids[0].clone()
        };
        let opts = ResolveEntitiesOptions {
            canonical_id: if explicit {
                Some(canonical.clone())
            } else {
                None
            },
            wiki_id: w.clone(),
        };
        let opts_none = ResolveEntitiesOptions {
            canonical_id: None,
            wiki_id: w.clone(),
        };

        let r1 = match store.resolve_entities(&ids, &opts).await {
            Ok(v) => v,
            Err(e) => {
                if cex.len() < 5 {
                    cex.push(format!("case {budget}: resolve_entities -> {e:?}"));
                }
                continue;
            }
        };
        if explicit && r1.canonical_id != canonical && cex.len() < 5 {
            cex.push(format!(
                "case {budget}: explicitly-chosen canonical {:?} not honored (got {:?})",
                canonical, r1.canonical_id
            ));
        }
        // Snapshot the durable alias→canonical mapping for every recorded alias.
        let mut snapshot: Vec<((DocumentId, NodeId), (DocumentId, NodeId))> = Vec::new();
        for a in &r1.aliases {
            let canon = store
                .entity_alias_canonical(&a.alias)
                .await
                .unwrap()
                .unwrap_or(a.canonical.clone());
            snapshot.push((a.alias.clone(), canon));
        }

        // Re-apply the identical call → equal result, mapping unchanged.
        let r2 = store.resolve_entities(&ids, &opts).await.unwrap();
        if r2 != r1 && cex.len() < 5 {
            cex.push(format!(
                "case {budget}: re-applied resolveEntities returned a different result"
            ));
        }
        let mut stable = true;
        for (alias, canon) in &snapshot {
            let now = store.entity_alias_canonical(alias).await.unwrap();
            if now != Some(canon.clone()) {
                stable = false;
            }
        }
        if !stable && cex.len() < 5 {
            cex.push(format!(
                "case {budget}: a resolved alias flipped its canonical under identical re-apply"
            ));
        }

        // Adversarial: a re-run after an intervening UNRELATED resolution — on a
        // DISJOINT set of never-resolved nodes (e5, e6) — must not flip any of
        // the recorded aliases.
        let unrelated: Vec<(DocumentId, NodeId)> =
            vec![(did.clone(), nid("e5")), (did.clone(), nid("e6"))];
        let _ = store.resolve_entities(&unrelated, &opts_none).await;
        for (alias, canon) in &snapshot {
            let now = store.entity_alias_canonical(alias).await.unwrap();
            if now != Some(canon.clone()) && cex.len() < 5 {
                cex.push(format!(
                    "case {budget}: alias {:?} flipped under an unrelated resolution",
                    alias
                ));
            }
        }
    }
    assert!(
        cex.is_empty(),
        "P-TP-2 BROKEN — {} counterexample(s):\n  - {}",
        cex.len(),
        cex.join("\n  - ")
    );
}

// ---------------------------------------------------------------------------
// P-TP-3 — mergeFacts idempotent / union-stable  (TP, `gen_facts`)
// ---------------------------------------------------------------------------
//
// `mergeFacts(keys, {canonical_key: c})`; then `getFact(w, c)` after one application
// vs. after two (and after merging `keys ∪ {c}`) returns the identical `value`,
// and its `citations` set equals the deduped union of the merged facts' citation
// sets in every read. Generator emits only all-equal-`value` fact sets (a
// `ConflictError` is a fail-state, excluded).
const ROW_TP3_TAG: u64 = 0x00_00_00_00_00_00_00_07;
const ROW_TP3_BUDGET: u64 = 55;

fn same_citation_set(a: &[(DocumentId, NodeId)], b: &[(DocumentId, NodeId)]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().all(|c| b.contains(c))
}

#[tokio::test]
async fn p_tp3_merge_facts_union_stable() {
    let mut cex: Vec<String> = Vec::new();
    let mut rng = SplitMix64::new(0x6D797A4CF0D09E5D ^ ROW_TP3_TAG);
    let node_pool: &[&str] = &["n1", "n2", "n3", "n4", "n5"];

    for budget in 1..=ROW_TP3_BUDGET {
        if cex.len() >= 5 {
            break;
        }
        let (store, w, doc) = fresh_doc(node_pool).await;
        let did = doc.document_id.clone();
        let n_keys = rng.range(1, 3) as usize; // 1..=3
        let mut keys: Vec<String> = Vec::new();
        let canonical_key = format!("k{}", rng.range(1, 1000));
        keys.push(canonical_key.clone());
        while keys.len() < n_keys {
            let k = format!("k{}_{}", budget, keys.len());
            if !keys.contains(&k) {
                keys.push(k);
            }
        }
        // Create each fact with a real citation-node set; value all-equal.
        for k in &keys {
            let n_cites = rng.range(1, 3) as usize;
            let mut cites: Vec<(DocumentId, NodeId)> = Vec::new();
            let mut seen_cnodes = 0usize;
            while cites.len() < n_cites.min(node_pool.len()) {
                let c = node_pool[rng.pick(node_pool.len())];
                let cnode = (did.clone(), nid(c));
                if !cites.contains(&cnode) {
                    cites.push(cnode);
                }
                seen_cnodes += 1;
                if seen_cnodes > 3 {
                    break;
                }
            }
            store
                .create_fact(&w, &did, k, "same-value", &cites)
                .await
                .unwrap();
        }
        // Expected union = deduped union across the created facts.
        let mut expected_union: Vec<(DocumentId, NodeId)> = Vec::new();
        for k in &keys {
            let f = store.get_fact(&w, k).await.unwrap();
            for c in f.citations {
                if !expected_union.contains(&c) {
                    expected_union.push(c);
                }
            }
        }

        let opts = MergeFactsOptions {
            canonical_key: canonical_key.clone(),
            wiki_id: w.clone(),
        };
        // Application 1.
        let m1 = match store.merge_facts(&keys, &opts).await {
            Ok(v) => v,
            Err(e) => {
                if cex.len() < 5 {
                    cex.push(format!(
                        "case {budget}: merge_facts(first) on equal-value facts -> {e:?}"
                    ));
                }
                continue;
            }
        };
        if m1.value != "same-value" && cex.len() < 5 {
            cex.push(format!(
                "case {budget}: canonical value not preserved after merge"
            ));
        }
        let g1 = store.get_fact(&w, &canonical_key).await.unwrap();
        if !same_citation_set(&g1.citations, &expected_union) && cex.len() < 5 {
            cex.push(format!(
                "case {budget}: persisted union lacks/carries citations ({} vs {} expected)",
                g1.citations.len(),
                expected_union.len()
            ));
        }
        // Application 2 (identical).
        let m2 = store.merge_facts(&keys, &opts).await.unwrap();
        if m2.value != "same-value" && cex.len() < 5 {
            cex.push(format!("case {budget}: value drifted on re-merge"));
        }
        let g2 = store.get_fact(&w, &canonical_key).await.unwrap();
        if !same_citation_set(&g2.citations, &expected_union) && cex.len() < 5 {
            cex.push(format!(
                "case {budget}: union drifted on re-merge ({} vs {})",
                g2.citations.len(),
                expected_union.len()
            ));
        }
        // Re-merge on `keys ∪ {canonical}` — still union-stable (no drift/dupes).
        let mut keys_plus = keys.clone();
        if !keys_plus.contains(&canonical_key) {
            keys_plus.push(canonical_key.clone());
        }
        let m3 = store.merge_facts(&keys_plus, &opts).await.unwrap();
        if m3.value != "same-value" && cex.len() < 5 {
            cex.push(format!(
                "case {budget}: value changed on keys∪{{canonical}} re-merge"
            ));
        }
        let g3 = store.get_fact(&w, &canonical_key).await.unwrap();
        if !same_citation_set(&g3.citations, &expected_union) && cex.len() < 5 {
            cex.push(format!(
                "case {budget}: union changed on keys∪{{canonical}} ({} vs {})",
                g3.citations.len(),
                expected_union.len()
            ));
        }
    }
    assert!(
        cex.is_empty(),
        "P-TP-3 BROKEN — {} counterexample(s):\n  - {}",
        cex.len(),
        cex.join("\n  - ")
    );
}

// ---------------------------------------------------------------------------
// P-TP-4 — Manual-override authority  (TP, `gen_community`)
// ---------------------------------------------------------------------------
//
// A manually declared community's `summary` is never overwritten by a
// member-touching operation; it changes only via an explicit
// `updateCommunitySummary`. Members may span one or two documents.
const ROW_TP4_TAG: u64 = 0x00_00_00_00_00_00_00_08;
const ROW_TP4_BUDGET: u64 = 50;

#[tokio::test]
async fn p_tp4_community_summary_manual_override() {
    let mut cex: Vec<String> = Vec::new();
    let mut rng = SplitMix64::new(0x6D797A4CF0D09E5D ^ ROW_TP4_TAG);

    for budget in 1..=ROW_TP4_BUDGET {
        if cex.len() >= 5 {
            break;
        }
        let (store, w, doc_a) = fresh_doc(&["n1", "n2", "n3"]).await;
        let did_a = doc_a.document_id.clone();
        let doc_b = fresh_doc_in(&store, &w, &["m1", "m2"]).await;
        let did_b = doc_b.document_id.clone();

        let two_doc = rng.pick(4) == 0;
        let mut members: Vec<(DocumentId, NodeId)> = Vec::new();
        members.push((did_a.clone(), nid("n1")));
        if two_doc {
            members.push((did_b.clone(), nid("m1")));
        } else if rng.pick(3) == 0 {
            members.push((did_a.clone(), nid("n2")));
        }
        // Occasionally probe a duplicate member id (dedup keeps members = set).
        if rng.pick(7) == 0 {
            members.push((did_a.clone(), nid("n1")));
        }

        let manual = format!("manual summary for case {budget}");
        let declared = store
            .declare_community(
                &members,
                &DeclareCommunityOptions {
                    summary: manual.clone(),
                    wiki_id: w.clone(),
                },
            )
            .await
            .unwrap();
        let cid = declared.community_id.clone();

        // Member-touching operation: a triple whose subject is a member.
        let member = members[0].clone();
        store
            .add_triple(&member, "affects", &(did_a.clone(), nid("n3")), &w)
            .await
            .unwrap();

        // Authority: the manual summary persists through the member-touching op.
        let after = store.get_community(&cid).await.unwrap();
        if after.summary != manual && cex.len() < 5 {
            cex.push(format!(
                "case {budget}: community summary auto-overwritten (got {:?}, manual {:?})",
                after.summary, manual
            ));
        }

        // A non-member touch (nothing should affect it either).
        let non_member = if two_doc {
            (did_a.clone(), nid("n2"))
        } else {
            (did_b.clone(), nid("m1"))
        };
        let _ = store
            .add_triple(&non_member, "affects", &(did_a.clone(), nid("n2")), &w)
            .await;
        let after2 = store.get_community(&cid).await.unwrap();
        if after2.summary != manual && cex.len() < 5 {
            cex.push(format!(
                "case {budget}: summary changed after non-member touch"
            ));
        }

        // Two-community adversarial: a second community whose members are NOT
        // touched must also keep its manual summary.
        if rng.pick(3) == 0 {
            let c2_manual = format!("second manual {budget}");
            let c2 = store
                .declare_community(
                    &[(did_a.clone(), nid("n2"))],
                    &DeclareCommunityOptions {
                        summary: c2_manual.clone(),
                        wiki_id: w.clone(),
                    },
                )
                .await
                .unwrap();
            let c2_after = store.get_community(&c2.community_id).await.unwrap();
            if c2_after.summary != c2_manual && cex.len() < 5 {
                cex.push(format!(
                    "case {budget}: untouched community summary changed"
                ));
            }
        }

        // Only an explicit update changes it.
        let new = format!("updated summary {budget}");
        let upd = store.update_community_summary(&cid, &new).await.unwrap();
        if upd.summary != new && cex.len() < 5 {
            cex.push(format!(
                "case {budget}: updateCommunitySummary returned {:?}",
                upd.summary
            ));
        }
        let fetched = store.get_community(&cid).await.unwrap();
        if fetched.summary != new && cex.len() < 5 {
            cex.push(format!(
                "case {budget}: updateCommunitySummary not reflected by getCommunity"
            ));
        }
    }
    assert!(
        cex.is_empty(),
        "P-TP-4 BROKEN — {} counterexample(s):\n  - {}",
        cex.len(),
        cex.join("\n  - ")
    );
}

/// Read a durable alias→canonical mapping (unwrap: the tests only query aliases
/// already established by a successful `resolve_entities`).
async fn aliased(store: &Store, alias: &(DocumentId, NodeId)) -> Option<(DocumentId, NodeId)> {
    store.entity_alias_canonical(alias).await.unwrap()
}

// ===========================================================================
// READ-ONLY ADVERSARIAL PBT AUDIT — §4.2 negative-generator / robustness probes
// ---------------------------------------------------------------------------
// DETERMINISTIC EXPLICIT assertion blocks (no PRNG-generated cases). The per-row
// PRNG budgets above still sum to exactly 400; these probes are `#[test]`s that
// build one fixed scenario each and assert the current GREEN behavior. Each probe
// name carries the audit probe id (P-SM-1, P-TP-2, P-SM-2, P-IM-2). Nothing here
// drives a `StoreError` on the legal inputs it exercises; where an assertion pins
// an *implemented* behavior (rather than a register invariant) it is explicitly
// commented as such.
//
// Pinned seed is unchanged (BASE_SEED = 0x6D797A4CF0D09E5D, derived per-row);
// these probes add no generated cases, so the ≤400 generated-case budget holds.

/// A `Relation` edge carrying a triple's `relation_type` (GRAPH-OWNS-RELATION-AND-MERGE).
fn relation_edge(from: (DocumentId, NodeId), to: (DocumentId, NodeId), relation: &str) -> Edge {
    let cross_wiki = from.0 != to.0;
    Edge {
        source: from,
        target: to,
        kind: EdgeKind::Relation,
        state: None,
        cross_wiki,
        relation_type: Some(relation.to_string()),
    }
}

/// §4.2.6 topological-order validator (deterministic): every resolved node is
/// unique, the `resolution_order`s are exactly `{1..=n}`, and every resolved
/// `Reference` node whose target is also resolved appears at a STRICTLY larger
/// order than its target. `target_of` is a prebuilt map node→reference-target
/// (None or absent for non-Reference nodes). Returns human-readable problems.
fn assert_topological(
    resolved: &[ResolvedFact],
    target_of: &HashMap<(DocumentId, NodeId), Option<(DocumentId, NodeId)>>,
) -> Vec<String> {
    let mut problems = Vec::new();
    let n = resolved.len() as u64;
    let mut orders: Vec<u64> = resolved.iter().map(|rf| rf.resolution_order).collect();
    orders.sort_unstable();
    if orders != (1..=n).collect::<Vec<u64>>() {
        problems.push(format!("resolution orders {:?} != 1..={n}", orders));
    }
    for (i, a) in resolved.iter().enumerate() {
        if resolved.iter().skip(i + 1).any(|b| b.node == a.node) {
            problems.push(format!(
                "node {:?} resolved more than once (redundant)",
                a.node
            ));
            break;
        }
    }
    for rf in resolved {
        if let Some(Some(t)) = target_of.get(&rf.node) {
            if let Some(to) = resolved
                .iter()
                .find(|b| &b.node == t)
                .map(|b| b.resolution_order)
            {
                if !(to < rf.resolution_order) {
                    problems.push(format!(
                        "dependent {:?} order {} is not strictly after its target {:?} order {}",
                        rf.node, rf.resolution_order, t, to
                    ));
                }
            }
        }
    }
    problems
}

// ---------------------------------------------------------------------------
// P-SM-1 — cascade SURVIVOR / over-prune detector  (SM, deterministic)
// ---------------------------------------------------------------------------
// Three nodes a, b, c with relations a→b and c→b; remove ONLY node c. The real
// cascade (§4.2.7.5) prunes the endpoint that vanished (c→b) but MUST keep a→b
// on BOTH adjacency surfaces (`edges_from(a)` and `edges_to(b)`). This is the
// case the register's cascade sub-assertion cannot catch: a hypothetical
// over-aggressive cascade that dropped the whole Relation set would still pass
// that sub-assertion, but is caught here because a→b must SURVIVE.
#[tokio::test]
async fn p_sm1_cascade_survivor_over_prune_detector() {
    let (store, _w, doc) = fresh_doc(&["a", "b", "c"]).await;
    let did = doc.document_id.clone();
    let a = (did.clone(), nid("a"));
    let b = (did.clone(), nid("b"));
    let c = (did.clone(), nid("c"));

    // Seed the doc graph with relations a→b and c→b (as graph-owned Relation
    // edges, mirroring what `addTriple` records).
    apply_graph(
        &store,
        &doc,
        Graph {
            nodes: vec![
                content_node(&did, "a", "va"),
                content_node(&did, "b", "vb"),
                content_node(&did, "c", "vc"),
            ],
            edges: vec![
                edge(
                    EdgeKind::DocHead,
                    (did.clone(), nid("ROOT")),
                    (did.clone(), nid("a")),
                    None,
                ),
                edge(
                    EdgeKind::DocEnd,
                    (did.clone(), nid("c")),
                    (did.clone(), nid("END")),
                    None,
                ),
                relation_edge(a.clone(), b.clone(), "a_dep_b"),
                relation_edge(c.clone(), b.clone(), "c_dep_b"),
            ],
        },
    )
    .await;

    // "Remove only c": replace the graph with a & b retained and all relations
    // STILL LISTED (the cascade must decide which to prune). c→b has a vanished
    // end-point and must be pruned; a→b has both endpoints retained and survives.
    apply_graph(
        &store,
        &doc,
        Graph {
            nodes: vec![content_node(&did, "a", "va"), content_node(&did, "b", "vb")],
            edges: vec![
                edge(
                    EdgeKind::DocHead,
                    (did.clone(), nid("ROOT")),
                    (did.clone(), nid("a")),
                    None,
                ),
                edge(
                    EdgeKind::DocEnd,
                    (did.clone(), nid("b")),
                    (did.clone(), nid("END")),
                    None,
                ),
                relation_edge(a.clone(), b.clone(), "a_dep_b"),
                relation_edge(c.clone(), b.clone(), "c_dep_b"),
            ],
        },
    )
    .await;

    // (1) The survivor a→b is present in the authoritative edge set.
    let full = store.edges_for_document(&did).await.unwrap();
    let survivor = full
        .iter()
        .find(|e| e.kind == EdgeKind::Relation && e.source == a && e.target == b);
    if survivor.is_none() {
        panic!("P-SM-1 SURVIVOR: removed only c, but a→b was cascaded away (over-prune)");
    }
    if survivor.unwrap().relation_type.as_deref() != Some("a_dep_b") {
        panic!("P-SM-1 SURVIVOR: a→b survived but lost its relation_type");
    }
    // (2) The pruned c→b is gone: no Relation edge sources from c anywhere.
    if full
        .iter()
        .any(|e| e.kind == EdgeKind::Relation && e.source == c)
    {
        panic!("P-SM-1 SURVIVOR: c→b not pruned after removing node c");
    }
    // (3) Survivor visible on BOTH adjacency surfaces.
    let from_a = store.edges_from(&did, &nid("a")).await.unwrap();
    if !from_a.iter().any(|e| {
        e.kind == EdgeKind::Relation
            && e.target == b
            && e.relation_type.as_deref() == Some("a_dep_b")
    }) {
        panic!("P-SM-1 SURVIVOR: a→b missing from edges_from(a) after removing c");
    }
    let to_b = store.edges_to(&did, &nid("b")).await.unwrap();
    if !to_b.iter().any(|e| {
        e.kind == EdgeKind::Relation
            && e.source == a
            && e.relation_type.as_deref() == Some("a_dep_b")
    }) {
        panic!("P-SM-1 SURVIVOR: a→b missing from edges_to(b) after removing c");
    }
    if to_b
        .iter()
        .any(|e| e.kind == EdgeKind::Relation && e.source == c)
    {
        panic!("P-SM-1 SURVIVOR: edges_to(b) still references removed node c");
    }
}

// ---------------------------------------------------------------------------
// P-TP-2 — resolve_entities overlap + convergence  (TP, deterministic)
// ---------------------------------------------------------------------------
// Exercises the register's "idempotent + convergent" for a SUBTLE overlapping
// re-resolution (not just the disjoint set already probed by the P-TP-2 row).
// Per RESOLVE-ENTITIES-AUTHORITATIVE-OVERWRITE, `resolveEntities` is an
// **authoritative overwrite**: each call re-stabilizes the durable alias→canonical
// map to an acyclic, flat alias graph. Two things are deliberately separated:
//   * SAME-CALL IDEMPOTENCE holds: re-applying the identical `(ids, canonical)`
//     call returns an equal `ResolutionResult` and never flips a recorded alias
//     under repetition (the register's guarantee, §4.2.9.1 / P-TP-2).
//   * An OVERLAPPING re-resolution with a DIFFERENT canonical legally flips
//     aliases — and, because the call is an authoritative overwrite, re-applying
//     the original call FULLY converges the map: the chosen canonical is never
//     itself an alias (its prior entry is dropped), every alias in the requested
//     set points directly to the canonical, and any prior alias that pointed to a
//     now-re-aliased node is re-pointed (path-compression). No residual alias, no
//     cycle — overlap convergence is GUARANTEED.
#[tokio::test]
async fn p_tp2_resolve_entities_overlap_and_convergence() {
    let (store, w, doc) = fresh_doc(&["e1", "e2", "e3"]).await;
    let did = doc.document_id.clone();
    let e1 = (did.clone(), nid("e1"));
    let e2 = (did.clone(), nid("e2"));
    let e3 = (did.clone(), nid("e3"));

    // --- original identical call: canonical = e1 over a 3-set ---
    let a_ids = vec![e1.clone(), e2.clone(), e3.clone()];
    let a_opts = ResolveEntitiesOptions {
        canonical_id: Some(e1.clone()),
        wiki_id: w.clone(),
    };
    let r1 = store.resolve_entities(&a_ids, &a_opts).await.unwrap();
    assert_eq!(
        r1.canonical_id, e1,
        "P-TP-2: explicit canonical e1 not honored"
    );
    // Snapshot the durable mapping the call records.
    assert_eq!(
        aliased(&store, &e2).await,
        Some(e1.clone()),
        "P-TP-2: e2 should alias e1"
    );
    assert_eq!(
        aliased(&store, &e3).await,
        Some(e1.clone()),
        "P-TP-2: e3 should alias e1"
    );
    // The canonical itself is NOT recorded as its own alias.
    assert_eq!(
        aliased(&store, &e1).await,
        None,
        "P-TP-2: canonical e1 should not alias itself"
    );

    // SAME-CALL IDEMPOTENCE: identical re-apply → equal result, no alias flips.
    let r1b = store.resolve_entities(&a_ids, &a_opts).await.unwrap();
    assert_eq!(
        r1b, r1,
        "P-TP-2: identical re-apply of (ids,e1) returned a different result"
    );
    assert_eq!(
        aliased(&store, &e2).await,
        Some(e1.clone()),
        "P-TP-2: e2 flipped under identical re-apply"
    );
    assert_eq!(
        aliased(&store, &e3).await,
        Some(e1.clone()),
        "P-TP-2: e3 flipped under identical re-apply"
    );

    // --- overlapping re-resolution with a CHANGED canonical over a shared set ---
    // [e1, e2], canonical = e2: legal; the previously-canonical e1 reappears as
    // an alias→e2, and e3 (not in the call) is PATH-COMPRESSED: it pointed to e1
    // (now re-aliased to e2), so it is re-pointed to the new root e2.
    let b_ids = vec![e1.clone(), e2.clone()];
    let b_opts = ResolveEntitiesOptions {
        canonical_id: Some(e2.clone()),
        wiki_id: w.clone(),
    };
    let rb = store.resolve_entities(&b_ids, &b_opts).await.unwrap();
    assert_eq!(
        rb.canonical_id, e2,
        "P-TP-2: overlapping canonical e2 not honored"
    );
    assert_eq!(
        aliased(&store, &e1).await,
        Some(e2.clone()),
        "P-TP-2: previously-canonical e1 should reappear as alias→e2"
    );
    assert_eq!(
        aliased(&store, &e3).await,
        Some(e2.clone()),
        "P-TP-2: e3 (outside the overlapping set) is path-compressed to the new root e2"
    );

    // --- re-apply the ORIGINAL identical call ([e1,e2,e3], e1) ---
    // Authoritative overwrite: the map FULLY converges back to {e2→e1, e3→e1}.
    // The overlapping call's residual `e1 → e2` entry is DROPPED (the chosen
    // canonical e1 is never itself an alias), so `aliased(e1) == None` — no
    // residual alias, no cycle.
    let r2 = store.resolve_entities(&a_ids, &a_opts).await.unwrap();
    assert_eq!(
        aliased(&store, &e2).await,
        Some(e1.clone()),
        "P-TP-2: e2 did not converge back toward e1"
    );
    assert_eq!(
        aliased(&store, &e3).await,
        Some(e1.clone()),
        "P-TP-2: e3 did not converge back toward e1"
    );
    // RESOLVE-ENTITIES-AUTHORITATIVE-OVERWRITE: the canonical is never itself an
    // alias — the overlapping re-resolution's residual `e1 → e2` entry must be
    // dropped, so the map fully converges (no residual alias, no cycle).
    assert_eq!(
        aliased(&store, &e1).await,
        None,
        "P-TP-2: authoritative overwrite must drop the residual e1→e2 entry (canonical e1 is never an alias)"
    );
    assert_eq!(
        r2.canonical_id, e1,
        "P-TP-2: re-applied original call must keep canonical e1"
    );

    // SAME-CALL IDEMPOTENCE also holds for the re-apply: a further identical call
    // is a no-op (equal result, no alias flips).
    let r2b = store.resolve_entities(&a_ids, &a_opts).await.unwrap();
    assert_eq!(
        r2b, r2,
        "P-TP-2: second identical re-apply changed the result"
    );
    assert_eq!(
        aliased(&store, &e2).await,
        Some(e1.clone()),
        "P-TP-2: e2 flipped under repeated identical call"
    );
    assert_eq!(
        aliased(&store, &e3).await,
        Some(e1.clone()),
        "P-TP-2: e3 flipped under repeated identical call"
    );
    assert_eq!(
        aliased(&store, &e1).await,
        None,
        "P-TP-2: e1 must remain canonical (not an alias) under repeated identical call"
    );
}

// ---------------------------------------------------------------------------
// P-TP-2 — resolve_entities overlap-flip: canonical flips, no residual, no cycle
// (TP, deterministic)
// ---------------------------------------------------------------------------
// RESOLVE-ENTITIES-AUTHORITATIVE-OVERWRITE: a later manual call supersedes an
// earlier one (manual-vs-manual last-write-wins). Resolving `[e1,e2]` with
// canonical=e1 then re-resolving the SAME pair with canonical=e2 must fully flip
// the map to `{e1→e2}`: the chosen canonical e2 is never itself an alias (its
// prior `e2→e1` entry is dropped), so `entity_alias_canonical(e2) == None` — no
// residual `e2→e1`, no cycle.
#[tokio::test]
async fn p_tp2_resolve_entities_overlap_flip_no_residual() {
    let (store, w, doc) = fresh_doc(&["e1", "e2"]).await;
    let did = doc.document_id.clone();
    let e1 = (did.clone(), nid("e1"));
    let e2 = (did.clone(), nid("e2"));
    let ids = vec![e1.clone(), e2.clone()];

    // --- first call: canonical = e1 → {e2→e1} ---
    let r1 = store
        .resolve_entities(
            &ids,
            &ResolveEntitiesOptions {
                canonical_id: Some(e1.clone()),
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap();
    assert_eq!(r1.canonical_id, e1);
    assert_eq!(
        aliased(&store, &e2).await,
        Some(e1.clone()),
        "P-TP-2: e2 should alias e1 after the first call"
    );
    assert_eq!(
        aliased(&store, &e1).await,
        None,
        "P-TP-2: canonical e1 must not alias itself"
    );

    // --- second call: canonical = e2 over the SAME pair → {e1→e2} ---
    let r2 = store
        .resolve_entities(
            &ids,
            &ResolveEntitiesOptions {
                canonical_id: Some(e2.clone()),
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap();
    assert_eq!(r2.canonical_id, e2);
    assert_eq!(
        aliased(&store, &e1).await,
        Some(e2.clone()),
        "P-TP-2: previously-canonical e1 should now alias e2"
    );
    // Authoritative overwrite: the chosen canonical e2 is never itself an alias —
    // its prior `e2→e1` entry is dropped, so no residual and no cycle.
    assert_eq!(
        aliased(&store, &e2).await,
        None,
        "P-TP-2: authoritative overwrite must drop the prior e2→e1 entry (canonical e2 is never an alias)"
    );
}

// ---------------------------------------------------------------------------
// P-TP-2 — resolve_entities chain-flatten: prior alias re-pointed to canonical
// (TP, deterministic)
// ---------------------------------------------------------------------------
// RESOLVE-ENTITIES-AUTHORITATIVE-OVERWRITE: any prior alias that pointed to a
// now-re-aliased node is re-pointed (path-compression) to the canonical, so the
// durable map stays acyclic and flat. Resolving `[e2,e3]` with canonical=e2
// records `{e3→e2}`; then resolving `[e1,e2]` with canonical=e1 re-aliases e2 to
// e1 and must re-point e3 directly to e1 — `entity_alias_canonical(e3) ==
// Some(e1)`, not the stale `Some(e2)`.
#[tokio::test]
async fn p_tp2_resolve_entities_chain_flatten() {
    let (store, w, doc) = fresh_doc(&["e1", "e2", "e3"]).await;
    let did = doc.document_id.clone();
    let e1 = (did.clone(), nid("e1"));
    let e2 = (did.clone(), nid("e2"));
    let e3 = (did.clone(), nid("e3"));

    // --- first call: canonical = e2 over [e2,e3] → {e3→e2} ---
    let r1 = store
        .resolve_entities(
            &[e2.clone(), e3.clone()],
            &ResolveEntitiesOptions {
                canonical_id: Some(e2.clone()),
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap();
    assert_eq!(r1.canonical_id, e2);
    assert_eq!(
        aliased(&store, &e3).await,
        Some(e2.clone()),
        "P-TP-2: e3 should alias e2 after the first call"
    );

    // --- second call: canonical = e1 over [e1,e2] → {e2→e1, e3→e1} ---
    let r2 = store
        .resolve_entities(
            &[e1.clone(), e2.clone()],
            &ResolveEntitiesOptions {
                canonical_id: Some(e1.clone()),
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap();
    assert_eq!(r2.canonical_id, e1);
    assert_eq!(
        aliased(&store, &e2).await,
        Some(e1.clone()),
        "P-TP-2: e2 should now alias e1"
    );
    // Path-compression: e3 previously pointed to e2, which is now re-aliased to
    // e1 — e3 must be re-pointed directly to e1 (flat, acyclic).
    assert_eq!(
        aliased(&store, &e3).await,
        Some(e1.clone()),
        "P-TP-2: e3 must be re-pointed (path-compressed) to e1, not stale e2"
    );
    assert_eq!(
        aliased(&store, &e1).await,
        None,
        "P-TP-2: canonical e1 must not alias itself"
    );
}

// ---------------------------------------------------------------------------
// P-TP-2 — resolve_entities 3+ call chain: multi-hop path-compression (TP, deterministic)
// ---------------------------------------------------------------------------
// RESOLVE-ENTITIES-AUTHORITATIVE-OVERWRITE: a chain of three authoritative calls
// must fully collapse every alias to the final canonical. Resolving `[e3,e4]` with
// canonical=e3 records `{e4→e3}`; then `[e2,e3]` with canonical=e2 re-aliases e3 to
// e2 and must re-point e4 to e2; then `[e1,e2]` with canonical=e1 re-aliases e2 to
// e1 and must re-point e3 AND e4 directly to e1 — `entity_alias_canonical(e4) ==
// Some(e1)` (multi-hop collapse, flat and acyclic).
#[tokio::test]
async fn p_tp2_resolve_entities_three_hop_chain_flatten() {
    let (store, w, doc) = fresh_doc(&["e1", "e2", "e3", "e4"]).await;
    let did = doc.document_id.clone();
    let e1 = (did.clone(), nid("e1"));
    let e2 = (did.clone(), nid("e2"));
    let e3 = (did.clone(), nid("e3"));
    let e4 = (did.clone(), nid("e4"));

    // --- call 1: canonical = e3 over [e3,e4] → {e4→e3} ---
    let r1 = store
        .resolve_entities(
            &[e3.clone(), e4.clone()],
            &ResolveEntitiesOptions {
                canonical_id: Some(e3.clone()),
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap();
    assert_eq!(r1.canonical_id, e3);
    assert_eq!(
        aliased(&store, &e4).await,
        Some(e3.clone()),
        "P-TP-2: e4 should alias e3 after the first call"
    );

    // --- call 2: canonical = e2 over [e2,e3] → {e3→e2, e4→e2} ---
    let r2 = store
        .resolve_entities(
            &[e2.clone(), e3.clone()],
            &ResolveEntitiesOptions {
                canonical_id: Some(e2.clone()),
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap();
    assert_eq!(r2.canonical_id, e2);
    assert_eq!(
        aliased(&store, &e3).await,
        Some(e2.clone()),
        "P-TP-2: e3 should now alias e2"
    );
    assert_eq!(
        aliased(&store, &e4).await,
        Some(e2.clone()),
        "P-TP-2: e4 must be re-pointed (path-compressed) to e2, not stale e3"
    );

    // --- call 3: canonical = e1 over [e1,e2] → {e2→e1, e3→e1, e4→e1} ---
    let r3 = store
        .resolve_entities(
            &[e1.clone(), e2.clone()],
            &ResolveEntitiesOptions {
                canonical_id: Some(e1.clone()),
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap();
    assert_eq!(r3.canonical_id, e1);
    assert_eq!(
        aliased(&store, &e2).await,
        Some(e1.clone()),
        "P-TP-2: e2 should now alias e1"
    );
    assert_eq!(
        aliased(&store, &e3).await,
        Some(e1.clone()),
        "P-TP-2: e3 must be re-pointed (path-compressed) to e1, not stale e2"
    );
    // Multi-hop collapse: e4 (two hops away) must land directly on e1.
    assert_eq!(
        aliased(&store, &e4).await,
        Some(e1.clone()),
        "P-TP-2: e4 must collapse all the way to e1 (multi-hop path-compression)"
    );
    assert_eq!(
        aliased(&store, &e1).await,
        None,
        "P-TP-2: canonical e1 must not alias itself"
    );
}

// ---------------------------------------------------------------------------
// P-TP-2 — resolve_entities flip where the new canonical was a root with aliases
// (TP, deterministic)
// ---------------------------------------------------------------------------
// RESOLVE-ENTITIES-AUTHORITATIVE-OVERWRITE: when a later call flips the canonical
// to a node that was previously a root carrying its own aliases, those aliases
// must be re-pointed to the new canonical (path-compression across a flip).
// Resolving `[e2,e3]` with canonical=e2 records `{e3→e2}`; then `[e1,e2]` with
// canonical=e1 records `{e2→e1, e3→e1}`; then re-resolving `[e1,e2]` with
// canonical=e2 flips the map to `{e1→e2, e3→e2}` — e3 must be re-pointed to e2.
#[tokio::test]
async fn p_tp2_resolve_entities_flip_root_with_aliases() {
    let (store, w, doc) = fresh_doc(&["e1", "e2", "e3"]).await;
    let did = doc.document_id.clone();
    let e1 = (did.clone(), nid("e1"));
    let e2 = (did.clone(), nid("e2"));
    let e3 = (did.clone(), nid("e3"));

    // --- call 1: canonical = e2 over [e2,e3] → {e3→e2} ---
    let r1 = store
        .resolve_entities(
            &[e2.clone(), e3.clone()],
            &ResolveEntitiesOptions {
                canonical_id: Some(e2.clone()),
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap();
    assert_eq!(r1.canonical_id, e2);
    assert_eq!(
        aliased(&store, &e3).await,
        Some(e2.clone()),
        "P-TP-2: e3 should alias e2 after the first call"
    );

    // --- call 2: canonical = e1 over [e1,e2] → {e2→e1, e3→e1} ---
    let r2 = store
        .resolve_entities(
            &[e1.clone(), e2.clone()],
            &ResolveEntitiesOptions {
                canonical_id: Some(e1.clone()),
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap();
    assert_eq!(r2.canonical_id, e1);
    assert_eq!(
        aliased(&store, &e2).await,
        Some(e1.clone()),
        "P-TP-2: e2 should now alias e1"
    );
    assert_eq!(
        aliased(&store, &e3).await,
        Some(e1.clone()),
        "P-TP-2: e3 must be re-pointed to e1"
    );

    // --- call 3: canonical = e2 over the SAME pair [e1,e2] → {e1→e2, e3→e2} ---
    let r3 = store
        .resolve_entities(
            &[e1.clone(), e2.clone()],
            &ResolveEntitiesOptions {
                canonical_id: Some(e2.clone()),
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap();
    assert_eq!(r3.canonical_id, e2);
    assert_eq!(
        aliased(&store, &e1).await,
        Some(e2.clone()),
        "P-TP-2: previously-canonical e1 should now alias e2"
    );
    // Path-compression across a flip: e3 previously pointed to e1, which is now
    // re-aliased to e2 — e3 must be re-pointed directly to e2.
    assert_eq!(
        aliased(&store, &e3).await,
        Some(e2.clone()),
        "P-TP-2: e3 must be re-pointed (path-compressed) to e2 across the flip"
    );
    assert_eq!(
        aliased(&store, &e2).await,
        None,
        "P-TP-2: canonical e2 must not alias itself"
    );
}

// ---------------------------------------------------------------------------
// P-TP-2 — resolve_entities duplicate entity_ids (TP, deterministic)
// ---------------------------------------------------------------------------
// RESOLVE-ENTITIES-AUTHORITATIVE-OVERWRITE: duplicate entity_ids in a single call
// must not create a self-loop or a duplicate alias entry. Resolving `[e1,e2,e2]`
// with canonical=e1 yields exactly `{e2→e1}` — `entity_alias_canonical(e2) ==
// Some(e1)`, `entity_alias_canonical(e1) == None`, and the result is well-formed
// (no self-loop, no duplicate).
#[tokio::test]
async fn p_tp2_resolve_entities_duplicate_ids() {
    let (store, w, doc) = fresh_doc(&["e1", "e2"]).await;
    let did = doc.document_id.clone();
    let e1 = (did.clone(), nid("e1"));
    let e2 = (did.clone(), nid("e2"));

    let r = store
        .resolve_entities(
            &[e1.clone(), e2.clone(), e2.clone()],
            &ResolveEntitiesOptions {
                canonical_id: Some(e1.clone()),
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap();
    assert_eq!(r.canonical_id, e1);
    assert_eq!(
        aliased(&store, &e2).await,
        Some(e1.clone()),
        "P-TP-2: e2 should alias e1 despite the duplicate entity_id"
    );
    assert_eq!(
        aliased(&store, &e1).await,
        None,
        "P-TP-2: canonical e1 must not alias itself (no self-loop)"
    );
    // Well-formed durable map: the duplicate entity_id must not create a duplicate
    // durable entry or a self-loop — the durable alias→canonical map holds exactly
    // `{e2→e1}` (a single entry, confirmed by `aliased(e2) == Some(e1)` and
    // `aliased(e1) == None`). The returned descriptor may carry one entry per
    // non-canonical occurrence; the durable map is what the authoritative-overwrite
    // semantics guarantee to be well-formed.
    assert_eq!(r.canonical_id, e1, "P-TP-2: canonical must be e1");
    assert!(
        r.aliases.iter().all(|a| a.alias == e2 && a.canonical == e1),
        "P-TP-2: every alias descriptor must be e2→e1 (no self-loop, no foreign target)"
    );
}

// ---------------------------------------------------------------------------
// P-TP-2 — resolve_entities multiple-roots map stays acyclic (TP, deterministic)
// ---------------------------------------------------------------------------
// RESOLVE-ENTITIES-AUTHORITATIVE-OVERWRITE: disjoint resolutions keep disjoint
// roots; an overlapping re-resolution must fold the second root's aliases into the
// surviving canonical so every alias points to a root and the map stays acyclic
// (no alias points to a non-root). Resolving `[e1,e2]` with canonical=e1 and
// `[e3,e4]` with canonical=e3 yields `{e2→e1, e4→e3}`; then `[e1,e3]` with
// canonical=e1 folds e3 (and its alias e4) into e1: `{e2→e1, e3→e1, e4→e1}`.
#[tokio::test]
async fn p_tp2_resolve_entities_multiple_roots_acyclic() {
    let (store, w, doc) = fresh_doc(&["e1", "e2", "e3", "e4"]).await;
    let did = doc.document_id.clone();
    let e1 = (did.clone(), nid("e1"));
    let e2 = (did.clone(), nid("e2"));
    let e3 = (did.clone(), nid("e3"));
    let e4 = (did.clone(), nid("e4"));

    // --- disjoint resolutions: {e2→e1} and {e4→e3} ---
    let r1 = store
        .resolve_entities(
            &[e1.clone(), e2.clone()],
            &ResolveEntitiesOptions {
                canonical_id: Some(e1.clone()),
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap();
    assert_eq!(r1.canonical_id, e1);
    let r2 = store
        .resolve_entities(
            &[e3.clone(), e4.clone()],
            &ResolveEntitiesOptions {
                canonical_id: Some(e3.clone()),
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap();
    assert_eq!(r2.canonical_id, e3);
    assert_eq!(
        aliased(&store, &e2).await,
        Some(e1.clone()),
        "P-TP-2: e2 should alias e1"
    );
    assert_eq!(
        aliased(&store, &e4).await,
        Some(e3.clone()),
        "P-TP-2: e4 should alias e3"
    );

    // --- overlap: canonical = e1 over [e1,e3] → {e2→e1, e3→e1, e4→e1} ---
    let r3 = store
        .resolve_entities(
            &[e1.clone(), e3.clone()],
            &ResolveEntitiesOptions {
                canonical_id: Some(e1.clone()),
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap();
    assert_eq!(r3.canonical_id, e1);
    assert_eq!(
        aliased(&store, &e3).await,
        Some(e1.clone()),
        "P-TP-2: e3 must be folded into e1"
    );
    assert_eq!(
        aliased(&store, &e4).await,
        Some(e1.clone()),
        "P-TP-2: e4 must be re-pointed (path-compressed) to e1"
    );
    // Every alias points to a root (e1); no alias points to a non-root.
    assert_eq!(
        aliased(&store, &e2).await,
        Some(e1.clone()),
        "P-TP-2: e2 must still point to root e1"
    );
    assert_eq!(
        aliased(&store, &e1).await,
        None,
        "P-TP-2: canonical e1 must not alias itself"
    );
    assert_eq!(
        aliased(&store, &e3).await,
        Some(e1.clone()),
        "P-TP-2: e3 must point to root e1 (not to a non-root)"
    );
    assert_eq!(
        aliased(&store, &e4).await,
        Some(e1.clone()),
        "P-TP-2: e4 must point to root e1 (not to a non-root)"
    );
}

// ---------------------------------------------------------------------------
// P-TP-2 — resolve_entities canonical currently an alias in the set (TP, deterministic)
// ---------------------------------------------------------------------------
// RESOLVE-ENTITIES-AUTHORITATIVE-OVERWRITE: when the chosen canonical is currently
// an alias pointing to a node in the set, the authoritative overwrite must drop the
// alias entry for the canonical (it is never itself an alias) and re-point the
// prior canonical to it. Resolving `[e1,e2]` with canonical=e1 yields `{e2→e1}`;
// then re-resolving `[e1,e2]` with canonical=e2 flips to `{e1→e2}` —
// `entity_alias_canonical(e2) == None` and `entity_alias_canonical(e1) == Some(e2)`.
#[tokio::test]
async fn p_tp2_resolve_entities_canonical_was_alias() {
    let (store, w, doc) = fresh_doc(&["e1", "e2"]).await;
    let did = doc.document_id.clone();
    let e1 = (did.clone(), nid("e1"));
    let e2 = (did.clone(), nid("e2"));

    // --- call 1: canonical = e1 over [e1,e2] → {e2→e1} ---
    let r1 = store
        .resolve_entities(
            &[e1.clone(), e2.clone()],
            &ResolveEntitiesOptions {
                canonical_id: Some(e1.clone()),
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap();
    assert_eq!(r1.canonical_id, e1);
    assert_eq!(
        aliased(&store, &e2).await,
        Some(e1.clone()),
        "P-TP-2: e2 should alias e1 after the first call"
    );

    // --- call 2: canonical = e2 over the SAME pair [e1,e2] → {e1→e2} ---
    let r2 = store
        .resolve_entities(
            &[e1.clone(), e2.clone()],
            &ResolveEntitiesOptions {
                canonical_id: Some(e2.clone()),
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap();
    assert_eq!(r2.canonical_id, e2);
    // The chosen canonical e2 was previously an alias (e2→e1); the authoritative
    // overwrite must drop that entry so e2 is never itself an alias.
    assert_eq!(
        aliased(&store, &e2).await,
        None,
        "P-TP-2: canonical e2 must never be an alias (prior e2→e1 dropped)"
    );
    assert_eq!(
        aliased(&store, &e1).await,
        Some(e2.clone()),
        "P-TP-2: previously-canonical e1 should now alias e2"
    );
}

// ---------------------------------------------------------------------------
// P-TP-2 — resolve_entities re-affirm no-op path: idempotence (TP, deterministic)
// ---------------------------------------------------------------------------
// RESOLVE-ENTITIES-AUTHORITATIVE-OVERWRITE: re-applying an identical call is a
// no-op — the map is unchanged and the two ResolutionResults are equal. Resolving
// `[e1,e2]` with canonical=e1 yields `{e2→e1}`; re-applying the identical call
// leaves `{e2→e1}` unchanged, `entity_alias_canonical(e2) == Some(e1)`, and the
// two results are equal (idempotence).
#[tokio::test]
async fn p_tp2_resolve_entities_reaffirm_noop_idempotent() {
    let (store, w, doc) = fresh_doc(&["e1", "e2"]).await;
    let did = doc.document_id.clone();
    let e1 = (did.clone(), nid("e1"));
    let e2 = (did.clone(), nid("e2"));
    let ids = vec![e1.clone(), e2.clone()];
    let opts = ResolveEntitiesOptions {
        canonical_id: Some(e1.clone()),
        wiki_id: w.clone(),
    };

    // --- first call: canonical = e1 → {e2→e1} ---
    let r1 = store.resolve_entities(&ids, &opts).await.unwrap();
    assert_eq!(r1.canonical_id, e1);
    assert_eq!(
        aliased(&store, &e2).await,
        Some(e1.clone()),
        "P-TP-2: e2 should alias e1 after the first call"
    );

    // --- re-apply the identical call: unchanged {e2→e1}, equal result ---
    let r2 = store.resolve_entities(&ids, &opts).await.unwrap();
    assert_eq!(
        r2, r1,
        "P-TP-2: re-applying the identical call must be a no-op (equal result)"
    );
    assert_eq!(
        aliased(&store, &e2).await,
        Some(e1.clone()),
        "P-TP-2: e2 must still alias e1 after the no-op re-apply"
    );
    assert_eq!(
        aliased(&store, &e1).await,
        None,
        "P-TP-2: canonical e1 must not alias itself"
    );
}

// ---------------------------------------------------------------------------
// P-TP-2 — explicit canonical dominates regardless of entity_ids order  (TP, deterministic)
// ---------------------------------------------------------------------------
// Pass `entity_ids` in several shuffled orders with an explicit canonical set to
// a NON-FIRST position and assert the explicit canonical wins in every order.
// Constructed explicitly (no reliance on any generator-side sort).
#[tokio::test]
async fn p_tp2_resolve_entities_non_sorted_arg_order() {
    let (store, w, doc) = fresh_doc(&["e1", "e2", "e3", "e4"]).await;
    let did = doc.document_id.clone();
    let nodes = [
        (did.clone(), nid("e1")),
        (did.clone(), nid("e2")),
        (did.clone(), nid("e3")),
        (did.clone(), nid("e4")),
    ];
    // Explicit canonical deliberately NOT first, and at a different position in
    // every order below (position independence is exactly what is under probe).
    let canonical = nodes[2].clone(); // e3
    let orders: Vec<Vec<(DocumentId, NodeId)>> = vec![
        vec![
            nodes[0].clone(),
            nodes[1].clone(),
            nodes[2].clone(),
            nodes[3].clone(),
        ],
        vec![
            nodes[3].clone(),
            nodes[2].clone(),
            nodes[0].clone(),
            nodes[1].clone(),
        ],
        vec![
            nodes[2].clone(),
            nodes[0].clone(),
            nodes[3].clone(),
            nodes[1].clone(),
        ],
        vec![
            nodes[1].clone(),
            nodes[3].clone(),
            nodes[0].clone(),
            nodes[2].clone(),
        ],
    ];
    let opts = ResolveEntitiesOptions {
        canonical_id: Some(canonical.clone()),
        wiki_id: w.clone(),
    };
    for (i, ids) in orders.into_iter().enumerate() {
        let r = store.resolve_entities(&ids, &opts).await.unwrap();
        assert_eq!(
            r.canonical_id, canonical,
            "P-TP-2: order #{i}: explicit canonical {canonical:?} did not dominate regardless of position"
        );
        for e in &ids {
            if *e != canonical {
                assert_eq!(
                    store.entity_alias_canonical(e).await.unwrap(),
                    Some(canonical.clone()),
                    "P-TP-2: order #{i}: alias {e:?} must point at the explicit canonical"
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// P-SM-2 — cap-`max_hops` success boundary AND cross-doc resolution  (SM, deterministic)
// ---------------------------------------------------------------------------
// (A) A reference chain of depth EXACTLY `max_hops == 5` (the permitted cap, the
//     success boundary) must resolve successfully and topologically.
// (B) A `reference` edge in doc A resolving to a `fact` node in doc B (root(s) in
//     A) — the topological order must still hold across the cross-doc resolution.
#[tokio::test]
async fn p_sm2_max_hops_cap_boundary_and_cross_doc() {
    // ---- (A) depth == 5 == max_hops: r5→r4→r3→r2→r1→f ----
    let (store, w, doc) = fresh_doc(&["ph"]).await;
    let did = doc.document_id.clone();
    let f = (did.clone(), nid("f"));
    let r1 = (did.clone(), nid("r1"));
    let r2 = (did.clone(), nid("r2"));
    let r3 = (did.clone(), nid("r3"));
    let r4 = (did.clone(), nid("r4"));
    let r5 = (did.clone(), nid("r5"));
    let hop_graph = Graph {
        nodes: vec![
            fact_node(&did, "f", "kf", "leaf"),
            reference_node(&did, "r1", f.clone()),
            reference_node(&did, "r2", r1.clone()),
            reference_node(&did, "r3", r2.clone()),
            reference_node(&did, "r4", r3.clone()),
            reference_node(&did, "r5", r4.clone()),
        ],
        edges: vec![
            edge(
                EdgeKind::DocHead,
                (did.clone(), nid("ROOT")),
                f.clone(),
                None,
            ),
            edge(
                EdgeKind::DocEnd,
                r5.clone(),
                (did.clone(), nid("END")),
                None,
            ),
            edge(
                EdgeKind::Link,
                r1.clone(),
                f.clone(),
                Some(ReferenceState::Resolved),
            ),
            edge(
                EdgeKind::Link,
                r2.clone(),
                r1.clone(),
                Some(ReferenceState::Resolved),
            ),
            edge(
                EdgeKind::Link,
                r3.clone(),
                r2.clone(),
                Some(ReferenceState::Resolved),
            ),
            edge(
                EdgeKind::Link,
                r4.clone(),
                r3.clone(),
                Some(ReferenceState::Resolved),
            ),
            edge(
                EdgeKind::Link,
                r5.clone(),
                r4.clone(),
                Some(ReferenceState::Resolved),
            ),
        ],
    };
    apply_graph(&store, &doc, hop_graph).await;
    let resolved = store
        .resolve_references(
            std::slice::from_ref(&r5),
            &ResolveOptions {
                wiki_id: w.clone(),
                max_hops: 5,
            },
        )
        .await
        .expect("P-SM-2: depth==5 chain at max_hops==5 is the success boundary and must resolve");
    let cur = store.get_document(&did).await.unwrap();
    let mut target_of: HashMap<(DocumentId, NodeId), Option<(DocumentId, NodeId)>> = HashMap::new();
    for n in &cur.graph.nodes {
        let loc = (did.clone(), n.node_id.clone());
        let t = (n.kind == NodeKind::Reference)
            .then_some(n.target.clone())
            .flatten();
        target_of.insert(loc, t);
    }
    let problems = assert_topological(&resolved, &target_of);
    assert!(
        problems.is_empty(),
        "P-SM-2 (A): depth==5 boundary — {} problem(s): {}",
        problems.len(),
        problems.join("; ")
    );
    assert_eq!(
        resolved.len(),
        6,
        "P-SM-2 (A): depth==5 chain must resolve 6 nodes"
    );

    // ---- (B) cross-doc: reference in doc A → fact node in doc B, root in A ----
    let doc_b = fresh_doc_in(&store, &w, &["ph"]).await;
    let did_b = doc_b.document_id.clone();
    let fact_b = (did_b.clone(), nid("F"));
    apply_graph(
        &store,
        &doc_b,
        Graph {
            nodes: vec![fact_node(&did_b, "F", "kF", "cross")],
            edges: vec![
                edge(
                    EdgeKind::DocHead,
                    (did_b.clone(), nid("ROOT")),
                    fact_b.clone(),
                    None,
                ),
                edge(
                    EdgeKind::DocEnd,
                    fact_b.clone(),
                    (did_b.clone(), nid("END")),
                    None,
                ),
            ],
        },
    )
    .await;
    let doc_a = fresh_doc_in(&store, &w, &["ph"]).await;
    let did_a = doc_a.document_id.clone();
    let ref_a = (did_a.clone(), nid("rA"));
    apply_graph(
        &store,
        &doc_a,
        Graph {
            nodes: vec![reference_node(&did_a, "rA", fact_b.clone())],
            edges: vec![
                edge(
                    EdgeKind::DocHead,
                    (did_a.clone(), nid("ROOT")),
                    ref_a.clone(),
                    None,
                ),
                edge(
                    EdgeKind::DocEnd,
                    ref_a.clone(),
                    (did_a.clone(), nid("END")),
                    None,
                ),
                edge(
                    EdgeKind::Link,
                    ref_a.clone(),
                    fact_b.clone(),
                    Some(ReferenceState::Resolved),
                ),
            ],
        },
    )
    .await;
    let cres = store
        .resolve_references(
            std::slice::from_ref(&ref_a),
            &ResolveOptions {
                wiki_id: w.clone(),
                max_hops: 1,
            },
        )
        .await
        .expect("P-SM-2 (B): cross-doc acyclic reference (1 edge) must resolve");
    // The fact (in doc B) must be resolved BEFORE the reference (in doc A).
    assert_eq!(resolved.len(), 6, "P-SM-2 (B): sanity on doc-A chain count");
    let fact_in_cres = cres.iter().find(|rf| rf.node == fact_b);
    let ref_in_cres = cres.iter().find(|rf| rf.node == ref_a);
    assert!(
        fact_in_cres.is_some(),
        "P-SM-2 (B): cross-doc fact node (doc B) missing from result"
    );
    assert!(
        ref_in_cres.is_some(),
        "P-SM-2 (B): doc-A reference node missing from result"
    );
    assert!(
        fact_in_cres.unwrap().resolution_order < ref_in_cres.unwrap().resolution_order,
        "P-SM-2 (B): cross-doc fact must carry a strictly smaller resolution_order than its referencing root"
    );
    // Validate full topological order across the two documents.
    let mut ctarget: HashMap<(DocumentId, NodeId), Option<(DocumentId, NodeId)>> = HashMap::new();
    for (cdoc, cnode) in [
        (did_a.clone(), ref_a.clone()),
        (did_b.clone(), fact_b.clone()),
    ] {
        let d = store.get_document(&cdoc).await.unwrap();
        for n in &d.graph.nodes {
            if n.node_id == cnode.1 {
                let t = (n.kind == NodeKind::Reference)
                    .then_some(n.target.clone())
                    .flatten();
                ctarget.insert((cdoc.clone(), n.node_id.clone()), t);
            }
        }
    }
    let cprobs = assert_topological(&cres, &ctarget);
    assert!(
        cprobs.is_empty(),
        "P-SM-2 (B): cross-doc topological order — {} problem(s): {}",
        cprobs.len(),
        cprobs.join("; ")
    );
}

// ---------------------------------------------------------------------------
// P-IM-2 — cross-doc `In` direction (all-shard scan)  (IM, deterministic)
// ---------------------------------------------------------------------------
// A triple whose OBJECT is a node in doc B (subject in doc A): querying the doc-B
// node with direction `In` (or `Both`) must surface the triple via the all-shard
// scan for cross-doc objects. This is the boundary the generator row does not
// separately pin.
#[tokio::test]
async fn p_im2_cross_doc_in_direction_all_shard_scan() {
    let (store, w, doc_a) = fresh_doc(&["a1"]).await;
    let did_a = doc_a.document_id.clone();
    let doc_b = fresh_doc_in(&store, &w, &["b1", "b2"]).await;
    let did_b = doc_b.document_id.clone();
    let subj = (did_a.clone(), nid("a1"));
    let obj = (did_b.clone(), nid("b1"));
    let rel = "cross_flows_to";
    store.add_triple(&subj, rel, &obj, &w).await.unwrap();

    for dir in [TripleDirection::In, TripleDirection::Both] {
        let got = store
            .get_triples(
                &obj,
                &GetTriplesFilter {
                    direction: dir,
                    relation_type: Some(rel.to_string()),
                    wiki_id: w.clone(),
                },
            )
            .await
            .expect("P-IM-2: cross-doc In query on a live doc-B node must not error");
        assert!(
            got.iter().any(|t| t.subject == subj && t.object == obj && t.relation == rel),
            "P-IM-2: cross-doc triple (A.a1 -{}-> B.b1) missing from get_triples(B.b1, dir={dir:?}) — all-shard scan failed",
            rel,
        );
    }

    // The pure `Out` direction on the doc-B object node must NOT surface it (it is
    // an incoming triple — the object is not a subject of this edge).
    let out = store
        .get_triples(
            &obj,
            &GetTriplesFilter {
                direction: TripleDirection::Out,
                relation_type: Some(rel.to_string()),
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap();
    assert!(
        !out.iter()
            .any(|t| t.subject == subj && t.object == obj && t.relation == rel),
        "P-IM-2: cross-doc triple spuriously returned under Out direction on its object node"
    );
}
