//! Document store (§4.1) — the persistence layer.
//!
//! This module defines the **public API surface** of the §4.1 document store
//! exactly as the TestWriter derives it from the canonical behavior contract
//! (`docs/specs/gnosis.md`) plus the concurrency design plan
//! (`docs/research/gnosis-data-structures-concurrency-plan.md`).
//!
//! The **real persistence logic is intentionally NOT implemented here** — the
//! `Store` implementation carries only `unimplemented!()` stub bodies. The
//! Implementer lands the §4.1.3 operations, the §4.1.4 optimistic-concurrency
//! guard (atomic compare-base-revision → apply → bump under the shard write
//! lock, per SHARDED-RWLOCK-STORE), and the §4.4 publish/delete gates to make
//! the red tests go green.
//!
//! ## Concurrency contract (from the plan + `docs/decisions.md`)
//!
//! - **SHARDED-RWLOCK-STORE** — authoritative store is a sharded `RwLock` map
//!   keyed by `(wikiId, documentId)`. Reads take per-shard `read()`; a
//!   document's mutation (including the §4.1.4 optimistic-concurrency guard)
//!   takes that shard's `write()`. No global store lock.
//! - **ARC-SHARED-ENGINE** — the store is shared as `Arc<Store>` (or the engine
//!   as `Arc<Gnosis>`) across tokio tasks. The `RagStore` trait methods below
//!   return `Send` futures so they can be `tokio::spawn`-ed.
//!
//! The trait is a **concrete (non-`dyn`) seam**: it is consumed as `Arc<Store>`,
//! which is exactly what the plan's engine shape (`store: Arc<Store>`) requires.

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::future::Future;

// ---------------------------------------------------------------------------
// §4.1.1 — Document model / §4.1.2 — Wiki model
// ---------------------------------------------------------------------------

/// A stable, globally-unique document identity (UUID v4). Immutable once
/// created; never reused (§4.1.1).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocumentId(pub String);

impl fmt::Display for DocumentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A Wiki identity (UUID v4) (§4.1.2).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WikiId(pub String);

impl fmt::Display for WikiId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A node identity within a document (§4.1.1 / §4.2.1).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub String);

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The document state machine (§4.1.1): `DRAFT → PUBLISHED → ARCHIVED`.
///
/// Legal transitions: `DRAFT→PUBLISHED`, `PUBLISHED→DRAFT` (unpublish),
/// `DRAFT→ARCHIVED`, `PUBLISHED→ARCHIVED`. `ARCHIVED` is terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DocState {
    /// Draft — the initial state of a newly created document.
    Draft,
    /// Published — the precondition for the Astral push.
    Published,
    /// Archived — terminal; no further transitions.
    Archived,
}

impl DocState {
    /// Whether a transition from `self` to `to` is legal per §4.1.1.
    pub fn can_transition_to(self, to: DocState) -> bool {
        use DocState::*;
        match self {
            Draft => matches!(to, Published | Archived),
            Published => matches!(to, Draft | Archived),
            Archived => false,
        }
    }
}

/// A Provident graph node kind (§4.2.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeKind {
    /// A normal authored node (text, heading, list, …).
    Content,
    /// A single source-of-truth fact node.
    Fact,
    /// A node that points at a fact (or another node).
    Reference,
}

/// A Provident graph edge kind (§4.2.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeKind {
    /// Navigational reference edge (target pointer only).
    Link,
    /// Shared data embed edge (snapshot + pointer).
    Embed,
    /// Cross-wiki reference edge.
    Crosslink,
    /// The document's entry edge (root → first node); exactly one per document.
    DocHead,
    /// Sequential ordering edge (section → next section).
    NextSection,
    /// The document's terminal edge (→ end sentinel); exactly one per document.
    DocEnd,
    /// Containment edge (parent node → child node).
    DocChild,
    /// A `relation` triple edge owned by the graph.
    Relation,
}

/// Reference edge state (§4.4.2) — the publish-gate inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReferenceState {
    /// An `embed` whose snapshot equals the canonical value.
    Fresh,
    /// An `embed` whose snapshot differs from the canonical value.
    Stale,
    /// A `link` whose target exists and is not archived.
    Resolved,
    /// A `link` whose target is missing or archived.
    Broken,
}

