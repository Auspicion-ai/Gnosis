//! §4.3 Fact/citation-tracking integration tests — written **RED-first** at the
//! TDD gate for the §4.3 unit.
//!
//! Every state and fail-state below is derived from the canonical behavior
//! contract alone (`docs/specs/gnosis.md` §4.3.1–§4.3.4, plus the pinned
//! `listFacts`/`getFact` rows of §4.5.4 and the §4.2.9 merge-fact citation
//! invariant) and the concurrency contract
//! (`docs/research/gnosis-data-structures-concurrency-plan.md`:
//! SHARDED-RWLOCK-STORE / ARC-SHARED-ENGINE / atomic compare-then-commit under a
//! lock).
//!
//! ## The §4.3 suite is GREEN
//!
//! The full §4.3 surface — the fact CRUD (`create_fact`/`get_fact`, re-derived
//! in §4.2 as the `mergeFacts` seeding mechanism) and the §4.3 additions
//! (`list_facts`, `update_fact`, `propose_candidate_fact`,
//! `get_query_audit_log`) — is **landed and green**: every state and fail-state
//! below passes on the current implementation. The pinned minimum-citation
//! invariant and the fail-closed gate are exercised both ways.
//!
//! **RED-follow-on (adversarial regressions).** The tests in the
//! "Adversarial regression surface" section at the bottom of this file pin
//! spec-correct behavior the current implementation does **not** yet enforce, so
//! they are **still RED** until the Implementer lands the matching fix:
//! (H1) `delete_document` must return `DocumentInUse` when a committed fact cites
//! a node of that document (the delete gate scans edges only, not `fact_store`);
//! (H2/H4) empty and whitespace-only fact `value`s and whitespace-only
//! `factKey`s are schema violations; (H5) an unknown wiki is `WikiNotFound` on
//! the whole fact surface (`create_fact`/`get_fact`/`update_fact`/
//! `propose_candidate_fact`); and (H1 TOCTOU proxy) a candidate whose cited
//! document was deleted is never committed.
//!
//! ## TestWriter contract decisions (for the Implementer to match exactly)
//!
//! - **`listFacts` `state?` filter** (§4.5.4) is undocumented in the spec and
//!   `Fact` carries no own `state` (the spec never defines a `FactState`). The
//!   reasonable reading, consistent with `listDocuments` (§4.1.3) whose `state?`
//!   uses `DocState`: the filter matches the **containing Document's** state — a
//!   fact is included iff the `Fact.document_id`'s document `state` equals the
//!   filter. `tests` assert exactly this.
//! - **`proposeCandidateFact`** is the deterministic, fail-closed **validation
//!   gate** of §4.3.2a. Success commits and returns `accepted: true` + `Fact`.
//!   Failures that §4.3.2a.4 pins as `StoreError` (dangling citation →
//!   `ValidationError`, duplicate `factKey` → `ConflictError`) surface as `Err`
//!   and commit nothing. All other gate failures (schema conformance: empty
//!   `factKey`/`value`/`citations`) return `accepted: false` +
//!   `Rejection {code, field, message}` and commit nothing. **Cross-field
//!   consistency (the value vs. the cited nodes' content) is NOT a runtime gate
//!   branch in §4.3** — those semantic content checks require the §4.5 embedding
//!   leg and entity resolution (§4.2.9), so per the adversarial review they are
//!   **deferred to §4.5**. No cross-field rejection test is fabricated for it
//!   here.
//! - **`proposeCandidateFact`'s LLM proposer** is NOT a runtime dependency for
//!   the gate: the gate is pure deterministic validation, so tests feed
//!   `CandidateFact` values directly (no live LLM).
//! - **`updateFact`** signature `(wikiId, factKey, {value, citations})`; the
//!   manual declaration still passes the gate (§4.3.2a.3).
//! - **§4.3.3 provenance (`trace`)** is deferred to §4.5: the `trace` attaches to
//!   `RagResult`, whose query path is a §4.5 surface not yet built (see the
//!   report). No trace accessor is fabricated now.

use std::sync::Arc;

use gnosis::{
    CandidateFact, CreateDocumentRequest, DocState, DocumentId, Edge, EdgeKind, Graph,
    ListFactsFilter, Node, NodeId, NodeKind, QueryAuditEntry, QueryAuditFilters, QueryMode,
    RagStore, Store, StoreError, UpdateFactRequest, WikiId,
};

// ---------------------------------------------------------------------------
// Helpers (mirroring the §4.2 suite)
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

