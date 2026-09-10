//! §7.5 F4 — community retrieval — `getCommunityContext` integration tests.
//!
//! Written **RED-first** at the TDD gate from the F4 behavior contract alone
//! (`docs/specs/f4-community-context-spec.md` §5.1/§5.2/§5.3) plus the F4 PBT
//! register (`docs/specs/4-2-graph-property-register.md` F4 section). The
//! `get_community_context` accessor + `CommunityContext` type are **RED-stage
//! stubs** (the trait method + `Store` impl body are `unimplemented!()`), so
//! every assertion below fails at runtime until the Implementer lands the real
//! pre-joined read.
//!
//! ## State enumeration (from the contract)
//!
//! Happy states:
//! - H-1  declared community → matching community_id/wiki_id/summary/members, Fresh
//! - H-2  after `updateDocument` rewriting a member node → Stale (summary/members unchanged)
//! - H-3  after `re_derive_community` → Fresh (summary/members unchanged)
//! - H-4  after `update_community_summary` → new manual summary, state unchanged
//! - H-5  members spanning two documents → full cross-document set
//! - H-6  single-member community → members.len() == 1
//! - H-7  a member that is a fact location → present in members
//! - H-8  staleness never triggered → Fresh (the `unwrap_or(Fresh)` observable)
//! - H-9  determinism — two consecutive reads, no mutation → equal
//! - H-10 membership completeness — members exactly equals the declared set
//! - H-11 manual-override — after member-change + re-derive, summary still manual
//!
//! Fail states:
//! - F-1  unknown communityId → `CommunityNotFound`
//! - F-2  `WikiNotFound` cannot fire (communityId-keyed accessor derives the wiki
//!   from the community record)

use std::sync::Arc;

use gnosis::{
    CommunityId, CreateDocumentRequest, DeclareCommunityOptions, DocumentId, Edge, EdgeKind, Graph,
    Node, NodeId, NodeKind, RagStore, ReferenceState, Store, StoreError, UpdateDocumentRequest,
    WikiId,
};

// ---------------------------------------------------------------------------
// Helpers (replicated from `tests/graph_integration.rs` — the community seeding
// pattern: `declare_community`, `fresh_doc`, `content_only_graph`, etc.)
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

/// Apply a graph to `doc` from its **current** revision (keeps the §4.1.4
/// optimistic-concurrency guard happy).
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

/// A graph of `names` content nodes with a `doc-head`/`doc-end` edge set.
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

/// A graph with a single `fact` node `f1` (a fact location) + head/end edges.
fn fact_location_graph(doc: &DocumentId) -> Graph {
    let f1 = fact_node(doc, "f1", "k1", "v1");
    Graph {
        nodes: vec![f1.clone()],
        edges: vec![
            edge(
                EdgeKind::DocHead,
                (doc.clone(), nid("ROOT")),
                (doc.clone(), f1.node_id.clone()),
                None,
            ),
            edge(
                EdgeKind::DocEnd,
                (doc.clone(), f1.node_id.clone()),
                (doc.clone(), nid("END")),
                None,
            ),
        ],
    }
}

fn declare_opts(w: &WikiId, summary: &str) -> DeclareCommunityOptions {
    DeclareCommunityOptions {
        summary: summary.to_string(),
        wiki_id: w.clone(),
    }
}

/// A fresh store + wiki + a doc with `names` content nodes.
async fn fresh_doc(names: &[&str]) -> (Arc<Store>, WikiId, gnosis::Document) {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "doc").await;
    apply_graph(&store, &doc, content_only_graph(&doc.document_id, names)).await;
    (store, w, doc)
}

// ---------------------------------------------------------------------------
// H-1 — declared community → matching fields, Fresh
// ---------------------------------------------------------------------------
#[tokio::test]
async fn h1_declared_community_returns_matching_context_fresh() {
    let (store, w, doc) = fresh_doc(&["n1", "n2"]).await;
    let members = vec![
        (doc.document_id.clone(), nid("n1")),
        (doc.document_id.clone(), nid("n2")),
    ];
    let declared = store
        .declare_community(&members, &declare_opts(&w, "manual summary"))
        .await
        .unwrap();

    let ctx = store
        .get_community_context(&declared.community_id)
        .await
        .unwrap();
    assert_eq!(ctx.community_id, declared.community_id, "H-1 community_id");
    assert_eq!(ctx.wiki_id, w, "H-1 wiki_id");
    assert_eq!(ctx.summary, "manual summary", "H-1 summary verbatim");
    assert_eq!(ctx.members, members, "H-1 members complete, stored order");
    assert_eq!(ctx.state, gnosis::CommunityState::Fresh, "H-1 Fresh");
}