/// A Provident graph node (§4.1.1, §4.2.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    pub document_id: DocumentId,
    pub node_id: NodeId,
    pub kind: NodeKind,
    /// Authored text (content) or canonical value (fact).
    pub value: Option<String>,
    /// Stable fact key, unique within a Wiki (fact nodes).
    pub fact_key: Option<String>,
    /// Reference target `(target documentId, target nodeId)` (reference nodes).
    pub target: Option<(DocumentId, NodeId)>,
}

/// A Provident graph edge (§4.1.1, §4.2.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edge {
    pub source: (DocumentId, NodeId),
    pub target: (DocumentId, NodeId),
    pub kind: EdgeKind,
    /// Reference state for `link`/`embed`/`crosslink` edges (§4.4.2, §4.2.3).
    pub state: Option<ReferenceState>,
    /// Whether the reference target lives in another Wiki.
    pub cross_wiki: bool,
}

/// The Provident graph (a Document's body): nodes + edges (§4.1.1, §4.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

/// A Document — the atomic unit of authoring (§4.1.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Document {
    pub document_id: DocumentId,
    pub wiki_id: WikiId,
    /// Monotonically increasing; `0` is the initial, bumped on every committed change.
    pub revision: u64,
    pub state: DocState,
    /// The document's body (the Provident graph).
    pub graph: Graph,
    pub title: String,
    /// ISO-8601 UTC.
    pub created_at: String,
    /// ISO-8601 UTC.
    pub updated_at: String,
    pub tags: Vec<String>,
    pub author: Option<String>,
}

/// A Wiki — a named collection of Documents (§4.1.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wiki {
    pub wiki_id: WikiId,
    pub name: String,
}

// ---------------------------------------------------------------------------
// §4.1.3 request / response shapes
// ---------------------------------------------------------------------------

/// `createDocument(wikiId, {title, tags?, author?})` input (§4.1.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateDocumentRequest {
    pub title: String,
    pub tags: Option<Vec<String>>,
    pub author: Option<String>,
}

/// `updateDocument(documentId, {graph, title?, tags?})` input, plus the caller's
/// §4.1.4 base revision for optimistic concurrency.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateDocumentRequest {
    /// The `revision` the caller read; a stale base is rejected with
    /// `ConflictError` (§4.1.4).
    pub base_revision: u64,
    pub graph: Graph,
    pub title: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// `listDocuments(wikiId, {state?, tag?, page?, pageSize?})` filters (§4.1.3).
///
/// A `None` field means "no filter" (all states / any tag). `page` and
/// `pageSize` are 1-based and bounded; see FS-3 for validation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListDocumentsFilter {
    pub state: Option<DocState>,
    pub tag: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

/// A lightweight `Document` summary for list responses (§4.1.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentSummary {
    pub document_id: DocumentId,
    pub wiki_id: WikiId,
    pub title: String,
    pub state: DocState,
    pub revision: u64,
    pub updated_at: String,
}

/// `listDocuments` return shape `{items, total, page, pageSize}` (§4.1.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentList {
    pub items: Vec<DocumentSummary>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

/// A single recorded mutation in the store's append-only **MutationJournal**
/// (WRITER-ACTOR-JOURNAL). Each entry captures the monotonic `seq`, the operation
/// name, the `base_revision` the mutation applied from, and the commit timestamp.
/// The current `epoch` / journal length feed §4.2–§4.4 layers' index-rebuild and
/// audit feeds. §4.1 keeps it minimal: the single-writer-under-shard-lock path
/// appends while inside its mutation critical section (no tokio actor yet).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalEntry {
    /// Monotonic, append-order sequence number (equals the epoch at commit).
    pub seq: u64,
    /// Operation name, e.g. `"create_document"`.
    pub op: &'static str,
    /// The document `revision` the mutation applied from (`0` for creates/wikis).
    pub base_revision: u64,
    /// Commit timestamp (ISO-8601 UTC).
    pub timestamp: String,
}

// ---------------------------------------------------------------------------
// Store errors — mirrors the §4.1.3 / §6 fail-states
// ---------------------------------------------------------------------------

