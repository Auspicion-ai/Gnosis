//! §4.4 Consistency-enforcement integration tests — written **RED-first** at the
//! TDD gate for the §4.4 unit.
//!
//! Every state and fail-state below is derived from the canonical behavior
//! contract alone (`docs/specs/gnosis.md` §4.4.1–§4.4.5) plus the concurrency
//! contract (`docs/research/gnosis-data-structures-concurrency-plan.md`:
//! SHARDED-RWLOCK-STORE / ARC-SHARED-ENGINE / atomic mutation under a lock).
//!
//! ## The suite is RED
//!
//! The §4.4 `RagStore` surface added by this TestWriter — `get_consistency_report`,
//! `re_sync_embed`, `re_derive_community`, `community_state` — has **compiling
//! stub bodies** (`unimplemented!()`), and the §4.4.1a staleness-propagation
//! hooks (fact-update / member-change → dependents stale) are NOT implemented, so
//! every test that touches the §4.4 surface fails at runtime. The §4.1/§4.2/§4.3
//! suites are untouched and stay green.
//!
//! ## TestWriter contract decisions (for the Implementer to match exactly)
//!
//! - **Canonical source of truth.** A fact's canonical value is its `Fact.value`
//!   in `fact_store` (§4.3.1), read via `create_fact`/`get_fact`/`update_fact`.
//!   Its location is `(Fact.document_id, Fact.node_id)` where `Fact.node_id =
//!   "fact-{fact_key}"` (as `create_fact` records it today).
//! - **Embed snapshot storage.** The frozen §4.1 `Node`/`Edge` carry **no** embed
//!   snapshot field. The embed's snapshot is the reference **node's** `value` and
//!   the embed edge's stored `ReferenceState` (§4.2.2: an embed "stores a
//!   snapshot of the target's value plus the target pointer"). A `link` node has
//!   `value: None` (no copy; resolved live).
//! - **Report scope (§4.4.4).** `get_consistency_report(wikiId)` returns one
//!   `ConsistencyReferenceReport` per **reference edge whose source document is in
//!   `wikiId`** — including `crosslink` edges (source in the wiki, target in
//!   another). `state` is the reference edge's stored `ReferenceState`; the report
//!   derives it from the store (`set_reference_state` / staleness propagation both
//!   write it).
//! - **Staleness triggers (§4.4.3).** (a) `update_fact` marks every embed edge
//!   whose `target == (fact.document_id, fact.node_id)` (in the wiki *and* across
//!   wikis) `STALE`, and every community whose members include that location
//!   `STALE`; links to the fact stay `RESOLVED`. (b) A graph member-node edit
//!   (`update_document` rewriting a node that is a community member) marks the
//!   community `STALE`. (c) Archive of a reference target marks links `BROKEN` and
//!   embeds `STALE`.
//! - **Re-sync (§4.4.3).** `re_sync_embed(document_id, node_id)` updates the
//!   reference node's `value` to the target fact's canonical `Fact.value` (found
//!   in `fact_store` at the embed edge's target), flips the embed edge to `FRESH`,
//!   and bumps the referencing document's `revision` (revision `+1`).
//! - **Community staleness surface (§4.4.1a vs §4.4.4).** §4.4.4 pins the report
//!   shape to reference kinds (`kind: 'link'|'embed'|'crosslink'`), so community
//!   staleness — the §4.4.1a third propagator — is surfaced through
//!   `community_state(community_id) → CommunityState::{Fresh,Stale}` (and through
//!   the publish gate), NOT as a report row of a non-reference kind. Tests assert
//!   community staleness via `community_state` for this reason.
//! - **Publish gate (§4.4.3).** `publish_document` fails with `UnresolvedReference`
//!   when the document contains a `STALE` embed not re-synced, a `STALE` community
//!   not re-derived, or any `BROKEN` link; a `BROKEN` link is never resolved by
//!   re-syncing an unrelated embed. (The `STALE` embed / `BROKEN` link branch is
//!   already covered by the green §4.1 gate; these tests also pin it.)
//!
//! ## The §4.4.1a report-surfaces-all-three reading
//!
//! Staleness propagates through facts, links/embeds, AND communities (§4.4.1a).
//! The reference report surfaces the link/embed/crosslink propagator; the
//! community propagator is surfaced through `community_state` (see decision
//! above) — together the report accessor and `community_state` cover §4.4.1a's
//! three propagators.

use std::sync::Arc;

use gnosis::{
    CommunityId, ConsistencyReferenceReport, CreateDocumentRequest, DeclareCommunityOptions,
    DocState, Document, DocumentId, Edge, EdgeKind, Graph, Node, NodeId, NodeKind, RagStore,
    ReferenceState, Store, StoreError, UpdateDocumentRequest, WikiId,
};

// ---------------------------------------------------------------------------
// Helpers (mirroring the §4.2 / §4.3 suites)
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

/// A `reference` node. `snapshot` is the embed's copied value (`Some`) for an
/// `embed` edge or `None` for a `link` (no copy stored, §4.2.2).
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

/// A graph with a single content node `cite` used to ground a fact's citation
/// (§4.3.2 minimum-citation invariant needs ≥1 resolvable citation node).
fn citation_graph(doc: &DocumentId) -> Graph {
    let cite = content_node(doc, "cite", "cited-content");
    Graph {
        nodes: vec![cite.clone()],
        edges: vec![
            edge(
                EdgeKind::DocHead,
                (doc.clone(), nid("ROOT")),
                (doc.clone(), cite.node_id.clone()),
                None,
                false,
            ),
            edge(
                EdgeKind::DocEnd,
                (doc.clone(), cite.node_id.clone()),
                (doc.clone(), nid("END")),
                None,
                false,
            ),
        ],
    }
}

