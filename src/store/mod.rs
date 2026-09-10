//! Document store (§4.1) — the persistence layer.
//!
//! This module defines the **public API surface** of the §4.1 document store
//! exactly as the TestWriter derives it from the canonical behavior contract
//! (`docs/specs/gnosis.md`) plus the concurrency design plan
//! (`docs/research/gnosis-data-structures-concurrency-plan.md`).
//!
//! The **real persistence logic is implemented here** (GREEN): the §4.1.3
//! operations, the §4.1.4 optimistic-concurrency guard (atomic
//! compare-base-revision → apply → bump under the shard write lock, per
//! SHARDED-RWLOCK-STORE), the §4.4 publish/delete gates, the §4.2 graph surface,
//! and the §4.3 fact/citation surface (with the H1 delete-gate fact-citation
//! check and grounding re-verified under the `fact_store` write lock).
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
use std::pin::Pin;

// ---------------------------------------------------------------------------
// §4.1.1 — Document model / §4.1.2 — Wiki model
// ---------------------------------------------------------------------------

/// A stable, globally-unique document identity (UUID v4). Immutable once
/// created; never reused (§4.1.1).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DocumentId(pub String);

impl fmt::Display for DocumentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A Wiki identity (UUID v4) (§4.1.2).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WikiId(pub String);

impl fmt::Display for WikiId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A node identity within a document (§4.1.1 / §4.2.1).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
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
// §4.2 — Knowledge graph: types added by the §4.2 TestWriter
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

/// §7.5 F4 — the pre-joined community retrieval unit returned by
/// `get_community_context` (decision `F4-COMMUNITY-RETRIEVAL`). A **read-only**
/// fusion of `get_community` (§4.2.8.2) + `community_state` (§4.4.1a/§4.4.3):
/// the authoritative manual `summary` + the declared `members` node set + the
/// current `state`. All fields are **verbatim** from the stored records — no
/// derivation, no generation, no mutation. The derives are part of the contract
/// (`PartialEq`/`Eq` let the TestWriter assert determinism/equality;
/// `Serialize`/`Deserialize` consistent with the store types).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommunityContext {
    pub community_id: CommunityId,
    pub wiki_id: WikiId,
    pub summary: String,
    pub members: Vec<(DocumentId, NodeId)>,
    pub state: CommunityState,
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
// §4.3 — Fact/citation tracking surface types (derived from §4.3)
//
// These are the §4.3.1–§4.3.4 surface types built on the **frozen** §4.2 `Fact`
// value (fact_key / value / document_id / node_id / updated_at / citations).
// The Implementer must NOT add fields to `Fact`.
// ---------------------------------------------------------------------------

/// `listFacts(wikiId, {state?, page?, pageSize?})` filters (§4.5.4). A `None`
/// field means "no filter". `page`/`pageSize` are 1-based and bounded (FS-3).
///
/// ## TestWriter resolution (spec ambiguity)
/// The spec's `listFacts` `state?` filter is undocumented and the §4.3.1 `Fact`
/// value carries **no** own `state` (the spec never defines a `FactState`). The
/// reasonable reading — consistent with `listDocuments` (§4.1.3), whose `state?`
/// uses `DocState` — is that the filter matches the state of the **containing
/// Document** (`DocState`): a fact is included iff its document's state equals
/// the filter. The Implementer must match exactly this.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListFactsFilter {
    pub state: Option<DocState>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

/// `listFacts` return shape `{items, total, page, pageSize}` (§4.5.4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactList {
    pub items: Vec<Fact>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

/// `updateFact(factKey, {value, citations})` input (§4.3.2). Updates the value +
/// the (≥1) grounding set. The manual declaration still passes the deterministic
/// gate (§4.3.2a.3) and refreshes `updatedAt`. Empty `citations` →
/// `ValidationError` ("fact requires at least one citation").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateFactRequest {
    pub value: String,
    pub citations: Vec<(DocumentId, NodeId)>,
}

/// The **Propose**-stage output (§4.3.2a.1): `{factKey, value, citations}`. A
/// light LLM (or a manual `createFact`) proposes one of these; the deterministic
/// gate confirms or rejects it before any commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateFact {
    pub fact_key: String,
    pub value: String,
    pub citations: Vec<(DocumentId, NodeId)>,
}

/// The machine-actionable rejection reason `{code, field, message}` (§4.3.2a.2):
/// a structured shape an agent/user can correct and re-submit. `field` names the
/// offending candidate field (`fact_key`/`value`/`citations`), `code` the exact
/// check that failed (schema/grounding/dedup/cross-field).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rejection {
    pub code: String,
    pub field: String,
    pub message: String,
}

/// `proposeCandidateFact(wikiId, candidate)` outcome. A **committed** candidate
/// → `accepted: true` + the `Fact`. A fail-closed rejection → `accepted: false`,
/// no `fact`, and the machine-actionable `Rejection` — the store has **no partial
/// commit** (§4.3.2a.2). Fail-states the spec pins as `StoreError` (dangling
/// citation / duplicate key) surface as `Err`, not as a rejected outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalOutcome {
    pub accepted: bool,
    pub fact: Option<Fact>,
    pub rejection: Option<Rejection>,
}

/// The RAG query mode — the audit-log `mode` (`'flat'|'graph'|'vector'|'hybrid'`,
/// §4.3.4 / §4.5.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QueryMode {
    Flat,
    Graph,
    Vector,
    Hybrid,
}

/// The `filters` recorded in a query audit-log entry (§4.3.4). Mirrors the pinned
/// §4.5.2 filters shape; `None` top-level means "no filters" (the log's `null`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryAuditFilters {
    pub node_kind: Option<NodeKind>,
    pub edge_type: Option<EdgeKind>,
    pub target: Option<(DocumentId, NodeId)>,
    pub state: Option<ReferenceState>,
}

/// A single query audit-log entry (§4.3.4):
/// `{query, filters, mode, resultCount, timestamp, requester}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryAuditEntry {
    pub query: String,
    pub filters: Option<QueryAuditFilters>,
    pub mode: QueryMode,
    pub result_count: u64,
    pub timestamp: String,
    pub requester: String,
}

// ---------------------------------------------------------------------------
// §4.4 — Consistency-enforcement surface types (derived from §4.4)
//
// These are the §4.4.4 consistency-report value and the §4.4.3 re-sync surfaces.
// The Implementer must NOT add fields to the frozen §4.1 `Node`/`Edge`; the §4.4
// re-sync operations update the reference node's `value` (the embed **snapshot**,
// §4.2.2) and the reference edge's `state` in place under the shard write lock.
// ---------------------------------------------------------------------------

/// One row of the §4.4.4 consistency report: `{documentId, nodeId, kind: 'link'|
/// 'embed'|'crosslink', state: 'BROKEN'|'STALE'|'FRESH'|'RESOLVED', target,
/// crossWiki}` for **every reference edge in the wiki**.
///
/// - `document_id` — the **referencing** (source) document of the reference edge.
/// - `node_id` — the reference **node** id (the edge's source node) inside that
///   document.
/// - `kind` — `EdgeKind::Link` | `EdgeKind::Embed` | `EdgeKind::Crosslink`.
/// - `state` — the reference edge's `ReferenceState` (§4.4.2).
/// - `target` — the reference edge's target `(documentId, nodeId)`.
/// - `cross_wiki` — whether the target lives in another wiki.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsistencyReferenceReport {
    pub document_id: DocumentId,
    pub node_id: NodeId,
    pub kind: EdgeKind,
    pub state: ReferenceState,
    pub target: (DocumentId, NodeId),
    pub cross_wiki: bool,
}

/// The §4.4.3 community staleness surface. A `community`'s derived summary is a
/// **projection** of its members (§4.2.8.3): a member node/edge change marks the
/// community `STALE` until `re_derive_community` clears it (§4.4.1a / §4.4.3).
/// Because §4.4.4's report shape pins reference kinds only (`kind: 'link'|'embed'|
/// 'crosslink'`), community staleness — the §4.4.1a third propagator — is surfaced
/// through this accessor (TestWriter resolution of the §4.4.1a-vs-§4.4.4 tension).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommunityState {
    /// The community's summary reflects its current members.
    Fresh,
    /// A member node/edge changed; the summary is stale until re-derived.
    Stale,
}

// ---------------------------------------------------------------------------
// §4.5 / §4.6 — RAG/agent-memory retrieval surface types (added by the §4.5
// TestWriter, RED-first). The canonical contract is `docs/specs/gnosis.md`
// §4.5.1–§4.5.4 and §4.6.1; the concurrency vehicle is
// `docs/research/gnosis-data-structures-concurrency-plan.md`
// (IMMUTABLE-DERIVED-SNAPSHOT, ARC-SHARED-ENGINE, LOCK-ORDER-REF-SHARD-SIDECAR).
//
// The `QueryMode` + `QueryAuditFilters` + `QueryAuditEntry` types that the §4.3
// TestWriter already landed (§4.3.4) are the audit-log `mode`/`filters` shape;
// `RagQueryOptions.filters` reuses `QueryAuditFilters` (it mirrors the pinned
// §4.5.2 filters shape exactly: `nodeKind`/`edgeType`/`target`/`state`).
// ---------------------------------------------------------------------------

// The RAG query mode extension of §4.5.1 — the four modes, mirroring the
// Zodiac `mode: graph|vector|hybrid` surface plus Gnosis's base `flat` mode.
// The existing `QueryMode` enum (`Flat`/`Graph`/`Vector`/`Hybrid`).

/// §4.6.1 — the engine connection state: `READY`/`STARTING`/`DEGRADED`/
/// `UNAVAILABLE`. `READY` = all subsystems up; `STARTING` = booting; `DEGRADED`
/// = a non-core subsystem (e.g. the embedding provider) is down while core
/// store/graph/lexical function works; `UNAVAILABLE` = not running/unreachable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EngineState {
    Ready,
    Starting,
    Degraded,
    Unavailable,
}

/// §4.6.1 — the per-subsystem health of `getEngineStatus`. Each boolean is
/// `available`. The shell surfaces `DEGRADED` distinctly (the embedding/
/// reranker are non-core; store/graph/lexical are core).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineSubsystems {
    pub store: bool,
    pub graph: bool,
    pub lexical: bool,
    pub vector: bool,
    pub embedding: bool,
    pub reranker: bool,
}

/// §4.6.1 — `getEngineStatus() → {state, version, subsystems, lastError?}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineStatus {
    pub state: EngineState,
    pub version: String,
    pub subsystems: EngineSubsystems,
    pub last_error: Option<String>,
}

/// §4.6.1 — `multiQuery?: {enabled, n}`. Query fan-out generates `n` variants
/// (default `n: 3`) and merges by stable `(documentId, nodeId)` identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MultiQueryOptions {
    pub enabled: bool,
    pub n: u64,
}
impl Default for MultiQueryOptions {
    fn default() -> Self {
        MultiQueryOptions {
            enabled: false,
            n: 3,
        }
    }
}

/// §4.6.1 / §4.5.3 — the contextual-compression mode: `none` | `filter`
/// (`binary` keep/drop, Phase 1) | `extract` (fact-value extraction, Phase 2) |
/// `graph` (sub-structure compression, Phase 3). On compressor failure the
/// engine degrades gracefully to uncompressed context (not a query failure).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompressionMode {
    None,
    Filter,
    Extract,
    Graph,
}

/// §4.5.2 / §4.6.1 — `expand?: 'none'|'parent'`. `Parent` returns the parent
/// Document (or node-cluster) for a retrieved child, capped by
/// `maxParentContext` (default 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExpandMode {
    None,
    Parent,
}

/// §4.5.3 / §4.6.1 — `subTaskDag?: {enabled}`. Opt-in inference-time sub-problem
/// DAG decomposition (distinct from the deterministic `graph` walk and from
/// multi-query fan-out). Fail-state `SubTaskDagFailed`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SubTaskDagOptions {
    pub enabled: bool,
}

/// §4.6.1 — `ragQuery(query, {wikiId?, topK?, filters?, mode?, maxHops?,
/// expand?, maxParentContext?, multiQuery?, compression?, hyde?,
/// binaryFirstPass?, binaryCandidatePool?, subTaskDag?})` options.
///
/// `None` means "not supplied" → the engine default. `requester` (the GUI user
/// id or MCP caller identity) is captured here so the §4.3.4 audit entry can
/// record it.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RagQueryOptions {
    pub wiki_id: Option<WikiId>,
    /// Top-K, validated 1–50 (FS-3). Default 10.
    pub top_k: Option<u64>,
    /// §4.5.2 filters (`nodeKind`/`edgeType`/`target`/`state`), `None` = no filter.
    pub filters: Option<QueryAuditFilters>,
    /// Default `'flat'` (§4.5.1).
    pub mode: Option<QueryMode>,
    /// §4.5.2 — 1–5, default 3.
    pub max_hops: Option<u64>,
    /// §4.5.2 / §4.6.1 — default `'none'`.
    pub expand: Option<ExpandMode>,
    /// §4.5.2 — cap on expanded parent hits, default 5.
    pub max_parent_context: Option<u64>,
    /// §4.5.3 — opt-in query fan-out, default disabled.
    pub multi_query: Option<MultiQueryOptions>,
    /// §4.5.3 — default `'none'`.
    pub compression: Option<CompressionMode>,
    /// §4.5.3 — opt-in HyDE wrapper, default false.
    pub hyde: Option<bool>,
    /// §4.5.3a — opt-in coarse-to-fine binary first-pass, default false.
    pub binary_first_pass: Option<bool>,
    /// §4.5.3a — candidate-pool cap, default 10× topK; must be a positive integer.
    pub binary_candidate_pool: Option<u64>,
    /// §4.5.3 — opt-in inference-time sub-task DAG, default disabled.
    pub sub_task_dag: Option<SubTaskDagOptions>,
    /// §4.3.4 — the audit-log requester (GUI user id or MCP caller identity).
    pub requester: Option<String>,
}

/// §4.5.1 — the `source` of a result item / trace: `'local'|'zodiac'`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Source {
    Local,
    Zodiac,
}

/// §4.5.2 — `expand: 'parent'` parent-context payload
/// `{documentId, title, snippet, stale}`. Present only for the capped expanded
/// hits (top `maxParentContext` by score). A `STALE` embed's parent carries
/// `stale: true` so the generator does not trust stale content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RagParent {
    pub document_id: DocumentId,
    pub title: String,
    pub snippet: String,
    pub stale: bool,
}

/// §4.5.1 / §4.6.1 — one item of `RagResult.results`:
/// `{documentId, nodeId, score, snippet, source, parent?, stale?}`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RagResultItem {
    pub document_id: DocumentId,
    pub node_id: NodeId,
    pub score: f64,
    pub snippet: String,
    pub source: Source,
    pub parent: Option<RagParent>,
    pub stale: Option<bool>,
}

/// §4.5.2 — one `{documentId, nodeId, state}` row of the empty-result
/// `blockedBy` list: the `BROKEN`/`STALE` nodes that blocked the walk. A valid
/// state (not an error) when the walk resolves no target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockedBy {
    pub document_id: DocumentId,
    pub node_id: NodeId,
    pub state: ReferenceState,
}

/// §4.3.3 — the **flat**/`vector` probe-trace descriptor
/// `{mode, engine, topK, source}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceDescriptor {
    pub mode: QueryMode,
    pub engine: String,
    pub top_k: u64,
    pub source: Source,
}

/// §4.3.3 — one step of the **graph**-mode ordered `reference`→`fact` path:
/// `{from, to, edge, state}` in traversal order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphTraceStep {
    pub from: (DocumentId, NodeId),
    pub to: (DocumentId, NodeId),
    pub edge: EdgeKind,
    pub state: ReferenceState,
}

/// §4.3.3 — the **hybrid**-mode trace `{mode, engine, legs, topK, source}`
/// (legs = `['graph','vector','lexical']`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HybridTrace {
    pub mode: QueryMode,
    pub engine: String,
    pub legs: Vec<String>,
    pub top_k: u64,
    pub source: Source,
}

/// §4.3.3 — the provenance `trace` carried by every `RagResult`, per-mode:
/// `flat`/`vector` → `TraceDescriptor`, `graph` → `Vec<GraphTraceStep>`,
/// `hybrid` → `HybridTrace`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RagTrace {
    Flat(TraceDescriptor),
    Graph(Vec<GraphTraceStep>),
    Vector(TraceDescriptor),
    Hybrid(HybridTrace),
}