/// Assert `s` matches exactly the ISO-8601 UTC shape the crate's `iso_now()`
/// emits (§4.1.1/§4.3.1): `YYYY-MM-DDTHH:MM:SS.nnnnnnnnnZ` (9 fractional digits
/// + trailing `Z`). Manual shape check (no `regex` dep in `Cargo.toml`).
///
/// DOC-REVIEW QUICK-PIN (docs/pending.md): the fact `updatedAt` format is
/// pinned, not just non-empty.
fn assert_iso8601_utc(s: &str) {
    let b = s.as_bytes();
    assert_eq!(
        b.len(),
        30,
        "ISO-8601 UTC must be 30 chars, got {s:?} (len {})",
        b.len()
    );
    for &i in &[0usize, 1, 2, 3, 5, 6, 8, 9] {
        assert!(b[i].is_ascii_digit(), "pos {i} of {s:?} must be a digit");
    }
    assert_eq!(b[4], b'-', "pos 4 of {s:?} must be '-'");
    assert_eq!(b[7], b'-', "pos 7 of {s:?} must be '-'");
    assert_eq!(b[10], b'T', "pos 10 of {s:?} must be 'T'");
    for &i in &[11usize, 12, 14, 15, 17, 18] {
        assert!(b[i].is_ascii_digit(), "pos {i} of {s:?} must be a digit");
    }
    assert_eq!(b[13], b':', "pos 13 of {s:?} must be ':'");
    assert_eq!(b[16], b':', "pos 16 of {s:?} must be ':'");
    assert_eq!(b[19], b'.', "pos 19 of {s:?} must be '.'");
    for (frac_i, &c) in b.iter().enumerate().take(29).skip(20) {
        assert!(c.is_ascii_digit(), "pos {frac_i} of {s:?} must be a digit");
    }
    assert_eq!(b[29], b'Z', "pos 29 of {s:?} must be 'Z' (UTC)");
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

/// Give `doc` a node graph named `names` and return the doc (updated). Facts are
/// created citing these nodes.
async fn seed_nodes(store: &Store, doc: &gnosis::Document, names: &[&str]) {
    apply_graph(store, doc, content_only_graph(&doc.document_id, names)).await;
}

// ---------------------------------------------------------------------------
// §4.3.1 / §4.3.2 — Fact CRUD + citations (re-derived surface; green today via
// the §4.2 `create_fact`/`get_fact` seeding mechanism — these are regression
// guards, not the red set).
// ---------------------------------------------------------------------------

/// State: a fact with ≥1 citation is stored; `factKey` is unique **per Wiki** —
/// the same key in a different Wiki is a distinct fact.
#[tokio::test]
async fn create_fact_stores_fact_with_citations_unique_per_wiki() {
    let store = Arc::new(Store::new());
    let w1 = new_wiki(&store, "w1").await;
    let w2 = new_wiki(&store, "w2").await;
    let doc1 = new_doc(&store, &w1, "f").await;
    let doc2 = new_doc(&store, &w2, "f").await;
    seed_nodes(&store, &doc1, &["n1"]).await;
    seed_nodes(&store, &doc2, &["m1"]).await;

    let f1 = store
        .create_fact(
            &w1,
            &doc1.document_id,
            "license",
            "MIT",
            &[(doc1.document_id.clone(), nid("n1"))],
        )
        .await
        .unwrap();
    assert_eq!(f1.fact_key, "license");
    assert_eq!(f1.value, "MIT");
    assert_eq!(f1.citations, vec![(doc1.document_id.clone(), nid("n1"))]);

    // Same factKey in another Wiki is a distinct fact (uniqueness is per-wiki).
    let f2 = store
        .create_fact(
            &w2,
            &doc2.document_id,
            "license",
            "MIT",
            &[(doc2.document_id.clone(), nid("m1"))],
        )
        .await
        .unwrap();
    assert_eq!(f2.citations, vec![(doc2.document_id.clone(), nid("m1"))]);

    let got1 = store.get_fact(&w1, "license").await.unwrap();
    let got2 = store.get_fact(&w2, "license").await.unwrap();
    assert_eq!(got1.document_id, doc1.document_id);
    assert_eq!(got2.document_id, doc2.document_id);
}

/// Fail-state: `createFact` with an empty `citations` set → `ValidationError`
/// ("fact requires at least one citation", §4.3.2 minimum-citation invariant).
#[tokio::test]
async fn create_fact_empty_citations_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "f").await;
    seed_nodes(&store, &doc, &["n1"]).await;
    let err = store
        .create_fact(&w, &doc.document_id, "license", "MIT", &[])
        .await
        .unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(ref m) if m.contains("at least one citation")),
        "minimum-citation invariant"
    );
}

/// Fail-state: a duplicate `factKey` within one Wiki → `ConflictError` (§4.3.1
/// key uniqueness).
#[tokio::test]
async fn create_fact_duplicate_fact_key_in_wiki_is_conflict_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "f").await;
    seed_nodes(&store, &doc, &["n1"]).await;
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
    let err = store
        .create_fact(
            &w,
            &doc.document_id,
            "license",
            "Apache",
            &[(doc.document_id.clone(), nid("n1"))],
        )
        .await
        .unwrap_err();
    assert_eq!(err, StoreError::ConflictError, "§4.3.1 duplicate factKey");
}

/// State: `citations` are deduped by first appearance (§4.3.2 grounding set).
#[tokio::test]
async fn create_fact_dedups_citations_by_first_appearance() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "f").await;
    seed_nodes(&store, &doc, &["n1", "n2"]).await;
    let fact = store
        .create_fact(
            &w,
            &doc.document_id,
            "license",
            "MIT",
            &[
                (doc.document_id.clone(), nid("n1")),
                (doc.document_id.clone(), nid("n2")),
                (doc.document_id.clone(), nid("n1")), // duplicate → removed
            ],
        )
        .await
        .unwrap();
    assert_eq!(
        fact.citations,
        vec![
            (doc.document_id.clone(), nid("n1")),
            (doc.document_id.clone(), nid("n2"))
        ],
        "citations deduped by first appearance, dupes removed"
    );
}