/// The store-level fail-states derived from §4.1.3, §4.1.4 and §6.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    /// FS-1 — unknown `documentId`.
    DocumentNotFound,
    /// FS-2 — unknown `wikiId`.
    WikiNotFound,
    /// FS-3 — validation failure (empty/overlong title/name, invalid graph,
    /// out-of-range pagination, …).
    ValidationError(String),
    /// FS-4 — optimistic-concurrency stale base revision (HTTP 409 / MCP error).
    ConflictError,
    /// FS-5 — delete of a document that other documents reference.
    DocumentInUse,
    /// FS-6 — illegal state transition (e.g. unpublish of a non-PUBLISHED doc,
    /// archive of an already-ARCHIVED doc).
    InvalidState,
    /// FS-7 — publish of a document with a `BROKEN` link or an unresynced
    /// `STALE` embed.
    UnresolvedReference,
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use StoreError::*;
        let msg = match self {
            DocumentNotFound => "document not found",
            WikiNotFound => "wiki not found",
            ValidationError(m) => m.as_str(),
            ConflictError => "optimistic-concurrency conflict: stale base revision",
            DocumentInUse => "document is in use (referenced by another document)",
            InvalidState => "illegal state transition",
            UnresolvedReference => "document contains unresolved references",
        };
        write!(f, "{msg}")
    }
}

impl Error for StoreError {}

// ---------------------------------------------------------------------------
// §4.1.5 — the RagStore (persistence seam)
// ---------------------------------------------------------------------------

/// The `RagStore` persistence seam (§4.1.5). The Astrographer shell proxies
/// these operations over the IPC/HTTP boundary. Operation signatures, return
/// shapes and fail-states follow §4.1.3/§4.1.4 exactly.
///
/// The trait is `Send + Sync` and every method returns a `Send` future, so an
/// implementation can be shared as `Arc<Store>` across tokio tasks
/// (ARC-SHARED-ENGINE) and `tokio::spawn`-ed for concurrent reads/writes.
pub trait RagStore: Send + Sync {
    /// `createDocument(wikiId, {title, tags?, author?}) → Document`.
    /// Failures: `WikiNotFound`, `ValidationError` (empty/overlong title).
    fn create_document(
        &self,
        wiki_id: &WikiId,
        request: CreateDocumentRequest,
    ) -> impl Future<Output = Result<Document, StoreError>> + Send;

    /// `getDocument(documentId) → Document`. Failures: `DocumentNotFound`.
    fn get_document(
        &self,
        document_id: &DocumentId,
    ) -> impl Future<Output = Result<Document, StoreError>> + Send;

    /// `updateDocument(documentId, {graph, title?, tags?}) → Document` at
    /// `revision + 1`. Failures: `DocumentNotFound`, `ValidationError` (invalid
    /// graph), `ConflictError` (stale base revision, §4.1.4).
    fn update_document(
        &self,
        document_id: &DocumentId,
        request: UpdateDocumentRequest,
    ) -> impl Future<Output = Result<Document, StoreError>> + Send;

    /// `deleteDocument(documentId) → void`. Failures: `DocumentNotFound`,
    /// `DocumentInUse` (another doc references it, §4.4.5).
    fn delete_document(
        &self,
        document_id: &DocumentId,
    ) -> impl Future<Output = Result<(), StoreError>> + Send;

    /// `publishDocument(documentId) → Document` in state `PUBLISHED`. Failures:
    /// `DocumentNotFound`, `UnresolvedReference` (§4.4.3 publish gate).
    fn publish_document(
        &self,
        document_id: &DocumentId,
    ) -> impl Future<Output = Result<Document, StoreError>> + Send;

    /// `unpublishDocument(documentId) → Document` in state `DRAFT`. Failures:
    /// `DocumentNotFound`, `InvalidState` (not `PUBLISHED`).
    fn unpublish_document(
        &self,
        document_id: &DocumentId,
    ) -> impl Future<Output = Result<Document, StoreError>> + Send;

    /// `archiveDocument(documentId) → Document` in state `ARCHIVED`. Failures:
    /// `DocumentNotFound`, `InvalidState` (already `ARCHIVED`).
    fn archive_document(
        &self,
        document_id: &DocumentId,
    ) -> impl Future<Output = Result<Document, StoreError>> + Send;

