//! §4.1 Document-store integration tests — written **RED-first** at the TDD gate.
//!
//! Every state and fail-state below is derived from the canonical behavior
//! contract alone (`docs/specs/gnosis.md` §4.1.1–§4.1.5, §6) plus the
//! concurrency contract (`docs/research/gnosis-data-structures-concurrency-plan.md`
//! and `docs/decisions.md`: SHARDED-RWLOCK-STORE / WRITER-ACTOR-JOURNAL /
//! ARC-SHARED-ENGINE).
//!
//! The suite is **GREEN** against the implemented `Store` in `src/store`
//! (50/50 passing, verified with the trio). The adversarial findings that this
//! suite now pins as regressions: out-of-range pagination must not panic,
//! `crosslink` broken/stale states must block publish, reference state is
//! derived from target existence (not trusted from caller input), and the two
//! concurrency tests run on a multi-threaded runtime with a barrier so they
//! genuinely contend.

use std::sync::Arc;

use gnosis::{
    CreateDocumentRequest, DocState, DocumentId, Edge, EdgeKind, Graph, ListDocumentsFilter, Node,
    NodeId, NodeKind, RagStore, ReferenceState, Store, StoreError, UpdateDocumentRequest, WikiId,
};

// ---------------------------------------------------------------------------
// Test-graph helpers
// ---------------------------------------------------------------------------

fn wiki(id: &str) -> WikiId {
    WikiId(id.to_string())
}

fn doc_id(id: &str) -> DocumentId {
    DocumentId(id.to_string())
}

/// A minimal **valid** Provident graph: one content node, exactly one `doc-head`
/// (root → first node) and one `doc-end` (→ end sentinel) edge (§4.2.2).
fn valid_graph() -> Graph {
    let n1 = Node {
        document_id: doc_id("d1"),
        node_id: NodeId("n1".into()),
        kind: NodeKind::Content,
        value: Some("hello".into()),
        fact_key: None,
        target: None,
    };
    Graph {
        nodes: vec![n1.clone()],
        edges: vec![
            Edge {
                source: (doc_id("d1"), NodeId("ROOT".into())),
                target: (doc_id("d1"), n1.node_id.clone()),
                kind: EdgeKind::DocHead,
                state: None,
                cross_wiki: false,
                relation_type: None,
            },
            Edge {
                source: (doc_id("d1"), n1.node_id.clone()),
                target: (doc_id("d1"), NodeId("END".into())),
                kind: EdgeKind::DocEnd,
                state: None,
                cross_wiki: false,
                relation_type: None,
            },
        ],
    }
}

/// A Provident graph with a **BROKEN** `link` reference (a `reference` node with
/// a `link` edge whose target is missing — the §4.4.3 / FS-7 publish-gate input).
fn graph_with_broken_reference() -> Graph {
    let mut g = valid_graph();
    g.nodes.push(Node {
        document_id: doc_id("d1"),
        node_id: NodeId("ref1".into()),
        kind: NodeKind::Reference,
        value: None,
        fact_key: None,
        target: Some((doc_id("ghost"), NodeId("missing".into()))),
    });
    g.edges.push(Edge {
        source: (doc_id("d1"), NodeId("ref1".into())),
        target: (doc_id("ghost"), NodeId("missing".into())),
        kind: EdgeKind::Link,
        state: Some(ReferenceState::Broken),
        cross_wiki: false,
        relation_type: None,
    });
    g
}

/// A Provident graph with a **STALE** `embed` (FS-7 publish-gate input).
fn graph_with_stale_embed() -> Graph {
    let mut g = valid_graph();
    g.edges.push(Edge {
        source: (doc_id("d1"), NodeId("ref1".into())),
        target: (doc_id("fact-src"), NodeId("f1".into())),
        kind: EdgeKind::Embed,
        state: Some(ReferenceState::Stale),
        cross_wiki: false,
        relation_type: None,
    });
    g
}

/// A Provident graph with a **cross-wiki** reference (permitted in §4.1.2).
fn graph_with_crosslink() -> Graph {
    let mut g = valid_graph();
    g.edges.push(Edge {
        source: (doc_id("d1"), NodeId("ref1".into())),
        target: (doc_id("e2"), NodeId("n1".into())),
        kind: EdgeKind::Crosslink,
        state: Some(ReferenceState::Resolved),
        cross_wiki: true,
        relation_type: None,
    });
    g
}

/// A bare document handle used to point a `d1`'s edges at `d1`.
fn make_meta_request(title: &str) -> CreateDocumentRequest {
    CreateDocumentRequest {
        title: title.to_string(),
        tags: Some(vec!["tag-a".into()]),
        author: Some("tester".into()),
    }
}

fn update(title: &str, base_revision: u64) -> UpdateDocumentRequest {
    UpdateDocumentRequest {
        base_revision,
        graph: valid_graph(),
        title: Some(title.to_string()),
        tags: None,
    }
}

/// Create a wiki and return its id, so a subsequent doc op on it is
/// spec-correct. §4.1.3: `createDocument`/`listDocuments` on an **unknown**
/// `wikiId` → `WikiNotFound`, so the wiki MUST pre-exist before create/list.
/// Every test that expects a doc op to SUCCEED on a wiki calls this first.
async fn new_wiki(store: &Store, name: &str) -> WikiId {
    store.create_wiki(name).await.unwrap().wiki_id
}