/// State: `getFact` returns the stored fact (§4.3.1 value type round-trip).
#[tokio::test]
async fn get_fact_returns_stored_fact() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "f").await;
    seed_nodes(&store, &doc, &["n1"]).await;
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

    let got = store.get_fact(&w, "license").await.unwrap();
    assert_eq!(got.fact_key, "license");
    assert_eq!(got.value, "MIT");
    assert_eq!(got.document_id, doc.document_id);
    assert_eq!(got.node_id, nid("fact-license"));
    assert!(!got.updated_at.is_empty(), "updatedAt is set (ISO-8601)");
    // DOC-REVIEW QUICK-PIN: a `create_fact`-produced `Fact`'s `updatedAt` is
    // ISO-8601 UTC (`YYYY-MM-DDTHH:MM:SS.nnnnnnnnnZ`), not just non-empty.
    assert_iso8601_utc(&got.updated_at);
    assert_eq!(got.citations, vec![(doc.document_id.clone(), nid("n1"))]);
}

// ---------------------------------------------------------------------------
// §4.5.4 / §4.3.1 — `listFacts` (NEW RED surface)
// ---------------------------------------------------------------------------

fn list_opts(
    state: Option<DocState>,
    page: Option<u64>,
    page_size: Option<u64>,
) -> ListFactsFilter {
    ListFactsFilter {
        state,
        page,
        page_size,
    }
}

/// State: `listFacts` paginates the facts table into `{items, total, page,
/// pageSize}`.
#[tokio::test]
async fn list_facts_paginates() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "facts").await;
    seed_nodes(&store, &doc, &["n1", "n2", "n3", "n4", "n5"]).await;
    for (i, k) in ["f1", "f2", "f3", "f4", "f5"].into_iter().enumerate() {
        store
            .create_fact(
                &w,
                &doc.document_id,
                k,
                "v",
                &[(doc.document_id.clone(), nid(format!("n{}", i + 1).as_str()))],
            )
            .await
            .unwrap();
    }

    let page2 = store
        .list_facts(&w, &list_opts(None, Some(2), Some(2)))
        .await
        .unwrap();
    assert_eq!(page2.total, 5);
    assert_eq!(page2.page, 2);
    assert_eq!(page2.page_size, 2);
    assert_eq!(
        page2.items.len(),
        2,
        "page 2 of pageSize 2 yields 2 facts from a 5-fact table"
    );
}

/// State: `listFacts` filters by `state` — per the contract decision, the state
/// of the fact's **containing Document** (`DocState`).
#[tokio::test]
async fn list_facts_filters_by_document_state() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;

    // A DRAFT document holding fact `fa`.
    let draft_doc = new_doc(&store, &w, "draft").await;
    seed_nodes(&store, &draft_doc, &["n1"]).await;
    store
        .create_fact(
            &w,
            &draft_doc.document_id,
            "fa",
            "v1",
            &[(draft_doc.document_id.clone(), nid("n1"))],
        )
        .await
        .unwrap();

    // A PUBLISHED document holding fact `fb`.
    let pub_doc = new_doc(&store, &w, "published").await;
    seed_nodes(&store, &pub_doc, &["m1"]).await;
    store.publish_document(&pub_doc.document_id).await.unwrap();
    store
        .create_fact(
            &w,
            &pub_doc.document_id,
            "fb",
            "v2",
            &[(pub_doc.document_id.clone(), nid("m1"))],
        )
        .await
        .unwrap();

    let published = store
        .list_facts(&w, &list_opts(Some(DocState::Published), None, None))
        .await
        .unwrap();
    let keys: Vec<&String> = published.items.iter().map(|f| &f.fact_key).collect();
    assert!(
        keys.iter().any(|k| k.as_str() == "fb"),
        "published doc's fact is listed for state=Published"
    );
    assert!(
        !keys.iter().any(|k| k.as_str() == "fa"),
        "draft doc's fact is excluded for state=Published"
    );
}

/// Fail-state: `page < 1` → `ValidationError` (FS-3).
#[tokio::test]
async fn list_facts_page_less_than_one_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let err = store
        .list_facts(&w, &list_opts(None, Some(0), Some(10)))
        .await
        .unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "page<1 is a ValidationError (FS-3): {err:?}"
    );
}

/// Fail-state: `pageSize < 1` → `ValidationError` (FS-3).
#[tokio::test]
async fn list_facts_page_size_less_than_one_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let err = store
        .list_facts(&w, &list_opts(None, Some(1), Some(0)))
        .await
        .unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "pageSize<1 is a ValidationError (FS-3): {err:?}"
    );
}

/// Fail-state: `pageSize > 100` → `ValidationError` (FS-3).
#[tokio::test]
async fn list_facts_page_size_over_one_hundred_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let err = store
        .list_facts(&w, &list_opts(None, Some(1), Some(101)))
        .await
        .unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "pageSize>100 is a ValidationError (FS-3): {err:?}"
    );
}

/// Fail-state: `listFacts` on an unknown wiki → `WikiNotFound` (FS-2).
#[tokio::test]
async fn list_facts_unknown_wiki_is_wiki_not_found() {
    let store = Arc::new(Store::new());
    let err = store
        .list_facts(&wiki("ghost"), &list_opts(None, None, None))
        .await
        .unwrap_err();
    assert_eq!(err, StoreError::WikiNotFound, "FS-2");
}

// ---------------------------------------------------------------------------
// §4.3.2 — `updateFact` (NEW RED surface)
// ---------------------------------------------------------------------------