/// Create a fact `fact_key` with value `value` in `src_doc` (grounded in its
/// `cite` node). Returns the fact's graph location `(document_id, node_id)`.
async fn seed_fact(
    store: &Store,
    w: &WikiId,
    src_doc: &Document,
    fact_key: &str,
    value: &str,
) -> (DocumentId, NodeId) {
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

/// Find a report row for `node_id` of `kind`.
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

// ---------------------------------------------------------------------------
// §4.4.3 — Staleness on fact update
// ---------------------------------------------------------------------------

/// State: updating a fact's `value` marks every `embed` snapshotting it `STALE`
/// (in the same wiki and across wikis), marks the community incorporating the
/// fact `STALE`, and leaves a non-embed `link` to it `RESOLVED` (§4.4.3
/// "On fact update"). RED — `get_consistency_report`/`community_state` are stubs.
#[tokio::test]
async fn fact_update_marks_embeds_stale_links_resolved_and_community_stale() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let w2 = new_wiki(&store, "w2").await;

    // src holds fact `license` = "MIT" at (src, fact-license).
    let src = new_doc(&store, &w, "source").await;
    let fact_loc = seed_fact(&store, &w, &src, "license", "MIT").await;

    // Referencing doc (same wiki) with an embed of the fact (fresh snapshot).
    let d1 = new_doc(&store, &w, "referrer").await;
    let ref1 = ref_node(&d1.document_id, "ref-embed", fact_loc.clone(), Some("MIT"));
    let ref_link = ref_node(&d1.document_id, "ref-link", fact_loc.clone(), None);
    apply_graph(
        &store,
        &d1,
        Graph {
            nodes: vec![
                content_node(&d1.document_id, "n", "x"),
                ref1.clone(),
                ref_link.clone(),
            ],
            edges: vec![
                edge(
                    EdgeKind::DocHead,
                    (d1.document_id.clone(), nid("ROOT")),
                    (d1.document_id.clone(), nid("n")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::DocEnd,
                    (d1.document_id.clone(), nid("n")),
                    (d1.document_id.clone(), nid("END")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::Embed,
                    (d1.document_id.clone(), ref1.node_id.clone()),
                    fact_loc.clone(),
                    Some(ReferenceState::Fresh),
                    false,
                ),
                edge(
                    EdgeKind::Link,
                    (d1.document_id.clone(), ref_link.node_id.clone()),
                    fact_loc.clone(),
                    Some(ReferenceState::Resolved),
                    false,
                ),
            ],
        },
    )
    .await;

    // Cross-wiki embed: a referrer doc in w2 embeds w's fact (a Crosslink edge).
    let d2 = new_doc(&store, &w2, "cross-referrer").await;
    let ref2 = ref_node(&d2.document_id, "xref-embed", fact_loc.clone(), Some("MIT"));
    apply_graph(
        &store,
        &d2,
        Graph {
            nodes: vec![ref2.clone()],
            edges: vec![
                edge(
                    EdgeKind::DocHead,
                    (d2.document_id.clone(), nid("ROOT")),
                    (d2.document_id.clone(), ref2.node_id.clone()),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::DocEnd,
                    (d2.document_id.clone(), ref2.node_id.clone()),
                    (d2.document_id.clone(), nid("END")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::Crosslink,
                    (d2.document_id.clone(), ref2.node_id.clone()),
                    fact_loc.clone(),
                    Some(ReferenceState::Fresh),
                    true,
                ),
            ],
        },
    )
    .await;

    // A community incorporating the fact.
    let community = store
        .declare_community(
            std::slice::from_ref(&fact_loc),
            &DeclareCommunityOptions {
                summary: "facts of wiki w".to_string(),
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap();

    // §4.4.3 trigger: the fact value changes.
    store
        .update_fact(
            &w,
            "license",
            &gnosis::UpdateFactRequest {
                value: "Apache-2.0".to_string(),
                citations: vec![(src.document_id.clone(), nid("cite"))],
            },
        )
        .await
        .unwrap();

    // Same-wiki embed → STALE; link stays RESOLVED.
    let report_w = store.get_consistency_report(&w).await.unwrap();
    assert_eq!(
        find_row(&report_w, "ref-embed", EdgeKind::Embed).state,
        ReferenceState::Stale,
        "embed snapshotting a changed fact becomes STALE (same wiki)"
    );
    assert_eq!(
        find_row(&report_w, "ref-link", EdgeKind::Link).state,
        ReferenceState::Resolved,
        "a link to a changed fact stays RESOLVED (resolved live, §4.4.2)"
    );

    // Cross-wiki embed → STALE (surfaced in w2's report).
    let report_w2 = store.get_consistency_report(&w2).await.unwrap();
    assert_eq!(
        find_row(&report_w2, "xref-embed", EdgeKind::Crosslink).state,
        ReferenceState::Stale,
        "embed across wikis becomes STALE too (§4.4.3)"
    );

    // Community incorporating the fact → STALE.
    assert_eq!(
        store
            .community_state(&community.community_id)
            .await
            .unwrap(),
        gnosis::CommunityState::Stale,
        "a community incorporating a changed fact becomes STALE (§4.4.1a)"
    );
}

// ---------------------------------------------------------------------------
// §4.4.3 — Staleness on reference target deleted/archived
// ---------------------------------------------------------------------------

/// State: archiving a reference target marks a `link` to it `BROKEN` and an
/// `embed` to it `STALE` (§4.4.2/§4.4.3). RED — `get_consistency_report` stub.
#[tokio::test]
async fn archiving_target_marks_link_broken_and_embed_stale() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;

    // Target doc with two nodes.
    let tgt = new_doc(&store, &w, "target").await;
    let tgt_graph = Graph {
        nodes: vec![
            content_node(&tgt.document_id, "n1", "first"),
            content_node(&tgt.document_id, "n2", "second"),
        ],
        edges: vec![
            edge(
                EdgeKind::DocHead,
                (tgt.document_id.clone(), nid("ROOT")),
                (tgt.document_id.clone(), nid("n1")),
                None,
                false,
            ),
            edge(
                EdgeKind::DocEnd,
                (tgt.document_id.clone(), nid("n1")),
                (tgt.document_id.clone(), nid("END")),
                None,
                false,
            ),
        ],
    };
    apply_graph(&store, &tgt, tgt_graph).await;

    // Referrer with a link to tgt.n1 and an embed of tgt.n2.
    let d = new_doc(&store, &w, "referrer").await;
    let link_ref = ref_node(
        &d.document_id,
        "l",
        (tgt.document_id.clone(), nid("n1")),
        None,
    );
    let embed_ref = ref_node(
        &d.document_id,
        "e",
        (tgt.document_id.clone(), nid("n2")),
        Some("second"),
    );
    apply_graph(
        &store,
        &d,
        Graph {
            nodes: vec![
                content_node(&d.document_id, "n", "x"),
                link_ref.clone(),
                embed_ref.clone(),
            ],
            edges: vec![
                edge(
                    EdgeKind::DocHead,
                    (d.document_id.clone(), nid("ROOT")),
                    (d.document_id.clone(), nid("n")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::DocEnd,
                    (d.document_id.clone(), nid("n")),
                    (d.document_id.clone(), nid("END")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::Link,
                    (d.document_id.clone(), link_ref.node_id.clone()),
                    (tgt.document_id.clone(), nid("n1")),
                    Some(ReferenceState::Resolved),
                    false,
                ),
                edge(
                    EdgeKind::Embed,
                    (d.document_id.clone(), embed_ref.node_id.clone()),
                    (tgt.document_id.clone(), nid("n2")),
                    Some(ReferenceState::Fresh),
                    false,
                ),
            ],
        },
    )
    .await;

    // Archive the target → references must be marked.
    store.archive_document(&tgt.document_id).await.unwrap();

    let report = store.get_consistency_report(&w).await.unwrap();
    assert_eq!(
        find_row(&report, "l", EdgeKind::Link).state,
        ReferenceState::Broken,
        "a link to an archived target is BROKEN (§4.4.2)"
    );
    assert_eq!(
        find_row(&report, "e", EdgeKind::Embed).state,
        ReferenceState::Stale,
        "an embed of an archived target is STALE (§4.4.3)"
    );
}

// ---------------------------------------------------------------------------
// §4.4.1a/§4.4.3 — Community staleness on member change
// ---------------------------------------------------------------------------

/// State: editing a node that is a community member marks the community `STALE`
/// (§4.4.1a/§4.4.3, the third propagator); surfaced via `community_state`.
/// RED — `community_state` stub.
#[tokio::test]
async fn member_node_change_marks_community_stale() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "member-host").await;

    let m = content_node(&doc.document_id, "member1", "v1");
    apply_graph(
        &store,
        &doc,
        Graph {
            nodes: vec![m.clone()],
            edges: vec![
                edge(
                    EdgeKind::DocHead,
                    (doc.document_id.clone(), nid("ROOT")),
                    (doc.document_id.clone(), m.node_id.clone()),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::DocEnd,
                    (doc.document_id.clone(), m.node_id.clone()),
                    (doc.document_id.clone(), nid("END")),
                    None,
                    false,
                ),
            ],
        },
    )
    .await;

    let community = store
        .declare_community(
            &[(doc.document_id.clone(), nid("member1"))],
            &DeclareCommunityOptions {
                summary: "a community of one".to_string(),
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap();
    assert_eq!(
        store
            .community_state(&community.community_id)
            .await
            .unwrap(),
        gnosis::CommunityState::Fresh,
        "a freshly declared community is Fresh (§4.2.8.3)"
    );

    // §4.4.3 trigger: a member node's value changes → community STALE.
    let m2 = content_node(&doc.document_id, "member1", "v2");
    apply_graph(
        &store,
        &doc,
        Graph {
            nodes: vec![m2.clone()],
            edges: vec![
                edge(
                    EdgeKind::DocHead,
                    (doc.document_id.clone(), nid("ROOT")),
                    (doc.document_id.clone(), m2.node_id.clone()),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::DocEnd,
                    (doc.document_id.clone(), m2.node_id.clone()),
                    (doc.document_id.clone(), nid("END")),
                    None,
                    false,
                ),
            ],
        },
    )
    .await;

    assert_eq!(
        store
            .community_state(&community.community_id)
            .await
            .unwrap(),
        gnosis::CommunityState::Stale,
        "a community whose member changed is STALE until re-derived"
    );
}

/// Fail-state: `community_state` on an unknown community → `CommunityNotFound`.
/// RED — `community_state` stub (and the fail-state is not enforced).
#[tokio::test]
async fn community_state_unknown_is_community_not_found() {
    let store = Arc::new(Store::new());
    let err = store
        .community_state(&CommunityId("ghost".into()))
        .await
        .unwrap_err();
    assert_eq!(err, StoreError::CommunityNotFound, "unknown community");
}

// ---------------------------------------------------------------------------
// §4.4.4 — getConsistencyReport shape + scope
// ---------------------------------------------------------------------------

/// State: the report returns one row per reference edge in the wiki with the
/// correct `{documentId, nodeId, kind, state, target, crossWiki}` fields, and
/// spans link / embed / crosslink kinds (§4.4.4). RED — `get_consistency_report`
/// stub.
#[tokio::test]
async fn consistency_report_lists_every_reference_with_fields() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let w2 = new_wiki(&store, "w2").await;

    // src holds a fact; d1 embeds + links it (same wiki); d2 crosslinks it.
    let src = new_doc(&store, &w, "source").await;
    let fact_loc = seed_fact(&store, &w, &src, "license", "MIT").await;

    let d1 = new_doc(&store, &w, "referrer").await;
    let e1 = ref_node(&d1.document_id, "em", fact_loc.clone(), Some("MIT"));
    let l1 = ref_node(&d1.document_id, "lk", fact_loc.clone(), None);
    apply_graph(
        &store,
        &d1,
        Graph {
            nodes: vec![
                content_node(&d1.document_id, "n", "x"),
                e1.clone(),
                l1.clone(),
            ],
            edges: vec![
                edge(
                    EdgeKind::DocHead,
                    (d1.document_id.clone(), nid("ROOT")),
                    (d1.document_id.clone(), nid("n")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::DocEnd,
                    (d1.document_id.clone(), nid("n")),
                    (d1.document_id.clone(), nid("END")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::Embed,
                    (d1.document_id.clone(), e1.node_id.clone()),
                    fact_loc.clone(),
                    Some(ReferenceState::Fresh),
                    false,
                ),
                edge(
                    EdgeKind::Link,
                    (d1.document_id.clone(), l1.node_id.clone()),
                    fact_loc.clone(),
                    Some(ReferenceState::Resolved),
                    true,
                ),
            ],
        },
    )
    .await;

    let d2 = new_doc(&store, &w2, "cross").await;
    let x1 = ref_node(&d2.document_id, "xl", fact_loc.clone(), None);
    apply_graph(
        &store,
        &d2,
        Graph {
            nodes: vec![x1.clone()],
            edges: vec![
                edge(
                    EdgeKind::DocHead,
                    (d2.document_id.clone(), nid("ROOT")),
                    (d2.document_id.clone(), x1.node_id.clone()),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::DocEnd,
                    (d2.document_id.clone(), x1.node_id.clone()),
                    (d2.document_id.clone(), nid("END")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::Crosslink,
                    (d2.document_id.clone(), x1.node_id.clone()),
                    fact_loc.clone(),
                    Some(ReferenceState::Resolved),
                    true,
                ),
            ],
        },
    )
    .await;

    // w2's doc is a crosslink (its target in w); d2 belongs to w2, so it is NOT
    // part of w's report.
    let report = store.get_consistency_report(&w).await.unwrap();
    let row_e = find_row(&report, "em", EdgeKind::Embed);
    assert_eq!(
        row_e.document_id, d1.document_id,
        "source (referencing) doc"
    );
    assert_eq!(row_e.kind, EdgeKind::Embed);
    assert_eq!(row_e.state, ReferenceState::Fresh);
    assert_eq!(row_e.target, fact_loc);
    assert!(!row_e.cross_wiki);

    let row_l = find_row(&report, "lk", EdgeKind::Link);
    assert_eq!(row_l.state, ReferenceState::Resolved);
    assert!(row_l.cross_wiki, "cross-link flagged crossWiki");

    // w2's report surfaces its own crosslink (source doc in w2).
    let report2 = store.get_consistency_report(&w2).await.unwrap();
    let row_x = find_row(&report2, "xl", EdgeKind::Crosslink);
    assert_eq!(row_x.document_id, d2.document_id);
    assert!(row_x.cross_wiki);
}

/// Fail-state: `getConsistencyReport` on an unknown wiki → `WikiNotFound`.
/// RED — `get_consistency_report` stub.
#[tokio::test]
async fn consistency_report_unknown_wiki_is_wiki_not_found() {
    let store = Arc::new(Store::new());
    let err = store
        .get_consistency_report(&wiki("ghost"))
        .await
        .unwrap_err();
    assert_eq!(err, StoreError::WikiNotFound, "FS-2 unknown wiki");
}

// ---------------------------------------------------------------------------
// §4.4.3 — Re-sync
// ---------------------------------------------------------------------------

/// State: re-syncing a `STALE` embed updates its snapshot to the canonical value
/// and bumps the referencing document's `revision` (§4.4.3). RED — `re_sync_embed`
/// stub.
#[tokio::test]
async fn resync_embed_updates_snapshot_and_bumps_revision() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let src = new_doc(&store, &w, "source").await;
    let fact_loc = seed_fact(&store, &w, &src, "license", "MIT").await;

    // Referrer embeds the fact with a *stale* snapshot ("MTI" vs canonical "MIT").
    let d = new_doc(&store, &w, "referrer").await;
    let e = ref_node(&d.document_id, "em", fact_loc.clone(), Some("MTI"));
    apply_graph(
        &store,
        &d,
        Graph {
            nodes: vec![content_node(&d.document_id, "n", "x"), e.clone()],
            edges: vec![
                edge(
                    EdgeKind::DocHead,
                    (d.document_id.clone(), nid("ROOT")),
                    (d.document_id.clone(), nid("n")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::DocEnd,
                    (d.document_id.clone(), nid("n")),
                    (d.document_id.clone(), nid("END")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::Embed,
                    (d.document_id.clone(), e.node_id.clone()),
                    fact_loc.clone(),
                    Some(ReferenceState::Stale),
                    false,
                ),
            ],
        },
    )
    .await;

    let before = store.get_document(&d.document_id).await.unwrap().revision;

    let updated = store
        .re_sync_embed(&d.document_id, &nid("em"))
        .await
        .unwrap();

    // Snapshot updated to canonical; embed FRESH; revision bumped.
    assert_eq!(
        updated.revision,
        before + 1,
        "re-sync bumps the referencing document's revision"
    );
    let node = updated
        .graph
        .nodes
        .iter()
        .find(|n| n.node_id == nid("em"))
        .expect("the reference node survives re-sync");
    assert_eq!(
        node.value.as_deref(),
        Some("MIT"),
        "re-sync updates the snapshot to the canonical value"
    );

    let report = store.get_consistency_report(&w).await.unwrap();
    assert_eq!(
        find_row(&report, "em", EdgeKind::Embed).state,
        ReferenceState::Fresh,
        "a re-synced embed is no longer STALE"
    );
}

/// Fail-state: `re_sync_embed` on an unknown document → `DocumentNotFound`.
/// RED — `re_sync_embed` stub.
#[tokio::test]
async fn resync_embed_unknown_document_is_document_not_found() {
    let store = Arc::new(Store::new());
    let err = store
        .re_sync_embed(&did("ghost"), &nid("em"))
        .await
        .unwrap_err();
    assert_eq!(err, StoreError::DocumentNotFound, "unknown document");
}

/// State: re-deriving a `STALE` community clears its staleness (§4.4.3; summary
/// derivation is PARKED F4, so re-derive = clear STALE). RED — `re_derive_community`
/// and `community_state` stubs.
#[tokio::test]
async fn rederive_community_clears_stale_flag() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "host").await;
    let m = content_node(&doc.document_id, "member1", "v1");
    apply_graph(
        &store,
        &doc,
        Graph {
            nodes: vec![m.clone()],
            edges: vec![
                edge(
                    EdgeKind::DocHead,
                    (doc.document_id.clone(), nid("ROOT")),
                    (doc.document_id.clone(), m.node_id.clone()),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::DocEnd,
                    (doc.document_id.clone(), m.node_id.clone()),
                    (doc.document_id.clone(), nid("END")),
                    None,
                    false,
                ),
            ],
        },
    )
    .await;
    let community = store
        .declare_community(
            &[(doc.document_id.clone(), nid("member1"))],
            &DeclareCommunityOptions {
                summary: "s".to_string(),
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap();

    // Member change → STALE.
    let m2 = content_node(&doc.document_id, "member1", "v2");
    apply_graph(
        &store,
        &doc,
        Graph {
            nodes: vec![m2.clone()],
            edges: vec![
                edge(
                    EdgeKind::DocHead,
                    (doc.document_id.clone(), nid("ROOT")),
                    (doc.document_id.clone(), m2.node_id.clone()),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::DocEnd,
                    (doc.document_id.clone(), m2.node_id.clone()),
                    (doc.document_id.clone(), nid("END")),
                    None,
                    false,
                ),
            ],
        },
    )
    .await;
    assert_eq!(
        store
            .community_state(&community.community_id)
            .await
            .unwrap(),
        gnosis::CommunityState::Stale,
        "member change marks the community STALE"
    );

    let rederived = store
        .re_derive_community(&community.community_id)
        .await
        .unwrap();
    assert_eq!(
        rederived.summary, "s",
        "re-derive does not regenerate the manual summary (§4.2.8.4)"
    );
    assert_eq!(
        store
            .community_state(&community.community_id)
            .await
            .unwrap(),
        gnosis::CommunityState::Fresh,
        "re-derive clears the STALE flag"
    );
}

/// Fail-state: `re_derive_community` on an unknown community → `CommunityNotFound`.
/// RED — `re_derive_community` stub.
#[tokio::test]
async fn rederive_community_unknown_is_community_not_found() {
    let store = Arc::new(Store::new());
    let err = store
        .re_derive_community(&CommunityId("ghost".into()))
        .await
        .unwrap_err();
    assert_eq!(err, StoreError::CommunityNotFound, "unknown community");
}

// ---------------------------------------------------------------------------
// §4.4.3 — Publish gate (the enforcement point)
// ---------------------------------------------------------------------------

/// Fail-state §4.4.3: a document containing a `STALE` embed that has NOT been
/// re-synced fails publish with `UnresolvedReference`; after re-sync, publish
/// succeeds. The STALE-embed block is already enforced by the green §4.1 gate.
/// RED because `re_sync_embed` is a stub.
#[tokio::test]
async fn stale_embed_blocks_publish_until_resynced() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let src = new_doc(&store, &w, "source").await;
    let fact_loc = seed_fact(&store, &w, &src, "license", "MIT").await;

    let d = new_doc(&store, &w, "doc").await;
    let e = ref_node(&d.document_id, "em", fact_loc.clone(), Some("MTI"));
    apply_graph(
        &store,
        &d,
        Graph {
            nodes: vec![content_node(&d.document_id, "n", "x"), e.clone()],
            edges: vec![
                edge(
                    EdgeKind::DocHead,
                    (d.document_id.clone(), nid("ROOT")),
                    (d.document_id.clone(), nid("n")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::DocEnd,
                    (d.document_id.clone(), nid("n")),
                    (d.document_id.clone(), nid("END")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::Embed,
                    (d.document_id.clone(), e.node_id.clone()),
                    fact_loc.clone(),
                    Some(ReferenceState::Stale),
                    false,
                ),
            ],
        },
    )
    .await;

    let err = store.publish_document(&d.document_id).await.unwrap_err();
    assert_eq!(
        err,
        StoreError::UnresolvedReference,
        "a STALE embed (unsynced) blocks publish (§4.4.3)"
    );

    // Re-sync → snapshot canonical → publish succeeds.
    store
        .re_sync_embed(&d.document_id, &nid("em"))
        .await
        .unwrap();
    let published = store.publish_document(&d.document_id).await.unwrap();
    assert_eq!(
        published.state,
        DocState::Published,
        "after re-sync the STALE embed no longer blocks publish"
    );
}

/// Fail-state §4.4.3: a `STALE` community that has not been re-derived blocks the
/// publish of a document whose node is a member; after re-derive it succeeds.
/// RED — neither the community-staleness propagation nor the gate community check
/// is implemented (a STALE community is never established, so publish currently
/// succeeds).
#[tokio::test]
async fn stale_community_blocks_publish_until_rederived() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "doc").await;
    let m = content_node(&doc.document_id, "member1", "v1");
    apply_graph(
        &store,
        &doc,
        Graph {
            nodes: vec![m.clone()],
            edges: vec![
                edge(
                    EdgeKind::DocHead,
                    (doc.document_id.clone(), nid("ROOT")),
                    (doc.document_id.clone(), m.node_id.clone()),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::DocEnd,
                    (doc.document_id.clone(), m.node_id.clone()),
                    (doc.document_id.clone(), nid("END")),
                    None,
                    false,
                ),
            ],
        },
    )
    .await;
    let community = store
        .declare_community(
            &[(doc.document_id.clone(), nid("member1"))],
            &DeclareCommunityOptions {
                summary: "s".to_string(),
                wiki_id: w.clone(),
            },
        )
        .await
        .unwrap();

    // Member change → community STALE.
    let m2 = content_node(&doc.document_id, "member1", "v2");
    apply_graph(
        &store,
        &doc,
        Graph {
            nodes: vec![m2.clone()],
            edges: vec![
                edge(
                    EdgeKind::DocHead,
                    (doc.document_id.clone(), nid("ROOT")),
                    (doc.document_id.clone(), m2.node_id.clone()),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::DocEnd,
                    (doc.document_id.clone(), m2.node_id.clone()),
                    (doc.document_id.clone(), nid("END")),
                    None,
                    false,
                ),
            ],
        },
    )
    .await;

    let err = store.publish_document(&doc.document_id).await.unwrap_err();
    assert_eq!(
        err,
        StoreError::UnresolvedReference,
        "a STALE (underived) community's member document must not publish"
    );

    store
        .re_derive_community(&community.community_id)
        .await
        .unwrap();
    let published = store.publish_document(&doc.document_id).await.unwrap();
    assert_eq!(
        published.state,
        DocState::Published,
        "publishing the member document succeeds once the community is re-derived"
    );
}

/// Fail-state §4.4.3: a `BROKEN` link is never resolved by re-syncing an unrelated
/// embed — publish still fails with `UnresolvedReference`. RED — `re_sync_embed`
/// stub.
#[tokio::test]
async fn broken_link_blocks_publish_even_after_resync_of_unrelated_embed() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let src = new_doc(&store, &w, "source").await;
    let fact_loc = seed_fact(&store, &w, &src, "license", "MIT").await;

    let d = new_doc(&store, &w, "doc").await;
    let e = ref_node(&d.document_id, "em", fact_loc.clone(), Some("MTI"));
    let broken = ref_node(&d.document_id, "bl", (did("ghost-tgt"), nid("gone")), None);
    apply_graph(
        &store,
        &d,
        Graph {
            nodes: vec![
                content_node(&d.document_id, "n", "x"),
                e.clone(),
                broken.clone(),
            ],
            edges: vec![
                edge(
                    EdgeKind::DocHead,
                    (d.document_id.clone(), nid("ROOT")),
                    (d.document_id.clone(), nid("n")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::DocEnd,
                    (d.document_id.clone(), nid("n")),
                    (d.document_id.clone(), nid("END")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::Embed,
                    (d.document_id.clone(), e.node_id.clone()),
                    fact_loc.clone(),
                    Some(ReferenceState::Stale),
                    false,
                ),
                edge(
                    EdgeKind::Link,
                    (d.document_id.clone(), broken.node_id.clone()),
                    (did("ghost-tgt"), nid("gone")),
                    Some(ReferenceState::Broken),
                    false,
                ),
            ],
        },
    )
    .await;

    // Re-sync the (unrelated) STALE embed — the BROKEN link must NOT be resolved
    // by this.
    store
        .re_sync_embed(&d.document_id, &nid("em"))
        .await
        .unwrap();

    let err = store.publish_document(&d.document_id).await.unwrap_err();
    assert_eq!(
        err,
        StoreError::UnresolvedReference,
        "a BROKEN link still blocks publish after an unrelated embed re-sync"
    );
}

// ---------------------------------------------------------------------------
// Concurrency contract (SHARDED-RWLOCK-STORE / ARC-SHARED-ENGINE)
// ---------------------------------------------------------------------------

/// Concurrency: N tasks concurrently re-sync the same `STALE` embed behind a
/// `Barrier`. The re-sync must be atomic under the referencing document's shard
/// write lock — every task succeeds, the snapshot is deterministic (canonical),
/// and a subsequent report read sees the embed `FRESH` (never a torn half-synced
/// state). RED — `re_sync_embed`/`get_consistency_report` stubs.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_resync_of_same_embed_is_atomic() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let src = new_doc(&store, &w, "source").await;
    let fact_loc = seed_fact(&store, &w, &src, "license", "MIT").await;

    let d = new_doc(&store, &w, "doc").await;
    let e = ref_node(&d.document_id, "em", fact_loc.clone(), Some("MTI"));
    apply_graph(
        &store,
        &d,
        Graph {
            nodes: vec![content_node(&d.document_id, "n", "x"), e.clone()],
            edges: vec![
                edge(
                    EdgeKind::DocHead,
                    (d.document_id.clone(), nid("ROOT")),
                    (d.document_id.clone(), nid("n")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::DocEnd,
                    (d.document_id.clone(), nid("n")),
                    (d.document_id.clone(), nid("END")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::Embed,
                    (d.document_id.clone(), e.node_id.clone()),
                    fact_loc.clone(),
                    Some(ReferenceState::Stale),
                    false,
                ),
            ],
        },
    )
    .await;
    let doc_id = Arc::new(d.document_id.clone());

    const N: usize = 8;
    let barrier = Arc::new(tokio::sync::Barrier::new(N));
    let mut handles = Vec::new();
    for _ in 0..N {
        let s = Arc::clone(&store);
        let id = Arc::clone(&doc_id);
        let b = Arc::clone(&barrier);
        handles.push(tokio::spawn(async move {
            b.wait().await;
            s.re_sync_embed(id.as_ref(), &nid("em")).await
        }));
    }
    // Every contender's re-sync must succeed.
    for h in handles {
        h.await
            .expect("concurrent re-sync task did not panic")
            .unwrap();
    }
    // Read the report only after all 8 re-syncs have completed, so the observed
    // state is the post-re-sync FRESH snapshot (not a pre-re-sync STALE race).
    let synced = store.get_consistency_report(&w).await.unwrap();
    assert_eq!(
        find_row(&synced, "em", EdgeKind::Embed).state,
        ReferenceState::Fresh,
        "the embed is deterministically FRESH after contended re-syncs"
    );
    let final_doc = store.get_document(&d.document_id).await.unwrap();
    let node = final_doc
        .graph
        .nodes
        .iter()
        .find(|n| n.node_id == nid("em"))
        .unwrap();
    assert_eq!(
        node.value.as_deref(),
        Some("MIT"),
        "the snapshot is deterministically the canonical value after contention"
    );
}

// ---------------------------------------------------------------------------
// Adversarial regression set (§4.4 correctness / concurrency) — RED-first where
// the current impl is incoherent; honest guards where it already behaves.
// ---------------------------------------------------------------------------

/// F3 — re-sync coherent with the **canonical value at commit**: after the target
/// fact CHANGES to a new value, `re_sync_embed` must carry the NEW canonical value
/// and the embed be `FRESH` — never an old snapshot stamped fresh.
///
/// RED-check note: the current `re_sync_embed` reads `fact_store` fresh at call
/// time, so it already picks up the new value → this test is GREEN today (kept as
/// a guard). The harder concurrent variant — a fact update landing *between* the
/// re-sync's snapshot-read and its commit — is NOT deterministically forceable
/// from the API surface, so ordering is pinned by the F4 repoint test instead.
#[tokio::test]
async fn resync_embed_after_fact_change_carries_new_canonical() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let src = new_doc(&store, &w, "source").await;
    let fact_loc = seed_fact(&store, &w, &src, "f", "A").await;

    let d = new_doc(&store, &w, "doc").await;
    let e = ref_node(&d.document_id, "em", fact_loc.clone(), Some("A"));
    apply_graph(
        &store,
        &d,
        Graph {
            nodes: vec![content_node(&d.document_id, "n", "x"), e.clone()],
            edges: vec![
                edge(
                    EdgeKind::DocHead,
                    (d.document_id.clone(), nid("ROOT")),
                    (d.document_id.clone(), nid("n")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::DocEnd,
                    (d.document_id.clone(), nid("n")),
                    (d.document_id.clone(), nid("END")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::Embed,
                    (d.document_id.clone(), e.node_id.clone()),
                    fact_loc.clone(),
                    Some(ReferenceState::Fresh),
                    false,
                ),
            ],
        },
    )
    .await;

    // Change the fact → the embed is STALE with the (now-OLD) "A" snapshot.
    store
        .update_fact(
            &w,
            "f",
            &gnosis::UpdateFactRequest {
                value: "B".to_string(),
                citations: vec![(src.document_id.clone(), nid("cite"))],
            },
        )
        .await
        .unwrap();
    let before = store.get_document(&d.document_id).await.unwrap().revision;

    let updated = store
        .re_sync_embed(&d.document_id, &nid("em"))
        .await
        .unwrap();
    let node = updated
        .graph
        .nodes
        .iter()
        .find(|n| n.node_id == nid("em"))
        .unwrap();
    assert_eq!(
        node.value.as_deref(),
        Some("B"),
        "F3: re-sync must carry the NEW canonical value, not an old snapshot"
    );
    assert_eq!(updated.revision, before + 1, "F3: re-sync bumps revision");
    let report = store.get_consistency_report(&w).await.unwrap();
    assert_eq!(
        find_row(&report, "em", EdgeKind::Embed).state,
        ReferenceState::Fresh,
        "F3: a re-synced embed reflecting canonical is FRESH"
    );
}

/// F4 — reference **target-change** is propagated: `update_document` repointing a
/// `link`'s target must re-stamp the edge from the NEW target's liveness (a
/// repoint to a missing target → `BROKEN`; a repoint back to a live node →
/// `RESOLVED`), NOT store the caller's `state` verbatim.
///
/// RED today: `update_document` stores a repointed edge's caller `state`
/// unchanged, and the report's `default_reference_state` fabricates `RESOLVED`
/// for a bare `link` regardless of whether its (new) target exists — so the first
/// assertion (repoint → `BROKEN`) fails.
#[tokio::test]
async fn repointing_link_restamps_state_from_target_liveness() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;

    // A live target doc with a real node `n1`.
    let tgt = new_doc(&store, &w, "target").await;
    apply_graph(
        &store,
        &tgt,
        Graph {
            nodes: vec![content_node(&tgt.document_id, "n1", "v")],
            edges: vec![
                edge(
                    EdgeKind::DocHead,
                    (tgt.document_id.clone(), nid("ROOT")),
                    (tgt.document_id.clone(), nid("n1")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::DocEnd,
                    (tgt.document_id.clone(), nid("n1")),
                    (tgt.document_id.clone(), nid("END")),
                    None,
                    false,
                ),
            ],
        },
    )
    .await;

    // A referrer doc with a live link into `tgt.n1`.
    let d = new_doc(&store, &w, "referrer").await;
    let l = ref_node(
        &d.document_id,
        "l",
        (tgt.document_id.clone(), nid("n1")),
        None,
    );
    let graph = |target: (DocumentId, NodeId)| Graph {
        nodes: vec![content_node(&d.document_id, "n", "x"), l.clone()],
        edges: vec![
            edge(
                EdgeKind::DocHead,
                (d.document_id.clone(), nid("ROOT")),
                (d.document_id.clone(), nid("n")),
                None,
                false,
            ),
            edge(
                EdgeKind::DocEnd,
                (d.document_id.clone(), nid("n")),
                (d.document_id.clone(), nid("END")),
                None,
                false,
            ),
            edge(
                EdgeKind::Link,
                (d.document_id.clone(), l.node_id.clone()),
                target,
                None,
                false,
            ),
        ],
    };
    apply_graph(&store, &d, graph((tgt.document_id.clone(), nid("n1")))).await;
    let report0 = store.get_consistency_report(&w).await.unwrap();
    assert_eq!(
        find_row(&report0, "l", EdgeKind::Link).state,
        ReferenceState::Resolved,
        "setup: a link to a live target is RESOLVED"
    );

    // REPOINT the link to a nonexistent target, caller sets NO state on the edge
    // → the report must DERIVE `BROKEN` from the missing target's liveness.
    apply_graph(&store, &d, graph((did("ghost-target"), nid("gone")))).await;
    let report1 = store.get_consistency_report(&w).await.unwrap();
    assert_eq!(
        find_row(&report1, "l", EdgeKind::Link).state,
        ReferenceState::Broken,
        "F4: repointing a link to a missing target derives BROKEN (not caller-verbatim)"
    );

    // REPOINT back to the live node → derives `RESOLVED` again.
    apply_graph(&store, &d, graph((tgt.document_id.clone(), nid("n1")))).await;
    let report2 = store.get_consistency_report(&w).await.unwrap();
    assert_eq!(
        find_row(&report2, "l", EdgeKind::Link).state,
        ReferenceState::Resolved,
        "F4: repointing back to a live target derives RESOLVED"
    );
}

/// F5a — None-state reporting derives truth from **target liveness**: a reference
/// edge stored with `state: None` pointing at a missing/archived target must be
/// reported per §4.4.2 (`BROKEN` for a link to a missing target, `STALE` for an
/// embed of an archived target) — never a fabricated `RESOLVED`/`FRESH`.
///
/// RED today: `default_reference_state` fabricates `RESOLVED` (link) / `FRESH`
/// (embed) without checking whether the target exists / is live.
#[tokio::test]
async fn none_state_reference_edges_derive_from_target_liveness() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;

    // A target doc with one node, to be archived for the embed half.
    let tgt = new_doc(&store, &w, "target").await;
    apply_graph(
        &store,
        &tgt,
        Graph {
            nodes: vec![content_node(&tgt.document_id, "n2", "second")],
            edges: vec![
                edge(
                    EdgeKind::DocHead,
                    (tgt.document_id.clone(), nid("ROOT")),
                    (tgt.document_id.clone(), nid("n2")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::DocEnd,
                    (tgt.document_id.clone(), nid("n2")),
                    (tgt.document_id.clone(), nid("END")),
                    None,
                    false,
                ),
            ],
        },
    )
    .await;
    store.archive_document(&tgt.document_id).await.unwrap();

    // A referrer doc holding two None-state edges: a link to a never-existing
    // target, and an embed snapshoting the (now-archived) target.
    let d = new_doc(&store, &w, "referrer").await;
    let link_ref = ref_node(
        &d.document_id,
        "gone-link",
        (did("ghost-target"), nid("missing")),
        None,
    );
    let embed_ref = ref_node(
        &d.document_id,
        "gone-embed",
        (tgt.document_id.clone(), nid("n2")),
        Some("second"),
    );
    apply_graph(
        &store,
        &d,
        Graph {
            nodes: vec![
                content_node(&d.document_id, "n", "x"),
                link_ref.clone(),
                embed_ref.clone(),
            ],
            edges: vec![
                edge(
                    EdgeKind::DocHead,
                    (d.document_id.clone(), nid("ROOT")),
                    (d.document_id.clone(), nid("n")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::DocEnd,
                    (d.document_id.clone(), nid("n")),
                    (d.document_id.clone(), nid("END")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::Link,
                    (d.document_id.clone(), link_ref.node_id.clone()),
                    (did("ghost-target"), nid("missing")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::Embed,
                    (d.document_id.clone(), embed_ref.node_id.clone()),
                    (tgt.document_id.clone(), nid("n2")),
                    None,
                    false,
                ),
            ],
        },
    )
    .await;

    let report = store.get_consistency_report(&w).await.unwrap();
    assert_eq!(
        find_row(&report, "gone-link", EdgeKind::Link).state,
        ReferenceState::Broken,
        "F5a: a None-state link to a missing target is BROKEN, not RESOLVED"
    );
    assert_eq!(
        find_row(&report, "gone-embed", EdgeKind::Embed).state,
        ReferenceState::Stale,
        "F5a: a None-state embed of an archived target is STALE, not FRESH"
    );
}

/// F5b — a `crosslink` whose source reference node carries NO copied snapshot
/// (`value: None`) is a **live reference** to a present target (§4.4.2 → a link),
/// so it must be reported `RESOLVED` — not `FRESH`.
///
/// RED today: `default_reference_state` returns `FRESH` for every Crosslink edge
/// unconditionally, even a live link that snapshots nothing.
#[tokio::test]
async fn crosslink_live_link_with_none_state_is_resolved() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let w2 = new_wiki(&store, "w2").await;

    // Fact lives in w; a crosslink in w2 points at it as a *live* link (no snapshot).
    let src = new_doc(&store, &w, "source").await;
    let fact_loc = seed_fact(&store, &w, &src, "license", "MIT").await;

    let d2 = new_doc(&store, &w2, "cross-referrer").await;
    let x = ref_node(&d2.document_id, "xl", fact_loc.clone(), None);
    apply_graph(
        &store,
        &d2,
        Graph {
            nodes: vec![x.clone()],
            edges: vec![
                edge(
                    EdgeKind::DocHead,
                    (d2.document_id.clone(), nid("ROOT")),
                    (d2.document_id.clone(), x.node_id.clone()),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::DocEnd,
                    (d2.document_id.clone(), x.node_id.clone()),
                    (d2.document_id.clone(), nid("END")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::Crosslink,
                    (d2.document_id.clone(), x.node_id.clone()),
                    fact_loc.clone(),
                    None,
                    true,
                ),
            ],
        },
    )
    .await;

    let report = store.get_consistency_report(&w2).await.unwrap();
    assert_eq!(
        find_row(&report, "xl", EdgeKind::Crosslink).state,
        ReferenceState::Resolved,
        "F5b: a live crosslink (value None) to a present target is RESOLVED, not FRESH"
    );
}

/// F2 — a deterministic proxy for propagation coverage: an embed that snapshots
/// the **OLD** fact value, created in a document AFTER the fact-update
/// propagation scan, must still be reported `STALE` (§4.4.3 — its snapshot
/// differs from the new canonical value), NOT `FRESH`.
///
/// Sequence: (a) seed fact = "A"; (b) doc A embeds it (snapshot "A"); (c)
/// `update_fact` → "B" marks A's embed STALE (scan only saw docs present then);
/// (d) create doc B (a DIFFERENT document) with an embed whose snapshot is the
/// old "A"; (e) `get_consistency_report` must report B's embed STALE.
///
/// RED today: `update_document` stores B's edge verbatim (`state: None`) and the
/// report's `default_reference_state` fabricates `FRESH` — no fresh-vs-stale
/// snapshot derivation catches B. (A naive propagation that only scans docs
/// present at update time misses B, so it stays stale-but-reported-fresh.)
#[tokio::test]
async fn propagation_covers_embeds_created_after_fact_update() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let src = new_doc(&store, &w, "source").await;
    let fact_loc = seed_fact(&store, &w, &src, "f", "A").await;

    // (b) doc A embeds the fact with a FRESH snapshot "A".
    let a = new_doc(&store, &w, "doc-a").await;
    let ea = ref_node(&a.document_id, "em", fact_loc.clone(), Some("A"));
    apply_graph(
        &store,
        &a,
        Graph {
            nodes: vec![content_node(&a.document_id, "n", "x"), ea.clone()],
            edges: vec![
                edge(
                    EdgeKind::DocHead,
                    (a.document_id.clone(), nid("ROOT")),
                    (a.document_id.clone(), nid("n")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::DocEnd,
                    (a.document_id.clone(), nid("n")),
                    (a.document_id.clone(), nid("END")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::Embed,
                    (a.document_id.clone(), ea.node_id.clone()),
                    fact_loc.clone(),
                    Some(ReferenceState::Fresh),
                    false,
                ),
            ],
        },
    )
    .await;

    // (c) change the fact → "B"; propagation marks doc A's embed STALE.
    store
        .update_fact(
            &w,
            "f",
            &gnosis::UpdateFactRequest {
                value: "B".to_string(),
                citations: vec![(src.document_id.clone(), nid("cite"))],
            },
        )
        .await
        .unwrap();
    {
        let report = store.get_consistency_report(&w).await.unwrap();
        assert_eq!(
            find_row(&report, "em", EdgeKind::Embed).state,
            ReferenceState::Stale,
            "setup: doc A's embed is STALE after the fact update"
        );
    }

    // (d) doc B — created AFTER the propagation scan — embeds the OLD "A".
    let b = new_doc(&store, &w, "doc-b").await;
    let eb = ref_node(&b.document_id, "em", fact_loc.clone(), Some("A"));
    apply_graph(
        &store,
        &b,
        Graph {
            nodes: vec![content_node(&b.document_id, "n", "x"), eb.clone()],
            edges: vec![
                edge(
                    EdgeKind::DocHead,
                    (b.document_id.clone(), nid("ROOT")),
                    (b.document_id.clone(), nid("n")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::DocEnd,
                    (b.document_id.clone(), nid("n")),
                    (b.document_id.clone(), nid("END")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::Embed,
                    (b.document_id.clone(), eb.node_id.clone()),
                    fact_loc.clone(),
                    None,
                    false,
                ),
            ],
        },
    )
    .await;

    // (e) B's embed snapshots "A" ≠ canonical "B" → must be STALE.
    let report = store.get_consistency_report(&w).await.unwrap();
    let row_b = report
        .iter()
        .find(|r| {
            r.document_id == b.document_id && r.kind == EdgeKind::Embed && r.node_id == nid("em")
        })
        .expect("doc B's embed appears in the report");
    assert_eq!(
        row_b.state,
        ReferenceState::Stale,
        "F2: an embed created after the propagation scan with a stale snapshot is STALE (§4.4.3)"
    );
}

/// F7 — propagation bumps the revision of every referencing document: after
/// `update_fact` propagates STALE to doc A's embed, A's `revision` must increase
/// so the mutation is observable to the journal/epoch and to
/// optimistic-concurrency reads.
///
/// RED today: `propagate_fact_staleness` inserts an updated `Document` with
/// `..cur.clone()` (preserving the original `revision`), so referencing docs'
/// revisions are NOT bumped.
#[tokio::test]
async fn fact_propagation_bumps_referencing_document_revision() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let src = new_doc(&store, &w, "source").await;
    let fact_loc = seed_fact(&store, &w, &src, "f", "A").await;

    let a = new_doc(&store, &w, "doc-a").await;
    let e = ref_node(&a.document_id, "em", fact_loc.clone(), Some("A"));
    apply_graph(
        &store,
        &a,
        Graph {
            nodes: vec![content_node(&a.document_id, "n", "x"), e.clone()],
            edges: vec![
                edge(
                    EdgeKind::DocHead,
                    (a.document_id.clone(), nid("ROOT")),
                    (a.document_id.clone(), nid("n")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::DocEnd,
                    (a.document_id.clone(), nid("n")),
                    (a.document_id.clone(), nid("END")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::Embed,
                    (a.document_id.clone(), e.node_id.clone()),
                    fact_loc.clone(),
                    Some(ReferenceState::Fresh),
                    false,
                ),
            ],
        },
    )
    .await;

    let rev_before = store.get_document(&a.document_id).await.unwrap().revision;

    // §4.4.3 fact update → propagation marks A's embed STALE.
    store
        .update_fact(
            &w,
            "f",
            &gnosis::UpdateFactRequest {
                value: "B".to_string(),
                citations: vec![(src.document_id.clone(), nid("cite"))],
            },
        )
        .await
        .unwrap();

    let rev_after = store.get_document(&a.document_id).await.unwrap().revision;
    {
        let report = store.get_consistency_report(&w).await.unwrap();
        assert_eq!(
            find_row(&report, "em", EdgeKind::Embed).state,
            ReferenceState::Stale,
            "setup: propagation marked A's embed STALE"
        );
    }
    assert!(
        rev_after > rev_before,
        "F7: propagation must bump the referencing document's revision ({} -> {})",
        rev_before,
        rev_after
    );
}

/// F1 — deadlock/correctness stress: a `multi_thread` runtime concurrently runs
/// `publish_document` on docs whose reference targets/facts live in the same shard
/// space as `update_fact` / `re_sync_embed`, all gated behind a `Barrier`, for
/// several iterations. The test asserts COMPLETION (no deadlock) and that every
/// handle resolves without panic.
///
/// RED-check note: with the current consistent locking (integrity `read` lock →
/// one shard write at a time, never two shard writes held), this test completes —
/// it is a **deadlock-regression guard** (green today, must STAY green). A
/// deadlock would hang rather than fail fast, so N iterations + a completion
/// assertion are used; run with a modest task count so it completes quickly once
/// fixed.
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn publish_fact_commit_resync_stress_completes() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let src = new_doc(&store, &w, "source").await;
    let fact_loc = seed_fact(&store, &w, &src, "f", "A").await;

    // A set of draft docs that all reference the fact in the same shard space.
    let mut drafts = Vec::new();
    for i in 0..5 {
        let d = new_doc(&store, &w, &format!("draft-{i}")).await;
        let e = ref_node(&d.document_id, "em", fact_loc.clone(), Some("A"));
        apply_graph(
            &store,
            &d,
            Graph {
                nodes: vec![content_node(&d.document_id, "n", "x"), e.clone()],
                edges: vec![
                    edge(
                        EdgeKind::DocHead,
                        (d.document_id.clone(), nid("ROOT")),
                        (d.document_id.clone(), nid("n")),
                        None,
                        false,
                    ),
                    edge(
                        EdgeKind::DocEnd,
                        (d.document_id.clone(), nid("n")),
                        (d.document_id.clone(), nid("END")),
                        None,
                        false,
                    ),
                    edge(
                        EdgeKind::Embed,
                        (d.document_id.clone(), e.node_id.clone()),
                        fact_loc.clone(),
                        Some(ReferenceState::Fresh),
                        false,
                    ),
                ],
            },
        )
        .await;
        drafts.push(d.document_id);
    }

    const ITERATIONS: usize = 8;
    const TASKS: usize = 6;
    for it in 0..ITERATIONS {
        let barrier = Arc::new(tokio::sync::Barrier::new(TASKS));
        let value = format!("val-{it}");
        let mut handles = Vec::new();
        for t in 0..TASKS {
            let s = Arc::clone(&store);
            let w = w.clone();
            let src_id = src.document_id.clone();
            let drafts = drafts.clone();
            let b = Arc::clone(&barrier);
            let val = value.clone();
            handles.push(tokio::spawn(async move {
                b.wait().await;
                match t {
                    0 => {
                        // Fact commit → propagation over the referencing shard space.
                        s.update_fact(
                            &w,
                            "f",
                            &gnosis::UpdateFactRequest {
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
                        // Publish the last draft. The embed may be freshly-consistent
                        // (FRESH → publish succeeds) or STALE (→ UnresolvedReference);
                        // re-publishing across iterations is InvalidState (already
                        // PUBLISHED). All three are coherent, non-deadlock, non-panic
                        // outcomes — only a deadlock/panic is the regression signal.
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
        for h in handles {
            // Every handle must RESOLVE — a deadlock hangs this line.
            h.await
                .expect("F1 stress: a concurrent publish/fact/resync task panicked")
                .unwrap();
        }
    }
    // The loop completing is the deadlock-regression signal.
}

/// Concurrency: a writer updates a fact while reader tasks read the consistency
/// report; the propagation atomicity holds — a reader never observes a torn state
/// (each report read returns a well-formed, coherent snapshot). RED — the
/// propagation and `get_consistency_report` stubs.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_fact_update_with_report_reads_are_coherent() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let src = new_doc(&store, &w, "source").await;
    let fact_loc = seed_fact(&store, &w, &src, "license", "MIT").await;

    let d = new_doc(&store, &w, "doc").await;
    let e = ref_node(&d.document_id, "em", fact_loc.clone(), Some("MIT"));
    apply_graph(
        &store,
        &d,
        Graph {
            nodes: vec![content_node(&d.document_id, "n", "x"), e.clone()],
            edges: vec![
                edge(
                    EdgeKind::DocHead,
                    (d.document_id.clone(), nid("ROOT")),
                    (d.document_id.clone(), nid("n")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::DocEnd,
                    (d.document_id.clone(), nid("n")),
                    (d.document_id.clone(), nid("END")),
                    None,
                    false,
                ),
                edge(
                    EdgeKind::Embed,
                    (d.document_id.clone(), e.node_id.clone()),
                    fact_loc.clone(),
                    Some(ReferenceState::Fresh),
                    false,
                ),
            ],
        },
    )
    .await;
    let doc_id = Arc::new(src.document_id.clone());

    const N: usize = 8;
    let barrier = Arc::new(tokio::sync::Barrier::new(N));
    let mut handles: Vec<tokio::task::JoinHandle<Result<(), StoreError>>> = Vec::new();
    for i in 0..N {
        let s = Arc::clone(&store);
        let w = w.clone();
        let id = Arc::clone(&doc_id);
        let b = Arc::clone(&barrier);
        handles.push(tokio::spawn(async move {
            b.wait().await;
            if i == 0 {
                // The single writer: change the fact value → propagation.
                s.update_fact(
                    &w,
                    "license",
                    &gnosis::UpdateFactRequest {
                        value: "Apache-2.0".to_string(),
                        citations: vec![(id.as_ref().clone(), nid("cite"))],
                    },
                )
                .await?;
            } else {
                // Readers: every report read must be coherent.
                let report = s.get_consistency_report(&w).await?;
                let row = report
                    .iter()
                    .find(|r| r.node_id == nid("em") && r.kind == EdgeKind::Embed)
                    .expect("the embed reference is always present in a coherent report");
                // Coherent: the embed is either the pre-update FRESH or the
                // post-update STALE — never any other (torn) state.
                assert!(
                    matches!(row.state, ReferenceState::Fresh | ReferenceState::Stale),
                    "report read never observes a torn embed state: {:?}",
                    row.state
                );
            }
            Ok(())
        }));
    }
    for h in handles {
        h.await
            .expect("concurrent reader/writer task did not panic")
            .unwrap();
    }
    // After the write + propagation, the embed is STALE.
    let report = store.get_consistency_report(&w).await.unwrap();
    assert_eq!(
        find_row(&report, "em", EdgeKind::Embed).state,
        ReferenceState::Stale,
        "the fact update marks the embed STALE (§4.4.3)"
    );
}