/// Assert `s` matches exactly the ISO-8601 UTC shape that the crate's
/// `iso_now()` emits (§4.1.1/§4.3.1): `YYYY-MM-DDTHH:MM:SS.nnnnnnnnnZ` (9
/// fractional digits + trailing `Z`). Manual shape check — no `regex` dep is in
/// `Cargo.toml`, so this asserts char-by-char on the fixed 30-byte layout.
///
/// DOC-REVIEW QUICK-PIN (docs/pending.md): the format is now pinned, not just
/// "non-empty".
fn assert_iso8601_utc(s: &str) {
    let b = s.as_bytes();
    assert_eq!(
        b.len(),
        30,
        "ISO-8601 UTC must be 30 chars, got {s:?} (len {})",
        b.len()
    );
    // YYYY-MM-DD (YYYY, '-' MM '-' DD)
    for &i in &[0usize, 1, 2, 3, 5, 6, 8, 9] {
        assert!(b[i].is_ascii_digit(), "pos {i} of {s:?} must be a digit");
    }
    assert_eq!(b[4], b'-', "pos 4 of {s:?} must be '-'");
    assert_eq!(b[7], b'-', "pos 7 of {s:?} must be '-'");
    // 'T' separator
    assert_eq!(b[10], b'T', "pos 10 of {s:?} must be 'T'");
    // HH:MM:SS (HH ':' MM ':' SS)
    for &i in &[11usize, 12, 14, 15, 17, 18] {
        assert!(b[i].is_ascii_digit(), "pos {i} of {s:?} must be a digit");
    }
    assert_eq!(b[13], b':', "pos 13 of {s:?} must be ':'");
    assert_eq!(b[16], b':', "pos 16 of {s:?} must be ':'");
    // '.' then exactly 9 fractional digits
    assert_eq!(b[19], b'.', "pos 19 of {s:?} must be '.'");
    for (frac_i, &c) in b.iter().enumerate().take(29).skip(20) {
        assert!(c.is_ascii_digit(), "pos {frac_i} of {s:?} must be a digit");
    }
    // trailing 'Z' = UTC
    assert_eq!(b[29], b'Z', "pos 29 of {s:?} must be 'Z' (UTC)");
}

/// A valid Provident graph for `self_id` carrying a `link` reference edge whose
/// target lives in `target_id`'s document. Used to make a referrer actually
/// reference `target` so `deleteDocument(target)` hits the §4.4.5
/// reference-integrity gate (`DocumentInUse`).
fn graph_with_link_to(self_id: &DocumentId, target_id: &DocumentId) -> Graph {
    let n1 = Node {
        document_id: self_id.clone(),
        node_id: NodeId("n1".into()),
        kind: NodeKind::Content,
        value: Some("hello".into()),
        fact_key: None,
        target: None,
    };
    let ref1 = Node {
        document_id: self_id.clone(),
        node_id: NodeId("ref1".into()),
        kind: NodeKind::Reference,
        value: None,
        fact_key: None,
        target: Some((target_id.clone(), NodeId("n1".into()))),
    };
    Graph {
        nodes: vec![n1.clone(), ref1.clone()],
        edges: vec![
            Edge {
                source: (self_id.clone(), NodeId("ROOT".into())),
                target: (self_id.clone(), n1.node_id.clone()),
                kind: EdgeKind::DocHead,
                state: None,
                cross_wiki: false,
                relation_type: None,
            },
            Edge {
                source: (self_id.clone(), n1.node_id.clone()),
                target: (self_id.clone(), NodeId("END".into())),
                kind: EdgeKind::DocEnd,
                state: None,
                cross_wiki: false,
                relation_type: None,
            },
            Edge {
                source: (self_id.clone(), ref1.node_id.clone()),
                target: (target_id.clone(), NodeId("n1".into())),
                kind: EdgeKind::Link,
                state: Some(ReferenceState::Resolved),
                cross_wiki: false,
                relation_type: None,
            },
        ],
    }
}

/// A valid Provident graph for `doc_id("d1")` carrying a **crosslink** reference
/// edge with an explicitly supplied reference state. Used to test the §4.4.3
/// publish gate on `crosslink` edges in `BROKEN`/`STALE` state (adversarial
/// finding #2 — the gate historically ignored crosslink state entirely).
fn graph_with_crosslink_state(state: ReferenceState) -> Graph {
    let mut g = valid_graph();
    g.nodes.push(Node {
        document_id: doc_id("d1"),
        node_id: NodeId("ref1".into()),
        kind: NodeKind::Reference,
        value: None,
        fact_key: None,
        target: Some((doc_id("e2"), NodeId("n1".into()))),
    });
    g.edges.push(Edge {
        source: (doc_id("d1"), NodeId("ref1".into())),
        target: (doc_id("e2"), NodeId("n1".into())),
        kind: EdgeKind::Crosslink,
        state: Some(state),
        cross_wiki: true,
        relation_type: None,
    });
    g
}

/// A valid Provident graph for `self_id` carrying a **non-reference** edge kind
/// (`DocChild`/`Relation`) whose target lives in `target_id`'s document. Such an
/// edge is *not* a `link`/`embed`/`crosslink` reference, so §4.4.5's delete gate
/// MUST NOT treat it as making `target_id` "in use".
fn graph_with_non_reference_edge_to(
    kind: EdgeKind,
    self_id: &DocumentId,
    target_id: &DocumentId,
) -> Graph {
    let n1 = Node {
        document_id: self_id.clone(),
        node_id: NodeId("n1".into()),
        kind: NodeKind::Content,
        value: Some("hello".into()),
        fact_key: None,
        target: None,
    };
    Graph {
        nodes: vec![n1.clone()],
        edges: vec![
            Edge {
                source: (self_id.clone(), NodeId("ROOT".into())),
                target: (self_id.clone(), n1.node_id.clone()),
                kind: EdgeKind::DocHead,
                state: None,
                cross_wiki: false,
                relation_type: None,
            },
            Edge {
                source: (self_id.clone(), n1.node_id.clone()),
                target: (self_id.clone(), NodeId("END".into())),
                kind: EdgeKind::DocEnd,
                state: None,
                cross_wiki: false,
                relation_type: None,
            },
            Edge {
                source: (self_id.clone(), n1.node_id.clone()),
                target: (target_id.clone(), NodeId("n1".into())),
                kind,
                state: None,
                cross_wiki: false,
                relation_type: None,
            },
        ],
    }
}

// ---------------------------------------------------------------------------
// §4.1.1 — document model / state-machine contract (pure type, GREEN)
// ---------------------------------------------------------------------------