    /// `listDocuments(wikiId, {state?, tag?, page?, pageSize?}) →
    /// {items, total, page, pageSize}`. Failures: `WikiNotFound`,
    /// `ValidationError` (page<1, pageSize<1, pageSize>100).
    fn list_documents(
        &self,
        wiki_id: &WikiId,
        filter: &ListDocumentsFilter,
    ) -> impl Future<Output = Result<DocumentList, StoreError>> + Send;

    /// `createWiki({name}) → Wiki`. Failures: `ValidationError` (empty/overlong name).
    fn create_wiki(&self, name: &str) -> impl Future<Output = Result<Wiki, StoreError>> + Send;

    /// `getWiki(wikiId) → Wiki`. Failures: `WikiNotFound`.
    fn get_wiki(&self, wiki_id: &WikiId) -> impl Future<Output = Result<Wiki, StoreError>> + Send;

    /// `listWikis() → Wiki[]`.
    fn list_wikis(&self) -> impl Future<Output = Result<Vec<Wiki>, StoreError>> + Send;
}

// ---------------------------------------------------------------------------
// Stub implementation — the RED state.
//
// All §4.1.3 operations are `unimplemented!()`: the crate compiles, every test
// loads, and every behavior assertion fails at runtime. The Implementer replaces
// these bodies with the real §4.1 logic (sharded RwLock store, optimistic
// concurrency under the shard write lock, publish gate, delete integrity gate,
// pagination + validation) — the red tests are the executable contract.
// ---------------------------------------------------------------------------

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Number of document shards — fixed at boot (SHARDED-RWLOCK-STORE).
const SHARD_COUNT: usize = 64;

/// One shard owns a slice of documents (keyed by global `DocumentId`).
#[derive(Debug, Default)]
struct StoreShard {
    /// Authoritative documents. Stored as `Arc<Document>` so reads clone a cheap
    /// ref and keep reading the immutable document even after the shard swaps it.
    docs: HashMap<DocumentId, Arc<Document>>,
}

/// The authoritative document store (§4.1). Shared as `Arc<Store>` across tokio
/// tasks (ARC-SHARED-ENGINE). A document's shard `write()` guards its mutation
/// critical section (compare-base-revision → apply → bump, §4.1.4). No global
/// store lock — reads take per-shard `read()` and proceed concurrently.
#[derive(Debug)]
pub struct Store {
    /// Fixed-size lock-striped document shards.
    shards: Box<[RwLock<StoreShard>]>,
    /// The set of explicitly created wikis (§4.1.2).
    wikis: RwLock<HashMap<WikiId, Wiki>>,
    /// Monotonic id source for new documents/wikis.
    next_id: AtomicU64,
    /// Store-wide **reference-integrity** lock (DELETE TOCTOU fix, §4.4.5).
    ///
    /// `delete_document` takes it as `write()` for its whole scan+remove so the
    /// reference scan and the removal are atomic against concurrency referrer
    /// adds. Every mutation that can add a `link`/`embed`/`crosslink` edge
    /// (`create_document`, `update_document`) takes it as `read()` around its
    /// edge-writing region. Always acquired **before** any shard lock and always
    /// in the same order, so it serializes only reference-edge writes vs delete —
    /// never a global store lock on the read/hot path.
    reference_lock: RwLock<()>,
    /// Append-only **MutationJournal** (WRITER-ACTOR-JOURNAL).
    journal: RwLock<Vec<JournalEntry>>,
    /// Monotonic epoch — the latest journal `seq`, bumped on every committed
    /// mutation. Drives §4.2–§4.4 index-rebuild/audit feeds.
    epoch: AtomicU64,
}

impl Store {
    /// Create an empty store with a fixed number of shards.
    pub fn new() -> Self {
        let shards = (0..SHARD_COUNT)
            .map(|_| RwLock::new(StoreShard::default()))
            .collect();
        Store {
            shards,
            wikis: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(0),
            reference_lock: RwLock::new(()),
            journal: RwLock::new(Vec::new()),
            epoch: AtomicU64::new(0),
        }
    }

