# Gnosis — Work Queue

Maintained by the document-archival loop. Open work on top; finished items move
to the tracker rows they produced. This is Gnosis's local next-steps.

Gnosis is the **production graph/vector engine** of the Auspicion Suite — a
Rust backend that owns the document store, the knowledge graph, fact/citation
tracking, consistency enforcement, and the RAG/agent-memory retrieval stack. It
is the production replacement for the archived Incanter prototype. The canonical
contract is `docs/specs/gnosis.md`.

## CURRENT WORK / handover-state

**Handover state (current): the core engine is fully implemented.** §4.1–§4.6 are
all DONE (see the DONE rows below) and adversarially hardened — **233 green tests**
(store 50 [incl. the §4.1.1 `doc_state_legal_transitions` state-machine test] +
graph 66 + facts 36 + consistency 22 + rag_query 34 + retrieval_stack 18 +
agent_memory 6 + the `tests/integration.rs` placeholder `scaffold_compiles` 1) —
with `build`/`clippy`/`fmt` clean. The **only OPEN unit is the §4.6.1/§5.1 F2 engine seam** (the concrete
IPC/HTTP transport between the shell and Gnosis over the `RagStore` +
`ragQuery`/`ragStream`/`engine-status` interface). The dated entries below are the
historical scaffold→implemented record.

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
are graph-owned; fixed + regression-pinned. Details in the DONE row.

**§4.3 fact/citation DONE (2026-09-09).** Red→green; the §4.3 adversarial gate
found dangling-citation / whitespace-value / wiki-consistency defects, all fixed +
regression-pinned. Cross-field consistency declared deferred to §4.5. Details in
the DONE row.

**§4.4 consistency enforcement DONE (2026-09-09).** Red→green; the §4.4
adversarial gate found a HIGH AB-BA deadlock (eliminated by the uniform lock-order
decision LOCK-ORDER-REF-SHARD-SIDECAR) + propagation/report defects, all fixed +
regression-pinned. Details in the DONE row.

**§4.5 RAG/agent-memory retrieval DONE (2026-09-09) — after an adversarial REJECT + rework.**
The initial green was correctly rejected by the §4.5 adversarial gate (vector/hybrid
ran the lexical leg, retrieval-stack fail-states were dead code, several greens
vacuous). A real rework wired the vector/hybrid legs, made the retrieval-stack
fail-states reachable, wiki-scoped the vector leg, fixed deterministic graph-leg
ordering, and de-vacuated the weak tests; a second adversarial re-audit confirmed
the blockers resolved (parked/reserved items documented honestly, not faked). **The
core engine (§4.1–§4.6) is now fully implemented and adversarially hardened.**

**§4.5 RE-AUDIT FIX PASS (2026-09-09):** the §4.5 re-audit left 2 genuine RED tests
+ several documentary items, all now closed:
**MEDIUM-A** (graph-leg determinism — sort walk roots + resolved results by
`(DocumentId, NodeId)` ascending before `top_k`/RRF in `graph_query` + `hybrid_graph_leg`)
**FIXED** → `graph_hybrid_ordering_is_deterministic` GREEN (stable); **LOW-E** (HyDE
embedding errors surface as `EmbeddingUnavailable`) **FIXED**; **LOW-G**
(`binaryCandidatePool` default `10×topK`) **code-resolved** and its spec-conflict test
**corrected** (seed arithmetic made consistent with the default) → `retrieval_stack` 18/18.
`subTaskDag`/`CompressionFailed`/`HyDEGenerationFailed` recorded as **reserved** variants
(decisions SUB-TASK-DAG-VALIDATED-ONLY + RESERVED-ERRVARIANTS-DISCIPLINE), sub-task DAG
parked (`docs/pending.md`), FS-13/14/15 lexical-index tension in `docs/HANDOFF.md`.

## OPEN

| Unit | Status | Notes |
| --- | --- | --- |
| **The IPC/HTTP engine seam** (spec §4.6.1, §5.1) | pending (next) | The exact IPC/process transport between the shell and Gnosis (the `RagStore` + `ragQuery`/`ragStream`/`engine-status` seam) — **F2** in the spec. The logical API now exists (the §4.6.1 query surface is implemented); the concrete wire/process transport is the F2 design step. Delegate after the documentation gates + before Astrographer wiring. |

## DONE