/// State: `updateFact` updates the value + citations and refreshes `updatedAt`
/// (§4.3.2); a manual update still passes the deterministic gate.
#[tokio::test]
async fn update_fact_changes_value_and_citations() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "f").await;
    seed_nodes(&store, &doc, &["n1", "n2"]).await;
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
    let created = store.get_fact(&w, "license").await.unwrap();

    let updated = store
        .update_fact(
            &w,
            "license",
            &UpdateFactRequest {
                value: "Apache-2.0".to_string(),
                citations: vec![
                    (doc.document_id.clone(), nid("n1")),
                    (doc.document_id.clone(), nid("n2")),
                ],
            },
        )
        .await
        .unwrap();
    assert_eq!(updated.value, "Apache-2.0");
    assert_eq!(
        updated.citations,
        vec![
            (doc.document_id.clone(), nid("n1")),
            (doc.document_id.clone(), nid("n2"))
        ]
    );
    assert!(
        updated.updated_at >= created.updated_at,
        "updatedAt refreshes"
    );
    // DOC-REVIEW QUICK-PIN: an `update_fact`-produced `Fact`'s refreshed
    // `updatedAt` is ISO-8601 UTC (`YYYY-MM-DDTHH:MM:SS.nnnnnnnnnZ`).
    assert_iso8601_utc(&updated.updated_at);
    assert_eq!(updated.fact_key, "license");
}

/// Fail-state: `updateFact` with empty `citations` → `ValidationError`
/// ("fact requires at least one citation", §4.3.2 minimum-citation invariant).
#[tokio::test]
async fn update_fact_empty_citations_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "f").await;
    seed_nodes(&store, &doc, &["n1"]).await;
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
    let err = store
        .update_fact(
            &w,
            "license",
            &UpdateFactRequest {
                value: "Apache-2.0".to_string(),
                citations: vec![],
            },
        )
        .await
        .unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(ref m) if m.contains("at least one citation")),
        "updateFact retains the minimum-citation invariant"
    );
}

/// Fail-state: `updateFact` on an unknown `factKey` → `DocumentNotFound`.
#[tokio::test]
async fn update_fact_unknown_fact_is_document_not_found() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "f").await;
    seed_nodes(&store, &doc, &["n1"]).await;
    let err = store
        .update_fact(
            &w,
            "nonexistent",
            &UpdateFactRequest {
                value: "v".to_string(),
                citations: vec![(doc.document_id.clone(), nid("n1"))],
            },
        )
        .await
        .unwrap_err();
    assert_eq!(err, StoreError::DocumentNotFound, "unknown fact key");
}

// ---------------------------------------------------------------------------
// §4.3.2a — Candidate-fact pipeline + deterministic validation (NEW RED surface)
// ---------------------------------------------------------------------------

/// State: a valid candidate passes the deterministic gate AND commits — the
/// committed fact is stored (not just returned) and retrievable via `getFact`.
#[tokio::test]
async fn valid_candidate_passes_gate_and_commits() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "cand").await;
    seed_nodes(&store, &doc, &["n1"]).await;

    let outcome = store
        .propose_candidate_fact(
            &w,
            &CandidateFact {
                fact_key: "license".to_string(),
                value: "MIT".to_string(),
                citations: vec![(doc.document_id.clone(), nid("n1"))],
            },
        )
        .await
        .unwrap();
    assert!(outcome.accepted, "valid candidate is accepted");
    assert!(
        outcome.rejection.is_none(),
        "no rejection on an accepted candidate"
    );
    let fact = outcome.fact.expect("accepted candidate carries the fact");
    assert_eq!(fact.fact_key, "license");
    assert_eq!(fact.value, "MIT");

    // Committed, not merely returned: verifiable through the store.
    let persisted = store.get_fact(&w, "license").await.unwrap();
    assert_eq!(persisted.fact_key, "license");
    assert_eq!(persisted.value, "MIT");
}

/// Fail-state: a schema-conformance failure (empty `value`) is rejected with the
/// machine-actionable `{code, field, message}` AND is NOT committed (no partial
/// commit — §4.3.2a.2).
#[tokio::test]
async fn candidate_empty_value_rejected_with_structured_reason_and_not_committed() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "cand").await;
    seed_nodes(&store, &doc, &["n1"]).await;

    let outcome = store
        .propose_candidate_fact(
            &w,
            &CandidateFact {
                fact_key: "license".to_string(),
                value: "".to_string(), // schema: empty value
                citations: vec![(doc.document_id.clone(), nid("n1"))],
            },
        )
        .await
        .unwrap();
    assert!(
        !outcome.accepted,
        "empty-value candidate is rejected fail-closed"
    );
    assert!(outcome.fact.is_none(), "no fact on a rejected candidate");
    let rej = outcome
        .rejection
        .expect("machine-actionable {code, field, message} rejection");
    assert!(
        !rej.code.is_empty() && !rej.message.is_empty(),
        "structured reason"
    );
    assert_eq!(rej.field, "value", "rejection names the offending field");

    // Fail-closed: nothing was committed.
    assert!(
        store.get_fact(&w, "license").await.is_err(),
        "rejected candidate leaves no partial commit in the store"
    );
}

/// Fail-state: a schema-conformance failure (missing/empty `factKey`) is
/// rejected with the structured reason AND is NOT committed.
#[tokio::test]
async fn candidate_missing_fact_key_rejected_with_structured_reason_and_not_committed() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "cand").await;
    seed_nodes(&store, &doc, &["n1"]).await;

    let outcome = store
        .propose_candidate_fact(
            &w,
            &CandidateFact {
                fact_key: "".to_string(), // schema: missing factKey
                value: "MIT".to_string(),
                citations: vec![(doc.document_id.clone(), nid("n1"))],
            },
        )
        .await
        .unwrap();
    assert!(
        !outcome.accepted,
        "missing-factKey candidate is rejected fail-closed"
    );
    let rej = outcome.rejection.expect("machine-actionable rejection");
    assert_eq!(rej.field, "fact_key", "rejection names the factKey field");
    assert!(
        store.get_fact(&w, "").await.is_err(),
        "rejected candidate leaves no partial commit"
    );
}