// ---------------------------------------------------------------------------
// H-2 — after `updateDocument` rewriting a member node → Stale
// ---------------------------------------------------------------------------
#[tokio::test]
async fn h2_update_document_rewriting_member_marks_stale() {
    let (store, w, doc) = fresh_doc(&["n1", "n2"]).await;
    let members = vec![
        (doc.document_id.clone(), nid("n1")),
        (doc.document_id.clone(), nid("n2")),
    ];
    let declared = store
        .declare_community(&members, &declare_opts(&w, "manual summary"))
        .await
        .unwrap();

    // Rewrite a member node (n1) via `updateDocument` — the crate-faithful
    // `mark_communities_stale` trigger (line 2527).
    apply_graph(
        &store,
        &doc,
        content_only_graph(&doc.document_id, &["n1", "n2"]),
    )
    .await;

    let ctx = store
        .get_community_context(&declared.community_id)
        .await
        .unwrap();
    assert_eq!(ctx.state, gnosis::CommunityState::Stale, "H-2 Stale");
    assert_eq!(
        ctx.summary, "manual summary",
        "H-2 summary unchanged (read-only)"
    );
    assert_eq!(ctx.members, members, "H-2 members unchanged (read-only)");
}

// ---------------------------------------------------------------------------
// H-3 — after `re_derive_community` → Fresh (summary/members unchanged)
// ---------------------------------------------------------------------------
#[tokio::test]
async fn h3_re_derive_clears_stale_keeps_summary_members() {
    let (store, w, doc) = fresh_doc(&["n1", "n2"]).await;
    let members = vec![
        (doc.document_id.clone(), nid("n1")),
        (doc.document_id.clone(), nid("n2")),
    ];
    let declared = store
        .declare_community(&members, &declare_opts(&w, "manual summary"))
        .await
        .unwrap();

    apply_graph(
        &store,
        &doc,
        content_only_graph(&doc.document_id, &["n1", "n2"]),
    )
    .await;
    let stale = store
        .get_community_context(&declared.community_id)
        .await
        .unwrap();
    assert_eq!(
        stale.state,
        gnosis::CommunityState::Stale,
        "precondition Stale"
    );

    store
        .re_derive_community(&declared.community_id)
        .await
        .unwrap();
    let ctx = store
        .get_community_context(&declared.community_id)
        .await
        .unwrap();
    assert_eq!(
        ctx.state,
        gnosis::CommunityState::Fresh,
        "H-3 Fresh after re-derive"
    );
    assert_eq!(ctx.summary, "manual summary", "H-3 summary unchanged");
    assert_eq!(ctx.members, members, "H-3 members unchanged");
}

// ---------------------------------------------------------------------------
// H-4 — after `update_community_summary` → new manual summary, state unchanged
// ---------------------------------------------------------------------------
#[tokio::test]
async fn h4_update_community_summary_reflects_new_manual_summary() {
    let (store, w, doc) = fresh_doc(&["n1"]).await;
    let members = vec![(doc.document_id.clone(), nid("n1"))];
    let declared = store
        .declare_community(&members, &declare_opts(&w, "old summary"))
        .await
        .unwrap();

    let before = store
        .get_community_context(&declared.community_id)
        .await
        .unwrap();
    assert_eq!(
        before.state,
        gnosis::CommunityState::Fresh,
        "precondition Fresh"
    );

    store
        .update_community_summary(&declared.community_id, "new manual summary")
        .await
        .unwrap();
    let ctx = store
        .get_community_context(&declared.community_id)
        .await
        .unwrap();
    assert_eq!(
        ctx.summary, "new manual summary",
        "H-4 new manual summary (§4.2.8.4)"
    );
    assert_eq!(ctx.state, before.state, "H-4 state unchanged");
    assert_eq!(ctx.members, members, "H-4 members unchanged");
}

// ---------------------------------------------------------------------------
// H-5 — members spanning two documents → full cross-document set
// ---------------------------------------------------------------------------
#[tokio::test]
async fn h5_members_spanning_two_documents() {
    let (store, w, doc_a) = fresh_doc(&["n1", "n2"]).await;
    let doc_b = new_doc(&store, &w, "doc-b").await;
    apply_graph(
        &store,
        &doc_b,
        content_only_graph(&doc_b.document_id, &["m1"]),
    )
    .await;

    let members = vec![
        (doc_a.document_id.clone(), nid("n1")),
        (doc_b.document_id.clone(), nid("m1")),
    ];
    let declared = store
        .declare_community(&members, &declare_opts(&w, "cross-doc"))
        .await
        .unwrap();

    let ctx = store
        .get_community_context(&declared.community_id)
        .await
        .unwrap();
    assert_eq!(ctx.members, members, "H-5 full cross-document member set");
    assert_eq!(ctx.members.len(), 2, "H-5 both members present");
}

