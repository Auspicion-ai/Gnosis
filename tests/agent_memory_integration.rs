//! §4.5.4 — Agent-memory retrieval surface integration tests — written
//! **RED-first** at the TDD gate for the §4.5 unit.
//!
//! Every state and fail-state below is derived from the canonical behavior
//! contract alone (`docs/specs/gnosis.md` §4.5.4, §4.3.1, §4.3.2, and the
//! `listFacts`/`getFact` rows of §4.5.4) plus the concurrency contract
//! (IMMUTABLE-DERIVED-SNAPSHOT — the profile summary is a **derived document**
//! regenerated from the facts table, a projection, single source of truth).
//!
//! ## RED set (this suite)
//!
//! `get_profile_summary` is a COMPILING STUB on the store (`unimplemented!()`
//! → panics), so every `get_profile_summary` test below is RED. The
//! `get_fact`/`list_facts` regression guards are GREEN (they are the §4.3
//! surface).
//!
//! ## TestWriter contract decisions (for the Implementer to match exactly)
//!
//! - **`getProfileSummary` return shape** (§4.5.4) is pinned
//!   `{wikiId, summary, regeneratedAt, factCount}`. `factCount` = the number of
//!   facts in the wiki's facts table at regeneration time. `summary` is the
//!   derived document text (regenerated on fact change / on demand); its exact
//!   textual form is not pinned by the spec (it is a derived projection), so the
//!   suite asserts the *shape* and the *factCount*-driven behavior, not the prose.
//! - **`getProfileSummary` on an unknown wiki** → `WikiNotFound` (FS-2, §4.5.4).
//! - **Regeneration** is on fact change or on demand: adding a fact updates
//!   `factCount` and advances `regeneratedAt` (the derived doc never drifts from
//!   the facts table — §4.5.4 "no drift", single source of truth).

use std::sync::Arc;

use gnosis::{
    CreateDocumentRequest, Document, DocumentId, Edge, EdgeKind, Graph, Node, NodeId, NodeKind,
    ProfileSummary, RagStore, ReferenceState, Store, StoreError, UpdateDocumentRequest, WikiId,
};

// ---------------------------------------------------------------------------
// Helpers (mirroring the §4.2/§4.3/§4.4 suites)
// ---------------------------------------------------------------------------

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

/// Give `doc` a single content node named `name`.
async fn seed_content(store: &Store, doc: &Document, name: &str, value: &str) {
    let doc_id = doc.document_id.clone();
    let node = content_node(&doc_id, name, value);
    // A valid Provident graph needs exactly one doc-head + one doc-end edge.
    let graph = Graph {
        nodes: vec![node.clone()],
        edges: vec![
            edge(
                EdgeKind::DocHead,
                (doc_id.clone(), nid("ROOT")),
                (doc_id.clone(), node.node_id.clone()),
                None,
            ),
            edge(
                EdgeKind::DocEnd,
                (doc_id.clone(), node.node_id.clone()),
                (doc_id.clone(), nid("END")),
                None,
            ),
        ],
    };
    apply_graph(store, doc, graph).await;
}

// ---------------------------------------------------------------------------
// §4.5.4 — getProfileSummary (RED: the store stub panics)
// ---------------------------------------------------------------------------

/// State: a wiki with N facts has a profile summary derived from the facts
/// table: `{wikiId, summary, regeneratedAt, factCount}` with `factCount == N`.
#[tokio::test]
async fn get_profile_summary_returns_derived_doc_with_fact_count() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "profile-source").await;
    seed_content(&store, &doc, "n1", "canonical-content").await;

    store
        .create_fact(
            &w,
            &doc.document_id,
            "license",
            "MIT",
            &[(doc.document_id.clone(), nid("n1"))],
        )
        .await
        .unwrap();
    store
        .create_fact(
            &w,
            &doc.document_id,
            "owner",
            "Auspicion",
            &[(doc.document_id.clone(), nid("n1"))],
        )
        .await
        .unwrap();

    let summary = store
        .get_profile_summary(&w)
        .await
        .expect("profile summary");
    assert_eq!(summary.wiki_id, w);
    assert_eq!(summary.fact_count, 2);
    assert!(
        !summary.summary.is_empty(),
        "summary is the derived document"
    );
    assert!(
        !summary.regenerated_at.is_empty(),
        "regeneratedAt is ISO-8601 UTC"
    );
}

/// Fail-state (§4.5.4 / FS-2): `getProfileSummary` on an unknown wiki →
/// `WikiNotFound`.
#[tokio::test]
async fn get_profile_summary_unknown_wiki_is_wiki_not_found() {
    let store = Arc::new(Store::new());
    let err = store
        .get_profile_summary(&wiki("missing"))
        .await
        .unwrap_err();
    assert_eq!(err, StoreError::WikiNotFound);
}

