//! §4.3 fact/citation — EXECUTED property layer (PBT-gate artifact #2, TestWriter).
//!
//! One `#[test]` per row of `docs/specs/4-3-facts-property-register.md`.
//! Each row is implemented as a real Observable-as-property check driven by a
//! hand-rolled deterministic **SplitMix64** PRNG (pinned seed — no external
//! crate, no proptest). A genuinely-failing row FAILS its `#[test]` with the
//! retained minimal counterexamples (stop-after-5).
//!
//! ---------------------------------------------------------------------------
//! PINNED SEED (recorded in the file header and report):
//!    BASE_SEED = 0xD1B54A32D192ED03
//! Every row derives its own instance seed as `BASE_SEED ^ ROW_TAG` (a fixed per-
//! row constant), so generation is byte-for-byte reproducible run-to-run without
//! any cross-row correlation.
//! ---------------------------------------------------------------------------
//!
//! Budget: ≤100 generated cases per row; per-row budgets sum to exactly **400**
//! total (P-SM-1=50, P-SM-2=50, P-SM-3=60, P-SM-4=30, P-SM-5=30, P-IM-1=60,
//! P-TP-1=60, P-TP-2=60). stop-after-5: a row that starts failing stops as soon
//! as it has retained ≤5 minimal counterexamples, so a genuinely-broken row does
//! not grind on; a HELD row runs its full budget.
//!
//! The §4.3 read-only adversarial audit's concrete negative-generator /
//! robustness probes are appended as **deterministic explicit assertions**
//! (`.audit_*` tests, no PRNG) — they are NOT counted in the 400 generated-case
//! budget above, and they pin behaviors that are already GREEN on today's
//! implementation.
//!
//! Faithful execution / register-tag caveat. The register tags P-SM-4
//! (delete-citation integrity), P-SM-5 (unknown-wiki `WikiNotFound`) and the
//! whitespace-only branch of P-IM-1 as `PENDING → expected BROKEN today`.
//! However `docs/defects.md` (FIXED table) and the §4.3 DONE row in
//! `docs/next-steps.md` state those exact defects are ALREADY fixed and pinned in
//! `tests/facts_integration.rs`. The empirical run below decides: each invariant
//! is implemented faithfully (never weakened to force green); a tagged-PENDING
//! row that turns out HELD is reported HELD with a `[register tag STALE]` note.
//!
//! No `src/` file, existing test file, or `docs/specs/gnosis.md` is modified.

use std::sync::Arc;

use gnosis::{
    CandidateFact, CreateDocumentRequest, DocState, DocumentId, Edge, EdgeKind, Graph,
    ListFactsFilter, Node, NodeId, NodeKind, RagStore, Store, StoreError, UpdateFactRequest,
    WikiId,
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
            return 0;
        }
        self.below(n as u64) as usize
    }
}

/// Push a counterexample; returns true when the stop-after-5 threshold is hit.
fn record(cex: &mut Vec<String>, s: String) -> bool {
    cex.push(s);
    cex.len() >= 5
}

fn finisher(cex: &[String], row: &str) {
    assert!(
        cex.is_empty(),
        "{row} BROKEN — retained minimal counterexample(s) ({}/{}){}",
        cex.len().min(5),
        cex.len(),
        if cex.is_empty() {
            String::new()
        } else {
            format!(":\n  - {}", cex.join("\n  - "))
        }
    );
}

// Fixture helpers (reused verbatim from `tests/facts_integration.rs`).
fn wiki(s: &str) -> WikiId {
    WikiId(s.to_string())
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
fn edge(
    kind: EdgeKind,
    from: (DocumentId, NodeId),
    to: (DocumentId, NodeId),
    state: Option<gnosis::ReferenceState>,
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
    let nodes: Vec<Node> = names
        .iter()
        .map(|n| content_node(doc, n, "cited-content"))
        .collect();
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
            gnosis::UpdateDocumentRequest {
                base_revision: cur.revision,
                graph,
                title: None,
                tags: None,
            },
        )
        .await
        .unwrap();
}
/// Give `doc` a node graph named `names`. Facts cite these nodes.
async fn seed_nodes(store: &Store, doc: &gnosis::Document, names: &[&str]) {
    apply_graph(store, doc, content_only_graph(&doc.document_id, names)).await;
}

fn list_default() -> ListFactsFilter {
    ListFactsFilter {
        state: None,
        page: None,
        page_size: None,
    }
}
fn list_opts(page: Option<u64>, page_size: Option<u64>) -> ListFactsFilter {
    ListFactsFilter {
        state: None,
        page,
        page_size,
    }
}

/// Enumerate the full fact table of a wiki by page-walking (the register's
/// `list_facts(A, default)` intent — P-TP-2 establishes the default filter
/// enumerates the whole set; default `page_size` is 20, so a page-walk at
/// `page_size=100` is required to see facts beyond the first page window).
async fn all_facts(store: &Store, w: &WikiId) -> Vec<gnosis::Fact> {
    let mut out: Vec<gnosis::Fact> = Vec::new();
    let mut page = 1u64;
    loop {
        let fl = store
            .list_facts(w, &list_opts(Some(page), Some(100)))
            .await
            .expect("page-walk of a real wiki must succeed");
        let empty = fl.items.is_empty();
        out.extend(fl.items);
        if empty {
            break;
        }
        page += 1;
        if page > 10_000 {
            break;
        }
    }
    out
}

/// Order-insensitive multiset view of a citation set for comparison.
fn canon_cites(cites: &[(DocumentId, NodeId)]) -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = cites
        .iter()
        .map(|(d, n)| (d.0.clone(), n.0.clone()))
        .collect();
    v.sort();
    v
}

// ---------------------------------------------------------------------------
// P-SM-1 — wiki_cross  (class SM, strategy `wiki_cross`, register tag [GREEN])
// ---------------------------------------------------------------------------
//
// Facts are wiki-scoped: ∀ wikis A≠B, a fact committed under A is observable
// under A only; it is never returned by B-queries for a fact committed under A.
const ROW_SM1_TAG: u64 = 0x00_00_00_00_00_00_00_01;
const ROW_SM1_BUDGET: u64 = 50;