// ---------------------------------------------------------------------------
// H-6 — single-member community → members.len() == 1
// ---------------------------------------------------------------------------
#[tokio::test]
async fn h6_single_member_community() {
    let (store, w, doc) = fresh_doc(&["n1"]).await;
    let members = vec![(doc.document_id.clone(), nid("n1"))];
    let declared = store
        .declare_community(&members, &declare_opts(&w, "solo"))
        .await
        .unwrap();

    let ctx = store
        .get_community_context(&declared.community_id)
        .await
        .unwrap();
    assert_eq!(ctx.members.len(), 1, "H-6 single member");
    assert_eq!(ctx.members, members, "H-6 the single member");
}

// ---------------------------------------------------------------------------
// H-7 — a member that is a fact location → present in members
// ---------------------------------------------------------------------------
#[tokio::test]
async fn h7_member_that_is_a_fact_location() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "facts").await;
    apply_graph(&store, &doc, fact_location_graph(&doc.document_id)).await;

    // The member is the fact node's location `(doc, f1)` — membership is by
    // declared id, not node kind.
    let members = vec![(doc.document_id.clone(), nid("f1"))];
    let declared = store
        .declare_community(&members, &declare_opts(&w, "fact community"))
        .await
        .unwrap();

    let ctx = store
        .get_community_context(&declared.community_id)
        .await
        .unwrap();
    assert_eq!(ctx.members, members, "H-7 fact-location member present");
}

// ---------------------------------------------------------------------------
// H-8 — staleness never triggered → Fresh (the `unwrap_or(Fresh)` observable)
// ---------------------------------------------------------------------------
#[tokio::test]
async fn h8_never_triggered_reports_fresh() {
    let (store, w, doc) = fresh_doc(&["n1"]).await;
    let members = vec![(doc.document_id.clone(), nid("n1"))];
    let declared = store
        .declare_community(&members, &declare_opts(&w, "fresh"))
        .await
        .unwrap();

    // No member-touching operation is applied — staleness was never triggered.
    let ctx = store
        .get_community_context(&declared.community_id)
        .await
        .unwrap();
    assert_eq!(
        ctx.state,
        gnosis::CommunityState::Fresh,
        "H-8 Fresh default"
    );
}

// ---------------------------------------------------------------------------
// H-9 — determinism: two consecutive reads, no mutation → equal
// ---------------------------------------------------------------------------
#[tokio::test]
async fn h9_two_consecutive_reads_are_equal() {
    let (store, w, doc) = fresh_doc(&["n1", "n2"]).await;
    let members = vec![
        (doc.document_id.clone(), nid("n1")),
        (doc.document_id.clone(), nid("n2")),
    ];
    let declared = store
        .declare_community(&members, &declare_opts(&w, "deterministic"))
        .await
        .unwrap();

    let a = store
        .get_community_context(&declared.community_id)
        .await
        .unwrap();
    let b = store
        .get_community_context(&declared.community_id)
        .await
        .unwrap();
    assert_eq!(a, b, "H-9 field-for-field equal (determinism)");
}

// ---------------------------------------------------------------------------
// H-10 — membership completeness: members exactly equals the declared set
// ---------------------------------------------------------------------------
#[tokio::test]
async fn h10_membership_completeness() {
    let (store, w, doc) = fresh_doc(&["n1", "n2", "n3"]).await;
    let members = vec![
        (doc.document_id.clone(), nid("n1")),
        (doc.document_id.clone(), nid("n2")),
        (doc.document_id.clone(), nid("n3")),
    ];
    let declared = store
        .declare_community(&members, &declare_opts(&w, "complete"))
        .await
        .unwrap();

    let ctx = store
        .get_community_context(&declared.community_id)
        .await
        .unwrap();
    assert_eq!(
        ctx.members, members,
        "H-10 members exactly the declared set"
    );
    assert_eq!(
        ctx.members.len(),
        members.len(),
        "H-10 no additions/removals"
    );
}