/// State: the profile summary is regenerated when facts change — adding a fact
/// advances `factCount` (a projection that never drifts from the facts table).
#[tokio::test]
async fn get_profile_summary_regenerates_when_facts_change() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "src").await;
    seed_content(&store, &doc, "n1", "c").await;

    let before = store.get_profile_summary(&w).await.unwrap();
    assert_eq!(before.fact_count, 0);

    store
        .create_fact(
            &w,
            &doc.document_id,
            "k1",
            "v1",
            &[(doc.document_id.clone(), nid("n1"))],
        )
        .await
        .unwrap();
    let after = store.get_profile_summary(&w).await.unwrap();
    assert_eq!(after.fact_count, 1);
    // The derived doc refreshed (facts are the single source of truth; the
    // profile is a projection over them — §4.5.4 "no drift").
}

/// State: the profile is a **derived** document — a fact update is reflected on
/// the next regeneration (single source of truth, no drift). The facts table is
/// the owner; `getProfileSummary` is a projection, never a second source.
///
/// De-vacuated: the summary TEXT must surface the NEW value after `update_fact`
/// (not merely that `factCount` is unchanged at 1).
#[tokio::test]
async fn profile_surfaces_a_fact_value_change_into_the_summary() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "src").await;
    seed_content(&store, &doc, "n1", "c").await;
    store
        .create_fact(
            &w,
            &doc.document_id,
            "license",
            "MIT",
            &[(doc.document_id.clone(), nid("n1"))],
        )
        .await
        .unwrap();

    // Mutate the fact (single source of truth) and re-read the derived profile.
    store
        .update_fact(
            &w,
            "license",
            &gnosis::UpdateFactRequest {
                value: "AGPL-3.0".into(),
                citations: vec![(doc.document_id.clone(), nid("n1"))],
            },
        )
        .await
        .unwrap();
    let summary: ProfileSummary = store.get_profile_summary(&w).await.unwrap();
    assert_eq!(summary.fact_count, 1);
    // The derived projection MUST reflect the NEW canonical value (no drift).
    assert!(
        summary.summary.contains("AGPL-3.0"),
        "summary must surface the updated fact value, got: {:?}",
        summary.summary
    );
    assert!(
        !summary.summary.contains("MIT"),
        "summary must not drift back to the stale value, got: {:?}",
        summary.summary
    );
}

// ---------------------------------------------------------------------------
// §4.5.4 — facts table regression guards (GREEN: the §4.3 surface)
// ---------------------------------------------------------------------------

/// State (§4.5.4 "facts table"): an agent retrieves a fact by its **exact**
/// `factKey` — exact-reference, not embedding similarity ("similarity ≠
/// relevance"). GREEN via the §4.3 `get_fact`.
#[tokio::test]
async fn get_fact_returns_exact_fact_key_matches_only() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "src").await;
    seed_content(&store, &doc, "n1", "c").await;
    store
        .create_fact(
            &w,
            &doc.document_id,
            "license",
            "MIT",
            &[(doc.document_id.clone(), nid("n1"))],
        )
        .await
        .unwrap();

    // The exact key resolves; a merely-similar key does not (exact-reference).
    let got = store.get_fact(&w, "license").await.unwrap();
    assert_eq!(got.fact_key, "license");
    assert_eq!(got.value, "MIT");
    let miss = store.get_fact(&w, "licens").await.unwrap_err();
    assert!(matches!(miss, StoreError::DocumentNotFound));
}

/// State (§4.5.4 "facts table"): `listFacts` returns a paginated list of `Fact`
/// nodes `{items, total, page, pageSize}`. GREEN via the §4.3 `list_facts`.
#[tokio::test]
async fn list_facts_returns_paginated_shape() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "src").await;
    seed_content(&store, &doc, "n1", "c").await;
    for (i, (k, v)) in [("k1", "v1"), ("k2", "v2"), ("k3", "v3")]
        .into_iter()
        .enumerate()
    {
        let _ = i;
        store
            .create_fact(
                &w,
                &doc.document_id,
                k,
                v,
                &[(doc.document_id.clone(), nid("n1"))],
            )
            .await
            .unwrap();
    }
    let page = store
        .list_facts(
            &w,
            &gnosis::ListFactsFilter {
                state: None,
                page: Some(1),
                page_size: Some(3),
            },
        )
        .await
        .unwrap();
    assert_eq!(page.total, 3);
    assert_eq!(page.items.len(), 3);
    assert_eq!(page.page, 1);
    assert_eq!(page.page_size, 3);
}
