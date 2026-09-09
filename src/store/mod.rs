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
    /// A cluster of related nodes/edges declared as a community (§4.2.8.3,
    /// `docs/specs/gnosis.md` §4.2.1 + §4.2.8). First-class node kind added by
    /// the §4.2 TestWriter.
    Community,
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
    /// A containment edge from a `community` node to one of its member nodes
    /// (§4.2.8.3). Added by the §4.2 TestWriter.
    Member,
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
    /// For `EdgeKind::Relation` edges, the triple's `relationType` (§4.2.7.2);
    /// `None` for every other edge kind. GRAPH-OWNS-RELATION-AND-MERGE: the
    /// graph owns the triple's relationship so a `relation` edge genuinely
    /// carries its `relationType` (no second-source-of-truth sidecar).
    pub relation_type: Option<String>,
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Wiki {
    pub wiki_id: WikiId,
    pub name: String,
}

// ---------------------------------------------------------------------------
// §4.2 — Knowledge graph: types added by the §4.2 TestWriter (RED-first)
//
// These are the §4.2.5–§4.2.9 surface types. The `Node`/`Edge` graph model
// (§4.1) is FROZEN — the Implementer must NOT add fields to them (that would
// break the §4.1 green suite's literal constructions). Where the spec models a
// `relationType`/`citations` property, it is carried by the dedicated value
// types below; the structural edge/node is what the graph owns.
// ---------------------------------------------------------------------------

/// A community identity (the `communityId` scalar of §4.2.8.2).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommunityId(pub String);

impl fmt::Display for CommunityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A triple — a directed semantic statement `(subject, relation, object)`
/// (§4.2.7). Stored as a `relation` edge owned by the knowledge graph
/// (§4.2.7.2); the subject is the edge's source node, the object its target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Triple {
    pub subject: (DocumentId, NodeId),
    pub relation: String,
    pub object: (DocumentId, NodeId),
    pub relation_type: String,
    pub created_at: String,
}

/// A declared community (§4.2.8.2). Stored as a `community` node whose members
/// are the declared node set (via `member` edges) and whose `summary` is the
/// provided summary (§4.2.8.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Community {
    pub community_id: CommunityId,
    pub members: Vec<(DocumentId, NodeId)>,
    pub summary: String,
    pub wiki_id: WikiId,
}

/// A single `{from, to}` merge pair in `ResolutionResult.merged` (§4.2.9.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityPair {
    pub from: (DocumentId, NodeId),
    pub to: (DocumentId, NodeId),
}

/// A single `{alias, canonical}` pair in `ResolutionResult.aliases` (§4.2.9.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Alias {
    pub alias: (DocumentId, NodeId),
    pub canonical: (DocumentId, NodeId),
}

/// `resolveEntities` return shape (§4.2.9.1):
/// `{merged: [{from, to}], aliases: [{alias, canonical}], canonicalId}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolutionResult {
    pub merged: Vec<EntityPair>,
    pub aliases: Vec<Alias>,
    pub canonical_id: (DocumentId, NodeId),
}

/// One node resolved by a topological `reference`→`fact` walk (§4.2.6). Every
/// reachable node appears exactly once — each resolved fact is reused across
/// all its dependents (no redundant re-resolution).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedFact {
    pub node: (DocumentId, NodeId),
    pub value: String,
    /// 1-based topological resolution order: a dependency is always resolved
    /// (has a strictly smaller `resolution_order`) before its dependent.
    pub resolution_order: u64,
}

/// A `fact` node with its citations (§4.3.1 / §4.2.9.1 return shape). Added so
/// `mergeFacts` (§4.2.9) has a concrete `Fact` to return; `citations` is the
/// union of the merged facts' citation sets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fact {
    pub fact_key: String,
    pub value: String,
    pub document_id: DocumentId,
    pub node_id: NodeId,
    pub updated_at: String,
    pub citations: Vec<(DocumentId, NodeId)>,
}

/// `getTriples(node, {direction?, relationType?, wikiId})` options (§4.2.7.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetTriplesFilter {
    pub direction: TripleDirection,
    pub relation_type: Option<String>,
    pub wiki_id: WikiId,
}

/// Triple query direction (§4.2.7.3): `out` (node is the subject), `in` (node
/// is the object), or `both`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TripleDirection {
    Out,
    In,
    Both,
}

/// `queryTriples(subject?, relation?, object?, …)` pattern (§4.2.7.3). Any
/// component may be `None` to act as a wildcard.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriplePattern {
    pub subject: Option<(DocumentId, NodeId)>,
    pub relation: Option<String>,
    pub object: Option<(DocumentId, NodeId)>,
}

/// `queryTriples(…, {wikiId, limit?})` options (§4.2.7.3). `limit` is bounded to
/// `1..=100` (FS-3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryTriplesOptions {
    pub wiki_id: WikiId,
    pub limit: Option<u64>,
}

/// `declareCommunity(nodeIds, {summary, wikiId})` options (§4.2.8.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeclareCommunityOptions {
    pub summary: String,
    pub wiki_id: WikiId,
}

/// `resolveEntities(entityIds, {canonicalId?, wikiId})` options (§4.2.9.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolveEntitiesOptions {
    pub canonical_id: Option<(DocumentId, NodeId)>,
    pub wiki_id: WikiId,
}

/// `mergeFacts(factKeys, {canonicalKey, wikiId})` options (§4.2.9.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeFactsOptions {
    pub canonical_key: String,
    pub wiki_id: WikiId,
}

