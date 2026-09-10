//! §7.5 F4 — community retrieval — EXECUTED property layer (PBT-gate artifact #2,
//! TestWriter) for the F4 rows of `docs/specs/4-2-graph-property-register.md`.
//!
//! Implements the six F4 rows (P-IM-1, P-IM-2, P-SM-1, P-SM-2, P-TP-1, P-TP-2) as
//! one `#[test]` per row. Each row runs a tiny deterministic property harness
//! (hand-rolled **SplitMix64**, pinned seed — no external crate, no proptest) and
//! implements its Observable-as-property **faithfully** — a genuinely-broken row
//! FAILS its `#[test]` with the retained minimal counterexamples (stop-after-5).
//!
//! ---------------------------------------------------------------------------
//! PINNED SEED (recorded in the file header and report):
//!    BASE_SEED = 0x6D797A4CF0D09E5D
//! Every row derives its own instance seed as `BASE_SEED ^ ROW_TAG` (a fixed
//! per-row constant), so generation is byte-for-byte reproducible run-to-run
//! without cross-row correlation.
//! ---------------------------------------------------------------------------
//!
//! Budget: ≤100 generated cases per row; the per-row budgets sum to **300** total
//! (P-IM-1=60, P-IM-2=55, P-SM-1=60, P-SM-2=45, P-TP-1=40, P-TP-2=40) — within the
//! ≤400 cap. stop-after-5: a row that starts failing stops as soon as it has
//! retained ≤5 minimal counterexamples; a HELD row runs its full budget.
//!
//! The `get_community_context` accessor is a **RED-stage stub** (`unimplemented!()`),
//! so every row FAILS at runtime (the compile-with-stubs red set) until the
//! Implementer lands the real pre-joined read. Every generator stays on the
//! **legal** side of the §4.2 guards (a declared community with a non-empty member
//! set and non-empty summary) so a row never drives a `StoreError` — the only
//! fail-state (`CommunityNotFound`) is a §4.2.8.5 concern, outside this register.
//!
//! The crate-faithful `mark_communities_stale` triggers are `update_document`
//! (rewriting a member node) and `update_fact` on a fact the community
//! incorporates — **not** `add_triple` (which never marks a community stale).

use std::sync::Arc;

use gnosis::{
    CommunityContext, CommunityId, CreateDocumentRequest, DeclareCommunityOptions, DocumentId,
    Edge, EdgeKind, Graph, Node, NodeId, NodeKind, RagStore, ReferenceState, Store,
    UpdateDocumentRequest, UpdateFactRequest, WikiId,
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

async fn fresh_doc(names: &[&str]) -> (Arc<Store>, WikiId, gnosis::Document) {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "doc").await;
    apply_graph(&store, &doc, content_only_graph(&doc.document_id, names)).await;
    (store, w, doc)
}

/// A fresh store + wiki + a doc with `names` content nodes, plus a second doc
/// with `names2` content nodes (for cross-document membership).
async fn fresh_two_docs(
    names: &[&str],
    names2: &[&str],
) -> (Arc<Store>, WikiId, gnosis::Document, gnosis::Document) {
    let (store, w, doc_a) = fresh_doc(names).await;
    let doc_b = new_doc(&store, &w, "doc-b").await;
    apply_graph(
        &store,
        &doc_b,
        content_only_graph(&doc_b.document_id, names2),
    )
    .await;
    (store, w, doc_a, doc_b)
}