// ---------------------------------------------------------------------------
// H-11 — manual-override: after member-change + re-derive, summary still manual
// ---------------------------------------------------------------------------
#[tokio::test]
async fn h11_manual_summary_survives_member_change_and_re_derive() {
    let (store, w, doc) = fresh_doc(&["n1", "n2"]).await;
    let members = vec![
        (doc.document_id.clone(), nid("n1")),
        (doc.document_id.clone(), nid("n2")),
    ];
    let declared = store
        .declare_community(&members, &declare_opts(&w, "manual summary"))
        .await
        .unwrap();

    // Member change (rewrite n1) → Stale.
    apply_graph(
        &store,
        &doc,
        content_only_graph(&doc.document_id, &["n1", "n2"]),
    )
    .await;
    // Re-derive → Fresh.
    store
        .re_derive_community(&declared.community_id)
        .await
        .unwrap();

    let ctx = store
        .get_community_context(&declared.community_id)
        .await
        .unwrap();
    assert_eq!(ctx.summary, "manual summary", "H-11 never auto-regenerated");
    assert_eq!(
        ctx.state,
        gnosis::CommunityState::Fresh,
        "H-11 Fresh after re-derive"
    );
}

// ---------------------------------------------------------------------------
// F-1 — unknown communityId → CommunityNotFound (the only fail-state)
// ---------------------------------------------------------------------------
#[tokio::test]
async fn f1_unknown_community_is_community_not_found() {
    let store = Arc::new(Store::new());
    let err = store
        .get_community_context(&CommunityId("ghost".into()))
        .await
        .unwrap_err();
    assert_eq!(
        err,
        StoreError::CommunityNotFound,
        "F-1 the only fail-state"
    );
}

// ---------------------------------------------------------------------------
// F-2 — `WikiNotFound` cannot fire (communityId-keyed accessor derives the wiki
// from the community record)
// ---------------------------------------------------------------------------
#[tokio::test]
async fn f2_wiki_not_found_cannot_fire() {
    let store = Arc::new(Store::new());
    // Declare a community whose `wiki_id` is a wiki that was NEVER created as a
    // wiki. `declare_community` does not validate the wiki exists, so the
    // community record carries a `wiki_id` that is otherwise unknown. The
    // accessor reads the wiki from the community record — it must succeed
    // (`Ok`), never `WikiNotFound`.
    let declared = store
        .declare_community(
            &[(did("d"), nid("n1"))],
            &declare_opts(&wiki("ghost-wiki"), "ghost community"),
        )
        .await
        .unwrap();

    let ctx = store
        .get_community_context(&declared.community_id)
        .await
        .unwrap();
    assert_eq!(
        ctx.wiki_id,
        wiki("ghost-wiki"),
        "F-2 wiki from the community record"
    );
    assert_eq!(
        ctx.community_id, declared.community_id,
        "F-2 Ok, not WikiNotFound"
    );
}

// ---------------------------------------------------------------------------
// NEG-4 — unknown communityId → CommunityNotFound (F-1), even with others present
// ---------------------------------------------------------------------------
#[tokio::test]
async fn neg4_unknown_community_not_found_with_others_present() {
    let (store, w, doc) = fresh_doc(&["n1"]).await;
    let declared = store
        .declare_community(
            &[(doc.document_id.clone(), nid("n1"))],
            &declare_opts(&w, "real community"),
        )
        .await
        .unwrap();

    // A community id that was never declared (the store has other communities).
    let ghost = CommunityId("comm-99999".into());
    assert_ne!(
        ghost, declared.community_id,
        "precondition: ghost is not the real id"
    );
    let err = store.get_community_context(&ghost).await.unwrap_err();
    assert_eq!(
        err,
        StoreError::CommunityNotFound,
        "NEG-4 unknown communityId → CommunityNotFound (F-1)"
    );
}

// ---------------------------------------------------------------------------
// NEG-5 — ghost wiki_id → Ok, never WikiNotFound (F-2)
// ---------------------------------------------------------------------------
#[tokio::test]
async fn neg5_ghost_wiki_never_wiki_not_found() {
    let store = Arc::new(Store::new());
    // A community whose `wiki_id` is a wiki that was NEVER created as a wiki.
    // `declare_community` does not validate the wiki exists, so the community
    // record carries a `wiki_id` that is otherwise unknown. The accessor reads
    // the wiki from the community record — it must succeed (`Ok`), never
    // `WikiNotFound`.
    let declared = store
        .declare_community(
            &[(did("d"), nid("n1"))],
            &declare_opts(&wiki("ghost-wiki"), "ghost community"),
        )
        .await
        .unwrap();

    let ctx = store
        .get_community_context(&declared.community_id)
        .await
        .unwrap();
    assert_eq!(
        ctx.wiki_id,
        wiki("ghost-wiki"),
        "NEG-5 wiki from the community record"
    );
    assert_eq!(
        ctx.community_id, declared.community_id,
        "NEG-5 Ok, not WikiNotFound"
    );
}