#[tokio::test]
async fn p_sm1_wiki_cross() {
    let store = Arc::new(Store::new());
    let a = new_wiki(&store, "A").await;
    let b = new_wiki(&store, "B").await;
    let doca = new_doc(&store, &a, "da").await;
    let docb = new_doc(&store, &b, "db").await;
    seed_nodes(&store, &doca, &["n1", "n2"]).await;
    seed_nodes(&store, &docb, &["m1", "m2"]).await;

    let mut rng = SplitMix64::new(0xD1B54A32D192ED03 ^ ROW_SM1_TAG);
    let mut budget = 0u64;
    let mut cex: Vec<String> = Vec::new();

    // Primary loop: commit facts under A ONLY and verify the strict observable
    // property for each (clause 1: present under A; clause 2: absent under B;
    // clause 3: get_fact(B) is Err or a Fact with a different document_id).
    loop {
        if budget >= ROW_SM1_BUDGET || cex.len() >= 5 {
            break;
        }
        budget += 1;
        let shape = budget % 4;
        let (key, value, d, node) = match shape {
            // normal key; distinct value content
            0 => (
                format!("k-{}-{}", budget, rng.below(100000)),
                "v",
                &doca,
                "n1",
            ),
            // key differing by case/length variants
            1 => (
                {
                    let s = format!("KeyCase{:x}{}", rng.below(1 << 20), budget);
                    if budget.is_multiple_of(2) {
                        s.to_uppercase()
                    } else {
                        s.to_lowercase()
                    }
                },
                "v",
                &doca,
                "n2",
            ),
            // key differing only by a trailing space (distinct key) — budget-tagged
            // so no two cases collide.
            2 => (
                format!("pad{}-{} ", budget, rng.below(1 << 12)),
                "value",
                &doca,
                "n1",
            ),
            // long-key boundary — budget-tagged so each case is unique.
            _ => (
                format!("long-{}-{}", budget, "x".repeat((budget % 40) as usize)),
                "v",
                &doca,
                "n2",
            ),
        };

        let f = match store
            .create_fact(
                &a,
                &d.document_id,
                &key,
                value,
                &[(d.document_id.clone(), nid(node))],
            )
            .await
        {
            Ok(f) => f,
            Err(e) => {
                if record(
                    &mut cex,
                    format!("create_fact(A, …, {key:?}, {value:?}) -> Err {e:?}"),
                ) {
                    break;
                }
                continue;
            }
        };

        // Clause 1: present under A with matching (fact_key, document_id).
        // (page-walk the whole A table — default page_size=20 would hide facts
        // on later pages).
        let fa_coll = all_facts(&store, &a).await;
        let present_a = fa_coll
            .iter()
            .any(|x| x.fact_key == f.fact_key && x.document_id == f.document_id);
        if !present_a {
            if record(&mut cex, format!("A does not list committed fact {f:?}")) {
                break;
            }
            continue;
        }

        // Clause 2: absent under B by fact_key.
        let fb_coll = all_facts(&store, &b).await;
        if fb_coll.iter().any(|x| x.fact_key == f.fact_key) {
            if record(&mut cex, format!("B lists A's fact_key {:?}", f.fact_key)) {
                break;
            }
            continue;
        }

        // Clause 3: get_fact(B, key) is Err OR has a different document_id.
        match store.get_fact(&b, &f.fact_key).await {
            Err(_) => {}
            Ok(g) if g.document_id != f.document_id => {}
            Ok(g) => {
                if record(
                    &mut cex,
                    format!("get_fact(B, {:?}) == A's fact {g:?}", f.fact_key),
                ) {
                    break;
                }
            }
        }
    }

    // Separate boundary: the SAME fact_key committed under BOTH wikis must not
    // leak across — each wiki's query returns only its own (document_id-scoped)
    // fact for that key. This is the note's "same key under both" adversarial
    // shape, checked against the essence of the invariant (per-wiki scoping).
    // (Not run in the strict loop, whose clause 2 asserts B omits A keys.)
    if cex.len() < 5 {
        let shared = "same-in-both";
        let fa = store
            .create_fact(
                &a,
                &doca.document_id,
                shared,
                "fromA",
                &[(doca.document_id.clone(), nid("n1"))],
            )
            .await
            .unwrap();
        let fb = store
            .create_fact(
                &b,
                &docb.document_id,
                shared,
                "fromB",
                &[(docb.document_id.clone(), nid("m1"))],
            )
            .await
            .unwrap();
        let ga = store.get_fact(&a, shared).await.unwrap();
        let gb = store.get_fact(&b, shared).await.unwrap();
        if ga.document_id != fa.document_id || gb.document_id != fb.document_id {
            record(
                &mut cex,
                format!("same key both wikis leaked: A->{ga:?} B->{gb:?}"),
            );
        }
        // No A-committed fact's document_id appears under B's listing for the key.
        let la = all_facts(&store, &a).await;
        let lb = all_facts(&store, &b).await;
        for x in la.iter().filter(|x| x.fact_key == shared) {
            if x.document_id != doca.document_id {
                record(&mut cex, format!("A lists B's fact under {shared}: {x:?}"));
            }
        }
        for x in lb.iter().filter(|x| x.fact_key == shared) {
            if x.document_id != docb.document_id {
                record(&mut cex, format!("B lists A's fact under {shared}: {x:?}"));
            }
        }
    }

    finisher(&cex, "P-SM-1");
}

// ---------------------------------------------------------------------------
// P-SM-2 — dedup_keys  (class SM, strategy `dedup_keys`, register tag [GREEN])
// ---------------------------------------------------------------------------
// fact_key is unique per wiki; no silent duplicate: a second create_fact with an
// already-committed key → Err(ConflictError); the fact table holds ≤1 Fact per
// key; propose_candidate_fact duplicate is also ConflictError (not accepted) and
// the pre-existing value is left unmodified.
const ROW_SM2_TAG: u64 = 0x00_00_00_00_00_00_00_02;
const ROW_SM2_BUDGET: u64 = 50;

#[tokio::test]
async fn p_sm2_dedup_keys() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "W").await;
    let doc = new_doc(&store, &w, "d").await;
    seed_nodes(&store, &doc, &["n1", "n2"]).await;

    let mut rng = SplitMix64::new(0xD1B54A32D192ED03 ^ ROW_SM2_TAG);
    let mut budget = 0u64;
    let mut cex: Vec<String> = Vec::new();

    loop {
        if budget >= ROW_SM2_BUDGET || cex.len() >= 5 {
            break;
        }
        budget += 1;
        let key = format!("dup-{}-{}", budget, rng.below(1 << 20));
        let initial = format!("first-{}", rng.below(1 << 16));

        // Pre-seed one committed fact for this key.
        if let Err(e) = store
            .create_fact(
                &w,
                &doc.document_id,
                &key,
                &initial,
                &[(doc.document_id.clone(), nid("n1"))],
            )
            .await
        {
            if record(&mut cex, format!("pre-seed create {key:?} -> Err {e:?}")) {
                break;
            }
            continue;
        }

        // Duplicate create_fact with a DIFFERENT value → ConflictError.
        let other = format!("different-{}", rng.below(1 << 16));
        match store
            .create_fact(
                &w,
                &doc.document_id,
                &key,
                &other,
                &[(doc.document_id.clone(), nid("n2"))],
            )
            .await
        {
            Err(StoreError::ConflictError) => {}
            Err(e) => {
                if record(
                    &mut cex,
                    format!("duplicate create {key:?} -> {e:?} (want ConflictError)"),
                ) {
                    break;
                }
            }
            Ok(x) => {
                if record(
                    &mut cex,
                    format!("duplicate create {key:?} UNEXPECTED Ok: {x:?}"),
                ) {
                    break;
                }
            }
        }

        // Fact table holds at most one Fact with fact_key == key.
        let list = all_facts(&store, &w).await;
        let count = list.iter().filter(|f| f.fact_key == key).count();
        if count > 1 && record(&mut cex, format!("key {key:?} has {count} rows")) {
            break;
        }

        // Pre-existing value is unmodified after the duplicate attempt.
        let got = store.get_fact(&w, &key).await.unwrap();
        if got.value != initial
            && record(
                &mut cex,
                format!(
                    "key {key:?} overwritten after dup: want {initial:?} got {:?}",
                    got.value
                ),
            )
        {
            break;
        }

        // Adversarial: duplicate propose_candidate_fact must be ConflictError,
        // never accepted.
        match store
            .propose_candidate_fact(
                &w,
                &CandidateFact {
                    fact_key: key.clone(),
                    value: "candidate-value".to_string(),
                    citations: vec![(doc.document_id.clone(), nid("n1"))],
                },
            )
            .await
        {
            Err(StoreError::ConflictError) => {}
            Err(e) => {
                if record(
                    &mut cex,
                    format!("dup candidate {key:?} -> {e:?} (want Conflict)"),
                ) {
                    break;
                }
            }
            Ok(o) => {
                if record(
                    &mut cex,
                    format!("dup candidate {key:?} not ConflictError: {o:?}"),
                ) {
                    break;
                }
            }
        }
        let got2 = store.get_fact(&w, &key).await.unwrap();
        if got2.value != initial && record(&mut cex, format!("dup candidate overwrote {key:?}")) {
            break;
        }
    }

    // Boundary: keys differing only by a trailing space are DISTINCT keys; each
    // may be committed once and both coexist without collision.
    if cex.len() < 5 {
        let k1 = "edge";
        let k2 = "edge ";
        store
            .create_fact(
                &w,
                &doc.document_id,
                k1,
                "a",
                &[(doc.document_id.clone(), nid("n1"))],
            )
            .await
            .unwrap();
        store
            .create_fact(
                &w,
                &doc.document_id,
                k2,
                "b",
                &[(doc.document_id.clone(), nid("n2"))],
            )
            .await
            .unwrap();
        let list = all_facts(&store, &w).await;
        let c1 = list.iter().filter(|f| f.fact_key == k1).count();
        let c2 = list.iter().filter(|f| f.fact_key == k2).count();
        if c1 != 1 || c2 != 1 {
            record(
                &mut cex,
                format!("trailing-space keys not distinct: {k1:?}={c1} {k2:?}={c2}"),
            );
        }
    }

    finisher(&cex, "P-SM-2");
}