    /// The current journal epoch — the latest committed mutation's `seq` (0 when
    /// the store is fresh). Read-only; safe to poll from §4.2–§4.4 layers.
    pub fn epoch(&self) -> u64 {
        self.epoch.load(Ordering::Acquire)
    }

    /// Number of entries currently recorded in the append-only mutation journal.
    pub fn journal_len(&self) -> usize {
        self.journal.read().unwrap().len()
    }

    /// Append a committed mutation to the journal and bump the epoch. Call from
    /// inside a mutation's critical section; `base_revision` is the document
    /// `revision` the mutation applied from (`0` for creates / wiki creation).
    fn append_journal(&self, op: &'static str, base_revision: u64) {
        // epoch doubles as the monotonic seq source: fetch_add yields the old
        // value, so seq = old + 1 makes the epoch equal the entry's seq.
        let seq = self.epoch.fetch_add(1, Ordering::Relaxed) + 1;
        let mut journal = self.journal.write().unwrap();
        journal.push(JournalEntry {
            seq,
            op,
            base_revision,
            timestamp: iso_now(),
        });
    }

    /// Stable shard index for a document (lock-striping, no global lock).
    fn shard_for(&self, id: &DocumentId) -> &RwLock<StoreShard> {
        let h =
            id.0.bytes()
                .fold(0u64, |a, b| a.wrapping_mul(31).wrapping_add(b as u64));
        &self.shards[(h % self.shards.len() as u64) as usize]
    }
}

impl Default for Store {
    fn default() -> Self {
        Store::new()
    }
}

impl RagStore for Store {
    async fn create_document(
        &self,
        wiki_id: &WikiId,
        request: CreateDocumentRequest,
    ) -> Result<Document, StoreError> {
        // FS-2 (§4.1.3): createDocument on an unknown `wikiId` → `WikiNotFound`.
        if !self.wikis.read().unwrap().contains_key(wiki_id) {
            return Err(StoreError::WikiNotFound);
        }
        let len = request.title.chars().count();
        if request.title.is_empty() || len > 200 {
            return Err(StoreError::ValidationError(
                "document title must be non-empty and at most 200 chars".into(),
            ));
        }
        let now = iso_now();
        let id = DocumentId(format!(
            "doc-{}",
            self.next_id.fetch_add(1, Ordering::Relaxed)
        ));
        let doc = Document {
            document_id: id.clone(),
            wiki_id: wiki_id.clone(),
            revision: 0,
            state: DocState::Draft,
            graph: Graph {
                nodes: vec![],
                edges: vec![],
            },
            title: request.title,
            created_at: now.clone(),
            updated_at: now,
            tags: request.tags.unwrap_or_default(),
            author: request.author,
        };
        // §4.4.5 delete TOCTOU: creates hold the integrity read lock while
        // adding their (edge-free) document, so delete can never scan a state
        // that races a reference-edge add. Acquired before the shard lock.
        let _integrity = self.reference_lock.read().unwrap();
        let mut guard = self.shard_for(&id).write().unwrap();
        guard.docs.insert(id, Arc::new(doc.clone()));
        self.append_journal("create_document", 0);
        Ok(doc)
    }

    async fn get_document(&self, document_id: &DocumentId) -> Result<Document, StoreError> {
        let guard = self.shard_for(document_id).read().unwrap();
        guard
            .docs
            .get(document_id)
            .map(|d| (**d).clone())
            .ok_or(StoreError::DocumentNotFound)
    }

