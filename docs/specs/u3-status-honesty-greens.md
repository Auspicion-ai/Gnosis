# §7.2 P2 **U3** — status honesty (the `EngineSubsystems` capability semantics) — **BLIND-GREENS SET**

- **Author-role:** blind_test_writer (documentation-only; **no `src/` file was read for
  expectations** — only public API *signatures* for compilation, plus the docs).
- **Sources (docs ONLY):** `docs/specs/p2-gnosis-server.md` — **§5.8** (subsystem-flag
  capability semantics: the flag = "this subsystem's full query-time capability is wired and
  functional for the current store"; the DEGRADED `embedding:true` false claim; the frozen
  `EngineSubsystems`; the additive signal scoped to `HealthReport` only), **§6** (the READY
  lifecycle: the boot sequence, `Ready` + flags reflecting the wired provider, the
  provider-unreachable ⇒ non-`Ready` fail-state, "the server does NOT fabricate READY"),
  **§9.5.2** (U3's five typed rows `P-IM-7`/`P-IM-8`/`P-IM-9`/`P-SM-5`/`P-SM-6` — the capability
  predicate table, the pinned `boot_wiring(...)` seam and the F16 read-time derivation, the
  adjudication notes 1–2), **§9.5.3** (the execution plan — the tag/seed pin, the
  held/broken reporting, **F11**'s lib-visible seam), **§9.5.4** (the per-row coverage notes);
  `docs/specs/engine-wire-contract.md` — **§9** (`HealthReport`, the `health(&EngineStatus)`
  projection rules, the camelCase top level, the PascalCase `state`), **§9.1** (the same
  capability semantics on the F2 side; `EngineSubsystems` **FROZEN — exactly six `bool`s**; the
  additive permission is `HealthReport`-only and **U3 adds none**; no new status surface),
  **§12 V-8/V-8.1/V-8.2** (the amended golden masks), **§13** (`health` per-fn valid/fail states
  — "any `EngineStatus` → deterministic `HealthReport`; none (pure)"); `docs/decisions.md`
  (`SUBSYSTEM-FLAG-CAPABILITY-SEMANTICS`, ACTIVE).
- **Executed by:** `tests/blind_u3_status_honesty_greens.rs` (the live `gnosis-server` bin via
  `CARGO_BIN_EXE_gnosis-server` for the HTTP rows, the engine lib's public API for the pure rows,
  and the **pinned** `boot_wiring` signature for the seam rows).
- **Blind discipline:** every expectation below quotes its clause. Where a clause does **not** pin
  an observable, that is listed in §Contract ambiguities (not blind-verifiable) rather than
  invented — and no scenario was edited to match the implementation.
- **Run:** `CARGO_HOME=/tmp/gnosis-cargo-home cargo test --test blind_u3_status_honesty_greens
  -- --test-threads=1`; the full-suite confirmation is
  `CARGO_HOME=/tmp/gnosis-cargo-home cargo test --lib --tests --no-fail-fast -- --test-threads=1`.
- **Post-greens documentation review (gate 8, 2026-09-17; docs-only — no scenario, expectation or
  ambiguity-list item changed).** This set was reconciled against the landed tree and the U3 records:
  the **13/13 PASS** run record stands (13 test functions, all async — 3 live-HTTP rows (`U3-1`/`U3-2`/`U3-12`)
  + 10 pure-lib rows (`U3-3`…`U3-11`, `U3-13`), §"Layer statement"), the layer statement matches
  `tests/blind_u3_status_honesty_greens.rs` (`:331`…`:1134`), §"Contract ambiguities" is correctly marked as
  **ambiguities, NOT pins** (the landed unit was **not** allowed to promote any of the eight items to a pin),
  and **`R-L2`'s live discharge is recorded** (2026-09-17: `state:"Ready"`, `embedding:true`, `vector:false`,
  `reranker:false`, core `true`, with `mode=vector` in the same session ⇒ 503 `vector_index_unavailable`), the
  row's home being `docs/specs/p2-gnosis-server-live-pending-battery.md` §3.5. The set's numbers agree with
  the register (§9.5.2's five rows, **127 executed ≤ 400**), the unit's suite (**604 passed / 0 failed**,
  serial) and the live battery (**9/9 rows PASS live**). Review record:
  `archive/reviews/2026-09-17-u3-status-honesty-doc-review.md` (gitignored provenance).
- **Date:** 2026-09-17.

**Reading convention.** "reachable state space" is the set a store can actually be put in by the
**non-mask** mutators (`swap_snapshot`, `set_embedding_provider`, `set_engine_state`) crossed with
the four `EngineState`s — §9.5.2's `P-IM-7` strategy (`strat:flag-truth-capability`) and §9.5.4's
coverage note both scope the corpus that way and both **exclude** "any state produced only by a
verbatim mask write". `P-IM-7`'s write-independence probe (`set_subsystems(<all-true>)` followed by
a re-read leaves the flags unchanged) is *asserted* (U3-14) while its *product* is never used as an
expectation source.