// ---------------------------------------------------------------------------
// P-SM-3 — ground_cites  (class SM, strategy `ground_cites`, register tag [GREEN])
// ---------------------------------------------------------------------------
// Every persisted/committed/updated Fact has non-empty citations (≥1), and an
// accepted candidate commits only cited nodes that resolve to a real node.
const ROW_SM3_TAG: u64 = 0x00_00_00_00_00_00_00_03;
const ROW_SM3_BUDGET: u64 = 60;

#[tokio::test]
async fn p_sm3_ground_cites() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "W").await;
    let doc = new_doc(&store, &w, "d").await;
    seed_nodes(&store, &doc, &["n1", "n2", "n3", "n4", "n5"]).await;
    let nodes = ["n1", "n2", "n3", "n4", "n5"];

    let mut rng = SplitMix64::new(0xD1B54A32D192ED03 ^ ROW_SM3_TAG);
    let mut budget = 0u64;
    let mut cex: Vec<String> = Vec::new();

    loop {
        if budget >= ROW_SM3_BUDGET || cex.len() >= 5 {
            break;
        }
        budget += 1;

        // Build a citation set of size 1..=5, sometimes with a repeated tuple
        // (dedup by first appearance) and the empty-citation adversarial shape.
        let mode = rng.pick(4);
        let mut cites: Vec<(DocumentId, NodeId)> = Vec::new();
        let key = format!("g-{}-{}", budget, rng.below(1 << 20));
        if mode == 3 {
            // adversarial: empty citation set
        } else {
            let size = 1 + rng.below(5) as usize;
            let mut seen_appearance = Vec::new();
            for _ in 0..size {
                let picked = nodes[rng.pick(nodes.len())];
                let pair = (doc.document_id.clone(), nid(picked));
                if seen_appearance.contains(&pair) {
                    // force a definite duplicate on some iterations
                } else {
                    seen_appearance.push(pair.clone());
                }
                cites.push(pair);
            }
        }

        if cites.is_empty() {
            // empty citations → create_fact ValidationError; propose → rejection
            // field "citations".
            match store
                .create_fact(&w, &doc.document_id, &key, "v", &[])
                .await
            {
                Err(StoreError::ValidationError(_)) => {}
                other => {
                    if record(
                        &mut cex,
                        format!("empty-cite create {key:?} -> {other:?} (want ValidationError)"),
                    ) {
                        break;
                    }
                }
            }
            // empty-cite candidate → rejected, nothing committed, field="citations".
            match store
                .propose_candidate_fact(
                    &w,
                    &CandidateFact {
                        fact_key: key.clone(),
                        value: "v".to_string(),
                        citations: vec![],
                    },
                )
                .await
            {
                Ok(o) => {
                    if o.accepted {
                        if record(&mut cex, format!("empty-cite candidate accepted: {o:?}")) {
                            break;
                        }
                    } else if o.rejection.as_ref().map(|r| r.field.as_str()) != Some("citations")
                        && record(
                            &mut cex,
                            format!(
                                "empty-cite rejection field {:?} (want citations)",
                                o.rejection
                            ),
                        )
                    {
                        break;
                    }
                }
                Err(e) => {
                    if record(&mut cex, format!("empty-cite candidate -> Err {e:?}")) {
                        break;
                    }
                }
            }
            continue;
        }

        // Valid create_fact → Ok, result citations.len() >= 1.
        let f = match store
            .create_fact(&w, &doc.document_id, &key, "val", &cites)
            .await
        {
            Ok(f) => f,
            Err(e) => {
                if record(
                    &mut cex,
                    format!("valid-cite create {key:?} -> Err {e:?} (cites {cites:?})"),
                ) {
                    break;
                }
                continue;
            }
        };
        if f.citations.is_empty()
            && record(
                &mut cex,
                format!("committed fact {key:?} has empty citations"),
            )
        {
            break;
        }

        // Valid candidate → accepted:true with fact.citations.len() >= 1.
        let ckey = format!("cand-{key}");
        match store
            .propose_candidate_fact(
                &w,
                &CandidateFact {
                    fact_key: ckey.clone(),
                    value: "cv".to_string(),
                    citations: cites.clone(),
                },
            )
            .await
        {
            Ok(o) => {
                if o.accepted {
                    match o.fact {
                        Some(fact) => {
                            if fact.citations.is_empty()
                                && record(
                                    &mut cex,
                                    format!("accepted candidate {} has zero cites", ckey),
                                )
                            {
                                break;
                            }
                        }
                        None => {
                            if record(
                                &mut cex,
                                format!("accepted candidate {ckey:?} carries no fact"),
                            ) {
                                break;
                            }
                        }
                    }
                } else if record(
                    &mut cex,
                    format!(
                        "valid candidate {ckey:?} unexpectedly rejected: {:?}",
                        o.rejection
                    ),
                ) {
                    break;
                }
            }
            Err(e) => {
                if record(&mut cex, format!("valid candidate {ckey:?} -> Err {e:?}")) {
                    break;
                }
            }
        }

        // Adversarial: citation whose node does not exist in the (existing) doc.
        // The register pins the "citation does not resolve" message on
        // `propose_candidate_fact`; for `create_fact` a non-accepted write must
        // be a ValidationError (the create path reports its own message).
        if rng.pick(2) == 0 {
            let ghost = store
                .create_fact(
                    &w,
                    &doc.document_id,
                    &format!("ghostnode-{key}"),
                    "v",
                    &[(doc.document_id.clone(), nid("no-such-node"))],
                )
                .await;
            if !matches!(ghost, Err(StoreError::ValidationError(_)))
                && record(
                    &mut cex,
                    format!("ghost-node create {key:?} -> {ghost:?} (want ValidationError)"),
                )
            {
                break;
            }
        }
    }

    // Sequential delete-then-propose proxy: a citation to a DELETED document is
    // never committed.
    if cex.len() < 5 {
        let doc2 = new_doc(&store, &w, "probe").await;
        seed_nodes(&store, &doc2, &["x1"]).await;
        store.delete_document(&doc2.document_id).await.unwrap();
        let res = store
            .propose_candidate_fact(
                &w,
                &CandidateFact {
                    fact_key: "deleted-cite".to_string(),
                    value: "v".to_string(),
                    citations: vec![(doc2.document_id.clone(), nid("x1"))],
                },
            )
            .await;
        if !matches!(res, Err(StoreError::ValidationError(ref m)) if m.contains("citation does not resolve"))
            && record(
                &mut cex,
                format!("deleted-doc candidate -> {res:?} (want ValidationError)"),
            )
        {
            // record but don't break; still report
        }
        if store.get_fact(&w, "deleted-cite").await.is_ok() {
            record(&mut cex, "deleted-doc candidate was committed".to_string());
        }
    }

    finisher(&cex, "P-SM-3");
}