/// §4.5.1 / §4.6.1 — `ragQuery(query) → RagResult`:
/// `{query, results, engine: 'gnosis', citations: [{documentId, nodeId}],
/// trace, blockedBy?}`. `engine` is `'gnosis'`; `blockedBy` is present only for
/// a graph-mode empty result that was blocked by `BROKEN`/`STALE` nodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RagResult {
    pub query: String,
    pub results: Vec<RagResultItem>,
    pub engine: String,
    pub citations: Vec<(DocumentId, NodeId)>,
    pub trace: RagTrace,
    #[serde(default)]
    pub blocked_by: Option<Vec<BlockedBy>>,
}

/// §4.6.1 — one SSE chunk of `ragStream`: `{type: 'result'|'done'|'error', …}`.
/// `Result` carries the full `RagResult` (with `citations`/`trace`); `Error`
/// carries a §4.6.1 fail-state and then the stream closes.
#[derive(Debug, Clone, PartialEq)]
pub enum RagChunk {
    Result(RagResult),
    Done,
    Error(StoreError),
}

/// §4.6.1 — the `ragStream` async SSE surface. The transport is a §5.1 seam;
/// `RagStream` is the async-iterable chunk stream the shell consumes.
pub type RagStream = Pin<Box<dyn futures::Stream<Item = RagChunk> + Send>>;

/// §4.5.4 — `getProfileSummary(wikiId) → {wikiId, summary, regeneratedAt,
/// factCount}`. A **derived document** regenerated from the facts table (a
/// projection, single source of truth, no drift). `factCount` = the number of
/// facts in the wiki's facts table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileSummary {
    pub wiki_id: WikiId,
    pub summary: String,
    pub regenerated_at: String,
    pub fact_count: u64,
}

/// §4.5.3a.1 — a named vector field type: `full` (high-fidelity cosine),
/// `binary` (fast first-pass ANN / Hamming), or `other` (any additional).
/// A node/chunk carries at least the `full` field; binary/other are optional,
/// additive.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FieldType {
    Full,
    Binary,
    Other(String),
}

/// §4.5.3a.2 — the **multi-field vector index**, an immutable derived snapshot
/// keyed by `(documentId, nodeId, fieldType)` (a derived, rebuildable index,
/// never a second source of truth). The frozen §4.1 `Node` cannot carry vector
/// storage, so vector fields live here (this is the `VectorIndex` home).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct VectorIndex {
    pub entries: HashMap<(DocumentId, NodeId, FieldType), Vec<f32>>,
}

/// §4.5.3 — the immutable BM25 **lexical index** snapshot over
/// `factKey`/`title`/`tags`/node text. `built` flags whether the leg is built
/// (a not-built leg → `LexicalIndexUnavailable`). Internals are a §4.5
/// Implementer concern.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LexicalIndex {
    pub built: bool,
}

/// IMMUTABLE-DERIVED-SNAPSHOT — the derived-snapshot container holding the
/// immutable BM25 + vector (and profile) indexes, swapped atomically behind the
/// store's `RwLock<Arc<DerivedIndexes>>`. Query-time reads clone the `Arc`
/// (cheap) and read lock-free; a writer rebuilds and swaps on the epoch feed.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DerivedIndexes {
    pub lexical: Option<LexicalIndex>,
    pub vectors: Option<VectorIndex>,
    pub epoch: u64,
}

/// §4.5.3a.3 — the coarse-to-fine vector-search first-pass options:
/// `binaryFirstPass` (select a candidate pool by binary distance first, then
/// full cosine on the `full` field) + `binaryCandidatePool` (default 10× topK).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FirstPassOptions {
    pub binary_first_pass: bool,
    pub binary_candidate_pool: u64,
}

/// §4.5.3 — the **embedding provider** abstraction: a test-injectable seam over
/// the local embedding/LLM HTTP provider (e.g. Ollama, default
/// `http://127.0.0.1:11434`). Tests inject a wiremock-mocked HTTP impl (no live
/// remote call). `embed` yields `EmbeddingUnavailable` when the provider is
/// unreachable.
pub trait EmbeddingProvider: Send + Sync {
    /// Embed `text` for the vector leg (one consistent model across index and
    /// query — no model drift).
    fn embed(
        &self,
        text: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<f32>, StoreError>> + Send + '_>>;
    /// Provider health probe (feeds `EmbeddingUnavailable` / the `DEGRADED`
    /// engine state).
    fn is_available(&self) -> Pin<Box<dyn Future<Output = bool> + Send + '_>>;
}

/// §4.5.3 — the embedding cache (`HashMap<text, Vec<f32>>`), a read-mostly
/// lookup to avoid re-embedding identical query/doc text.
#[derive(Debug, Default)]
pub struct EmbeddingCache {
    inner: RwLock<HashMap<String, Vec<f32>>>,
}

impl EmbeddingCache {
    /// Read the cached embedding for `text`, if any.
    pub fn get(&self, text: &str) -> Option<Vec<f32>> {
        self.inner.read().unwrap().get(text).cloned()
    }
    /// Insert an embedding for `text`.
    pub fn insert(&self, text: &str, embedding: Vec<f32>) {
        self.inner
            .write()
            .unwrap()
            .insert(text.to_string(), embedding);
    }
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
    // -----------------------------------------------------------------------
    // §4.5 RAG/agent-memory + §4.6 query-surface fail-states (added by the §4.5
    // TestWriter, RED-first). FS numbers reference the §6 catalogue.
    // -----------------------------------------------------------------------
    /// FS-8 (§4.6.1) — `ragQuery`/`ragStream` when the engine is not `READY`
    /// (state `UNAVAILABLE`/`STARTING`).
    EngineUnavailable,
    /// FS-9 (§4.6.1) — `ragQuery`/`ragStream` when the engine returns a
    /// malformed result.
    EngineError,
    /// FS-10 (§4.3.3) — `ragQuery`/`ragStream` when the engine produces a result
    /// without a `trace` (a result set without a trace is not a valid `RagResult`).
    TraceUnavailable,
    /// FS-13 (§4.5.3/§4.6.1) — `ragQuery`/`ragStream` in `mode: 'vector'`/
    /// `'hybrid'` (or with `hyde: true`) when the embedding provider is unreachable.
    EmbeddingUnavailable,
    /// FS-14 (§4.5.3/§4.6.1) — `ragQuery`/`ragStream` in `mode: 'vector'`/
    /// `'hybrid'` when the vector index is not built.
    VectorIndexUnavailable,
    /// FS-15 (§4.5.3/§4.6.1) — `ragQuery`/`ragStream` in `mode: 'hybrid'` when
    /// the BM25 lexical index is not built.
    LexicalIndexUnavailable,
    /// FS-16 (§4.5.3/§4.6.1) — when the reranker model is unavailable and
    /// reranking is requested.
    RerankerUnavailable,
    /// FS-17 (§4.5.3/§4.6.1) — when the compressor fails and cannot degrade to
    /// uncompressed context.
    CompressionFailed,
    /// FS-18 (§4.5.3/§4.6.1) — with `hyde: true` when hypothetical-doc
    /// generation fails.
    HyDEGenerationFailed,
    /// FS-19 (§4.5.3/§4.6.1) — with `multiQuery.enabled` when query expansion fails.
    MultiQueryExpansionFailed,
    /// FS-26 (§4.5.3/§4.6.1) — with `subTaskDag.enabled` when the sub-task DAG
    /// decomposition fails.
    SubTaskDagFailed,
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
            EngineUnavailable => "engine is not ready",
            EngineError => "engine returned a malformed result",
            TraceUnavailable => "result carries no provenance trace",
            EmbeddingUnavailable => "embedding provider unreachable",
            VectorIndexUnavailable => "vector index is not built",
            LexicalIndexUnavailable => "lexical BM25 index is not built",
            RerankerUnavailable => "reranker model unavailable",
            CompressionFailed => "compression failed and could not degrade",
            HyDEGenerationFailed => "HyDE hypothetical-doc generation failed",
            MultiQueryExpansionFailed => "multi-query expansion failed",
            SubTaskDagFailed => "sub-task DAG decomposition failed",
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
    // §4.2 — Knowledge-graph surface (added by the §4.2 TestWriter, GREEN now).
    //
    // Implemented in `Store`. Exact signatures/return shapes/fail-states follow
    // §4.2.5–§4.2.9.
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
    /// descriptor) can read the mapping back. GREEN — implemented in `Store`.
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
    /// Failures: `ValidationError` (empty/whitespace fact key); `WikiNotFound`
    /// (unknown wiki, FS-2); `DocumentNotFound` (no such fact). GREEN — implemented
    /// in `Store`.
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

    // -----------------------------------------------------------------------
    // §4.3 — Fact/citation tracking surface (added by the §4.3 TestWriter,
    // GREEN now; derived from §4.3.1–§4.3.4 and implemented in `Store`).
    // -----------------------------------------------------------------------

    /// `listFacts(wikiId, {state?, page?, pageSize?}) → {items, total, page,
    /// pageSize}` (§4.5.4). Paginated list of `Fact` nodes. Failures:
    /// `WikiNotFound` (FS-2); `ValidationError` (`page < 1`, `pageSize < 1`,
    /// `pageSize > 100`).
    fn list_facts(
        &self,
        wiki_id: &WikiId,
        filter: &ListFactsFilter,
    ) -> impl Future<Output = Result<FactList, StoreError>> + Send;

    /// `updateFact(factKey, {value, citations}) → Fact` (§4.3.2) — updates a
    /// stored fact's `value` + `citations`; the manual declaration still passes
    /// the deterministic gate (§4.3.2a.3). Failures: `DocumentNotFound` (unknown
    /// fact); `ValidationError` (empty `citations` — the §4.3.2 minimum-citation
    /// invariant, "fact requires at least one citation"; and a dangling citation
    /// → "citation does not resolve").
    fn update_fact(
        &self,
        wiki_id: &WikiId,
        fact_key: &str,
        request: &UpdateFactRequest,
    ) -> impl Future<Output = Result<Fact, StoreError>> + Send;

    /// `proposeCandidateFact(wikiId, candidate) → ProposalOutcome` (§4.3.2a) —
    /// the deterministic validation gate. Runs the fail-closed checks (schema
    /// conformance, grounding/provenance, dedup/conflict, cross-field
    /// consistency) and, on a pass, **commits** the fact (`accepted: true` +
    /// `fact`). On a fail it returns `accepted: false` with a machine-actionable
    /// `{code, field, message}` `Rejection` and commits **nothing** (no partial
    /// commit, §4.3.2a.2). Fail-states the spec pins in §4.3.2a.4 surface as
    /// `StoreError`: a dangling citation → `ValidationError` ("citation does not
    /// resolve"); a duplicate `factKey` → `ConflictError` (or routed to entity
    /// resolution).
    fn propose_candidate_fact(
        &self,
        wiki_id: &WikiId,
        candidate: &CandidateFact,
    ) -> impl Future<Output = Result<ProposalOutcome, StoreError>> + Send;

    /// `getQueryAuditLog() → QueryAuditEntry[]` (§4.3.4). Returns the engine-side
    /// audit log — every `ragQuery`/`ragStream` call as `{query, filters, mode,
    /// resultCount, timestamp, requester}`. The **recording** is engine-side
    /// (§4.3.4) and fed by the §4.5 query path; this accessor is the reachable
    /// surface now.
    fn get_query_audit_log(
        &self,
    ) -> impl Future<Output = Result<Vec<QueryAuditEntry>, StoreError>> + Send;

    // -----------------------------------------------------------------------
    // §4.4 — Consistency enforcement surface (added by the §4.4 TestWriter,
    // RED-first). These bodies are COMPILING STUBS — they must NOT implement real
    // staleness logic.
    // -----------------------------------------------------------------------

    /// `getConsistencyReport(wikiId) → ConsistencyReferenceReport[]` (§4.4.4).
    /// Returns one entry (`{documentId, nodeId, kind, state, target, crossWiki}`)
    /// for **every reference edge** (`link`/`embed`/`crosslink`) in the wiki. The
    /// report surfaces the reference propagator (§4.4.1a). Fail-states:
    /// `WikiNotFound` (unknown wiki).
    fn get_consistency_report(
        &self,
        wiki_id: &WikiId,
    ) -> impl Future<Output = Result<Vec<ConsistencyReferenceReport>, StoreError>> + Send;

    /// §4.4.3 re-sync of a single `STALE` embed: updates the reference node's
    /// snapshot (its `value`) to the target's canonical value, flips the embed
    /// edge's state to `FRESH`, and bumps the referencing document's `revision`.
    /// Failures: `DocumentNotFound` (unknown doc); `ValidationError` (the node is
    /// not a reference/embed node, or a non-embed kind).
    fn re_sync_embed(
        &self,
        document_id: &DocumentId,
        node_id: &NodeId,
    ) -> impl Future<Output = Result<Document, StoreError>> + Send;

    /// §4.4.3 re-derive of a `STALE` community. Automatic summary derivation is
    /// PARKED (F4), so re-derive = clear the `STALE` flag (refresh the community
    /// back to `Fresh`); the manual summary is authoritative and never
    /// regenerated (§4.2.8.4). Failures: `CommunityNotFound` (unknown community).
    fn re_derive_community(
        &self,
        community_id: &CommunityId,
    ) -> impl Future<Output = Result<Community, StoreError>> + Send;

    /// §4.4.3/§4.4.1a community-staleness accessor — the §4.4.1a third propagator
    /// surfaced independently of the reference-typed report (whose §4.4.4 shape
    /// pins `link`/`embed`/`crosslink` only). A community incorporating a changed
    /// fact/reference reports `STALE` until re-derived. Failures:
    /// `CommunityNotFound` (unknown community).
    fn community_state(
        &self,
        community_id: &CommunityId,
    ) -> impl Future<Output = Result<CommunityState, StoreError>> + Send;

    // -----------------------------------------------------------------------
    // §4.5 / §4.6 — RAG/agent-memory retrieval surface (added by the §4.5
    // TestWriter, RED-first). These bodies are COMPILING STUBS — they must NOT
    // implement real §4.5 retrieval logic. The Implementer lands that logic to
    // turn the §4.5 tests green.
    // -----------------------------------------------------------------------

    /// `ragQuery(query, options) → RagResult` (§4.6.1). Failures: the full
    /// §4.6.1 catalogue — `EngineUnavailable` (engine not `READY`);
    /// `ValidationError` (empty `query`, `topK < 1`/`> 50`, `maxHops` out of
    /// range 1–5, malformed `filters`, `multiQuery.n` out of range, invalid
    /// `compression`, non-boolean `hyde`/`binaryFirstPass`, non-positive
    /// `binaryCandidatePool`, non-boolean-object `subTaskDag`); `EngineError`;
    /// `TraceUnavailable`; `HopLimitExceeded`; `CycleDetected`;
    /// `EmbeddingUnavailable`; `VectorIndexUnavailable`;
    /// `LexicalIndexUnavailable`; `RerankerUnavailable`; `CompressionFailed`;
    /// `HyDEGenerationFailed`; `MultiQueryExpansionFailed`; `SubTaskDagFailed`.
    /// Every call **appends** a `QueryAuditEntry` (§4.3.4) recording
    /// `{query, filters, mode, resultCount, timestamp, requester}` so
    /// `get_query_audit_log()` returns real entries.
    fn rag_query(
        &self,
        query: &str,
        options: &RagQueryOptions,
    ) -> impl Future<Output = Result<RagResult, StoreError>> + Send;

    /// `ragStream(query, options)` (§4.6.1) — the SSE chunk stream
    /// (`RagChunk::Result`/`Done`/`Error`); each `Result` chunk carries the full
    /// `RagResult`. A mid-stream fail-state is emitted as a `type: 'error'`
    /// `RagChunk::Error`, then the stream closes.
    fn rag_stream(
        &self,
        query: &str,
        options: &RagQueryOptions,
    ) -> impl Future<Output = Result<RagStream, StoreError>> + Send;

    /// `getEngineStatus() → {state, version, subsystems, lastError?}` (§4.6.1).
    fn get_engine_status(&self) -> impl Future<Output = EngineStatus> + Send;

    /// `getProfileSummary(wikiId) → ProfileSummary` (§4.5.4) — a derived doc
    /// regenerated from the facts table (projection, single source of truth).
    /// Failures: `WikiNotFound` (unknown wiki).
    fn get_profile_summary(
        &self,
        wiki_id: &WikiId,
    ) -> impl Future<Output = Result<ProfileSummary, StoreError>> + Send;

