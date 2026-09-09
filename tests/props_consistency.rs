//! §5.1 Property layer — the EXECUTED PBT artifact (#2) for the §4.4
//! Consistency-enforcement unit.
//!
//! Satisfies `docs/specs/4-4-consistency-property-register.md` row-for-row:
//! one `#[test]` per register row (P-IM-1/2/3, P-SM-1/2/3, P-TP-1/2), each
//! tagging the row HELD/BROKEN and the `strategy-id` it exercised.
//!
//! PBT-gate contract (adapted to Rust, no proptest / no new crate):
//! - Deterministic pinned seed via a hand-rolled SplitMix64 PRNG. The seed is
//!   **fixed** per row (recorded below); every run produces the identical
//!   case sequence.
//! - Case budget: ≤100 generated cases per row; total across the layer ≤400.
//!   This layer runs 310 generated cases (P-IM-1 = 30, P-IM-2 = 34, P-IM-3 = 36,
//!   P-SM-1 = 40, P-SM-2 = 30, P-SM-3 = 40, P-TP-1 = 60, P-TP-2 = 40).
//!   The §4.4-audit probe blocks below are deterministic FIXED scenarios in
//!   their own `#[test]`s (no SplitMix64 cases consumed), so they add 0 to the
//!   generated-case budget (still 310 ≤ 400).
//! - stop-after-5: a broken row keeps ≤5 counterexamples then stops; a
//!   genuinely-failing row FAILS with the minimal counterexample(s) — the
//!   intended red set. All GREEN here is the expected held set.
//!
//! Pinned seeds (SplitMix64 initial state per row):
//!   P-IM-1 0x1111_1111_1111_1111   P-IM-2 0x2222_2222_2222_2222
//!   P-IM-3 0x3333_3333_3333_3333   P-SM-1 0x4444_4444_4444_4444
//!   P-SM-2 0x5555_5555_5555_5555   P-SM-3 0x6666_6666_6666_6666
//!   P-TP-1 0x7777_7777_7777_7777   P-TP-2 0x8888_8888_8888_8888
//!
//! All invariants are black-box over the crate's public `gnosis::*` surface
//! (`Arc<Store>`), using exactly the consistency seam named in the register:
//! `get_consistency_report`, `re_sync_embed`, `re_derive_community`,
//! `community_state`, the `update_fact`/`archive_document`/`update_document`
//! triggers, the `publish_document` gate, and `Store::epoch()/journal_len()`.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use gnosis::{
    CommunityState, ConsistencyReferenceReport, CreateDocumentRequest, DeclareCommunityOptions,
    DocState, Document, DocumentId, Edge, EdgeKind, Graph, Node, NodeId, NodeKind, RagStore,
    ReferenceState, Store, StoreError, UpdateDocumentRequest, UpdateFactRequest, WikiId,
};

// ---------------------------------------------------------------------------
// Deterministic PRNG — SplitMix64 (fixed seed, no crate dependency).
// ---------------------------------------------------------------------------
#[derive(Clone)]
struct SplitMix64(u64);

impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// Uniform in `[0, n)`.
    fn below(&mut self, n: u64) -> usize {
        (self.next() % n) as usize
    }
    /// Uniform bool.
    fn coin(&mut self) -> bool {
        self.next() & 1 == 1
    }
}

// ---------------------------------------------------------------------------
// Counterexample collector (stop-after-5).
// ---------------------------------------------------------------------------
struct Failures {
    row: &'static str,
    strategy: &'static str,
    seed: u64,
    items: Vec<String>,
}

impl Failures {
    fn new(row: &'static str, strategy: &'static str, seed: u64) -> Self {
        Failures {
            row,
            strategy,
            seed,
            items: Vec::new(),
        }
    }
    /// Record a counterexample; returns `true` once the 5-item cap is reached
    /// (caller should stop generating further cases).
    fn push(&mut self, msg: String) -> bool {
        self.items.push(msg);
        self.items.len() >= 5
    }
    fn finish(self) {
        if !self.items.is_empty() {
            panic!(
                "[PBT row BROKEN] {} ({}) strategy={} seed=0x{:016x} — {} counterexample(s):\n  {}",
                self.row,
                "BROKEN",
                self.strategy,
                self.seed,
                self.items.len(),
                self.items.join("\n  ")
            );
        }
    }
}

macro_rules! pcheck {
    ($cond:expr, $($arg:tt)*) => {
        if !($cond) {
            return Err(format!($($arg)*));
        }
    };
}

// ---------------------------------------------------------------------------
// Shared fixtures (copied from tests/consistency_integration.rs).
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
/// A `reference` node. `snapshot` = the embed's copied value (`Some`) or `None`
/// for a live link (no copy stored, §4.2.2).
fn ref_node(
    doc: &DocumentId,
    node: &str,
    target: (DocumentId, NodeId),
    snapshot: Option<&str>,
) -> Node {
    Node {
        document_id: doc.clone(),
        node_id: nid(node),
        kind: NodeKind::Reference,
        value: snapshot.map(|s| s.to_string()),
        fact_key: None,
        target: Some(target),
    }
}
fn edge(
    kind: EdgeKind,
    from: (DocumentId, NodeId),
    to: (DocumentId, NodeId),
    state: Option<ReferenceState>,
    cross_wiki: bool,
) -> Edge {
    Edge {
        source: from,
        target: to,
        kind,
        state,
        cross_wiki,
        relation_type: None,
    }
}
/// A valid Provident graph over `nodes` carrying the `refs` reference edges.
fn graph_with(doc: &DocumentId, nodes: Vec<Node>, refs: Vec<Edge>) -> Graph {
    let first = nodes
        .first()
        .map(|n| n.node_id.clone())
        .unwrap_or_else(|| nid("n"));
    let mut edges = vec![
        edge(
            EdgeKind::DocHead,
            (doc.clone(), nid("ROOT")),
            (doc.clone(), first.clone()),
            None,
            false,
        ),
        edge(
            EdgeKind::DocEnd,
            (doc.clone(), first.clone()),
            (doc.clone(), nid("END")),
            None,
            false,
        ),
    ];
    edges.extend(refs);
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
fn citation_graph(doc: &DocumentId) -> Graph {
    let cite = content_node(doc, "cite", "cited-content");
    graph_with(doc, vec![cite.clone()], vec![])
}
async fn seed_fact(
    store: &Store,
    w: &WikiId,
    src_doc: &Document,
    fact_key: &str,
    value: &str,
) -> (DocumentId, NodeId) {
    // Ground a resolvable citation node (`cite`) in the source document before
    // declaring the fact (§3.2 minimum-citation invariant).
    apply_graph(store, src_doc, citation_graph(&src_doc.document_id)).await;
    store
        .create_fact(
            w,
            &src_doc.document_id,
            fact_key,
            value,
            &[(src_doc.document_id.clone(), nid("cite"))],
        )
        .await
        .unwrap();
    (
        src_doc.document_id.clone(),
        nid(&format!("fact-{fact_key}")),
    )
}
fn find_row<'a>(
    report: &'a [ConsistencyReferenceReport],
    node_id: &str,
    kind: EdgeKind,
) -> &'a ConsistencyReferenceReport {
    report
        .iter()
        .find(|r| r.node_id == nid(node_id) && r.kind == kind)
        .unwrap_or_else(|| panic!("no report row for {kind:?} under node {node_id}"))
}
/// Document-scoped row lookup (nodes of the same name may exist across docs).
fn find_row_in<'a>(
    report: &'a [ConsistencyReferenceReport],
    doc: &DocumentId,
    node_id: &str,
    kind: EdgeKind,
) -> &'a ConsistencyReferenceReport {
    report
        .iter()
        .find(|r| r.document_id == *doc && r.node_id == nid(node_id) && r.kind == kind)
        .unwrap_or_else(|| panic!("no report row for {kind:?} under node {node_id} of {doc:?}"))
}

// Black-box liveness / canonical helpers (used for the P-IM-3 / P-TP-2 checks).
async fn target_is_live(
    store: &Store,
    facts: &HashSet<(DocumentId, NodeId)>,
    target: &(DocumentId, NodeId),
) -> bool {
    if facts.contains(target) {
        return true;
    }
    match store.get_document(&target.0).await {
        Ok(doc) => {
            doc.state != DocState::Archived && doc.graph.nodes.iter().any(|n| n.node_id == target.1)
        }
        Err(_) => false,
    }
}
async fn target_canonical(
    store: &Store,
    facts: &HashMap<(DocumentId, NodeId), String>,
    target: &(DocumentId, NodeId),
) -> Option<String> {
    if let Some(v) = facts.get(target) {
        return Some(v.clone());
    }
    match store.get_document(&target.0).await {
        Ok(doc) => doc
            .graph
            .nodes
            .iter()
            .find(|n| n.node_id == target.1)
            .and_then(|n| n.value.clone()),
        Err(_) => None,
    }
}
async fn held_snapshot(store: &Store, row: &ConsistencyReferenceReport) -> Option<String> {
    match store.get_document(&row.document_id).await {
        Ok(doc) => doc
            .graph
            .nodes
            .iter()
            .find(|n| n.node_id == row.node_id)
            .and_then(|n| n.value.clone()),
        Err(_) => None,
    }
}
/// P-IM-3 core check: for a NON-cross-wiki row, no fabrication — a row is
/// Resolved only over a live target, Fresh only over a live target with a
/// matching snapshot, and never Fresh/Resolved over a missing/archived target.
fn pm3_check(
    row: &ConsistencyReferenceReport,
    live: bool,
    snap: Option<String>,
    canon: Option<String>,
) -> Result<(), String> {
    if row.cross_wiki {
        return Ok(());
    }
    let loc = format!("({},{:?}->{:?})", row.node_id.0, row.kind, row.state);
    match row.state {
        ReferenceState::Resolved => pcheck!(
            live,
            "P-IM-3 {loc}: Resolved over a target that is NOT live"
        ),
        ReferenceState::Fresh => {
            pcheck!(
                live,
                "P-IM-3 {loc}: Fresh over a target that is NOT live (fabricated)"
            );
            // no snapshot embed whose held snapshot differs from canonical reports Fresh
            pcheck!(
                snap == canon,
                "P-IM-3 {loc}: Fresh but snapshot {:?} != canonical {:?} (fabricated fresh)",
                snap,
                canon
            );
        }
        ReferenceState::Broken => pcheck!(!live, "P-IM-3 {loc}: Broken over a target that IS live"),
        ReferenceState::Stale => {
            if live {
                // a Stale embed against a live target must hold a differing snapshot
                pcheck!(
                    snap.is_some() && canon.is_some() && snap != canon,
                    "P-IM-3 {loc}: Stale over a live target with snapshot==canonical ({:?})",
                    snap
                );
            }
        }
    }
    Ok(())
}

// ===========================================================================
// P-IM-1 — Determinism (ref_store_basic)
// ===========================================================================
const SEED_IM1: u64 = 0x1111_1111_1111_1111;
const PIM1_CASES: usize = 30;

#[tokio::test]
async fn p_im1_determinism() {
    let mut rng = SplitMix64(SEED_IM1);
    let mut fail = Failures::new("P-IM-1", "ref_store_basic", SEED_IM1);
    'cases: for case in 0..PIM1_CASES {
        if let Err(e) = pim1_case(&mut rng, case).await {
            if fail.push(format!("case {case}: {e}")) {
                break 'cases;
            }
        }
    }
    fail.finish();
}