/// `resolveReferences(roots, {wikiId, maxHops})` options (§4.2.4/§4.2.6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolveOptions {
    pub wiki_id: WikiId,
    /// Bounded-resolution hop cap; exceeding it → `HopLimitExceeded` (§4.2.6).
    pub max_hops: u64,
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
    /// FS-25 (§4.2.8.5) — `getCommunity`/`listCommunities`/`updateCommunitySummary`
    /// on an unknown `communityId`.
    CommunityNotFound,
    /// FS-12 (§4.2.6) — a topological `reference`→`fact` resolution detects a
    /// cycle (a node already on the current resolution path).
    CycleDetected,
    /// FS-11 (§4.2.6) — a bounded `reference`→`fact` resolution exceeds its hop cap.
    HopLimitExceeded,
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
            CommunityNotFound => "community not found",
            CycleDetected => "reference→fact resolution detected a cycle",
            HopLimitExceeded => "reference→fact resolution exceeded its hop cap",
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

    // -----------------------------------------------------------------------
    // §4.2 — Knowledge-graph surface (added by the §4.2 TestWriter, RED-first)
    //
    // Every method is a **stub** (`unimplemented!()` in `Store`): the crate
    // compiles, the tests load, and every §4.2 assertion fails at runtime. The
    // Implementer replaces these bodies with the real graph logic. Exact
    // signatures/return shapes/fail-states follow §4.2.5–§4.2.9.
    // -----------------------------------------------------------------------

    /// `edgesFrom(documentId, nodeId) → Edge[]` (§4.2.5). All edges whose source
    /// is `(documentId, nodeId)`. Failures: `DocumentNotFound`, `ValidationError`
    /// (invalid `nodeId`).
    fn edges_from(
        &self,
        document_id: &DocumentId,
        node_id: &NodeId,
    ) -> impl Future<Output = Result<Vec<Edge>, StoreError>> + Send;

    /// `edgesTo(documentId, nodeId) → Edge[]` (§4.2.5). All edges whose target
    /// is `(documentId, nodeId)`. Failures: `DocumentNotFound`,
    /// `ValidationError` (invalid `nodeId`).
    fn edges_to(
        &self,
        document_id: &DocumentId,
        node_id: &NodeId,
    ) -> impl Future<Output = Result<Vec<Edge>, StoreError>> + Send;

    /// `edgesByKind(documentId, nodeId, kind) → Edge[]` (§4.2.5). All edges from
    /// `(documentId, nodeId)` of the given `kind`. Failures: `DocumentNotFound`,
    /// `ValidationError` (invalid `nodeId`).
    fn edges_by_kind(
        &self,
        document_id: &DocumentId,
        node_id: &NodeId,
        kind: EdgeKind,
    ) -> impl Future<Output = Result<Vec<Edge>, StoreError>> + Send;

    /// `edgesForDocument(documentId) → Edge[]` (§4.2.5). All edges in the
    /// document. Failures: `DocumentNotFound`.
    fn edges_for_document(
        &self,
        document_id: &DocumentId,
    ) -> impl Future<Output = Result<Vec<Edge>, StoreError>> + Send;

    /// `docHeadForDocument(documentId) → Node` (§4.2.5). The document's head
    /// node (the target of its `doc-head` edge). Failures: `DocumentNotFound`;
    /// `InvalidState` if the document has no `doc-head` edge (malformed graph).
    fn doc_head_for_document(
        &self,
        document_id: &DocumentId,
    ) -> impl Future<Output = Result<Node, StoreError>> + Send;

    /// Topological `reference`→`fact` resolution (§4.2.4/§4.2.6). Resolves
    /// `roots` in dependency order (each target `fact` before its `reference`
    /// dépendent), reusing each resolved fact across all its dependents (each
    /// node appears exactly once). Failures: `DocumentNotFound`,
    /// `CycleDetected` (a node already on the current path), `HopLimitExceeded`
    /// (resolution exceeds `maxHops`).
    fn resolve_references(
        &self,
        roots: &[(DocumentId, NodeId)],
        options: &ResolveOptions,
    ) -> impl Future<Output = Result<Vec<ResolvedFact>, StoreError>> + Send;

    /// `addTriple(subject, relation, object, {wikiId}) → Triple` (§4.2.7.3).
    /// Stores a `relation` edge owned by the graph. Failures: `DocumentNotFound`
    /// (`subject`/`object` document unknown); `ValidationError` (`relation`
    /// empty or `subject`/`object` node invalid).
    fn add_triple(
        &self,
        subject: &(DocumentId, NodeId),
        relation: &str,
        object: &(DocumentId, NodeId),
        wiki_id: &WikiId,
    ) -> impl Future<Output = Result<Triple, StoreError>> + Send;

    /// `getTriples(node, {direction?, relationType?, wikiId}) → Triple[]`
    /// (§4.2.7.3). Failures: `DocumentNotFound`; `ValidationError` (`node`
    /// invalid or `relationType` empty).
    fn get_triples(
        &self,
        node: &(DocumentId, NodeId),
        options: &GetTriplesFilter,
    ) -> impl Future<Output = Result<Vec<Triple>, StoreError>> + Send;

    /// `queryTriples(subject?, relation?, object?, {wikiId, limit?}) → Triple[]`
    /// (§4.2.7.3). Any `None` pattern component is a wildcard. Failures:
    /// `WikiNotFound`; `ValidationError` (`limit < 1` or `limit > 100`).
    fn query_triples(
        &self,
        pattern: &TriplePattern,
        options: &QueryTriplesOptions,
    ) -> impl Future<Output = Result<Vec<Triple>, StoreError>> + Send;

    /// `declareCommunity(nodeIds, {summary, wikiId}) → Community` (§4.2.8.2).
    /// Stores a `community` node whose members are the node set (via `member`
    /// edges) with the provided `summary`. Failures: `ValidationError` (empty
    /// node set or empty `summary`).
    fn declare_community(
        &self,
        node_ids: &[(DocumentId, NodeId)],
        options: &DeclareCommunityOptions,
    ) -> impl Future<Output = Result<Community, StoreError>> + Send;

    /// `getCommunity(communityId) → Community` (§4.2.8.2). Failures:
    /// `CommunityNotFound`.
    fn get_community(
        &self,
        community_id: &CommunityId,
    ) -> impl Future<Output = Result<Community, StoreError>> + Send;

    /// `listCommunities(wikiId) → Community[]` (§4.2.8.2). Failures:
    /// `WikiNotFound`.
    fn list_communities(
        &self,
        wiki_id: &WikiId,
    ) -> impl Future<Output = Result<Vec<Community>, StoreError>> + Send;

    /// `updateCommunitySummary(communityId, summary) → Community` (§4.2.8.2).
    /// Failures: `CommunityNotFound`, `ValidationError` (empty `summary`).
    fn update_community_summary(
        &self,
        community_id: &CommunityId,
        summary: &str,
    ) -> impl Future<Output = Result<Community, StoreError>> + Send;

    /// `setReferenceState(documentId, source, target, state) → Edge`
    /// (§4.2.8.5/§4.4.2). Marks the reference edge `source→target` as
    /// `FRESH`/`STALE`/`RESOLVED`/`BROKEN`. Failures: `DocumentNotFound`;
    /// `ValidationError` (invalid `state` string, or unknown source/target node).
    fn set_reference_state(
        &self,
        document_id: &DocumentId,
        source: &NodeId,
        target: &NodeId,
        state: &str,
    ) -> impl Future<Output = Result<Edge, StoreError>> + Send;

    /// `resolveEntities(entityIds, {canonicalId?, wikiId}) → ResolutionResult`
    /// (§4.2.9.1). Failures: `DocumentNotFound`; `ValidationError` (`entityIds`
    /// empty or `canonicalId` invalid/not among `entityIds`).
    fn resolve_entities(
        &self,
        entity_ids: &[(DocumentId, NodeId)],
        options: &ResolveEntitiesOptions,
    ) -> impl Future<Output = Result<ResolutionResult, StoreError>> + Send;

    /// GRAPH-OWNS-RELATION-AND-MERGE: read-only accessor for the **durable**
    /// alias→canonical mapping recorded by `resolveEntities` (§4.2.9.1). Returns
    /// the canonical entity `(docId, nodeId)` for a resolved alias, or `None`
    /// when the alias has no recorded resolution. Added by the TestWriter so the
    /// CRITICAL #2 regression (resolveEntities must *persist*, not just return a
    /// descriptor) can read the mapping back. RED today: the Store body is a
    /// stub the Implementer must fill.
    fn entity_alias_canonical(
        &self,
        alias: &(DocumentId, NodeId),
    ) -> impl Future<Output = Result<Option<(DocumentId, NodeId)>, StoreError>> + Send;

    /// `mergeFacts(factKeys, {canonicalKey, wikiId}) → Fact` (§4.2.9.1). Merges
    /// duplicate `fact` nodes: canonical value + union of citations. Failures:
    /// `DocumentNotFound`; `ValidationError` (`factKeys` empty, `canonicalKey`
    /// invalid, or a merge would leave the canonical fact with zero citations
    /// — §4.3.2 minimum-citation invariant); `ConflictError` (conflicting
    /// values with no resolution).
    fn merge_facts(
        &self,
        fact_keys: &[String],
        options: &MergeFactsOptions,
    ) -> impl Future<Output = Result<Fact, StoreError>> + Send;

    /// GRAPH-OWNS-RELATION-AND-MERGE: read-only accessor that returns the
    /// **persisted** fact for `fact_key` in `wiki_id` from the canonical fact
    /// store (§4.3.1). Added so the CRITICAL #1 regression (mergeFacts must
    /// *persist* the merged fact — union citations + refreshed `updatedAt`,
    /// journaled — not merely return a descriptor) can read the merge back.
    /// Failures: `ValidationError` (no such fact). RED today: the Store body is
    /// a stub the Implementer must fill.
    fn get_fact(
        &self,
        wiki_id: &WikiId,
        fact_key: &str,
    ) -> impl Future<Output = Result<Fact, StoreError>> + Send;

    /// `createFact` (§4.2.8.1 / §4.3.1) — the manual override that declares a
    /// fact + its citations. Added by the §4.2 TestWriter **only as the seeding
    /// mechanism** for `mergeFacts` (§4.2.9.1): the frozen §4.1 `Node` cannot
    /// carry a per-fact `citations` set, so facts with citations are established
    /// through this spec-documented manual override. Failures: `ValidationError`
    /// (empty `factKey`/`value`, or empty `citations` — §4.3.2 minimum-citation
    /// invariant), `ConflictError` (duplicate `factKey` in the wiki).
    fn create_fact(
        &self,
        wiki_id: &WikiId,
        document_id: &DocumentId,
        fact_key: &str,
        value: &str,
        citations: &[(DocumentId, NodeId)],
    ) -> impl Future<Output = Result<Fact, StoreError>> + Send;
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
    // -----------------------------------------------------------------------
    // §4.2 sidecar stores.
    //
    // The frozen §4.1 `Node`/`Edge` cannot carry the §4.2/§4.3 properties that
    // are not part of the structural graph (a triple's `relationType`, a fact's
    // `citations`, a community's `summary`). Per SHARDED-RWLOCK-STORE and the
    // "derived index is not a second owner" rule (§4.2.7.2), these are held as
    // small sidecar maps owned by the store. The authoritative adjacency lives
    // in the sharded document graph (`Relation`/`Member`/`Community` nodes and
    // edges); these maps carry the property payloads the structural types are
    // frozen without, and each write is journaled like any other mutation.
    // -----------------------------------------------------------------------
    /// Declared communities, keyed by `communityId` (§4.2.8.2); the authoritative
    /// `summary` that the manual override must never be overwritten by (§4.2.8.4).
    communities: RwLock<HashMap<CommunityId, Community>>,
    /// Declared facts, keyed by `wikiId` then by the wiki-unique `factKey`
    /// (§4.3.1). Carries each fact's `citations` + `updatedAt` — the properties
    /// the frozen fact `Node` cannot store.
    fact_store: RwLock<HashMap<WikiId, HashMap<String, Fact>>>,
    /// Declared triples (§4.2.7). The graph's `Relation` edges are the
    /// authoritative owner of each triple (§4.2.7.2, GRAPH-OWNS-RELATION-AND-MERGE);
    /// this list is kept purely as a **derived provenance cache** for the
    /// `Triple.created_at` payload the frozen `Edge` cannot hold. Membership is
    /// read from the graph's `Relation` edges, never from this cache, and this
    /// cache is always reconciled to the graph (records whose source/subject
    /// edge no longer exists are pruned), so a stale record can never resurface
    /// or duplicate a re-added triple.
    triple_store: RwLock<Vec<TripleRecord>>,
    /// Revision numbers this document reached via a **state-annotation-only**
    /// `set_reference_state` bump (no structural/content change). Lets
    /// `update_document` distinguish a stale base caused solely by a concurrent
    /// `set_reference_state` (reconcile: both edits land, current reference-edge
    /// states preserved) from a genuine content conflict (`ConflictError`, §4.1.4).
    state_annotations: RwLock<HashMap<DocumentId, std::collections::HashSet<u64>>>,
    /// Durable, journaled alias→canonical entity mapping recorded by
    /// `resolve_entities` (§4.2.9.1, GRAPH-OWNS-RELATION-AND-MERGE). An alias
    /// `(docId, nodeId)` maps to its resolved canonical `(docId, nodeId)`.
    entity_resolution: RwLock<HashMap<(DocumentId, NodeId), (DocumentId, NodeId)>>,
    /// Every node id that has **ever** been a node of a document (§4.2.5/§4.2.7.5).
    /// Lets `getTriples` distinguish a node that was removed by a cascade (→ `Ok([])`)
    /// from a node that never existed (→ `ValidationError`, FS-3).
    known_nodes: RwLock<HashMap<DocumentId, std::collections::HashSet<NodeId>>>,
}