    /// §4.5.3 — the lexical BM25 leg over `factKey`/`title`/`tags`/node text.
    /// Failures: `LexicalIndexUnavailable` if the BM25 index is not built.
    fn bm25_search(
        &self,
        wiki_id: &WikiId,
        query: &str,
        top_k: u64,
    ) -> impl Future<Output = Result<Vec<RagResultItem>, StoreError>> + Send;

    /// §4.5.3 / §4.5.3a — the vector leg over the embedding provider + the
    /// multi-field vector index (keyed `(documentId, nodeId, fieldType)`), with
    /// optional coarse-to-fine `binaryFirstPass`. Failures:
    /// `EmbeddingUnavailable` (provider unreachable), `VectorIndexUnavailable`
    /// (vector index not built). A `binaryFirstPass` when the `binary` field is
    /// not built **degrades to the `full`-field search`** (not an error).
    fn vector_search(
        &self,
        wiki_id: &WikiId,
        query: &str,
        top_k: u64,
        provider: &dyn EmbeddingProvider,
        first_pass: &FirstPassOptions,
    ) -> impl Future<Output = Result<Vec<RagResultItem>, StoreError>> + Send;

    /// §7.5 F4 — `getCommunityContext(communityId) → CommunityContext` (decision
    /// `F4-COMMUNITY-RETRIEVAL`). A **read-only** pre-joined read of
    /// `get_community` + `community_state`: returns the authoritative manual
    /// `summary`, the declared `members` node set, and the current `state` as a
    /// single `CommunityContext` value. No derivation, no generation, no
    /// mutation; side-effect-free; deterministic. Fail-state: `CommunityNotFound`
    /// **only** — a `communityId`-keyed accessor derives the wiki from the
    /// community record, so `WikiNotFound` cannot fire (matching `get_community`'s
    /// FS-25). **RED-stage stub** — the body is `unimplemented!()`; the
    /// Implementer lands the real pre-joined read to go green.
    fn get_community_context(
        &self,
        community_id: &CommunityId,
    ) -> impl Future<Output = Result<CommunityContext, StoreError>> + Send;
}

// ---------------------------------------------------------------------------
// Store implementation — GREEN (the §4.1/§4.2/§4.3/§4.4 behavior is implemented:
// sharded RwLock store, optimistic concurrency under the shard write lock,
// publish gate, delete integrity gate — including the §4.3 fact-citation check —
// and the §4.3 fact/citation surface).

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::retrieval::rrf_fuse;

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
    /// The authoritative community-staleness flag (§4.4.1a/§4.4.3). A community's
    /// derived summary is a **projection** of its members: a member node/edge
    /// change or a change to a fact the community incorporates marks it `STALE`
    /// until `re_derive_community` returns it to `Fresh`. Defaults to `Fresh`
    /// (via the reading accessor's `unwrap_or`) so communities declared before
    /// this map holds a row — and the manual-override rule (§4.2.8.4) — read
    /// `Fresh` until a triggering mutation marks them stale.
    community_states: RwLock<HashMap<CommunityId, CommunityState>>,
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
    // -----------------------------------------------------------------------
    // §4.5 — RAG/agent-memory retrieval state (added by the §4.5 TestWriter,
    // RED-first). These are the §4.5 retrieval seams' hosts.
    // -----------------------------------------------------------------------
    /// IMMUTABLE-DERIVED-SNAPSHOT — the immutable derived snapshot (BM25 +
    /// multi-field vector + profile indexes) swapped atomically behind this
    /// `RwLock<Arc<DerivedIndexes>>`. Query-time reads clone the `Arc` and read
    /// lock-free; a writer rebuilds and `swap_snapshot`s (IMMUTABLE-DERIVED-
    /// SNAPSHOT, `docs/research/gnosis-data-structures-concurrency-plan.md` §3).
    derived: RwLock<Arc<DerivedIndexes>>,
    /// §4.6.1 — the engine connection state (`READY`/`STARTING`/`DEGRADED`/
    /// `UNAVAILABLE`). `rag_query`/`rag_stream` honor `EngineUnavailable` when
    /// not `READY` (FS-8). Initialized `Unavailable`; the boot/§4.5 wiring sets it.
    engine_state: RwLock<EngineState>,
    /// §4.6.1 — per-subsystem health feeding `getEngineStatus` + the
    /// `DEGRADED`/`UNAVAILABLE` distinction.
    subsystems: RwLock<EngineSubsystems>,
    /// §4.3.4 — the engine-side **query audit log** (append-only). `rag_query`
    /// appends `QueryAuditEntry`s; `get_query_audit_log()` reads them.
    query_audit_log: RwLock<Vec<QueryAuditEntry>>,
    /// §4.5.3 — the embedding cache (shared across the vector legs).
    #[allow(dead_code)] // wired by the §4.5 Implementer with `vector_search`.
    embedding_cache: RwLock<EmbeddingCache>,
    /// §4.5.3 — the **query-surface embedding-provider seam** (a STUB). The
    /// `ragQuery` vector/hybrid/hyde legs (§4.6.1) consult this provider to run
    /// the dense leg from the query surface (HIGH-1 / HIGH-3, `docs/specs/
    /// gnosis.md` §4.5.3/§4.6.1). Injected via `set_embedding_provider`.
    /// Today this is only a **host** — it stores the provider but the §4.5
    /// Implementer wires it (reads it) inside `vector_query`/`hybrid_query`/
    /// HyDE routing; no retrieval logic lives here. The `EmbeddingProviderSlot`
    /// newtype keeps the `#[derive(Debug)]` on `Store` (the trait object itself
    /// is not `Debug`; the slot prints as an opaque `<injected>` marker).
    #[allow(dead_code)] // stub seam — read by the §4.5 Implementer's vector wire.
    embedding_provider: EmbeddingProviderSlot,
}

/// Stub seam slot so `Store`'s `#[derive(Debug)]` stays intact while carrying
/// `Option<Arc<dyn EmbeddingProvider>>` (a non-`Debug` trait object). Debug
/// prints an opaque marker — the provider itself is never formatted.
#[derive(Default)]
struct EmbeddingProviderSlot(RwLock<Option<Arc<dyn EmbeddingProvider>>>);