/// Legal transitions per §4.1.1. This is the state-machine contract the store's
/// publish/unpublish/archive operations must enforce.
#[test]
fn doc_state_legal_transitions() {
    use DocState::*;
    assert!(Draft.can_transition_to(Published), "DRAFT→PUBLISHED legal");
    assert!(Draft.can_transition_to(Archived), "DRAFT→ARCHIVED legal");
    assert!(
        Published.can_transition_to(Draft),
        "PUBLISHED→DRAFT (unpublish) legal"
    );
    assert!(
        Published.can_transition_to(Archived),
        "PUBLISHED→ARCHIVED legal"
    );
    assert!(!Draft.can_transition_to(Draft), "no self-transition");
    assert!(
        !Published.can_transition_to(Published),
        "no self-transition"
    );
    assert!(
        !Archived.can_transition_to(Published),
        "ARCHIVED is terminal"
    );
    assert!(!Archived.can_transition_to(Draft), "ARCHIVED is terminal");
    assert!(
        !Archived.can_transition_to(Archived),
        "ARCHIVED is terminal"
    );
}

// ---------------------------------------------------------------------------
// §4.1.3 createDocument
// ---------------------------------------------------------------------------

/// Happy path: a new Document is `revision = 0`, state `DRAFT`, belongs to the
/// wiki, and carries `createdAt`/`updatedAt`.
#[tokio::test]
async fn create_document_initial_state_revision_zero_draft() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w1").await;
    let doc = store
        .create_document(&w, make_meta_request("My doc"))
        .await
        .unwrap();
    assert_eq!(doc.revision, 0, "initial revision must be 0 (§4.1.1)");
    assert_eq!(doc.state, DocState::Draft, "initial state must be DRAFT");
    assert_eq!(doc.wiki_id, w, "Document belongs to exactly one Wiki");
    assert!(
        !doc.created_at.is_empty(),
        "createdAt (ISO-8601 UTC) must be set"
    );
    assert!(
        !doc.updated_at.is_empty(),
        "updatedAt (ISO-8601 UTC) must be set"
    );
    // DOC-REVIEW QUICK-PIN: pin the ISO-8601 UTC FORMAT, not just non-empty.
    assert_iso8601_utc(&doc.created_at);
    assert_iso8601_utc(&doc.updated_at);
    assert_eq!(doc.tags, vec!["tag-a".to_string()]);
}

/// createDocument on an unknown `wikiId` → `WikiNotFound` (FS-2).
#[tokio::test]
async fn create_document_unknown_wiki_returns_wiki_not_found() {
    let store = Arc::new(Store::new());
    let err = store
        .create_document(&wiki("unknown"), make_meta_request("x"))
        .await
        .unwrap_err();
    assert_eq!(err, StoreError::WikiNotFound, "FS-2 WikiNotFound");
}

/// createDocument with an empty title → `ValidationError` (FS-3). The wiki is
/// created first so the failure is unambiguously the validation (not masked by
/// `WikiNotFound` on an unknown wiki).
#[tokio::test]
async fn create_document_empty_title_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let err = store
        .create_document(&w, make_meta_request(""))
        .await
        .unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "FS-3 empty title"
    );
}

/// createDocument with a title > 200 chars → `ValidationError` (FS-3). As
/// above, the wiki pre-exists so the failure is the validation alone.
#[tokio::test]
async fn create_document_overlong_title_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let long = "x".repeat(201);
    let err = store
        .create_document(&w, make_meta_request(&long))
        .await
        .unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "FS-3 title>200"
    );
}

/// Exact-length boundary: a title of **exactly 200 chars** is valid (§4.1.3
/// bounds are "non-empty and at most 200"). The sibling test only pins the
/// over-limit (201) side; this locks the inclusive 200 boundary so a later
/// off-by-one in the validator cannot silently regress it.
#[tokio::test]
async fn create_document_title_exactly_200_is_valid() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let title = "x".repeat(200);
    assert_eq!(
        title.chars().count(),
        200,
        "test premise: exactly 200 chars"
    );
    let doc = store
        .create_document(&w, make_meta_request(&title))
        .await
        .unwrap_or_else(|e| panic!("exactly-200-char title must create, got {e:?}"));
    // It actually persisted, not just "didn't error" — get it back and check.
    let fetched = store.get_document(&doc.document_id).await.unwrap();
    assert_eq!(fetched.title.chars().count(), 200);
}

// ---------------------------------------------------------------------------
// §4.1.3 getDocument
// ---------------------------------------------------------------------------

/// Happy path: `getDocument` returns the Document at its current revision, and a
/// freshly created one returns `revision = 0`.
#[tokio::test]
async fn get_document_returns_document_at_current_revision() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let created = store
        .create_document(&w, make_meta_request("t"))
        .await
        .unwrap();
    let fetched = store.get_document(&created.document_id).await.unwrap();
    assert_eq!(fetched.document_id, created.document_id);
    assert_eq!(fetched.revision, created.revision);
    assert_eq!(fetched.state, DocState::Draft);
}

/// getDocument on an unknown `documentId` → `DocumentNotFound` (FS-1).
#[tokio::test]
async fn get_document_unknown_id_is_document_not_found() {
    let store = Arc::new(Store::new());
    let err = store
        .get_document(&doc_id("does-not-exist"))
        .await
        .unwrap_err();
    assert_eq!(err, StoreError::DocumentNotFound, "FS-1 DocumentNotFound");
}

// ---------------------------------------------------------------------------
// §4.1.3 / §4.1.4 updateDocument (optimistic concurrency)
// ---------------------------------------------------------------------------

/// Happy path: a valid update from `base_revision = 0` bumps the document to
/// `revision = 1` and mutates title/tags (§4.1.4: revision monotonic, bump on
/// commit).
#[tokio::test]
async fn update_document_bumps_revision_and_applies_changes() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let created = store
        .create_document(&w, make_meta_request("v1"))
        .await
        .unwrap();
    let updated = store
        .update_document(&created.document_id, update("v2", 0))
        .await
        .unwrap();
    assert_eq!(updated.revision, 1, "revision must bump to base+1 (§4.1.4)");
    assert_eq!(updated.title, "v2");
}

/// A stale base revision → `ConflictError` (FS-4). Read at rev 0, someone else
/// bumps to rev 1, then a caller still holding `base_revision = 0` is rejected.
#[tokio::test]
async fn update_document_stale_base_is_conflict_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let created = store
        .create_document(&w, make_meta_request("v0"))
        .await
        .unwrap();
    // First committed update moves the stored revision 0 → 1.
    store
        .update_document(&created.document_id, update("v1", 0))
        .await
        .unwrap();
    // Reusing `base_revision = 0` is now stale.
    let err = store
        .update_document(&created.document_id, update("v2", 0))
        .await
        .unwrap_err();
    assert_eq!(
        err,
        StoreError::ConflictError,
        "FS-4 ConflictError (HTTP 409)"
    );
}