/// A triple's property payload (§4.2.7.2). The structural `Relation` edge is what
/// the graph owns (and adjacency exposes); this record carries the
/// `relationType` + provenance that the frozen §4.1 `Edge` cannot.
#[derive(Debug, Clone, PartialEq, Eq)]
struct TripleRecord {
    wiki_id: WikiId,
    subject: (DocumentId, NodeId),
    object: (DocumentId, NodeId),
    relation: String,
    created_at: String,
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
            communities: RwLock::new(HashMap::new()),
            fact_store: RwLock::new(HashMap::new()),
            triple_store: RwLock::new(Vec::new()),
            state_annotations: RwLock::new(HashMap::new()),
            entity_resolution: RwLock::new(HashMap::new()),
            known_nodes: RwLock::new(HashMap::new()),
        }
    }

    /// Read one node from the authoritative store by `(documentId, nodeId)`.
    /// Failures follow the §4.2.5 adjacency contract: unknown `documentId` →
    /// `DocumentNotFound`; a `nodeId` that is not a node in the document →
    /// `ValidationError`. Takes only that document's shard `read()` and releases
    /// it before returning, so callers can hold no shard lock while recursing.
    fn read_node(&self, document_id: &DocumentId, node_id: &NodeId) -> Result<Node, StoreError> {
        let guard = self.shard_for(document_id).read().unwrap();
        let doc = guard
            .docs
            .get(document_id)
            .ok_or(StoreError::DocumentNotFound)?;
        doc.graph
            .nodes
            .iter()
            .find(|n| &n.node_id == node_id)
            .cloned()
            .ok_or_else(|| {
                StoreError::ValidationError(format!(
                    "node {node_id} does not exist in document {document_id}"
                ))
            })
    }

    /// Topologically resolve one `reference`→`fact` dependency (§4.2.6).
    ///
    /// - A `fact` node's value is its own canonical value.
    /// - A `reference` node's value is the resolved value of its `target` (its
    ///   dependent resolves only after the target — the target gets a strictly
    ///   smaller `resolution_order`, assigned from `order`).
    /// - Already-resolved nodes are reused from `memo` (diamond reuse: each node
    ///   appears exactly once in `result`).
    /// - A node already on the current `path` → `CycleDetected`.
    /// - `hops > max_hops` → `HopLimitExceeded`.
    ///
    /// Read-only: takes only per-shard `read()` locks (one at a time, released
    /// before recursing) so it never contends with the mutation `reference_lock`
    /// and never holds two shards at once.
    #[allow(clippy::too_many_arguments)]
    fn resolve_node(
        &self,
        node_id: &(DocumentId, NodeId),
        hops: u64,
        max_hops: u64,
        path: &mut Vec<(DocumentId, NodeId)>,
        order: &mut u64,
        result: &mut Vec<ResolvedFact>,
        memo: &mut HashMap<(DocumentId, NodeId), String>,
    ) -> Result<String, StoreError> {
        if hops > max_hops {
            return Err(StoreError::HopLimitExceeded);
        }
        if path.contains(node_id) {
            return Err(StoreError::CycleDetected);
        }
        if let Some(v) = memo.get(node_id) {
            return Ok(v.clone());
        }
        let node = self.read_node(&node_id.0, &node_id.1)?;
        path.push(node_id.clone());
        let value = match node.kind {
            NodeKind::Reference => {
                let target = node.target.ok_or_else(|| {
                    StoreError::ValidationError("reference node has no target".into())
                })?;
                self.resolve_node(&target, hops + 1, max_hops, path, order, result, memo)?
            }
            _ => node.value.clone().unwrap_or_default(),
        };
        *order += 1;
        result.push(ResolvedFact {
            node: node_id.clone(),
            value: value.clone(),
            resolution_order: *order,
        });
        memo.insert(node_id.clone(), value.clone());
        path.pop();
        Ok(value)
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

    /// Reconcile the derived `triple_store` provenance cache to the authoritative
    /// graph (`Relation` edges). GRAPH-OWNS-RELATION-AND-MERGE (§4.2.7.2): the
    /// graph owns triples; the cache is only a `created_at` provenance lookup, so
    /// any record whose subject/object endpoints no longer exist is pruned. Call
    /// from inside a mutation's critical section (after the shard write).
    fn reconcile_triple_store(&self, graph: &Graph) {
        let mut store = self.triple_store.write().unwrap();
        store.retain(|rec| {
            let subject_ok = graph
                .nodes
                .iter()
                .any(|n| n.document_id == rec.subject.0 && n.node_id == rec.subject.1);
            if !subject_ok {
                return false;
            }
            // An object in the same document's graph — same check. A cross-document
            // object is not verifiable here; membership is graph-owned anyway.
            if rec.object.0 == rec.subject.0 {
                graph
                    .nodes
                    .iter()
                    .any(|n| n.document_id == rec.object.0 && n.node_id == rec.object.1)
            } else {
                true
            }
        });
    }

    /// Build the `Triple` a `Relation` edge stands for, using the derived
    /// `triple_store` cache for its `created_at` provenance (the frozen `Edge`
    /// cannot hold it). The edge is the authoritative source (§4.2.7.2).
    fn triple_from_edge(&self, edge: &Edge) -> Triple {
        let relation = edge.relation_type.clone().unwrap_or_default();
        let created_at = self
            .triple_store
            .read()
            .unwrap()
            .iter()
            .find(|r| {
                r.subject == edge.source
                    && r.object == edge.target
                    && (r.relation == relation && !relation.is_empty())
            })
            .map(|r| r.created_at.clone())
            .unwrap_or_default();
        Triple {
            subject: edge.source.clone(),
            relation: relation.clone(),
            object: edge.target.clone(),
            relation_type: relation,
            created_at,
        }
    }

    /// Record every node id in `graph` as "ever been a node of `document_id`"
    /// (§4.2.7.5). Call from inside a mutation's critical section.
    fn remember_nodes(&self, document_id: &DocumentId, graph: &Graph) {
        let mut known = self.known_nodes.write().unwrap();
        let set = known.entry(document_id.clone()).or_default();
        for n in &graph.nodes {
            set.insert(n.node_id.clone());
        }
    }

    /// Does `document_id` exist in the store?
    fn doc_exists(&self, document_id: &DocumentId) -> Result<(), StoreError> {
        let guard = self.shard_for(document_id).read().unwrap();
        if guard.docs.contains_key(document_id) {
            Ok(())
        } else {
            Err(StoreError::DocumentNotFound)
        }
    }

    /// Is `node_id` currently a node of `document_id`?
    fn node_is_present(&self, document_id: &DocumentId, node_id: &NodeId) -> bool {
        let guard = self.shard_for(document_id).read().unwrap();
        guard
            .docs
            .get(document_id)
            .map(|d| d.graph.nodes.iter().any(|n| n.node_id == *node_id))
            .unwrap_or(false)
    }

    /// Did `node_id` ever exist as a node of `document_id`?
    fn node_ever_existed(&self, document_id: &DocumentId, node_id: &NodeId) -> bool {
        self.known_nodes
            .read()
            .unwrap()
            .get(document_id)
            .map(|s| s.contains(node_id))
            .unwrap_or(false)
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
        let base_revision = request.base_revision;
        let current_revision = current.revision;
        // §4.1.4 optimistic-concurrency guard — atomic under the shard write lock.
        //
        // GRAPH-OWNS-RELATION-AND-MERGE (§4.2.9 decision): a stale base caused
        // **solely** by a concurrent `set_reference_state` (a state-annotation
        // bump that carries no structural/content change) is reconciled rather
        // than rejected, so both concurrent edits land without data loss. The
        // caller's structural graph is applied on top of the current document,
        // preserving the reference-edge states stamped by the concurrent
        // `set_reference_state`. Any other stale base (a genuine content
        // conflict, §4.1.4) is still a `ConflictError`.
        let reconcile_state_annotations = if current_revision != base_revision {
            let ok = {
                let anns = self.state_annotations.read().unwrap();
                anns.get(document_id)
                    .map(|s| (base_revision + 1..=current_revision).all(|r| s.contains(&r)))
                    .unwrap_or(false)
            };
            if !ok {
                return Err(StoreError::ConflictError);
            }
            true
        } else {
            false
        };
        if !valid_provident_graph(&request.graph) {
            return Err(StoreError::ValidationError(
                "graph must be a valid Provident graph (exactly one doc-head and one doc-end)"
                    .into(),
            ));
        }
        // GRAPH-OWNS-RELATION-AND-MERGE: a real cascade — a `Relation` edge whose
        // subject or object node no longer exists in the (replaced) graph is
        // pruned, so a removed end-point removes the triple (never a zombie that
        // resurfaces on restore, §4.2.7.5).
        let mut new_graph = request.graph;
        if reconcile_state_annotations {
            // Preserve the reference-edge states a concurrent `set_reference_state`
            // stamped onto edges the caller's graph still carries.
            reconcile_reference_states(&mut new_graph, &current.graph);
        }
        prune_cascaded_relation_edges(&mut new_graph, document_id);
        let updated = Document {
            document_id: current.document_id.clone(),
            wiki_id: current.wiki_id.clone(),
            revision: current_revision + 1,
            state: current.state,
            graph: new_graph,
            title: request.title.unwrap_or_else(|| current.title.clone()),
            created_at: current.created_at.clone(),
            updated_at: iso_now(),
            tags: request.tags.unwrap_or_else(|| current.tags.clone()),
            author: current.author.clone(),
        };
        guard
            .docs
            .insert(document_id.clone(), Arc::new(updated.clone()));
        // §4.2.7.5 / GRAPH-OWNS-RELATION-AND-MERGE: remember every node id that has
        // ever been a node of this document, so a later `getTriples` on a
        // cascaded-away node returns the (empty) authoritative triple set rather
        // than a misleading "invalid node" error.
        self.remember_nodes(document_id, &updated.graph);
        // GRAPH-OWNS-RELATION-AND-MERGE: reconcile the derived triple cache to the
        // (authoritative) graph — drop records whose subject/object node no longer
        // exists (same critical section, after the shard write).
        self.reconcile_triple_store(&updated.graph);
        self.append_journal("update_document", base_revision);
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

    // -----------------------------------------------------------------------
    // §4.2 stub bodies — the RED state.
    //
    // Every §4.2 operation is `unimplemented!()`: the crate compiles, every test
    // loads, and every §4.2 behavior assertion fails at runtime (the executable
    // contract is `tests/graph_integration.rs`). The Implementer replaces these
    // bodies with the real §4.2 logic. They must NOT add fields to the frozen
    // `Node`/`Edge` types (§4.1).
    // -----------------------------------------------------------------------

    async fn edges_from(
        &self,
        document_id: &DocumentId,
        node_id: &NodeId,
    ) -> Result<Vec<Edge>, StoreError> {
        let guard = self.shard_for(document_id).read().unwrap();
        let doc = guard
            .docs
            .get(document_id)
            .ok_or(StoreError::DocumentNotFound)?;
        if !doc.graph.nodes.iter().any(|n| &n.node_id == node_id) {
            return Err(StoreError::ValidationError(
                "node does not exist in document".into(),
            ));
        }
        Ok(doc
            .graph
            .edges
            .iter()
            .filter(|e| e.source == (document_id.clone(), node_id.clone()))
            .cloned()
            .collect())
    }

    async fn edges_to(
        &self,
        document_id: &DocumentId,
        node_id: &NodeId,
    ) -> Result<Vec<Edge>, StoreError> {
        let guard = self.shard_for(document_id).read().unwrap();
        let doc = guard
            .docs
            .get(document_id)
            .ok_or(StoreError::DocumentNotFound)?;
        if !doc.graph.nodes.iter().any(|n| &n.node_id == node_id) {
            return Err(StoreError::ValidationError(
                "node does not exist in document".into(),
            ));
        }
        Ok(doc
            .graph
            .edges
            .iter()
            .filter(|e| e.target == (document_id.clone(), node_id.clone()))
            .cloned()
            .collect())
    }

    async fn edges_by_kind(
        &self,
        document_id: &DocumentId,
        node_id: &NodeId,
        kind: EdgeKind,
    ) -> Result<Vec<Edge>, StoreError> {
        let guard = self.shard_for(document_id).read().unwrap();
        let doc = guard
            .docs
            .get(document_id)
            .ok_or(StoreError::DocumentNotFound)?;
        if !doc.graph.nodes.iter().any(|n| &n.node_id == node_id) {
            return Err(StoreError::ValidationError(
                "node does not exist in document".into(),
            ));
        }
        Ok(doc
            .graph
            .edges
            .iter()
            .filter(|e| e.kind == kind && e.source == (document_id.clone(), node_id.clone()))
            .cloned()
            .collect())
    }

    async fn edges_for_document(&self, document_id: &DocumentId) -> Result<Vec<Edge>, StoreError> {
        let guard = self.shard_for(document_id).read().unwrap();
        let doc = guard
            .docs
            .get(document_id)
            .ok_or(StoreError::DocumentNotFound)?;
        Ok(doc.graph.edges.clone())
    }

    async fn doc_head_for_document(&self, document_id: &DocumentId) -> Result<Node, StoreError> {
        let guard = self.shard_for(document_id).read().unwrap();
        let doc = guard
            .docs
            .get(document_id)
            .ok_or(StoreError::DocumentNotFound)?;
        let head = doc
            .graph
            .edges
            .iter()
            .find(|e| e.kind == EdgeKind::DocHead)
            .ok_or(StoreError::InvalidState)?;
        doc.graph
            .nodes
            .iter()
            .find(|n| n.node_id == head.target.1)
            .cloned()
            .ok_or(StoreError::InvalidState)
    }

    async fn resolve_references(
        &self,
        roots: &[(DocumentId, NodeId)],
        options: &ResolveOptions,
    ) -> Result<Vec<ResolvedFact>, StoreError> {
        // HIGH #8 / FS-2: an unknown wiki → `WikiNotFound` (scope resolution to
        // the wiki like every other §4.2 op).
        if !self.wikis.read().unwrap().contains_key(&options.wiki_id) {
            return Err(StoreError::WikiNotFound);
        }
        // HIGH #8 / FS-3: `maxHops` is bounded to 1..=5 (§4.2.6 / the spec pins
        // `maxHops` 1–5). `0` or `> 5` → `ValidationError`.
        if !(1..=5).contains(&options.max_hops) {
            return Err(StoreError::ValidationError(
                "maxHops must be in 1..=5".into(),
            ));
        }
        let mut result: Vec<ResolvedFact> = Vec::new();
        let mut order: u64 = 0;
        // `memo` is shared across roots so a downstream fact resolved by one root
        // is reused by every other root (diamond reuse, §4.2.6); `path` is
        // per-root (a cycle is path-local).
        let mut memo: HashMap<(DocumentId, NodeId), String> = HashMap::new();
        for root in roots {
            let mut path = Vec::new();
            self.resolve_node(
                root,
                0,
                options.max_hops,
                &mut path,
                &mut order,
                &mut result,
                &mut memo,
            )?;
        }
        Ok(result)
    }

    async fn add_triple(
        &self,
        subject: &(DocumentId, NodeId),
        relation: &str,
        object: &(DocumentId, NodeId),
        wiki_id: &WikiId,
    ) -> Result<Triple, StoreError> {
        // MEDIUM #10 / FS-2: addTriple on an unknown wiki → `WikiNotFound`.
        if !self.wikis.read().unwrap().contains_key(wiki_id) {
            return Err(StoreError::WikiNotFound);
        }
        if relation.is_empty() {
            return Err(StoreError::ValidationError(
                "relation must be non-empty".into(),
            ));
        }
        // Validate the subject/object documents + nodes (§4.2.7.3).
        self.read_node(&subject.0, &subject.1)?;
        self.read_node(&object.0, &object.1)?;
        let now = iso_now();
        // Store the triple as a `Relation` edge owned by the subject's graph
        // (§4.2.7.2). Acquire only the subject shard write lock (the object's
        // node was validated above).
        {
            let mut guard = self.shard_for(&subject.0).write().unwrap();
            let current = guard
                .docs
                .get(&subject.0)
                .ok_or(StoreError::DocumentNotFound)?;
            let current = (**current).clone();
            let mut new_graph = current.graph.clone();
            // CRITICAL #4: the `Relation` edge genuinely carries the triple's
            // `relationType` (GRAPH-OWNS-RELATION-AND-MERGE, §4.2.7.2).
            new_graph.edges.push(Edge {
                source: subject.clone(),
                target: object.clone(),
                kind: EdgeKind::Relation,
                state: None,
                cross_wiki: subject.0 != object.0,
                relation_type: Some(relation.to_string()),
            });
            let updated = Document {
                revision: current.revision + 1,
                graph: new_graph,
                updated_at: now.clone(),
                ..current
            };
            let base = updated.revision.wrapping_sub(1);
            guard.docs.insert(subject.0.clone(), Arc::new(updated));
            self.append_journal("add_triple", base);
        }
        // Record the `created_at` provenance for the derived triple cache; the
        // authoritative membership lives in the graph's `Relation` edges.
        self.triple_store.write().unwrap().push(TripleRecord {
            wiki_id: wiki_id.clone(),
            subject: subject.clone(),
            object: object.clone(),
            relation: relation.to_string(),
            created_at: now.clone(),
        });
        Ok(Triple {
            subject: subject.clone(),
            relation: relation.to_string(),
            object: object.clone(),
            relation_type: relation.to_string(),
            created_at: now,
        })
    }

    async fn get_triples(
        &self,
        node: &(DocumentId, NodeId),
        options: &GetTriplesFilter,
    ) -> Result<Vec<Triple>, StoreError> {
        // MEDIUM #10 / FS-2: getTriples on an unknown wiki → `WikiNotFound`.
        if !self.wikis.read().unwrap().contains_key(&options.wiki_id) {
            return Err(StoreError::WikiNotFound);
        }
        // Validate the queried node (§4.2.7.3). A node that **never** existed is a
        // `ValidationError`; a node cascaded away (it did exist, its end-point was
        // removed) is valid and yields only the (empty) authoritative triples.
        self.doc_exists(&node.0)?;
        if !self.node_is_present(&node.0, &node.1) && !self.node_ever_existed(&node.0, &node.1) {
            return Err(StoreError::ValidationError(
                "node does not exist in document".into(),
            ));
        }
        if let Some(rt) = &options.relation_type {
            if rt.is_empty() {
                return Err(StoreError::ValidationError(
                    "relationType must be non-empty".into(),
                ));
            }
        }
        // GRAPH-OWNS-RELATION-AND-MERGE (§4.2.7.2): derive triples from the
        // authoritative graph's `Relation` edges, not a persistent second store.
        let mut out = Vec::new();
        match options.direction {
            TripleDirection::Out | TripleDirection::Both => {
                let guard = self.shard_for(&node.0).read().unwrap();
                if let Some(doc) = guard.docs.get(&node.0) {
                    if doc.wiki_id == options.wiki_id {
                        for e in doc
                            .graph
                            .edges
                            .iter()
                            .filter(|e| e.kind == EdgeKind::Relation && &e.source == node)
                        {
                            if triple_matches(e, &options.relation_type) {
                                out.push(self.triple_from_edge(e));
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        if matches!(
            options.direction,
            TripleDirection::In | TripleDirection::Both
        ) {
            for shard in self.shards.iter() {
                let guard = shard.read().unwrap();
                for doc in guard.docs.values() {
                    if doc.wiki_id != options.wiki_id {
                        continue;
                    }
                    for e in doc
                        .graph
                        .edges
                        .iter()
                        .filter(|e| e.kind == EdgeKind::Relation && &e.target == node)
                    {
                        if triple_matches(e, &options.relation_type)
                            && !out.iter().any(|t: &Triple| {
                                t.subject == e.source
                                    && t.object == e.target
                                    && t.relation == e.relation_type.clone().unwrap_or_default()
                            })
                        {
                            out.push(self.triple_from_edge(e));
                        }
                    }
                }
            }
        }
        Ok(out)
    }

    async fn query_triples(
        &self,
        pattern: &TriplePattern,
        options: &QueryTriplesOptions,
    ) -> Result<Vec<Triple>, StoreError> {
        // FS-2: queryTriples on an unknown wiki → `WikiNotFound`.
        if !self.wikis.read().unwrap().contains_key(&options.wiki_id) {
            return Err(StoreError::WikiNotFound);
        }
        if let Some(limit) = options.limit {
            if !(1..=100).contains(&limit) {
                return Err(StoreError::ValidationError(
                    "limit must be in 1..=100".into(),
                ));
            }
        }
        // GRAPH-OWNS-RELATION-AND-MERGE (§4.2.7.2): derive from the authoritative
        // graph's `Relation` edges (the graph owns triples; no second store).
        let mut out = Vec::new();
        for shard in self.shards.iter() {
            let guard = shard.read().unwrap();
            for doc in guard.docs.values() {
                if doc.wiki_id != options.wiki_id {
                    continue;
                }
                for e in doc
                    .graph
                    .edges
                    .iter()
                    .filter(|e| e.kind == EdgeKind::Relation)
                {
                    if let Some(s) = &pattern.subject {
                        if &e.source != s {
                            continue;
                        }
                    }
                    if let Some(r) = &pattern.relation {
                        if e.relation_type.as_deref() != Some(r.as_str()) {
                            continue;
                        }
                    }
                    if let Some(o) = &pattern.object {
                        if &e.target != o {
                            continue;
                        }
                    }
                    out.push(self.triple_from_edge(e));
                }
            }
        }
        if let Some(limit) = options.limit {
            out.truncate(limit as usize);
        }
        Ok(out)
    }

    async fn declare_community(
        &self,
        node_ids: &[(DocumentId, NodeId)],
        options: &DeclareCommunityOptions,
    ) -> Result<Community, StoreError> {
        if node_ids.is_empty() {
            return Err(StoreError::ValidationError(
                "a community requires at least one member node".into(),
            ));
        }
        if options.summary.is_empty() {
            return Err(StoreError::ValidationError(
                "community summary must be non-empty".into(),
            ));
        }
        let community_id = CommunityId(format!(
            "comm-{}",
            self.next_id.fetch_add(1, Ordering::Relaxed)
        ));
        let community = Community {
            community_id: community_id.clone(),
            members: node_ids.to_vec(),
            summary: options.summary.clone(),
            wiki_id: options.wiki_id.clone(),
        };
        self.communities
            .write()
            .unwrap()
            .insert(community_id, community.clone());
        self.append_journal("declare_community", 0);
        Ok(community)
    }

    async fn get_community(&self, community_id: &CommunityId) -> Result<Community, StoreError> {
        self.communities
            .read()
            .unwrap()
            .get(community_id)
            .cloned()
            .ok_or(StoreError::CommunityNotFound)
    }

    async fn list_communities(&self, wiki_id: &WikiId) -> Result<Vec<Community>, StoreError> {
        // FS-2: consistent with store-wide wiki scoping.
        if !self.wikis.read().unwrap().contains_key(wiki_id) {
            return Err(StoreError::WikiNotFound);
        }
        let guard = self.communities.read().unwrap();
        Ok(guard
            .values()
            .filter(|c| c.wiki_id == *wiki_id)
            .cloned()
            .collect())
    }

    async fn update_community_summary(
        &self,
        community_id: &CommunityId,
        summary: &str,
    ) -> Result<Community, StoreError> {
        if summary.is_empty() {
            return Err(StoreError::ValidationError(
                "community summary must be non-empty".into(),
            ));
        }
        let mut guard = self.communities.write().unwrap();
        let community = guard
            .get_mut(community_id)
            .ok_or(StoreError::CommunityNotFound)?;
        community.summary = summary.to_string();
        let updated = community.clone();
        self.append_journal("update_community_summary", 0);
        Ok(updated)
    }

    async fn set_reference_state(
        &self,
        document_id: &DocumentId,
        source: &NodeId,
        target: &NodeId,
        state: &str,
    ) -> Result<Edge, StoreError> {
        // Case-insensitive parse (the tests drive the vocabulary
        // FRESH/STALE/RESOLVED/BROKEN).
        let new_state = match state.to_ascii_uppercase().as_str() {
            "FRESH" => ReferenceState::Fresh,
            "STALE" => ReferenceState::Stale,
            "RESOLVED" => ReferenceState::Resolved,
            "BROKEN" => ReferenceState::Broken,
            _ => {
                return Err(StoreError::ValidationError(
                    "invalid reference state".into(),
                ))
            }
        };
        // HIGH #5: a *targeted* edit under the store-wide integrity read lock —
        // mutate ONLY the matched reference edge(s)' `state` (and the doc's
        // `updated_at`/`revision`), never wholesale-clobber the graph. The
        // optimistic-concurrency base compare happens inside the shard `write()`
        // (the base is the revision observed under the lock, so any concurrent
        // structural edit already committed is preserved — never dropped).
        let _integrity = self.reference_lock.read().unwrap();
        let mut guard = self.shard_for(document_id).write().unwrap();
        let current = guard
            .docs
            .get(document_id)
            .ok_or(StoreError::DocumentNotFound)?;
        let current = (**current).clone();
        let base = current.revision;
        // A reference edge is identified by its source/target node ids (§4.4.2).
        let reference_edge = |e: &Edge| {
            e.source.1 == *source
                && e.target.1 == *target
                && matches!(
                    e.kind,
                    EdgeKind::Link | EdgeKind::Embed | EdgeKind::Crosslink
                )
        };
        if !current.graph.edges.iter().any(reference_edge) {
            return Err(StoreError::ValidationError(
                "reference edge not found for the given source/target".into(),
            ));
        }
        let updated_edge = Edge {
            state: Some(new_state),
            ..current
                .graph
                .edges
                .iter()
                .find(|e| reference_edge(e))
                .unwrap()
                .clone()
        };
        // Mutate only the matched reference edge(s)' `state`; leave every other
        // node and edge (and any concurrent structural edit already committed)
        // intact.
        let new_graph = Graph {
            nodes: current.graph.nodes.clone(),
            edges: current
                .graph
                .edges
                .iter()
                .map(|e| {
                    if reference_edge(e) {
                        Edge {
                            state: Some(new_state),
                            ..e.clone()
                        }
                    } else {
                        e.clone()
                    }
                })
                .collect(),
        };
        let new_revision = current.revision + 1;
        let updated = Document {
            revision: new_revision,
            graph: new_graph,
            updated_at: iso_now(),
            ..current
        };
        guard.docs.insert(document_id.clone(), Arc::new(updated));
        // Record this bump as a state-annotation-only revision so a concurrent
        // `update_document` carrying the pre-state base can reconcile (both edits
        // land) instead of falsely conflicting. Sidecar write follows the shard
        // write — consistent lock ordering.
        self.state_annotations
            .write()
            .unwrap()
            .entry(document_id.clone())
            .or_default()
            .insert(new_revision);
        self.append_journal("set_reference_state", base);
        Ok(updated_edge)
    }

    async fn resolve_entities(
        &self,
        entity_ids: &[(DocumentId, NodeId)],
        options: &ResolveEntitiesOptions,
    ) -> Result<ResolutionResult, StoreError> {
        if entity_ids.is_empty() {
            return Err(StoreError::ValidationError(
                "entityIds must be non-empty".into(),
            ));
        }
        // FS-1: every entity's document must be known.
        for (doc, _) in entity_ids {
            if !self.shard_for(doc).read().unwrap().docs.contains_key(doc) {
                return Err(StoreError::DocumentNotFound);
            }
        }
        let canonical = match &options.canonical_id {
            Some(c) => {
                if !entity_ids.contains(c) {
                    return Err(StoreError::ValidationError(
                        "canonicalId must be among entityIds".into(),
                    ));
                }
                c.clone()
            }
            // Deterministic default: the first declared entity (§4.2.9.1).
            None => entity_ids[0].clone(),
        };
        let mut merged = Vec::new();
        let mut aliases = Vec::new();
        for e in entity_ids {
            if *e == canonical {
                continue;
            }
            merged.push(EntityPair {
                from: e.clone(),
                to: canonical.clone(),
            });
            aliases.push(Alias {
                alias: e.clone(),
                canonical: canonical.clone(),
            });
            // CRITICAL #2 (GRAPH-OWNS-RELATION-AND-MERGE): record a **durable**,
            // journaled alias→canonical mapping so the effect is persisted, not
            // merely returned as a descriptor.
            self.entity_resolution
                .write()
                .unwrap()
                .insert(e.clone(), canonical.clone());
        }
        self.append_journal("resolve_entities", 0);
        Ok(ResolutionResult {
            merged,
            aliases,
            canonical_id: canonical,
        })
    }

    async fn entity_alias_canonical(
        &self,
        alias: &(DocumentId, NodeId),
    ) -> Result<Option<(DocumentId, NodeId)>, StoreError> {
        // CRITICAL #2: read the durable alias→canonical mapping back.
        Ok(self.entity_resolution.read().unwrap().get(alias).cloned())
    }

    async fn merge_facts(
        &self,
        fact_keys: &[String],
        options: &MergeFactsOptions,
    ) -> Result<Fact, StoreError> {
        if fact_keys.is_empty() {
            return Err(StoreError::ValidationError(
                "factKeys must be non-empty".into(),
            ));
        }
        if !fact_keys.contains(&options.canonical_key) {
            return Err(StoreError::ValidationError(
                "canonicalKey must be among factKeys".into(),
            ));
        }
        let facts = self.fact_store.read().unwrap();
        let by_key = facts
            .get(&options.wiki_id)
            .ok_or_else(|| StoreError::ValidationError("no facts exist in this wiki".into()))?;
        // CRITICAL #1 (GRAPH-OWNS-RELATION-AND-MERGE): a missing (unknown) fact
        // key must NOT be silently dropped or masked to a misleading citation
        // error — the spec fail-state is §4.2.9.1 `DocumentNotFound`.
        for k in fact_keys {
            if !by_key.contains_key(k) {
                return Err(StoreError::DocumentNotFound);
            }
        }
        let canonical = by_key
            .get(&options.canonical_key)
            .ok_or_else(|| StoreError::ValidationError("canonical fact not found".into()))?;
        // Gather the merged facts (all present — verified above).
        let merged_facts: Vec<&Fact> = fact_keys.iter().filter_map(|k| by_key.get(k)).collect();
        // §4.2.9.1/§4.3.2: a merge with conflicting values and no resolution →
        // ConflictError.
        for f in &merged_facts {
            if f.value != canonical.value {
                return Err(StoreError::ConflictError);
            }
        }
        // Union of the merged facts' citations, deduped by first appearance.
        let mut citations: Vec<(DocumentId, NodeId)> = Vec::new();
        for f in &merged_facts {
            for c in &f.citations {
                if !citations.contains(c) {
                    citations.push(c.clone());
                }
            }
        }
        // Minimum-citation invariant (§4.3.2) — unreachable through the compliant
        // surface (create_fact enforces ≥1), guarded for correctness.
        if citations.is_empty() {
            return Err(StoreError::ValidationError(
                "fact requires at least one citation".into(),
            ));
        }
        let updated_at = iso_now();
        let merged = Fact {
            fact_key: options.canonical_key.clone(),
            value: canonical.value.clone(),
            document_id: canonical.document_id.clone(),
            node_id: canonical.node_id.clone(),
            updated_at: updated_at.clone(),
            citations,
        };
        drop(facts);
        // CRITICAL #1: PERSIST the merged canonical fact — union citations,
        // refreshed `updatedAt` — under `fact_store.write()` (not just a returned
        // descriptor). The read-only `get_fact` reads it back.
        self.fact_store
            .write()
            .unwrap()
            .entry(options.wiki_id.clone())
            .or_default()
            .insert(options.canonical_key.clone(), merged.clone());
        self.append_journal("merge_facts", 0);
        Ok(merged)
    }

    async fn get_fact(&self, wiki_id: &WikiId, fact_key: &str) -> Result<Fact, StoreError> {
        // CRITICAL #1 read-back: empty `factKey` → `ValidationError`; a missing
        // fact → `DocumentNotFound` (§4.2.9.1), never a misleading citation error.
        if fact_key.is_empty() {
            return Err(StoreError::ValidationError(
                "fact key must be non-empty".into(),
            ));
        }
        self.fact_store
            .read()
            .unwrap()
            .get(wiki_id)
            .and_then(|by_key| by_key.get(fact_key))
            .cloned()
            .ok_or_else(|| {
                if !self.wikis.read().unwrap().contains_key(wiki_id) {
                    StoreError::ValidationError("unknown wiki".into())
                } else {
                    StoreError::DocumentNotFound
                }
            })
    }

    async fn create_fact(
        &self,
        wiki_id: &WikiId,
        document_id: &DocumentId,
        fact_key: &str,
        value: &str,
        citations: &[(DocumentId, NodeId)],
    ) -> Result<Fact, StoreError> {
        if fact_key.is_empty() || value.is_empty() {
            return Err(StoreError::ValidationError(
                "fact key and value must be non-empty".into(),
            ));
        }
        // §4.3.2 minimum-citation invariant.
        if citations.is_empty() {
            return Err(StoreError::ValidationError(
                "fact requires at least one citation".into(),
            ));
        }
        // Grounding: every citation resolves to a real node (fail-closed §4.3.2a.2).
        for (cdoc, cnode) in citations {
            self.read_node(cdoc, cnode)?;
        }
        // §4.3.1: factKey unique within the Wiki.
        let mut facts = self.fact_store.write().unwrap();
        let by_key = facts.entry(wiki_id.clone()).or_default();
        if by_key.contains_key(fact_key) {
            return Err(StoreError::ConflictError);
        }
        let mut deduped: Vec<(DocumentId, NodeId)> = Vec::new();
        for c in citations.iter() {
            if !deduped.contains(c) {
                deduped.push(c.clone());
            }
        }
        let fact = Fact {
            fact_key: fact_key.to_string(),
            value: value.to_string(),
            document_id: document_id.clone(),
            node_id: NodeId(format!("fact-{fact_key}")),
            updated_at: iso_now(),
            citations: deduped,
        };
        by_key.insert(fact_key.to_string(), fact.clone());
        drop(facts);
        self.append_journal("create_fact", 0);
        Ok(fact)
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

/// GRAPH-OWNS-RELATION-AND-MERGE (HIGH #5): a real cascade — a `Relation` edge
/// whose subject or object node no longer exists in the (replaced) graph (for
/// nodes in `document_id`'s document) is pruned. A removed end-point removes the
/// triple (§4.2.7.5); a cross-document endpoint is left to the owning document's
/// own pruning pass.
fn prune_cascaded_relation_edges(graph: &mut Graph, document_id: &DocumentId) {
    graph.edges.retain(|e| {
        if e.kind != EdgeKind::Relation {
            return true;
        }
        let source_present =
            e.source.0 != *document_id || graph.nodes.iter().any(|n| n.node_id == e.source.1);
        if !source_present {
            return false;
        }
        // If the object lives in this same document, its node must exist here.
        if e.target.0 == *document_id {
            return graph.nodes.iter().any(|n| n.node_id == e.target.1);
        }
        true
    });
}

/// HIGH #5 reconcile: a caller's structural `update_document` that is stale
/// **only** because of a concurrent `set_reference_state` re-applies its graph
/// on top of the current document while preserving the reference-edge states the
/// concurrent `set_reference_state` stamped (so neither edit is lost). Reference
/// edges in `incoming` that match (same `source`/`target`/kind) a reference edge
/// in `current` carrying a `state` adopt that `state`.
fn reconcile_reference_states(incoming: &mut Graph, current: &Graph) {
    for edge in incoming.edges.iter_mut() {
        if !matches!(
            edge.kind,
            EdgeKind::Link | EdgeKind::Embed | EdgeKind::Crosslink
        ) {
            continue;
        }
        if let Some(cur) = current
            .edges
            .iter()
            .find(|c| c.source == edge.source && c.target == edge.target && c.kind == edge.kind)
        {
            if cur.state.is_some() {
                edge.state = cur.state;
            }
        }
    }
}

/// Does a `Relation` edge match an optional `relationType` filter (§4.2.7.3)?
/// A `None` filter matches any relation; a `Some` filter must equal the edge's
/// `relation_type`.
fn triple_matches(edge: &Edge, relation_type: &Option<String>) -> bool {
    match relation_type {
        Some(rt) => edge.relation_type.as_deref() == Some(rt.as_str()),
        None => true,
    }
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