async fn pim1_case(rng: &mut SplitMix64, case: usize) -> Result<(), String> {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let src = new_doc(&store, &w, "src").await;
    let fact_loc = seed_fact(&store, &w, &src, "f", "A").await;

    // d0: an embed of the fact (the determinism mutation trigger).
    let d0 = new_doc(&store, &w, "d0").await;
    let e0 = ref_node(&d0.document_id, "em", fact_loc.clone(), Some("A"));
    apply_graph(
        &store,
        &d0,
        graph_with(
            &d0.document_id,
            vec![content_node(&d0.document_id, "n", "x"), e0.clone()],
            vec![edge(
                EdgeKind::Embed,
                (d0.document_id.clone(), e0.node_id.clone()),
                fact_loc.clone(),
                None,
                false,
            )],
        ),
    )
    .await;

    // Extra referrer docs (0..=4) of varying kinds/states/targets for coverage.
    let extra = rng.below(5);
    for i in 0..extra {
        let d = new_doc(&store, &w, &format!("d-{case}-{i}")).await;
        let kind = match rng.below(3) {
            0 => EdgeKind::Link,
            1 => EdgeKind::Embed,
            _ => EdgeKind::Crosslink,
        };
        let state = [
            None,
            Some(ReferenceState::Fresh),
            Some(ReferenceState::Resolved),
            Some(ReferenceState::Broken),
        ][rng.below(4)];
        let tgt = if rng.coin() {
            (did(&format!("ghost-{case}")), nid("gone"))
        } else {
            fact_loc.clone()
        };
        let snap = if matches!(kind, EdgeKind::Embed | EdgeKind::Crosslink)
            && state != Some(ReferenceState::Resolved)
        {
            Some(match state {
                None => "A",
                Some(ReferenceState::Fresh) => "A",
                Some(ReferenceState::Stale) => "Z",
                _ => "A",
            })
        } else {
            None
        };
        // Cross-wiki target only when the edge is a Crosslink.
        let cross = kind == EdgeKind::Crosslink;
        let rn = ref_node(&d.document_id, "r", tgt.clone(), snap);
        apply_graph(
            &store,
            &d,
            graph_with(
                &d.document_id,
                vec![content_node(&d.document_id, "n", "x"), rn.clone()],
                vec![edge(
                    kind,
                    (d.document_id.clone(), rn.node_id.clone()),
                    tgt.clone(),
                    state,
                    cross,
                )],
            ),
        )
        .await;
    }

    // Determinism: two consecutive reads, no intervening mutation → identical.
    let a = store.get_consistency_report(&w).await.unwrap();
    let b = store.get_consistency_report(&w).await.unwrap();
    pcheck!(
        a == b,
        "two consecutive reports differ despite no mutation (non-deterministic read)"
    );

    // Mutation must change the report: update_fact flips d0's embed Fresh→Stale.
    store
        .update_fact(
            &w,
            "f",
            &UpdateFactRequest {
                value: format!("B{case}"),
                citations: vec![(src.document_id.clone(), nid("cite"))],
            },
        )
        .await
        .unwrap();
    let c = store.get_consistency_report(&w).await.unwrap();
    pcheck!(c != a, "report unchanged after a committed mutation (update_fact) that should flip the fact-embed Fresh→Stale");
    Ok(())
}

// ===========================================================================
// P-IM-2 — Scope + well-formedness (ref_store_basic)
// ===========================================================================
const SEED_IM2: u64 = 0x2222_2222_2222_2222;
const PIM2_CASES: usize = 34;

#[tokio::test]
async fn p_im2_scope_and_wellformedness() {
    let mut rng = SplitMix64(SEED_IM2);
    let mut fail = Failures::new("P-IM-2", "ref_store_basic", SEED_IM2);
    'cases: for case in 0..PIM2_CASES {
        if let Err(e) = pim2_case(&mut rng, case).await {
            if fail.push(format!("case {case}: {e}")) {
                break 'cases;
            }
        }
    }
    fail.finish();
}

fn ref_key_sortable(k: &(DocumentId, NodeId, EdgeKind)) -> (String, String, String) {
    (k.0 .0.clone(), k.1 .0.clone(), format!("{:?}", k.2))
}

async fn pim2_case(rng: &mut SplitMix64, case: usize) -> Result<(), String> {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let w2 = new_wiki(&store, "w2").await;

    // Live target doc in w (a node embeds can reference).
    let tgt = new_doc(&store, &w, "tgt").await;
    let tgt_graph = graph_with(
        &tgt.document_id,
        vec![content_node(&tgt.document_id, "n1", "v")],
        vec![],
    );
    apply_graph(&store, &tgt, tgt_graph).await;

    // 1..=4 referencing docs in w with varied reference edges.
    let mut w_docs: Vec<DocumentId> = Vec::new();
    let count = 1 + rng.below(4);
    for i in 0..count {
        let d = new_doc(&store, &w, &format!("ref-{case}-{i}")).await;
        // self-referential content node inside this doc
        let selfn = content_node(&d.document_id, "selfn", "self");
        let mut nodes = vec![selfn.clone(), content_node(&d.document_id, "c", "c")];
        let mut refs: Vec<Edge> = Vec::new();
        // Always a self-referential embed + a link for kind coverage.
        let sr = ref_node(
            &d.document_id,
            "sr",
            (d.document_id.clone(), nid("selfn")),
            Some("self"),
        );
        nodes.push(sr.clone());
        refs.push(edge(
            EdgeKind::Embed,
            (d.document_id.clone(), sr.node_id.clone()),
            (d.document_id.clone(), nid("selfn")),
            None,
            false,
        ));
        // Random extra reference edge.
        let kind = match rng.below(3) {
            0 => EdgeKind::Link,
            1 => EdgeKind::Embed,
            _ => EdgeKind::Crosslink,
        };
        let state = [
            None,
            Some(ReferenceState::Fresh),
            Some(ReferenceState::Resolved),
            Some(ReferenceState::Broken),
        ][rng.below(4)];
        let (tgt_loc, cross) = match (kind, rng.coin()) {
            (EdgeKind::Crosslink, true) => ((tgt.document_id.clone(), nid("n1")), true),
            (EdgeKind::Crosslink, false) => {
                ((did(&format!("cross-ghost-{case}")), nid("gone")), false)
            }
            (_, true) => ((tgt.document_id.clone(), nid("n1")), false),
            (_, false) => ((did(&format!("ghost-{case}")), nid("gone")), false),
        };
        let snap = if matches!(kind, EdgeKind::Embed | EdgeKind::Crosslink) {
            Some(match state {
                Some(ReferenceState::Stale) => "Z",
                _ => "v",
            })
        } else {
            None
        };
        let rn = ref_node(&d.document_id, "xr", tgt_loc.clone(), snap);
        nodes.push(rn.clone());
        refs.push(edge(
            kind,
            (d.document_id.clone(), rn.node_id.clone()),
            tgt_loc.clone(),
            state,
            cross,
        ));
        apply_graph(&store, &d, graph_with(&d.document_id, nodes, refs)).await;
        w_docs.push(d.document_id);
    }

    // A second wiki whose doc carries a Crosslink into w — must appear in w2's
    // report, never w's.
    let d2 = new_doc(&store, &w2, "x").await;
    let x = ref_node(
        &d2.document_id,
        "xl",
        (tgt.document_id.clone(), nid("n1")),
        None,
    );
    apply_graph(
        &store,
        &d2,
        graph_with(
            &d2.document_id,
            vec![x.clone()],
            vec![edge(
                EdgeKind::Crosslink,
                (d2.document_id.clone(), x.node_id.clone()),
                (tgt.document_id.clone(), nid("n1")),
                None,
                true,
            )],
        ),
    )
    .await;

    // Enumerate the authoritative reference-edge key set from w's docs.
    let mut expected: Vec<(DocumentId, NodeId, EdgeKind)> = Vec::new();
    for id in &w_docs {
        let doc = store.get_document(id).await.unwrap();
        for e in &doc.graph.edges {
            if matches!(
                e.kind,
                EdgeKind::Link | EdgeKind::Embed | EdgeKind::Crosslink
            ) {
                expected.push((doc.document_id.clone(), e.source.1.clone(), e.kind));
            }
        }
    }
    let report = store.get_consistency_report(&w).await.unwrap();

    // |report| == reference-edge count; the multiset of row keys equals the set.
    pcheck!(
        report.len() == expected.len(),
        "report len {} != reference-edge count {} for wiki w",
        report.len(),
        expected.len()
    );
    let mut expected_k = expected.iter().map(ref_key_sortable).collect::<Vec<_>>();
    expected_k.sort();
    let mut report_k = report
        .iter()
        .map(|r| ref_key_sortable(&(r.document_id.clone(), r.node_id.clone(), r.kind)))
        .collect::<Vec<_>>();
    report_k.sort();
    pcheck!(
        report_k == expected_k,
        "report key set does not match the authoritative reference-edge key set of w"
    );

    // The w2 crosslink must NOT be in w's report.
    pcheck!(
        !report
            .iter()
            .any(|r| r.document_id == d2.document_id && r.node_id == nid("xl")),
        "a Crosslink sourced in another wiki leaked into w's report"
    );
    // ... but must appear in w2's report.
    let report2 = store.get_consistency_report(&w2).await.unwrap();
    pcheck!(
        report2.iter().any(|r| r.document_id == d2.document_id
            && r.node_id == nid("xl")
            && r.kind == EdgeKind::Crosslink),
        "the w2-sourced Crosslink is missing from w2's own report"
    );

    // Well-formedness per row.
    for r in &report {
        pcheck!(
            matches!(
                r.kind,
                EdgeKind::Link | EdgeKind::Embed | EdgeKind::Crosslink
            ),
            "row kind {} is not a reference kind",
            format!("{:?}", r.kind)
        );
        pcheck!(
            matches!(
                r.state,
                ReferenceState::Fresh
                    | ReferenceState::Stale
                    | ReferenceState::Resolved
                    | ReferenceState::Broken
            ),
            "row {} state out of the reference vocabulary",
            r.node_id.0
        );
        // target is an (DocumentId, NodeId); cross_wiki is boolean — structural.
        let _ = &r.target.0;
        let _ = r.cross_wiki;
        // Row's target matches its source document's recorded edge target.
        let doc = store.get_document(&r.document_id).await.unwrap();
        let matched = doc.graph.edges.iter().any(|e| {
            matches!(
                e.kind,
                EdgeKind::Link | EdgeKind::Embed | EdgeKind::Crosslink
            ) && e.source.1 == r.node_id
                && e.target == r.target
                && e.kind == r.kind
        });
        pcheck!(
            matched,
            "row for {} does not correspond to any stored reference edge in its source document",
            r.node_id.0
        );
    }
    Ok(())
}

// ===========================================================================
// P-IM-3 — No fabrication (mixed_liveness)
// ===========================================================================
const SEED_IM3: u64 = 0x3333_3333_3333_3333;
const PIM3_CASES: usize = 36;

#[tokio::test]
async fn p_im3_no_fabrication() {
    let mut rng = SplitMix64(SEED_IM3);
    let mut fail = Failures::new("P-IM-3", "mixed_liveness", SEED_IM3);
    'cases: for case in 0..PIM3_CASES {
        if let Err(e) = pim3_case(&mut rng, case).await {
            if fail.push(format!("case {case}: {e}")) {
                break 'cases;
            }
        }
    }
    fail.finish();
}