/// updateDocument on an unknown `documentId` → `DocumentNotFound` (FS-1).
#[tokio::test]
async fn update_document_unknown_id_is_document_not_found() {
    let store = Arc::new(Store::new());
    let err = store
        .update_document(&doc_id("nope"), update("x", 0))
        .await
        .unwrap_err();
    assert_eq!(err, StoreError::DocumentNotFound, "FS-1 DocumentNotFound");
}

/// updateDocument with an invalid Provident graph (no `doc-head`, no `doc-end`)
/// → `ValidationError` (FS-3).
#[tokio::test]
async fn update_document_invalid_graph_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let created = store
        .create_document(&w, make_meta_request("t"))
        .await
        .unwrap();
    let invalid = UpdateDocumentRequest {
        base_revision: 0,
        graph: Graph {
            nodes: vec![],
            edges: vec![],
        }, // no doc-head / doc-end
        title: Some("broken".into()),
        tags: None,
    };
    let err = store
        .update_document(&created.document_id, invalid)
        .await
        .unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "FS-3 invalid graph"
    );
}

// ---------------------------------------------------------------------------
// §4.4.5 / §4.1.3 deleteDocument
// ---------------------------------------------------------------------------

/// Happy path: after deletion the document is gone (get → DocumentNotFound) and
/// the delete returns unit.
#[tokio::test]
async fn delete_document_removes_it() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let created = store
        .create_document(&w, make_meta_request("t"))
        .await
        .unwrap();
    store.delete_document(&created.document_id).await.unwrap();
    let err = store.get_document(&created.document_id).await.unwrap_err();
    assert_eq!(err, StoreError::DocumentNotFound, "deleted doc is gone");
}

/// deleteDocument on an unknown `documentId` → `DocumentNotFound` (FS-1).
#[tokio::test]
async fn delete_document_unknown_id_is_document_not_found() {
    let store = Arc::new(Store::new());
    let err = store.delete_document(&doc_id("nope")).await.unwrap_err();
    assert_eq!(err, StoreError::DocumentNotFound, "FS-1 DocumentNotFound");
}

/// deleteDocument of a document that another document references with a
/// `link`/`embed`/`crosslink` whose target is in it → `DocumentInUse` (FS-5).
#[tokio::test]
async fn delete_document_when_referenced_is_document_in_use() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let target = store
        .create_document(&w, make_meta_request("target-doc"))
        .await
        .unwrap();
    // A second document in the same wiki holds a `link` edge whose target is in
    // `target`'s document, so §4.4.5 forbids deleting `target` (FS-5).
    let referrer = store
        .create_document(&w, make_meta_request("referrer"))
        .await
        .unwrap();
    store
        .update_document(
            &referrer.document_id,
            UpdateDocumentRequest {
                base_revision: 0,
                graph: graph_with_link_to(&referrer.document_id, &target.document_id),
                title: None,
                tags: None,
            },
        )
        .await
        .unwrap();
    let err = store
        .delete_document(&target.document_id)
        .await
        .unwrap_err();
    assert_eq!(
        err,
        StoreError::DocumentInUse,
        "FS-5 DocumentInUse (§4.4.5)"
    );
}

/// **Adversarial finding #3 (negative, GREEN today).** §4.4.5's delete gate
/// blocks a deletion only when another document holds a `link`/`embed`/`crosslink`
/// **reference** edge into the deleted doc. A `DocChild` (containment) or
/// `Relation` (triple) edge whose target happens to live in another document is
/// *not* a reference and must NOT trigger `DocumentInUse`. This locks in the
/// correct behavior so a future over-broad delete gate cannot silently regress
/// it. (Cross-wiki containment/relation is unusual but is valid arbitrary-graph
/// data; the gate must still be kind-specific.)
#[tokio::test]
async fn delete_document_with_non_reference_edge_to_other_doc_is_allowed() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;

    // DocChild edge targeting another document → delete target must succeed.
    let target_child = store
        .create_document(&w, make_meta_request("target-child"))
        .await
        .unwrap();
    let referrer_child = store
        .create_document(&w, make_meta_request("referrer-child"))
        .await
        .unwrap();
    store
        .update_document(
            &referrer_child.document_id,
            UpdateDocumentRequest {
                base_revision: 0,
                graph: graph_with_non_reference_edge_to(
                    EdgeKind::DocChild,
                    &referrer_child.document_id,
                    &target_child.document_id,
                ),
                title: None,
                tags: None,
            },
        )
        .await
        .unwrap();
    store
        .delete_document(&target_child.document_id)
        .await
        .unwrap_or_else(|e| panic!("DocChild edge must not block delete, got {e:?}"));

    // Relation edge targeting another document → delete target must succeed.
    let target_rel = store
        .create_document(&w, make_meta_request("target-relation"))
        .await
        .unwrap();
    let referrer_rel = store
        .create_document(&w, make_meta_request("referrer-relation"))
        .await
        .unwrap();
    store
        .update_document(
            &referrer_rel.document_id,
            UpdateDocumentRequest {
                base_revision: 0,
                graph: graph_with_non_reference_edge_to(
                    EdgeKind::Relation,
                    &referrer_rel.document_id,
                    &target_rel.document_id,
                ),
                title: None,
                tags: None,
            },
        )
        .await
        .unwrap();
    store
        .delete_document(&target_rel.document_id)
        .await
        .unwrap_or_else(|e| panic!("Relation edge must not block delete, got {e:?}"));
}

// ---------------------------------------------------------------------------
// publishDocument + §4.4.3 publish gate
// ---------------------------------------------------------------------------

/// Happy path: `DRAFT → PUBLISHED`; the returned Document carries the new state.
#[tokio::test]
async fn publish_document_transitions_draft_to_published() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let created = store
        .create_document(&w, make_meta_request("t"))
        .await
        .unwrap();
    let published = store.publish_document(&created.document_id).await.unwrap();
    assert_eq!(published.state, DocState::Published);
}