| Unit | Red set | Green | Adversarial findings | Blind-greens | Doc-review | Trio |
| --- | --- | --- | --- | --- | --- | --- |
| **§4.1 document store** (spec §4.1) | TestWriter red: **41 red + 1 green** at the compile-with-stubs stage; after the red-set fixture corrections, the implementer landed **42/42**, then the adversarial regression set added **8 tests** (5 red) → **50/50** | 50/50 (`tests/store_integration.rs`) + placeholder 1 | **HIGH** out-of-range pagination panic (fixed: clamp/overflow-safe); **MEDIUM** delete TOCTOU dangling-reference (fixed: store-wide `reference_integrity` `RwLock`); **MEDIUM** crosslink `Broken`/`Stale` not gated on publish (fixed); **MEDIUM** fabricated reference state trusted (fixed: derive state from target existence, skip cross-wiki); **MEDIUM** WRITER-ACTOR-JOURNAL not honored (fixed: minimal mutation journal + epoch feed); concurrency tests single-threaded false security (fixed: multi-threaded + Barrier); plus low/`unwrap`/ordering notes | pending (documentation gates) | pending (documentation gates) | `cargo test` 51 pass (store 50 + placeholder 1) · `build` clean · `clippy` clean · `fmt` clean |
| **§4.2 knowledge graph** (spec §4.2) | TestWriter red: **55 red** at the stubs stage; after the §4.2 adversarial gate, **10 regression tests** pinned the CRITICAL/HIGH findings (rejected first green) → **66/66** after the graph-owned type evolution + a `max_hops` spec-conflict rebase | 66/66 (`tests/graph_integration.rs`) + store 50 + placeholder 1 | §4.2 adversarial rejected the first green. **CRITICAL (all fixed + regression-pinned):** triple `relationType` lived in a sidecar, not the graph (§4.2.7.2 — fixed via decided type evolution, `Edge.relation_type`); triple cascade only masked, leaving a `triple_store` second-source-of-truth and duplicate-on-re-add (fixed: prune relation edges on node delete + derive membership from the graph); `merge_facts` computed but never persisted (fixed: write back union citations + `updated_at` + journal + `get_fact`); `resolve_entities` had no durable effect (fixed: journaled alias→canonical `entity_resolution` + `entity_alias_canonical`). **HIGH:** `set_reference_state` whole-graph clobber + no reference-lock/optimistic compare (fixed: targeted edge-state mutation under `reference_lock` + revision-aware reconcile so concurrent updates don't lose data); `resolve_references` unbounded `max_hops` + ignored wiki (fixed: validate 1–5 → `ValidationError`, unknown wiki → `WikiNotFound`); `add_triple`/`get_triples` unknown wiki (fixed). **SPEC-CONFLICT:** three `resolve_references` tests used `max_hops:10` outside the pinned 1–5 — rebased to `2` (still ≥ chain length). **HANDOFF (upstream):** spec §4.2.9.1/§4.2.7.2/`resolveEntities` result-shape should be reconciled to match the graph-owned realization | pending (documentation gates) | pending (documentation gates) | `cargo test` 117 pass (graph 66 + store 50 + placeholder 1) · `build` clean · `clippy` clean · `fmt` clean |
| **§4.3 fact/citation tracking** (spec §4.3) | TestWriter red: **18 red + 6 green guards** at the stubs stage; after the §4.3 adversarial gate, **11 regression tests** pinned the findings → **36/36** | 36/36 (`tests/facts_integration.rs`) + graph 66 + store 50 + placeholder 1 | **HIGH:** delete gate didn't block/prune fact citations → dangling citations + a commit-time grounding TOCTOU (fixed: delete gate scans `fact_store` → `DocumentInUse`; fact commits take `reference_lock.read()` + re-verify grounding inside `fact_store.write()`). **MEDIUM:** `update_fact` accepted empty/whitespace value (fixed: trim→`ValidationError`); whitespace-only key/value passed schema conformance (fixed: trim before `is_empty` in gate + `create_fact`); inconsistent/absent unknown-wiki on the fact surface (fixed: `WikiNotFound` up front on `create_fact`/`update_fact`/`propose_candidate_fact`/`get_fact`); **cross-field consistency** doc mismatch — declared deferred to §4.5 (needs the embedding leg), test header corrected (not faked). **DEFERRED (pending):** fact-store sharding (single global `RwLock` → shard by wiki later); fact `node_id` is a store handle not a live graph node (reconcile by materializing graph `fact` nodes or dropping the claim); engine-side query **audit-log recording sink** (the `getQueryAuditLog` accessor exists; the recording feed lands in §4.5). **NIT/comment:** stale RED headers corrected | pending (documentation gates) | pending (documentation gates) | `cargo test` 153 pass (facts 36 + graph 66 + store 50 + placeholder 1) · `build` clean · `clippy` clean · `fmt` clean |
| **§4.4 consistency enforcement** (spec §4.4) | TestWriter red: **15 red** at the stubs stage (then two test-fixture defects fixed: a `"MTI"` seed typo + a read-before-join race) → **22/22** after the adversarial fix pass | 22/22 (`tests/consistency_integration.rs`) + facts 36 + graph 66 + store 50 + placeholder 1 | **HIGH (F1):** `publish_document` held a shard write while reading `fact_store`/other shards — an AB-BA deadlock vs the fact-commit paths (fixed: uniform lock order decision **LOCK-ORDER-REF-SHARD-SIDECAR**; publish validates under read locks then takes the single shard write only for the transition; fact-commits ground under `reference_lock.read()` before `fact_store.write()`). **MEDIUM:** propagation TOCTOU — embeds created after a fact-update scan could stay `FRESH` with a stale snapshot (fixed: derive embed state from snapshot-vs-canonical); `re_sync_embed` incoherent snapshot + non-fact targets (fixed); reference target-**change** (repoint) not propagated (fixed: re-stamp from new-target liveness); propagation didn't bump `revision`/journal on referencing docs (fixed). **LOW:** report fabricated state for `None`-state edges + wrong crosslink default (fixed: derive from target liveness; crosslink-with-snapshot = embed, crosslink-without = live link `RESOLVED`). **Concurrency tests:** added publish∥fact-commit∥re-sync stress + propagation/ordering regressions | pending (documentation gates) | pending (documentation gates) | `cargo test` 175 pass (consistency 22 + facts 36 + graph 66 + store 50 + placeholder 1) · `build` clean · `clippy` clean · `fmt` clean · concurrency stress stable + deadlock-free |
| **§4.5 RAG/agent-memory retrieval** (spec §4.5–§4.6) | TestWriter red: **47 red** at the stubs stage (3 files) + 4 green guards; **initially GREEN but ADVERSARIAL-REJECTED**; after the real rework + re-audit → **58/58** | 58/58 (rag_query 34 + retrieval_stack 18 + agent_memory 6) + store 50 + graph 66 + facts 36 + consistency 22 + placeholder 1 | First adversarial gate **rejected** the initial green (real, not shape-only): HIGH-1 `ragQuery(mode:vector)` ran the LEXICAL leg behind a vector trace (fixed: real dense vector leg + provider seam + `EmbeddingUnavailable`/`VectorIndexUnavailable` reachable from `rag_query`); HIGH-2 hybrid used the lexical list twice → degenerate flat (fixed: 3 distinct graph/vector/lexical legs + exact RRF); HIGH-3 retrieval fail-states were unreachable dead code (mostly fixed; `subTaskDag`/`CompressionFailed`/`HyDEGenerationFailed` recorded as **reserved** variants via decisions SUB-TASK-DAG-VALIDATED-ONLY + RESERVED-ERRVARIANTS-DISCIPLINE, sub-task DAG parked, FS-13/14/15 lexical tension → HANDOFF); HIGH-4 multi-query/compression/hyde were silently ignored (fixed: real fan-out/merge, filter compression, HyDE routing); MEDIUM-5 cross-wiki vector leak (fixed: wiki-scope); MEDIUM-6/8/9 vacuous greens (stale-parent, profile-value, non-contending concurrency) — all de-vacuated; MEDIUM-7 fabricated `Resolved` walk default (kept, ingest-derivation reachable); MEDIUM-10 `blocked_by` on non-empty (fixed). Second re-audit: MEDIUM-A **graph-leg nondeterminism in RRF/hybrid/graph** (fixed: sort by `(DocumentId,NodeId)` before `top_k`/RRF), LOW-G `binaryCandidatePool` default 10×topK (fixed + spec-conflict test corrected), LOW-E HyDE error mapping (fixed), LOW-F HyDE test de-vacuated. **233 tests total, trio clean** | pending (documentation gates) | pending (documentation gates) | `cargo test` 233 pass (rag_query 34 + retrieval 18 + agent_memory 6 + store 50 + graph 66 + facts 36 + consistency 22 + placeholder 1) · `build` clean · `clippy` clean · `fmt` clean · determinism + concurrency stable |
