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
pub mod store; // §4.1 document store (persistence, RAG store interface)

// §4.1 document-store public API surface (re-exported for the shell / tests).
pub use self::store::{
    CreateDocumentRequest, DocState, Document, DocumentId, DocumentList, DocumentSummary, Edge,
    EdgeKind, Graph, ListDocumentsFilter, Node, NodeId, NodeKind, RagStore, ReferenceState, Store,
    StoreError, UpdateDocumentRequest, Wiki, WikiId,
};
