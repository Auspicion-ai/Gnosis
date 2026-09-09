# Gnosis — Work Queue

Maintained by the document-archival loop. Open work on top; finished items move
to the tracker rows they produced. This is Gnosis's local next-steps.

Gnosis is the **production graph/vector engine** of the Auspicion Suite — a
Rust backend that owns the document store, the knowledge graph, fact/citation
tracking, consistency enforcement, and the RAG/agent-memory retrieval stack. It
is the production replacement for the archived Incanter prototype. The canonical
contract is `docs/specs/gnosis.md`.

## CURRENT WORK / handover-state

**Scaffold (2026-09-09): the project is scaffolded.** The folder structure, the
canonical spec (`docs/specs/gnosis.md`, copied from the Auspicion Suite), the
relevant research notes (`docs/research/`), the integration-surface docs
(`docs/integrations/`), the Rust crate skeleton (`Cargo.toml`, `src/`), and the
trackers are in place. No implementation code has been written yet.

**Housekeeping (2026-09-09):** `archive/` is now gitignored (decision
ARCHIVE-GITIGNORED) and `Cargo.lock` is committed (decision LOCKFILE-COMMITTED)
so build reproducibility is pinned. First archival-loop pass run — nothing to
archive yet (no obsolete docs or findings reports exist at the scaffold stage);
all citations verified against current files.