/// A document with a **resolved** cross-wiki reference (§4.1.2) may be
/// published — cross-wiki references are permitted and a resolved one does not
/// block the publish gate.
#[tokio::test]
async fn publish_document_with_resolved_crosslink_is_allowed() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let req = CreateDocumentRequest {
        title: "t".into(),
        tags: None,
        author: None,
    };
    let created = store.create_document(&w, req).await.unwrap();
    // Apply the cross-wiki reference graph via update so the publish gate
    // (§4.4.3) inspects the real graph. A *resolved* crosslink does not block
    // the gate, so the document publishes.
    store
        .update_document(
            &created.document_id,
            UpdateDocumentRequest {
                base_revision: 0,
                graph: graph_with_crosslink(),
                title: None,
                tags: None,
            },
        )
        .await
        .unwrap();
    let published = store.publish_document(&created.document_id).await.unwrap();
    assert_eq!(published.state, DocState::Published);
}

/// publishDocument on an unknown `documentId` → `DocumentNotFound` (FS-1).
#[tokio::test]
async fn publish_document_unknown_id_is_document_not_found() {
    let store = Arc::new(Store::new());
    let err = store.publish_document(&doc_id("nope")).await.unwrap_err();
    assert_eq!(err, StoreError::DocumentNotFound, "FS-1 DocumentNotFound");
}

/// Publish gate: a document containing a **BROKEN** link → `UnresolvedReference`
/// (FS-7 / §4.4.3).
#[tokio::test]
async fn publish_document_with_broken_reference_is_unresolved_reference() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let req = CreateDocumentRequest {
        title: "t".into(),
        tags: None,
        author: None,
    };
    let created = store.create_document(&w, req).await.unwrap();
    // Apply the broken-reference graph via update (base_revision 0 → revision
    // 1), so the publish gate (§4.4.3) sees the BROKEN link and blocks.
    store
        .update_document(
            &created.document_id,
            UpdateDocumentRequest {
                base_revision: 0,
                graph: graph_with_broken_reference(),
                title: None,
                tags: None,
            },
        )
        .await
        .unwrap();
    let err = store
        .publish_document(&created.document_id)
        .await
        .unwrap_err();
    assert_eq!(
        err,
        StoreError::UnresolvedReference,
        "FS-7 BROKEN link blocks publish"
    );
}

/// Publish gate: a document containing a **STALE** embed → `UnresolvedReference`
/// (FS-7 / §4.4.3).
#[tokio::test]
async fn publish_document_with_stale_embed_is_unresolved_reference() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let created = store
        .create_document(&w, make_meta_request("t"))
        .await
        .unwrap();
    // Apply the stale-embed graph via update so the publish gate (§4.4.3) sees
    // the STALE embed and blocks.
    store
        .update_document(
            &created.document_id,
            UpdateDocumentRequest {
                base_revision: 0,
                graph: graph_with_stale_embed(),
                title: None,
                tags: None,
            },
        )
        .await
        .unwrap();
    let err = store
        .publish_document(&created.document_id)
        .await
        .unwrap_err();
    assert_eq!(
        err,
        StoreError::UnresolvedReference,
        "FS-7 STALE embed blocks publish"
    );
}

/// **Adversarial finding #2 (MEDIUM).** The §4.4.3 publish gate must apply to a
/// `crosslink` reference edge in `BROKEN` state too — a `crosslink` carries the
/// same reference state (§4.4.2: `FRESH`/`STALE`/`RESOLVED`/`BROKEN`), and §4.4.3
/// forbids silently traversing a `BROKEN` reference (§4.4.1a includes crosslink
/// in the staleness surface). Today the gate matches only `(Link, Broken)` and
/// `(Embed, Stale)`, so an explicit `BROKEN` crosslink slips through and the doc
/// publishes — this test is **RED** against the current implementation.
#[tokio::test]
async fn publish_document_with_broken_crosslink_is_unresolved_reference() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let req = CreateDocumentRequest {
        title: "t".into(),
        tags: None,
        author: None,
    };
    let created = store.create_document(&w, req).await.unwrap();
    // Apply a graph whose crosslink edge is explicitly BROKEN.
    store
        .update_document(
            &created.document_id,
            UpdateDocumentRequest {
                base_revision: 0,
                graph: graph_with_crosslink_state(ReferenceState::Broken),
                title: None,
                tags: None,
            },
        )
        .await
        .unwrap();
    let err = store
        .publish_document(&created.document_id)
        .await
        .unwrap_err();
    assert_eq!(
        err,
        StoreError::UnresolvedReference,
        "FS-7 BROKEN crosslink blocks publish (§4.4.2/§4.4.3)"
    );
}

/// **Adversarial finding #2 (MEDIUM), `STALE` variant.** A `crosslink` in `STALE`
/// state (unsynced snapshot) must equally block the publish gate — §4.4.3 fails
/// publish on any `STALE` reference that has not been re-synced. Today the cross
/// link state is ignored, so this test is **RED** against the current
/// implementation.
#[tokio::test]
async fn publish_document_with_stale_crosslink_is_unresolved_reference() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let req = CreateDocumentRequest {
        title: "t".into(),
        tags: None,
        author: None,
    };
    let created = store.create_document(&w, req).await.unwrap();
    store
        .update_document(
            &created.document_id,
            UpdateDocumentRequest {
                base_revision: 0,
                graph: graph_with_crosslink_state(ReferenceState::Stale),
                title: None,
                tags: None,
            },
        )
        .await
        .unwrap();
    let err = store
        .publish_document(&created.document_id)
        .await
        .unwrap_err();
    assert_eq!(
        err,
        StoreError::UnresolvedReference,
        "FS-7 STALE crosslink blocks publish (§4.4.2/§4.4.3)"
    );
}