    async fn update_document(
        &self,
        document_id: &DocumentId,
        request: UpdateDocumentRequest,
    ) -> Result<Document, StoreError> {
        // §4.4.5 delete TOCTOU: update can write `link`/`embed`/`crosslink` edges,
        // so it holds the integrity read lock for its whole edge-writing region.
        // Acquired before the shard lock (consistent ordering → no deadlock).
        let _integrity = self.reference_lock.read().unwrap();
        let mut guard = self.shard_for(document_id).write().unwrap();
        let current = guard
            .docs
            .get(document_id)
            .ok_or(StoreError::DocumentNotFound)?;
        // §4.1.4 optimistic-concurrency guard — atomic under the shard write lock.
        if current.revision != request.base_revision {
            return Err(StoreError::ConflictError);
        }
        if !valid_provident_graph(&request.graph) {
            return Err(StoreError::ValidationError(
                "graph must be a valid Provident graph (exactly one doc-head and one doc-end)"
                    .into(),
            ));
        }
        let updated = Document {
            document_id: current.document_id.clone(),
            wiki_id: current.wiki_id.clone(),
            revision: current.revision + 1,
            state: current.state,
            graph: request.graph,
            title: request.title.unwrap_or_else(|| current.title.clone()),
            created_at: current.created_at.clone(),
            updated_at: iso_now(),
            tags: request.tags.unwrap_or_else(|| current.tags.clone()),
            author: current.author.clone(),
        };
        guard
            .docs
            .insert(document_id.clone(), Arc::new(updated.clone()));
        self.append_journal("update_document", request.base_revision);
        Ok(updated)
    }

    async fn delete_document(&self, document_id: &DocumentId) -> Result<(), StoreError> {
        // §4.4.5 reference-integrity gate + DELETE TOCTOU fix: hold the store-wide
        // integrity `write` lock for the entire scan → remove. This makes the
        // referrer scan and the removal atomic against a concurrent
        // `create_document`/`update_document` adding a link/embed/crosslink edge
        // that targets this document — no dangling reference can slip in between
        // scan and remove. Acquired before any shard lock (consistent order).
        let _integrity = self.reference_lock.write().unwrap();
        // §4.4.5 reference-integrity gate: any OTHER document holding a
        // link/embed/crosslink edge targeting this document blocks deletion.
        let mut removed = None;
        for shard in self.shards.iter() {
            let guard = shard.read().unwrap();
            for (id, doc) in guard.docs.iter() {
                if id == document_id {
                    removed = Some((**doc).clone());
                    continue;
                }
                for edge in &doc.graph.edges {
                    if matches!(
                        edge.kind,
                        EdgeKind::Link | EdgeKind::Embed | EdgeKind::Crosslink
                    ) && &edge.target.0 == document_id
                    {
                        return Err(StoreError::DocumentInUse);
                    }
                }
            }
        }
        let mut guard = self.shard_for(document_id).write().unwrap();
        if guard.docs.remove(document_id).is_none() {
            return Err(StoreError::DocumentNotFound);
        }
        self.append_journal(
            "delete_document",
            removed.as_ref().map(|d| d.revision).unwrap_or(0),
        );
        Ok(())
    }

    async fn publish_document(&self, document_id: &DocumentId) -> Result<Document, StoreError> {
        let mut guard = self.shard_for(document_id).write().unwrap();
        let current = guard
            .docs
            .get(document_id)
            .ok_or(StoreError::DocumentNotFound)?;
        let held_shard = self.shard_for(document_id);
        // §4.4.3 publish gate — applied to every reference edge
        // (`link`/`embed`/`crosslink`) in the graph:
        //
        // 1. Explicit state gate. A `BROKEN` reference (any kind) or a `STALE`
        //    (unsynced) reference blocks publish — §4.4.2 gives all three kinds
        //    the same reference-state vocabulary, and §4.4.3 forbids publishing
        //    a doc whose references are unresolved. (Adversarial finding #2.)
        // 2. Derived target-existence gate. Store-supplied `state` is fabricated
        //    input, not ground truth: a caller can stamp a `link` `Resolved`
        //    whose target document does not exist. We derive the true reference
        //    state by verifying the target `(documentId, nodeId)` exists in the
        //    store (and is not archived); a missing target is effectively
        //    `Broken` and blocks publish. Cross-wiki targets (`cross_wiki: true`)
        //    live in another wiki/store and cannot be verified locally, so their
        //    stored state is honored. (Adversarial finding #4.)
        for edge in &current.graph.edges {
            if !matches!(
                edge.kind,
                EdgeKind::Link | EdgeKind::Embed | EdgeKind::Crosslink
            ) {
                continue;
            }
            if let Some(s) = edge.state {
                if s == ReferenceState::Broken || s == ReferenceState::Stale {
                    return Err(StoreError::UnresolvedReference);
                }
            }
            if !edge.cross_wiki
                && !reference_target_exists(&guard.docs, held_shard, &self.shards, &edge.target)
            {
                return Err(StoreError::UnresolvedReference);
            }
        }
        if !current.state.can_transition_to(DocState::Published) {
            return Err(StoreError::InvalidState);
        }
        let base = current.revision;
        let updated = transition(&mut guard, document_id, DocState::Published)?;
        self.append_journal("publish_document", base);
        Ok(updated)
    }

