# Gnosis — Data Structure & Concurrency Plan

**Date:** 2026-09-09 · **Status:** REALIZED (design plan; the design it pins is
recorded as the ACTIVE `docs/decisions.md` decisions and implemented across
§4.1–§4.6).
**Type:** design/architecture analysis (not a behavior spec — the spec stays
`docs/specs/gnosis.md`).

This is the data-structure plan for the Gnosis engine, written before any
implementation unit. It answers three hard constraints raised for the core:

1. **It is a live memory store** — must support *updating* and *overlaying* data,
   not just static load.
2. **It must be fast on significant volumes** under **multiple concurrent
   requests** (the `ragQuery`/`ragStream`/`getEngineStatus` seam plus the
   `RagStore` persistence seam).
3. **Safe Rust** does not let data be both mutable and freely shared across
   threads — interior mutability and ownership splitting are required.

Every design choice below maps to a spec section so a TestWriter can still derive
states from the canonical contract **and** from the concurrency/update behavior.

---

## 0. The core tension, resolved by ownership splitting

Safe Rust's borrow rule: many readers (`&T`) or one writer (`&mut T`), enforced.
The resolution is to **split the state by how it is touched**:

| Slice of state | Touched how | Safe-Rust vehicle |
| --- | --- | --- |
| **Authoritative store** (Documents, graph nodes/edges, facts, wikis) | mutated, but mutations are *less frequent* than reads; writes are per-document | **sharded `RwLock`** map keyed by `(wikiId, documentId)` |
| **Derived indexes** (lexical BM25, vector index/multi-field, triple index, profile summary) | read-mostly, hot query path; **not** mutated in place — *rebuilt* | **immutable snapshot** `Arc<Snapshot>` swapped atomically (`RwLock<Arc<T>>`) |
| **Mutation journal + query audit log** (appended only) | single-writer append | **writer-actor** (tokio task) or `Mutex<append>`; never a global lock on reads |
| **Overlay layers** | pending writes scoped to an agent/workspace/transaction | overlay-first lookup over a revisioned base |