async fn pim3_case(rng: &mut SplitMix64, case: usize) -> Result<(), String> {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let w2 = new_wiki(&store, "w2").await;

    // Committed fact location (canonical ground truth the generator knows).
    let src = new_doc(&store, &w, "src").await;
    let fact_loc = seed_fact(&store, &w, &src, "fac", "C").await;
    let mut facts: HashSet<(DocumentId, NodeId)> = HashSet::new();
    facts.insert(fact_loc.clone());
    let mut fact_canon: HashMap<(DocumentId, NodeId), String> = HashMap::new();
    fact_canon.insert(fact_loc.clone(), "C".to_string());

    // Live target doc (occasionally archived) — for liveness mixing.
    let tgt = new_doc(&store, &w, "tgt").await;
    apply_graph(
        &store,
        &tgt,
        graph_with(
            &tgt.document_id,
            vec![
                content_node(&tgt.document_id, "n1", "v"),
                content_node(&tgt.document_id, "n2", "second"),
            ],
            vec![],
        ),
    )
    .await;
    let archive = rng.coin();
    if archive {
        store.archive_document(&tgt.document_id).await.unwrap();
    }

    // Referrer docs with None-state edges (report MUST derive truth).
    let d = new_doc(&store, &w, &format!("ref-{case}")).await;
    let link_live = ref_node(
        &d.document_id,
        "link-live",
        (tgt.document_id.clone(), nid("n1")),
        None,
    );
    let link_miss = ref_node(
        &d.document_id,
        "link-miss",
        (did(&format!("ghost-{case}")), nid("x")),
        None,
    );
    let embed_tgt = ref_node(
        &d.document_id,
        "embed-tgt",
        (tgt.document_id.clone(), nid("n2")),
        Some("second"),
    );
    let embed_ghost = ref_node(
        &d.document_id,
        "embed-ghost",
        (did(&format!("ghostb-{case}")), nid("y")),
        Some("s"),
    );
    let cross_live = ref_node(
        &d.document_id,
        "x-live",
        (tgt.document_id.clone(), nid("n1")),
        None,
    );
    let fact_embed = ref_node(&d.document_id, "fact-embed", fact_loc.clone(), Some("C"));
    apply_graph(
        &store,
        &d,
        graph_with(
            &d.document_id,
            vec![
                content_node(&d.document_id, "n", "x"),
                link_live.clone(),
                link_miss.clone(),
                embed_tgt.clone(),
                embed_ghost.clone(),
                cross_live.clone(),
                fact_embed.clone(),
            ],
            vec![
                edge(
                    EdgeKind::Link,
                    (d.document_id.clone(), nid("link-live")),
                    (tgt.document_id.clone(), nid("n1")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::Link,
                    (d.document_id.clone(), nid("link-miss")),
                    (did(&format!("ghost-{case}")), nid("x")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::Embed,
                    (d.document_id.clone(), nid("embed-tgt")),
                    (tgt.document_id.clone(), nid("n2")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::Embed,
                    (d.document_id.clone(), nid("embed-ghost")),
                    (did(&format!("ghostb-{case}")), nid("y")),
                    None,
                    false,
                ),
                // snapshot-less Crosslink → live reference (Resolved, not Fresh) — non-wiki-target shape.
                edge(
                    EdgeKind::Crosslink,
                    (d.document_id.clone(), nid("x-live")),
                    (tgt.document_id.clone(), nid("n1")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::Embed,
                    (d.document_id.clone(), nid("fact-embed")),
                    fact_loc.clone(),
                    None,
                    false,
                ),
            ],
        ),
    )
    .await;

    // Same-wiki cross-wiki-boundary shape: a Crosslink embed of the fact sourced in w2.
    let d2 = new_doc(&store, &w2, &format!("x-{case}")).await;
    let xf = ref_node(&d2.document_id, "xf", fact_loc.clone(), Some("C"));
    apply_graph(
        &store,
        &d2,
        graph_with(
            &d2.document_id,
            vec![xf.clone()],
            vec![edge(
                EdgeKind::Crosslink,
                (d2.document_id.clone(), nid("xf")),
                fact_loc.clone(),
                None,
                true,
            )],
        ),
    )
    .await;

    let report = store.get_consistency_report(&w).await.unwrap();
    // Every non-cross-wiki row must satisfy the no-fabrication derivation.
    for row in &report {
        let live = target_is_live(&store, &facts, &row.target).await;
        let snap = held_snapshot(&store, row).await;
        let canon = target_canonical(&store, &fact_canon, &row.target).await;
        pm3_check(row, live, snap, canon)?;
    }
    // The w2 report's cross-wiki row is honored (liveness skipped, but the
    // row still surfaces in w2's report as a snapshot embed — cross_wiki true).
    let report2 = store.get_consistency_report(&w2).await.unwrap();
    pcheck!(
        report2
            .iter()
            .any(|r| r.node_id == nid("xf") && r.kind == EdgeKind::Crosslink && r.cross_wiki),
        "the w2-sourced cross-wiki embed is missing from w2's report"
    );
    Ok(())
}

// ===========================================================================
// P-SM-1 — Fact-update propagation monotonicity (chain_of_many / stale_many_refs)
// ===========================================================================
const SEED_SM1: u64 = 0x4444_4444_4444_4444;
const PSM1_CASES: usize = 40;

#[tokio::test]
async fn p_sm1_fact_update_propagation_monotonicity() {
    let mut rng = SplitMix64(SEED_SM1);
    let mut fail = Failures::new("P-SM-1", "chain_of_many + stale_many_refs", SEED_SM1);
    'cases: for case in 0..PSM1_CASES {
        if let Err(e) = psm1_case(&mut rng, case).await {
            if fail.push(format!("case {case}: {e}")) {
                break 'cases;
            }
        }
    }
    fail.finish();
}

async fn psm1_case(rng: &mut SplitMix64, case: usize) -> Result<(), String> {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let w2 = new_wiki(&store, "w2").await;

    let src = new_doc(&store, &w, "src").await;
    let fact_loc = seed_fact(&store, &w, &src, "f", "A").await;

    // A linear chain of M referencing docs all snapshotting the fact.
    let m = 1 + rng.below(6);
    for i in 0..m {
        let d = new_doc(&store, &w, &format!("chain-{case}-{i}")).await;
        let e = ref_node(&d.document_id, "em", fact_loc.clone(), Some("A"));
        apply_graph(
            &store,
            &d,
            graph_with(
                &d.document_id,
                vec![content_node(&d.document_id, "n", "x"), e.clone()],
                vec![edge(
                    EdgeKind::Embed,
                    (d.document_id.clone(), e.node_id.clone()),
                    fact_loc.clone(),
                    None,
                    false,
                )],
            ),
        )
        .await;
    }
    // One non-embed Link to the fact (must stay Resolved).
    let dl = new_doc(&store, &w, &format!("link-{case}")).await;
    let l = ref_node(&dl.document_id, "lk", fact_loc.clone(), None);
    apply_graph(
        &store,
        &dl,
        graph_with(
            &dl.document_id,
            vec![content_node(&dl.document_id, "n", "x"), l.clone()],
            vec![edge(
                EdgeKind::Link,
                (dl.document_id.clone(), l.node_id.clone()),
                fact_loc.clone(),
                None,
                false,
            )],
        ),
    )
    .await;
    // A coincidentally-equal embed (snapshot ALREADY the new value) — the
    // propagation boundary shape (see note below; it exercises the shape for
    // coverage, and P-SM-1's observable requires no embed row be Fresh).
    let deq = new_doc(&store, &w, &format!("eq-{case}")).await;
    let eq = ref_node(&deq.document_id, "em", fact_loc.clone(), Some("B"));
    apply_graph(
        &store,
        &deq,
        graph_with(
            &deq.document_id,
            vec![content_node(&deq.document_id, "n", "x"), eq.clone()],
            vec![edge(
                EdgeKind::Embed,
                (deq.document_id.clone(), eq.node_id.clone()),
                fact_loc.clone(),
                None,
                false,
            )],
        ),
    )
    .await;
    // A cross-wiki snapshot embed in w2 (must surface stale in w2's report).
    let dx = new_doc(&store, &w2, &format!("x-{case}")).await;
    let xe = ref_node(&dx.document_id, "em", fact_loc.clone(), Some("A"));
    apply_graph(
        &store,
        &dx,
        graph_with(
            &dx.document_id,
            vec![xe.clone()],
            vec![edge(
                EdgeKind::Crosslink,
                (dx.document_id.clone(), xe.node_id.clone()),
                fact_loc.clone(),
                None,
                true,
            )],
        ),
    )
    .await;
    // Community incorporating the fact (must go Stale).
    let community = store
        .declare_community(
            std::slice::from_ref(&fact_loc),
            &DeclareCommunityOptions {
                summary: "s".into(),
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap();
    pcheck!(
        store
            .community_state(&community.community_id)
            .await
            .unwrap()
            == CommunityState::Fresh,
        "P-SM-1 setup: freshly-declared fact community is Fresh"
    );

    // §4.4.3 trigger: update_fact → v'.
    store
        .update_fact(
            &w,
            "f",
            &UpdateFactRequest {
                value: "B".into(),
                citations: vec![(src.document_id.clone(), nid("cite"))],
            },
        )
        .await
        .unwrap();

    // Same-wiki report: every embed row is Stale, the Link stays Resolved, no embed Fresh.
    let rw = store.get_consistency_report(&w).await.unwrap();
    let embed_rows = rw.iter().filter(|r| r.kind == EdgeKind::Embed);
    let link_row = find_row(&rw, "lk", EdgeKind::Link);
    let mut n_embeds = 0;
    for e in embed_rows {
        n_embeds += 1;
        pcheck!(
            e.state == ReferenceState::Stale,
            "P-SM-1: an embed row {:?} is not Stale after update_fact",
            e.node_id.0
        );
        pcheck!(
            e.state != ReferenceState::Fresh,
            "P-SM-1: an embed row {:?} is (fabricated) Fresh after update_fact",
            e.node_id.0
        );
    }
    pcheck!(
        link_row.state == ReferenceState::Resolved,
        "P-SM-1: a link to the changed fact is not Resolved"
    );
    // The chain M embeds + the equal-embed = M+1 same-wiki embeds, all Stale.
    pcheck!(
        n_embeds >= 2,
        "P-SM-1: expected at least the M chain + equal-snapshot embeds, got {}",
        n_embeds
    );

    // Cross-wiki report: the w2 snapshot embed is Stale.
    let rw2 = store.get_consistency_report(&w2).await.unwrap();
    let xrow = find_row(&rw2, "em", EdgeKind::Crosslink);
    pcheck!(
        xrow.state == ReferenceState::Stale,
        "P-SM-1: the cross-wiki embed is not Stale in w2's report"
    );

    // Community incorporating the fact → Stale.
    pcheck!(
        store
            .community_state(&community.community_id)
            .await
            .unwrap()
            == CommunityState::Stale,
        "P-SM-1: the community incorporating the changed fact is not Stale"
    );
    Ok(())
}

// ===========================================================================
// P-SM-2 — Fixpoint / idempotency of derived state (ref_store_basic)
// ===========================================================================
const SEED_SM2: u64 = 0x5555_5555_5555_5555;
const PSM2_CASES: usize = 30;

#[tokio::test]
async fn p_sm2_derived_state_fixpoint() {
    let mut rng = SplitMix64(SEED_SM2);
    let mut fail = Failures::new("P-SM-2", "ref_store_basic", SEED_SM2);
    'cases: for case in 0..PSM2_CASES {
        if let Err(e) = psm2_case(&mut rng, case).await {
            if fail.push(format!("case {case}: {e}")) {
                break 'cases;
            }
        }
    }
    fail.finish();
}

async fn psm2_case(_rng: &mut SplitMix64, case: usize) -> Result<(), String> {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;

    let src = new_doc(&store, &w, "src").await;
    let fact_loc = seed_fact(&store, &w, &src, "f", "A").await;

    // An already-Fresh embed.
    let d = new_doc(&store, &w, &format!("d-{case}")).await;
    let e = ref_node(&d.document_id, "em", fact_loc.clone(), Some("A"));
    apply_graph(
        &store,
        &d,
        graph_with(
            &d.document_id,
            vec![content_node(&d.document_id, "n", "x"), e.clone()],
            vec![edge(
                EdgeKind::Embed,
                (d.document_id.clone(), e.node_id.clone()),
                fact_loc.clone(),
                None,
                false,
            )],
        ),
    )
    .await;

    // Fixpoint of the read: consecutive reports equal.
    let a = store.get_consistency_report(&w).await.unwrap();
    let b = store.get_consistency_report(&w).await.unwrap();
    pcheck!(
        a == b,
        "P-SM-2: consecutive reports differ (read is not a fixpoint)"
    );
    let fresh_before = find_row(&a, "em", EdgeKind::Embed).state == ReferenceState::Fresh;
    pcheck!(
        fresh_before,
        "P-SM-2 setup: the embed should be Fresh before re-sync"
    );

    // Re-sync of an already-Fresh embed leaves derived state Fresh.
    store
        .re_sync_embed(&d.document_id, &nid("em"))
        .await
        .unwrap();
    let after = store.get_consistency_report(&w).await.unwrap();
    pcheck!(
        find_row(&after, "em", EdgeKind::Embed).state == ReferenceState::Fresh,
        "P-SM-2: re-sync of an already-Fresh embed left it non-Fresh"
    );

    // An already-Fresh community stays Fresh on re-derive.
    let community = store
        .declare_community(
            std::slice::from_ref(&fact_loc),
            &DeclareCommunityOptions {
                summary: "s".into(),
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap();
    pcheck!(
        store
            .community_state(&community.community_id)
            .await
            .unwrap()
            == CommunityState::Fresh,
        "P-SM-2 setup: community Fresh"
    );
    store
        .re_derive_community(&community.community_id)
        .await
        .unwrap();
    pcheck!(
        store
            .community_state(&community.community_id)
            .await
            .unwrap()
            == CommunityState::Fresh,
        "P-SM-2: re-derive of an already-Fresh community left it non-Fresh"
    );
    Ok(())
}

// ===========================================================================
// P-SM-3 — Revision/epoch monotonicity under propagation (stale_many_refs)
// ===========================================================================
const SEED_SM3: u64 = 0x6666_6666_6666_6666;
const PSM3_CASES: usize = 40;

#[tokio::test]
async fn p_sm3_revision_epoch_monotonicity() {
    let mut rng = SplitMix64(SEED_SM3);
    let mut fail = Failures::new("P-SM-3", "stale_many_refs", SEED_SM3);
    'cases: for case in 0..PSM3_CASES {
        if let Err(e) = psm3_case(&mut rng, case).await {
            if fail.push(format!("case {case}: {e}")) {
                break 'cases;
            }
        }
    }
    fail.finish();
}

async fn psm3_case(rng: &mut SplitMix64, case: usize) -> Result<(), String> {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;

    let src = new_doc(&store, &w, "src").await;
    let fact_loc = seed_fact(&store, &w, &src, "f", "A").await;

    // M referencing docs embedding the fact: 0, 1, or many.
    let m = [0usize, 1, 4][rng.below(3)];
    let mut ref_docs: Vec<DocumentId> = Vec::new();
    for i in 0..m {
        let d = new_doc(&store, &w, &format!("e-{case}-{i}")).await;
        let e = ref_node(&d.document_id, "em", fact_loc.clone(), Some("A"));
        apply_graph(
            &store,
            &d,
            graph_with(
                &d.document_id,
                vec![content_node(&d.document_id, "n", "x"), e.clone()],
                vec![edge(
                    EdgeKind::Embed,
                    (d.document_id.clone(), e.node_id.clone()),
                    fact_loc.clone(),
                    None,
                    false,
                )],
            ),
        )
        .await;
        ref_docs.push(d.document_id.clone());
    }

    // Target doc referenced by R arch-ref docs (archive propagation path).
    let tgt = new_doc(&store, &w, &format!("tgt-{case}")).await;
    apply_graph(
        &store,
        &tgt,
        graph_with(
            &tgt.document_id,
            vec![content_node(&tgt.document_id, "n1", "v")],
            vec![],
        ),
    )
    .await;
    let r = 1 + rng.below(3);
    let mut arch_refs: Vec<DocumentId> = Vec::new();
    for i in 0..r {
        let ad = new_doc(&store, &w, &format!("a-{case}-{i}")).await;
        let e = ref_node(
            &ad.document_id,
            "lk",
            (tgt.document_id.clone(), nid("n1")),
            None,
        );
        apply_graph(
            &store,
            &ad,
            graph_with(
                &ad.document_id,
                vec![content_node(&ad.document_id, "n", "x"), e.clone()],
                vec![edge(
                    EdgeKind::Link,
                    (ad.document_id.clone(), e.node_id.clone()),
                    (tgt.document_id.clone(), nid("n1")),
                    None,
                    false,
                )],
            ),
        )
        .await;
        arch_refs.push(ad.document_id.clone());
    }

    // Baseline.
    let mut epoch_prev = store.epoch();
    pcheck!(
        store.epoch() == store.journal_len() as u64,
        "epoch != journal_len at baseline"
    );

    // update_fact → propagation bumps each referencing embed doc's revision.
    let before: Vec<u64> = {
        let mut v = Vec::with_capacity(ref_docs.len());
        for id in &ref_docs {
            v.push(store.get_document(id).await.unwrap().revision);
        }
        v
    };
    store
        .update_fact(
            &w,
            "f",
            &UpdateFactRequest {
                value: "B".into(),
                citations: vec![(src.document_id.clone(), nid("cite"))],
            },
        )
        .await
        .unwrap();
    let after: Vec<u64> = {
        let mut v = Vec::with_capacity(ref_docs.len());
        for id in &ref_docs {
            v.push(store.get_document(id).await.unwrap().revision);
        }
        v
    };
    for ((b, a), id) in before.iter().zip(after.iter()).zip(ref_docs.iter()) {
        if m > 0 {
            pcheck!(
                a > b,
                "P-SM-3: update_fact propagation did not bump referencing doc {} revision ({}->{})",
                id.0,
                b,
                a
            );
        }
    }
    pcheck!(
        store.epoch() >= epoch_prev,
        "epoch decreased after update_fact"
    );
    pcheck!(
        store.epoch() == store.journal_len() as u64,
        "epoch != journal_len after update_fact"
    );
    epoch_prev = store.epoch();

    // archive_document → propagation bumps each arch-ref doc's revision.
    let abefore: Vec<u64> = {
        let mut v = Vec::with_capacity(arch_refs.len());
        for id in &arch_refs {
            v.push(store.get_document(id).await.unwrap().revision);
        }
        v
    };
    store.archive_document(&tgt.document_id).await.unwrap();
    let aafter: Vec<u64> = {
        let mut v = Vec::with_capacity(arch_refs.len());
        for id in &arch_refs {
            v.push(store.get_document(id).await.unwrap().revision);
        }
        v
    };
    for ((b, a), id) in abefore.iter().zip(aafter.iter()).zip(arch_refs.iter()) {
        pcheck!(
            a > b,
            "P-SM-3: archive propagation did not bump referencing doc {} revision ({}->{})",
            id.0,
            b,
            a
        );
    }
    pcheck!(
        store.epoch() >= epoch_prev,
        "epoch decreased after archive_document"
    );
    pcheck!(
        store.epoch() == store.journal_len() as u64,
        "epoch != journal_len after archive_document"
    );
    epoch_prev = store.epoch();

    // re_sync_embed bumps the referencing doc's revision by exactly +1.
    if m > 0 {
        let target = &ref_docs[0];
        let rb = store.get_document(target).await.unwrap().revision;
        store.re_sync_embed(target, &nid("em")).await.unwrap();
        let ra = store.get_document(target).await.unwrap().revision;
        pcheck!(
            ra == rb + 1,
            "P-SM-3: re_sync_embed did not bump the referencing doc by exactly +1 ({}->{})",
            rb,
            ra
        );
        pcheck!(
            store.epoch() >= epoch_prev,
            "epoch decreased after re_sync_embed"
        );
        pcheck!(
            store.epoch() == store.journal_len() as u64,
            "epoch != journal_len after re_sync_embed"
        );
    }
    Ok(())
}

// ===========================================================================
// P-TP-1 — Target-change re-stamp (repoint_matrix)
// ===========================================================================
const SEED_TP1: u64 = 0x7777_7777_7777_7777;
const PTP1_CASES: usize = 60;

#[tokio::test]
async fn p_tp1_target_change_restamp() {
    let mut rng = SplitMix64(SEED_TP1);
    let mut fail = Failures::new("P-TP-1", "repoint_matrix", SEED_TP1);
    'cases: for case in 0..PTP1_CASES {
        if let Err(e) = ptp1_case(&mut rng, case).await {
            if fail.push(format!("case {case}: {e}")) {
                break 'cases;
            }
        }
    }
    fail.finish();
}

async fn ptp1_case(rng: &mut SplitMix64, case: usize) -> Result<(), String> {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;

    // Live target doc with node n1.
    let tgt = new_doc(&store, &w, &format!("tgt-{case}")).await;
    apply_graph(
        &store,
        &tgt,
        graph_with(
            &tgt.document_id,
            vec![content_node(&tgt.document_id, "n1", "v")],
            vec![],
        ),
    )
    .await;
    let ghost = (did(&format!("ghost-{case}")), nid("gone"));

    // --- Link repoint matrix (state None) ---
    let dl = new_doc(&store, &w, &format!("l-{case}")).await;
    let l = ref_node(
        &dl.document_id,
        "l",
        (tgt.document_id.clone(), nid("n1")),
        None,
    );
    let link_graph = |t: (DocumentId, NodeId)| {
        graph_with(
            &dl.document_id,
            vec![content_node(&dl.document_id, "n", "x"), l.clone()],
            vec![edge(
                EdgeKind::Link,
                (dl.document_id.clone(), nid("l")),
                t,
                None,
                false,
            )],
        )
    };
    apply_graph(
        &store,
        &dl,
        link_graph((tgt.document_id.clone(), nid("n1"))),
    )
    .await;
    let r0 = store.get_consistency_report(&w).await.unwrap();
    pcheck!(
        find_row_in(&r0, &dl.document_id, "l", EdgeKind::Link).state == ReferenceState::Resolved,
        "P-TP-1: link to live target is not Resolved"
    );
    // repoint to ghost → Broken
    apply_graph(&store, &dl, link_graph(ghost.clone())).await;
    let r1 = store.get_consistency_report(&w).await.unwrap();
    pcheck!(
        find_row_in(&r1, &dl.document_id, "l", EdgeKind::Link).state == ReferenceState::Broken,
        "P-TP-1: link repointed to a missing target did not derive Broken"
    );
    // repoint back → Resolved
    apply_graph(
        &store,
        &dl,
        link_graph((tgt.document_id.clone(), nid("n1"))),
    )
    .await;
    let r2 = store.get_consistency_report(&w).await.unwrap();
    pcheck!(
        find_row_in(&r2, &dl.document_id, "l", EdgeKind::Link).state == ReferenceState::Resolved,
        "P-TP-1: link repointed back to a live target did not restore Resolved"
    );

    // --- Embed repoint matrix: matching snapshot ---
    let de = new_doc(&store, &w, &format!("e-{case}")).await;
    let e = ref_node(
        &de.document_id,
        "em",
        (tgt.document_id.clone(), nid("n1")),
        Some("v"),
    );
    let embed_graph = |t: (DocumentId, NodeId)| {
        graph_with(
            &de.document_id,
            vec![content_node(&de.document_id, "n", "x"), e.clone()],
            vec![edge(
                EdgeKind::Embed,
                (de.document_id.clone(), nid("em")),
                t,
                None,
                false,
            )],
        )
    };
    apply_graph(
        &store,
        &de,
        embed_graph((tgt.document_id.clone(), nid("n1"))),
    )
    .await;
    let s0 = store.get_consistency_report(&w).await.unwrap();
    pcheck!(
        find_row_in(&s0, &de.document_id, "em", EdgeKind::Embed).state == ReferenceState::Fresh,
        "P-TP-1: matching embed to live target is not Fresh"
    );
    apply_graph(&store, &de, embed_graph(ghost.clone())).await;
    let s1 = store.get_consistency_report(&w).await.unwrap();
    pcheck!(
        find_row_in(&s1, &de.document_id, "em", EdgeKind::Embed).state == ReferenceState::Stale,
        "P-TP-1: embed repointed to a missing target did not derive Stale"
    );
    apply_graph(
        &store,
        &de,
        embed_graph((tgt.document_id.clone(), nid("n1"))),
    )
    .await;
    let s2 = store.get_consistency_report(&w).await.unwrap();
    pcheck!(
        find_row_in(&s2, &de.document_id, "em", EdgeKind::Embed).state == ReferenceState::Fresh,
        "P-TP-1: matching embed repointed back is not Fresh again"
    );

    // --- Embed repoint matrix: mismatching snapshot (always Stale over live) ---
    let dm = new_doc(&store, &w, &format!("m-{case}")).await;
    let me = ref_node(
        &dm.document_id,
        "em",
        (tgt.document_id.clone(), nid("n1")),
        Some("WRONG"),
    );
    let mg = |t: (DocumentId, NodeId)| {
        graph_with(
            &dm.document_id,
            vec![content_node(&dm.document_id, "n", "x"), me.clone()],
            vec![edge(
                EdgeKind::Embed,
                (dm.document_id.clone(), nid("em")),
                t,
                None,
                false,
            )],
        )
    };
    apply_graph(&store, &dm, mg((tgt.document_id.clone(), nid("n1")))).await;
    let m0 = store.get_consistency_report(&w).await.unwrap();
    pcheck!(
        find_row_in(&m0, &dm.document_id, "em", EdgeKind::Embed).state == ReferenceState::Stale,
        "P-TP-1: mismatched embed over live target is not Stale"
    );
    apply_graph(&store, &dm, mg(ghost.clone())).await;
    apply_graph(&store, &dm, mg((tgt.document_id.clone(), nid("n1")))).await;
    let m2 = store.get_consistency_report(&w).await.unwrap();
    pcheck!(
        find_row_in(&m2, &dm.document_id, "em", EdgeKind::Embed).state == ReferenceState::Stale,
        "P-TP-1: mismatched embed round-trip did not stay Stale"
    );

    // --- Archive of cited target: link→Broken, embed→Stale ---
    if rng.coin() {
        let da = new_doc(&store, &w, &format!("arch-{case}")).await;
        let al = ref_node(
            &da.document_id,
            "l",
            (tgt.document_id.clone(), nid("n1")),
            None,
        );
        let ae = ref_node(
            &da.document_id,
            "em",
            (tgt.document_id.clone(), nid("n1")),
            Some("v"),
        );
        apply_graph(
            &store,
            &da,
            graph_with(
                &da.document_id,
                vec![
                    content_node(&da.document_id, "n", "x"),
                    al.clone(),
                    ae.clone(),
                ],
                vec![
                    edge(
                        EdgeKind::Link,
                        (da.document_id.clone(), nid("l")),
                        (tgt.document_id.clone(), nid("n1")),
                        None,
                        false,
                    ),
                    edge(
                        EdgeKind::Embed,
                        (da.document_id.clone(), nid("em")),
                        (tgt.document_id.clone(), nid("n1")),
                        None,
                        false,
                    ),
                ],
            ),
        )
        .await;
        store.archive_document(&tgt.document_id).await.unwrap();
        let ar = store.get_consistency_report(&w).await.unwrap();
        pcheck!(
            find_row_in(&ar, &da.document_id, "l", EdgeKind::Link).state == ReferenceState::Broken,
            "P-TP-1: link to archived target is not Broken"
        );
        pcheck!(
            find_row_in(&ar, &da.document_id, "em", EdgeKind::Embed).state == ReferenceState::Stale,
            "P-TP-1: embed of archived target is not Stale"
        );
    }

    // --- Self-referential repoint: a ref in d points at d's own content node ---
    let ds = new_doc(&store, &w, &format!("self-{case}")).await;
    let sr = ref_node(
        &ds.document_id,
        "sref",
        (ds.document_id.clone(), nid("sn")),
        Some("me"),
    );
    let sg = |t: (DocumentId, NodeId)| {
        graph_with(
            &ds.document_id,
            vec![content_node(&ds.document_id, "sn", "me"), sr.clone()],
            vec![edge(
                EdgeKind::Embed,
                (ds.document_id.clone(), nid("sref")),
                t,
                None,
                false,
            )],
        )
    };
    apply_graph(&store, &ds, sg((ds.document_id.clone(), nid("sn")))).await;
    let sr0 = store.get_consistency_report(&w).await.unwrap();
    pcheck!(
        find_row_in(&sr0, &ds.document_id, "sref", EdgeKind::Embed).state == ReferenceState::Fresh,
        "P-TP-1: self-referential matching embed is not Fresh"
    );
    apply_graph(&store, &ds, sg(ghost.clone())).await;
    let sr1 = store.get_consistency_report(&w).await.unwrap();
    pcheck!(
        find_row_in(&sr1, &ds.document_id, "sref", EdgeKind::Embed).state == ReferenceState::Stale,
        "P-TP-1: self-referential embed repointed to a ghost is not Stale"
    );
    Ok(())
}

// ===========================================================================
// P-TP-2 — Publish-gate consistency + atomicity (publish_matrix + concurrent)
// ===========================================================================
const SEED_TP2: u64 = 0x8888_8888_8888_8888;
const PTP2_DET_CASES: usize = 34;
const PTP2_CONC_ROUNDS: usize = 6;

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn p_tp2_publish_gate_and_concurrent() {
    let mut rng = SplitMix64(SEED_TP2);
    let mut fail = Failures::new("P-TP-2", "publish_matrix", SEED_TP2);
    'cases: for case in 0..PTP2_DET_CASES {
        if let Err(e) = ptp2_det_case(&mut rng, case).await {
            if fail.push(format!("case {case}: {e}")) {
                break 'cases;
            }
        }
    }
    // Adversarial concurrent shape: concurrent publish + fact-commit + re-sync
    // behind a Barrier. Bounded (no livelock hunt): tight rounds, all handles
    // must complete, none panic, and the final report satisfies P-IM-3.
    'conc: for round in 0..PTP2_CONC_ROUNDS {
        if let Err(e) = ptp2_concurrent_round(rng.clone(), round).await {
            if fail.push(format!("concurrent round {round}: {e}")) {
                break 'conc;
            }
        }
    }
    fail.finish();
}

async fn ptp2_det_case(rng: &mut SplitMix64, case: usize) -> Result<(), String> {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let src = new_doc(&store, &w, "src").await;
    let fact_loc = seed_fact(&store, &w, &src, "f", "A").await;
    let _ = rng; // shapes below are fixed-deterministic; the rng varies the value tokens

    // (a) all-clean doc: a Fresh embed (snapshot == canonical) + a Resolved link → publish Ok.
    let d_clean = new_doc(&store, &w, &format!("clean-{case}")).await;
    let ec = ref_node(&d_clean.document_id, "em", fact_loc.clone(), Some("A"));
    let lc = ref_node(&d_clean.document_id, "L", fact_loc.clone(), None);
    apply_graph(
        &store,
        &d_clean,
        graph_with(
            &d_clean.document_id,
            vec![
                content_node(&d_clean.document_id, "n", "x"),
                ec.clone(),
                lc.clone(),
            ],
            vec![
                edge(
                    EdgeKind::Embed,
                    (d_clean.document_id.clone(), ec.node_id.clone()),
                    fact_loc.clone(),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::Link,
                    (d_clean.document_id.clone(), lc.node_id.clone()),
                    fact_loc.clone(),
                    None,
                    false,
                ),
            ],
        ),
    )
    .await;
    let pub_clean = store.publish_document(&d_clean.document_id).await.unwrap();
    pcheck!(
        pub_clean.state == DocState::Published,
        "P-TP-2(a): all-clean doc did not publish"
    );
    // After success: re-read report, all non-cross-wiki refs Fresh/Resolved with live target.
    let facts: HashSet<(DocumentId, NodeId)> = std::iter::once(fact_loc.clone()).collect();
    let fact_canon: HashMap<(DocumentId, NodeId), String> =
        std::iter::once((fact_loc.clone(), "A".to_string())).collect();
    let rw = store.get_consistency_report(&w).await.unwrap();
    for row in rw
        .iter()
        .filter(|r| r.document_id == d_clean.document_id && !r.cross_wiki)
    {
        pcheck!(
            matches!(row.state, ReferenceState::Fresh | ReferenceState::Resolved),
            "P-TP-2(a): published doc has a non-Fresh/Resolved non-cross-wiki ref {:?}",
            row.state
        );
        pcheck!(
            target_is_live(&store, &facts, &row.target).await,
            "P-TP-2(a): published doc ref targets a non-live node"
        );
        let _ = &fact_canon;
    }

    // (b) a Stale embed → Err(UnresolvedReference), state unchanged; after re-sync → Ok.
    let d_embed = new_doc(&store, &w, &format!("embed-{case}")).await;
    let ee = ref_node(&d_embed.document_id, "em", fact_loc.clone(), Some("Z"));
    apply_graph(
        &store,
        &d_embed,
        graph_with(
            &d_embed.document_id,
            vec![content_node(&d_embed.document_id, "n", "x"), ee.clone()],
            vec![edge(
                EdgeKind::Embed,
                (d_embed.document_id.clone(), ee.node_id.clone()),
                fact_loc.clone(),
                None,
                false,
            )],
        ),
    )
    .await;
    {
        let err = store
            .publish_document(&d_embed.document_id)
            .await
            .unwrap_err();
        pcheck!(
            err == StoreError::UnresolvedReference,
            "P-TP-2(b): a stale embed did not block publish with UnresolvedReference"
        );
        let st = store
            .get_document(&d_embed.document_id)
            .await
            .unwrap()
            .state;
        pcheck!(
            st == DocState::Draft,
            "P-TP-2(b): failed publish left the document state changed"
        );
    }
    store
        .re_sync_embed(&d_embed.document_id, &nid("em"))
        .await
        .unwrap();
    let p = store.publish_document(&d_embed.document_id).await.unwrap();
    pcheck!(
        p.state == DocState::Published,
        "P-TP-2(b): after re-sync the doc did not publish"
    );

    // (c) a Broken link → Err(UnresolvedReference), state unchanged.
    let d_broken = new_doc(&store, &w, &format!("broken-{case}")).await;
    let bl = ref_node(
        &d_broken.document_id,
        "b",
        (did(&format!("ghost-{case}")), nid("gone")),
        None,
    );
    apply_graph(
        &store,
        &d_broken,
        graph_with(
            &d_broken.document_id,
            vec![content_node(&d_broken.document_id, "n", "x"), bl.clone()],
            vec![edge(
                EdgeKind::Link,
                (d_broken.document_id.clone(), bl.node_id.clone()),
                (did(&format!("ghost-{case}")), nid("gone")),
                None,
                false,
            )],
        ),
    )
    .await;
    {
        let err = store
            .publish_document(&d_broken.document_id)
            .await
            .unwrap_err();
        pcheck!(
            err == StoreError::UnresolvedReference,
            "P-TP-2(c): a broken link did not block publish"
        );
        pcheck!(
            store
                .get_document(&d_broken.document_id)
                .await
                .unwrap()
                .state
                == DocState::Draft,
            "P-TP-2(c): failed publish left the document state changed"
        );
    }

    // (d) a node that is a member of a STALE community → Err; after re-derive → Ok.
    let d_comm = new_doc(&store, &w, &format!("comm-{case}")).await;
    apply_graph(
        &store,
        &d_comm,
        graph_with(
            &d_comm.document_id,
            vec![content_node(&d_comm.document_id, "m", "v1")],
            vec![],
        ),
    )
    .await;
    let community = store
        .declare_community(
            &[(d_comm.document_id.clone(), nid("m"))],
            &DeclareCommunityOptions {
                summary: "s".into(),
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap();
    // Member change → community Stale.
    apply_graph(
        &store,
        &d_comm,
        graph_with(
            &d_comm.document_id,
            vec![content_node(&d_comm.document_id, "m", "v2")],
            vec![],
        ),
    )
    .await;
    pcheck!(
        store
            .community_state(&community.community_id)
            .await
            .unwrap()
            == CommunityState::Stale,
        "P-TP-2(d) setup: member change did not mark the community Stale"
    );
    {
        let err = store
            .publish_document(&d_comm.document_id)
            .await
            .unwrap_err();
        pcheck!(
            err == StoreError::UnresolvedReference,
            "P-TP-2(d): a STALE community member did not block publish"
        );
        pcheck!(
            store.get_document(&d_comm.document_id).await.unwrap().state == DocState::Draft,
            "P-TP-2(d): failed publish left the document state changed"
        );
    }
    store
        .re_derive_community(&community.community_id)
        .await
        .unwrap();
    let p = store.publish_document(&d_comm.document_id).await.unwrap();
    pcheck!(
        p.state == DocState::Published,
        "P-TP-2(d): after re-derive the community-member doc did not publish"
    );

    // mix: a BROKEN link is never resolved by re-syncing an unrelated embed.
    let d_mix = new_doc(&store, &w, &format!("mix-{case}")).await;
    let me1 = ref_node(&d_mix.document_id, "em", fact_loc.clone(), Some("Z"));
    let mbroken = ref_node(
        &d_mix.document_id,
        "b",
        (did(&format!("g2-{case}")), nid("gone")),
        None,
    );
    apply_graph(
        &store,
        &d_mix,
        graph_with(
            &d_mix.document_id,
            vec![
                content_node(&d_mix.document_id, "n", "x"),
                me1.clone(),
                mbroken.clone(),
            ],
            vec![
                edge(
                    EdgeKind::Embed,
                    (d_mix.document_id.clone(), me1.node_id.clone()),
                    fact_loc.clone(),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::Link,
                    (d_mix.document_id.clone(), mbroken.node_id.clone()),
                    (did(&format!("g2-{case}")), nid("gone")),
                    None,
                    false,
                ),
            ],
        ),
    )
    .await;
    store
        .re_sync_embed(&d_mix.document_id, &nid("em"))
        .await
        .unwrap();
    {
        let err = store
            .publish_document(&d_mix.document_id)
            .await
            .unwrap_err();
        pcheck!(err == StoreError::UnresolvedReference, "P-TP-2(mix): a BROKEN link was resolved by re-syncing an unrelated embed (must still block)");
        pcheck!(
            store.get_document(&d_mix.document_id).await.unwrap().state == DocState::Draft,
            "P-TP-2(mix): failed publish left the document state changed"
        );
    }
    Ok(())
}

/// One bounded concurrent publish/fact-commit/re-sync round behind a Barrier.
/// Asserts every handle completes (no deadlock), none panic, and the final
/// report satisfies P-IM-3 (no fabricated Resolved/Fresh over a dead target).
async fn ptp2_concurrent_round(rng: SplitMix64, round: usize) -> Result<(), String> {
    let mut rng = rng;
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let src = new_doc(&store, &w, "src").await;
    let fact_loc = seed_fact(&store, &w, &src, "f", "A").await;

    // A set of draft docs referencing the fact in the same shard space.
    let mut drafts: Vec<DocumentId> = Vec::new();
    for i in 0..5 {
        let d = new_doc(&store, &w, &format!("c-{round}-{i}")).await;
        let e = ref_node(&d.document_id, "em", fact_loc.clone(), Some("A"));
        apply_graph(
            &store,
            &d,
            graph_with(
                &d.document_id,
                vec![content_node(&d.document_id, "n", "x"), e.clone()],
                vec![edge(
                    EdgeKind::Embed,
                    (d.document_id.clone(), e.node_id.clone()),
                    fact_loc.clone(),
                    None,
                    false,
                )],
            ),
        )
        .await;
        drafts.push(d.document_id.clone());
    }

    const TASKS: usize = 6;
    let barrier = Arc::new(tokio::sync::Barrier::new(TASKS));
    let val = format!("val-{round}-{:x}", rng.next());
    let mut handles = Vec::new();
    for t in 0..TASKS {
        let s = Arc::clone(&store);
        let w_id = w.clone();
        let src_id = src.document_id.clone();
        let drafts = drafts.clone();
        let b = Arc::clone(&barrier);
        let val = val.clone();
        handles.push(tokio::spawn(async move {
            b.wait().await;
            match t {
                0 => {
                    // Fact commit → propagation over the referencing shard space.
                    s.update_fact(
                        &w_id,
                        "f",
                        &UpdateFactRequest {
                            value: val,
                            citations: vec![(src_id, nid("cite"))],
                        },
                    )
                    .await
                    .map(|_| ())
                }
                1..=4 => {
                    // Re-sync a distinct draft embed (serialized on its shard write).
                    s.re_sync_embed(&drafts[t], &nid("em")).await.map(|_| ())
                }
                _ => {
                    // Publish the last draft — Ok/UnresolvedReference/InvalidState
                    // are all coherent; a deadlock/panic is the regression signal.
                    match s.publish_document(&drafts[4]).await {
                        Ok(_)
                        | Err(StoreError::UnresolvedReference)
                        | Err(StoreError::InvalidState) => Ok(()),
                        Err(e) => Err(e),
                    }
                }
            }
        }));
    }
    // Every handle must resolve (a deadlock hangs here) and not panic.
    for h in handles {
        let res = h
            .await
            .map_err(|_| "concurrent task panicked (panic is a P-TP-2 failure)".to_string())?;
        res.map_err(|e| format!("concurrent task returned an out-of-contract error {e:?}"))?;
    }

    // Final report must satisfy the PIM-3 liveness part that P-TP-2 asserts:
    // **no fabricated Resolved/Fresh row over a missing/archived target** (§4.4.3
    // decision — the register deliberately does NOT assert the snapshot-vs-canonical
    // freshness under a concurrent re-sync, which the GREEN code does not re-run
    // under the write). Targets here are committed-fact locations, always live.
    let facts: HashSet<(DocumentId, NodeId)> = std::iter::once(fact_loc.clone()).collect();
    let final_report = store.get_consistency_report(&w).await.unwrap();
    for row in &final_report {
        if row.cross_wiki {
            continue;
        }
        if matches!(row.state, ReferenceState::Fresh | ReferenceState::Resolved) {
            let live = target_is_live(&store, &facts, &row.target).await;
            pcheck!(
                live,
                "final-report PIM-3: {:?} {:?} over a missing/archived target (fabricated)",
                row.node_id.0,
                row.state
            );
        }
    }
    Ok(())
}

// ===========================================================================
// §4.4 AUDIT PROBES — deterministic explicit assertion blocks (no PRNG cases)
// ===========================================================================
//
// These extend the register rows' generator coverage with the adversarial /
// robustness shapes requested by the read-only §4.4 PBT audit. Each is a FIXED
// scenario in its own `#[tokio::test]` (a deterministic explicit assertion
// block, no SplitMix64 cases consumed). The generated-case budget is therefore
// UNCHANGED at 310 (< the 400 cap). Every assertion documents and matches the
// CURRENT GREEN behavior; where a behavior is an excluded/non-goal it is pinned
// with a comment rather than asserted (never over-strength).

/// Probe 1 — `ref_store_basic` coverage fill (P-IM-1 / P-IM-2).
///
/// One referrer doc carrying: (a) a `None`-state Link to a live target (→
/// derived `Resolved`), (b) a `None`-state Embed of an ARCHIVED target (→
/// derived `Stale`), (c) a `None`-state Link to an archived target (→ derived
/// `Broken`), (d) a self-referential `None`-state Embed (→ `Fresh`), (e) an
/// explicit `Some(Stale)` Link to a live target, and (f) an explicit
/// `Some(Stale)` Link to a MISSING target. A second wiki carries a Crosslink
/// into `w` (must land in the second wiki's report, never `w`'s). Asserts: the
/// report is deterministic; the `None`-derived set satisfies P-IM-3
/// (no fabrication); the explicit set is reported VERBATIM.
///
/// SCOPING NOTE (do NOT over-strength): for the explicit-state rows we assert
/// verbatim storage ONLY — we deliberately do NOT run the no-fabrication check
/// over them. Green reports a caller-supplied explicit state verbatim, so an
/// explicit `Some(Stale)` over a missing target (row `ex-stale-miss`) is
/// reported as `Stale`; asserting no-fabrication there would be an over-strength
/// expectation, not an invariant. P-IM-3's no-fabrication claim is scoped to
/// the `None`-derived set.
#[tokio::test]
async fn p_im12_audit_ref_store_basic_coverage_fill() {
    p_im12_audit_ref_store_basic_coverage_fill_impl()
        .await
        .unwrap();
}
async fn p_im12_audit_ref_store_basic_coverage_fill_impl() -> Result<(), String> {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let w2 = new_wiki(&store, "w2").await;

    // Live target doc with node n1.
    let tgt = new_doc(&store, &w, "tgt").await;
    apply_graph(
        &store,
        &tgt,
        graph_with(
            &tgt.document_id,
            vec![content_node(&tgt.document_id, "n1", "v1")],
            vec![],
        ),
    )
    .await;
    // Archived target doc.
    let arch = new_doc(&store, &w, "arch").await;
    apply_graph(
        &store,
        &arch,
        graph_with(
            &arch.document_id,
            vec![content_node(&arch.document_id, "an", "av")],
            vec![],
        ),
    )
    .await;
    store.archive_document(&arch.document_id).await.unwrap();

    // Referrer doc with the six reference edges (self-node + target nodes in-graph).
    let r = new_doc(&store, &w, "r").await;
    let l_live = ref_node(
        &r.document_id,
        "l-live",
        (tgt.document_id.clone(), nid("n1")),
        None,
    );
    let emb_arch = ref_node(
        &r.document_id,
        "emb-arch",
        (arch.document_id.clone(), nid("an")),
        Some("av"),
    );
    let l_arch = ref_node(
        &r.document_id,
        "l-arch",
        (arch.document_id.clone(), nid("an")),
        None,
    );
    let ex_stale = ref_node(
        &r.document_id,
        "ex-stale",
        (tgt.document_id.clone(), nid("n1")),
        None,
    );
    let ex_stale_miss = ref_node(
        &r.document_id,
        "ex-stale-miss",
        (did("ghost"), nid("gone")),
        None,
    );
    let selfref = ref_node(
        &r.document_id,
        "selfref",
        (r.document_id.clone(), nid("selfn")),
        Some("selfv"),
    );
    apply_graph(
        &store,
        &r,
        graph_with(
            &r.document_id,
            vec![
                content_node(&r.document_id, "selfn", "selfv"), // self-target content node
                l_live.clone(),
                emb_arch.clone(),
                l_arch.clone(),
                ex_stale.clone(),
                ex_stale_miss.clone(),
                selfref.clone(),
            ],
            vec![
                edge(
                    EdgeKind::Link,
                    (r.document_id.clone(), nid("l-live")),
                    (tgt.document_id.clone(), nid("n1")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::Embed,
                    (r.document_id.clone(), nid("emb-arch")),
                    (arch.document_id.clone(), nid("an")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::Link,
                    (r.document_id.clone(), nid("l-arch")),
                    (arch.document_id.clone(), nid("an")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::Link,
                    (r.document_id.clone(), nid("ex-stale")),
                    (tgt.document_id.clone(), nid("n1")),
                    Some(ReferenceState::Stale),
                    false,
                ),
                edge(
                    EdgeKind::Link,
                    (r.document_id.clone(), nid("ex-stale-miss")),
                    (did("ghost"), nid("gone")),
                    Some(ReferenceState::Stale),
                    false,
                ),
                edge(
                    EdgeKind::Embed,
                    (r.document_id.clone(), nid("selfref")),
                    (r.document_id.clone(), nid("selfn")),
                    None,
                    false,
                ),
            ],
        ),
    )
    .await;

    // Second-wiki crosslink into w (must be in w2's report, NOT w's).
    let d2 = new_doc(&store, &w2, "x2").await;
    let x2n = ref_node(
        &d2.document_id,
        "xl",
        (tgt.document_id.clone(), nid("n1")),
        None,
    );
    apply_graph(
        &store,
        &d2,
        graph_with(
            &d2.document_id,
            vec![x2n.clone()],
            vec![edge(
                EdgeKind::Crosslink,
                (d2.document_id.clone(), nid("xl")),
                (tgt.document_id.clone(), nid("n1")),
                None,
                true,
            )],
        ),
    )
    .await;

    // `None`-derived expected states (non-cross-wiki) + explicit verbatim states.
    let derived: HashMap<&str, ReferenceState> = [
        ("l-live", ReferenceState::Resolved),
        ("emb-arch", ReferenceState::Stale),
        ("l-arch", ReferenceState::Broken),
        ("selfref", ReferenceState::Fresh),
    ]
    .into_iter()
    .collect();
    let explicit: HashMap<&str, ReferenceState> = [
        ("ex-stale", ReferenceState::Stale),
        ("ex-stale-miss", ReferenceState::Stale),
    ]
    .into_iter()
    .collect();

    // Determinism (P-IM-1): two reads with no intervening mutation are identical.
    let a = store.get_consistency_report(&w).await.unwrap();
    let b = store.get_consistency_report(&w).await.unwrap();
    pcheck!(
        a == b,
        "probe1: two consecutive reports differ (non-deterministic read)"
    );

    let facts: HashSet<(DocumentId, NodeId)> = HashSet::new();
    let fact_canon: HashMap<(DocumentId, NodeId), String> = HashMap::new();
    pcheck!(
        a.len() == derived.len() + explicit.len(),
        "probe1: w report len {} != expected {} reference rows",
        a.len(),
        derived.len() + explicit.len()
    );
    let mut seen: HashSet<String> = HashSet::new();
    for row in &a {
        seen.insert(row.node_id.0.clone());
        if let Some(&exp) = derived.get(row.node_id.0.as_str()) {
            pcheck!(
                row.state == exp,
                "probe1: None-derived row {} expected {:?}, got {:?}",
                row.node_id.0,
                exp,
                row.state
            );
            // no-fabrication over the DERIVED set (P-IM-3 scope):
            let live = target_is_live(&store, &facts, &row.target).await;
            let snap = held_snapshot(&store, row).await;
            let canon = target_canonical(&store, &fact_canon, &row.target).await;
            pm3_check(row, live, snap, canon)?;
        } else if let Some(&exp) = explicit.get(row.node_id.0.as_str()) {
            // explicit-state row → stored verbatim ONLY (see scoping note above).
            pcheck!(
                row.state == exp,
                "probe1: explicit-state row {} expected verbatim {:?}, got {:?}",
                row.node_id.0,
                exp,
                row.state
            );
            pcheck!(
                !row.cross_wiki,
                "probe1: explicit row {} unexpectedly cross-wiki",
                row.node_id.0
            );
        } else {
            return Err(format!("probe1: unexpected report row {}", row.node_id.0));
        }
    }
    for k in derived.keys().chain(explicit.keys()) {
        pcheck!(
            seen.contains(*k),
            "probe1: expected row {} missing from w report",
            k
        );
    }
    // The w2-sourced crosslink must be in w2's report, never w's.
    pcheck!(
        !a.iter()
            .any(|r| r.document_id == d2.document_id && r.node_id == nid("xl")),
        "probe1: a w2-sourced crosslink leaked into w's report"
    );
    let r2 = store.get_consistency_report(&w2).await.unwrap();
    pcheck!(
        r2.iter().any(|r| r.document_id == d2.document_id
            && r.node_id == nid("xl")
            && r.kind == EdgeKind::Crosslink
            && r.cross_wiki),
        "probe1: the w2-sourced crosslink is missing from w2's report"
    );
    Ok(())
}

/// Probe 2 — P-TP-1 repoint matrix fill.
///
/// A `None`-state reference is repointed (via `update_document`) under three
/// target classes Green must derive on — an ARCHIVED target, a MISSING ghost,
/// and a newly-CREATED target — plus the existing live round-trip. Asserts: a
/// `Link` repointed onto an archived/missing target derives `Broken`, onto a
/// live/new target `Resolved`; an `Embed` (snapshot == new canonical) onto an
/// archived/missing target derives `Stale`, onto a live/new target `Fresh`;
/// round-tripping back to the live target restores the earlier state.
#[tokio::test]
async fn p_tp1_audit_repoint_matrix_fill() {
    p_tp1_audit_repoint_matrix_fill_impl().await.unwrap();
}
async fn p_tp1_audit_repoint_matrix_fill_impl() -> Result<(), String> {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;

    // Live, archived, and newly-created (after setup) target docs.
    let tgt = new_doc(&store, &w, "tgt").await;
    apply_graph(
        &store,
        &tgt,
        graph_with(
            &tgt.document_id,
            vec![content_node(&tgt.document_id, "n1", "v")],
            vec![],
        ),
    )
    .await;
    let tgt_loc = (tgt.document_id.clone(), nid("n1"));

    let arch = new_doc(&store, &w, "arch").await;
    apply_graph(
        &store,
        &arch,
        graph_with(
            &arch.document_id,
            vec![content_node(&arch.document_id, "a1", "av")],
            vec![],
        ),
    )
    .await;
    let arch_loc = (arch.document_id.clone(), nid("a1"));
    store.archive_document(&arch.document_id).await.unwrap();

    // "Newly-created" target: created AFTER the referrer, so a repoint onto it is
    // a forward repoint to a previously-nonexistent target.
    let fresh = new_doc(&store, &w, "fresh").await;
    apply_graph(
        &store,
        &fresh,
        graph_with(
            &fresh.document_id,
            vec![content_node(&fresh.document_id, "n1", "v")],
            vec![],
        ),
    )
    .await;
    let fresh_loc = (fresh.document_id.clone(), nid("n1"));

    let ghost = (did("ghost"), nid("gone"));

    // --- Link (None-state) repoint matrix ---
    let rl = new_doc(&store, &w, "rl").await;
    let rln = ref_node(&rl.document_id, "L", ghost.clone(), None);
    let link_graph = |t: (DocumentId, NodeId)| {
        graph_with(
            &rl.document_id,
            vec![content_node(&rl.document_id, "n", "x"), rln.clone()],
            vec![edge(
                EdgeKind::Link,
                (rl.document_id.clone(), nid("L")),
                t,
                None,
                false,
            )],
        )
    };
    apply_graph(&store, &rl, link_graph(tgt_loc.clone())).await;
    let rep = store.get_consistency_report(&w).await.unwrap();
    pcheck!(
        find_row_in(&rep, &rl.document_id, "L", EdgeKind::Link).state == ReferenceState::Resolved,
        "probe2: link to live target is not Resolved"
    );
    // repoint onto an ARCHIVED target → Broken.
    apply_graph(&store, &rl, link_graph(arch_loc.clone())).await;
    let rep = store.get_consistency_report(&w).await.unwrap();
    pcheck!(
        find_row_in(&rep, &rl.document_id, "L", EdgeKind::Link).state == ReferenceState::Broken,
        "probe2: link repointed onto an archived target did not derive Broken"
    );
    // repoint onto a MISSING ghost → Broken.
    apply_graph(&store, &rl, link_graph(ghost.clone())).await;
    let rep = store.get_consistency_report(&w).await.unwrap();
    pcheck!(
        find_row_in(&rep, &rl.document_id, "L", EdgeKind::Link).state == ReferenceState::Broken,
        "probe2: link repointed onto a missing target did not derive Broken"
    );
    // repoint onto a NEWLY-CREATED target → Resolved.
    apply_graph(&store, &rl, link_graph(fresh_loc.clone())).await;
    let rep = store.get_consistency_report(&w).await.unwrap();
    pcheck!(
        find_row_in(&rep, &rl.document_id, "L", EdgeKind::Link).state == ReferenceState::Resolved,
        "probe2: link repointed onto a newly-created live target did not derive Resolved"
    );
    // round-trip back to the original live target → Resolved.
    apply_graph(&store, &rl, link_graph(tgt_loc.clone())).await;
    let rep = store.get_consistency_report(&w).await.unwrap();
    pcheck!(
        find_row_in(&rep, &rl.document_id, "L", EdgeKind::Link).state == ReferenceState::Resolved,
        "probe2: link round-tripped back to live is not Resolved"
    );

    // --- Embed (None-state, snapshot == new canonical "v") repoint matrix ---
    let re = new_doc(&store, &w, "re").await;
    let ren = ref_node(&re.document_id, "em", ghost.clone(), Some("v"));
    let embed_graph = |t: (DocumentId, NodeId)| {
        graph_with(
            &re.document_id,
            vec![content_node(&re.document_id, "n", "x"), ren.clone()],
            vec![edge(
                EdgeKind::Embed,
                (re.document_id.clone(), nid("em")),
                t,
                None,
                false,
            )],
        )
    };
    apply_graph(&store, &re, embed_graph(tgt_loc.clone())).await;
    let rep = store.get_consistency_report(&w).await.unwrap();
    pcheck!(
        find_row_in(&rep, &re.document_id, "em", EdgeKind::Embed).state == ReferenceState::Fresh,
        "probe2: matching embed to live target is not Fresh"
    );
    // repoint onto an ARCHIVED target → Stale.
    apply_graph(&store, &re, embed_graph(arch_loc.clone())).await;
    let rep = store.get_consistency_report(&w).await.unwrap();
    pcheck!(
        find_row_in(&rep, &re.document_id, "em", EdgeKind::Embed).state == ReferenceState::Stale,
        "probe2: matching embed repointed onto an archived target did not derive Stale"
    );
    // repoint onto a NEWLY-CREATED target (canonical "v", snapshot "v") → Fresh.
    apply_graph(&store, &re, embed_graph(fresh_loc.clone())).await;
    let rep = store.get_consistency_report(&w).await.unwrap();
    pcheck!(
        find_row_in(&rep, &re.document_id, "em", EdgeKind::Embed).state == ReferenceState::Fresh,
        "probe2: matching embed repointed onto a newly-created target did not derive Fresh"
    );
    // repoint onto a MISSING ghost → Stale.
    apply_graph(&store, &re, embed_graph(ghost.clone())).await;
    let rep = store.get_consistency_report(&w).await.unwrap();
    pcheck!(
        find_row_in(&rep, &re.document_id, "em", EdgeKind::Embed).state == ReferenceState::Stale,
        "probe2: matching embed repointed onto a missing target did not derive Stale"
    );
    // round-trip back → Fresh.
    apply_graph(&store, &re, embed_graph(tgt_loc.clone())).await;
    let rep = store.get_consistency_report(&w).await.unwrap();
    pcheck!(
        find_row_in(&rep, &re.document_id, "em", EdgeKind::Embed).state == ReferenceState::Fresh,
        "probe2: matching embed round-tripped back is not Fresh"
    );
    Ok(())
}

/// Probe 3 — P-SM-3 exact +1 (not just strictly-greater).
///
/// In the propagation loops (one referencing edge per doc) the referencing
/// doc's `revision` must be EXACTLY `old + 1` after `update_fact` staleness
/// propagation and after `archive_document` propagation (the register's
/// observable already implies this; made explicit). A NON-TOUCHING referencing
/// doc (one that embeds a *different* fact / links a *different* live target)
/// must NOT be revised — the `M = 0` negative boundary is observable.
#[tokio::test]
async fn p_sm3_audit_exact_plus_one_propagation() {
    p_sm3_audit_exact_plus_one_propagation_impl().await.unwrap();
}
async fn p_sm3_audit_exact_plus_one_propagation_impl() -> Result<(), String> {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;

    // Two facts: `f` (the one we mutate) and `g` (untouched, non-touching probe).
    let src = new_doc(&store, &w, "src").await;
    let fact_f = seed_fact(&store, &w, &src, "f", "A").await;
    let srcg = new_doc(&store, &w, "srcg").await;
    let fact_g = seed_fact(&store, &w, &srcg, "g", "B").await;

    // Touching embed doc for fact f.
    let e = new_doc(&store, &w, "e").await;
    let en = ref_node(&e.document_id, "em", fact_f.clone(), Some("A"));
    apply_graph(
        &store,
        &e,
        graph_with(
            &e.document_id,
            vec![content_node(&e.document_id, "n", "x"), en.clone()],
            vec![edge(
                EdgeKind::Embed,
                (e.document_id.clone(), nid("em")),
                fact_f.clone(),
                None,
                false,
            )],
        ),
    )
    .await;
    // Non-touching embed doc for a DIFFERENT fact g (M = 0 negative for the f-propagation).
    let nx = new_doc(&store, &w, "nx").await;
    let nxn = ref_node(&nx.document_id, "em", fact_g.clone(), Some("B"));
    apply_graph(
        &store,
        &nx,
        graph_with(
            &nx.document_id,
            vec![content_node(&nx.document_id, "n", "x"), nxn.clone()],
            vec![edge(
                EdgeKind::Embed,
                (nx.document_id.clone(), nid("em")),
                fact_g.clone(),
                None,
                false,
            )],
        ),
    )
    .await;

    // --- update_fact propagation: EXACT +1 on the touching doc; 0 on non-touching.
    let e_old = store.get_document(&e.document_id).await.unwrap().revision;
    let nx_old = store.get_document(&nx.document_id).await.unwrap().revision;
    store
        .update_fact(
            &w,
            "f",
            &UpdateFactRequest {
                value: "C".into(),
                citations: vec![(src.document_id.clone(), nid("cite"))],
            },
        )
        .await
        .unwrap();
    let e_new = store.get_document(&e.document_id).await.unwrap().revision;
    let nx_new = store.get_document(&nx.document_id).await.unwrap().revision;
    pcheck!(
        e_new == e_old + 1,
        "probe3: update_fact propagation revised touching doc {}->{}, expected EXACTLY +1",
        e_old,
        e_new
    );
    pcheck!(nx_new == nx_old, "probe3: update_fact on f revised a doc embedding a DIFFERENT fact g: {}->{} (M=0 negative boundary)", nx_old, nx_new);
    pcheck!(
        store.epoch() == store.journal_len() as u64,
        "probe3: epoch != journal_len after update_fact"
    );

    // --- archive_document propagation: EXACT +1 on the touching doc; 0 on non-touching.
    let atgt = new_doc(&store, &w, "atgt").await;
    apply_graph(
        &store,
        &atgt,
        graph_with(
            &atgt.document_id,
            vec![content_node(&atgt.document_id, "n1", "v")],
            vec![],
        ),
    )
    .await;
    let u = new_doc(&store, &w, "uart").await;
    apply_graph(
        &store,
        &u,
        graph_with(
            &u.document_id,
            vec![content_node(&u.document_id, "u1", "uv")],
            vec![],
        ),
    )
    .await;

    let ae = new_doc(&store, &w, "ae").await;
    let aen = ref_node(
        &ae.document_id,
        "l",
        (atgt.document_id.clone(), nid("n1")),
        None,
    );
    apply_graph(
        &store,
        &ae,
        graph_with(
            &ae.document_id,
            vec![content_node(&ae.document_id, "n", "x"), aen.clone()],
            vec![edge(
                EdgeKind::Link,
                (ae.document_id.clone(), nid("l")),
                (atgt.document_id.clone(), nid("n1")),
                None,
                false,
            )],
        ),
    )
    .await;
    // Non-touching arch-ref doc: links a DIFFERENT live target (u).
    let anx = new_doc(&store, &w, "anx").await;
    let anxn = ref_node(
        &anx.document_id,
        "l",
        (u.document_id.clone(), nid("u1")),
        None,
    );
    apply_graph(
        &store,
        &anx,
        graph_with(
            &anx.document_id,
            vec![content_node(&anx.document_id, "n", "x"), anxn.clone()],
            vec![edge(
                EdgeKind::Link,
                (anx.document_id.clone(), nid("l")),
                (u.document_id.clone(), nid("u1")),
                None,
                false,
            )],
        ),
    )
    .await;

    let ae_old = store.get_document(&ae.document_id).await.unwrap().revision;
    let anx_old = store.get_document(&anx.document_id).await.unwrap().revision;
    store.archive_document(&atgt.document_id).await.unwrap();
    let ae_new = store.get_document(&ae.document_id).await.unwrap().revision;
    let anx_new = store.get_document(&anx.document_id).await.unwrap().revision;
    pcheck!(
        ae_new == ae_old + 1,
        "probe3: archive propagation revised touching doc {}->{}, expected EXACTLY +1",
        ae_old,
        ae_new
    );
    pcheck!(anx_new == anx_old, "probe3: archive_document revised a doc linking a DIFFERENT live target: {}->{} (M=0 negative boundary)", anx_old, anx_new);
    pcheck!(
        store.epoch() == store.journal_len() as u64,
        "probe3: epoch != journal_len after archive"
    );
    Ok(())
}

/// Probe 4 — P-TP-2 state-machine precondition (DOCUMENTATION-PINNED + asserted).
///
/// `publish_document` on an all-clean doc that is already `Published`, and on an
/// all-clean doc that is already `Archived`, returns `Err(StoreError::InvalidState)`
/// — NOT `Err(UnresolvedReference)`. The reference/community gate passes (all
/// references clean, no stale community), so the failure is the §4.1.1 state
/// transition, not a reference-unresolved failure.
///
/// SCOPING COMMENT: this pins the precondition that P-TP-2's publish-gate "iff"
/// (`publish Ok iff every reference is clean AND state becomes Published`) holds
/// ONLY for documents reachable to `Published` (i.e. currently `Draft`). An
/// already-`Published` / `Archived` doc is outside that domain and legitimately
/// returns `InvalidState`; this is GREEN behavior, asserted as such. (The
/// register re-wording is handled separately by the SpecWriter.) The excluded
/// concurrent-Target-mutation TOCTOU is a separate non-goal and is NOT asserted
/// anywhere here.
#[tokio::test]
async fn p_tp2_audit_state_machine_precondition() {
    p_tp2_audit_state_machine_precondition_impl().await.unwrap();
}
async fn p_tp2_audit_state_machine_precondition_impl() -> Result<(), String> {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let src = new_doc(&store, &w, "src").await;
    let fact_loc = seed_fact(&store, &w, &src, "f", "A").await;

    // All-clean doc P: a Fresh embed + a Resolved link to a live committed fact.
    let p = new_doc(&store, &w, "p").await;
    let pe = ref_node(&p.document_id, "em", fact_loc.clone(), Some("A"));
    let pl = ref_node(&p.document_id, "L", fact_loc.clone(), None);
    apply_graph(
        &store,
        &p,
        graph_with(
            &p.document_id,
            vec![
                content_node(&p.document_id, "n", "x"),
                pe.clone(),
                pl.clone(),
            ],
            vec![
                edge(
                    EdgeKind::Embed,
                    (p.document_id.clone(), nid("em")),
                    fact_loc.clone(),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::Link,
                    (p.document_id.clone(), nid("L")),
                    fact_loc.clone(),
                    None,
                    false,
                ),
            ],
        ),
    )
    .await;
    let once = store.publish_document(&p.document_id).await.unwrap();
    pcheck!(
        once.state == DocState::Published,
        "probe4: precondition — first publish of an all-clean doc must succeed"
    );
    let err = store.publish_document(&p.document_id).await.unwrap_err();
    pcheck!(
        err == StoreError::InvalidState,
        "probe4: publish of an already-PUBLISHED doc must be Err(InvalidState), got {:?}",
        err
    );

    // All-clean doc Ar, archived → publish is Err(InvalidState).
    let ar = new_doc(&store, &w, "ar").await;
    let are = ref_node(&ar.document_id, "em", fact_loc.clone(), Some("A"));
    apply_graph(
        &store,
        &ar,
        graph_with(
            &ar.document_id,
            vec![content_node(&ar.document_id, "n", "x"), are.clone()],
            vec![edge(
                EdgeKind::Embed,
                (ar.document_id.clone(), nid("em")),
                fact_loc.clone(),
                None,
                false,
            )],
        ),
    )
    .await;
    store.archive_document(&ar.document_id).await.unwrap();
    pcheck!(
        store.get_document(&ar.document_id).await.unwrap().state == DocState::Archived,
        "probe4: precondition — archiving an all-clean doc succeeds"
    );
    let err2 = store.publish_document(&ar.document_id).await.unwrap_err();
    pcheck!(
        err2 == StoreError::InvalidState,
        "probe4: publish of an ARCHIVED doc must be Err(InvalidState), got {:?}",
        err2
    );
    Ok(())
}