---

## Scenario table

| # | id | clause | request / observation | expectation | result |
| --- | --- | --- | --- | --- | --- |
| 1 | `U3-1` | §5.8 "GET /engine/status remains the single status surface, **always 200**"; F2 §9 (`HealthReport` shape); F2 §9.1 "no new endpoint, no additive field in U3" | live: spawn the real bin **with the provider env removed**; `GET /engine/status` | HTTP **200**; the body's `subsystems` object carries **exactly** the six keys `store`/`graph`/`lexical`/`vector`/`embedding`/`reranker`, every one a JSON **bool**; the top level is **exactly** `schemaVersion`/`idFormat`/`state`/`version`/`subsystems`/`lastError` (no additive field); `schemaVersion == 1`; `idFormat == "opaque-string-v1"`; `state` ∈ the four PascalCase values | PASS |
| 2 | `U3-2` | §6 fail-state ("a provider-unreachable boot … does NOT fabricate READY"); §9.5.2 `P-IM-8`/`P-IM-9`, adjudication note 2; V-8.2 | live: provider-**absent** boot ⇒ `GET /engine/status` | `state != "Ready"` (`∈ {Degraded, Unavailable}` — **not** the fabricated `Ready`); `embedding == false`, `vector == false`, `reranker == false`; core `store`/`graph`/`lexical` all `true` | PASS |
| 3 | `U3-3` | §9.5.2 `P-IM-7` predicate table (`vector` ⇔ "the **current derived snapshot** carries a vector index", `snapshot().vectors.is_some()`) + §9.5.4's `P-IM-7`-boundary `swap_snapshot` with/without vectors | pure: a fresh store; then `swap_snapshot(DerivedIndexes{vectors: None})`; then `swap_snapshot(DerivedIndexes{vectors: Some(VectorIndex::default())})` — **the pinned `Some(empty VectorIndex)` case**; then a populated index | `vector` is `false`/`false`/`true`/`true` respectively — i.e. **the capability is the index being *wired*, not its entry count**; `store`/`graph`/`lexical` `true` and `reranker` `false` in all four | PASS |
| 4 | `U3-4` | §9.5.2 `P-IM-7` (the four `EngineState`s × provider present/absent) + `P-IM-8` ("the flag tracks the **provider seam**, not the state") + §9.5.4's `P-IM-8`-adversarial ("assert the flag tracks the provider seam, not the state") | pure: for each `s ∈ {Ready, Starting, Degraded, Unavailable}` × `{no provider, `set_embedding_provider` wired}`: `set_engine_state(s)`, read `get_engine_status()` | `subsystems.embedding == embedding_provider().is_some()` in **all eight**; `store`/`graph`/`lexical` `true` and `reranker` `false` in all eight; `vector == snapshot().vectors.is_some()` in all eight; `state == s` | PASS |
| 5 | `U3-5` | §9.5.2 `P-IM-8` "**Symmetrically, a store whose provider *is* wired reports `embedding: true`**" + "a wired-then-unreachable provider keeps `embedding: true` and the honest signal in that case is the `state`/`last_error` pair"; §9.5.4's `P-IM-8`-adversarial (`Degraded` + `Some` ⇒ `true`) | pure: a store with a **wired** provider, `set_engine_state(Degraded)`, `last_error = Some(<the V-8.2 message>)`; read the status | the `(state, last_error)` pair carries the degradation **and** `embedding == true` — the flag is the **wired** capability, not provider reachability | PASS |
| 6 | `U3-6` | §9.5.2 `P-IM-7` (`reranker` ⇒ "**`false` in every reachable state**"; §5.8 "no reranker implementation exists anywhere in `src/`"; V-8.1 "`reranker:false` is **unconditional** in every reachable state") | pure: the whole reachable corpus of U3-3 + U3-4 (four states × provider present/absent × index present/absent), plus one `health(&…)` projection of the same status | `reranker == false` in **every** reachable state, and the projection keeps it `false` (F2 §9's verbatim mapping invents nothing) | PASS |
| 7 | `U3-7` | §9.5.2 `P-IM-9`'s pinned observable ("∀ `p ∈ BootProvider`, `snap`: `let (st, f) = boot_wiring(p, &snap)`; `st == Unavailable \| Degraded \| Ready` exactly as above") + §9.5.4's `P-IM-9`-boundary | pure: `boot_wiring(p, &snap)` for `p ∈ {Absent, Unreachable, Reachable(provider)}` × `snap ∈ {no vectors, `vectors: Some(VectorIndex::default())`}` | `Absent ⇒ state Unavailable`; `Unreachable ⇒ state Degraded`; `Reachable ⇒ state Ready`; in all six, `store`/`graph`/`lexical` `true`, `reranker` `false`, `embedding == (p is Reachable)`, `vector == snap.vectors.is_some()`; the `Unreachable` outcome is **Degraded** — never the "always `Degraded` when the provider is absent" mis-reading (adjudication note 2) | PASS |
| 8 | `U3-8` | §9.5.2 `P-IM-9` "the lib assertion is therefore **two-sided**: `boot_wiring(...)`'s returned pair **plus an independent derived read** … asserted **element-wise equal**" + "the boot applies **only the `EngineState`** … (never a `set_subsystems` write)" | pure: apply the pinned wiring to a real `Store` — `set_engine_state(st)`, `swap_snapshot(snap)`, and `set_embedding_provider` **only** in the `Reachable` case, **never** `set_subsystems` — then compare `s.get_engine_status().subsystems` to the returned vector | element-wise **equal** for all three `BootProvider` outcomes; the returned vector at U3-time is exactly `{store:true, graph:true, lexical:true, vector:false, embedding:true, reranker:false}` for `Reachable` and `{…, vector:false, embedding:false, reranker:false}` for `Absent`/`Unreachable` | PASS |
| 9 | `U3-9` | §9.5.2 `P-SM-5` ("For **any** store state, repeated `get_engine_status()` calls with no intervening mutation return **element-wise identical** `EngineStatus` values … the journal/epoch counter does not advance and the derived snapshot (its `Arc` identity and its `vectors` presence) is unchanged") | pure: in each of the four states × provider present/absent, read the status twice; capture `epoch()`, `journal_len()`, the snapshot `Arc` identity and `vectors.is_some()` before/after | `a == b` element-wise (state, `version`, every flag, `last_error`); `epoch()`/`journal_len()` identical; `Arc::ptr_eq` on the before/after snapshot; `vectors.is_some()` unchanged | PASS |
| 10 | `U3-10` | F2 §9 "Pure, deterministic function of `EngineStatus`; equal `EngineStatus` → equal `HealthReport`" + "maps `state`, `version`, `subsystems`, `last_error` **verbatim**"; F2 §13 (`health` fail-states: none (pure)); F2 §9.1 ("U3 lands **NO** additive `HealthReport` field"); V-8/V-8.1/V-8.2 shape rules | pure: `health(&status)` over all four states × representative masks (incl. an all-false mask and the contradictory `{Degraded, last_error: None}` input), serialized to JSON | `state` identical; `version` identical; each of the six flags identical; `last_error.is_some()` identical (and the same `String`); `schemaVersion == 1`; `idFormat == "opaque-string-v1"` (never derived from the input); the top-level JSON key set is **exactly** the six frozen keys; the `subsystems` JSON object is **exactly** the six flags — no additive field | PASS |
| 11 | `U3-11` | §9.5.2 `P-IM-7` "**and the projection is write-independent:** the derivation is the **read-time projection inside `get_engine_status`** (F16), so no caller-visible value depends on a written mask"; §9.5.4's `P-IM-7`-adversarial ("a deliberately injected false mask MUST NOT flip any flag") | pure: read the derived flags; write an **all-true mask** through the (test-hook) `set_subsystems`; re-read | the re-read is **element-wise unchanged** — the hook is a write-only vehicle, never the producer of the returned value | PASS |
| 12 | `U3-12` | §9.5.2 `P-SM-5` (the read mutates nothing observable) — the **live** half: §5.8/§6 ("`GET /engine/status` is the single status surface, always 200") | live: on the provider-absent boot, read `GET /engine/status` twice (the boot already polled it once) | the two bodies are **byte-identical** in `state`, `version`, `subsystems` and `lastError`; both 200 | PASS |
| 13 | `U3-13` | V-8.1 + §9.5.2 U3-note 3 + §5.8 (`vector` flips to `true` in **U5**, not U3) + `P-IM-8`'s `{Degraded, None}` instance | pure: the **U3-time** honest vectors — `{Ready, provider wired, no index}` and `{Degraded, provider absent, no index}` — asserted field by field against the V-8.1 U3-stage variant and V-8.2 | `{store:true, graph:true, lexical:true, vector:false, embedding:true, reranker:false}` and `{store:true, graph:true, lexical:true, vector:false, embedding:false, reranker:false}` — and both projections carry the V-8.2 `lastError` string **verbatim** when the input's `last_error` is that string | PASS |

**Scenario count: 13 (`U3-1`..`U3-13`).** Each row is one `#[test]` in
`tests/blind_u3_status_honesty_greens.rs`; the file therefore carries **13** test functions
(the `P-IM-7` write-independence probe is folded into `U3-11`). Attribute split: **11
`#[tokio::test]` + 2 `#[test]`** — `get_engine_status` and `boot_wiring`'s comparison leg are read
through the store's async surface, so most pure rows are async *functions*; the split that matters
is the **surface** split below (live-HTTP vs pure-lib), not the attribute.

### Layer statement (which rows run where — stated so nothing is credited twice)

- **3 live-HTTP rows** — they spawn the compiled `gnosis-server` bin on an ephemeral loopback port
  with `GNOSIS_SERVER_OLLAMA_URL`/`GNOSIS_SERVER_OLLAMA_MODEL` **removed** so the not-`Ready`
  precondition is deterministic rather than ambient: **`U3-1`, `U3-2`, `U3-12`.**
- **10 pure-lib rows** — they need no server, no network and no provider probe: every state is
  *constructed* through the documented non-mask mutators and every flag is read back from the
  store's own public API (`get_engine_status`/`snapshot`/`embedding_provider`/`swap_snapshot`/
  `set_embedding_provider`/`set_engine_state`), the pinned `boot_wiring` seam, or
  `gnosis::wire::status::health`: **`U3-3`..`U3-11`, `U3-13`.**
