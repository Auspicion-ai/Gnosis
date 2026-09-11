//! Gnosis — the production graph/vector engine of the Auspicion Suite.
//!
//! A pure backend to the Astrographer Electron shell: the document store, the
//! knowledge graph, fact/citation tracking, consistency enforcement, and the
//! RAG/agent-memory retrieval stack.
//!
//! The canonical behavior contract is `docs/specs/gnosis.md`; every module
//! below corresponds to a contract section and a TestWriter derives its red set
//! from that spec.

pub mod consistency; // §4.4 consistency enforcement
pub mod facts; // §4.3 fact/citation tracking
pub mod graph; // §4.2 knowledge graph (subject-relation model, entity resolution)
pub mod retrieval; // §4.5 RAG/agent-memory retrieval (query modes + vector fields)
pub mod server; // §7.2 P2 — pure server-side mapping fns (status / decode-outcome / routing)
pub mod store; // §4.1 document store (persistence, RAG store interface)
pub mod wire; // §7.2 F2 engine wire contract (codecs + validate + SSE + health + envelope)

// §7.2 F2 wire re-export surface (flat paths the shell / TestWriter reach).
pub use self::wire::codecs;
pub use self::wire::crud;
pub use self::wire::crud::{CrudMethod, CrudResponseError, CrudResult, CrudValidationFailure};
pub use self::wire::decode;
pub use self::wire::decode::{DecodeError, ValidationFailure};
pub use self::wire::envelope;
pub use self::wire::envelope::Envelope;
pub use self::wire::error; // crate-root `gnosis::error` (no collision exists)
pub use self::wire::sse;
pub use self::wire::status;
pub use self::wire::status::HealthReport;

// §7.2 P2 — the pure server-side mapping fns (re-exported so the TestWriter
// reaches them as `gnosis::server_status` etc.).
pub use self::server::{request_decode_status, route_bijection, server_status};

// §4.5.3 retrieval-stack pure helpers (RRF fusion, §4.5.3 — k=60, EXACT rule).
pub use self::retrieval::{rrf_fuse, RRF_K};

// §7.8 F6 — the four pure deterministic RAG-evaluation metric fns (re-exported
// so the TestWriter reaches them as `gnosis::contextual_precision` etc.).
pub use self::retrieval::{contextual_precision, contextual_recall, mrr_at_k, ndcg_at_k};

// §4.1 document-store public API surface (re-exported for the shell / tests).
pub use self::store::{
    Alias, BlockedBy, CandidateFact, Community, CommunityContext, CommunityId, CommunityState,
    CompressionMode, ConsistencyReferenceReport, CreateDocumentRequest, DeclareCommunityOptions,
    DerivedIndexes, DocState, Document, DocumentId, DocumentList, DocumentSummary, Edge, EdgeKind,
    EmbeddingCache, EmbeddingProvider, EngineState, EngineStatus, EngineSubsystems, EntityPair,
    ExpandMode, Fact, FactList, FieldType, FirstPassOptions, GetTriplesFilter, Graph,
    GraphTraceStep, HybridTrace, LexicalIndex, ListDocumentsFilter, ListFactsFilter,
    MergeFactsOptions, MultiQueryOptions, Node, NodeId, NodeKind, ProfileSummary, ProposalOutcome,
    QueryAuditEntry, QueryAuditFilters, QueryMode, QueryTriplesOptions, RagChunk, RagParent,
    RagQueryOptions, RagResult, RagResultItem, RagStore, RagStream, RagTrace, ReferenceState,
    Rejection, ResolutionResult, ResolveEntitiesOptions, ResolveOptions, ResolvedFact, Source,
    Store, StoreError, SubTaskDagOptions, TraceDescriptor, Triple, TripleDirection, TriplePattern,
    UpdateDocumentRequest, UpdateFactRequest, VectorIndex, Wiki, WikiId,
};