    async fn unpublish_document(&self, document_id: &DocumentId) -> Result<Document, StoreError> {
        let mut guard = self.shard_for(document_id).write().unwrap();
        let current = guard
            .docs
            .get(document_id)
            .ok_or(StoreError::DocumentNotFound)?;
        if current.state != DocState::Published {
            return Err(StoreError::InvalidState);
        }
        let base = current.revision;
        let updated = transition(&mut guard, document_id, DocState::Draft)?;
        self.append_journal("unpublish_document", base);
        Ok(updated)
    }

    async fn archive_document(&self, document_id: &DocumentId) -> Result<Document, StoreError> {
        let mut guard = self.shard_for(document_id).write().unwrap();
        let current = guard
            .docs
            .get(document_id)
            .ok_or(StoreError::DocumentNotFound)?;
        if current.state == DocState::Archived {
            return Err(StoreError::InvalidState);
        }
        let base = current.revision;
        let updated = transition(&mut guard, document_id, DocState::Archived)?;
        self.append_journal("archive_document", base);
        Ok(updated)
    }

    async fn list_documents(
        &self,
        wiki_id: &WikiId,
        filter: &ListDocumentsFilter,
    ) -> Result<DocumentList, StoreError> {
        // FS-2 (§4.1.3): listDocuments on an unknown `wikiId` → `WikiNotFound`.
        if !self.wikis.read().unwrap().contains_key(wiki_id) {
            return Err(StoreError::WikiNotFound);
        }
        let page = filter.page.unwrap_or(1);
        let page_size = filter.page_size.unwrap_or(20);
        if page < 1 || !(1..=100).contains(&page_size) {
            return Err(StoreError::ValidationError(
                "page must be >= 1 and page_size must be in 1..=100".into(),
            ));
        }
        let mut matched = Vec::new();
        for shard in self.shards.iter() {
            let guard = shard.read().unwrap();
            for doc in guard.docs.values() {
                if doc.wiki_id != *wiki_id {
                    continue;
                }
                if let Some(state) = filter.state {
                    if doc.state != state {
                        continue;
                    }
                }
                if let Some(tag) = &filter.tag {
                    if !doc.tags.iter().any(|t| t == tag) {
                        continue;
                    }
                }
                matched.push(doc.clone());
            }
        }
        let total = matched.len();
        let start = (page.saturating_sub(1)) as usize;
        let start = start.saturating_mul(page_size as usize);
        // §4.1.3 pagination guard (adversarial finding #1, HIGH): a requested
        // `page` beyond the data yields a valid, non-panicking **empty** items
        // slice on a correctly-shaped page. Guard `start >= total` *before* any
        // slicing or `start + page_size` arithmetic so `page: u64::MAX` cannot
        // overflow (saturating math already present) or panic.
        if start >= total {
            return Ok(DocumentList {
                items: vec![],
                total: total as u64,
                page,
                page_size,
            });
        }
        let end = (start + page_size as usize).min(total);
        let items: Vec<DocumentSummary> = matched[start..end]
            .iter()
            .map(|d| DocumentSummary {
                document_id: d.document_id.clone(),
                wiki_id: d.wiki_id.clone(),
                title: d.title.clone(),
                state: d.state,
                revision: d.revision,
                updated_at: d.updated_at.clone(),
            })
            .collect();
        Ok(DocumentList {
            items,
            total: total as u64,
            page,
            page_size,
        })
    }

    async fn create_wiki(&self, name: &str) -> Result<Wiki, StoreError> {
        let len = name.chars().count();
        if name.is_empty() || len > 100 {
            return Err(StoreError::ValidationError(
                "wiki name must be non-empty and at most 100 chars".into(),
            ));
        }
        let id = WikiId(format!(
            "wiki-{}",
            self.next_id.fetch_add(1, Ordering::Relaxed)
        ));
        let wiki = Wiki {
            wiki_id: id.clone(),
            name: name.to_string(),
        };
        self.wikis.write().unwrap().insert(id, wiki.clone());
        self.append_journal("create_wiki", 0);
        Ok(wiki)
    }