/// Fail-state: a schema-conformance failure (empty `citations` —
/// minimum-citation invariant) is rejected with the structured reason AND is NOT
/// committed.
#[tokio::test]
async fn candidate_empty_citations_rejected_with_structured_reason_and_not_committed() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "cand").await;
    seed_nodes(&store, &doc, &["n1"]).await;

    let outcome = store
        .propose_candidate_fact(
            &w,
            &CandidateFact {
                fact_key: "license".to_string(),
                value: "MIT".to_string(),
                citations: vec![], // schema: empty citations (minimum-citation)
            },
        )
        .await
        .unwrap();
    assert!(
        !outcome.accepted,
        "empty-citations candidate is rejected fail-closed"
    );
    let rej = outcome.rejection.expect("machine-actionable rejection");
    assert_eq!(
        rej.field, "citations",
        "rejection names the citations field"
    );
    assert!(
        store.get_fact(&w, "license").await.is_err(),
        "rejected candidate leaves no partial commit"
    );
}

/// Fail-state (§4.3.2a.4): a candidate with a dangling citation (a citation that
/// does not resolve to a real node) → `ValidationError` ("citation does not
/// resolve") AND is NOT committed.
#[tokio::test]
async fn candidate_dangling_citation_is_validation_error_and_not_committed() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "cand").await;
    seed_nodes(&store, &doc, &["n1"]).await;

    // A citation to a node/document that does not exist.
    let err = store
        .propose_candidate_fact(
            &w,
            &CandidateFact {
                fact_key: "license".to_string(),
                value: "MIT".to_string(),
                citations: vec![(did("ghost-doc"), nid("ghost-node"))],
            },
        )
        .await
        .unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(ref m) if m.contains("citation does not resolve")),
        "dangling citation is a ValidationError 'citation does not resolve'"
    );
    assert!(
        store.get_fact(&w, "license").await.is_err(),
        "dangling-citation candidate leaves no partial commit"
    );
}

/// Fail-state (§4.3.2a.4): a candidate with a duplicate `factKey` →
/// `ConflictError` AND the pre-existing (manual) fact is untouched.
#[tokio::test]
async fn candidate_duplicate_fact_key_is_conflict_error_and_not_committed() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "cand").await;
    seed_nodes(&store, &doc, &["n1"]).await;
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

    let err = store
        .propose_candidate_fact(
            &w,
            &CandidateFact {
                fact_key: "license".to_string(), // duplicate
                value: "Apache-2.0".to_string(),
                citations: vec![(doc.document_id.clone(), nid("n1"))],
            },
        )
        .await
        .unwrap_err();
    assert_eq!(
        err,
        StoreError::ConflictError,
        "§4.3.2a.4 duplicate factKey"
    );

    // The pre-existing fact is authoritative and untouched.
    let existing = store.get_fact(&w, "license").await.unwrap();
    assert_eq!(
        existing.value, "MIT",
        "duplicate candidate did not overwrite"
    );
}

// ---------------------------------------------------------------------------
// §4.3.2a.3 / §4.2.8.4 — Manual-override precedence
// ---------------------------------------------------------------------------

/// State: a manually declared fact (`createFact`) is authoritative and is NEVER
/// overwritten by an automatic/candidate extraction — after a conflicting
/// automatic candidate is rejected/routed (ConflictError), the manual value
/// persists.
#[tokio::test]
async fn manual_fact_is_authoritative_and_not_overwritten_by_conflicting_candidate() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "cand").await;
    seed_nodes(&store, &doc, &["n1", "n2"]).await;

    // The manual declaration wins (§4.3.2a.3 / §4.2.8.4).
    store
        .create_fact(
            &w,
            &doc.document_id,
            "license",
            "MIT (manual)",
            &[(doc.document_id.clone(), nid("n1"))],
        )
        .await
        .unwrap();

    // An automatic/candidate extraction proposes a conflicting value for the
    // same key. It must be rejected (duplicate key → ConflictError) and the
    // manual value must persist verbatim.
    let res = store
        .propose_candidate_fact(
            &w,
            &CandidateFact {
                fact_key: "license".to_string(),
                value: "Auto-Extracted Value".to_string(),
                citations: vec![(doc.document_id.clone(), nid("n2"))],
            },
        )
        .await;
    assert!(
        res.is_err(),
        "conflicting automatic candidate is rejected, never overwrites the manual fact"
    );

    let persisted = store.get_fact(&w, "license").await.unwrap();
    assert_eq!(
        persisted.value, "MIT (manual)",
        "the manual declaration is authoritative and never overwritten (§4.2.8.4)"
    );
}

// ---------------------------------------------------------------------------
// §4.3.4 — Query audit log (NEW RED surface; return-type contract)
// ---------------------------------------------------------------------------