- **The live *battery*'s row `R-L2` is NOT covered here (layer boundary).**
  `docs/specs/p2-gnosis-server-live-pending-battery.md` §3.5 carries **`R-L2`** — the
  provider-**reachable** boot as a *live server* state (`state:"Ready"` + `embedding:true`,
  `vector:false`, `reranker:false`, core all `true` from `GET /engine/status`). §9.5.2's
  adjudication note 1 and `P-IM-9`'s observable cell both pin it **live-battery-only**: it needs a
  **controlled provider** (an Ollama-compatible endpoint answering `/api/tags`), which is a
  precondition a node test may not assume. This set therefore asserts the provider-reachable
  outcome **only** through the pinned lib seam (`boot_wiring(BootProvider::Reachable(_), …)`,
  `U3-7`/`U3-8`) and **never** claims the live `Ready` boot. A node test that spawned the bin
  against the ambient environment would either silently pass on a machine with an Ollama up or
  silently assert the wrong branch elsewhere — which is exactly why that path belongs to `R-L2`.

### The provider-reachable boot is out of a node test's reach (stated once)

`R-L2`'s precondition is a **controlled provider**, and its home is named in the register
(`P-IM-9`) as the battery's row. Two consequences for this set, both deliberate:

1. Every live row here boots with the provider configuration **removed**, so `state != "Ready"` is
   a deterministic precondition rather than an ambient accident (the same discipline as
   `tests/gnosis_server_e2e.rs::spawn_server_without_provider`).