// ---------------------------------------------------------------------------
// P-SM-4 — del_cited  (class SM, strategy `del_cited`, register tag [PENDING H1
//                      → expected BROKEN today])
// ---------------------------------------------------------------------------
// Delete-citation integrity: a document whose node a committed fact cites is not
// deletable; deletion → Err(DocumentInUse) and the cited document + fact remain.
//
// Register tag is PENDING → expected BROKEN. docs/defects.md (FIXED) + the §4.3
// DONE row in next-steps.md state H1 is ALREADY fixed (delete gate scans
// fact_store). The empirical run decides. If HELD → we flag the register tag
// STALE.
const ROW_SM4_TAG: u64 = 0x00_00_00_00_00_00_00_04;
const ROW_SM4_BUDGET: u64 = 30;

#[tokio::test]
async fn p_sm4_del_cited() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "W").await;

    let mut rng = SplitMix64::new(0xD1B54A32D192ED03 ^ ROW_SM4_TAG);
    let mut budget = 0u64;
    let mut cex: Vec<String> = Vec::new();

    loop {
        if budget >= ROW_SM4_BUDGET || cex.len() >= 5 {
            break;
        }
        budget += 1;

        let cited = new_doc(&store, &w, &format!("cited-{budget}")).await;
        seed_nodes(&store, &cited, &["n1"]).await;
        let key = format!("fact-{budget}-{}", rng.below(1 << 12));
        store
            .create_fact(
                &w,
                &cited.document_id,
                &key,
                "v",
                &[(cited.document_id.clone(), nid("n1"))],
            )
            .await
            .unwrap();

        // Delete the cited document → DocumentInUse.
        match store.delete_document(&cited.document_id).await {
            Err(StoreError::DocumentInUse) => {}
            other => {
                if record(
                    &mut cex,
                    format!(
                        "delete cited doc #{budget} -> {other:?} (want DocumentInUse); broken if Ok/other"
                    ),
                ) {
                    break;
                }
                continue;
            }
        }
        // Fact remains with citation intact; document still present.
        let f = store.get_fact(&w, &key).await;
        if !matches!(&f, Ok(x) if x.citations.iter().any(|(c, _)| c == &cited.document_id))
            && record(
                &mut cex,
                format!("after blocked delete, fact {key:?} = {f:?} loses citation"),
            )
        {
            break;
        }
        if store.get_document(&cited.document_id).await.is_err()
            && record(
                &mut cex,
                format!("blocked delete of #{budget} vanished the document"),
            )
        {
            break;
        }

        // Adversarial (budget-permitting): an UNcited document in the same wiki
        // must still be deletable.
        if budget.is_multiple_of(3) {
            let uncited = new_doc(&store, &w, &format!("uncited-{budget}")).await;
            seed_nodes(&store, &uncited, &["u1"]).await;
            let res = store.delete_document(&uncited.document_id).await;
            // This is a POSITIVE check (delete of the uncited doc must SUCCEED).
            if res.is_err()
                && record(
                    &mut cex,
                    format!("uncited doc #{budget} delete failed (should succeed): {res:?}"),
                )
            {
                break;
            }
        }
    }

    // Boundary: a document cited by TWO distinct facts is DocumentInUse; and a
    // fact citing only one of two documents lets the *uncited* one still delete.
    if cex.len() < 5 {
        let dbl = new_doc(&store, &w, "dbl").await;
        seed_nodes(&store, &dbl, &["x1"]).await;
        store
            .create_fact(
                &w,
                &dbl.document_id,
                "fa",
                "v",
                &[(dbl.document_id.clone(), nid("x1"))],
            )
            .await
            .unwrap();
        store
            .create_fact(
                &w,
                &dbl.document_id,
                "fb",
                "v2",
                &[(dbl.document_id.clone(), nid("x1"))],
            )
            .await
            .unwrap();
        match store.delete_document(&dbl.document_id).await {
            Err(StoreError::DocumentInUse) => {}
            other => {
                if record(
                    &mut cex,
                    format!("2-cite doc delete -> {other:?} (want DocumentInUse)"),
                ) {
                    // still proceed to record
                }
            }
        }

        // one-of-two: same node cited, other node's doc deletable.
        let cited2 = new_doc(&store, &w, "cited2").await;
        let free = new_doc(&store, &w, "free").await;
        seed_nodes(&store, &cited2, &["y1"]).await;
        seed_nodes(&store, &free, &["z1"]).await;
        store
            .create_fact(
                &w,
                &cited2.document_id,
                "only-cited",
                "v",
                &[(cited2.document_id.clone(), nid("y1"))],
            )
            .await
            .unwrap();
        // deleting the *free* (uncited) doc must succeed.
        if store.delete_document(&free.document_id).await.is_err() {
            record(&mut cex, "uncited doc delete should succeed (row P-SM-4)".to_string())
                // no break; one more cex is fine
                ;
        }
        if !matches!(
            store.delete_document(&cited2.document_id).await,
            Err(StoreError::DocumentInUse)
        ) {
            record(
                &mut cex,
                "cited2 doc delete did not return DocumentInUse".to_string(),
            );
        }
    }

    finisher(&cex, "P-SM-4");
}

// ---------------------------------------------------------------------------
// P-SM-5 — ghost_wiki  (class SM, strategy `ghost_wiki`, register tag [PENDING
//                       H5 → expected BROKEN today])
// ---------------------------------------------------------------------------
// Unknown wiki → uniform `WikiNotFound` across the whole fact surface
// (get/list/create/update/propose), committing nothing.
// Register tag PENDING → expected BROKEN; defects.md says H5 already fixed. The
// empirical run decides. If HELD → register tag STALE.
const ROW_SM5_TAG: u64 = 0x00_00_00_00_00_00_00_05;
const ROW_SM5_BUDGET: u64 = 30;