/// State/contract: `getQueryAuditLog()` returns the `[{query, filters, mode,
/// resultCount, timestamp, requester}]` shape. The **recording** is engine-side
/// (§4.3.4) and fed by the §4.5 query path, which is not yet built; this test
/// pins that the accessor is reachable and returns the recorded shape. It also
/// constructs one full `QueryAuditEntry` literal to pin the six fields at the
/// type level.
#[tokio::test]
async fn get_query_audit_log_returns_audit_entry_shape() {
    let _entry_shape: QueryAuditEntry = QueryAuditEntry {
        query: "what is the license?".to_string(),
        filters: Some(QueryAuditFilters {
            node_kind: Some(NodeKind::Fact),
            edge_type: None,
            target: None,
            state: None,
        }),
        mode: QueryMode::Graph,
        result_count: 3,
        timestamp: "2026-09-09T00:00:00.000000000Z".to_string(),
        requester: "gui-user-42".to_string(),
    };

    let store = Arc::new(Store::new());
    let log: Vec<QueryAuditEntry> = store
        .get_query_audit_log()
        .await
        .expect("the query audit-log accessor is reachable");
    assert!(
        log.is_empty(),
        "empty log on a fresh store (recording feeds in via the §4.5 query path)"
    );
}

// ---------------------------------------------------------------------------
// Concurrency contract (SHARDED-RWLOCK-STORE / ARC-SHARED-ENGINE)
// ---------------------------------------------------------------------------

/// Concurrency state: concurrently creating the same `factKey` in a Wiki —
/// exactly one wins, every other contender gets `ConflictError`, and reads see
/// the single committed fact. create_fact's compare-then-commit is atomic under
/// the `fact_store` write lock (§4.3.1 key uniqueness). **Green today** (this
/// path was landed by §4.2 as the mergeFacts seeding mechanism) — it is the
/// regression guard for the fact-key uniqueness invariant under contention.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_create_fact_unique_key_exactly_one_wins() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "conc").await;
    seed_nodes(&store, &doc, &["n1"]).await;
    let id = doc.document_id.clone();

    const N: usize = 8;
    let barrier = Arc::new(tokio::sync::Barrier::new(N));
    let handles: Vec<_> = (0..N)
        .map(|_| {
            let s = Arc::clone(&store);
            let w = w.clone();
            let id = id.clone();
            let b = Arc::clone(&barrier);
            tokio::spawn(async move {
                b.wait().await;
                s.create_fact(&w, &id, "license", "MIT", &[(id.clone(), nid("n1"))])
                    .await
            })
        })
        .collect();

    let mut ok = 0;
    let mut conflict = 0;
    for h in handles {
        match h.await.expect("concurrent create_fact task did not panic") {
            Ok(_) => ok += 1,
            Err(StoreError::ConflictError) => conflict += 1,
            Err(other) => panic!("unexpected error under contention: {other:?}"),
        }
    }
    assert_eq!(ok, 1, "exactly one concurrent create_fact wins");
    assert_eq!(conflict, N - 1, "every other contender gets ConflictError");
    let stored = store.get_fact(&w, "license").await.unwrap();
    assert_eq!(stored.value, "MIT", "reads see the single committed fact");
}

/// Concurrency state (GREEN): concurrently proposing the same candidate factKey —
/// the fail-closed gate must commit **exactly one** fact; every other contender
/// is rejected with `ConflictError` and the store has one atomic result. The
/// candidate gate is landed, so this is green (regression guard for the atomic
/// commit under contention).
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_candidate_same_key_commits_atomically_exactly_one() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "conc-cand").await;
    seed_nodes(&store, &doc, &["n1"]).await;
    let id = doc.document_id.clone();

    const N: usize = 8;
    let barrier = Arc::new(tokio::sync::Barrier::new(N));
    let handles: Vec<_> = (0..N)
        .map(|_| {
            let s = Arc::clone(&store);
            let w = w.clone();
            let id = id.clone();
            let b = Arc::clone(&barrier);
            tokio::spawn(async move {
                b.wait().await;
                s.propose_candidate_fact(
                    &w,
                    &CandidateFact {
                        fact_key: "license".to_string(),
                        value: "MIT".to_string(),
                        citations: vec![(id.clone(), nid("n1"))],
                    },
                )
                .await
            })
        })
        .collect();

    let mut accepted = 0;
    for h in handles {
        match h.await.expect("concurrent candidate task did not panic") {
            Ok(outcome) if outcome.accepted => accepted += 1,
            Ok(_) => {}
            Err(StoreError::ConflictError) => {}
            Err(other) => panic!("unexpected error under contention: {other:?}"),
        }
    }
    assert_eq!(
        accepted, 1,
        "the candidate gate commits exactly one fact for a contended factKey"
    );
    let stored = store.get_fact(&w, "license").await.unwrap();
    assert_eq!(stored.value, "MIT");
}

// ---------------------------------------------------------------------------
// Adversarial regression surface (RED on the current implementation)
// ---------------------------------------------------------------------------
//
// These tests pin spec-correct behavior that the current implementation does
// NOT yet enforce, surfaced by an adversarial review of §4.3/§4.4.5. They are
// written RED-first: they FAIL against the current impl and flip GREEN only when
// the Implementer lands the matching fix. The section header comment of this
// file (above) lists each finding it pins (H1 / H2 / H4 / H5).

// ---------------------------------------------------------------------------
// H1 (HIGH) — Delete must not leave a fact with a dangling citation
// ---------------------------------------------------------------------------
//
// §4.3.2 "every citation resolves to a real node" + §4.4.5 reference-integrity:
// deleting a document that a *committed fact* cites must fail with
// `DocumentInUse`. The current `delete_document` gate scans edges only
// (link/embed/crosslink target), not `fact_store`, so the delete succeeds and
// the committed fact dangles. RED today: the delete returns `Ok`.

