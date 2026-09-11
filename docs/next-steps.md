# Gnosis — Work Queue

> **⚠️ SESSION CHECKPOINT (2026-09-10):** the prior session was corrupted. A fresh supervisor
> should read **`CHECKPOINT-2026-09-10.md`** (at the repo root) FIRST — it is the authoritative
> handover for the CRUD-unblock roadmap (P1a + P2 LANDED; A1 in progress with a subagent
> reliability issue — the A1 files do not land on disk and must be re-done + verified).
> **RESOLVED (2026-09-10):** the A1 subagent-reliability hazard is closed — all A1 files are
> verified on disk and the A1 unit is LANDED-GREEN (see the CURRENT WORK block below).

Maintained by the document-archival loop. Open work on top; finished items move
to the tracker rows they produced. This is Gnosis's local next-steps.

Gnosis is the **production graph/vector engine** of the Auspicion Suite — a
Rust backend that owns the document store, the knowledge graph, fact/citation
tracking, consistency enforcement, and the RAG/agent-memory retrieval stack. It
is the production replacement for the archived Incanter prototype. The canonical
contract is `docs/specs/gnosis.md`.

## CURRENT WORK / handover-state

**Handover state (current): the core engine, the wire-contract seam, the F6
RAG-evaluation dev/QA gate, the F4 community-retrieval unit, the §7.2 P1a
document-CRUD wire-contract unit, AND the §7.2 P2 `gnosis-server` binary crate
are fully implemented, adversarially hardened, and trio-green.** §4.1–§4.6 (the
core engine), the §7.2/F2 engine wire contract, the §7.8/F6 RAG-evaluation
harness, the §7.5/F4 community-retrieval accessor, the §7.2/P1a document-CRUD
wire shapes, and the §7.2/P2 `gnosis-server` bin are all DONE (see the DONE rows
below) — **523 green tests** (baseline 454 + the P2 set 63:
`gnosis_server_conformance.rs` 25 + `props_gnosis_server.rs` 7, all 7 property
rows HELD + `gnosis_server_e2e.rs` 16; the P2 blind-greens
`tests/blind_p2_gnosis_server_greens.rs` 15 = 15/15 scenarios PASS are a separate
verification layer), with `build`/`clippy`/`fmt` clean.
**All previously-unparked items in this repo's scope are now complete**:
`resolve_entities` authoritative-overwrite, the UUID-v4 id decision (RESOLVED as
`ID-SCHEME-RECONCILE-MONOTONIC`), F6, F4, P1a, and P2. **The Astrographer-side
CRUD routing client (A1) is LANDED (2026-09-10)** — the shell-side client that
consumes the same paths + shapes the `gnosis-server` bin serves (the 11 §4.1
document-CRUD wire client over the frozen P1a wire; see the A1 DONE row in the
Astrographer `docs/next-steps.md` + the ENGINE-CRUD-WIRE-CLIENT / ENCODE+DECODE /
P4-IDEMPOTENCY-RETRY decision rows in the Astrographer `docs/decisions.md`). **The
Astrographer-side A2 document-CRUD D4 wiring is LANDED (2026-09-10)** — the final
MVP unit: the 11 `gnosis.document.*`/`gnosis.wiki.*` MCP tools + the `gnosis-edit`
group + the extended `handleGnosisTool` + the `AuthorityStore` + the
`IdempotencyRegistry` + the GUI document-editor/wiki screens over the LANDED A1
proxy (see the A2 DONE row in the Astrographer `docs/next-steps.md` + the
GNOSIS-CRUD-EDIT-GROUP / GNOSIS-DOCUMENT-TOOLS / GNOSIS-RBAC-CALLER-STORE /
GNOSIS-409-UX / GNOSIS-IDEMPOTENCY-DEDUP decision rows in the Astrographer
`docs/decisions.md`). **THE DOCUMENT-CRUD MVP IS COMPLETE (P1a + P2 + A1 + A2).**
The P1a/P2/A1/A2 live batteries un-park on a running `gnosis-server` + a running
app (the A2 live battery is PARKED — the Gnosis engine is absent; revisit
condition: the engine running on loopback + the `gnosis-edit` group enabled).
**RBAC caller threading RESOLVED as re-scoped to the shell (2026-09-10, proposal
gate — `docs/specs/rbac-caller-review.md`):** the engine-side RBAC enforcement is
deliberately NOT implemented (the engine has no authority-mapping source; the C5
boundary + the pure-backend framing place the authorization gate at the SHELL;
the shell's A2 caller-side deny is the enforcement). Decision
`GNOSIS-RBAC-EDIT-ENFORCEMENT` amended to align with C5 (RBAC → SHELL); the
`defects.md` + `HANDOFF.md` OPEN rows closed as re-scoped to the shell; a
canonical-contract reconcile request recorded in `docs/HANDOFF.md`. A
fresh supervisor picks up the **shell-integration unit** next (SSE client +
bind/auth/TLS + full `RagStore` CRUD routing + boot→READY lifecycle +
HTTP-status rendering + D2-absent shell behavior). The **suite-side DeepEval Python harness**
(faithfulness/answer-relevancy, answer generation, user-facing retrieval-quality
surface), the **F4-LLM integration** (a suite tool with a harnessed LLM drives
Gnosis's manual-override-authoritative enrichment surfaces), and the
**authorship-source code-bearing unit** (decision AUTHORSHIP-SOURCE-PROPERTY)
remain deferred/out-of-repo and are recorded in `docs/specs/7-2-f2-review.md`
(§"Deferred / OUT of F2" + §"What a later shell-integration unit must own"),
`docs/specs/6-f6-eval-review.md`, `docs/specs/f4-llm-enrichment-integration.md`,
`docs/specs/authorship-source-review.md`, `docs/pending.md`, and
`docs/decisions.md` (`F2-WIRE-CONTRACT-A1`, `F6-EVAL-RE-SCOPED`,
`F4-COMMUNITY-RETRIEVAL`, `AUTHORSHIP-SOURCE-PROPERTY`). The dated entries below
are the historical scaffold→implemented record.

**§4.5-DEFERRED REVISIT (2026-09-09, change-analysis pass):** the four
`ENGINE-INTERNAL DEFERRED` rows previously framed as "revisit with §4.5" were
re-derived against the actual crate now that §4.5 has landed:
**audit-log recording sink — RESOLVED** (landed in §4.5: `rag_query`/`rag_stream`
append real `QueryAuditEntry`s; §4.3.4 returns real entries, pinned by two tests);
**fact-store sharding — KEEP DEFERRED** (re-scoped to a **measured** contention/throughput signal; no §4.5 load path justifies it yet); **fact `node_id` coherence — CLOSED** as a spec-wording tension (`factKey` is the canonical identity; engine will not materialize fact nodes); **cross-field consistency gate — CLOSED** as a documented non-goal (deterministic fail-closed gate; reconcile spec §4.3.2a.1 as aspirational/offline). All recorded in `docs/pending.md` + `docs/defects.md`; the two closures carry spec-reconcile notes in `docs/HANDOFF.md`. None of the four requires new engine work.

**DOC-REVIEW QUICK-PIN PASS (2026-09-09, DONE):** the doc-review gaps that are
implementable/checkable now were pinned by a test pass — `author` round-trip was
already **CLOSED by the PBT retrofit** (`props_store.rs` P-TP-2); **ISO-8601 UTC
format** pinned via `assert_iso8601_utc` in store/facts/graph suites;
**`MultiQueryExpansionFailed` (FS-19) reachability** pinned via an empty-wiki seed
in `rag_query_integration.rs` (+ stream error-chunk mirror). Engine ids not
RFC-4122 UUID v4 remains **deferred to the F2 seam** (an on-the-wire contract
decision, not a test pin). Test count 293 → **295** (see the DONE row).

**PBT-GATE RETROFIT (2026-09-09, decision PBT-GATE-MANDATORY):** the mandatory
property-based-testing gate now applies to every code-bearing unit. Five typed
property registers (`docs/specs/4-*-property-register.md`, P-IM/P-SM/P-TP ≤8 rows
each), five executed property layers (`tests/props_*.rs`, each a deterministic
hand-rolled SplitMix64/Xoshiro PRNG with a pinned seed, ≤100 generated
cases/row, ≤400 total, stop-after-5, HELD/BROKEN + strategy-id), and five read-only
PBT audits landed. Register rows are invariants-only (never §6/FS-n or §7/F-gap
rows), each TRUE of the green implementation. Result: **39 of 40 property rows
HELD**; the gate surfaced and host-fixed a **genuine defect** — `rrf_fuse`'s f64
RRF accumulation was non-associative so exact-tied keys landed 1 ULP apart by
input-list order, breaking §4.5.3 exact-merge determinism (register P-IM-2);
fixed (canonical sorted order of per-key contributions) + regression test in
`tests/retrieval_stack_integration.rs`. Negative-generator probes added from the
audits across all 5 units; the §4.3 register's stale `[PENDING]` tags were
re-tagged GREEN (those defects were already fixed). Two engine-internal
non-goals/behaviours recorded: the §4.4 `publish_document` concurrent-atomicity
TOCTOU (defects.md + HANDOFF.md) and the §4.2 `resolve_entities` residual-alias
behaviour (defects.md + pending.md).

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

**§7.2 F2 engine wire contract DONE (2026-09-09).** Red→green; the last code-unit in
this repo's scope is now implemented + adversarially hardened + trio-green. Contract:
`docs/specs/engine-wire-contract.md` (+ property register
`docs/specs/7-2-wire-property-register.md`), proposal-review record `docs/specs/7-2-f2-review.md`,
decision `F2-WIRE-CONTRACT-A1`. Details in the DONE row. **360 green tests** (baseline 295 +
65 F2: `wire_conformance` 57 [47 conformance + 2 regressions + 8 probes] + `props_wire` 8).

## OPEN

| Unit | Status | Notes |
| --- | --- | --- |
| **_(shell-integration unit + suite-side DeepEval harness + F4-LLM integration + authorship-source code-bearing unit)_** | **shell-integration: RE-SCOPED (2026-09-10, proposal gate — `docs/specs/shell-integration-review.md` in the Astrographer repo); the rest DEFERRED / out of this repo's current scope** | The shell-integration unit's **7-item scope is largely LANDED** (P2 server host + READY lifecycle + §11 rendering; GN SSE client + auth/TLS options; A1/A2 document-CRUD routing; GN-MCP-UI + A2 D4 panes). The proposal gate **re-scoped it to Option B** — the five genuine remaining deliverables, owned by an Astrographer-shell unit: (1) TLS application (a shared `engine-transport.ts` https-agent-backed `fetch`); (2) a fetch-based SSE client (sends the Bearer header, no auto-reconnect); (3) a concrete bind/auth/TLS policy record; (4) the D2-fallback clarification (the document-CRUD surface surfaces `EngineUnavailable`; the local `rag.*` surface is the parallel D2 fallback); (5) the e2e transport test / live-battery revisit. Item 4 (full `RagStore` CRUD routing) is **dropped** — the 11-method A1/A2 routing is landed; the ~45-method surface stays deferred per `GNOSIS-CRUD-MVP-SCOPE`. The **suite-side DeepEval Python harness** (faithfulness/answer-relevancy, answer generation, user-facing retrieval-quality surface) is **NOT Gnosis** — recorded in `docs/specs/6-f6-eval-review.md` + `docs/decisions.md` `F6-EVAL-RE-SCOPED`. The **F4-LLM integration** (a suite tool with a harnessed LLM drives Gnosis's manual-override-authoritative enrichment surfaces) is **NOT a Gnosis-repo code unit** — recorded in `docs/specs/f4-llm-enrichment-integration.md` + `docs/pending.md`. The **authorship-source code-bearing unit** (decision AUTHORSHIP-SOURCE-PROPERTY) is **doc/design-only now** — recorded in `docs/specs/authorship-source-review.md` + `docs/decisions.md`. The **UUID-v4 id decision** is **RESOLVED** as `ID-SCHEME-RECONCILE-MONOTONIC` (ids stay monotonic opaque strings; RFC-4122 not taken up; reconcile request in `docs/HANDOFF.md`). |

## DONE

| Unit | Red set | Green | Adversarial findings | Blind-greens | Doc-review | Trio |
| --- | --- | --- | --- | --- | --- | --- |
| **§7.2 P1a document-CRUD wire contract** (`src/wire/crud.rs` + `DecodeError::UnknownMethod` in `src/wire/decode.rs` + re-export surface in `src/lib.rs`; spec `docs/specs/p1a-document-crud-wire.md`, PBT register §5.x in the spec, blind-greens `docs/specs/p1a-document-crud-wire-greens.md`, live-pending battery `docs/specs/p1a-document-crud-wire-live-pending-battery.md`) | TestWriter red: **missing `src/wire/crud.rs`** (the 11 `CrudMethod` variants + `CrudRequestArgs`/`CrudResult`/`CrudResponseError`/`CrudValidationFailure` + the codecs + `ENGINE_ENDPOINTS`/`ENDPOINT_*`), **`DecodeError::UnknownMethod`** (new variant in `src/wire/decode.rs`), and the re-export surface in `src/lib.rs` | **64 P1a tests** (35 conformance in `tests/crud_wire_conformance.rs` + 8 property rows in `tests/props_crud_wire.rs` + 21 blind-greens in `tests/blind_p1a_crud_wire_greens.rs`); **full suite 454 pass / 0 fail** (baseline 411 + 43: conformance 35 + props 8; the 21 blind-greens are a separate verification layer) | Adversarial gate: **4 HOST-MINOR findings, all fixed + regression-tested; no PACKAGE** (host findings fixed in the unit's own `tests/`; no defect row warranted) | **19/19 scenarios PASS** (`docs/specs/p1a-document-crud-wire-greens.md` S1–S19, executed via `tests/blind_p1a_crud_wire_greens.rs` 21 `#[test]` = S1..S19 + 1 extra S8 endpoint-constant assertion + 1 supporting envelope-constant assertion) | **LIVE-SCENARIO PENDING** (parked on **A1** — the Astrographer CRUD client did not exist yet; the `gnosis-server` bin (P2) now exists and serves the CRUD endpoints, so the P1a battery resumes once A1 lands; battery `docs/specs/p1a-document-crud-wire-live-pending-battery.md` L1–L19 written); **A1 LANDED 2026-09-10** — the battery un-parks on A1 + a running app (see the CURRENT WORK block); doc-review reconciled spec/greens/battery + trackers | `cargo test` **454 pass, 0 fail** · `build` clean · `clippy` 0 warnings · `fmt` clean |
| **§7.2 P2 `gnosis-server` binary crate** (`src/bin/gnosis_server.rs` + `src/server.rs` + re-export surface in `src/lib.rs` + `[[bin]]`/axum in `Cargo.toml`; spec `docs/specs/p2-gnosis-server.md`, PBT register §5.x in the spec, blind-greens `docs/specs/p2-gnosis-server-greens.md`, live-pending battery `docs/specs/p2-gnosis-server-live-pending-battery.md`) | TestWriter red: **missing `src/bin/gnosis_server.rs`** (the loopback bind, the 14 REST/SSE endpoints, the READY boot lifecycle, the §11 status rendering + NEW-2 request-decode outcome, the RBAC `caller` threading), **`src/server.rs`** (the pure `server_status`/`request_decode_status`/`route_bijection` fns), and the re-export surface in `src/lib.rs` | **63 P2 tests** (25 conformance in `tests/gnosis_server_conformance.rs` + 7 property rows in `tests/props_gnosis_server.rs` + 16 e2e in `tests/gnosis_server_e2e.rs` + 15 blind-greens in `tests/blind_p2_gnosis_server_greens.rs`); **full suite 523 pass / 0 fail** (baseline 454 + 48: conformance 25 + props 7 + e2e 16; the 15 blind-greens are a separate verification layer); **all 7 property rows HELD** (P-IM-1..3, P-SM-1..3, P-TP-1) | Adversarial gate: **2 HOST-MAJOR + 4 HOST-MINOR findings, all fixed + regression-tested; 2 PACKAGE findings recorded** in `docs/defects.md` + `docs/HANDOFF.md` | **15/15 scenarios PASS** (`docs/specs/p2-gnosis-server-greens.md` S1–S15, executed via `tests/blind_p2_gnosis_server_greens.rs` 15 `#[test]`) | **LIVE-SCENARIO PARTIAL** — **15/15 server-endpoint scenarios PASS live** (S1–S15 against the running `gnosis-server` bin + the compiled blind-greens 15/15 + e2e 16/16 binaries); the **20 MCP/UI parity scenarios M1–M20 are PARKED on A1** (the Astrographer CRUD routing client did not exist yet; battery `docs/specs/p2-gnosis-server-live-pending-battery.md` M1–M20 written); **A1 LANDED 2026-09-10** — the battery un-parks on A1 + a running app (see the CURRENT WORK block); doc-review reconciled spec/greens/battery + trackers | `cargo test` **523 pass, 0 fail** · `build` clean · `clippy` 0 warnings · `fmt` clean |
| **§7.5 F4 community retrieval** (`src/store/mod.rs` `get_community_context` + `CommunityContext`; spec `docs/specs/f4-community-context-spec.md`, PBT register `docs/specs/4-2-graph-property-register.md` §7.5 F4 section, proposal-review `docs/specs/f4-community-summaries-review.md`) | TestWriter red: **19** (13 happy/fail in `tests/community_context_integration.rs` — `h1_declared_community_returns_matching_context_fresh`, `h2_update_document_rewriting_member_marks_stale`, `h3_re_derive_clears_stale_keeps_summary_members`, `h4_update_community_summary_reflects_new_manual_summary`, `h5_members_spanning_two_documents`, `h6_single_member_community`, `h7_member_that_is_a_fact_location`, `h8_never_triggered_reports_fresh`, `h9_two_consecutive_reads_are_equal`, `h10_membership_completeness`, `h11_manual_summary_survives_member_change_and_re_derive`, `f1_unknown_community_is_community_not_found`, `f2_wiki_not_found_cannot_fire` — + 6 PBT rows in `tests/props_community_context.rs` — `p_im1_context_determinism`, `p_im2_membership_completeness`, `p_sm1_read_side_effect_free`, `p_sm2_state_reflects_staleness`, `p_tp1_manual_summary_authority`, `p_tp2_faithful_projection`) | **411/411 total** (baseline 387 + 24 F4: `community_context_integration.rs` 15 [13 happy/fail + 2 negative probes `neg4_unknown_community_not_found_with_others_present`, `neg5_ghost_wiki_never_wiki_not_found`] + `props_community_context.rs` 9 [6 PBT rows + 3 negative probes `neg1_fact_incorporation_isolation`, `neg2_member_node_removal_stays_fresh`, `neg3_stale_fresh_stale_cycle`]) | Adversarial gate: **2 host findings, all fixed** — **HOST-1** the non-member-touch probe had an **id-collision** (`fresh_doc` built a NEW store whose first doc is `doc-0` — the same id as the outer `doc_a` — so the old probe rewrote the MEMBER's document, not a separate non-member doc) → fixed by creating a genuinely non-member doc on the SAME store + a regression assert; **P-SM-1** the **revisions leg** was hardened (the register's side-effect-free observable now also covers "no document revision bump" — LOW-1). **5 negative probes** added (`neg1`/`neg2`/`neg3` in props + `neg4`/`neg5` in integration). **PBT audit: 6/6 rows HELD** (P-IM-1, P-IM-2, P-SM-1, P-SM-2, P-TP-1, P-TP-2) | **33 GREEN / 0 RED / 2 NOT-VERIFIED** (`docs/greens/4-2-community-context-greens.md`: signature/return-shape, H-1..H-11, F-1/F-2, boundary shapes, addTriple non-trigger, 6 PBT rows; NOT-VERIFIED = the two unconstructible empty-set cases — empty member set + empty summary, both rejected by `declare_community` with `ValidationError`) | doc-review reconciled `docs/next-steps.md` (F4 DONE, 411), `docs/pending.md` (F4 LANDED), `docs/decisions.md` (F4-COMMUNITY-RETRIEVAL LANDED), `docs/defects.md` (no new defect row — adversarial findings were host-fixed), `docs/specs/f4-community-context-spec.md` (status + line refs), `docs/specs/4-2-graph-property-register.md` (line refs), `docs/specs/f4-community-summaries-review.md` (status) | `cargo test` **411 pass, 0 fail** · `build` clean · `clippy` 0 warnings · `fmt` clean |
| **§7.8 F6 RAG evaluation harness** (`src/retrieval/eval.rs` + `src/bin/gnosis_eval.rs`; spec `docs/specs/6-f6-eval-spec.md`, property register `docs/specs/6-f6-eval-property-register.md`, proposal-review `docs/specs/6-f6-eval-review.md`) | TestWriter red: **17** (9 red-set in `tests/eval_harness.rs` — `contextual_precision_reference_values`, `contextual_recall_reference_values`, `ndcg_at_k_reference_values`, `mrr_at_k_reference_values`, `empty_set_conventions`, `k_boundaries`, `duplicate_key_handling`, `no_panic_no_nan_bounded`, `corpus_parsing_and_metric_path` — + 8 property rows in `tests/props_eval.rs`); `query_mode_serde_pascal_case` was already-green (PascalCase serde, no stub) | **387/387 total** (baseline 368 + 19 F6: `eval_harness.rs` 11 [9 red-set + `query_mode_serde_pascal_case` + adversarial-fix bin smoke `bin_smoke_report_shape`] + `props_eval.rs` 8 [*p_im_1_metric_bounds*, *p_im_2_metric_repeat*, *p_im_3_empty_results*, *p_im_4_empty_relevant*, *p_sm_1_rank_improve*, *p_sm_2_k_monotone*, *p_tp_1_perfect_retrieval*, *p_tp_2_no_relevant*]) | Adversarial gate: **4 host findings, all fixed** — (1) the `--live` path was a **fake no-op** → replaced with a **real Ollama `LiveProvider`** (`reqwest` to `/api/embed`, `GNOSIS_EVAL_OLLAMA_URL`/`GNOSIS_EVAL_OLLAMA_MODEL`); (2) `map_mode` **panicked** on an unknown mode → now returns `Result` + graceful non-zero exit; (3) latent **`wellformed_graph` empty-nodes panic** → guarded (returns a well-formed empty graph); (4) the §10 **bin smoke test was missing** → added `bin_smoke_report_shape` (runs the built bin against the fixture, asserts the §7.3 report shape). **PBT audit: 8/8 rows HELD** (P-IM-1..4, P-SM-1..2, P-TP-1..2) | **35 GREEN / 0 RED / 3 NOT-VERIFIED** (`docs/greens/6-f6-eval-greens.md`: metric happy/fail-states, partial rankings, boundaries, empty-set conventions, bin report shape, mode-serde note; NOT-VERIFIED = `--live` real-provider path, per-case `rag_query` error branch, non-zero exit on hard failure) | doc-review reconciled `docs/next-steps.md` (387, F6 DONE, F4 next), `docs/pending.md` (F6 LANDED), `docs/decisions.md` (F6-EVAL-RE-SCOPED LANDED), spec + review status headers; **no new defect row** (adversarial findings host-fixed, not defects) | `cargo test` **387 pass, 0 fail** · `build` clean · `clippy` 0 warnings · `fmt` clean |
| **§4.2 `resolve_entities` authoritative-overwrite** (decision RESOLVE-ENTITIES-AUTHORITATIVE-OVERWRITE; unparked from `docs/pending.md` §ENGINE-INTERNAL DEFERRED) | TestWriter red: **3 red** (updated `p_tp2_resolve_entities_overlap_and_convergence` to assert convergence + new `p_tp2_resolve_entities_overlap_flip_no_residual` + `p_tp2_resolve_entities_chain_flatten`) vs the additive impl | **368/368 total** (baseline 360 + 8: 3 red-set tests + 6 adversarial probes − 1 updated overlap test; `props_graph.rs` 15→21) | Adversarial gate: **no host defects** (flatten algorithm correct — acyclic + flat + full convergence incl. chains/flips; contract/concurrency/regression clean). **PBT audit P-TP-2 HELD**; tightened the register observable (multiple roots legal, scoped to requested set + path-compressed aliases) + **6 negative probes** (`p_tp2_resolve_entities_three_hop_chain_flatten`, `_flip_root_with_aliases`, `_duplicate_ids`, `_multiple_roots_acyclic`, `_canonical_was_alias`, `_reaffirm_noop_idempotent`) | **7/7 GREEN, 0 RED** (`docs/greens/4-2-resolve-entities-greens.md`: basic, idempotence, overlap-flip, chain-flatten, convergence-back, fail-states, return shape) | doc-review reconciled `docs/pending.md` (RESOLVED), `docs/defects.md` (FIXED), `docs/decisions.md` (decision row), `docs/specs/4-2-graph-property-register.md` P-TP-2 | `cargo test` **368 pass, 0 fail** · `build` clean · `clippy` 0 warnings · `fmt` clean |
| **§7.2 F2 engine wire contract** (`src/wire/`; spec §4.6.1/§5.1 seam; contract `docs/specs/engine-wire-contract.md`, property register `docs/specs/7-2-wire-property-register.md`) | TestWriter red: **55** (47 conformance in `tests/wire_conformance.rs` + 8 property rows in `tests/props_wire.rs`), then the adversarial fix pass added **2 regression tests + 8 probes** → conformance 57; props wire 8/8 **HELD** | **360/360 total** (baseline 295 + 65 F2: `wire_conformance.rs` 57 [47 conformance + 2 regressions *`envelope_from_json_rejects_oversized_and_negative_schema_version`*, *`decode_done_with_extra_keys_is_malformed`* + 8 `probe_*` adversarial probes] + `props_wire.rs` 8 [*p_im_1_chunk_roundtrip*, *p_im_2_result_bijective*, *p_im_3_code_unique*, *p_im_4_sse_roundtrip*, *p_sm_1_envelope_stable*, *p_sm_2_validation_msg*, *p_sm_3_health_determinism*, *p_tp_1_encode_validates*]) | Adversarial gate on the wire unit: **A1** done-chunk exactness (`decode_chunk_payload` accepts `RagChunk::Done` only for exactly `{"type":"done"}` — extra keys malformed, regression *`decode_done_with_extra_keys_is_malformed`*); **A2** `schemaVersion` truncation (`from_json` rejects an out-of-`u32`-range `schemaVersion` rather than wrapping it — regression *`envelope_from_json_rejects_oversized_and_negative_schema_version`*); **A4** `code_table` display doc (the `ValidationError` row carries the fixed §5 label `"validation failed"`, never the dynamic wire `message` — documented in `src/wire/error.rs`; no code defect); plus PBT test fixes (P-IM-4/A3 SSE framing, P-IM-3 code-uniqueness) and **8 `probe_*`** adversarial probes (*probe_blocked_by_without_graph_across_flat_and_hybrid*, *probe_trace_mode_mismatch_is_transparent_not_a_failure*, *probe_unknown_schema_version_and_id_format_are_errors*, *probe_health_faithful_on_contradictory_input*, *probe_sse_event_data_mismatch_all_ordered_pairs*, *probe_error_codec_missing_message_and_unknown_code*, *probe_validation_error_message_with_code_message_data_survives*, *probe_done_chunk_strictness_via_sse_path*). **8/8 `props_wire` PBT rows HELD.** | **50 GREEN / 1 RED / 3 NOT-VERIFIED** (`docs/greens/7-2-wire-greens.md`). The single RED — `decode_rag_result` precedence for a body that is *both* malformed and trace-less (`{"query":123}` → `MissingTrace`, not `InvalidJson`) — was **resolved as a contract-precedence DOC-fix, not a code defect**: the contract now pins the precedence in §7 + §12 V-9 (trace-`key`-presence is checked **first**, so a malformed-and-traceless body stays FS-10 `TraceUnavailable`; both codes map to HTTP 502 so rendered status is identical), and `decode_rag_result` (`src/wire/decode.rs`) is verified to check trace-presence before structural deserialization. The empty-object `{}`-payload `UnknownType` nuance was folded into the same reconciliation (GREEN-with-note; net `EngineError` outcome unchanged). | This documentation-review pass reconciled the F2 spec + greens + trackers against the landed crate; **360** is the statically-enumerated `#[test]`/`#[tokio::test]` total (see `archive/reviews/2026-09-09-f2-wire-doc-review.md`). `docs/decisions.md` `F2-WIRE-CONTRACT-A1` confirmed accurate + landed; `docs/pending.md` F2 row updated to reflect the landed wire contract + remaining transport deferral; the blind-greens RED row annotated as resolved; no code defect row added (resolved as doc-fix). | `cargo test` **360 pass, 0 fail** (store 50 + graph 66 + facts 36 + consistency 22 + rag_query 36 + retrieval_stack 19 + agent_memory 6 + `integration.rs` `scaffold_compiles` 1 + props store 12/graph 13/facts 14/consistency 12/retrieval 8/wire 8 + wire_conformance 57) · `build` clean · `clippy` 0 warnings · `fmt` clean |
| **DOC-REVIEW QUICK-PIN pass** (2 open pins from `docs/pending.md` §DOC-REVIEW GAPS) | TestWriter red: **PIN B `MultiQueryExpansionFailed` (FS-19) reachable** — empty-wiki + `multiQuery:{enabled,n:2}` seed genuinely triggers it (RED→green, no seam); **PIN A ISO-8601 UTC format** — a new format assertion, RED→green (existing tests only asserted non-empty) | **295/295** (baseline 293 + 2: `rag_query_multi_query_empty_wiki_is_expansion_failed`, `rag_stream_multi_query_empty_wiki_emits_error_chunk`); `assert_iso8601_utc` wired into store/facts/graph create/update/merge assertions | N/A (correctness pins against already-green engine; no new behavior, no adversarial findings) | N/A (no UI scenario; unit-level test pins) | proofread + reconciled `docs/pending.md` (PIN A/B closed, `author` closed by prior PBT retrofit), `docs/next-steps.md` (293→295) | `cargo test` **295 pass, 0 fail** · `build` clean · `clippy` 0 warnings · `fmt` clean |
| **PBT-gate retrofit** (all code-bearing units, decision PBT-GATE-MANDATORY) | TestWriter red: §4.1–§4.4 + §4.5 property layers all **HELD** (40 rows); **§4.5 P-IM-2 genuinely BROKEN** (`rrf_fuse` f64 RRF accumulation non-associative → exact ties land 1 ULP apart by input order, defeating the id-ascending §4.5.3 tie-break) → host-fixed | **40/40 property rows HELD** after the `rrf_fuse` host fix (5 suites: props_store 12, props_graph 13, props_facts 14, props_consistency 12, props_retrieval 8) + negative-generator probes + 1 `rrf_fuse` regression (retrieval_stack 18→19) | 5 read-only PBT audits. **§4.5 P-IM-2 = genuine host defect (fixed + regression-pinned);** register re-scopes (store P-IM-1/P-SM-1/P-SM-2 monotonic-id + annotation-reconcile; graph P-TP-2 same-call idempotence + residual-alias; consistency P-IM-3/P-TP-2/P-SM-1/P-SM-3 scoping); §4.3 stale `[PENDING]` tags re-tagged GREEN (already-fixed defects); negative-generator probes added; **2 engine-internal items recorded**: §4.4 `publish_document` concurrent-atomicity TOCTOU non-goal (defects.md + HANDOFF.md), §4.2 `resolve_entities` residual-alias behaviour (defects.md + pending.md) | N/A (property-layer gates, not UI scenario) | proofreader applied all register/tracker corrections; test count reconciled to **293** (the archived doc-review's 234 overcount was itself wrong — actual baseline was 233) | `cargo test` **293 pass, 0 fail** · `build` clean · `clippy` 0 warnings · `fmt` clean |
| **§4.1 document store** (spec §4.1) | TestWriter red: **41 red + 1 green** at the compile-with-stubs stage; after the red-set fixture corrections, the implementer landed **42/42**, then the adversarial regression set added **8 tests** (5 red) → **50/50** | 50/50 (`tests/store_integration.rs`) + placeholder 1 | **HIGH** out-of-range pagination panic (fixed: clamp/overflow-safe); **MEDIUM** delete TOCTOU dangling-reference (fixed: store-wide `reference_integrity` `RwLock`); **MEDIUM** crosslink `Broken`/`Stale` not gated on publish (fixed); **MEDIUM** fabricated reference state trusted (fixed: derive state from target existence, skip cross-wiki); **MEDIUM** WRITER-ACTOR-JOURNAL not honored (fixed: minimal mutation journal + epoch feed); concurrency tests single-threaded false security (fixed: multi-threaded + Barrier); plus low/`unwrap`/ordering notes | pending (documentation gates) | pending (documentation gates) | `cargo test` 51 pass (store 50 + placeholder 1) · `build` clean · `clippy` clean · `fmt` clean |
| **§4.2 knowledge graph** (spec §4.2) | TestWriter red: **55 red** at the stubs stage; after the §4.2 adversarial gate, **10 regression tests** pinned the CRITICAL/HIGH findings (rejected first green) → **66/66** after the graph-owned type evolution + a `max_hops` spec-conflict rebase | 66/66 (`tests/graph_integration.rs`) + store 50 + placeholder 1 | §4.2 adversarial rejected the first green. **CRITICAL (all fixed + regression-pinned):** triple `relationType` lived in a sidecar, not the graph (§4.2.7.2 — fixed via decided type evolution, `Edge.relation_type`); triple cascade only masked, leaving a `triple_store` second-source-of-truth and duplicate-on-re-add (fixed: prune relation edges on node delete + derive membership from the graph); `merge_facts` computed but never persisted (fixed: write back union citations + `updated_at` + journal + `get_fact`); `resolve_entities` had no durable effect (fixed: journaled alias→canonical `entity_resolution` + `entity_alias_canonical`). **HIGH:** `set_reference_state` whole-graph clobber + no reference-lock/optimistic compare (fixed: targeted edge-state mutation under `reference_lock` + revision-aware reconcile so concurrent updates don't lose data); `resolve_references` unbounded `max_hops` + ignored wiki (fixed: validate 1–5 → `ValidationError`, unknown wiki → `WikiNotFound`); `add_triple`/`get_triples` unknown wiki (fixed). **SPEC-CONFLICT:** three `resolve_references` tests used `max_hops:10` outside the pinned 1–5 — rebased to `2` (still ≥ chain length). **HANDOFF (upstream):** spec §4.2.9.1/§4.2.7.2/`resolveEntities` result-shape should be reconciled to match the graph-owned realization | pending (documentation gates) | pending (documentation gates) | `cargo test` 117 pass (graph 66 + store 50 + placeholder 1) · `build` clean · `clippy` clean · `fmt` clean |
| **§4.3 fact/citation tracking** (spec §4.3) | TestWriter red: **18 red + 6 green guards** at the stubs stage; after the §4.3 adversarial gate, **11 regression tests** pinned the findings → **36/36** | 36/36 (`tests/facts_integration.rs`) + graph 66 + store 50 + placeholder 1 | **HIGH:** delete gate didn't block/prune fact citations → dangling citations + a commit-time grounding TOCTOU (fixed: delete gate scans `fact_store` → `DocumentInUse`; fact commits take `reference_lock.read()` + re-verify grounding inside `fact_store.write()`). **MEDIUM:** `update_fact` accepted empty/whitespace value (fixed: trim→`ValidationError`); whitespace-only key/value passed schema conformance (fixed: trim before `is_empty` in gate + `create_fact`); inconsistent/absent unknown-wiki on the fact surface (fixed: `WikiNotFound` up front on `create_fact`/`update_fact`/`propose_candidate_fact`/`get_fact`); **cross-field consistency** doc mismatch — declared deferred to §4.5 (needs the embedding leg), test header corrected (not faked). **DEFERRED (pending):** fact-store sharding (single global `RwLock` → shard by wiki later); fact `node_id` is a store handle not a live graph node (reconcile by materializing graph `fact` nodes or dropping the claim); engine-side query **audit-log recording sink** (the `getQueryAuditLog` accessor exists; the recording feed lands in §4.5). **NIT/comment:** stale RED headers corrected | pending (documentation gates) | pending (documentation gates) | `cargo test` 153 pass (facts 36 + graph 66 + store 50 + placeholder 1) · `build` clean · `clippy` clean · `fmt` clean |
| **§4.4 consistency enforcement** (spec §4.4) | TestWriter red: **15 red** at the stubs stage (then two test-fixture defects fixed: a `"MTI"` seed typo + a read-before-join race) → **22/22** after the adversarial fix pass | 22/22 (`tests/consistency_integration.rs`) + facts 36 + graph 66 + store 50 + placeholder 1 | **HIGH (F1):** `publish_document` held a shard write while reading `fact_store`/other shards — an AB-BA deadlock vs the fact-commit paths (fixed: uniform lock order decision **LOCK-ORDER-REF-SHARD-SIDECAR**; publish validates under read locks then takes the single shard write only for the transition; fact-commits ground under `reference_lock.read()` before `fact_store.write()`). **MEDIUM:** propagation TOCTOU — embeds created after a fact-update scan could stay `FRESH` with a stale snapshot (fixed: derive embed state from snapshot-vs-canonical); `re_sync_embed` incoherent snapshot + non-fact targets (fixed); reference target-**change** (repoint) not propagated (fixed: re-stamp from new-target liveness); propagation didn't bump `revision`/journal on referencing docs (fixed). **LOW:** report fabricated state for `None`-state edges + wrong crosslink default (fixed: derive from target liveness; crosslink-with-snapshot = embed, crosslink-without = live link `RESOLVED`). **Concurrency tests:** added publish∥fact-commit∥re-sync stress + propagation/ordering regressions | pending (documentation gates) | pending (documentation gates) | `cargo test` 175 pass (consistency 22 + facts 36 + graph 66 + store 50 + placeholder 1) · `build` clean · `clippy` clean · `fmt` clean · concurrency stress stable + deadlock-free |
| **§4.5 RAG/agent-memory retrieval** (spec §4.5–§4.6) | TestWriter red: **47 red** at the stubs stage (3 files) + 4 green guards; **initially GREEN but ADVERSARIAL-REJECTED**; after the real rework + re-audit → **58/58** | 58/58 (rag_query 34 + retrieval_stack 18 + agent_memory 6) + store 50 + graph 66 + facts 36 + consistency 22 + placeholder 1 | First adversarial gate **rejected** the initial green (real, not shape-only): HIGH-1 `ragQuery(mode:vector)` ran the LEXICAL leg behind a vector trace (fixed: real dense vector leg + provider seam + `EmbeddingUnavailable`/`VectorIndexUnavailable` reachable from `rag_query`); HIGH-2 hybrid used the lexical list twice → degenerate flat (fixed: 3 distinct graph/vector/lexical legs + exact RRF); HIGH-3 retrieval fail-states were unreachable dead code (mostly fixed; `subTaskDag`/`CompressionFailed`/`HyDEGenerationFailed` recorded as **reserved** variants via decisions SUB-TASK-DAG-VALIDATED-ONLY + RESERVED-ERRVARIANTS-DISCIPLINE, sub-task DAG parked, FS-13/14/15 lexical tension → HANDOFF); HIGH-4 multi-query/compression/hyde were silently ignored (fixed: real fan-out/merge, filter compression, HyDE routing); MEDIUM-5 cross-wiki vector leak (fixed: wiki-scope); MEDIUM-6/8/9 vacuous greens (stale-parent, profile-value, non-contending concurrency) — all de-vacuated; MEDIUM-7 fabricated `Resolved` walk default (kept, ingest-derivation reachable); MEDIUM-10 `blocked_by` on non-empty (fixed). Second re-audit: MEDIUM-A **graph-leg nondeterminism in RRF/hybrid/graph** (fixed: sort by `(DocumentId,NodeId)` before `top_k`/RRF), LOW-G `binaryCandidatePool` default 10×topK (fixed + spec-conflict test corrected), LOW-E HyDE error mapping (fixed), LOW-F HyDE test de-vacuated. **233 tests total, trio clean** | pending (documentation gates) | pending (documentation gates) | `cargo test` 233 pass (rag_query 34 + retrieval 18 + agent_memory 6 + store 50 + graph 66 + facts 36 + consistency 22 + placeholder 1) · `build` clean · `clippy` clean · `fmt` clean · determinism + concurrency stable |