**Trio green at scaffold (2026-09-09):** `cargo build`, `cargo test`
(1 scaffold test), `cargo clippy --all-targets`, and `cargo fmt --check` are all
clean after installing the host build prerequisites. **Linux build/dep
prerequisites:** a C linker (`gcc`/`cc`), `pkg-config`, and `libssl-dev` (for
`reqwest`'s native-tls TLS). If a CI/consumer host lacks them, `reqwest`'s TLS
backend can be switched to `rustls` at the cost of a `cmake`-building provider —
currently the native-tls default is kept.

**Data-structure & concurrency plan (2026-09-09):** the pre-implementation design
is `docs/research/gnosis-data-structures-concurrency-plan.md` — answers live-
update/overlay, speed-at-volume under concurrent requests, and Safe-Rust
mutable+shared tension via ownership splitting. Decisions pinned:
SHARDED-RWLOCK-STORE, IMMUTABLE-DERIVED-SNAPSHOT, WRITER-ACTOR-JOURNAL,
LAYERED-OVERLAY, ARC-SHARED-ENGINE (`docs/decisions.md`). The core-unit TestWriter
red sets (§4.1–§4.4) must now exercise the concurrency / optimistic-concurrency /
overlay states that this plan pins.

**§4.1 document store DONE (2026-09-09).** TDD red→green to green with the
adversarial findings fixed; full details in the DONE row below. The next unit to
delegate is **§4.2 knowledge graph** (nodes/edges/properties, subject-relation
model, entity resolution, manual overrides), which attaches to the store's new
journal/epoch feed.

**§4.2 knowledge graph DONE (2026-09-09).** Red→green; the §4.2 adversarial gate
rejected the first green (sidecar second-source-of-truth) and drove a decided
**type evolution** (GRAPH-OWNS-RELATION-AND-MERGE) so triples/merges/resolutions
are graph-owned; fixed + regression-pinned. Details in the DONE row. Next unit:
**§4.3 fact/citation**.

## OPEN

| Unit | Status | Notes |
| --- | --- | --- |
| **Fact/citation tracking** (spec §4.3) | ready to delegate (next) | Fact nodes, citations/provenance, the candidate-fact + deterministic-validation pipeline. Builds on the graph-owned fact store (`create_fact`/`get_fact`/`merge_facts` already landed). Own red→green→adversarial→blind-greens→doc-review cycle, from the spec alone. |
| **Consistency enforcement** (spec §4.4) | pending | The consistency invariant, staleness propagation, the publish gate, `getConsistencyReport`. After §4.3. |
| **The RAG/agent-memory retrieval stack** (spec §4.5–§4.6) | ready to delegate (after the core) | The query modes (`flat`/`graph`/`vector`/`hybrid`), the retrieval stack (lexical BM25, reranking, multi-query, compression, HyDE, sub-task DAG), multiple vector fields, the agent-memory surface, and the query/stream/engine-status API. Delegate per unit after the core lands. |
| **The IPC/HTTP engine seam** (spec §4.6.1, §5.1) | pending | The exact IPC/process transport between the shell and Gnosis (the `RagStore` + query/stream/engine-status seam) — F2 in the spec. Design and delegate once the core API exists. |

## DONE

| Unit | Red set | Green | Adversarial findings | Blind-greens | Doc-review | Trio |
| --- | --- | --- | --- | --- | --- | --- |
| **§4.1 document store** (spec §4.1) | TestWriter red: **41 red + 1 green** at the compile-with-stubs stage; after the red-set fixture corrections, the implementer landed **42/42**, then the adversarial regression set added **8 tests** (5 red) → **50/50** | 50/50 (`tests/store_integration.rs`) + placeholder 1 | **HIGH** out-of-range pagination panic (fixed: clamp/overflow-safe); **MEDIUM** delete TOCTOU dangling-reference (fixed: store-wide `reference_integrity` `RwLock`); **MEDIUM** crosslink `Broken`/`Stale` not gated on publish (fixed); **MEDIUM** fabricated reference state trusted (fixed: derive state from target existence, skip cross-wiki); **MEDIUM** WRITER-ACTOR-JOURNAL not honored (fixed: minimal mutation journal + epoch feed); concurrency tests single-threaded false security (fixed: multi-threaded + Barrier); plus low/`unwrap`/ordering notes | pending (documentation gates) | pending (documentation gates) | `cargo test` 51 pass (50 store + 1 placeholder) · `build` clean · `clippy` clean · `fmt` clean |
| **§4.2 knowledge graph** (spec §4.2) | TestWriter red: **55 red** at the stubs stage; after the §4.2 adversarial gate, **10 regression tests** pinned the CRITICAL/HIGH findings (rejected first green) → **66/66** after the graph-owned type evolution + a `max_hops` spec-conflict rebase | 66/66 (`tests/graph_integration.rs`) + store 50 + placeholder 1 | §4.2 adversarial rejected the first green. **CRITICAL (all fixed + regression-pinned):** triple `relationType` lived in a sidecar, not the graph (§4.2.7.2 — fixed via decided type evolution, `Edge.relation_type`); triple cascade only masked, leaving a `triple_store` second-source-of-truth and duplicate-on-re-add (fixed: prune relation edges on node delete + derive membership from the graph); `merge_facts` computed but never persisted (fixed: write back union citations + `updated_at` + journal + `get_fact`); `resolve_entities` had no durable effect (fixed: journaled alias→canonical `entity_resolution` + `entity_alias_canonical`). **HIGH:** `set_reference_state` whole-graph clobber + no reference-lock/optimistic compare (fixed: targeted edge-state mutation under `reference_lock` + revision-aware reconcile so concurrent updates don't lose data); `resolve_references` unbounded `max_hops` + ignored wiki (fixed: validate 1–5 → `ValidationError`, unknown wiki → `WikiNotFound`); `add_triple`/`get_triples` unknown wiki (fixed). **SPEC-CONFLICT:** three `resolve_references` tests used `max_hops:10` outside the pinned 1–5 — rebased to `2` (still ≥ chain length). **HANDOFF (upstream):** spec §4.2.9.1/§4.2.7.2/`resolveEntities` result-shape should be reconciled to match the graph-owned realization | pending (documentation gates) | pending (documentation gates) | `cargo test` 117 pass (graph 66 + store 50 + placeholder 1) · `build` clean · `clippy` clean · `fmt` clean |