/// Fail-state (H1): `deleteDocument` on a document whose node a committed fact
/// cites → `DocumentInUse` (the fact's grounding must not be deleted out from
/// under it).
#[tokio::test]
async fn delete_document_is_document_in_use_when_committed_fact_cites_it() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "held").await;
    seed_nodes(&store, &doc, &["n1"]).await;

    // A committed fact cites node (D, n1) of this document.
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

    let err = store.delete_document(&doc.document_id).await.unwrap_err();
    assert_eq!(
        err,
        StoreError::DocumentInUse,
        "a document that a committed fact cites is DocumentInUse (no dangling citation)"
    );
}

// ---------------------------------------------------------------------------
// H2 — `updateFact` must reject an empty / whitespace value
// ---------------------------------------------------------------------------
//
// §4.3.2a.1 schema conformance requires a **non-empty** `value`; an empty or
// whitespace-only value is a schema violation. The current `update_fact` applies
// `value` verbatim with no non-empty check. RED today: the empty/whitespace value
// is applied and the update returns `Ok`.

/// Fail-state (H2): `updateFact` with an empty `value` → `ValidationError`.
#[tokio::test]
async fn update_fact_empty_value_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "f").await;
    seed_nodes(&store, &doc, &["n1"]).await;
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

    let err = store
        .update_fact(
            &w,
            "license",
            &UpdateFactRequest {
                value: "".to_string(), // schema: empty value
                citations: vec![(doc.document_id.clone(), nid("n1"))],
            },
        )
        .await
        .unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "empty updateFact value is a ValidationError: {err:?}"
    );
}

/// Fail-state (H2): `updateFact` with a whitespace-only `value` → `ValidationError`.
#[tokio::test]
async fn update_fact_whitespace_value_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "f").await;
    seed_nodes(&store, &doc, &["n1"]).await;
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

    let err = store
        .update_fact(
            &w,
            "license",
            &UpdateFactRequest {
                value: "   ".to_string(), // schema: whitespace-only value
                citations: vec![(doc.document_id.clone(), nid("n1"))],
            },
        )
        .await
        .unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "whitespace-only updateFact value is a ValidationError: {err:?}"
    );
}

// ---------------------------------------------------------------------------
// H4 — Whitespace-only `factKey`/`value` must fail schema conformance
// ---------------------------------------------------------------------------
//
// §4.3.2a.1 requires a **valid** (well-formed) `factKey` and non-empty `value`;
// `.is_empty()` accepts whitespace-only strings as if they were meaningful.
// `create_fact` and `propose_candidate_fact` both use `.is_empty()`, so a
// `"   "` key/value is treated as valid and committed. RED today: these commits
// succeed (create → `Ok(..)`, candidate → `accepted:true`).

/// Fail-state (H4): `createFact` with a whitespace-only `value` →
/// `ValidationError` (not committed).
#[tokio::test]
async fn create_fact_whitespace_value_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "f").await;
    seed_nodes(&store, &doc, &["n1"]).await;

    let err = store
        .create_fact(
            &w,
            &doc.document_id,
            "license",
            "   ", // schema: whitespace-only value
            &[(doc.document_id.clone(), nid("n1"))],
        )
        .await
        .unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "whitespace-only createFact value is a ValidationError: {err:?}"
    );
}

/// Fail-state (H4): `createFact` with a whitespace-only `factKey` →
/// `ValidationError` (not committed).
#[tokio::test]
async fn create_fact_whitespace_fact_key_is_validation_error() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "f").await;
    seed_nodes(&store, &doc, &["n1"]).await;

    let err = store
        .create_fact(
            &w,
            &doc.document_id,
            "   ", // schema: whitespace-only factKey
            "MIT",
            &[(doc.document_id.clone(), nid("n1"))],
        )
        .await
        .unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(_)),
        "whitespace-only createFact factKey is a ValidationError: {err:?}"
    );
}

/// Fail-state (H4): `proposeCandidateFact` with a whitespace-only `factKey` is
/// rejected with a structured reason AND not committed.
#[tokio::test]
async fn candidate_whitespace_fact_key_rejected_and_not_committed() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "cand").await;
    seed_nodes(&store, &doc, &["n1"]).await;

    let outcome = store
        .propose_candidate_fact(
            &w,
            &CandidateFact {
                fact_key: "   ".to_string(), // schema: whitespace-only factKey
                value: "MIT".to_string(),
                citations: vec![(doc.document_id.clone(), nid("n1"))],
            },
        )
        .await
        .unwrap();
    assert!(
        !outcome.accepted,
        "whitespace-only factKey candidate is rejected fail-closed"
    );
    let rej = outcome
        .rejection
        .expect("machine-actionable rejection for whitespace factKey");
    assert_eq!(rej.field, "fact_key", "rejection names the factKey field");
    assert!(
        store.get_fact(&w, "   ").await.is_err(),
        "whitespace-factKey candidate leaves no partial commit"
    );
}