/// **Adversarial finding #4 (forward-looking guard).** A stored reference state
/// can be *fabricated* — e.g. a `link` edge stamped `Resolved` whose target
/// document does not exist. The §4.4.1 invariant ("every `link` must resolve to
/// a live, non-stale target") means the publish gate must not trust the stored
/// `state` alone: it must derive/verify target existence. So publishing a
/// document that references a nonexistent document must be
/// `UnresolvedReference` regardless of the stored `state`. The current gate only
/// inspects the stored `state` (and only for `Link/Broken`), so this test is
/// **RED** — please verify the reported red/green status after running.
#[tokio::test]
async fn publish_document_with_resolved_link_to_nonexistent_target_is_unresolved_reference() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let created = store
        .create_document(&w, make_meta_request("t"))
        .await
        .unwrap();
    // A `link` edge whose stored state claims `Resolved` but whose target doc
    // (`ghost`) was never created. The gate must reject on target absence, not
    // on the client-supplied state.
    let ghost = doc_id("ghost-doc-that-does-not-exist");
    store
        .update_document(
            &created.document_id,
            UpdateDocumentRequest {
                base_revision: 0,
                graph: graph_with_link_to(&created.document_id, &ghost),
                title: None,
                tags: None,
            },
        )
        .await
        .unwrap();
    let err = store
        .publish_document(&created.document_id)
        .await
        .unwrap_err();
    assert_eq!(
        err,
        StoreError::UnresolvedReference,
        "link to a nonexistent document is UnresolvedReference regardless of stored state"
    );
}

// ---------------------------------------------------------------------------
// unpublishDocument
// ---------------------------------------------------------------------------

/// Happy path: `PUBLISHED → DRAFT` (unpublish).
#[tokio::test]
async fn unpublish_document_transitions_published_to_draft() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let created = store
        .create_document(&w, make_meta_request("t"))
        .await
        .unwrap();
    store.publish_document(&created.document_id).await.unwrap();
    let doc = store
        .unpublish_document(&created.document_id)
        .await
        .unwrap();
    assert_eq!(doc.state, DocState::Draft);
}

/// unpublishDocument on a document not in `PUBLISHED` → `InvalidState` (FS-6).
#[tokio::test]
async fn unpublish_document_not_published_is_invalid_state() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let created = store
        .create_document(&w, make_meta_request("t"))
        .await
        .unwrap(); // DRAFT
    let err = store
        .unpublish_document(&created.document_id)
        .await
        .unwrap_err();
    assert_eq!(
        err,
        StoreError::InvalidState,
        "FS-6 unpublish of non-PUBLISHED"
    );
}

/// unpublishDocument on an unknown `documentId` → `DocumentNotFound` (FS-1).
#[tokio::test]
async fn unpublish_document_unknown_id_is_document_not_found() {
    let store = Arc::new(Store::new());
    let err = store.unpublish_document(&doc_id("nope")).await.unwrap_err();
    assert_eq!(err, StoreError::DocumentNotFound, "FS-1 DocumentNotFound");
}

// ---------------------------------------------------------------------------
// archiveDocument
// ---------------------------------------------------------------------------

/// Happy path: `DRAFT → ARCHIVED`.
#[tokio::test]
async fn archive_document_transitions_draft_to_archived() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let created = store
        .create_document(&w, make_meta_request("t"))
        .await
        .unwrap();
    let doc = store.archive_document(&created.document_id).await.unwrap();
    assert_eq!(doc.state, DocState::Archived);
}

/// Happy path: `PUBLISHED → ARCHIVED`.
#[tokio::test]
async fn archive_document_transitions_published_to_archived() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let created = store
        .create_document(&w, make_meta_request("t"))
        .await
        .unwrap();
    store.publish_document(&created.document_id).await.unwrap();
    let doc = store.archive_document(&created.document_id).await.unwrap();
    assert_eq!(doc.state, DocState::Archived);
}

/// archiveDocument of an already-`ARCHIVED` document → `InvalidState` (FS-6).
#[tokio::test]
async fn archive_document_already_archived_is_invalid_state() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let created = store
        .create_document(&w, make_meta_request("t"))
        .await
        .unwrap();
    store.archive_document(&created.document_id).await.unwrap();
    let err = store
        .archive_document(&created.document_id)
        .await
        .unwrap_err();
    assert_eq!(
        err,
        StoreError::InvalidState,
        "FS-6 archive of ARCHIVED is terminal"
    );
}

/// archiveDocument on an unknown `documentId` → `DocumentNotFound` (FS-1).
#[tokio::test]
async fn archive_document_unknown_id_is_document_not_found() {
    let store = Arc::new(Store::new());
    let err = store.archive_document(&doc_id("nope")).await.unwrap_err();
    assert_eq!(err, StoreError::DocumentNotFound, "FS-1 DocumentNotFound");
}

// ---------------------------------------------------------------------------
// listDocuments — pagination, filters, shape, fail-states
// ---------------------------------------------------------------------------

/// Happy path: `listDocuments` returns the `{items, total, page, pageSize}`
/// shape with all documents in the wiki.
#[tokio::test]
async fn list_documents_returns_page_shape() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    store
        .create_document(&w, make_meta_request("a"))
        .await
        .unwrap();
    store
        .create_document(&w, make_meta_request("b"))
        .await
        .unwrap();
    let filter = ListDocumentsFilter {
        page: Some(1),
        page_size: Some(10),
        state: None,
        tag: None,
    };
    let list = store.list_documents(&w, &filter).await.unwrap();
    assert_eq!(list.page, 1);
    assert_eq!(list.page_size, 10);
    assert_eq!(list.total, 2);
    assert_eq!(list.items.len(), 2);
}

/// `listDocuments` on an unknown `wikiId` → `WikiNotFound` (FS-2).
#[tokio::test]
async fn list_documents_unknown_wiki_is_wiki_not_found() {
    let store = Arc::new(Store::new());
    let filter = ListDocumentsFilter {
        page: Some(1),
        page_size: Some(10),
        state: None,
        tag: None,
    };
    let err = store
        .list_documents(&wiki("unknown"), &filter)
        .await
        .unwrap_err();
    assert_eq!(err, StoreError::WikiNotFound, "FS-2 WikiNotFound");
}