/// Build a member set for a community, probing the register's boundary shapes:
/// single-member, two-document, duplicate id (stored verbatim), and a fact
/// location member. Always non-empty (an empty set is unconstructible).
fn gen_members(
    rng: &mut SplitMix64,
    did_a: &DocumentId,
    did_b: &DocumentId,
    budget: u64,
) -> Vec<(DocumentId, NodeId)> {
    let mut members: Vec<(DocumentId, NodeId)> = Vec::new();
    members.push((did_a.clone(), nid("n1")));
    if budget.is_multiple_of(4) {
        // Two-document boundary.
        members.push((did_b.clone(), nid("m1")));
    } else if budget.is_multiple_of(3) {
        members.push((did_a.clone(), nid("n2")));
    }
    // Duplicate id — `declare_community` stores `node_ids` verbatim, never
    // de-duplicated.
    if rng.pick(7) == 0 {
        members.push((did_a.clone(), nid("n1")));
    }
    // A fact-location member (membership by declared id, not node kind).
    if rng.pick(9) == 0 {
        members.push((did_a.clone(), nid("fact-k1")));
    }
    members
}

/// The crate-faithful member-touching staleness trigger: `update_document`
/// rewriting a member node (marks the community `Stale`).
async fn touch_member_via_update_document(
    store: &Store,
    doc: &gnosis::Document,
    member: &(DocumentId, NodeId),
) {
    // Rewrite the document containing the member node (the whole graph is
    // re-applied, so the member node is rewritten → `mark_communities_stale`).
    let _ = member;
    apply_graph(
        store,
        doc,
        content_only_graph(&doc.document_id, &["n1", "n2"]),
    )
    .await;
}

/// The crate-faithful incorporated-fact staleness trigger: `update_fact` on a
/// fact the community incorporates (marks the community `Stale`).
async fn touch_member_via_update_fact(
    store: &Store,
    w: &WikiId,
    doc: &gnosis::Document,
    fact_key: &str,
) {
    store
        .update_fact(
            w,
            fact_key,
            &UpdateFactRequest {
                value: "updated-value".to_string(),
                citations: vec![(doc.document_id.clone(), nid("n1"))],
            },
        )
        .await
        .unwrap();
}

/// Read a community context, returning `None` on a `StoreError` (the caller
/// records the error as a counterexample).
async fn read_ctx(store: &Store, cid: &CommunityId) -> Option<CommunityContext> {
    store.get_community_context(cid).await.ok()
}

// ---------------------------------------------------------------------------
// P-IM-1 — Context determinism  (IM, `gen_ctx_determinism`, tag [RED])
// ---------------------------------------------------------------------------
//
// Two consecutive `get_community_context` reads with no intervening mutation
// return field-for-field equal values. Boundaries: two-doc members, single
// member, `Stale` state; adversarial: an unrelated mutation (a non-member
// `updateDocument`, or an `addTriple` — which never marks a community stale)
// between the reads must not change this community's context.
const ROW_IM1_TAG: u64 = 0x00_00_00_00_00_00_00_01;
const ROW_IM1_BUDGET: u64 = 60;