impl std::fmt::Debug for EmbeddingProviderSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmbeddingProviderSlot")
            .field("provider", &"<injected>")
            .finish()
    }
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
            community_states: RwLock::new(HashMap::new()),
            fact_store: RwLock::new(HashMap::new()),
            triple_store: RwLock::new(Vec::new()),
            state_annotations: RwLock::new(HashMap::new()),
            entity_resolution: RwLock::new(HashMap::new()),
            known_nodes: RwLock::new(HashMap::new()),
            // §4.5 — fresh store starts with no derived indexes built (the §4.2/
            // §4.4 rebuild-on-epoch feed builds them) and a `UNAVAILABLE` engine
            // (the §4.5 boot wiring transitions it to `READY` once subsystems are
            // up). The audit log starts empty.
            derived: RwLock::new(Arc::new(DerivedIndexes::default())),
            engine_state: RwLock::new(EngineState::Unavailable),
            subsystems: RwLock::new(EngineSubsystems {
                store: true,
                graph: true,
                lexical: true,
                vector: true,
                embedding: true,
                reranker: true,
            }),
            query_audit_log: RwLock::new(Vec::new()),
            embedding_cache: RwLock::new(EmbeddingCache::default()),
            // §4.5.3 — fresh store has no query-surface provider wired; the
            // §4.5 boot/Implementer injects one to run the dense leg from
            // `ragQuery` (the vector/hybrid/hyde legs read this seam).
            embedding_provider: EmbeddingProviderSlot::default(),
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

    // -----------------------------------------------------------------------
    // §4.5 — RAG/agent-memory retrieval infrastructure (RED-first). The
    // `snapshot`/`swap_snapshot` pair is the **immutable-derived-snapshot**
    // concurrency vehicle (IMMUTABLE-DERIVED-SNAPSHOT) — trivial Arc get/set,
    // not §4.5 retrieval logic — so the concurrency test can genuinely contend
    // concurrent readers on the lock-free snapshot with a concurrent rebuild swap.
    // -----------------------------------------------------------------------

    /// IMMUTABLE-DERIVED-SNAPSHOT — clone the current immutable derived snapshot
    /// (a cheap `Arc` bump; the read side is lock-free on the snapshot afterward).
    pub fn snapshot(&self) -> Arc<DerivedIndexes> {
        self.derived.read().unwrap().clone()
    }

    /// IMMUTABLE-DERIVED-SNAPSHOT — atomically swap in a freshly rebuilt derived
    /// snapshot (the epoch-driven rebuild vehicle). Readers in flight keep the
    /// old snapshot — no partial index states, no data races.
    pub fn swap_snapshot(&self, new: DerivedIndexes) {
        *self.derived.write().unwrap() = Arc::new(new);
    }

    /// §4.6.1 test/boot hook — set the engine connection state
    /// (`READY`/`STARTING`/`DEGRADED`/`UNAVAILABLE`).
    pub fn set_engine_state(&self, state: EngineState) {
        *self.engine_state.write().unwrap() = state;
    }

    /// §4.6.1 test/boot hook — set the per-subsystem health feeding
    /// `getEngineStatus`.
    pub fn set_subsystems(&self, subsystems: EngineSubsystems) {
        *self.subsystems.write().unwrap() = subsystems;
    }

    /// §4.5.3 / §4.6.1 — **query-surface embedding-provider seam** (STUB): inject
    /// the embedding provider the `ragQuery` vector/hybrid/hyde legs (§4.6.1)
    /// should consult to run the dense leg from the query surface. A test/boot
    /// seam that only **stores** the provider — the §4.5 Implementer reads it
    /// inside `vector_query`/`hybrid_query`/HyDE routing; no retrieval logic
    /// lives here (HIGH-1 / HIGH-3, `docs/specs/gnosis.md` §4.6.1 FS-13/FS-14).
    #[allow(dead_code)] // stub seam — read by the §4.5 Implementer's vector wire.
    pub fn set_embedding_provider(&self, provider: Arc<dyn EmbeddingProvider>) {
        *self.embedding_provider.0.write().unwrap() = Some(provider);
    }

    /// §4.5.3 / §4.6.1 — the currently injected query-surface embedding
    /// provider (`None` when none is wired). Consumed by the §4.5 Implementer's
    /// vector/hybrid/hyde legs from the `ragQuery` surface.
    #[allow(dead_code)] // stub seam — read by the §4.5 Implementer's vector wire.
    pub fn embedding_provider(&self) -> Option<Arc<dyn EmbeddingProvider>> {
        self.embedding_provider.0.read().unwrap().clone()
    }

    /// §4.3.4 — append a query audit entry (engine-side recording). `rag_query`
    /// calls this after producing a result so `get_query_audit_log()` returns
    /// real entries.
    #[allow(dead_code)] // wired by the §4.5 Implementer in `rag_query`.
    fn append_query_audit(&self, entry: QueryAuditEntry) {
        self.query_audit_log.write().unwrap().push(entry);
    }

    /// §4.3.4 — the current query audit log (read-only).
    fn query_audit(&self) -> Vec<QueryAuditEntry> {
        self.query_audit_log.read().unwrap().clone()
    }

    /// §4.5.3 — read the embedding cache (shared across vector legs).
    #[allow(dead_code)] // wired by the §4.5 Implementer in `vector_search`.
    fn cache_get(&self, text: &str) -> Option<Vec<f32>> {
        self.embedding_cache.read().unwrap().get(text)
    }

    /// §4.5.3 — write the embedding cache (shared across vector legs).
    #[allow(dead_code)] // wired by the §4.5 Implementer in `vector_search`.
    fn cache_put(&self, text: &str, embedding: Vec<f32>) {
        self.embedding_cache
            .write()
            .unwrap()
            .insert(text, embedding);
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

    /// The containing document's `DocState` for a fact (`None` if the document
    /// no longer exists). Used by `list_facts`' §4.5.4 `state?` filter: a fact is
    /// included iff its document's state equals the filter (§4.1.3-consistent
    /// reading of the undocumented filter).
    fn doc_state(&self, document_id: &DocumentId) -> Option<DocState> {
        let guard = self.shard_for(document_id).read().unwrap();
        guard.docs.get(document_id).map(|d| d.state)
    }

    /// Mark every declared community whose member set intersects `locs` as
    /// `STALE` (§4.4.1a / §4.4.3). Called by the fact-update and member-change
    /// propagators. Takes only the `communities`/`community_states` sidecars.
    fn mark_communities_stale(&self, locs: &[(DocumentId, NodeId)]) {
        let ids: Vec<CommunityId> = {
            let comms = self.communities.read().unwrap();
            comms
                .values()
                .filter(|c| c.members.iter().any(|m| locs.contains(m)))
                .map(|c| c.community_id.clone())
                .collect()
        };
        if ids.is_empty() {
            return;
        }
        let mut states = self.community_states.write().unwrap();
        for id in ids {
            states.insert(id, CommunityState::Stale);
        }
    }

    /// Is a reference edge a **snapshot embed** — an `embed`, or a `crosslink`
    /// whose source reference node carries a copied snapshot `value` (§4.2.2)?
    /// A pure `link` (or a `crosslink` with no copied value) is a live
    /// reference resolved at render time, never a snapshot. Drives which
    /// references staleness propagation marks `STALE` as opposed to `BROKEN`/
    /// staying resolved.
    fn is_snapshot_embed(&self, edge: &Edge, doc: &Document) -> bool {
        match edge.kind {
            EdgeKind::Embed => true,
            EdgeKind::Link => false,
            EdgeKind::Crosslink => doc
                .graph
                .nodes
                .iter()
                .any(|n| n.node_id == edge.source.1 && n.value.is_some()),
            _ => false,
        }
    }

    /// Snapshot every committed fact's canonical location → value. Read-only
    /// (§4.3.1 canonical source). Taken **once** per derivation pass so the
    /// report/derivation never holds `fact_store.read()` across shard reads and
    /// never re-locks it per edge — deadlock-free against the fact-commit paths.
    fn fact_canonical_snapshot(&self) -> HashMap<(DocumentId, NodeId), String> {
        let facts = self.fact_store.read().unwrap();
        facts
            .values()
            .flat_map(|by_key| by_key.values())
            .map(|f| ((f.document_id.clone(), f.node_id.clone()), f.value.clone()))
            .collect()
    }

    /// Is a reference target `(docId, nodeId)` **live** — a real, non-archived
    /// node, or a committed fact location (§4.3.1 — a fact node lives in
    /// `fact_store`, not the hosting graph). `src_nodes`/`self_doc` let a
    /// **self-referential** target (a node in the same document, still in the
    /// in-hand graph) be resolved without a store read. Read-only; takes only
    /// per-shard `read()` locks (and the passed snapshot) — the caller must NOT
    /// hold a shard `write()` while calling this (uniform lock order, F1).
    fn target_is_live(
        &self,
        target: &(DocumentId, NodeId),
        self_doc: &DocumentId,
        src_nodes: &[Node],
        facts: &HashMap<(DocumentId, NodeId), String>,
    ) -> bool {
        if &target.0 == self_doc {
            return src_nodes.iter().any(|n| n.node_id == target.1);
        }
        if facts.contains_key(target) {
            return true;
        }
        for shard in self.shards.iter() {
            let guard = shard.read().unwrap();
            if let Some(doc) = guard.docs.get(&target.0) {
                return doc.state != DocState::Archived
                    && doc.graph.nodes.iter().any(|n| n.node_id == target.1);
            }
        }
        false
    }

    /// The canonical value a snapshot embed is compared against: the target
    /// fact's committed `Fact.value`, else the target node's authored `value`.
    /// Read-only (per-shard `read()` only, no `write()` held by the caller).
    fn target_canonical_value(
        &self,
        target: &(DocumentId, NodeId),
        self_doc: &DocumentId,
        src_nodes: &[Node],
        facts: &HashMap<(DocumentId, NodeId), String>,
    ) -> Option<String> {
        if let Some(v) = facts.get(target) {
            return Some(v.clone());
        }
        if &target.0 == self_doc {
            return src_nodes
                .iter()
                .find(|n| n.node_id == target.1)
                .and_then(|n| n.value.clone());
        }
        for shard in self.shards.iter() {
            let guard = shard.read().unwrap();
            if let Some(doc) = guard.docs.get(&target.0) {
                if let Some(n) = doc.graph.nodes.iter().find(|n| n.node_id == target.1) {
                    return n.value.clone();
                }
            }
        }
        None
    }

    /// §4.4.2 / F4 / F5 — derive the **true** reference state of a
    /// `link`/`embed`/`crosslink` edge from the **target's liveness** (exists +
    /// not archived, or a committed fact) and, for a snapshot `embed`, from
    /// **snapshot-vs-canonical**:
    ///
    /// - target missing/archived → `BROKEN` (`link`) / `STALE` (`embed`);
    /// - live target: link → `RESOLVED`; embed with snapshot == canonical →
    ///   `FRESH`, snapshot != canonical → `STALE`.
    ///
    /// `src_nodes` is the source document's node set; `self_doc` its id (for
    /// self-referential targets). Never trusts a caller-supplied state — this is
    /// ground truth from the store (§4.4.1 "every link must resolve to a live,
    /// non-stale target").
    fn derive_reference_state(
        &self,
        edge: &Edge,
        self_doc: &DocumentId,
        src_nodes: &[Node],
        facts: &HashMap<(DocumentId, NodeId), String>,
    ) -> ReferenceState {
        let is_embed = match edge.kind {
            EdgeKind::Embed => true,
            EdgeKind::Link => false,
            EdgeKind::Crosslink => src_nodes
                .iter()
                .any(|n| n.node_id == edge.source.1 && n.value.is_some()),
            _ => false,
        };
        let live = self.target_is_live(&edge.target, self_doc, src_nodes, facts);
        if is_embed {
            if !live {
                return ReferenceState::Stale;
            }
            let snapshot = src_nodes
                .iter()
                .find(|n| n.node_id == edge.source.1)
                .and_then(|n| n.value.clone());
            let canonical = self.target_canonical_value(&edge.target, self_doc, src_nodes, facts);
            match (snapshot, canonical) {
                (Some(s), Some(c)) if s == c => ReferenceState::Fresh,
                (Some(_), Some(_)) => ReferenceState::Stale,
                _ => ReferenceState::Fresh,
            }
        } else if live {
            ReferenceState::Resolved
        } else {
            ReferenceState::Broken
        }
    }

    /// Re-stamp reference-edge states that carry no caller state (`None`) and
    /// are **not** cross-wiki (a cross-wiki target lives in another wiki/store
    /// and cannot be verified locally, so its stored state is preserved) from
    /// the target's liveness / snapshot-vs-canonical (F4 target-change
    /// propagation). Explicit caller states are left untouched. Mutates the
    /// in-hand `graph` in place. Read-only w.r.t. locks.
    fn derive_unstated_reference_states(
        &self,
        graph: &mut Graph,
        self_doc: &DocumentId,
        facts: &HashMap<(DocumentId, NodeId), String>,
    ) {
        for edge in graph.edges.iter_mut() {
            if !matches!(
                edge.kind,
                EdgeKind::Link | EdgeKind::Embed | EdgeKind::Crosslink
            ) {
                continue;
            }
            if edge.cross_wiki {
                continue;
            }
            if edge.state.is_some() {
                continue;
            }
            edge.state = Some(self.derive_reference_state(edge, self_doc, &graph.nodes, facts));
        }
    }

    /// §4.4.3 "On fact update": when a fact's value changes, every **snapshot
    /// embed** edge (same-wiki *and* cross-wiki) whose target is the changed
    /// fact's location becomes `STALE`; a `link` to that fact stays `RESOLVED`
    /// (§4.4.2 live resolution). Also marks any community that incorporates the
    /// fact `STALE` (§4.4.1a). Each affected document's shard is written one at
    /// a time (a shard write is never held while acquiring another) — no
    /// deadlock; the caller holds the mutation's integrity read lock.
    fn propagate_fact_staleness(&self, fact_loc: &(DocumentId, NodeId)) {
        for shard in self.shards.iter() {
            let affected: Vec<DocumentId> = {
                let guard = shard.read().unwrap();
                guard
                    .docs
                    .values()
                    .filter(|doc| {
                        doc.graph.edges.iter().any(|e| {
                            matches!(
                                e.kind,
                                EdgeKind::Link | EdgeKind::Embed | EdgeKind::Crosslink
                            ) && &e.target == fact_loc
                                && self.is_snapshot_embed(e, doc)
                        })
                    })
                    .map(|d| d.document_id.clone())
                    .collect()
            };
            for id in affected {
                let mut guard = shard.write().unwrap();
                let Some(arc) = guard.docs.get(&id).cloned() else {
                    continue;
                };
                let cur = (*arc).clone();
                let edges: Vec<Edge> = cur
                    .graph
                    .edges
                    .iter()
                    .map(|e| {
                        let stales = matches!(
                            e.kind,
                            EdgeKind::Link | EdgeKind::Embed | EdgeKind::Crosslink
                        ) && &e.target == fact_loc
                            && self.is_snapshot_embed(e, &cur);
                        if stales {
                            Edge {
                                state: Some(ReferenceState::Stale),
                                ..e.clone()
                            }
                        } else {
                            e.clone()
                        }
                    })
                    .collect();
                guard.docs.insert(
                    id.clone(),
                    Arc::new(Document {
                        revision: cur.revision + 1,
                        graph: Graph {
                            nodes: cur.graph.nodes.clone(),
                            edges,
                        },
                        updated_at: iso_now(),
                        ..cur.clone()
                    }),
                );
                // F7: propagation is an observable mutation — bump the referencing
                // document's `revision` (above) and record a journal/epoch entry so
                // the change is visible to the §4.1.4 optimistic-concurrency guard
                // and the journal/epoch feeds.
                self.append_journal("propagate_fact_staleness", cur.revision);
            }
        }
        self.mark_communities_stale(std::slice::from_ref(fact_loc));
    }

    /// §4.4.3 "On reference update" / §4.4.5: when a document is archived, every
    /// reference edge elsewhere whose target is in the archived document is
    /// marked — a `link` → `BROKEN`, a snapshot `embed` → `STALE` (§4.4.2). Each
    /// shard is written one at a time (never two shard write guards held
    /// together), so it cannot deadlock with any per-document mutation.
    fn propagate_archived_target(&self, archived_doc: &DocumentId) {
        for shard in self.shards.iter() {
            let affected: Vec<DocumentId> = {
                let guard = shard.read().unwrap();
                guard
                    .docs
                    .values()
                    .filter(|doc| doc.document_id != *archived_doc)
                    .filter(|doc| {
                        doc.graph.edges.iter().any(|e| {
                            matches!(
                                e.kind,
                                EdgeKind::Link | EdgeKind::Embed | EdgeKind::Crosslink
                            ) && &e.target.0 == archived_doc
                        })
                    })
                    .map(|d| d.document_id.clone())
                    .collect()
            };
            for id in affected {
                let mut guard = shard.write().unwrap();
                let Some(arc) = guard.docs.get(&id).cloned() else {
                    continue;
                };
                let cur = (*arc).clone();
                let edges: Vec<Edge> = cur
                    .graph
                    .edges
                    .iter()
                    .map(|e| {
                        let targets_archived = matches!(
                            e.kind,
                            EdgeKind::Link | EdgeKind::Embed | EdgeKind::Crosslink
                        ) && &e.target.0 == archived_doc;
                        if !targets_archived {
                            return e.clone();
                        }
                        Edge {
                            state: Some(if self.is_snapshot_embed(e, &cur) {
                                ReferenceState::Stale
                            } else {
                                ReferenceState::Broken
                            }),
                            ..e.clone()
                        }
                    })
                    .collect();
                guard.docs.insert(
                    id.clone(),
                    Arc::new(Document {
                        revision: cur.revision + 1,
                        graph: Graph {
                            nodes: cur.graph.nodes.clone(),
                            edges,
                        },
                        updated_at: iso_now(),
                        ..cur.clone()
                    }),
                );
                // F7: archiving a reference target's propagation is an observable
                // mutation — bump the referencing document's `revision` and journal.
                self.append_journal("propagate_archived_target", cur.revision);
            }
        }
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
        //
        // F4 / F1 deadlock-free rework: the reference-edge **target-change**
        // derivation (which reads the target's other shard + `fact_store`) runs in
        // a **read-only pre-pass** under `reference_lock.read()` BEFORE any shard
        // `write()` is taken — so no path that derives a state holds a shard write
        // while acquiring another shard's or `fact_store`'s lock (no AB-BA deadlock).
        let _integrity = self.reference_lock.read().unwrap();
        let base_revision = request.base_revision;
        // Phase A (read-only): validate + re-stamp `None` reference states from the
        // new target's liveness (F4 repoint propagation / F2 snapshot-vs-canonical).
        if !valid_provident_graph(&request.graph) {
            return Err(StoreError::ValidationError(
                "graph must be a valid Provident graph (exactly one doc-head and one doc-end)"
                    .into(),
            ));
        }
        let facts = self.fact_canonical_snapshot();
        let mut new_graph = request.graph;
        self.derive_unstated_reference_states(&mut new_graph, document_id, &facts);
        // GRAPH-OWNS-RELATION-AND-MERGE: a real cascade — a `Relation` edge whose
        // subject or object node no longer exists in the (replaced) graph is
        // pruned, so a removed end-point removes the triple (never a zombie that
        // resurfaces on restore, §4.2.7.5).
        prune_cascaded_relation_edges(&mut new_graph, document_id);
        // Phase B: single shard `write()` for the optimistic-concurrency apply.
        let mut guard = self.shard_for(document_id).write().unwrap();
        let current = guard
            .docs
            .get(document_id)
            .ok_or(StoreError::DocumentNotFound)?;
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
        if reconcile_state_annotations {
            // Preserve the reference-edge states a concurrent `set_reference_state`
            // stamped onto edges the caller's graph still carries (these override
            // the derived states from Phase A).
            reconcile_reference_states(&mut new_graph, &current.graph);
        }
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
        // §4.4.1a / §4.4.3 "On community member change": rewriting a node that is
        // a member of a community marks that community `STALE` (its derived
        // summary/projection is out of date until re-derived). Any community
        // holding one of this document's nodes as a member is affected.
        let member_locs: Vec<(DocumentId, NodeId)> = updated
            .graph
            .nodes
            .iter()
            .map(|n| (document_id.clone(), n.node_id.clone()))
            .collect();
        self.mark_communities_stale(&member_locs);
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
        // §4.4.5 fact-citation integrity gate (H1): a **committed** fact whose
        // `citations` reference a node of this document blocks deletion (a delete
        // must never leave a dangling fact citation). This scan runs inside the
        // store-wide integrity `write` lock, and the fact-commit paths
        // (`create_fact`/`update_fact`/`propose_candidate_fact`) take that same
        // lock as `read()` for their entire ground+commit critical section, so no
        // citing fact can be grounded/committed between this scan and the removal
        // below — there is no dangling-citation TOCTOU.
        {
            let facts = self.fact_store.read().unwrap();
            for by_key in facts.values() {
                for f in by_key.values() {
                    if f.citations.iter().any(|(cdoc, _)| cdoc == document_id) {
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
        // §4.4.3 publish gate — applied to every reference edge
        // (`link`/`embed`/`crosslink`) in the graph.
        //
        // F1 (deadlock-free rework): the gate is validated under **SHARD READ** /
        // `fact_store.read()` / community **read** locks only — never holding a
        // shard `write()` while acquiring another shard's or `fact_store`'s lock
        // (the uniform order `reference_lock → shard/fact_store`). Then the
        // `DRAFT→PUBLISHED` transition re-acquires `reference_lock.read()` and the
        // single document's shard `write()`. No path holds a shard write while
        // acquiring another shard or `fact_store`, and no fact-commit holds
        // `fact_store.write()` while acquiring a shard → no AB-BA deadlock.
        //
        // (ii) ALL publish-gate validation under read locks. All reads are
        // synchronous and scoped so no non-Send `std::sync` guard is held across
        // an `.await` (the async future must stay `Send`), and no shard `write()`
        // is held while acquiring another shard / `fact_store`.
        let base = {
            let _integrity = self.reference_lock.read().unwrap();
            let current = {
                let guard = self.shard_for(document_id).read().unwrap();
                guard
                    .docs
                    .get(document_id)
                    .map(|d| (**d).clone())
                    .ok_or(StoreError::DocumentNotFound)?
            };
            let facts = self.fact_canonical_snapshot();
            for edge in &current.graph.edges {
                if !matches!(
                    edge.kind,
                    EdgeKind::Link | EdgeKind::Embed | EdgeKind::Crosslink
                ) {
                    continue;
                }
                // 1. Explicit state gate. A `BROKEN` reference (any kind) or a
                //    `STALE` (unsynced) reference blocks publish — §4.4.2 gives all
                //    three kinds the same reference-state vocabulary (§4.4.3;
                //    adversarial #2).
                if let Some(s) = edge.state {
                    if s == ReferenceState::Broken || s == ReferenceState::Stale {
                        return Err(StoreError::UnresolvedReference);
                    }
                }
                // 2. Derived target-liveness gate (adversarial #4). Store-supplied
                //    `state` is fabricated input, not ground truth — the true state
                //    is derived from the target `(documentId, nodeId)` being live
                //    (exists + not archived) or a committed fact (§4.4.1). Cross-wiki
                //    targets (`cross_wiki: true`) live in another wiki/store and
                //    cannot be verified locally, so their stored state is honored.
                if !edge.cross_wiki
                    && !self.target_is_live(&edge.target, document_id, &current.graph.nodes, &facts)
                {
                    return Err(StoreError::UnresolvedReference);
                }
            }
            // §4.4.1a / §4.4.3 community-staleness gate: a node of this document that
            // is a member of a `STALE` (not yet re-derived) community blocks publish.
            if current.graph.nodes.iter().any(|n| {
                let states = self.community_states.read().unwrap();
                let comms = self.communities.read().unwrap();
                comms.values().any(|c| {
                    states
                        .get(&c.community_id)
                        .copied()
                        .unwrap_or(CommunityState::Fresh)
                        == CommunityState::Stale
                        && c.members
                            .contains(&(document_id.clone(), n.node_id.clone()))
                })
            }) {
                return Err(StoreError::UnresolvedReference);
            }
            if !current.state.can_transition_to(DocState::Published) {
                return Err(StoreError::InvalidState);
            }
            // (iii) release all read locks (`integrity`/`current`/`facts` drop at
            // the end of this block).
            current.revision
        };
        // (iv) re-acquire + single doc's shard write for the DRAFT→PUBLISHED
        // transition (transition re-checks `DocumentNotFound` / `can_transition`).
        let integrity2 = self.reference_lock.read().unwrap();
        let mut guard = self.shard_for(document_id).write().unwrap();
        let updated = transition(&mut guard, document_id, DocState::Published)?;
        drop(guard);
        drop(integrity2);
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
        // §4.4.3 "On reference update": archiving a reference *target* marks a
        // `link` to it `BROKEN` and an `embed` of it `STALE` (§4.4.2). The scan+
        // propagate holds the store-wide integrity read lock so a concurrent
        // reference-edge write cannot race it (same ordering as the delete gate:
        // `reference_lock` → shard). The target's own shard write is released
        // before the cross-shard propagation so no two shard writes are held at
        // once — no deadlock.
        let _integrity = self.reference_lock.read().unwrap();
        let (updated, base) = {
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
            (updated, base)
        };
        self.propagate_archived_target(document_id);
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
    // §4.2 — graph surface (GREEN).
    //
    // Implemented in `Store` (§4.2.5–§4.2.9, GRAPH-OWNS-RELATION-AND-MERGE).
    // They must NOT add fields to the frozen `Node`/`Edge` types (§4.1).
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
            .insert(community_id.clone(), community.clone());
        // §4.4.3: a freshly declared community is `Fresh` — its derived summary
        // reflects its current members (§4.2.8.3, asserted by the §4.4 suite).
        self.community_states
            .write()
            .unwrap()
            .insert(community_id, CommunityState::Fresh);
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
        let mut map = self.entity_resolution.write().unwrap();
        // RESOLVE-ENTITIES-AUTHORITATIVE-OVERWRITE: the chosen canonical is never
        // itself an alias — drop any prior entry so the map stays acyclic (kills
        // reverse-cycles) and the canonical is always a true root.
        map.remove(&canonical);
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
            //
            // RESOLVE-ENTITIES-AUTHORITATIVE-OVERWRITE (path-compression): any
            // prior alias that pointed to `e` is re-pointed directly to the
            // canonical, flattening chains so the map stays acyclic and flat.
            let keys: Vec<_> = map
                .iter()
                .filter(|(k, v)| *k != &canonical && *v == e)
                .map(|(k, _)| k.clone())
                .collect();
            for k in keys {
                map.insert(k, canonical.clone());
            }
            map.insert(e.clone(), canonical.clone());
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
        // H5 (FS-2): an unknown `wikiId` is `WikiNotFound`, checked up front before
        // any other validation — uniformly across the whole fact surface.
        if !self.wikis.read().unwrap().contains_key(wiki_id) {
            return Err(StoreError::WikiNotFound);
        }
        // CRITICAL #1 read-back: empty `factKey` → `ValidationError`; a missing
        // fact → `DocumentNotFound` (§4.2.9.1), never a misleading citation error.
        if fact_key.trim().is_empty() {
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
            .ok_or(StoreError::DocumentNotFound)
    }

    async fn create_fact(
        &self,
        wiki_id: &WikiId,
        document_id: &DocumentId,
        fact_key: &str,
        value: &str,
        citations: &[(DocumentId, NodeId)],
    ) -> Result<Fact, StoreError> {
        // H5 (FS-2): an unknown `wikiId` is `WikiNotFound`, checked up front — a
        // fact must never be committed into a nonexistent wiki.
        if !self.wikis.read().unwrap().contains_key(wiki_id) {
            return Err(StoreError::WikiNotFound);
        }
        // H4 (§4.3.2a.1): whitespace-only `factKey`/`value` are schema-conformance
        // failures — trim before `is_empty`.
        if fact_key.trim().is_empty() || value.trim().is_empty() {
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
        // Grounding + uniqueness + commit are atomic under the store-wide
        // integrity `read` lock: a concurrent `delete_document` holds the `write`
        // side, so it cannot remove a cited node between the grounding reads and
        // the commit — no dangling-citation TOCTOU. Grounding runs **before** the
        // `fact_store.write()` critical section, so the fact-commit paths never
        // hold `fact_store.write()` while acquiring a shard `read()` (F1: uniform
        // lock order `reference_lock → shard → fact_store`, no AB-BA deadlock).
        let _integrity = self.reference_lock.read().unwrap();
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

    // -----------------------------------------------------------------------
    // §4.3 — fact/citation surface (GREEN).
    //
    // These operations are implemented (not RED stubs) and must NOT add fields
    // to the frozen `Fact` value type. The §4.3.2a.2 deterministic gate and the
    // §4.3.1 wiki-unique `factKey` invariant are enforced here, with grounding
    // re-verified under the `fact_store` write lock (H1) so a concurrent
    // `delete_document` cannot leave a dangling citation.
    // -----------------------------------------------------------------------

    async fn list_facts(
        &self,
        wiki_id: &WikiId,
        filter: &ListFactsFilter,
    ) -> Result<FactList, StoreError> {
        // FS-2 (§4.5.4): listFacts on an unknown `wikiId` → `WikiNotFound`.
        if !self.wikis.read().unwrap().contains_key(wiki_id) {
            return Err(StoreError::WikiNotFound);
        }
        let page = filter.page.unwrap_or(1);
        let page_size = filter.page_size.unwrap_or(20);
        // FS-3 (§4.5.4): page<1 / pageSize<1 / pageSize>100 → `ValidationError`.
        if page < 1 || !(1..=100).contains(&page_size) {
            return Err(StoreError::ValidationError(
                "page must be >= 1 and page_size must be in 1..=100".into(),
            ));
        }
        let facts = self.fact_store.read().unwrap();
        let by_key = match facts.get(wiki_id) {
            Some(m) => m,
            None => {
                return Ok(FactList {
                    items: vec![],
                    total: 0,
                    page,
                    page_size,
                })
            }
        };
        // §4.5.4 `state?` filter (TestWriter resolution): the state of the fact's
        // **containing Document** (`DocState`). An unfiltered `None` matches all.
        let mut matched: Vec<Fact> = Vec::new();
        for f in by_key.values() {
            if let Some(state) = filter.state {
                if self.doc_state(&f.document_id) != Some(state) {
                    continue;
                }
            }
            matched.push(f.clone());
        }
        // Deterministic pagination ordering (§4.5.4 paginates a table).
        matched.sort_by(|a, b| a.fact_key.cmp(&b.fact_key));
        let total = matched.len();
        let start = (page as usize - 1).saturating_mul(page_size as usize);
        if start >= total {
            return Ok(FactList {
                items: vec![],
                total: total as u64,
                page,
                page_size,
            });
        }
        let end = (start + page_size as usize).min(total);
        Ok(FactList {
            items: matched[start..end].to_vec(),
            total: total as u64,
            page,
            page_size,
        })
    }

    async fn update_fact(
        &self,
        wiki_id: &WikiId,
        fact_key: &str,
        request: &UpdateFactRequest,
    ) -> Result<Fact, StoreError> {
        // H5 (FS-2): an unknown `wikiId` is `WikiNotFound`, checked up front.
        if !self.wikis.read().unwrap().contains_key(wiki_id) {
            return Err(StoreError::WikiNotFound);
        }
        // H2 (§4.3.2a.1): an empty or whitespace-only `value` is a schema
        // conformance failure — reject before applying.
        if request.value.trim().is_empty() {
            return Err(StoreError::ValidationError(
                "fact value must be non-empty".into(),
            ));
        }
        // §4.3.2 minimum-citation invariant — empty `citations` is a
        // `ValidationError` regardless of the fact's existence.
        if request.citations.is_empty() {
            return Err(StoreError::ValidationError(
                "fact requires at least one citation".into(),
            ));
        }
        // Grounding + apply are atomic under the integrity `read` lock (a
        // concurrent `delete_document` holds the `write` side, so a cited node
        // cannot be removed between the grounding reads and the apply — no
        // dangling-citation TOCTOU). Grounding runs **before** the `fact_store`
        // write lock, so `update_fact` never holds `fact_store.write()` while
        // acquiring a shard `read()` (F1: uniform lock order, no deadlock).
        let _integrity = self.reference_lock.read().unwrap();
        // Grounding (§4.3.2a.2): every citation must resolve to a real node.
        for (cdoc, cnode) in &request.citations {
            if self.read_node(cdoc, cnode).is_err() {
                return Err(StoreError::ValidationError(
                    "citation does not resolve".into(),
                ));
            }
        }
        // Unknown fact → `DocumentNotFound` (§4.3.2). Compare-then-apply under the
        // `fact_store` write lock so the update is atomic with the existence check.
        let mut facts = self.fact_store.write().unwrap();
        let by_key = match facts.get_mut(wiki_id) {
            Some(m) => m,
            None => return Err(StoreError::DocumentNotFound),
        };
        let existing = match by_key.get_mut(fact_key) {
            Some(f) => f,
            None => return Err(StoreError::DocumentNotFound),
        };
        // Dedupe citations by first appearance (§4.3.2 grounding set).
        let mut deduped: Vec<(DocumentId, NodeId)> = Vec::new();
        for c in request.citations.iter() {
            if !deduped.contains(c) {
                deduped.push(c.clone());
            }
        }
        existing.value = request.value.clone();
        existing.citations = deduped;
        existing.updated_at = iso_now();
        let updated = existing.clone();
        drop(facts);
        // §4.4.3 "On fact update": the fact's canonical value changed, so every
        // snapshot embed (same-wiki and cross-wiki) of this fact becomes `STALE`,
        // links stay `RESOLVED`, and every community incorporating the fact
        // becomes `STALE`. Runs under the mutation's integrity read lock; each
        // affected shard is written one at a time (no two shard writes held).
        self.propagate_fact_staleness(&(updated.document_id.clone(), updated.node_id.clone()));
        self.append_journal("update_fact", 0);
        Ok(updated)
    }

    async fn propose_candidate_fact(
        &self,
        wiki_id: &WikiId,
        candidate: &CandidateFact,
    ) -> Result<ProposalOutcome, StoreError> {
        // H5 (FS-2): an unknown `wikiId` is `WikiNotFound`, checked up front — a
        // candidate is never committed into a nonexistent wiki.
        if !self.wikis.read().unwrap().contains_key(wiki_id) {
            return Err(StoreError::WikiNotFound);
        }
        // §4.3.2a.2 deterministic fail-closed validation gate. Schema-conformance
        // failures (other than those the spec pins as `StoreError`) return
        // `accepted:false` + a machine-actionable `Rejection` and commit nothing.
        // H4: whitespace-only `factKey`/`value` are schema failures — trim first.
        if candidate.fact_key.trim().is_empty() {
            return Ok(rejected_outcome(
                "schema",
                "fact_key",
                "fact key must be non-empty",
            ));
        }
        if candidate.value.trim().is_empty() {
            return Ok(rejected_outcome(
                "schema",
                "value",
                "fact value must be non-empty",
            ));
        }
        // §4.3.2 minimum-citation invariant (empty `citations`).
        if candidate.citations.is_empty() {
            return Ok(rejected_outcome(
                "schema",
                "citations",
                "fact requires at least one citation",
            ));
        }
        // Grounding + commit are atomic under the integrity `read` lock (a
        // concurrent `delete_document` holds the `write` side, so a cited node
        // cannot be removed between the grounding reads and the commit — no
        // dangling-citation TOCTOU). Grounding runs **before** the `fact_store`
        // write lock, so commit paths never hold `fact_store.write()` while
        // acquiring a shard `read()` (F1: uniform lock order, no deadlock).
        let _integrity = self.reference_lock.read().unwrap();
        // Grounding/provenance (§4.3.2a.4): a dangling citation → `Err`
        // `ValidationError("citation does not resolve")`, nothing committed.
        for (cdoc, cnode) in &candidate.citations {
            if self.read_node(cdoc, cnode).is_err() {
                return Err(StoreError::ValidationError(
                    "citation does not resolve".into(),
                ));
            }
        }
        // The commit — compare-then-insert under the `fact_store` write lock
        // (atomic vs. concurrent same-key candidates; §4.3.2a.4 duplicate key →
        // `ConflictError`; no partial commit). The whole gate above ran before
        // any write, so a rejected candidate leaves the store untouched.
        let mut facts = self.fact_store.write().unwrap();
        let by_key = facts.entry(wiki_id.clone()).or_default();
        if by_key.contains_key(&candidate.fact_key) {
            return Err(StoreError::ConflictError);
        }
        // Dedupe citations by first appearance (§4.3.2 grounding set).
        let mut deduped: Vec<(DocumentId, NodeId)> = Vec::new();
        for c in candidate.citations.iter() {
            if !deduped.contains(c) {
                deduped.push(c.clone());
            }
        }
        let fact = Fact {
            fact_key: candidate.fact_key.clone(),
            value: candidate.value.clone(),
            // The fact belongs to the document its first citation grounds in.
            document_id: candidate.citations[0].0.clone(),
            node_id: NodeId(format!("fact-{}", candidate.fact_key)),
            updated_at: iso_now(),
            citations: deduped,
        };
        by_key.insert(candidate.fact_key.clone(), fact.clone());
        drop(facts);
        self.append_journal("propose_candidate_fact", 0);
        Ok(ProposalOutcome {
            accepted: true,
            fact: Some(fact),
            rejection: None,
        })
    }

    async fn get_query_audit_log(&self) -> Result<Vec<QueryAuditEntry>, StoreError> {
        // §4.3.4 accessor. The `[{query, filters, mode, resultCount, timestamp,
        // requester}]` recording is engine-side and fed by the §4.5 query path;
        // every committed `rag_query`/`rag_stream` appends a `QueryAuditEntry`
        // (via `append_query_audit`) so this returns real entries. Empty on a
        // fresh store (no queries yet).
        Ok(self.query_audit())
    }

    // -----------------------------------------------------------------------
    // §4.4 — Consistency enforcement surface — COMPILING STUBS (RED).
    //
    // These stub bodies intentionally implement NO §4.4 staleness logic — the
    // TDD gate is RED-first and the Implementer lands the real logic next. They
    // exist ONLY so the §4.4 TestWriter's `tests/consistency_integration.rs`
    // compiles against the extended `RagStore` surface and FAILS at runtime.
    // -----------------------------------------------------------------------

    async fn get_consistency_report(
        &self,
        wiki_id: &WikiId,
    ) -> Result<Vec<ConsistencyReferenceReport>, StoreError> {
        // FS-2 (§4.4.4): unknown wiki → `WikiNotFound`.
        if !self.wikis.read().unwrap().contains_key(wiki_id) {
            return Err(StoreError::WikiNotFound);
        }
        // One row per reference edge (`link`/`embed`/`crosslink`) whose **source
        // document** is in `wiki_id` — including `crosslink` edges whose target
        // lives in another wiki. `state` is the reference edge's stored
        // `ReferenceState`, or — for an edge stored with `state: None` (F2/F4/F5 a
        // fabricatable default would lie) — derived from the **target's liveness**
        // and (for a snapshot embed) **snapshot-vs-canonical** (§4.4.2/§4.4.3).
        // Read-only: `fact_store.read()` is snapshot into a local map first (then
        // released) and only per-shard `read()`s are held — deadlock-free.
        let facts = self.fact_canonical_snapshot();
        let mut report = Vec::new();
        for shard in self.shards.iter() {
            let guard = shard.read().unwrap();
            for doc in guard.docs.values() {
                if doc.wiki_id != *wiki_id {
                    continue;
                }
                for e in doc.graph.edges.iter() {
                    if !matches!(
                        e.kind,
                        EdgeKind::Link | EdgeKind::Embed | EdgeKind::Crosslink
                    ) {
                        continue;
                    }
                    let state = match e.state {
                        Some(s) => s,
                        None => self.derive_reference_state(
                            e,
                            &doc.document_id,
                            &doc.graph.nodes,
                            &facts,
                        ),
                    };
                    report.push(ConsistencyReferenceReport {
                        document_id: doc.document_id.clone(),
                        node_id: e.source.1.clone(),
                        kind: e.kind,
                        state,
                        target: e.target.clone(),
                        cross_wiki: e.cross_wiki,
                    });
                }
            }
        }
        Ok(report)
    }

    async fn re_sync_embed(
        &self,
        document_id: &DocumentId,
        node_id: &NodeId,
    ) -> Result<Document, StoreError> {
        // §4.4.3: re-sync a `STALE` embed — update the reference node's snapshot
        // (`value`) to the target fact's canonical `Fact.value` (found in
        // `fact_store` under the embed edge's target), flip the embed edge to
        // `FRESH`, and bump the referencing document's `revision`.
        //
        // Locking: the canonical fact values are snapshotted into a local map
        // under `fact_store.read()` **before** the shard write is taken, so the
        // shard write and the `fact_store` read are never held together — this
        // avoids the fact-update ↔ re-sync deadlock (one holding `fact_store`
        // write while scanning shards, the other holding a shard write while
        // reading `fact_store`).
        let _integrity = self.reference_lock.read().unwrap();
        let fact_values: HashMap<(DocumentId, NodeId), String> = {
            let facts = self.fact_store.read().unwrap();
            facts
                .values()
                .flat_map(|by_key| by_key.values())
                .map(|f| ((f.document_id.clone(), f.node_id.clone()), f.value.clone()))
                .collect()
        };
        let mut guard = self.shard_for(document_id).write().unwrap();
        let current = guard
            .docs
            .get(document_id)
            .ok_or(StoreError::DocumentNotFound)?;
        let current = (**current).clone();
        let base = current.revision;
        // The node must be a `Reference` node with an embed (snapshot) edge.
        let node = current.graph.nodes.iter().find(|n| &n.node_id == node_id);
        let node_ok = matches!(node, Some(n) if n.kind == NodeKind::Reference);
        let embed_edge = current.graph.edges.iter().find(|e| {
            e.source == (document_id.clone(), node_id.clone())
                && matches!(e.kind, EdgeKind::Embed | EdgeKind::Crosslink)
        });
        let (Some(edge), true) = (embed_edge, node_ok) else {
            return Err(StoreError::ValidationError(
                "node is not an embed reference node".into(),
            ));
        };
        let canonical = fact_values.get(&edge.target).cloned().ok_or_else(|| {
            StoreError::ValidationError("embed target has no canonical value to re-sync".into())
        })?;
        let nodes: Vec<Node> = current
            .graph
            .nodes
            .iter()
            .map(|n| {
                if &n.node_id == node_id {
                    Node {
                        value: Some(canonical.clone()),
                        ..n.clone()
                    }
                } else {
                    n.clone()
                }
            })
            .collect();
        let edges: Vec<Edge> = current
            .graph
            .edges
            .iter()
            .map(|e| {
                if e.source == (document_id.clone(), node_id.clone())
                    && matches!(e.kind, EdgeKind::Embed | EdgeKind::Crosslink)
                {
                    Edge {
                        state: Some(ReferenceState::Fresh),
                        ..e.clone()
                    }
                } else {
                    e.clone()
                }
            })
            .collect();
        let updated = Document {
            revision: base + 1,
            graph: Graph { nodes, edges },
            updated_at: iso_now(),
            ..current
        };
        guard
            .docs
            .insert(document_id.clone(), Arc::new(updated.clone()));
        self.append_journal("re_sync_embed", base);
        Ok(updated)
    }

    async fn re_derive_community(
        &self,
        community_id: &CommunityId,
    ) -> Result<Community, StoreError> {
        // §4.4.3: re-derive a `STALE` community. Automatic summary derivation is
        // PARKED (F4), so re-derive = clear the `STALE` flag (refresh to `Fresh`);
        // the manual summary is authoritative and never regenerated (§4.2.8.4).
        let community = self
            .communities
            .read()
            .unwrap()
            .get(community_id)
            .cloned()
            .ok_or(StoreError::CommunityNotFound)?;
        self.community_states
            .write()
            .unwrap()
            .insert(community_id.clone(), CommunityState::Fresh);
        self.append_journal("re_derive_community", 0);
        Ok(community)
    }

    async fn community_state(
        &self,
        community_id: &CommunityId,
    ) -> Result<CommunityState, StoreError> {
        // FS: unknown community → `CommunityNotFound`.
        if !self.communities.read().unwrap().contains_key(community_id) {
            return Err(StoreError::CommunityNotFound);
        }
        // §4.4.1a / §4.4.3: `Fresh` by default (a community whose staleness was
        // never triggered — including one declared before the §4.4 map was
        // populated) until a member change / incorporated-fact change marks it.
        Ok(self
            .community_states
            .read()
            .unwrap()
            .get(community_id)
            .copied()
            .unwrap_or(CommunityState::Fresh))
    }

    // -----------------------------------------------------------------------
    // §4.5 / §4.6 — RAG/agent-memory retrieval surface (GREEN).
    //
    // Real §4.5 retrieval logic: validation (FS-3), the engine-readiness gate
    // (FS-8), the four query modes (§4.5.1), the multi-hop walk (§4.5.2), the
    // lexical BM25 + vector legs (§4.5.3/§4.5.3a), provenance traces (§4.3.3),
    // the query-audit recording (§4.3.4), the agent-memory profile summary
    // (§4.5.4), and the SSE stream surface (§4.6.1). Pure helpers live in the
    // `impl Store` block / module fns below.
    // -----------------------------------------------------------------------

    async fn rag_query(
        &self,
        query: &str,
        options: &RagQueryOptions,
    ) -> Result<RagResult, StoreError> {
        // §4.6.1. Validate options (FS-3) then honor EngineUnavailable (FS-8).
        Store::validate_rag_options(query, options)?;
        let mode = options.mode.unwrap_or(QueryMode::Flat);
        // Engine-readiness gate. `EngineUnavailable` when the engine is not
        // `READY` and the mode needs the retrieval subsystems. `graph` mode is a
        // deterministic local `reference`→`fact` walk that runs on the core store
        // alone, so it is not blocked by the readiness gate (it is bounded by its
        // own `maxHops`/cycle fail-states).
        if mode != QueryMode::Graph && *self.engine_state.read().unwrap() != EngineState::Ready {
            return Err(StoreError::EngineUnavailable);
        }
        let top_k = options.top_k.unwrap_or(10) as usize;
        let max_hops = options.max_hops.unwrap_or(3);

        // §4.5.3 multi-query fan-out (HIGH-4): when `multiQuery: {enabled, n}` is
        // on, deterministically expand the query into `n` variants and run the
        // chosen mode for each, merging results by stable `(documentId, nodeId)`
        // identity (first-seen order). An expansion failure → `MultiQueryExpansionFailed`.
        let fan: bool = options.multi_query.map(|m| m.enabled).unwrap_or(false);
        let variants: Vec<String> = if fan {
            let n = options.multi_query.unwrap().n;
            self.expand_query_variants(query, n, options)?
        } else {
            vec![query.to_string()]
        };

        let mut merged_results: Vec<RagResultItem> = Vec::new();
        let mut merged_citations: Vec<(DocumentId, NodeId)> = Vec::new();
        let mut seen: std::collections::HashSet<(DocumentId, NodeId)> =
            std::collections::HashSet::new();
        let mut seen_cit: std::collections::HashSet<(DocumentId, NodeId)> =
            std::collections::HashSet::new();
        let mut trace: Option<RagTrace> = None;
        let mut blocked_any: Vec<BlockedBy> = Vec::new();
        for vq in &variants {
            let (res, tr, cites, blocked) = match mode {
                QueryMode::Flat => self.flat_query(vq, top_k, options)?,
                QueryMode::Vector => self.vector_query(vq, top_k, options).await?,
                QueryMode::Hybrid => self.hybrid_query(vq, top_k, options).await?,
                QueryMode::Graph => {
                    let (r, steps, c, b) = self.graph_query(
                        options.wiki_id.as_ref(),
                        max_hops,
                        options.filters.as_ref(),
                        top_k,
                    )?;
                    (r, RagTrace::Graph(steps), c, b)
                }
            };
            if trace.is_none() {
                trace = Some(tr);
            }
            for r in res {
                if seen.insert((r.document_id.clone(), r.node_id.clone())) {
                    merged_results.push(r);
                }
            }
            for c in cites {
                if seen_cit.insert(c.clone()) {
                    merged_citations.push(c);
                }
            }
            if let Some(b) = blocked {
                blocked_any.extend(b);
            }
        }
        let trace = trace.expect("every mode produces a trace");

        // §4.5.3 compression (HIGH-4): post-process the retrieved snippets. The
        // compressor degrades gracefully to uncompressed context on its own
        // failure (NOT a query failure) — `filter` changes the returned snippet
        // set, `extract`/`graph` run the higher-phase transforms.
        let compression = options.compression.unwrap_or(CompressionMode::None);
        let results = self.apply_compression(merged_results, query, compression);
        let blocked_by = if results.is_empty() && !blocked_any.is_empty() {
            Some(blocked_any)
        } else {
            None
        };
        let results = self.apply_expand(results, options);
        let result = RagResult {
            query: query.to_string(),
            results,
            engine: "gnosis".to_string(),
            citations: merged_citations,
            trace,
            blocked_by,
        };
        // §4.3.4 — every ragQuery appends an audit entry.
        self.append_query_audit(QueryAuditEntry {
            query: query.to_string(),
            filters: options.filters.clone(),
            mode,
            result_count: result.results.len() as u64,
            timestamp: iso_now(),
            requester: options.requester.clone().unwrap_or_default(),
        });
        Ok(result)
    }

    async fn rag_stream(
        &self,
        query: &str,
        options: &RagQueryOptions,
    ) -> Result<RagStream, StoreError> {
        // §4.6.1. Validate + honor EngineUnavailable when not READY (FS-8),
        // mirroring `rag_query`'s mode-aware readiness gate. On success the stream
        // emits the full `RagResult` as a `Result` chunk then `Done`; a fail-state
        // is emitted as an `Error` chunk then the stream closes.
        Store::validate_rag_options(query, options)?;
        if options.mode.unwrap_or(QueryMode::Flat) != QueryMode::Graph
            && *self.engine_state.read().unwrap() != EngineState::Ready
        {
            return Err(StoreError::EngineUnavailable);
        }
        let outcome = self.rag_query(query, options).await;
        let chunks = futures::stream::iter(match &outcome {
            Ok(result) => vec![RagChunk::Result(result.clone()), RagChunk::Done],
            Err(e) => vec![RagChunk::Error(e.clone()), RagChunk::Done],
        });
        Ok(Box::pin(chunks))
    }

    async fn get_engine_status(&self) -> EngineStatus {
        // §4.6.1 — surface the stored engine state + per-subsystem health.
        let state = *self.engine_state.read().unwrap();
        let subsystems = self.subsystems.read().unwrap().clone();
        EngineStatus {
            state,
            version: env!("CARGO_PKG_VERSION").to_string(),
            subsystems,
            last_error: if state == EngineState::Degraded {
                Some("a non-core subsystem (embedding/reranker) is unavailable".into())
            } else {
                None
            },
        }
    }

    async fn get_profile_summary(&self, wiki_id: &WikiId) -> Result<ProfileSummary, StoreError> {
        // §4.5.4 — a derived doc regenerated from the wiki's facts table (a
        // projection, single source of truth, no drift). Regenerated on demand
        // (and therefore on fact change). FS-2: unknown wiki → `WikiNotFound`.
        if !self.wikis.read().unwrap().contains_key(wiki_id) {
            return Err(StoreError::WikiNotFound);
        }
        let facts: Vec<Fact> = self
            .fact_store
            .read()
            .unwrap()
            .get(wiki_id)
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default();
        let summary = facts
            .iter()
            .map(|f| format!("{}: {}", f.fact_key, f.value))
            .collect::<Vec<_>>()
            .join("; ");
        Ok(ProfileSummary {
            wiki_id: wiki_id.clone(),
            summary,
            regenerated_at: iso_now(),
            fact_count: facts.len() as u64,
        })
    }

    async fn bm25_search(
        &self,
        wiki_id: &WikiId,
        query: &str,
        top_k: u64,
    ) -> Result<Vec<RagResultItem>, StoreError> {
        // §4.5.3 — the lexical BM25 leg over factKey/title/tags/node text. The
        // index is derived from the store's contents; a wiki with nothing to
        // index is an unbuilt leg → `LexicalIndexUnavailable` (FS-15).
        if !self.wiki_has_documents(wiki_id) {
            return Err(StoreError::LexicalIndexUnavailable);
        }
        Ok(self.lexical_rank(Some(wiki_id), query, top_k as usize, None))
    }

    async fn vector_search(
        &self,
        wiki_id: &WikiId,
        query: &str,
        top_k: u64,
        provider: &dyn EmbeddingProvider,
        first_pass: &FirstPassOptions,
    ) -> Result<Vec<RagResultItem>, StoreError> {
        // §4.5.3a — multi-field vector leg. FS-3: invalid `binaryCandidatePool`
        // (`0` with `binaryFirstPass`) → `ValidationError`. FS-14: no vector
        // index built → `VectorIndexUnavailable`. FS-13: unreachable provider →
        // `EmbeddingUnavailable`. §4.5.3a.4: `binaryFirstPass` with no `binary`
        // field degrades to the `full` search (not an error).
        if first_pass.binary_first_pass && first_pass.binary_candidate_pool == 0 {
            return Err(StoreError::ValidationError(
                "binaryCandidatePool must be a positive integer".into(),
            ));
        }
        let index = match self.snapshot().vectors.as_ref() {
            Some(v) => v.clone(),
            None => return Err(StoreError::VectorIndexUnavailable),
        };
        let qvec = self.embed_text(query, provider).await?;
        Ok(self.vector_leg(&index, Some(wiki_id), &qvec, top_k as usize, first_pass))
    }

    async fn get_community_context(
        &self,
        _community_id: &CommunityId,
    ) -> Result<CommunityContext, StoreError> {
        // §7.5 F4 — RED-stage stub. The Implementer lands the real pre-joined
        // read of `get_community` + `community_state` (CommunityNotFound only)
        // to go green. The `unimplemented!()` body makes every F4 assertion fail
        // at runtime (the compile-with-stubs red set).
        unimplemented!("get_community_context is a RED-stage stub (F4)")
    }
}

// ---------------------------------------------------------------------------
// §4.5 / §4.6 — retrieval helper methods (GREEN). Pure, deterministic helpers
// the `RagStore` §4.5 surface routes through.
// ---------------------------------------------------------------------------

impl Store {
    /// §4.6.1 / FS-3 — validate the `ragQuery`/`ragStream` options before any
    /// work. Mirrors the §4.6.1 fail-state catalogue for the representable cases.
    fn validate_rag_options(query: &str, options: &RagQueryOptions) -> Result<(), StoreError> {
        if query.trim().is_empty() {
            return Err(StoreError::ValidationError(
                "query must be non-empty".into(),
            ));
        }
        if let Some(top_k) = options.top_k {
            if !(1..=50).contains(&top_k) {
                return Err(StoreError::ValidationError("topK must be in 1..=50".into()));
            }
        }
        if let Some(hops) = options.max_hops {
            if !(1..=5).contains(&hops) {
                return Err(StoreError::ValidationError(
                    "maxHops must be in 1..=5".into(),
                ));
            }
        }
        if let Some(filters) = &options.filters {
            if let Some(edge_type) = filters.edge_type {
                if !matches!(
                    edge_type,
                    EdgeKind::Link | EdgeKind::Embed | EdgeKind::Crosslink
                ) {
                    return Err(StoreError::ValidationError(
                        "filters.edgeType must be link, embed, or crosslink".into(),
                    ));
                }
            }
        }
        if let Some(mq) = &options.multi_query {
            if mq.enabled && mq.n == 0 {
                return Err(StoreError::ValidationError(
                    "multiQuery.n must be a positive integer".into(),
                ));
            }
        }
        if let Some(true) = options.binary_first_pass {
            if options.binary_candidate_pool == Some(0) {
                return Err(StoreError::ValidationError(
                    "binaryCandidatePool must be a positive integer".into(),
                ));
            }
        }
        Ok(())
    }

    /// §4.5.3 — embed `text` via `provider`, caching per text across legs.
    async fn embed_text(
        &self,
        text: &str,
        provider: &dyn EmbeddingProvider,
    ) -> Result<Vec<f32>, StoreError> {
        if let Some(v) = self.cache_get(text) {
            return Ok(v);
        }
        let v = provider.embed(text).await?;
        self.cache_put(text, v.clone());
        Ok(v)
    }

    /// §4.5.1 `flat` mode — top-k by the lexical (BM25) leg.
    #[allow(clippy::type_complexity)]
    fn flat_query(
        &self,
        query: &str,
        top_k: usize,
        options: &RagQueryOptions,
    ) -> Result<
        (
            Vec<RagResultItem>,
            RagTrace,
            Vec<(DocumentId, NodeId)>,
            Option<Vec<BlockedBy>>,
        ),
        StoreError,
    > {
        let results = self.lexical_rank(
            options.wiki_id.as_ref(),
            query,
            top_k,
            options.filters.as_ref(),
        );
        let trace = RagTrace::Flat(TraceDescriptor {
            mode: QueryMode::Flat,
            engine: "gnosis".to_string(),
            top_k: top_k as u64,
            source: Source::Local,
        });
        Ok((results, trace, Vec::new(), None))
    }

    /// §4.5.1 `vector` mode — the REAL dense leg driven from the `ragQuery` query
    /// surface (HIGH-1 / HIGH-3). Embeds the query via the injected
    /// embedding-provider seam (`set_embedding_provider`), cosine-similarity over
    /// the wiki-scoped `full`-field vector snapshot, and returns the vector-leg
    /// hits. Fail-states are genuinely reachable from `ragQuery`:
    /// `VectorIndexUnavailable` (index not built, FS-14), `EmbeddingUnavailable`
    /// (provider unreachable, FS-13). With `hyde: true` the hypothetical snippet
    /// is embedded and routed through the vector leg instead of the raw query
    /// text; `generate_hypothetical` is total (LOW-E), so `HyDEGenerationFailed`
    /// is a reserved variant — a provider embed failure here is an
    /// `EmbeddingUnavailable`, not a relabelled generation failure.
    #[allow(clippy::type_complexity)]
    async fn vector_query(
        &self,
        query: &str,
        top_k: usize,
        options: &RagQueryOptions,
    ) -> Result<
        (
            Vec<RagResultItem>,
            RagTrace,
            Vec<(DocumentId, NodeId)>,
            Option<Vec<BlockedBy>>,
        ),
        StoreError,
    > {
        let index = match self.snapshot().vectors.as_ref() {
            Some(v) => v.clone(),
            None => return Err(StoreError::VectorIndexUnavailable),
        };
        let provider = self
            .embedding_provider()
            .ok_or(StoreError::EmbeddingUnavailable)?;
        let fp = FirstPassOptions {
            binary_first_pass: options.binary_first_pass.unwrap_or(false),
            // LOW-G (§4.5.3a.3): default `binaryCandidatePool` is 10× topK when
            // omitted (not `0`, which would be the uncapped sentinel).
            binary_candidate_pool: options.binary_candidate_pool.unwrap_or(10 * top_k as u64),
        };
        // HyDE routing: embed the hypothetical relevant snippet and run the
        // vector leg against it (a generation failure is a hard HyDE fail-state).
        let hyde = options.hyde.unwrap_or(false);
        let qvec = if hyde {
            let hypot = self.generate_hypothetical(query);
            // LOW-E (§4.5.3): `HyDEGenerationFailed` names the *generation*
            // step, not the embedding step. `generate_hypothetical` is total
            // (a deterministic stand-in) so it never constructs the variant;
            // an embedding-provider failure while embedding the hypothetical is
            // an `EmbeddingUnavailable`, NOT a `HyDEGenerationFailed` — surface
            // the provider error honestly instead of relabelling it.
            self.embed_text(&hypot, provider.as_ref()).await?
        } else {
            self.embed_text(query, provider.as_ref()).await?
        };
        let results = self.vector_leg(&index, options.wiki_id.as_ref(), &qvec, top_k, &fp);
        let trace = RagTrace::Vector(TraceDescriptor {
            mode: QueryMode::Vector,
            engine: "gnosis".to_string(),
            top_k: top_k as u64,
            source: Source::Local,
        });
        Ok((results, trace, Vec::new(), None))
    }

    /// §4.5.1 `hybrid` mode — THREE distinct, non-degenerate legs fused by RRF
    /// (HIGH-2): the graph (`reference`→`fact` walk), the real vector leg
    /// (provider + wiki-scoped `full` index), and the lexical BM25 leg. A
    /// missing vector index or unwired/unreachable provider degrades the vector
    /// leg to empty (not a query failure, per the red `multi_query` expectation)
    /// — never a collapse onto the lexical list.
    #[allow(clippy::type_complexity)]
    async fn hybrid_query(
        &self,
        query: &str,
        top_k: usize,
        options: &RagQueryOptions,
    ) -> Result<
        (
            Vec<RagResultItem>,
            RagTrace,
            Vec<(DocumentId, NodeId)>,
            Option<Vec<BlockedBy>>,
        ),
        StoreError,
    > {
        let max_hops = options.max_hops.unwrap_or(3);
        // GRAPH leg — the resolved `reference`→`fact` roots (+ facts they cite),
        // wiki-scoped and filter-respecting. Blocked/errored roots just don't
        // contribute (a hybrid merges legs, it does not fail on one.
        let graph_vals = self.hybrid_graph_leg(
            options.wiki_id.as_ref(),
            max_hops,
            options.filters.as_ref(),
            top_k,
        );

        // LEXICAL leg.
        let lex_vals = self.lexical_rank(
            options.wiki_id.as_ref(),
            query,
            top_k,
            options.filters.as_ref(),
        );

        // VECTOR leg — the real dense leg, degrading to empty when the index is
        // not built or no provider is wired (HIGH-2). A provider embed failure in
        // hybrid also degrades the leg to empty (hybrid is resilient).
        let mut vec_vals: Vec<RagResultItem> = Vec::new();
        if let Some(index) = self.snapshot().vectors.as_ref().cloned() {
            if let Some(provider) = self.embedding_provider() {
                let fp = FirstPassOptions {
                    binary_first_pass: options.binary_first_pass.unwrap_or(false),
                    // LOW-G (§4.5.3a.3): default `binaryCandidatePool` is 10×
                    // topK when omitted (not `0`, the uncapped sentinel).
                    binary_candidate_pool: options
                        .binary_candidate_pool
                        .unwrap_or(10 * top_k as u64),
                };
                let hyde = options.hyde.unwrap_or(false);
                let embedded = if hyde {
                    let hypot = self.generate_hypothetical(query);
                    self.embed_text(&hypot, provider.as_ref()).await
                } else {
                    self.embed_text(query, provider.as_ref()).await
                };
                if let Ok(qvec) = embedded {
                    vec_vals = self.vector_leg(&index, options.wiki_id.as_ref(), &qvec, top_k, &fp);
                }
            }
        }

        let graph_ids: Vec<(DocumentId, NodeId)> = graph_vals
            .0
            .iter()
            .map(|r| (r.document_id.clone(), r.node_id.clone()))
            .collect();
        let vector_ids: Vec<(DocumentId, NodeId)> = vec_vals
            .iter()
            .map(|r| (r.document_id.clone(), r.node_id.clone()))
            .collect();
        let lexical_ids: Vec<(DocumentId, NodeId)> = lex_vals
            .iter()
            .map(|r| (r.document_id.clone(), r.node_id.clone()))
            .collect();
        let merged = rrf_fuse(&[graph_ids, vector_ids, lexical_ids], top_k);
        let mut item_map: std::collections::HashMap<(DocumentId, NodeId), RagResultItem> =
            std::collections::HashMap::new();
        for it in graph_vals
            .0
            .iter()
            .chain(vec_vals.iter())
            .chain(lex_vals.iter())
        {
            item_map
                .entry((it.document_id.clone(), it.node_id.clone()))
                .or_insert_with(|| it.clone());
        }
        let results = merged
            .into_iter()
            .filter_map(|id| item_map.get(&id).cloned())
            .collect();
        let trace = RagTrace::Hybrid(HybridTrace {
            mode: QueryMode::Hybrid,
            engine: "gnosis".to_string(),
            legs: vec![
                "graph".to_string(),
                "vector".to_string(),
                "lexical".to_string(),
            ],
            top_k: top_k as u64,
            source: Source::Local,
        });
        Ok((results, trace, graph_vals.1, None))
    }

    /// §4.5.2 `graph` mode — a deterministic, hop-limited `reference`→`fact`
    /// walk. Roots are reference nodes in the (optional) wiki matching the
    /// filters. `BROKEN`/`STALE` references surface in `blockedBy` (never
    /// silently traversed); `HopLimitExceeded`/`CycleDetected` propagate as
    /// errors.
    #[allow(clippy::type_complexity)]
    fn graph_query(
        &self,
        wiki: Option<&WikiId>,
        max_hops: u64,
        filters: Option<&QueryAuditFilters>,
        top_k: usize,
    ) -> Result<
        (
            Vec<RagResultItem>,
            Vec<GraphTraceStep>,
            Vec<(DocumentId, NodeId)>,
            Option<Vec<BlockedBy>>,
        ),
        StoreError,
    > {
        let mut roots: Vec<(DocumentId, NodeId)> = Vec::new();
        for shard in self.shards.iter() {
            let guard = shard.read().unwrap();
            for doc in guard.docs.values() {
                if let Some(w) = wiki {
                    if &doc.wiki_id != w {
                        continue;
                    }
                }
                for node in &doc.graph.nodes {
                    if node.kind != NodeKind::Reference {
                        continue;
                    }
                    if !self.node_matches_filters(node, doc, filters) {
                        continue;
                    }
                    roots.push((doc.document_id.clone(), node.node_id.clone()));
                }
            }
        }
        // Determinism (MEDIUM-A / §4.5.2): the reference roots are collected
        // from an unordered HashMap (`docs.values()`), so their order varies per
        // store instance. Sort the graph-leg walk roots by `(DocumentId, NodeId)`
        // ascending so the reference-root ordering — and hence the graph-leg
        // result list — is deterministic BEFORE any top_k truncate or RRF merge.
        roots.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
        let mut steps: Vec<GraphTraceStep> = Vec::new();
        let mut citations: Vec<(DocumentId, NodeId)> = Vec::new();
        let mut blocked: Vec<BlockedBy> = Vec::new();
        let mut resolved: Vec<((DocumentId, NodeId), f64)> = Vec::new();
        for root in &roots {
            let mut path: Vec<(DocumentId, NodeId)> = Vec::new();
            let mut step_buf: Vec<GraphTraceStep> = Vec::new();
            let mut cite_buf: Vec<(DocumentId, NodeId)> = Vec::new();
            let mut block_buf: Vec<BlockedBy> = Vec::new();
            let ok = self.graph_walk(
                root,
                max_hops,
                0,
                &mut path,
                &mut step_buf,
                &mut cite_buf,
                &mut block_buf,
            )?;
            if ok.is_some() {
                resolved.push((root.clone(), 1.0));
            }
            steps.extend(step_buf);
            citations.extend(cite_buf);
            blocked.extend(block_buf);
        }
        let mut seen_c: std::collections::HashSet<(DocumentId, NodeId)> =
            std::collections::HashSet::new();
        citations.retain(|c| seen_c.insert(c.clone()));
        let mut seen_b = std::collections::HashSet::new();
        blocked.retain(|b| seen_b.insert((b.document_id.clone(), b.node_id.clone())));
        // Determinism (MEDIUM-A / §4.5.2): resolved roots are ordered by
        // `(DocumentId, NodeId)` ascending before the top_k truncate, so the
        // tie-break at the top_k boundary is stable across fresh stores.
        resolved.sort_by(|a, b| a.0 .0.cmp(&b.0 .0).then_with(|| a.0 .1.cmp(&b.0 .1)));
        resolved.truncate(top_k);
        // MEDIUM-10 (§4.5.2): `blockedBy` is only valid on an EMPTY result set.
        // A walk that resolved at least one root returns results WITHOUT
        // `blockedBy`; only a fully-blocked (empty) walk carries the block list.
        let resolved_empty = resolved.is_empty();
        let results = resolved
            .into_iter()
            .map(|((d, n), score)| RagResultItem {
                document_id: d.clone(),
                node_id: n.clone(),
                score,
                snippet: self.node_snippet(&d, &n),
                source: Source::Local,
                parent: None,
                stale: None,
            })
            .collect();
        let blocked_by = if resolved_empty && !blocked.is_empty() {
            Some(blocked)
        } else {
            None
        };
        Ok((results, steps, citations, blocked_by))
    }

    /// One step of the §4.5.2 walk: follow a reference node to its target,
    /// recording the edge/state, reusing already-resolved facts, `Broken`/
    /// `Stale` blocking, and `HopLimitExceeded`/`CycleDetected` errors.
    #[allow(clippy::too_many_arguments)]
    fn graph_walk(
        &self,
        node: &(DocumentId, NodeId),
        max_hops: u64,
        hops: u64,
        path: &mut Vec<(DocumentId, NodeId)>,
        steps: &mut Vec<GraphTraceStep>,
        citations: &mut Vec<(DocumentId, NodeId)>,
        blocked: &mut Vec<BlockedBy>,
    ) -> Result<Option<String>, StoreError> {
        if hops > max_hops {
            return Err(StoreError::HopLimitExceeded);
        }
        if path.contains(node) {
            return Err(StoreError::CycleDetected);
        }
        let n = self.read_node(&node.0, &node.1)?;
        match n.kind {
            NodeKind::Reference => {
                let edge = self
                    .edges_from_reference(&node.0, &node.1)
                    .into_iter()
                    .next()
                    .or_else(|| {
                        n.target.clone().map(|target| Edge {
                            source: node.clone(),
                            target,
                            kind: EdgeKind::Link,
                            state: None,
                            cross_wiki: false,
                            relation_type: None,
                        })
                    });
                let Some(edge) = edge else {
                    blocked.push(BlockedBy {
                        document_id: node.0.clone(),
                        node_id: node.1.clone(),
                        state: ReferenceState::Broken,
                    });
                    return Ok(None);
                };
                let state = edge.state.unwrap_or(ReferenceState::Resolved);
                if matches!(state, ReferenceState::Broken | ReferenceState::Stale) {
                    blocked.push(BlockedBy {
                        document_id: node.0.clone(),
                        node_id: node.1.clone(),
                        state,
                    });
                    return Ok(None);
                }
                let target = edge.target.clone();
                path.push(node.clone());
                steps.push(GraphTraceStep {
                    from: node.clone(),
                    to: target.clone(),
                    edge: edge.kind,
                    state,
                });
                let val =
                    self.graph_walk(&target, max_hops, hops + 1, path, steps, citations, blocked)?;
                path.pop();
                Ok(val)
            }
            NodeKind::Fact | NodeKind::Content => {
                if n.kind == NodeKind::Fact && !citations.contains(node) {
                    citations.push(node.clone());
                }
                Ok(n.value.clone())
            }
            NodeKind::Community => Ok(None),
        }
    }

    /// The reference edges (`link`/`embed`/`crosslink`) sourced from a node.
    fn edges_from_reference(&self, doc: &DocumentId, node: &NodeId) -> Vec<Edge> {
        let guard = self.shard_for(doc).read().unwrap();
        guard
            .docs
            .get(doc)
            .map(|d| {
                d.graph
                    .edges
                    .iter()
                    .filter(|e| {
                        e.source == (doc.clone(), node.clone())
                            && matches!(
                                e.kind,
                                EdgeKind::Link | EdgeKind::Embed | EdgeKind::Crosslink
                            )
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Does `node` satisfy an optional `QueryAuditFilters` predicate?
    /// `node_kind` restrics by kind; `edge_type`/`state`/`target` restrict via
    /// the node's reference edges.
    fn node_matches_filters(
        &self,
        node: &Node,
        doc: &Document,
        filters: Option<&QueryAuditFilters>,
    ) -> bool {
        let Some(f) = filters else { return true };
        if let Some(k) = f.node_kind {
            if node.kind != k {
                return false;
            }
        }
        if f.edge_type.is_none() && f.state.is_none() && f.target.is_none() {
            return true;
        }
        doc.graph.edges.iter().any(|e| {
            if e.source != (doc.document_id.clone(), node.node_id.clone())
                || !matches!(
                    e.kind,
                    EdgeKind::Link | EdgeKind::Embed | EdgeKind::Crosslink
                )
            {
                return false;
            }
            if let Some(et) = f.edge_type {
                if e.kind != et {
                    return false;
                }
            }
            if let Some(st) = f.state {
                if e.state != Some(st) {
                    return false;
                }
            }
            if let Some(t) = &f.target {
                if &e.target != t {
                    return false;
                }
            }
            true
        })
    }

    /// §4.5.3 — BM25-ranked nodes over `factKey`/`title`/`tags`/node text within
    /// a wiki (or across the whole store when `wiki` is `None`), respecting the
    /// node filters. Returns `top_k` items sorted by score desc, id asc.
    fn lexical_rank(
        &self,
        wiki: Option<&WikiId>,
        query: &str,
        top_k: usize,
        filters: Option<&QueryAuditFilters>,
    ) -> Vec<RagResultItem> {
        let query_terms = tokenize(query);
        if query_terms.is_empty() {
            return Vec::new();
        }
        // Candidate nodes (matching filters) with their indexable text.
        let mut candidates: Vec<(DocumentId, NodeId, String)> = Vec::new();
        for shard in self.shards.iter() {
            let guard = shard.read().unwrap();
            for doc in guard.docs.values() {
                if let Some(w) = wiki {
                    if &doc.wiki_id != w {
                        continue;
                    }
                }
                let doc_text = format!("{} {}", doc.title, doc.tags.join(" "));
                for node in &doc.graph.nodes {
                    if !self.node_matches_filters(node, doc, filters) {
                        continue;
                    }
                    let mut text = String::new();
                    if let Some(v) = &node.value {
                        text.push_str(v);
                        text.push(' ');
                    }
                    if let Some(fk) = &node.fact_key {
                        text.push_str(fk);
                        text.push(' ');
                    }
                    text.push_str(&doc_text);
                    candidates.push((doc.document_id.clone(), node.node_id.clone(), text));
                }
            }
        }
        let n = candidates.len() as f64;
        if n == 0.0 {
            return Vec::new();
        }
        let mut df: std::collections::HashMap<&str, f64> = std::collections::HashMap::new();
        let mut dl: Vec<f64> = Vec::with_capacity(candidates.len());
        for (_, _, text) in &candidates {
            let terms = tokenize(text);
            dl.push(terms.len() as f64);
            for t in &query_terms {
                if terms.contains(t) {
                    *df.entry(t.as_str()).or_insert(0.0) += 1.0;
                }
            }
        }
        let avgdl = if dl.is_empty() {
            1.0
        } else {
            dl.iter().sum::<f64>() / n
        };
        const K1: f64 = 1.2;
        const B: f64 = 0.75;
        let mut scored: Vec<((DocumentId, NodeId), f64)> = Vec::new();
        for idx in 0..candidates.len() {
            let (doc_id, node_id, text) = &candidates[idx];
            let terms = tokenize(text);
            let mut score = 0.0;
            for qt in &query_terms {
                let f_t = terms.iter().filter(|t| *t == qt).count() as f64;
                if f_t == 0.0 {
                    continue;
                }
                let df_t = df.get(qt.as_str()).copied().unwrap_or(0.0);
                if df_t == 0.0 {
                    continue;
                }
                let idf = ((n - df_t + 0.5) / (df_t + 0.5)).ln_1p();
                let denom = f_t + K1 * (1.0 - B + B * dl[idx] / avgdl);
                score += idf * ((f_t * (K1 + 1.0)) / denom);
            }
            if score > 0.0 {
                scored.push(((doc_id.clone(), node_id.clone()), score));
            }
        }
        scored.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0 .0.cmp(&b.0 .0))
                .then_with(|| a.0 .1.cmp(&b.0 .1))
        });
        scored.truncate(top_k);
        scored
            .into_iter()
            .map(|((d, n), score)| RagResultItem {
                document_id: d.clone(),
                node_id: n.clone(),
                score,
                snippet: self.node_snippet(&d, &n),
                source: Source::Local,
                parent: None,
                stale: None,
            })
            .collect()
    }

    /// A node's snippet (its authored `value`).
    fn node_snippet(&self, doc: &DocumentId, node: &NodeId) -> String {
        self.read_node(doc, node)
            .ok()
            .and_then(|n| n.value)
            .unwrap_or_default()
    }

    /// §4.5.2 `expand: 'parent'` — attach the parent Document payload to the top
    /// `maxParentContext` hits (present-only for the capped hits); a `STALE`
    /// embed's parent carries `stale: true`.
    fn apply_expand(
        &self,
        results: Vec<RagResultItem>,
        options: &RagQueryOptions,
    ) -> Vec<RagResultItem> {
        if options.expand != Some(ExpandMode::Parent) {
            return results;
        }
        let cap = options.max_parent_context.unwrap_or(5) as usize;
        let mut out = Vec::with_capacity(results.len());
        for (i, mut r) in results.into_iter().enumerate() {
            if i < cap {
                r.parent = Some(self.parent_for(&r.document_id, &r.node_id));
            }
            out.push(r);
        }
        out
    }

    /// The parent-context payload for a retrieved node.
    fn parent_for(&self, doc_id: &DocumentId, node_id: &NodeId) -> RagParent {
        let doc = self
            .shard_for(doc_id)
            .read()
            .unwrap()
            .docs
            .get(doc_id)
            .map(|d| (**d).clone());
        let title = doc.as_ref().map(|d| d.title.clone()).unwrap_or_default();
        let snippet = self.node_snippet(doc_id, node_id);
        let stale = doc
            .map(|d| {
                d.graph.edges.iter().any(|e| {
                    e.source == (doc_id.clone(), node_id.clone())
                        && matches!(e.kind, EdgeKind::Embed | EdgeKind::Crosslink)
                        && e.state == Some(ReferenceState::Stale)
                })
            })
            .unwrap_or(false);
        RagParent {
            document_id: doc_id.clone(),
            title,
            snippet,
            stale,
        }
    }

    /// §4.5.3a.3 — select a `binaryCandidatePool`-capped candidate subset by
    /// ascending Hamming distance between the query's binary code and each
    /// node's `binary` field (coarse first pass).
    fn binary_candidate_pool(
        &self,
        index: &VectorIndex,
        qvec: &[f32],
        nodes: &[(DocumentId, NodeId)],
        pool: u64,
    ) -> Vec<(DocumentId, NodeId)> {
        let cap = if pool == 0 { usize::MAX } else { pool as usize };
        let qb = binary_code(qvec);
        let mut ranked: Vec<((DocumentId, NodeId), u64)> = Vec::new();
        for (d, n) in nodes {
            if let Some(bv) = index
                .entries
                .get(&(d.clone(), n.clone(), FieldType::Binary))
            {
                ranked.push(((d.clone(), n.clone()), hamming(&qb, bv)));
            }
        }
        ranked.sort_by(|a, b| {
            a.1.cmp(&b.1)
                .then_with(|| a.0 .0.cmp(&b.0 .0))
                .then_with(|| a.0 .1.cmp(&b.0 .1))
        });
        ranked.truncate(cap);
        ranked.into_iter().map(|((d, n), _)| (d, n)).collect()
    }

    /// Does `wiki_id` own at least one document (i.e. anything to lexical-index)?
    fn wiki_has_documents(&self, wiki_id: &WikiId) -> bool {
        self.shards.iter().any(|s| {
            s.read()
                .unwrap()
                .docs
                .values()
                .any(|d| d.wiki_id == *wiki_id)
        })
    }

    /// MEDIUM-5 — resolve the owning wiki of a document id (for wiki-scoping the
    /// vector leg), or `None` when the document does not exist in the store.
    fn document_wiki_id(&self, doc: &DocumentId) -> Option<WikiId> {
        self.shard_for(doc)
            .read()
            .unwrap()
            .docs
            .get(doc)
            .map(|d| d.wiki_id.clone())
    }

    /// §4.5.3 / §4.5.3a — the shared vector leg: cosine-similarity over the
    /// `full` field of the immutable vector snapshot, MEDIUM-5 wiki-scoped to the
    /// queried wiki (a vector hosted under a doc in wiki A is never returned for
    /// a wiki-B query). Coarse-to-fine `binaryFirstPass` degrades to the `full`
    /// search when the `binary` field is not built.
    fn vector_leg(
        &self,
        index: &VectorIndex,
        wiki: Option<&WikiId>,
        qvec: &[f32],
        top_k: usize,
        fp: &FirstPassOptions,
    ) -> Vec<RagResultItem> {
        let mut nodes: Vec<(DocumentId, NodeId)> = Vec::new();
        let mut seen: std::collections::HashSet<(DocumentId, NodeId)> =
            std::collections::HashSet::new();
        for key in index.entries.keys() {
            if !matches!(key.2, FieldType::Full) || !seen.insert((key.0.clone(), key.1.clone())) {
                continue;
            }
            if let Some(w) = wiki {
                if self.document_wiki_id(&key.0).as_ref() != Some(w) {
                    continue;
                }
            }
            nodes.push((key.0.clone(), key.1.clone()));
        }
        let binary_built = index
            .entries
            .keys()
            .any(|k| matches!(k.2, FieldType::Binary));
        let pool = if fp.binary_first_pass && binary_built {
            self.binary_candidate_pool(index, qvec, &nodes, fp.binary_candidate_pool)
        } else {
            nodes
        };
        let mut scored: Vec<((DocumentId, NodeId), f64)> = Vec::new();
        for (d, n) in &pool {
            if let Some(v) = index.entries.get(&(d.clone(), n.clone(), FieldType::Full)) {
                scored.push(((d.clone(), n.clone()), cosine(qvec, v)));
            }
        }
        scored.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0 .0.cmp(&b.0 .0))
                .then_with(|| a.0 .1.cmp(&b.0 .1))
        });
        scored.truncate(top_k);
        scored
            .into_iter()
            .map(|((d, n), score)| {
                let snippet = self.node_snippet(&d, &n);
                RagResultItem {
                    document_id: d,
                    node_id: n,
                    score,
                    snippet,
                    source: Source::Local,
                    parent: None,
                    stale: None,
                }
            })
            .collect()
    }

    /// §4.5.3 — the HyDE hypothetical relevant snippet (a deterministic stand-in
    /// for the generation leg; the provider embeds it and routes it through the
    /// vector leg).
    fn generate_hypothetical(&self, query: &str) -> String {
        format!("{query} relevant passage")
    }

    /// §4.5.1 `hybrid` graph leg — the resolved `reference`→`fact` walk roots
    /// (wiki-scoped, filter-respecting). Roots that are blocked, cyclic, or
    /// over the hop cap contribute nothing (hybrid degrades a failing leg).
    /// Returns `(result_items, cited_fact_ids)`.
    #[allow(clippy::type_complexity)]
    fn hybrid_graph_leg(
        &self,
        wiki: Option<&WikiId>,
        max_hops: u64,
        filters: Option<&QueryAuditFilters>,
        top_k: usize,
    ) -> (Vec<RagResultItem>, Vec<(DocumentId, NodeId)>) {
        let mut roots: Vec<(DocumentId, NodeId)> = Vec::new();
        for shard in self.shards.iter() {
            let guard = shard.read().unwrap();
            for doc in guard.docs.values() {
                if let Some(w) = wiki {
                    if &doc.wiki_id != w {
                        continue;
                    }
                }
                for node in &doc.graph.nodes {
                    if node.kind != NodeKind::Reference
                        || !self.node_matches_filters(node, doc, filters)
                    {
                        continue;
                    }
                    roots.push((doc.document_id.clone(), node.node_id.clone()));
                }
            }
        }
        // Determinism (MEDIUM-A / §4.5.2): the reference roots above are
        // collected from an unordered HashMap — sort by `(DocumentId, NodeId)`
        // ascending so the graph-leg order (and the RRF rank at the top_k
        // boundary) is deterministic before the truncate / RRF merge.
        roots.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
        let mut items: Vec<RagResultItem> = Vec::new();
        let mut cite_set: std::collections::HashSet<(DocumentId, NodeId)> =
            std::collections::HashSet::new();
        for root in &roots {
            let mut path: Vec<(DocumentId, NodeId)> = Vec::new();
            let mut step_buf: Vec<GraphTraceStep> = Vec::new();
            let mut cite_buf: Vec<(DocumentId, NodeId)> = Vec::new();
            let mut block_buf: Vec<BlockedBy> = Vec::new();
            let ok = self.graph_walk(
                root,
                max_hops,
                0,
                &mut path,
                &mut step_buf,
                &mut cite_buf,
                &mut block_buf,
            );
            if matches!(ok, Ok(Some(_))) {
                let d = root.0.clone();
                let n = root.1.clone();
                let snippet = self.node_snippet(&d, &n);
                items.push(RagResultItem {
                    document_id: d,
                    node_id: n,
                    score: 1.0,
                    snippet,
                    source: Source::Local,
                    parent: None,
                    stale: None,
                });
                cite_set.extend(cite_buf);
            }
        }
        // Determinism (MEDIUM-A / §4.5.2): stable `(DocumentId, NodeId)` sort of
        // the resolved graph-leg items before the top_k truncate.
        items.sort_by(|a, b| {
            a.document_id
                .cmp(&b.document_id)
                .then_with(|| a.node_id.cmp(&b.node_id))
        });
        items.truncate(top_k);
        (items, cite_set.into_iter().collect())
    }

    /// §4.5.3 multi-query expansion (HIGH-4) — deterministically fan the query
    /// out into `n` variants (the query + successive document-popularity
    /// expansion terms), starting from the original query. An expansion with
    /// nothing to expand from (no indexable terms) → `MultiQueryExpansionFailed`.
    fn expand_query_variants(
        &self,
        query: &str,
        n: u64,
        options: &RagQueryOptions,
    ) -> Result<Vec<String>, StoreError> {
        let n = n.max(1) as usize;
        let mut variants = vec![query.to_string()];
        if n <= 1 {
            return Ok(variants);
        }
        let mut used: std::collections::HashSet<String> = tokenize(query).into_iter().collect();
        let ranked =
            self.term_popularity(options.wiki_id.as_ref(), options.filters.as_ref(), &used);
        let mut current = query.to_string();
        let mut added = false;
        for (term, _) in ranked {
            if variants.len() >= n {
                break;
            }
            current = format!("{} {}", current, term);
            used.insert(term);
            variants.push(current.clone());
            added = true;
        }
        // Real multi-query failure path: with n > 1 and no distinct term to fan
        // out to, the expansion cannot run → `MultiQueryExpansionFailed` (FS-19).
        if !added {
            return Err(StoreError::MultiQueryExpansionFailed);
        }
        while variants.len() < n {
            variants.push(current.clone());
        }
        Ok(variants)
    }

    /// §4.5.3 — the deterministically-ranked indexable terms of the (optional)
    /// wiki's candidate docs (title/tags/node value/factKey), descending by
    /// frequency (ties alphabetical), excluding `exclude`. Drives multi-query
    /// expansion (pseudo-relevance feedback).
    fn term_popularity(
        &self,
        wiki: Option<&WikiId>,
        filters: Option<&QueryAuditFilters>,
        exclude: &std::collections::HashSet<String>,
    ) -> Vec<(String, usize)> {
        let mut freq: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for shard in self.shards.iter() {
            let guard = shard.read().unwrap();
            for doc in guard.docs.values() {
                if let Some(w) = wiki {
                    if &doc.wiki_id != w {
                        continue;
                    }
                }
                let mut text = format!("{} {}", doc.title, doc.tags.join(" "));
                for node in &doc.graph.nodes {
                    if !self.node_matches_filters(node, doc, filters) {
                        continue;
                    }
                    if let Some(v) = &node.value {
                        text.push(' ');
                        text.push_str(v);
                    }
                    if let Some(fk) = &node.fact_key {
                        text.push(' ');
                        text.push_str(fk);
                    }
                }
                for t in tokenize(&text) {
                    if exclude.contains(&t) {
                        continue;
                    }
                    *freq.entry(t).or_insert(0) += 1;
                }
            }
        }
        let mut ranked: Vec<(String, usize)> = freq.into_iter().collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        ranked
    }

    /// §4.5.3 compression (HIGH-4) — post-retrieval snippet transform. `filter`
    /// is a query-aware binary keep/drop that changes the returned snippet set;
    /// `extract`/`graph` are the higher-phase transforms (pass-through here). The
    /// compressor degrades gracefully to the uncompressed context on failure
    /// (NOT a query failure); `CompressionFailed` is reserved for a total
    /// failure that cannot degrade.
    fn apply_compression(
        &self,
        results: Vec<RagResultItem>,
        query: &str,
        mode: CompressionMode,
    ) -> Vec<RagResultItem> {
        match mode {
            CompressionMode::None => results,
            CompressionMode::Filter => {
                let qterms: std::collections::HashSet<String> =
                    tokenize(query).into_iter().collect();
                results
                    .into_iter()
                    .filter(|r| relevant_snippet(&qterms, &r.snippet))
                    .collect()
            }
            CompressionMode::Extract | CompressionMode::Graph => results,
        }
    }
}

/// §4.5.3 / §4.5.3a — cosine similarity over the shared prefix (handles the
/// index vs query dimension difference from test providers). Zero vector → 0.0.
fn cosine(a: &[f32], b: &[f32]) -> f64 {
    let n = a.len().min(b.len());
    let a = &a[..n];
    let b = &b[..n];
    let mut dot = 0.0_f64;
    let mut na = 0.0_f64;
    let mut nb = 0.0_f64;
    for i in 0..n {
        dot += a[i] as f64 * b[i] as f64;
        na += a[i] as f64 * a[i] as f64;
        nb += b[i] as f64 * b[i] as f64;
    }
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na.sqrt() * nb.sqrt())
    }
}

/// Tokenize a string into lowercase alphanumeric terms.
fn tokenize(s: &str) -> Vec<String> {
    s.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .collect()
}

/// §4.5.3 — the query-aware binary relevance filter used by compression `filter`:
/// keep a snippet whose query-term density is at least 25% (a tight, on-topic
/// snippet); drop a low-relevance, query-token-diluted snippet.
fn relevant_snippet(qterms: &std::collections::HashSet<String>, snippet: &str) -> bool {
    if qterms.is_empty() {
        return true;
    }
    let tokens = tokenize(snippet);
    if tokens.is_empty() {
        return false;
    }
    let hits = tokens.iter().filter(|t| qterms.contains(*t)).count() as f64;
    hits / tokens.len() as f64 >= 0.25
}

/// §4.5.3a.1 — sign-binarize a vector to a compact binary code (used for the
/// coarse first-pass Hamming comparison).
fn binary_code(v: &[f32]) -> Vec<u8> {
    v.iter().map(|x| if *x > 0.0 { 1 } else { 0 }).collect()
}

/// §4.5.3a.3 — Hamming distance between a binary code and (the binarized form
/// of) a node's binary field, over the shared prefix.
fn hamming(a: &[u8], b: &[f32]) -> u64 {
    let n = a.len().min(b.len());
    let mut d = 0u64;
    for i in 0..n {
        let bit = if b[i] > 0.0 { 1 } else { 0 };
        if a[i] != bit {
            d += 1;
        }
    }
    d
}

/// The machine-actionable fail-closed rejection `{code, field, message}`
/// (§4.3.2a.2) for a schema-conformance gate failure. `field` names the
/// offending candidate field; `code` the exact check that failed.
fn rejected_outcome(code: &str, field: &str, message: &str) -> ProposalOutcome {
    ProposalOutcome {
        accepted: false,
        fact: None,
        rejection: Some(Rejection {
            code: code.to_string(),
            field: field.to_string(),
            message: message.to_string(),
        }),
    }
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
