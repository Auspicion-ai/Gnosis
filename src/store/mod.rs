//! Document store (§4.1) — the persistence layer.
//!
//! Nodes/edges, revisioning, optimistic concurrency, and the `RagStore`
//! interface that the Astrographer shell proxies over the IPC seam.

/// A document (a Provident graph) in the store.
pub struct Document {
    pub document_id: String,
    pub wiki_id: String,
    pub revision: u64,
    pub title: String,
    // nodes + edges
}

/// The seam the Astrographer shell proxies over IPC/HTTP.
pub trait RagStore {
    // §4.1.3 store operations
    // ... (implemented per the spec)
}