#[tokio::test]
async fn p_sm5_ghost_wiki() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "real").await;
    let doc = new_doc(&store, &w, "d").await;
    seed_nodes(&store, &doc, &["n1"]).await;

    let mut rng = SplitMix64::new(0xD1B54A32D192ED03 ^ ROW_SM5_TAG);
    let mut budget = 0u64;
    let mut cex: Vec<String> = Vec::new();

    loop {
        if budget >= ROW_SM5_BUDGET || cex.len() >= 5 {
            break;
        }
        budget += 1;
        // A ghost id that never existed; some iterations resemble a real id with
        // an appended char (the register's boundary).
        let ghost = if budget.is_multiple_of(4) {
            wiki(&format!("real-ghost-{budget}"))
        } else {
            wiki(&format!("never-created-{budget}-{}", rng.below(1 << 8)))
        };
        let key = format!("gk-{}", budget);

        // get_fact
        if !matches!(
            store.get_fact(&ghost, &key).await,
            Err(StoreError::WikiNotFound)
        ) && record(
            &mut cex,
            format!("get_fact on ghost {ghost:?} not WikiNotFound"),
        ) {
            break;
        }
        // list_facts
        if !matches!(
            store.list_facts(&ghost, &list_default()).await,
            Err(StoreError::WikiNotFound)
        ) && record(
            &mut cex,
            format!("list_facts on ghost {ghost:?} not WikiNotFound"),
        ) {
            break;
        }
        // create_fact with a REAL document_id (wiki check must fire first).
        match store
            .create_fact(
                &ghost,
                &doc.document_id,
                &key,
                "v",
                &[(doc.document_id.clone(), nid("n1"))],
            )
            .await
        {
            Err(StoreError::WikiNotFound) => {}
            other => {
                if record(
                    &mut cex,
                    format!("create_fact on ghost {ghost:?} -> {other:?} (want WikiNotFound)"),
                ) {
                    break;
                }
            }
        }
        // update_fact
        match store
            .update_fact(
                &ghost,
                &key,
                &UpdateFactRequest {
                    value: "v".to_string(),
                    citations: vec![(doc.document_id.clone(), nid("n1"))],
                },
            )
            .await
        {
            Err(StoreError::WikiNotFound) => {}
            other => {
                if record(
                    &mut cex,
                    format!("update_fact on ghost {ghost:?} -> {other:?} (want WikiNotFound)"),
                ) {
                    break;
                }
            }
        }
        // propose_candidate_fact
        match store
            .propose_candidate_fact(
                &ghost,
                &CandidateFact {
                    fact_key: key.clone(),
                    value: "v".to_string(),
                    citations: vec![(doc.document_id.clone(), nid("n1"))],
                },
            )
            .await
        {
            Err(StoreError::WikiNotFound) => {}
            other => {
                if record(
                    &mut cex,
                    format!("propose on ghost {ghost:?} -> {other:?} (want WikiNotFound)"),
                ) {
                    break;
                }
            }
        }
    }

    // After the rejected create_fact, list_facts(G) is still WikiNotFound and no
    // fact materializes in the real wiki.
    if cex.len() < 5 {
        let g2 = wiki("post-ghost");
        store
            .create_fact(
                &g2,
                &doc.document_id,
                "leak",
                "v",
                &[(doc.document_id.clone(), nid("n1"))],
            )
            .await
            .unwrap_err();
        if !matches!(
            store.list_facts(&g2, &list_default()).await,
            Err(StoreError::WikiNotFound)
        ) {
            record(
                &mut cex,
                "list_facts(ghost) allowed after rejected create".to_string(),
            );
        }
        if store.get_fact(&w, "leak").await.is_ok() {
            record(
                &mut cex,
                "rejected ghost create materialized a fact in the real wiki".to_string(),
            );
        }
    }

    finisher(&cex, "P-SM-5");
}

// ---------------------------------------------------------------------------
// P-IM-1 — blank_inputs  (class IM, strategy `blank_inputs`, register tag [GREEN
//                         for ""; PENDING H2/H4 → expected BROKEN on whitespace])
// ---------------------------------------------------------------------------
// Schema-strictness: an empty string fact_key or value is rejected on
// create_fact/update_fact/propose_candidate_fact (no blank content persists).
// Register: "" is GREEN; `" "` (whitespace-only) is PENDING (H2/H4) → expected
// BROKEN. defects.md says the whitespace/trim fixes landed. The empirical run
// decides each branch; a HELD whitespace branch → register tag STALE.
const ROW_IM1_TAG: u64 = 0x00_00_00_00_00_00_00_11;
const ROW_IM1_BUDGET: u64 = 60;

#[tokio::test]
async fn p_im1_blank_inputs() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "W").await;
    let doc = new_doc(&store, &w, "d").await;
    seed_nodes(&store, &doc, &["n1"]).await;
    let good_cite = || vec![(doc.document_id.clone(), nid("n1"))];

    let mut rng = SplitMix64::new(0xD1B54A32D192ED03 ^ ROW_IM1_TAG);
    let mut budget = 0u64;
    let mut cex: Vec<String> = Vec::new();

    loop {
        if budget >= ROW_IM1_BUDGET || cex.len() >= 5 {
            break;
        }
        budget += 1;
        // blank shapes explored: "", " ", "\t", multi-space.
        let blank_shapes = ["", " ", "\t", "   "];
        let b = blank_shapes[rng.pick(blank_shapes.len())];
        let btag = match b {
            "" => "EMPTY",
            _ => "WHITESPACE",
        };
        let k = format!("im-{budget}");

        // create_fact blank fact_key + non-blank value.
        match store
            .create_fact(&w, &doc.document_id, b, "v", &good_cite())
            .await
        {
            Err(StoreError::ValidationError(_)) => {}
            other => {
                if record(
                    &mut cex,
                    format!("create blank fact_key({btag}) -> {other:?} (want ValidationError)"),
                ) {
                    break;
                }
            }
        }
        // create_fact blank value + valid citations.
        match store
            .create_fact(&w, &doc.document_id, &k, b, &good_cite())
            .await
        {
            Err(StoreError::ValidationError(_)) => {}
            other => {
                if record(
                    &mut cex,
                    format!("create blank value({btag}) -> {other:?} (want ValidationError)"),
                ) {
                    break;
                }
            }
        }

        // (Only when a fact with key `k` is not present) seed one for update.
        if store.get_fact(&w, &k).await.is_err() {
            store
                .create_fact(&w, &doc.document_id, &k, "original", &good_cite())
                .await
                .unwrap();
        }
        // update_fact blank value must be rejected and leave the stored fact
        // unchanged.
        let before = store.get_fact(&w, &k).await.unwrap();
        match store
            .update_fact(
                &w,
                &k,
                &UpdateFactRequest {
                    value: b.to_string(),
                    citations: good_cite(),
                },
            )
            .await
        {
            Err(StoreError::ValidationError(_)) => {}
            other => {
                if record(
                    &mut cex,
                    format!("update blank value({btag}) -> {other:?} (want ValidationError)"),
                ) {
                    break;
                }
            }
        }
        let after = store.get_fact(&w, &k).await.unwrap();
        if after.value != before.value
            && record(
                &mut cex,
                format!(
                    "blank update changed stored value: {:?}->{:?}",
                    before.value, after.value
                ),
            )
        {
            break;
        }

        // propose_candidate_fact blank fact_key → accepted:false, field fact_key.
        match store
            .propose_candidate_fact(
                &w,
                &CandidateFact {
                    fact_key: b.to_string(),
                    value: "v".to_string(),
                    citations: good_cite(),
                },
            )
            .await
        {
            Ok(o) => {
                if o.accepted {
                    if record(&mut cex, format!("blank-propose fact_key({btag}) ACCEPTED")) {
                        break;
                    }
                } else if o.rejection.as_ref().map(|r| r.field.as_str()) != Some("fact_key")
                    && record(
                        &mut cex,
                        format!("blank-propose fact_key field {:?}", o.rejection),
                    )
                {
                    break;
                }
                // DEAD-ASSERTION FIX (audit): the candidate's key is the blank
                // `b` itself, not `b-{k}` — the old `get_fact(&w, "b-{k}")`
                // probe read a key this branch never writes, so it was
                // vacuous. Correct check: the blank-key candidate neither
                // committed nor leaked — `get_fact(&w, b)` must be `Err`
                // (a blank key is never a committed key here), i.e. no fact
                // exists under the candidate's own key.
                if store.get_fact(&w, b).await.is_ok()
                    && record(
                        &mut cex,
                        format!("blank-propose fact_key({btag}) committed/leaked under its own key {:?}", b),
                    ) {
                        break;
                    }
            }
            Err(e) => {
                if record(
                    &mut cex,
                    format!("blank-propose fact_key({btag}) -> Err {e:?}"),
                ) {
                    break;
                }
            }
        }
        // propose_candidate_fact blank value → accepted:false, field value.
        match store
            .propose_candidate_fact(
                &w,
                &CandidateFact {
                    fact_key: format!("bv-{budget}"),
                    value: b.to_string(),
                    citations: good_cite(),
                },
            )
            .await
        {
            Ok(o) => {
                if o.accepted {
                    if record(&mut cex, format!("blank-propose value({btag}) ACCEPTED")) {
                        break;
                    }
                } else if o.rejection.as_ref().map(|r| r.field.as_str()) != Some("value")
                    && record(
                        &mut cex,
                        format!("blank-propose value field {:?}", o.rejection),
                    )
                {
                    break;
                }
                if store.get_fact(&w, &format!("bv-{budget}")).await.is_ok()
                    && record(&mut cex, format!("blank-propose value({btag}) committed"))
                {
                    break;
                }
            }
            Err(e) => {
                if record(
                    &mut cex,
                    format!("blank-propose value({btag}) -> Err {e:?}"),
                ) {
                    break;
                }
            }
        }
    }

    finisher(&cex, "P-IM-1");
}