/// Fail-state (H4): `proposeCandidateFact` with a whitespace-only `value` is
/// rejected with a structured reason AND not committed.
#[tokio::test]
async fn candidate_whitespace_value_rejected_and_not_committed() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "cand").await;
    seed_nodes(&store, &doc, &["n1"]).await;

    let outcome = store
        .propose_candidate_fact(
            &w,
            &CandidateFact {
                fact_key: "license".to_string(),
                value: "   ".to_string(), // schema: whitespace-only value
                citations: vec![(doc.document_id.clone(), nid("n1"))],
            },
        )
        .await
        .unwrap();
    assert!(
        !outcome.accepted,
        "whitespace-only value candidate is rejected fail-closed"
    );
    let rej = outcome
        .rejection
        .expect("machine-actionable rejection for whitespace value");
    assert_eq!(rej.field, "value", "rejection names the value field");
    assert!(
        store.get_fact(&w, "license").await.is_err(),
        "whitespace-value candidate leaves no partial commit"
    );
}

// ---------------------------------------------------------------------------
// H5 — Unknown wiki → `WikiNotFound` on the whole fact surface (FS-2)
// ---------------------------------------------------------------------------
//
// FS-2 (§4.1.3/§4.5.4) pins `WikiNotFound` for an unknown `wikiId`; it must be
// uniform across the fact surface. The current `create_fact`/`propose_candidate_fact`
// never check the wiki (they commit into a ghost-wiki bucket — `Ok` /
// `accepted:true`), `get_fact` returns `ValidationError("unknown wiki")` (not
// `WikiNotFound`), and `update_fact` returns `DocumentNotFound`. RED today: every
// test asserts the consistent `WikiNotFound`.

/// Fail-state (H5): `createFact` on an unknown wiki → `WikiNotFound`.
#[tokio::test]
async fn create_fact_unknown_wiki_is_wiki_not_found() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "f").await;
    seed_nodes(&store, &doc, &["n1"]).await;

    let err = store
        .create_fact(
            &wiki("ghost"),
            &doc.document_id,
            "license",
            "MIT",
            &[(doc.document_id.clone(), nid("n1"))],
        )
        .await
        .unwrap_err();
    assert_eq!(
        err,
        StoreError::WikiNotFound,
        "FS-2: createFact on unknown wiki"
    );
}

/// Fail-state (H5): `getFact` on an unknown wiki → `WikiNotFound`.
#[tokio::test]
async fn get_fact_unknown_wiki_is_wiki_not_found() {
    let store = Arc::new(Store::new());
    let err = store.get_fact(&wiki("ghost"), "license").await.unwrap_err();
    assert_eq!(
        err,
        StoreError::WikiNotFound,
        "FS-2: getFact on unknown wiki"
    );
}

/// Fail-state (H5): `updateFact` on an unknown wiki → `WikiNotFound`.
#[tokio::test]
async fn update_fact_unknown_wiki_is_wiki_not_found() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "f").await;
    seed_nodes(&store, &doc, &["n1"]).await;

    let err = store
        .update_fact(
            &wiki("ghost"),
            "license",
            &UpdateFactRequest {
                value: "MIT".to_string(),
                citations: vec![(doc.document_id.clone(), nid("n1"))],
            },
        )
        .await
        .unwrap_err();
    assert_eq!(
        err,
        StoreError::WikiNotFound,
        "FS-2: updateFact on unknown wiki"
    );
}

/// Fail-state (H5): `proposeCandidateFact` on an unknown wiki → `WikiNotFound`.
#[tokio::test]
async fn propose_candidate_fact_unknown_wiki_is_wiki_not_found() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "cand").await;
    seed_nodes(&store, &doc, &["n1"]).await;

    let err = store
        .propose_candidate_fact(
            &wiki("ghost"),
            &CandidateFact {
                fact_key: "license".to_string(),
                value: "MIT".to_string(),
                citations: vec![(doc.document_id.clone(), nid("n1"))],
            },
        )
        .await
        .unwrap_err();
    assert_eq!(
        err,
        StoreError::WikiNotFound,
        "FS-2: proposeCandidateFact on unknown wiki"
    );
}

// ---------------------------------------------------------------------------
// H1 core TOCTOU guard — delete + propose ordering (sequential proxy)
// ---------------------------------------------------------------------------
//
// The true concurrent delete+propose interleaving (a candidate whose cited node
// is deleted in another task between grounding and commit) cannot be forced
// deterministically here, so this is the **sequential proxy**: delete the cited
// document first (no committed fact yet, so the #1 delete gate does not block),
// then propose a candidate citing its node. The gate must NOT commit a dangling
// fact → `ValidationError("citation does not resolve")`, and `getFact` proves
// nothing was committed. Already GREEN today (the candidate dangling-citation
// guard is landed) — regression guard for the ordering-dependent invariant.

#[tokio::test]
async fn candidate_citing_a_deleted_document_is_not_committed() {
    let store = Arc::new(Store::new());
    let w = new_wiki(&store, "w").await;
    let doc = new_doc(&store, &w, "probe").await;
    seed_nodes(&store, &doc, &["n1"]).await;

    // Delete the cited document before any fact is recorded (the #1 delete gate
    // only fires for a *committed* citation, so this delete succeeds).
    store.delete_document(&doc.document_id).await.unwrap();

    let err = store
        .propose_candidate_fact(
            &w,
            &CandidateFact {
                fact_key: "license".to_string(),
                value: "MIT".to_string(),
                citations: vec![(doc.document_id.clone(), nid("n1"))],
            },
        )
        .await
        .unwrap_err();
    assert!(
        matches!(err, StoreError::ValidationError(ref m) if m.contains("citation does not resolve")),
        "a candidate citing a deleted document is not committed"
    );
    assert!(
        store.get_fact(&w, "license").await.is_err(),
        "nothing was committed for the deleted-citation candidate"
    );
}