/// `page < 1` → `ValidationError` (FS-3). The wiki pre-exists so the failure is
/// unambiguously the pagination validation, not `WikiNotFound`.
#[tokio::test]
async fn list_documents_page_less_than_one_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let filter = ListDocumentsFilter {
        page: Some(0),
        page_size: Some(10),
        state: None,
        tag: None,
    };
    let err = store.list_documents(&w, &filter).await.unwrap_err();
    assert!(matches!(err, StoreError::ValidationError(_)), "FS-3 page<1");
}

/// `pageSize < 1` → `ValidationError` (FS-3).
#[tokio::test]
async fn list_documents_page_size_less_than_one_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let filter = ListDocumentsFilter {
        page: Some(1),
        page_size: Some(0),
        state: None,
        tag: None,
    };
    let err = store.list_documents(&w, &filter).await.unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "FS-3 pageSize<1"
    );
}

/// `pageSize > 100` → `ValidationError` (FS-3).
#[tokio::test]
async fn list_documents_page_size_over_one_hundred_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let filter = ListDocumentsFilter {
        page: Some(1),
        page_size: Some(101),
        state: None,
        tag: None,
    };
    let err = store.list_documents(&w, &filter).await.unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "FS-3 pageSize>100"
    );
}

/// `state` filter returns only documents in that state.
#[tokio::test]
async fn list_documents_filters_by_state() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let published = store
        .create_document(&w, make_meta_request("pub"))
        .await
        .unwrap();
    store
        .publish_document(&published.document_id)
        .await
        .unwrap();
    let filter = ListDocumentsFilter {
        state: Some(DocState::Published),
        page: Some(1),
        page_size: Some(10),
        tag: None,
    };
    let list = store.list_documents(&w, &filter).await.unwrap();
    assert!(list.items.iter().all(|s| s.state == DocState::Published));
}

/// `tag` filter returns only documents carrying that tag (others excluded).
#[tokio::test]
async fn list_documents_filters_by_tag() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let tagged = store
        .create_document(&w, make_meta_request("a"))
        .await
        .unwrap(); // tag-a
    store
        .create_document(
            &w,
            CreateDocumentRequest {
                title: "b".into(),
                tags: Some(vec!["other".into()]),
                author: None,
            },
        )
        .await
        .unwrap();
    let filter = ListDocumentsFilter {
        tag: Some("tag-a".into()),
        page: Some(1),
        page_size: Some(10),
        state: None,
    };
    let list = store.list_documents(&w, &filter).await.unwrap();
    assert_eq!(list.total, 1, "only the tag-a document matches");
    assert_eq!(list.items.len(), 1);
    assert_eq!(list.items[0].document_id, tagged.document_id);
}

/// An empty (but **existing**) wiki lists nobody: `total = 0`, `items` empty.
/// The wiki MUST pre-exist — `listDocuments` on an *unknown* wiki is
/// `WikiNotFound` (§4.1.3), so an empty page only follows a previously-created
/// wiki with no documents yet.
#[tokio::test]
async fn list_documents_on_empty_wiki_returns_empty_page() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let filter = ListDocumentsFilter {
        page: Some(1),
        page_size: Some(10),
        state: None,
        tag: None,
    };
    let list = store.list_documents(&w, &filter).await.unwrap();
    assert_eq!(list.total, 0);
    assert!(list.items.is_empty());
}

/// **Adversarial finding #1 (HIGH).** A requested `page` beyond the data
/// (2 documents but `page: 2` with the default `page_size` → 20) must be a
/// **valid, non-panicking** request that returns an **empty `items`** slice on a
/// correctly-shaped page (`total == 2`, `page == 2`). Today the store slices
/// `matched[(page-1)*pageSize .. (page-1)*pageSize+pageSize]` past the end and
/// **panics** — this test is RED against the current implementation.
#[tokio::test]
async fn list_documents_page_beyond_range_returns_empty_items() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    store
        .create_document(&w, make_meta_request("a"))
        .await
        .unwrap();
    store
        .create_document(&w, make_meta_request("b"))
        .await
        .unwrap();
    // page = 2, page_size = None → default page_size (20), which exceeds the
    // 2-document dataset.
    let filter = ListDocumentsFilter {
        page: Some(2),
        page_size: None, // default page_size
        state: None,
        tag: None,
    };
    let list = store.list_documents(&w, &filter).await.unwrap();
    assert!(
        list.items.is_empty(),
        "page beyond the data returns an empty items slice (no panic)"
    );
    assert_eq!(list.total, 2, "total is unaffected by out-of-range page");
    assert_eq!(list.page, 2, "the echoed page matches the request");
}

/// **Adversarial finding #1 (HIGH).** `page: u64::MAX` similarly must not panic
/// and must return empty `items` — the extreme bound of the out-of-range case.
#[tokio::test]
async fn list_documents_page_u64_max_returns_empty_items() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    store
        .create_document(&w, make_meta_request("a"))
        .await
        .unwrap();
    store
        .create_document(&w, make_meta_request("b"))
        .await
        .unwrap();
    let filter = ListDocumentsFilter {
        page: Some(u64::MAX),
        page_size: None, // default page_size
        state: None,
        tag: None,
    };
    let list = store.list_documents(&w, &filter).await.unwrap();
    assert!(
        list.items.is_empty(),
        "page u64::MAX returns empty items (no panic / no overflow)"
    );
    assert_eq!(list.total, 2);
}

// ---------------------------------------------------------------------------
// Wiki operations — §4.1.2 / §4.1.3
// ---------------------------------------------------------------------------

/// createWiki happy path: returns a Wiki with the given name.
#[tokio::test]
async fn create_wiki_returns_wiki() {
    let store = Arc::new(Store::new());
    let w = store.create_wiki("My Wiki").await.unwrap();
    assert_eq!(w.name, "My Wiki");
}

/// createWiki with an empty name → `ValidationError` (FS-3).
#[tokio::test]
async fn create_wiki_empty_name_is_validation_error() {
    let store = Arc::new(Store::new());
    let err = store.create_wiki("").await.unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "FS-3 empty wiki name"
    );
}

/// createWiki with a name > 100 chars → `ValidationError` (FS-3).
#[tokio::test]
async fn create_wiki_overlong_name_is_validation_error() {
    let store = Arc::new(Store::new());
    let err = store.create_wiki(&"x".repeat(101)).await.unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "FS-3 wiki name>100"
    );
}

