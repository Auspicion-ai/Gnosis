# Gnosis — Active Decisions Summary

Maintained by the document-archival loop. ACTIVE = governs the current Gnosis
architecture; SUPERSEDED = replaced (provenance only). The four design
constraints (D1–D4) are the suite's constitution — every proposal is reviewed
through them.

## Imported suite decisions (from `../Auspicion Suite/docs/decisions.md`)

| Decision | What it pins |
| --- | --- |
| **GNOSIS-ENGINE** | Gnosis is the production graph/vector engine; Incanter is ARCHIVED as a prototype lexical system. |
| **INCANTER-DISPOSITION** | The production engine is non-JS (Rust); Incanter was the prototype (now archived). |
| **D1–D4** | open-source first (AGPL-3.0), local-first, interconnection, MCP-GUI parity (at the shell). |
| **MCP-SURFACE-DEFERRED** | The suite-wide MCP surface stays PROVISIONAL until each tool's implementation pass. |
| **D4-CARVEOUT-SCOPE** | The D4 security-configuration carve-out (GUI-only) minimum set + per-tool additions. |
| **GRAPH-PUSH-FORMAT** | `provident-graph/1` is the canonical push contract. |
| **FAMILIAR-MEMORY-STORE-INTEGRATION** | Familiar can OPTIONALLY use Gnosis (via Astrographer) as its memory store. |

## Gnosis-local decisions

| Decision | Date | What it pins |
| --- | --- | --- |
| **ARCHIVE-GITIGNORED** | 2026-09-09 | The `archive/` directory is **GITIGNORED** per the archival-loop convention (obsolete docs, stale test data, findings reports, and historical review records move there and are never committed). Content there is local provenance, not part of the tracked repo. |
| **LOCKFILE-COMMITTED** | 2026-09-09 | Cargo.lock is **committed** for this binary/lib crate so build reproducibility is pinned for consumers of the engine. |
| **SHARDED-RWLOCK-STORE** | 2026-09-09 | The authoritative store is a **sharded `RwLock` map** keyed by `(wikiId, documentId)`. Reads take per-shard read locks (many concurrent); a document's mutation holds only that shard's write lock. Optimistic concurrency (§4.1.4) — compare-base-revision → apply → bump — is atomic under the shard write lock. No global store lock. |
| **IMMUTABLE-DERIVED-SNAPSHOT** | 2026-09-09 | All derived indexes (lexical BM25, multi-field vector index, triple index, profile summary) are **immutable `Arc<Snapshot>`s swapped atomically** (behind `RwLock<Arc<T>>` / arc-swap). Query-time reads are lock-free on the snapshot; a writer rebuilds and swaps on the epoch/dirty feed. Honors spec §4.2.7.2 "derived index is not a second owner". |
| **WRITER-ACTOR-JOURNAL** | 2026-09-09 | All mutations route through a **single-writer journal actor** (append-only mutation journal + separate query audit log). A single owner mutates; everything else reads. Epoch/dirty counter drives index rebuilds in batches (not per-write). Resolves the mutable-—multi-thread tension by partitioning mutation to one owner. **Applied (2026-09-09, §4.1):** a minimal `MutationJournal` (`RwLock<Vec<JournalEntry>>` + `AtomicU64 epoch`) is appended in each mutation critical section with `Store::epoch()`/`journal_len()` accessors; a tokio writer-actor is held off until §4.2–§4.4 need the async index-rebuild/audit feed and the write-path latency it can add is justified. |
| **LAYERED-OVERLAY** | 2026-09-09 | Live updates are **layered over a revisioned base**: overlay-first reads (pending `OverlayLayer` for an agent/workspace/transaction scope), commit flattens into the authoritative store under the shard lock. Journal preserves prior values for rollback/provenance/reconstruct. Manual declarations are authoritative and never overwritten by automatic extraction (§4.2.8.4). |
| **ARC-SHARED-ENGINE** | 2026-09-09 | The engine is an **`Arc<Gnosis>`** shared across tokio tasks (the `ragQuery`/`ragStream`/`getEngineStatus` seam + `RagStore` seam). Reads use the immutable snapshot; mutation serializes at the per-document shard or the journal actor — never a global state lock, so many concurrent requests do not block each other. |

Design details in `docs/research/gnosis-data-structures-concurrency-plan.md`.

The Gnosis spec pins new decision rows when the relevant unit lands, e.g.
RESULT-LEVEL-PROVENANCE, QUERY-AUDIT-LOG, GRAPH-MODE-WALK,
REFERENCE-GRAPH-ADDITIVE-FIELDS, FANOUT-INTERLEAVE-MERGE.

## SUPERSEDED

_(none yet.)_