    async fn get_wiki(&self, wiki_id: &WikiId) -> Result<Wiki, StoreError> {
        self.wikis
            .read()
            .unwrap()
            .get(wiki_id)
            .cloned()
            .ok_or(StoreError::WikiNotFound)
    }

    async fn list_wikis(&self) -> Result<Vec<Wiki>, StoreError> {
        let guard = self.wikis.read().unwrap();
        Ok(guard.values().cloned().collect())
    }
}

/// Does a reference target `(docId, nodeId)` exist in the store as a **live**
/// (non-archived) node? Used by the §4.4.3 publish gate to derive the true
/// reference state instead of trusting caller-supplied `state` (which is
/// fabricated input, not ground truth).
///
/// `held` is the hash map of the shard currently held under a write guard (the
/// publishing document's own shard), and `held_shard` identifies that same
/// shard so we skip re-locking it — `std::sync::RwLock` is **not** reentrant, so
/// a self-referential target in the held shard must be resolved from the
/// in-hand map, not by taking the shard `read()` again.
fn reference_target_exists(
    held: &HashMap<DocumentId, Arc<Document>>,
    held_shard: &RwLock<StoreShard>,
    shards: &[RwLock<StoreShard>],
    target: &(DocumentId, NodeId),
) -> bool {
    if let Some(doc) = held.get(&target.0) {
        return doc.state != DocState::Archived
            && doc.graph.nodes.iter().any(|n| n.node_id == target.1);
    }
    for shard in shards {
        if std::ptr::eq(shard, held_shard) {
            continue;
        }
        let guard = shard.read().unwrap();
        if let Some(doc) = guard.docs.get(&target.0) {
            return doc.state != DocState::Archived
                && doc.graph.nodes.iter().any(|n| n.node_id == target.1);
        }
    }
    false
}

/// Replace the stored document under a shard write guard with a new state.
///
/// `guard` is the live `RwLockWriteGuard`. We read the current doc, verify the
/// transition is legal, bump the revision and swap in the new `Arc`.
fn transition(
    guard: &mut std::sync::RwLockWriteGuard<'_, StoreShard>,
    document_id: &DocumentId,
    to: DocState,
) -> Result<Document, StoreError> {
    let current = guard
        .docs
        .get(document_id)
        .ok_or(StoreError::DocumentNotFound)?;
    if !current.state.can_transition_to(to) {
        return Err(StoreError::InvalidState);
    }
    let updated = Document {
        document_id: current.document_id.clone(),
        wiki_id: current.wiki_id.clone(),
        revision: current.revision + 1,
        state: to,
        graph: current.graph.clone(),
        title: current.title.clone(),
        created_at: current.created_at.clone(),
        updated_at: iso_now(),
        tags: current.tags.clone(),
        author: current.author.clone(),
    };
    guard
        .docs
        .insert(document_id.clone(), Arc::new(updated.clone()));
    Ok(updated)
}

/// A Provident graph is well-formed enough for the store when it has exactly one
/// `doc-head` edge and one `doc-end` edge (§4.2.2).
fn valid_provident_graph(g: &Graph) -> bool {
    let heads = g
        .edges
        .iter()
        .filter(|e| e.kind == EdgeKind::DocHead)
        .count();
    let ends = g
        .edges
        .iter()
        .filter(|e| e.kind == EdgeKind::DocEnd)
        .count();
    heads == 1 && ends == 1
}

/// Current time as an ISO-8601 UTC string (§4.1.1). No external time crate.
fn iso_now() -> String {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    let nanos = dur.subsec_nanos();
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let (hh, mm, ss) = (rem / 3_600, (rem % 3_600) / 60, rem % 60);
    let (y, mo, d) = civil_from_days(days);
    format!("{y:04}-{mo:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}.{nanos:09}Z")
}

/// Days-since-epoch → (year, month, day) in the proleptic Gregorian calendar.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}