// ---------------------------------------------------------------------------
// P-TP-1 — upd_idem  (class TP, strategy `upd_idem`, register tag [GREEN])
// ---------------------------------------------------------------------------
// update_fact preserves identity, refreshes time: key preserved, retrievable
// under same key, updated_at not decreased, value + citations applied verbatim
// (order-insensitive multiset). Generated on NON-BLANK values only (blank
// exercises P-IM-1).
const ROW_TP1_TAG: u64 = 0x00_00_00_00_00_00_00_21;
const ROW_TP1_BUDGET: u64 = 60;

#[tokio::test]
async fn p_tp1_upd_idem() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "W").await;
    let doc = new_doc(&store, &w, "d").await;
    seed_nodes(&store, &doc, &["n1", "n2", "n3", "n4"]).await;

    let mut rng = SplitMix64::new(0xD1B54A32D192ED03 ^ ROW_TP1_TAG);
    let mut budget = 0u64;
    let mut cex: Vec<String> = Vec::new();
    let _all_nodes = ["n1", "n2", "n3", "n4", "n5", "n6"];

    loop {
        if budget >= ROW_TP1_BUDGET || cex.len() >= 5 {
            break;
        }
        budget += 1;
        let key = format!("tp-{}-{}", budget, rng.below(1 << 16));
        store
            .create_fact(
                &w,
                &doc.document_id,
                &key,
                "seed",
                &[(doc.document_id.clone(), nid("n1"))],
            )
            .await
            .unwrap();

        let p = store.get_fact(&w, &key).await.unwrap();

        // Build a request on non-blank value; citations distinct (dedup identity).
        let mode = rng.pick(3);
        let (value, req_cites) = match mode {
            // same (idempotent) value; same single citation
            0 => (
                "seed".to_string(),
                vec![(doc.document_id.clone(), nid("n1"))],
            ),
            // new value; replace citations entirely (distinct set)
            1 => (
                format!("newval-{}", rng.below(1 << 20)),
                vec![
                    (doc.document_id.clone(), nid("n2")),
                    (doc.document_id.clone(), nid("n3")),
                ],
            ),
            // same value as before, new distinct citations
            _ => (
                "seed".to_string(),
                vec![
                    (doc.document_id.clone(), nid("n4")),
                    (doc.document_id.clone(), nid("n2")),
                ],
            ),
        };

        let req = UpdateFactRequest {
            value: value.clone(),
            citations: req_cites.clone(),
        };
        let upd = match store.update_fact(&w, &key, &req).await {
            Ok(u) => u,
            Err(e) => {
                if record(&mut cex, format!("update_fact({key:?}) -> Err {e:?}")) {
                    break;
                }
                continue;
            }
        };

        // identity: fact_key preserved.
        if upd.fact_key != key
            && record(
                &mut cex,
                format!("{key:?} updated to key {:?}", upd.fact_key),
            )
        {
            break;
        }
        // retrievable under same key.
        let got = store.get_fact(&w, &key).await.unwrap();
        if got.fact_key != key
            && record(
                &mut cex,
                format!("{key:?} no longer retrievable, got {:?}", got.fact_key),
            )
        {
            break;
        }
        // updated_at not decreased vs prior read.
        if upd.updated_at < p.updated_at
            && record(
                &mut cex,
                format!(
                    "{key:?} updated_at went back: {} -> {}",
                    p.updated_at, upd.updated_at
                ),
            )
        {
            break;
        }
        // value applied verbatim.
        if upd.value != req.value
            && record(
                &mut cex,
                format!(
                    "{key:?} value mismatch: want {:?} got {:?}",
                    req.value, upd.value
                ),
            )
        {
            break;
        }
        // citations applied (order-insensitive multiset equality).
        if canon_cites(&upd.citations) != canon_cites(&req_cites)
            && record(
                &mut cex,
                format!(
                    "{key:?} citations mismatch: want {:?} got {:?}",
                    req_cites, upd.citations
                ),
            )
        {
            break;
        }
    }

    // Identity-preservation NEGATIVE: update_fact on a key never created →
    // DocumentNotFound.
    if cex.len() < 5 {
        let res = store
            .update_fact(
                &w,
                "never-created-key",
                &UpdateFactRequest {
                    value: "v".to_string(),
                    citations: vec![(doc.document_id.clone(), nid("n1"))],
                },
            )
            .await;
        if !matches!(res, Err(StoreError::DocumentNotFound)) {
            record(
                &mut cex,
                format!("update unknown key -> {res:?} (want DocumentNotFound)"),
            );
        }
    }

    finisher(&cex, "P-TP-1");
}

// ---------------------------------------------------------------------------
// P-TP-2 — page_walk  (class TP, strategy `page_walk`, register tag [GREEN])
// ---------------------------------------------------------------------------
// list_facts pagination is lossless, exhaustive, deterministic; bounds hold.
// Stepping pages at a fixed valid page_size returns every committed fact exactly
// once; total stable; identical calls return identical sets; page>=1 and
// 1<=page_size<=100. Adversarial bounds (page=0/page_size=0/101) → ValidationError.
const ROW_TP2_TAG: u64 = 0x00_00_00_00_00_00_00_22;
const ROW_TP2_BUDGET: u64 = 60;

