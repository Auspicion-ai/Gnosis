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
| **GRAPH-OWNS-RELATION-AND-MERGE** | 2026-09-09 | §4.2's **adversarial** findings force a type evolution: the graph types **own** the data the spec pins rather than a sidecar. `Edge` gains `relation_type: Option<String>` (so a `relation` edge genuinely carries its `relationType`, §4.2.7.2 — a triple is read from the graph's relation edges, not a `triple_store` second-source-of-truth); node deletion **prunes** the relation edges (real cascade, §4.2.7.5); `merge_facts` **persists** the merged fact (union citations + `updated_at` + journal, §4.2.9.1); `resolve_entities` writes a durable alias→canonical mapping (jourled, §4.2.9.1); `set_reference_state` mutates only the matched edge's state under `reference_lock` with an optimistic-concurrency base compare (no whole-document clobber); communities are graph-owned node+`member`-edge structures with member existence validated (§4.2.8.3). This supersedes the earlier "frozen §4.1 types / sidecar-projection" approach the §4.2 TestWriter used, which the adversarial gate rejected as a second source of truth. |
| **LOCK-ORDER-REF-SHARD-SIDECAR** | 2026-09-09 | §4.4's adversarial gate found an AB-BA deadlock and dictated a **single uniform lock order** across the engine: **`reference_lock` → shard (read/write) → `fact_store`/communities/journal**. No path holds a shard **write** while acquiring another shard or `fact_store`; no path holds `fact_store.write()` while acquiring a shard (fact-commit paths ground citations under `reference_lock.read()` before taking `fact_store.write()`); `publish_document` validates the whole gate under read locks then takes the single shard write only for the state transition, re-validating the doc's revision (concurrent change → `ConflictError`/retry). This is the concurrency contract every future unit (esp. §4.5 retrieval) must honor. |
| **SUB-TASK-DAG-VALIDATED-ONLY** | 2026-09-09 | The inference-time `subTaskDag` option is **opto-in / SHOULD-HAVE** (spec's own framing, alongside adaptive RAG, which is PARKED). Currently it is parsed + validated only: it decomposes **nothing** — a single-node decomposition (the query itself) is the honest stand-in, **no fake decomposition is fabricated**. `SubTaskDagFailed` is a **RESERVED** variant (throwable once a real DAG decomposition lands). The full sub-task DAG is PARKED (engine-internal deferred), recorded under `docs/pending.md`. No test requires it to throw. |
| **RESERVED-ERRVARIANTS-DISCIPLINE** | 2026-09-09 | A fail-state variant is only **constructed when its step genuinely runs and fails**; a variant whose generating step is currently **total** (cannot fail) or **not implemented** is documented as **RESERVED**, never fabricated into a throw-path. Applies to `HyDEGenerationFailed` (LOW-E — `generate_hypothetical` is total, so a provider embed failure surfaces as `EmbeddingUnavailable`, not a relabelled generation failure), `CompressionFailed` (the local compressor is total — Filter/Extract/Graph only transform and **degrade gracefully to uncompressed** on any downstream problem; the variant is reserved for a future non-degradable compressor leg), and `SubTaskDagFailed` (see SUB-TASK-DAG-VALIDATED-ONLY). |
| **PBT-GATE-MANDATORY** | 2026-09-09 | The property-based-testing (PBT) gate is **mandatory** for every code-bearing unit. Per unit three artifacts are required: (1) a **typed §5.x Property register** authored by the spec_writer (`P-IM`/`P-SM`/`P-TP` rows, ≤ 8 rows, invariants ONLY — never §6 `FS-n` fail-state or §7 `F-` gap rows, and each row TRUE of the current green implementation); (2) an **executed property layer** (deterministic pinned seed, run under `cargo test`, ≤ 100 generated cases/row, ≤ 400 total, stop-after-5, each row recorded HELD/BROKEN + strategy-id); (3) a **read-only PBT audit** (over-strength reasoning per row, generator-coverage audit, prose counterexamples, negative-generator requests — reviewers never run generators). A code-bearing DONE row without register + property layer + audit is a review finding. **Retrofitted 2026-09-09 to §4.1–§4.5:** 5 registers in `docs/specs/4-*-property-register.md`, 5 executed property layers (`tests/props_{store,graph,facts,consistency,retrieval}.rs`), 5 read-only audits. The gate surfaced and fixed a **genuine host defect**: `rrf_fuse`'s f64 RRF accumulation was non-associative, so mathematically-exact-tied keys landed 1 ULP apart by outer-list order, defeating the id-ascending tie-break and breaking the §4.5.3 exact-merge determinism (register P-IM-2) — fixed by summing each key's contributions in canonical (ascending-sorted) order + a regression test. Zero-row exemption requires an explicit recorded, justified exception (doc/config-only units). |

Design details in `docs/research/gnosis-data-structures-concurrency-plan.md`.

The Gnosis spec pins new decision rows when the relevant unit lands, e.g.
RESULT-LEVEL-PROVENANCE, QUERY-AUDIT-LOG, GRAPH-MODE-WALK,
REFERENCE-GRAPH-ADDITIVE-FIELDS, FANOUT-INTERLEAVE-MERGE.

## SUPERSEDED

_(none yet.)_