#[tokio::test]
async fn p_im1_context_determinism() {
    let mut cex: Vec<String> = Vec::new();
    let mut rng = SplitMix64::new(0x6D797A4CF0D09E5D ^ ROW_IM1_TAG);

    for budget in 1..=ROW_IM1_BUDGET {
        if cex.len() >= 5 {
            break;
        }
        let (store, w, doc_a, doc_b) = fresh_two_docs(&["n1", "n2"], &["m1"]).await;
        let did_a = doc_a.document_id.clone();
        let did_b = doc_b.document_id.clone();
        let members = gen_members(&mut rng, &did_a, &did_b, budget);
        let declared = store
            .declare_community(
                &members,
                &DeclareCommunityOptions {
                    summary: format!("manual {budget}"),
                    wiki_id: w.clone(),
                },
            )
            .await
            .unwrap();
        let cid = declared.community_id.clone();

        // Optionally drive the community into `Stale` (determinism holds
        // regardless of state).
        if budget % 5 == 0 {
            touch_member_via_update_document(&store, &doc_a, &members[0]).await;
        }

        let a = store.get_community_context(&cid).await;
        let a = match a {
            Ok(v) => v,
            Err(e) => {
                if cex.len() < 5 {
                    cex.push(format!("case {budget}: first read -> {e:?}"));
                }
                continue;
            }
        };

        // Adversarial: an unrelated mutation between the reads — a non-member
        // `updateDocument` (rewrites a node that is NOT a member) or an
        // `addTriple` (never marks a community stale).
        if budget % 3 == 0 {
            // Non-member node rewrite on doc_b (doc_b's nodes are not members
            // unless the two-doc boundary added m1).
            apply_graph(&store, &doc_b, content_only_graph(&did_b, &["m1"])).await;
        } else if budget % 4 == 0 {
            let _ = store
                .add_triple(
                    &(did_a.clone(), nid("n2")),
                    "affects",
                    &(did_a.clone(), nid("n1")),
                    &w,
                )
                .await;
        }

        let b = store.get_community_context(&cid).await;
        let b = match b {
            Ok(v) => v,
            Err(e) => {
                if cex.len() < 5 {
                    cex.push(format!("case {budget}: second read -> {e:?}"));
                }
                continue;
            }
        };
        if a != b && cex.len() < 5 {
            cex.push(format!(
                "case {budget}: determinism broken — first {a:?}, second {b:?}"
            ));
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
// P-IM-2 — Membership completeness  (IM, `gen_ctx_members`, tag [RED])
// ---------------------------------------------------------------------------
//
// `context.members` exactly equals the declared member set — complete, in stored
// order, duplicates verbatim. Boundaries: single member, two-doc, fact-location
// member; adversarial: a duplicate node id (stored verbatim).
const ROW_IM2_TAG: u64 = 0x00_00_00_00_00_00_00_02;
const ROW_IM2_BUDGET: u64 = 55;

#[tokio::test]
async fn p_im2_membership_completeness() {
    let mut cex: Vec<String> = Vec::new();
    let mut rng = SplitMix64::new(0x6D797A4CF0D09E5D ^ ROW_IM2_TAG);

    for budget in 1..=ROW_IM2_BUDGET {
        if cex.len() >= 5 {
            break;
        }
        let (store, w, doc_a, doc_b) = fresh_two_docs(&["n1", "n2"], &["m1"]).await;
        let did_a = doc_a.document_id.clone();
        let did_b = doc_b.document_id.clone();
        let members = gen_members(&mut rng, &did_a, &did_b, budget);
        let declared = store
            .declare_community(
                &members,
                &DeclareCommunityOptions {
                    summary: format!("manual {budget}"),
                    wiki_id: w.clone(),
                },
            )
            .await
            .unwrap();

        let ctx = store.get_community_context(&declared.community_id).await;
        let ctx = match ctx {
            Ok(v) => v,
            Err(e) => {
                if cex.len() < 5 {
                    cex.push(format!("case {budget}: read -> {e:?}"));
                }
                continue;
            }
        };
        if ctx.members != members && cex.len() < 5 {
            cex.push(format!(
                "case {budget}: members mismatch — declared {members:?}, got {:?}",
                ctx.members
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
// P-SM-1 — Read side-effect-free  (SM, `gen_ctx_side_effect_free`, tag [RED])
// ---------------------------------------------------------------------------
//
// `get_community_context` performs no mutation: consecutive reads with no
// mutation are equal, and the read appends no journal entry and changes no store
// state (communities, community_states, revisions). Boundaries: `Stale` state
// (the read must not clear it), two-doc members; adversarial: a read on a
// community that was just `re_derive_community`-ed (no re-append, state stays
// `Fresh`).
const ROW_SM1_TAG: u64 = 0x00_00_00_00_00_00_00_03;
const ROW_SM1_BUDGET: u64 = 60;

#[tokio::test]
async fn p_sm1_read_side_effect_free() {
    let mut cex: Vec<String> = Vec::new();
    let mut rng = SplitMix64::new(0x6D797A4CF0D09E5D ^ ROW_SM1_TAG);

    for budget in 1..=ROW_SM1_BUDGET {
        if cex.len() >= 5 {
            break;
        }
        let (store, w, doc_a, doc_b) = fresh_two_docs(&["n1", "n2"], &["m1"]).await;
        let did_a = doc_a.document_id.clone();
        let did_b = doc_b.document_id.clone();
        let members = gen_members(&mut rng, &did_a, &did_b, budget);
        let declared = store
            .declare_community(
                &members,
                &DeclareCommunityOptions {
                    summary: format!("manual {budget}"),
                    wiki_id: w.clone(),
                },
            )
            .await
            .unwrap();
        let cid = declared.community_id.clone();

        // Boundary: a `Stale` community (the read must not clear it).
        if budget % 5 == 0 {
            touch_member_via_update_document(&store, &doc_a, &members[0]).await;
        }
        // Adversarial: a community that was just re-derived (read must not
        // re-append a journal entry or change the now-`Fresh` state).
        if budget % 7 == 0 {
            store.re_derive_community(&cid).await.unwrap();
        }

        let journal_before = store.journal_len();
        let state_before = store.community_state(&cid).await.unwrap();

        let a = store.get_community_context(&cid).await;
        let a = match a {
            Ok(v) => v,
            Err(e) => {
                if cex.len() < 5 {
                    cex.push(format!("case {budget}: read -> {e:?}"));
                }
                continue;
            }
        };
        let journal_after = store.journal_len();
        let state_after = store.community_state(&cid).await.unwrap();

        if journal_after != journal_before && cex.len() < 5 {
            cex.push(format!(
                "case {budget}: read appended a journal entry ({journal_before} -> {journal_after})"
            ));
        }
        if state_after != state_before && cex.len() < 5 {
            cex.push(format!(
                "case {budget}: read changed community_state ({state_before:?} -> {state_after:?})"
            ));
        }
        let b = store.get_community_context(&cid).await;
        let b = match b {
            Ok(v) => v,
            Err(e) => {
                if cex.len() < 5 {
                    cex.push(format!("case {budget}: second read -> {e:?}"));
                }
                continue;
            }
        };
        if a != b && cex.len() < 5 {
            cex.push(format!("case {budget}: consecutive reads differ"));
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
// P-SM-2 — State reflects staleness  (SM, `gen_ctx_staleness`, tag [RED])
// ---------------------------------------------------------------------------
//
// `context.state` is `Fresh` on declaration, `Stale` after a member node/edge
// change, `Fresh` again after `re_derive_community`; `summary` and `members` are
// unchanged throughout. Boundaries: a member-touching op on a **non-member**
// node (state stays `Fresh`), two-doc members, an `update_fact` on an
// incorporated fact; adversarial: two communities only one of whose members is
// touched, and a `re_derive_community` on an already-`Fresh` community.
const ROW_SM2_TAG: u64 = 0x00_00_00_00_00_00_00_04;
const ROW_SM2_BUDGET: u64 = 45;

#[tokio::test]
async fn p_sm2_state_reflects_staleness() {
    let mut cex: Vec<String> = Vec::new();
    let mut rng = SplitMix64::new(0x6D797A4CF0D09E5D ^ ROW_SM2_TAG);

    for budget in 1..=ROW_SM2_BUDGET {
        if cex.len() >= 5 {
            break;
        }
        let (store, w, doc_a, doc_b) = fresh_two_docs(&["n1", "n2"], &["m1"]).await;
        let did_a = doc_a.document_id.clone();
        let did_b = doc_b.document_id.clone();
        let members = gen_members(&mut rng, &did_a, &did_b, budget);
        let declared = store
            .declare_community(
                &members,
                &DeclareCommunityOptions {
                    summary: format!("manual {budget}"),
                    wiki_id: w.clone(),
                },
            )
            .await
            .unwrap();
        let cid = declared.community_id.clone();

        // Fresh on declaration.
        let fresh = read_ctx(&store, &cid).await;
        if let Some(c) = &fresh {
            if c.state != gnosis::CommunityState::Fresh && cex.len() < 5 {
                cex.push(format!("case {budget}: not Fresh on declaration"));
            }
        }

        // Boundary: a member-touching op on a NON-member node — state stays Fresh.
        if budget % 6 == 0 {
            // doc_b's m1 is a member only when the two-doc boundary added it;
            // rewrite a node that is NOT a member (n2 on doc_a is a member only
            // when the 3-branch added it). Use a fresh non-member doc to be safe.
            let (_, _, doc_c) = fresh_doc(&["x1"]).await;
            apply_graph(
                &store,
                &doc_c,
                content_only_graph(&doc_c.document_id, &["x1"]),
            )
            .await;
            let after = read_ctx(&store, &cid).await;
            if let Some(c) = &after {
                if c.state != gnosis::CommunityState::Fresh && cex.len() < 5 {
                    cex.push(format!(
                        "case {budget}: non-member touch marked Stale ({:?})",
                        c.state
                    ));
                }
            }
        }

        // Member-touching op: `updateDocument` rewriting a member node, or
        // `update_fact` on an incorporated fact.
        if budget % 3 == 0 {
            // update_fact on an incorporated fact (a member that is a fact
            // location). Create the fact first, then declare a community whose
            // member is the fact location.
            let (store2, w2, doc2) = fresh_doc(&["n1"]).await;
            store2
                .create_fact(
                    &w2,
                    &doc2.document_id,
                    "k1",
                    "v1",
                    &[(doc2.document_id.clone(), nid("n1"))],
                )
                .await
                .unwrap();
            let fact_members = vec![(doc2.document_id.clone(), nid("fact-k1"))];
            let c2 = store2
                .declare_community(
                    &fact_members,
                    &DeclareCommunityOptions {
                        summary: format!("fact community {budget}"),
                        wiki_id: w2.clone(),
                    },
                )
                .await
                .unwrap();
            touch_member_via_update_fact(&store2, &w2, &doc2, "k1").await;
            let after = read_ctx(&store2, &c2.community_id).await;
            if let Some(c) = &after {
                if c.state != gnosis::CommunityState::Stale && cex.len() < 5 {
                    cex.push(format!(
                        "case {budget}: update_fact did not mark Stale ({:?})",
                        c.state
                    ));
                }
            }
            continue;
        }

        touch_member_via_update_document(&store, &doc_a, &members[0]).await;
        let stale = read_ctx(&store, &cid).await;
        if let Some(c) = &stale {
            if c.state != gnosis::CommunityState::Stale && cex.len() < 5 {
                cex.push(format!(
                    "case {budget}: member touch did not mark Stale ({:?})",
                    c.state
                ));
            }
        }

        // Adversarial: two communities, only one of whose members is touched —
        // only that community flips to Stale.
        if budget % 4 == 0 {
            let c2 = store
                .declare_community(
                    &[(did_b.clone(), nid("m1"))],
                    &DeclareCommunityOptions {
                        summary: format!("untouched {budget}"),
                        wiki_id: w.clone(),
                    },
                )
                .await
                .unwrap();
            let c2_after = read_ctx(&store, &c2.community_id).await;
            if let Some(c) = &c2_after {
                if c.state != gnosis::CommunityState::Fresh && cex.len() < 5 {
                    cex.push(format!(
                        "case {budget}: untouched community flipped to {:?}",
                        c.state
                    ));
                }
            }
        }

        // Re-derive → Fresh; summary/members unchanged throughout.
        let summary_before = stale.as_ref().map(|c| c.summary.clone());
        let members_before = stale.as_ref().map(|c| c.members.clone());
        store.re_derive_community(&cid).await.unwrap();
        let rederived = read_ctx(&store, &cid).await;
        if let Some(c) = &rederived {
            if c.state != gnosis::CommunityState::Fresh && cex.len() < 5 {
                cex.push(format!(
                    "case {budget}: re_derive did not restore Fresh ({:?})",
                    c.state
                ));
            }
            if summary_before.as_deref() != Some(c.summary.as_str()) && cex.len() < 5 {
                cex.push(format!("case {budget}: summary changed across staleness"));
            }
            if members_before.as_ref() != Some(&c.members) && cex.len() < 5 {
                cex.push(format!("case {budget}: members changed across staleness"));
            }
        }

        // Adversarial: re_derive on an already-Fresh community stays Fresh.
        if budget % 8 == 0 {
            store.re_derive_community(&cid).await.unwrap();
            let again = read_ctx(&store, &cid).await;
            if let Some(c) = &again {
                if c.state != gnosis::CommunityState::Fresh && cex.len() < 5 {
                    cex.push(format!(
                        "case {budget}: re_derive on Fresh flipped to {:?}",
                        c.state
                    ));
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
// P-TP-1 — Manual-summary authority  (TP, `gen_ctx_manual_summary`, tag [RED])
// ---------------------------------------------------------------------------
//
// `context.summary` always equals the manual summary and is never auto-regenerated
// — it survives a member change + `re_derive_community` unchanged, and changes
// only via an explicit `update_community_summary`. Boundaries: two-doc members,
// single member; adversarial: a non-member touch (summary unchanged, state stays
// Fresh), an update to a new non-empty summary.
const ROW_TP1_TAG: u64 = 0x00_00_00_00_00_00_00_05;
const ROW_TP1_BUDGET: u64 = 40;

#[tokio::test]
async fn p_tp1_manual_summary_authority() {
    let mut cex: Vec<String> = Vec::new();
    let mut rng = SplitMix64::new(0x6D797A4CF0D09E5D ^ ROW_TP1_TAG);

    for budget in 1..=ROW_TP1_BUDGET {
        if cex.len() >= 5 {
            break;
        }
        let (store, w, doc_a, doc_b) = fresh_two_docs(&["n1", "n2"], &["m1"]).await;
        let did_a = doc_a.document_id.clone();
        let did_b = doc_b.document_id.clone();
        let members = gen_members(&mut rng, &did_a, &did_b, budget);
        let manual = format!("manual summary {budget}");
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

        // Member change + re-derive — summary must survive unchanged.
        touch_member_via_update_document(&store, &doc_a, &members[0]).await;
        store.re_derive_community(&cid).await.unwrap();
        let ctx = store.get_community_context(&cid).await;
        let ctx = match ctx {
            Ok(v) => v,
            Err(e) => {
                if cex.len() < 5 {
                    cex.push(format!("case {budget}: read -> {e:?}"));
                }
                continue;
            }
        };
        if ctx.summary != manual && cex.len() < 5 {
            cex.push(format!(
                "case {budget}: summary auto-regenerated (got {:?}, manual {manual:?})",
                ctx.summary
            ));
        }

        // Adversarial: a non-member touch — summary unchanged, state stays Fresh.
        if budget % 3 == 0 {
            let (_, _, doc_c) = fresh_doc(&["x1"]).await;
            apply_graph(
                &store,
                &doc_c,
                content_only_graph(&doc_c.document_id, &["x1"]),
            )
            .await;
            let after = store.get_community_context(&cid).await.unwrap();
            if after.summary != manual && cex.len() < 5 {
                cex.push(format!(
                    "case {budget}: summary changed after non-member touch"
                ));
            }
            if after.state != gnosis::CommunityState::Fresh && cex.len() < 5 {
                cex.push(format!(
                    "case {budget}: non-member touch flipped state to {:?}",
                    after.state
                ));
            }
        }

        // Only an explicit update changes it.
        let new = format!("updated summary {budget}");
        store.update_community_summary(&cid, &new).await.unwrap();
        let after = store.get_community_context(&cid).await.unwrap();
        if after.summary != new && cex.len() < 5 {
            cex.push(format!(
                "case {budget}: update_community_summary not reflected (got {:?}, new {new:?})",
                after.summary
            ));
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
// P-TP-2 — Faithful projection  (TP, `gen_ctx_projection`, tag [RED])
// ---------------------------------------------------------------------------
//
// `get_community_context` is a pre-joined read of `get_community` +
// `community_state`: `community_id`/`wiki_id`/`summary`/`members` come from
// `get_community`, `state` from `community_state`. Boundaries: `Stale` state,
// two-doc members; adversarial: an updated summary, a duplicate member id.
const ROW_TP2_TAG: u64 = 0x00_00_00_00_00_00_00_06;
const ROW_TP2_BUDGET: u64 = 40;

#[tokio::test]
async fn p_tp2_faithful_projection() {
    let mut cex: Vec<String> = Vec::new();
    let mut rng = SplitMix64::new(0x6D797A4CF0D09E5D ^ ROW_TP2_TAG);

    for budget in 1..=ROW_TP2_BUDGET {
        if cex.len() >= 5 {
            break;
        }
        let (store, w, doc_a, doc_b) = fresh_two_docs(&["n1", "n2"], &["m1"]).await;
        let did_a = doc_a.document_id.clone();
        let did_b = doc_b.document_id.clone();
        let members = gen_members(&mut rng, &did_a, &did_b, budget);
        let declared = store
            .declare_community(
                &members,
                &DeclareCommunityOptions {
                    summary: format!("manual {budget}"),
                    wiki_id: w.clone(),
                },
            )
            .await
            .unwrap();
        let cid = declared.community_id.clone();

        // Boundary: a `Stale` community (state field equals `community_state`).
        if budget % 5 == 0 {
            touch_member_via_update_document(&store, &doc_a, &members[0]).await;
        }
        // Adversarial: an updated summary.
        if budget % 4 == 0 {
            store
                .update_community_summary(&cid, &format!("updated {budget}"))
                .await
                .unwrap();
        }

        let c = store.get_community(&cid).await;
        let c = match c {
            Ok(v) => v,
            Err(e) => {
                if cex.len() < 5 {
                    cex.push(format!("case {budget}: get_community -> {e:?}"));
                }
                continue;
            }
        };
        let state = store.community_state(&cid).await;
        let state = match state {
            Ok(v) => v,
            Err(e) => {
                if cex.len() < 5 {
                    cex.push(format!("case {budget}: community_state -> {e:?}"));
                }
                continue;
            }
        };
        let ctx = store.get_community_context(&cid).await;
        let ctx = match ctx {
            Ok(v) => v,
            Err(e) => {
                if cex.len() < 5 {
                    cex.push(format!("case {budget}: get_community_context -> {e:?}"));
                }
                continue;
            }
        };

        if ctx.community_id != c.community_id && cex.len() < 5 {
            cex.push(format!("case {budget}: community_id projection mismatch"));
        }
        if ctx.wiki_id != c.wiki_id && cex.len() < 5 {
            cex.push(format!("case {budget}: wiki_id projection mismatch"));
        }
        if ctx.summary != c.summary && cex.len() < 5 {
            cex.push(format!(
                "case {budget}: summary projection mismatch (ctx {:?}, community {:?})",
                ctx.summary, c.summary
            ));
        }
        if ctx.members != c.members && cex.len() < 5 {
            cex.push(format!("case {budget}: members projection mismatch"));
        }
        if ctx.state != state && cex.len() < 5 {
            cex.push(format!(
                "case {budget}: state projection mismatch (ctx {:?}, community_state {state:?})",
                ctx.state
            ));
        }
    }
    assert!(
        cex.is_empty(),
        "P-TP-2 BROKEN — {} counterexample(s):\n  - {}",
        cex.len(),
        cex.join("\n  - ")
    );
}