The single source of truth (spec §4.2.7.2 "Derived index not a second
owner") is directly honored: **the graph owns** triples/vector fields/communities;
all indexes are **rebuildable projections** of it. That maps naturally to
"immutable snapshot, rebuilt on change."

---

## 1. Authoritative store — sharded read/write

```rust
// One shard owns a slice of documents. Reads take read(); the document's
// mutation (with optimistic-concurrency guard) takes write().
struct StoreShard {
    by_id: HashMap<DocumentId, Arc<Document>>,   // authoritative, stable identity
    // ... wiki membership, per-wiki indexes
}

pub struct Store {
    shards: Box<[RwLock<StoreShard>]>,           // fixed shard count (e.g. 64)
    // shard(wi, id) = stable: id % shards.len()
}
```

- **Reads (high concurrency):** a query touches many documents → takes `read()`
  on the relevant shards *concurrently* (multiple readers, no writer blocked on a
  read it isn't touching).
- **Writes (exclusive per document only):** `updateDocument` holds that
  document's shard `write()` for the **compare-base-revision → apply → bump**
  critical section (§4.1.4 optimistic concurrency is atomic under the shard
  lock). Two different documents in different shards never contend.
- **`Arc<Document>`** as the stored unit means readers hold a cheap clone of the
  `Arc` and keep reading the immutable `Document` even if the shard is later
  written — copy-on-write at the document level; the swap replaces the `Arc` in
  the map after revision bump.

**Authoritative value types (per spec §4.1–§4.3):**

```rust
struct Document {
    document_id: DocumentId,      // UUID v4, immutable
    wiki_id: WikiId,
    revision: u64,                // monotonic; = 0 initial; bumped on commit
    state: DocState,              // DRAFT | PUBLISHED | ARCHIVED
    nodes: Vec<Node>, edges: Vec<Edge>,
    title: String, created_at/updated_at, tags, author,
}

enum NodeKind { Content, Fact, Reference }
struct Node {
    document_id: DocumentId, node_id: NodeId,
    kind: NodeKind,
    properties: NodeProperties,   // kind, text/value, factKey, target+mode, ...
    vector_fields: BTreeMap<FieldType, VecBytes>,  // full/binary/other (derived, §4.5.3a)
}

enum EdgeKind { Link, Embed, Crosslink, DocHead, NextSection, DocEnd, DocChild, Relation }
struct Edge {
    source: (DocumentId, NodeId), target: (DocumentId, NodeId),
    kind: EdgeKind, properties: EdgeProperties,  // state: FRESH|RESOLVED|STALE|BROKEN, crossWiki, relationType
}

struct Fact {  // a `fact` Node + §4.3
    fact_key: String, value: Value,
    document_id: DocumentId, node_id: NodeId,
    updated_at: timestamp, citations: Vec<(DocumentId, NodeId)>, // >=1 (minimum-citation invariant)
}
```

Triples are **`Relation` edges owned by the graph** (spec §4.2.7.2) — a derived
triple index accelerates `queryTriples` but is not an owner.

---

## 2. Live update / overlay — a revisioned base + mutation journal

"Live memory store … updating or overlaying data" is implemented as **layered
revisioning**: the committed base plus ordered overlay layers, with a journal as
the non-destructive history (the rollback / provenance / rebuild feed).

```rust
// One write path. The actor serializes mutations; every handler is read-only.
struct Journal {
    // append-only, single writer (the WriterActor below)
    entries: VecDeque<JournalEntry>,
    epoch: u64,                    // bumped on every commit -> drives index rebuilds
}

struct JournalEntry {
    seq: u64, source: Source,      // gui_user | mcp_caller | auto_extract | manual
    op: Op,                        // CreateDocument/Update/DeleteFact/AddTriple/Overlay/Commit
    base_revision: Option<u64>,    // for optimistic-concurrency guard (§4.1.4)
}

// Layered reads: overlay-first, then base.
struct OverlayLayer {
    overlay_id: OverlayId, scope: OverlayScope, // Agent | Workspace | Transaction
    deltas: HashMap<NodeKey, ValueOverlay>,     // pending fact/node value overlays
    base_epoch: u64,                            // guard: which base this overlays
}
```

Overlay semantics (the "overlay new info over old"):
- **Upsert (live update):** `overlayFact(key, value, source, baseRevision)`. If
  `baseRevision` matches the committed revision, it **replaces** the value
  (revision bump) and the prior value stays in the journal (history/provenance).
  If the base is stale → `ConflictError` (§4.1.4).
- **Layer (overlay):** a delimited batch is applied as a layer against a base
  epoch. The overlay is either committed (flattened into the authoritative store
  under the shard lock, bumping revision) or kept pending and visible only to the
  request scope. **Manual declarations are authoritative and never overwritten by
  automatic extraction** (§4.2.8.4); a conflicting automatic candidate is routed
  to entity resolution / rejected fail-closed (§4.3.2a).
- **Rollback / reconstruct:** because prior values live in the journal, any prior
  epoch is reconstructable — the "overlay" is non-destructive.

Journal is also the **event feed** for derived-index rebuilds: an `epoch`/dirty
counter avoids rebuilding on every write (batch the rebuild).

---

## 3. Derived indexes — immutable snapshots, atomic swap

The hot query path (vector, lexical, triple, profile) never takes a big lock:

```rust
// Immutable, rebuildable projections of the authoritative store.
struct DerivedSnapshot {
    lexical: LexicalIndex,              // BM25 over factKey/title/tags/node text (§4.5.3)
    vectors: VectorIndex,               // multi-field, keyed (docId,nodeId,fieldType) (§4.5.3a.2)
    triples: TripleIndex,               // derived subject/relation/object lookup (§4.2.7.2)
    profile: ProfileSummary,            // derived document (§4.5.4)
    epoch: u64,
}
// In the engine: RwLock<Arc<DerivedSnapshot>> (or arc-swap for lock-free reads).
```

- **Read path** `ragQuery`/`ragStream`: clone the `Arc<DerivedSnapshot>` (cheap)
  and read **lock-free** on the immutable snapshot. Concurrent requests never
  block on each other for retrieval.
- **Write path**: a mutation bumps `epoch`. A single rebuild task (or lazy rebuild
  on next dirty query) computes a **new snapshot** and **atomically swaps** the
  `Arc`. Readers already in flight keep the old snapshot — no partial index
  states, no data races.
- **Fast at volume:** the `binary` vector field enables coarse-to-fine ANN
  (first pass Hamming over binary codes → candidate pool → full cosine on `full`
  field only, spec §4.5.3a.3) — a pure read over the snapshot, lock-free.

---

## 4. Engine shape (multi-request safety)

```rust
pub struct Gnosis {
    store: Arc<Store>,                     // sharded RwLock store
    snapshot: RwLock<Arc<DerivedSnapshot>>,// immutable derived state
    journal: Arc<Journal>,                 // single-writer
    writer: tokio::task::JoinHandle<()>,   // the WriterActor consuming journal ops
}

// Handlers (tokio) clone Arc<Gnosis> per request; retrieval = snapshot read,
// mutation = send op to the WriterActor (never block on the store in the hot path).
```

- The engine is an **`Arc<Gnosis>`** shared across tokio tasks.
- **No global state lock**: reads use the immutable snapshot; writes serialize
  only at the per-document shard **or** the journal actor, never a global store
  lock. This directly resolves the "mutable + multi-thread" tension: mutation is
  partitioned (shards, actor) while sharing is immutable `Arc`s.

---

## 5. Safe-Rust pattern mapping (the answer to constraint 3)

| Pattern | Where | Why it's sound |
| --- | --- | --- |
| `Arc<Document>` / `Arc<Gnosis>` | shared ownership | immutable shared reads; clone = cheap shared ref |
| `RwLock` (sharded) | authoritative store | many readers OR one writer **per shard**; never both on one shard |
| `RwLock<Arc<DerivedSnapshot>>` (or arc-swap) | derived indexes | readers get an immutable snapshot; writer swaps the Arc — no &mut to shared data |
| **WriterActor** (message channel) | mutation journal | a single owner mutates; everything else reads — no concurrent mutation |
| shard keying `id % N` | store partitioning | deterministic shard → lock-striping, no global lock |
| `RwLock<BTreeMap>` per shard | adjacency | fine-grained, read-concurrent |

The one mutation point that must be atomic — the optimistic-concurrency
`compare-base → apply → bump-revision` — lives **inside the shard `write()`** (or
the WriterActor), so the "check then act" cannot race across threads.

---

## 6. Cost-benefit / risks

- **Snapshot rebuild cost** on a large store is real; mitigated by epoch batching
  (rebuild on commit batches, not every write) and by the fact that memory writes
  are far rarer than reads. A dirty-epoch lazy rebuild avoids rebuild-on-thrash.
- **Shard count** is a tuning knob (fixed at boot, e.g. 64). Cross-document
  topological resolution takes multiple shard `read()` locks — safe (all reads),
  never deadlocks writers it doesn't contend with.
- **Journal growth** is bounded by a compaction policy; the journal is also the
  audit/history surface, so compaction must retain provenance (facts keep
  citations/history, spec §4.3).
- No external graph DB / vector DB dependency: this is all in-process Rust,
  honoring "if it can be local it should be local" (D2).

---

## 7. Decisions this plan pins (for `docs/decisions.md`)

- **SHARDED-RWLOCK-STORE** — authoritative store is a sharded `RwLock` map keyed
  by `(wikiId, documentId)`; per-shard read/write; optimistic concurrency atomic
  under the shard write lock.
- **IMMUTABLE-DERIVED-SNAPSHOT** — all derived indexes (lexical/vector/triple/
  profile) are immutable `Arc<Snapshot>` swapped atomically; lock-free query reads.
- **WRITER-ACTOR-JOURNAL** — all mutations route through a single-writer journal
  actor (append-only) driving epoch-based index rebuilds; reads never block.
- **LAYERED-OVERLAY** — live updates are layered over a revisioned base
  (overlay-first reads, commit flattens into the store); journal preserves history
  for rollback/provenance; manual declarations authoritative (§4.2.8.4).
- **ARC-SHARED-ENGINE** — engine is `Arc<Gnosis>` shared across tokio tasks; no
  global store lock.

These are recorded in `docs/decisions.md` and informed the §4.1–§4.6 TestWriter
red sets (which exercised concurrency/optimistic-concurrency/overlay states);
the core engine is now implemented to them.

**Realization note (per `docs/decisions.md`):** SHARDED-RWLOCK-STORE,
IMMUTABLE-DERIVED-SNAPSHOT (`snapshot()`/`swap_snapshot()` on a
`RwLock<Arc<DerivedIndexes>>`), and ARC-SHARED-ENGINE are applied as described
here. WRITER-ACTOR-JOURNAL is applied in its **minimal** form — a synchronous
`MutationJournal` (`RwLock<Vec<JournalEntry>>` + `AtomicU64 epoch`) appended in
each mutation critical section — with the tokio writer-actor held off (the
decision records this explicitly; the engine-side audit-log recording sink the
actor would feed is deferred in `docs/pending.md`). The full writer-actor and the
LAYERED-OVERLAY overlay layers are forward work: the overlay is decided but not
yet realized in the store, so its overlaid-read states are not yet exercised by
the current test set.
