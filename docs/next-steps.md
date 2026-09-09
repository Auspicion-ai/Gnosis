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

## OPEN

| Unit | Status | Notes |
| --- | --- | --- |
| **Knowledge graph** (spec §4.2) | ready to delegate (next) | Nodes/edges/properties, `reference`→`fact` graph, subject-relation triple model (§4.2.7), entity resolution, manual overrides (§4.2.8). Builds on the §4.1 store's journal/epoch feed. Own red→green→adversarial→blind-greens→doc-review cycle, from the spec alone. |
| **Fact/citation tracking** (spec §4.3) | pending | Fact nodes, citations/provenance, the candidate-fact + deterministic-validation pipeline. After §4.2. |
| **Consistency enforcement** (spec §4.4) | pending | The consistency invariant, staleness propagation, the publish gate, `getConsistencyReport`. After §4.2/§4.3. |
| **The RAG/agent-memory retrieval stack** (spec §4.5–§4.6) | ready to delegate (after the core) | The query modes (`flat`/`graph`/`vector`/`hybrid`), the retrieval stack (lexical BM25, reranking, multi-query, compression, HyDE, sub-task DAG), multiple vector fields, the agent-memory surface, and the query/stream/engine-status API. Delegate per unit after the core lands. |
| **The IPC/HTTP engine seam** (spec §4.6.1, §5.1) | pending | The exact IPC/process transport between the shell and Gnosis (the `RagStore` + query/stream/engine-status seam) — F2 in the spec. Design and delegate once the core API exists. |

## DONE

| Unit | Red set | Green | Adversarial findings | Blind-greens | Doc-review | Trio |
| --- | --- | --- | --- | --- | --- | --- |
| **§4.1 document store** (spec §4.1) | TestWriter red: **41 red + 1 green** at the compile-with-stubs stage; after the red-set fixture corrections, the implementer landed **42/42**, then the adversarial regression set added **8 tests** (5 red) → **50/50** | 50/50 (`tests/store_integration.rs`) + placeholder 1 | **HIGH** out-of-range pagination panic (fixed: clamp/overflow-safe); **MEDIUM** delete TOCTOU dangling-reference (fixed: store-wide reference-integrity `RwLock`); **MEDIUM** crosslink `Broken`/`Stale` not gated on publish (fixed); **MEDIUM** fabricated reference state trusted (fixed: derive state from target existence, skip cross-wiki); **MEDIUM** WRITER-ACTOR-JOURNAL not honored (fixed: minimal mutation journal + epoch feed); concurrency tests were single-threaded false security (fixed: multi-threaded + Barrier); plus low/`unwrap`/ordering notes | pending (documentation gates) | pending (documentation gates) | `cargo test` 51 pass (50 store + 1 placeholder) · `build` clean · `clippy` clean · `fmt` clean |