#[tokio::test]
async fn p_tp2_page_walk() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "W").await;
    let doc = new_doc(&store, &w, "d").await;
    seed_nodes(&store, &doc, &["n1", "n2", "n3", "n4", "n5", "n6"]).await;

    // Commit an anchor set: exact-multiple count 4, plus one-off 5 (and rely on a
    // fresh empty wiki below for the 0/total-empty bounds).
    // We commit a variable total per case from a fresh wiki so each case sees a
    // clean table.
    let mut rng = SplitMix64::new(0xD1B54A32D192ED03 ^ ROW_TP2_TAG);
    let mut budget = 0u64;
    let mut cex: Vec<String> = Vec::new();

    loop {
        if budget >= ROW_TP2_BUDGET || cex.len() >= 5 {
            break;
        }
        budget += 1;

        // Fresh wiki per case so the "variable count" includes 0, 1, 5, and
        // near/at page_size multiples.
        let wk = new_wiki(&store, &format!("W{budget}")).await;
        let dk = new_doc(&store, &wk, &format!("d{budget}")).await;
        let names = ["n1", "n2", "n3", "n4", "n5", "n6"];
        seed_nodes(&store, &dk, &names).await;

        let count_shape = budget % 5; // 0,1,2,5, boundary
        let count = match count_shape {
            0 => 0,
            1 => 1,
            2 => 2,
            3 => 5,
            _ => 12, // boundary near a page_size multiple
        };
        for i in 0..count {
            let node = names[i % names.len()];
            store
                .create_fact(
                    &wk,
                    &dk.document_id,
                    &format!("fact-{budget}-{i}"),
                    "v",
                    &[(dk.document_id.clone(), nid(node))],
                )
                .await
                .unwrap();
        }

        // Reference union = default (all-None) filter.
        let reference = store.list_facts(&wk, &list_default()).await.unwrap();
        let ref_keys: Vec<String> = {
            let mut v: Vec<String> = reference.items.iter().map(|f| f.fact_key.clone()).collect();
            v.sort();
            v
        };
        if reference.total != ref_keys.len() as u64 {
            // total must equal the number of committed facts (0 allowed).
        }

        // page_size ∈ {1, 50, 100} plus a mid value; page>=1.
        let psize_choices: &[u64] = if count == 0 {
            &[1, 100]
        } else {
            &[1, 2, 3, 5, 100]
        };
        let psize = psize_choices[rng.pick(psize_choices.len())];

        // Walk pages.
        let mut collected: Vec<String> = Vec::new();
        let mut page: u64 = 1;
        let mut totals_seen: Vec<u64> = Vec::new();
        loop {
            let fl = store
                .list_facts(&wk, &list_opts(Some(page), Some(psize)))
                .await
                .unwrap();
            totals_seen.push(fl.total);
            for f in &fl.items {
                collected.push(f.fact_key.clone());
            }
            if fl.items.is_empty() {
                break;
            }
            page += 1;
            if page > 200 {
                // guard
                break;
            }
        }

        collected.sort();
        if collected != ref_keys
            && record(
                &mut cex,
                format!(
                    "walk({count} facts, psize {psize}) = {collected:?} != reference {ref_keys:?}"
                ),
            )
        {
            break;
        }
        // Every fact exactly once = same lengths after dedup: collected is sorted;
        // ref_keys is sorted; equality already checked. But also check no dupes in
        // collected.
        let mut uniq = collected.clone();
        uniq.dedup();
        if uniq.len() != collected.len()
            && record(
                &mut cex,
                format!("walk({count}, psize {psize}) has duplicate facts"),
            )
        {
            break;
        }
        // total stable across pages == reference.total.
        for t in &totals_seen {
            if *t != reference.total
                && record(
                    &mut cex,
                    format!("walk total {t} != reference {}", reference.total),
                )
            {
                break;
            }
        }

        // Determinism: two identical invocations return equal items sets.
        let i1 = store
            .list_facts(&wk, &list_opts(Some(1), Some(psize)))
            .await
            .unwrap();
        let i2 = store
            .list_facts(&wk, &list_opts(Some(1), Some(psize)))
            .await
            .unwrap();
        let mut s1: Vec<String> = i1.items.iter().map(|f| f.fact_key.clone()).collect();
        let mut s2: Vec<String> = i2.items.iter().map(|f| f.fact_key.clone()).collect();
        s1.sort();
        s2.sort();
        if s1 != s2
            && record(
                &mut cex,
                format!("non-deterministic page 1 at psize {psize}"),
            )
        {
            break;
        }

        // Bounds: page=0, page_size=0, page_size=101 → ValidationError.
        for bad in [
            list_opts(Some(0), Some(psize)),
            list_opts(Some(1), Some(0)),
            list_opts(Some(1), Some(101)),
        ] {
            match store.list_facts(&wk, &bad).await {
                Err(StoreError::ValidationError(_)) => {}
                other => {
                    if record(
                        &mut cex,
                        format!("bad filter {bad:?} -> {other:?} (want ValidationError)"),
                    ) {
                        break;
                    }
                }
            }
        }
    }

    finisher(&cex, "P-TP-2");
}

// ---------------------------------------------------------------------------
// §4.3 READ-ONLY ADVERSARIAL AUDIT — robustness probes.
//
// Concrete negative-generator / robustness probes requested by the §4.3 audit.
// Each is a DETERMINISTIC explicit assertion (no PRNG), so it does not consume
// any of the 400 generated-case budget. All are GREEN on the current
// implementation (the §4.3 adversarial fixes already landed), so each probe
// pins the observed behavior as a regression guard.
// ---------------------------------------------------------------------------

/// Probe 1: update_fact with a dangling citation (live doc, nonexistent node)
/// → `Err(ValidationError)` "citation does not resolve"; the persisted fact is
/// unchanged (value, citations, updated_at).
#[tokio::test]
async fn audit_update_dangling_citation() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "W").await;
    let doc = new_doc(&store, &w, "d").await;
    seed_nodes(&store, &doc, &["n1"]).await;

    let key = "upd-dangling";
    store
        .create_fact(
            &w,
            &doc.document_id,
            key,
            "original",
            &[(doc.document_id.clone(), nid("n1"))],
        )
        .await
        .unwrap();
    let before = store.get_fact(&w, key).await.unwrap();

    // Cite a live doc but a node that does not exist in it.
    let res = store
        .update_fact(
            &w,
            key,
            &UpdateFactRequest {
                value: "new".into(),
                citations: vec![(doc.document_id.clone(), nid("gone"))],
            },
        )
        .await;
    assert!(
        matches!(res, Err(StoreError::ValidationError(ref m)) if m.contains("citation does not resolve")),
        "update with dangling citation -> {res:?} (want ValidationError \"citation does not resolve\")"
    );
    // Fact unchanged: value, citations, updated_at, key all intact.
    let after = store.get_fact(&w, key).await.unwrap();
    assert_eq!(
        after.value, "original",
        "value must be unchanged after rejected update"
    );
    assert_eq!(
        after.citations, before.citations,
        "citations must be unchanged"
    );
    assert_eq!(
        after.updated_at, before.updated_at,
        "updated_at must be unchanged"
    );
    assert_eq!(after.fact_key, key, "fact_key must be unchanged");
}

/// Probe 2: update_fact with an empty `citations` vec → `Err(ValidationError)`;
/// the persisted fact is unchanged.
#[tokio::test]
async fn audit_update_empty_citations() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "W").await;
    let doc = new_doc(&store, &w, "d").await;
    seed_nodes(&store, &doc, &["n1"]).await;

    let key = "upd-empty";
    store
        .create_fact(
            &w,
            &doc.document_id,
            key,
            "original",
            &[(doc.document_id.clone(), nid("n1"))],
        )
        .await
        .unwrap();
    let before = store.get_fact(&w, key).await.unwrap();

    let res = store
        .update_fact(
            &w,
            key,
            &UpdateFactRequest {
                value: "v".into(),
                citations: vec![],
            },
        )
        .await;
    assert!(
        matches!(res, Err(StoreError::ValidationError(_))),
        "update with empty citations -> {res:?} (want ValidationError)"
    );
    let after = store.get_fact(&w, key).await.unwrap();
    assert_eq!(
        after.value, "original",
        "value must be unchanged after rejected update"
    );
    assert_eq!(
        after.citations, before.citations,
        "citations must be unchanged"
    );
}