/// Exact-length boundary: a wiki name of **exactly 100 chars** is valid
/// (§4.1.2 bounds are "non-empty and at most 100"). Pins the inclusive 100
/// boundary on the same side the sibling test leaves untested.
#[tokio::test]
async fn create_wiki_name_exactly_100_is_valid() {
    let store = Arc::new(Store::new());
    let name = "x".repeat(100);
    assert_eq!(name.chars().count(), 100, "test premise: exactly 100 chars");
    let wiki = store
        .create_wiki(&name)
        .await
        .unwrap_or_else(|e| panic!("exactly-100-char name must create, got {e:?}"));
    assert_eq!(wiki.name.chars().count(), 100);
}

/// getWiki happy path returns the named wiki.
#[tokio::test]
async fn get_wiki_returns_wiki() {
    let store = Arc::new(Store::new());
    let created = store.create_wiki("w").await.unwrap();
    let got = store.get_wiki(&created.wiki_id).await.unwrap();
    assert_eq!(got.name, "w");
}

/// getWiki on an unknown `wikiId` → `WikiNotFound` (FS-2).
#[tokio::test]
async fn get_wiki_unknown_id_is_wiki_not_found() {
    let store = Arc::new(Store::new());
    let err = store.get_wiki(&wiki("nope")).await.unwrap_err();
    assert_eq!(err, StoreError::WikiNotFound, "FS-2 WikiNotFound");
}

/// listWikis returns all wikis.
#[tokio::test]
async fn list_wikis_returns_all_wikis() {
    let store = Arc::new(Store::new());
    store.create_wiki("wiki-a").await.unwrap();
    store.create_wiki("wiki-b").await.unwrap();
    let all = store.list_wikis().await.unwrap();
    assert_eq!(all.len(), 2);
}

// ---------------------------------------------------------------------------
// Concurrency contract (SHARDED-RWLOCK-STORE / ARC-SHARED-ENGINE)
// ---------------------------------------------------------------------------

/// Concurrent readers: many tokio tasks `getDocument` a shared `Arc<Store>`
/// simultaneously; every read succeeds and sees the same `revision`/`state`.
///
/// **Hardened (adversarial finding: the old version ran on the single-threaded
/// default `#[tokio::test]` runtime and its op bodies had no `.await`, so the 16
/// spawned tasks were fully serialized and proved nothing about concurrency).**
/// This version runs on a **multi-thread** runtime (`worker_threads = 4`) and
/// every reader waits on a shared `Barrier` before issuing its `get_document`,
/// so 8 readers genuinely race the read path across worker threads — a buggy
/// per-reader guard (e.g. one that swallows errors or returns a torn/partial
/// revision) must be exposed.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_readers_all_get_document() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let created = store
        .create_document(&w, make_meta_request("shared"))
        .await
        .unwrap();
    let id = created.document_id.clone();

    const N: usize = 8;
    let barrier = Arc::new(tokio::sync::Barrier::new(N));
    let handles: Vec<_> = (0..N)
        .map(|_| {
            let s = Arc::clone(&store);
            let id = id.clone();
            let barrier = Arc::clone(&barrier);
            tokio::spawn(async move {
                // All 8 readers block here and are released together, so they
                // truly overlap in flight across worker threads.
                barrier.wait().await;
                let d = s.get_document(&id).await.expect("reader saw the document");
                (d.revision, d.state)
            })
        })
        .collect();

    for h in handles {
        let (revision, state) = h.await.expect("reader task did not panic");
        assert_eq!(
            revision, created.revision,
            "all concurrent readers see the same revision"
        );
        assert_eq!(
            state,
            DocState::Draft,
            "all concurrent readers see the same state"
        );
    }
}

/// Optimistic-concurrency atomicity: two tasks `updateDocument` the same doc
/// with the **same base revision** (`0`). Under the shard `write()` guard
/// (compare-base-revision → apply → bump, §4.1.4 + SHARDED-RWLOCK-STORE), exactly
/// **one** succeeds; the other gets `ConflictError`.
///
/// **Hardened (adversarial finding: the old version ran single-threaded with no
/// `.await`, so the two updates were serialized and the test did not exercise
/// contention at all).** This version runs on a **multi-thread** runtime and
/// both tasks wait on `Barrier(2)` before issuing their update, so both read the
/// same base revision `R` and race to commit — if the store's critical section
/// is sound, exactly one wins and the other is `ConflictError`.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_update_same_base_exactly_one_succeeds() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let created = store
        .create_document(&w, make_meta_request("v0"))
        .await
        .unwrap();
    let id = created.document_id.clone();
    // R — the base revision both contenders read and both try to commit from.
    let base = created.revision;

    let s1 = Arc::clone(&store);
    let s2 = Arc::clone(&store);
    let id1 = id.clone();
    let id2 = id.clone();
    let req1 = update("v1", base);
    let req2 = update("v2", base);

    let barrier = Arc::new(tokio::sync::Barrier::new(2));
    let b1 = Arc::clone(&barrier);
    let b2 = Arc::clone(&barrier);

    // Both tasks wait on the barrier before calling `update_document`, so both
    // hold base R = created.revision while they race to commit from it.
    let t1 = tokio::spawn(async move {
        b1.wait().await;
        s1.update_document(&id1, req1).await
    });
    let t2 = tokio::spawn(async move {
        b2.wait().await;
        s2.update_document(&id2, req2).await
    });

    let mut succeeded = 0usize;
    let mut conflicted = 0usize;
    let mut unexpected = 0usize;
    for h in [t1, t2] {
        match h.await {
            Ok(Ok(_)) => succeeded += 1,
            Ok(Err(StoreError::ConflictError)) => conflicted += 1,
            Ok(Err(other)) => {
                let _ = other;
                unexpected += 1;
            }
            Err(_) => unexpected += 1,
        }
    }
    assert_eq!(
        unexpected, 0,
        "no unexpected errors/panics under concurrent update"
    );
    assert_eq!(
        succeeded, 1,
        "exactly one update must win under the same base revision (§4.1.4)"
    );
    assert_eq!(
        conflicted, 1,
        "the loser must be rejected with ConflictError (HTTP 409)"
    );
}