2. The `Ready` + `embedding:true` pair is asserted **at lib level** (`U3-7`/`U3-8`) and **not**
   over HTTP. The two layers agree by construction — U3-8 asserts the seam's vector equals the
   store's own derived read after the pinned wiring — and the live layer's half is `R-L2`.

---

## Contract ambiguities (cannot be blind-verified)

These are **not** failures, and — restated so the list is not misread after the unit landed — they are
**ambiguities, NOT pins**: nothing in `docs/specs/p2-gnosis-server.md` (or in
`docs/specs/engine-wire-contract.md` §9/§9.1/§12/§13, or in this set's cited clauses) claims any of the
following is **pinned**, and this set therefore does not assert them. The unit's landing does **not**
convert them into pins: U3's verified state (five rows HELD, 13/13 here, live battery 9/9) is asserted
**only** over the halves that *are* pinned. Where an item below was an unpinned **observation** at writing
time and is now a **verified landed behavior**, that is marked as such — an observation is still not a pin.
They are the places where the documentation does not pin the
observable, so this set either does not assert it or asserts only the pinned half.

1. **`EngineStatus::version`'s *value* is not pinned.** Every clause pins only that `health`
   mirrors the input `version` **verbatim** (`P-SM-6`, F2 §9) and every V-8.x literal abbreviates
   it as `"…"`. The set asserts the mirroring and `version` being a `String`, never a literal.
   *(Still unpinned after U3: the landed value is `env!("CARGO_PKG_VERSION")`, `src/store/mod.rs:4224` —
   a producer detail no clause pins. The live rows read the value and compare it for stability only.)*
2. **The docs do *not* force the provider-absent *live boot* to take the `Unavailable` branch
   specifically** — §9.5.2's adjudication note 2 pins the *construction-state* value
   (`EngineState::Unavailable`) and the rule "`Ready` iff the provider was reachable, else
   `∈ {Degraded, Unavailable}`". `U3-2` therefore asserts the pinned **disjunction**
   (`state != "Ready"`) plus the flag vector — which `P-IM-8` makes state-independent — rather than
   inventing one of the two states. (Observed in this environment: the provider-absent boot reports
   `state:"Unavailable"`, `lastError:null` — matching the construction value. Recorded as an
   observation, not asserted as contract. **Still an ambiguity after U3** — the observation was not
   promoted to a pin, and `P-IM-9`'s `BootProvider::Absent` branch is asserted at lib level.)
3. **The `state` of a store after `set_engine_state(Starting)` or `Unavailable` carries no flag
   consequence.** Under `P-IM-7`/`P-IM-8` the flags are a projection of the snapshot + the provider
   seam only, so the set asserts exactly that and nothing about what `Starting` should imply (no
   clause names a flag effect for it).
4. **Whether a provider reachable-and-wired store also *serves* `rag_query` is not in U3's
   scope.** U3 is the flag axis; the boot's `Ready` condition is explicitly **not** tightened in
   U3 (§9.5.2's F13 note / the defects row "`Ready` does not imply every subsystem flag is
   `true`"). No scenario asserts a query outcome. *(The live battery's `R-L2` pass, 2026-09-17,
   cross-checked `mode=vector` ⇒ 503 `vector_index_unavailable` in the same session — recorded as the
   battery's own evidence, still **not** a U3 contract claim.)*
5. **The wire `Content-Type` of `GET /engine/status` is not pinned** by any clause this set
   cites, so only the status and the JSON body are asserted (the body is parsed, not
   header-checked). **This is an unpinned gap that matters to a consumer**, and it is recorded as such
   rather than promoted to a pin on the strength of the landed implementation: `docs/specs/p2-gnosis-server.md`
   §10's `/engine/status` row now carries the explicit **recorded-ambiguity** note (the header is what axum's
   `Json` responder emits, `src/bin/gnosis_server.rs:285-289`) and names its home — the transport/status unit
   that owns §5.5's rendering discipline (the same unit that owns `docs/defects.md` **P-5**), where a pin lands
   with an e2e header assertion. No U3 row was widened to cover it.
6. **`set_subsystems`' storage is not observable through the documented read** — §9.5.2's F16 pin
   makes it a write-only test hook. The set uses it **only** as `U3-11`'s adversarial write and
   never as an expectation source (F11/F16 explicitly forbid the read-back assertion).
7. **The U5-time `vector:true` live boot is out of scope.** V-8.1's `vector:true` presumes a built
   index; until U5 the honest live value is `false` (§5.8's unit split). The set asserts the U3
   stage in both the live rows and the pure vectors, and would need U5's unit to change.
   *(U3 landed without touching this: `vector` is still `false` in every live reachable state, and the
   `mode=vector` ⇒ 503 residual stands until U5.)*
8. **The `last_error` *text* is pinned for `Degraded` only.** §9.5.2's `P-SM-6`-adjacent notes and
   V-8.2 pin the store's fixed DEGRADED message verbatim (and forbid rewording it), and §9 says
   `last_error` is `Some` **exactly** when the input's is `Some` — but no clause this set cites
   pins which of the other three states carry a `lastError` on the wire. The set therefore asserts
   the **mirror** (`Some` iff the input's is `Some`) and the fixed `Degraded` string, and asserts
   only the **stability** of the live pair across repeated reads (`U3-12`) — never a `null`/non-null
   expectation for a state whose producer behavior is unpinned. *(The landed producer is explicit —
   `Some` iff `state == Degraded`, `src/store/mod.rs:4226-4230` — so `Ready`/`Starting`/`Unavailable`
   carry `null`; that is a recorded observation of the landed code, still not a clause-pinned
   contract, and the live battery's `Unavailable` read (`lastError:null`) is the same observation.)*

---

## Run result

```
$ CARGO_HOME=/tmp/gnosis-cargo-home cargo test --test blind_u3_status_honesty_greens -- --test-threads=1
running 13 tests
test u3_1_engine_status_is_200_with_exactly_the_six_flag_keys ... ok
test u3_2_provider_absent_boot_is_not_ready_and_claims_no_dead_capability ... ok
test u3_3_vector_flag_is_the_wired_index_including_some_empty ... ok
test u3_4_flags_track_the_provider_seam_across_the_four_states ... ok
test u3_5_wired_provider_keeps_embedding_true_with_the_degradation_in_state ... ok
test u3_6_reranker_is_false_in_every_reachable_state ... ok
test u3_7_boot_wiring_three_outcomes_are_exact ... ok
test u3_8_boot_wiring_vector_equals_the_stores_derived_read ... ok
test u3_9_repeated_status_read_is_side_effect_free ... ok
test u3_10_health_report_is_frozen_shape_and_verbatim_projection ... ok
test u3_11_injected_mask_does_not_produce_any_flag ... ok
test u3_12_repeated_live_status_read_is_identical ... ok
test u3_13_honest_u3_time_vectors_match_v8_1_stage_and_v8_2 ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
```

**N/N PASS: 13 passed / 0 failed (13 scenarios)** — 3 live-HTTP + 10 pure-lib. Full-suite
confirmation (serial, `--test-threads=1`, run with `GNOSIS_SERVER_OLLAMA_URL=http://127.0.0.1:11434`
**set** and an Ollama reachable, so the hermeticity of the provider-removed rows is itself
exercised):

```
$ CARGO_HOME=/tmp/gnosis-cargo-home GNOSIS_SERVER_OLLAMA_URL=http://127.0.0.1:11434 \
    cargo test --lib --tests --no-fail-fast -- --test-threads=1
(aggregate) 604 passed / 0 failed   ⇒ exit 0        (32 test binaries)
$ CARGO_HOME=/tmp/gnosis-cargo-home cargo fmt -- --check            ⇒ exit 0
$ CARGO_HOME=/tmp/gnosis-cargo-home cargo clippy --all-targets      ⇒ 0 warnings
```

The same U3 command re-run **with** `GNOSIS_SERVER_OLLAMA_URL=http://127.0.0.1:11434` set is also
**13/13** — the live rows remove the boot's provider configuration, so the not-`Ready`
precondition is deterministic either way (that ambient path is `R-L2`'s, not this set's).

**Failures: none.** No scenario was edited to match the implementation, and no assertion was
weakened. The three live rows boot with the provider env removed; they are green with an Ollama
reachable at `http://127.0.0.1:11434` **and** with it absent, which is the hermeticity claim.

---

## What this set does NOT cover (out of U3 scope)

- **U5** — the boot vector-index build (and therefore V-8.1's `vector:true` live value).
- **`R-L2`** — the provider-**reachable** live-server boot (the live battery's row, §3.5 above). **DISCHARGED
  LIVE 2026-09-17** by the U3 live battery (9/9 rows PASS live: `state:"Ready"`, `embedding:true`,
  `vector:false`, `reranker:false`, core all `true`, with `mode=vector` in the same session ⇒ 503
  `vector_index_unavailable`) — the row is recorded in
  `docs/specs/p2-gnosis-server-live-pending-battery.md` §3.5, not in this scenario set (this set's layer
  boundary is unchanged: the provider-reachable outcome is asserted here **only** through the pinned lib seam,
  `U3-7`/`U3-8`).
- **§9.5.3's property layer** (`tests/props_gnosis_server.rs`, tags `U3PIM7`/`U3PIM8`/`U3PIM9`/
  `U3PSM5`/`U3PSM6`) — this is the blind **scenario** set; the register's generated property layer
  is a separate artifact with its own seed/tag/budget discipline.
- **U4** — the change cursor, `GET /changes` and the paged reads.