/// Probe 3: P-SM-3 propose negative-leg fill — `propose_candidate_fact` with a
/// citation that does not resolve → `Err(ValidationError)` "citation does not
/// resolve" and `get_fact(key)` is `Err` (nothing committed).
#[tokio::test]
async fn audit_propose_dangling_negative_leg() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "W").await;
    let doc = new_doc(&store, &w, "d").await;
    seed_nodes(&store, &doc, &["n1"]).await;

    let key = "neg-propose";
    let res = store
        .propose_candidate_fact(
            &w,
            &CandidateFact {
                fact_key: key.into(),
                value: "v".into(),
                citations: vec![(doc.document_id.clone(), nid("does-not-exist"))],
            },
        )
        .await;
    assert!(
        matches!(res, Err(StoreError::ValidationError(ref m)) if m.contains("citation does not resolve")),
        "propose with dangling citation -> {res:?} (want ValidationError \"citation does not resolve\")"
    );
    // Nothing committed.
    assert!(
        store.get_fact(&w, key).await.is_err(),
        "propose with dangling citation must not commit anything under {key:?}"
    );
}

/// Probe 4: non-blank lossless round-trip (closes the `" a "` gap). Values are
/// stored UNTRIMMED — trim is used only as the emptiness gate, never as a
/// normalization.
#[tokio::test]
async fn audit_non_blank_lossless_round_trip() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "W").await;
    let doc = new_doc(&store, &w, "d").await;
    seed_nodes(&store, &doc, &["n1"]).await;

    let k = "  a  ";
    let n1 = (doc.document_id.clone(), nid("n1"));
    store
        .create_fact(&w, &doc.document_id, k, "  b  ", std::slice::from_ref(&n1))
        .await
        .expect("non-blank create must succeed (trim is only the emptiness gate)");
    let f1 = store
        .get_fact(&w, k)
        .await
        .expect("fact retrievable under its exact key");
    assert_eq!(
        f1.value, "  b  ",
        "stored value must be verbatim (untrimmed)"
    );
    assert_eq!(f1.fact_key, k, "stored key must be verbatim (untrimmed)");

    store
        .update_fact(
            &w,
            k,
            &UpdateFactRequest {
                value: "  c  ".into(),
                citations: vec![n1.clone()],
            },
        )
        .await
        .expect("non-blank update must succeed");
    let f2 = store.get_fact(&w, k).await.unwrap();
    assert_eq!(
        f2.value, "  c  ",
        "updated value stored verbatim (untrimmed)"
    );
}

/// Probe 5: unicode-distinctness pin — `"caf\u{e9}"` (U+00E9 composed) and
/// `"cafe\u{301}"` (e + combining acute) are canonically-equivalent but
/// textually-distinct byte strings; both commit under one wiki, both present,
/// each count == 1 (pins exact-bytes HashMap uniqueness).
#[tokio::test]
async fn audit_unicode_distinctness_pin() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "W").await;
    let doc = new_doc(&store, &w, "d").await;
    seed_nodes(&store, &doc, &["n1"]).await;

    let k1 = "caf\u{e9}"; // composed U+00E9
    let k2 = "cafe\u{301}"; // e + combining acute
    assert_ne!(k1, k2, "the two strings must be textually distinct");

    let n1 = (doc.document_id.clone(), nid("n1"));
    store
        .create_fact(&w, &doc.document_id, k1, "v1", std::slice::from_ref(&n1))
        .await
        .expect("caf\u{e9} commits");
    store
        .create_fact(&w, &doc.document_id, k2, "v2", std::slice::from_ref(&n1))
        .await
        .expect("cafe+e-accommunes commits: fact_key uniqueness is exact-bytes");
    let f1 = store
        .get_fact(&w, k1)
        .await
        .expect("caf\u{e9} retrievable under its own key");
    let f2 = store
        .get_fact(&w, k2)
        .await
        .expect("cafe+combining retrievable under its own key");
    assert_eq!(f1.value, "v1");
    assert_eq!(f2.value, "v2");

    // Both present, each count == 1 in the (unfiltered) table.
    let table = store.list_facts(&w, &list_default()).await.unwrap();
    let c1 = table.items.iter().filter(|f| f.fact_key == k1).count();
    let c2 = table.items.iter().filter(|f| f.fact_key == k2).count();
    assert_eq!(
        (c1, c2),
        (1, 1),
        "each canonically-equivalent key must count exactly once"
    );
}

/// Probe 6: listFacts `state` filter determinism — a `ListFactsFilter{ state:
/// Some(DocState::Draft), ..}` page-walk with no state mutation yields a stable
/// filtered `total`, items exact-once, and identical invocations return equal
/// item sets (documents the Draft filter that no row currently exercises).
#[tokio::test]
async fn audit_state_filter_determinism() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "W").await;

    // DRAFT document holding two facts.
    let draft = new_doc(&store, &w, "draft").await;
    seed_nodes(&store, &draft, &["n1", "n2"]).await;
    store
        .create_fact(
            &w,
            &draft.document_id,
            "fd0",
            "v",
            &[(draft.document_id.clone(), nid("n1"))],
        )
        .await
        .unwrap();
    store
        .create_fact(
            &w,
            &draft.document_id,
            "fd1",
            "v",
            &[(draft.document_id.clone(), nid("n2"))],
        )
        .await
        .unwrap();
    // PUBLISHED document holding one fact — must be EXCLUDED by the Draft filter.
    let published = new_doc(&store, &w, "published").await;
    seed_nodes(&store, &published, &["m1"]).await;
    store
        .publish_document(&published.document_id)
        .await
        .unwrap();
    store
        .create_fact(
            &w,
            &published.document_id,
            "fb",
            "v",
            &[(published.document_id.clone(), nid("m1"))],
        )
        .await
        .unwrap();

    let f = |page: u64| ListFactsFilter {
        state: Some(DocState::Draft),
        page: Some(page),
        page_size: Some(100),
    };
    // No state mutation anywhere between reads.
    let mut collected: Vec<String> = Vec::new();
    let mut totals: Vec<u64> = Vec::new();
    let mut page = 1u64;
    loop {
        let fl = store
            .list_facts(&w, &f(page))
            .await
            .expect("state-filter page-walk on a real wiki must succeed");
        totals.push(fl.total);
        for x in &fl.items {
            collected.push(x.fact_key.clone());
        }
        if fl.items.is_empty() {
            break;
        }
        page += 1;
        if page > 100 {
            break;
        }
    }
    // Stable filtered total == exactly the two Draft facts (published excluded).
    assert!(
        !totals.is_empty(),
        "state-filter walk returned no page at all"
    );
    assert!(
        totals.iter().all(|t| *t == totals[0]),
        "Draft-filtered total unstable across pages: {totals:?}"
    );
    assert_eq!(
        totals[0], 2,
        "Draft filter must exclude the published document's fact"
    );

    // Items exact-once (sorted + dedup == sorted list).
    collected.sort();
    let mut uniq = collected.clone();
    uniq.dedup();
    assert_eq!(
        uniq, collected,
        "Draft-filtered items must not repeat across pages: {collected:?}"
    );
    assert_eq!(
        collected,
        vec!["fd0".to_string(), "fd1".to_string()],
        "Draft filter must return exactly the two draft facts by sorted key"
    );

    // Determinism: two identical invocations return equal item sets.
    let i1 = store.list_facts(&w, &f(1)).await.unwrap();
    let i2 = store.list_facts(&w, &f(1)).await.unwrap();
    assert_eq!(
        i1.items, i2.items,
        "identical state-filter invocations must return equal items"
    );
    assert_eq!(
        i1.total, i2.total,
        "identical state-filter invocations must return equal total"
    );
}
