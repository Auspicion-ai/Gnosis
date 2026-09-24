# Gnosis → Suite / Foundation — Issue Handoff

This is the issue-handoff document: gaps discovered while building Gnosis that
are owned by another project (the Auspicion Suite top-level repo, the
provident-ssr foundation, or another suite tool). Gnosis never patches another
project's source.

## OPEN handoff items

**INBOUND from the Astrographer shell (2026-09-16).** The shell (a consumer of `gnosis-server`) has filed a
consolidated feature-request set: **`<Astrographer repo>/docs/feature-requests/gnosis-engine-feature-requests.md`
(GR-1..GR-9)** — each with live repro evidence, a proposed wire shape and testable acceptance criteria.
Highest priority: **GR-1** (`POST /rag/query` must accept the F2 envelope — the shell currently gets a masked
`text/plain` 400), **GR-2** (the POST query route ignores `mode`/`topK` while the SSE route honours both),
**GR-3** (the vector index is never built for the server's store, so `mode=vector` cannot work), **GR-4**
(a revisioned bulk projection-snapshot read route) and **GR-5** (a store-change notification route with a
monotonic revision) — GR-4/GR-5 are what would let the engine own the store without the consumer calling the
store interface. Parked destination items: GR-6 (bulk markdown ingestion: atomicity + cap + progress/cancel +
document path segments), GR-7 (server-side persistence), GR-8 (enrichment/traversal routes for the parked
F4-LLM integration), GR-9 (the machine-caller authority contract — note the RBAC row above re-scoped engine
enforcement to the SHELL, so GR-9 asks for the documented caller contract only, not new enforcement).

**GATE-1 RECORD (2026-09-16).** The proposal-review gate has now run its four read-only passes
(validity ∥ critique → architecture review → change-analysis) over this inbound set, all
`file:line`-grounded against the working tree: `docs/specs/gnosis-gr-inbound-review.md`. Verdict:
**PROCEED-WITH-AMENDMENTS on a re-shaped subset (U0–U3 now; U4/U5 gated)**, with **GR-6/GR-7/GR-8 parked
behind named triggers** and **GR-9 refused as an engine deliverable** (the C5/SHELL answer below already
covers it — do NOT author an engine authority contract). **UPDATE (2026-09-16):** the go-ahead was given —
**U0 is LANDED** (this record, the tracker-truth pass, and the JS cleanup) and **U1 (the
P2/F2 contract amendment) is now LANDED too** (authored by the SpecWriter on the user's "clean the JS files and
then proceed" instruction; its spec-gate reviewer loop returned **EMPTY**) — the engine-side pins are in
`docs/specs/p2-gnosis-server.md` §U1/§5.3–§5.9 and `docs/specs/engine-wire-contract.md`
§4.5–§4.7/§9.1/§16, and the upstream reconcile ask below (this file's addendum) is the live one. **The spec
gate is therefore OPEN**, while **U2/U3 are AUTHORIZED (2026-09-16 — their typed registers are authored in
`docs/specs/p2-gnosis-server.md` §9.5.1/§9.5.2, so their *code* is what remains owed) and U4/U5 remain HELD**
pending the user's next go-ahead. **UPDATE (2026-09-17, proofread pass): U2's code has LANDED
REALIZED-GREEN** (`cargo test` 582 / 0, its property layer 400 cases HELD, the blind set
`tests/blind_u2_query_post_greens.rs` 24/24, the live battery's `R-L1` PASSED live — `docs/specs/p2-gnosis-server.md`
§U1), so "the code is owed" now applies to **U3 only**; **U4/U5 remain HELD/NOT authorized**. **SUPERSEDED
AGAIN (2026-09-17, the U3 documentation pass): U3's code has LANDED-GREEN** (`cargo test` 604 / 0 serial, its
five §9.5.2 rows HELD with 127 executed ≤ 400, the blind set 13/13, the live battery 9/9 incl. `R-L2` — this
file's §"U3 documentation pass (2026-09-17)" block), so **no U2/U3 code is owed any longer**; **U4/U5 are the
only units still HELD/NOT authorized**. The two gate decisions
(`GNOSIS-CHANGE-CURSOR`, `SUBSYSTEM-FLAG-CAPABILITY-SEMANTICS`) are **ACTIVE** rows in
`docs/decisions.md`; no engine code lands before **U2/U3** (U2/U3 are the recommended next scope).
**SUPERSEDED (2026-09-17, post-greens doc-review): U2's code has LANDED REALIZED-GREEN** (`cargo test` 582/0,
all 7 §9.5.1 rows HELD with the layer's 400 executed cases, the blind set 24/24, the live row `R-L1` PASSED
live — `docs/next-steps.md`'s U2 DONE row), so **no engine code remains owed before U3**; **U3 is AUTHORIZED
(its register is authored in `p2` §9.5.2) and its code is owed** — **and that code has since LANDED-GREEN
(2026-09-17: `cargo test` 604/0, the five rows HELD with 127 executed, blind set 13/13, live battery 9/9 incl.
`R-L2`)**, so **no engine code remains owed before U4/U5**; **U4/U5 remain HELD/NOT authorized**. The
historical clause read "no engine code lands before **U2/U3** (U2/U3 are the recommended next scope)".


**INBOUND from the Astrographer shell (2026-09-21) — the re-filed P2 prerequisite set `GRQ-1..GRQ-11`.** The shell
has consolidated its engine asks into one set: **`<Astrographer repo>/docs/feature-requests/gnosis-engine-prerequisites.md`**
(11 requests in 5 groups; its own census: 7 `OWED` / 4 `REQUESTED`; P0/P1/P2 = 3/6/2; + 2 recorded disagreements
and 1 non-engine row). **The proposal-review gate has run all four read-only stages over it** (validity ∥ critique →
architecture review → change-analysis), every engine-side claim `file:line`-grounded against this tree; the
engine-side record — and **this repo's index row for the set** (no `GRQ` token existed anywhere under `docs/` before
it) — is **`docs/specs/gnosis-grq-inbound-review.md`**, with its change-analysis stage in that file's §15.
**Verdict: PROCEED-WITH-AMENDMENTS — the review PASSES and NOTHING is authorized** (11 gate conditions, §15.5); the
go-ahead is **outstanding** (§13's five questions). **Verdict counts:** 11 mapped — **VALID-AMENDED 7**
(GRQ-1/2/3/5/6/8/9) · **DUPLICATE-ALREADY-RULED 2** (GRQ-4, GRQ-10) · **OVERSTATED 2** (GRQ-7, GRQ-11) ·
**UNSUPPORTED 0**. **7 of the 11 are re-filings of already-adjudicated items** (GRQ-1≈GR-7, GRQ-4≈GR-4,
GRQ-5≈GR-6a, GRQ-6≈GR-6a/6b, GRQ-7≈GR-2's defect, GRQ-10≈GR-5, GRQ-11≈GR-4/5's token question) — **no GRQ supplies
new evidence for disturbing a standing ruling**. **What the set establishes engine-side:** GRQ-1/GRQ-2/GRQ-11 +
§6(c) collapse into the (unauthorized) **durability design unit**; GRQ-3 is a **rider inside that unit** (a
**DERIVED** `durability` field, additive on `HealthReport` only, never on the frozen `EngineSubsystems`, with its
V-8 golden edit in the same unit); GRQ-5/GRQ-6 stay **PARKED on GR-6a's unchanged trigger** (`{files:[paths]}` stays
REJECTED); GRQ-10 and GRQ-4's bulk-read half fold into **U4** (HELD; additionally needs `SHELL-2`); **GRQ-7 is
already fixed** (U2's landed shared decoder) and **GRQ-9's decode-body half already landed** (U2); GRQ-8 reduces to
a **documented refusal**. **Citation discipline:** the set's engine-side citations are **second-hand by its own
header** (authored read-only in the consumer repo — "the sibling repo is not readable from this role"), so the
record separates **consumer-tree citations** (legitimate cross-repo pointers, unverifiable from here) from
**engine-state claims** (13 re-verified against this tree, **3 falsified**). **Two asks this set does not file,
recorded here as asks of the CONSUMER/USER — not of this repo (see the two rows below):** the **durable-authority
fork**, and GRQ-7's **stale live repro** (its defect predates U2).

| Item | Owner | Detail |
| --- | --- | --- |
| **§8 RBAC `caller` threading — the `RagStore` mutating methods take no `caller` param** | Astrographer shell / the CRUD roadmap (`docs/specs/unblock-gnosis-remaining-endpoints.md` §8, decision `GNOSIS-RBAC-EDIT-ENFORCEMENT`) | **RESOLVED as re-scoped to the shell (2026-09-10, proposal gate — `docs/specs/rbac-caller-review.md`).** The engine-side RBAC enforcement is deliberately NOT implemented: the engine has no authority-mapping source, and the C5 boundary classification + the pure-backend framing place the authorization gate at the SHELL. The shell's A2 caller-side deny (`handleGnosisTool` `resolveCallerCredential` → `Error('<tool>: caller has no edit authority')`, fail-closed, before any proxy call) **is** the enforcement. The P2 server threads the `caller` through the request-decode layer only (validates presence on the 7 mutating methods → 400 if missing, per P1a §10; does NOT pass it to the engine). Decision `GNOSIS-RBAC-EDIT-ENFORCEMENT` amended to align with C5 (RBAC → SHELL). Residual risk (accepted): the engine's mutating methods are callable by any process that can reach the loopback port directly; acceptable under the loopback-only + local-shell-only trust model. Trigger condition for revisiting: a future non-shell direct client → mitigation is a caller-aware decorator at the access boundary (Option B), NOT a trait change. Recorded in `docs/defects.md` (CLOSED). |
| **Spec §4.4.3 publish-gate crosslink ambiguity** | Auspicion Suite top-level `docs/specs/gnosis.md` | §4.4.3's publish-gate text names only "any `BROKEN` link" and "any `STALE` embed"; it does **not** enumerate `crosslink`, unlike §4.4.5's delete gate ("link/embed/crosslink") and §4.2.2 ("crosslinks resolved the same way as link/embed"). Gnosis's store + tests now **gate crosslink `Broken`/`Stale` on publish** (the §4.2.2 reading). Please reconcile the upstream contract to state the crosslink publish-gate behavior explicitly so the contract and the tests agree. |
| **Spec §4.2.7.2 / §4.2.9.1 graph-owned realization** | Auspicion Suite top-level `docs/specs/gnosis.md` | The §4.2 adversarial gate (decision **GRAPH-OWNS-RELATION-AND-MERGE**) realized the triple/merge/resolution model as **graph-owned data**: the `relation` edge carries `relationType`; `merge_facts` persists the merged fact; `resolve_entities` writes a durable alias→canonical mapping. The contract should be **reconciled to match this** (it currently implies edge-property `relationType` and a persisted merged-fact record, and `resolveEntities`'s `ResolutionResult` shape treats it as a plan/descriptor while the behavior is effectful). |
| **Spec FS-13/14/15 want index-not-built errors in hybrid, but the engine degrades legs gracefully (FS-15 design tension)** | Auspicion Suite top-level `docs/specs/gnosis.md` | The BM25 "index" is currently a **live shard scan**, so "index not built" is reached only when the queried wiki has **no documents** — effectively **unreachable on the `ragQuery` surface** (a wiki with docs always has a lexical leg). `LexicalIndexUnavailable` is only surfaced via the `bm25_search` API. The spec's FS-13 (index not built on hybrid) / FS-14 (vector) / FS-15 (BM25) rows want index-not-built errors in hybrid, while §4.5.1 also says legs **degrade gracefully on a failing leg**. Please **reconcile**: pin whether an unbuilt/unavailable lexical index on the hybrid `ragQuery` surface is an error or a degraded-to-empty leg (the engine currently degrades gracefully). Recorded as a spec-reconcile / HANDOFF item rather than forcing an unreachable error on `ragQuery`. **UPDATE (2026-09-16):** the gate-1 review of the inbound GR-1..GR-9 set (`docs/specs/gnosis-gr-inbound-review.md`) resolves the reconcile direction as **explicit-leg-only errors** (`mode=vector` with no index → `VectorIndexUnavailable` → **503**) while the **hybrid fusion contract keeps degrading a failing leg to empty** (recommended option A of that record's open question 3, **now ANSWERED — the user's "clean the JS files and then proceed" instruction accepted the elaboration as the ruling; see Appendix B of that record**). **Scheduled for U1's explicit-leg-only reconcile — U1 IS AUTHORIZED (2026-09-16), so this reconcile is now a scheduled contract amendment; the code units that touch the query/flag paths (U2/U3) remain HELD** — no spec or code change in this pass. **U1 LANDED (2026-09-16); the engine-side pins are in the specs cited above.** **SUPERSEDED (2026-09-16) as to the code units: U2 and U3 ARE AUTHORIZED — their typed registers are authored in `docs/specs/p2-gnosis-server.md` §9.5.1/§9.5.2 (the code is owed); U4/U5 remain HELD.** **SUPERSEDED (engine-side landed by U1, 2026-09-16):** everything before this marker is historical (including the "scheduled for U1" clause); U1 **landed the engine-side reconcile** as `docs/specs/engine-wire-contract.md` §16 and `docs/specs/p2-gnosis-server.md` §5.9 (explicit-leg errors / fusion degrades, including the extended narrowing of **§4.6.1's `ragQuery` throw column and FS-13's `hyde: true` clause**), and the **upstream ask now lives in the new row below** ("FS-3 vs FS-13/14/15 … + the change cursor / `GET /changes` / subsystem-flag capability semantics"). This row is **kept (NOT deleted)** as the historical record of the open reconcile; the ask is the row below, not this one. |
| **Spec F3/F4/F5 open-question §-refs point to non-existent §4.5.6/§4.5.7/§4.5.8** | Auspicion Suite top-level `docs/specs/gnosis.md` | §1 and §3 cite the parked F3 adaptive-RAG routing to **§4.5.6** (§7.4), F4 community summaries to **§4.5.7** (§7.5), and F5 dynamic graph query to **§4.5.8** (§7.6). The only "Parked retrieval surfaces" subsection in §4.5 is **§4.5.5** ("Parked retrieval surfaces (noted, not in scope)..."). There is no §4.5.6/§4.5.7/§4.5.8 body — the refs are stale section numbers. Please **reconcile** the F3/F4/F5 refs to §4.5.5 (the parked surfaces that actually cover all three). |
| **Spec §4.4.3 publish-gate atomicity reconciliation** | Auspicion Suite top-level `docs/specs/gnosis.md` | The engine's `publish_document` does **not** re-run the target-liveness check under the shard write (it validates under read locks), so a concurrent target mutation between validation and the `DRAFT→PUBLISHED` transition can pass with a later-`stale` reference; please reconcile whether the contract requires the atomic re-check or accepts the read-lock-validated publish (engine currently accepts the non-atomic publish; recorded as an engine-internal non-goal in `docs/defects.md`). |
| **Spec §4.3.1 fact-location wording vs `factKey` canonical identity** | Auspicion Suite top-level `docs/specs/gnosis.md` | §4.3.1's "the fact's location in the store" implies `(documentId, nodeId)` is a live graph node, but the engine's stored `node_id` is a `fact-{fact_key}` handle and §4.5.4 makes `factKey` the canonical retrieval identity. Please **reconcile** §4.3.1 to state that a fact's canonical identity is `factKey`, and `(documentId, nodeId)` is a derived conventional handle that corresponds to a live graph `fact` node only when the caller authored one. (Engine will **not** materialize fact nodes on commit — see `docs/defects.md` + `docs/pending.md`.) |
| **Spec §4.3.2a.1 cross-field consistency step** | Auspicion Suite top-level `docs/specs/gnosis.md` | §4.3.2a.1 lists "cross-field consistency" as a gate check, but the §4.3 runtime gate enforces schema/grounding/dedup only (a semantic cross-field check would need an LLM/embedding judgment on the commit path, breaking the deterministic fail-closed framing). Please **reconcile** the step to mark cross-field as an aspirational/offline check, not a runtime §4.3 branch (recorded as a deliberate non-goal — see `docs/defects.md` + `docs/pending.md`). |
| **Spec §4.1.1/§4.1.2 "UUID v4" wording vs monotonic engine ids** | Auspicion Suite top-level `docs/specs/gnosis.md` | The engine assigns `documentId`/`wikiId` as **monotonic counter-prefix ids** (`doc-{N}`/`wiki-{N}`), which satisfy the spec's operative invariant (stable, globally-unique, never-reused, distinct-on-recreate — pinned by PBT register P-IM-1/P-TP-3). The landed F2 wire contract treats ids as opaque strings and does not require UUID v4. Please **reconcile** §4.1.1/§4.1.2's "UUID v4" wording to "stable, globally-unique, never-reused" (monotonic counter prefix is a valid realization). Decision `ID-SCHEME-RECONCILE-MONOTONIC`. |
| **Spec RBAC enforcement — the authorization gate is a SHELL concern, not the engine** | Auspicion Suite top-level `docs/specs/gnosis.md` | The canonical contract has **no RBAC/caller/authority-enforcement section** today. The proposal gate (`docs/specs/rbac-caller-review.md`, 2026-09-10) resolved that the **RBAC edit-authority enforcement lives in the SHELL, not the engine**: the engine's `RagStore` mutating methods take no `caller` param and hold no authority mapping; the shell's A2 caller-side deny is the enforcement; the P2 server threads the `caller` through the request-decode layer only. Please **reconcile** the contract to state that the authorization gate at the access boundary is a SHELL concern (per the C5 boundary classification), so a future contract change does not re-introduce engine-side RBAC. Decision `GNOSIS-RBAC-EDIT-ENFORCEMENT` (amended to align with C5). **UPDATE (2026-09-16):** the inbound set's **GR-9 asked for an engine-side caller/authority contract**; the gate-1 review (`docs/specs/gnosis-gr-inbound-review.md`) **refuted it as an engine gap** — the observed `caller has no edit authority` is **shell-side fail-closed** from an empty authority mapping, the engine has **no** denial code and **discards `caller` after the decode layer** (`src/bin/gnosis_server.rs:121-201`, landed post-U2 — the
engine's decode/dispatch path; the pre-U2 citation this row carries was `:73-123`), and its `RagStore` mutating methods take no `caller` param. **The answer is this row's C5/SHELL ruling — there is no engine deliverable, and no engine authority contract may be authored.** |
| **GR-6 — bulk markdown ingestion (atomicity / cap / progress / cancel) + document `path` segments** | **this repo (Gnosis engine), PARKED with triggers — 2026-09-16, gate-1 review (`docs/specs/gnosis-gr-inbound-review.md`); go-ahead = U0 + U1 landed (2026-09-16); spec gate OPEN; U2 LANDED (2026-09-17); **U3 LANDED-GREEN (2026-09-17)**; U4/U5 HELD** | **RE-FILED by `GRQ-5` + `GRQ-6` (2026-09-21) — this row's ruling is UNCHANGED (PARKED; `{files:[paths]}` REJECTED); see `docs/specs/gnosis-grq-inbound-review.md` §2.1/§7(iii)/§11.** Inbound request from the Astrographer shell. **PARKED**, and the `{files:[paths]}` variant is **REJECTED**: ingestion is genuinely absent (no parser, no route, no batch primitive, and `Document` carries no path field), but the requested contract is **self-contradictory** ("the whole corpus commits as one journal entry" vs "chunked commits or an SSE progress stream"), and `{files:[paths]}` would grant the engine a **filesystem-read capability** on a loopback-only server whose accepted residual risk is "any process that can reach the loopback port may call the mutating methods". The `path`/segment half collides with the **FROZEN** P1a `Document` body (`docs/specs/p1a-document-crud-wire.md:253-256`, `P-IM-2`). **Trigger:** an accepted spec resolving **one-commit-vs-chunked** + a **cap** + a **§11 fail-state allocation**, plus a **P1a wire-version decision** for `path`, plus an **explicit user decision** on the filesystem capability. Recorded in `docs/pending.md`. |
| **GR-7 — server-side persistence for the engine store** | **this repo (Gnosis engine), PARKED with trigger — 2026-09-16, gate-1 review (`docs/specs/gnosis-gr-inbound-review.md`); go-ahead = U0 + U1 landed (2026-09-16); spec gate OPEN; U2 LANDED (2026-09-17); **U3 LANDED-GREEN (2026-09-17)**; U4/U5 HELD** | **RE-FILED by `GRQ-1` + `GRQ-2` + `GRQ-11` (2026-09-21) — DIRECTION-ONLY / NOT ACTIVE stands, the 3-conjunct trigger is UNCHANGED (this repo's text governs), and the durable-authority fork is now recorded as an upstream ask in the row above; see `docs/specs/gnosis-grq-inbound-review.md` §2.1/§7(i)/§11.** Inbound request from the Astrographer shell. **PARKED.** The store is genuinely in-memory (`let store = Arc::new(Store::new())`, only `--port`), and persistence+revisioning **are** engine-owned work (`docs/specs/gnosis.md:82,190`; boundary A1) — but the request **raises the decisive questions it does not answer** ("what a revision means across a restore", crash-mid-commit), would re-open `SHARDED-RWLOCK-STORE`/`IMMUTABLE-DERIVED-SNAPSHOT`, and collides with the consumer's own parked `SINGLE-WRITER-STORE` (O-8) amendment. The request's premise that "the store trait's persistence seam is a trait with no disk implementation" is a **misstatement** — there is **no persistence abstraction in the crate at all**. **Trigger:** the consumer's **O-8 amendment recorded** + a **durability design** (format, atomic commit, recovery, store-scope revision semantics) + a scheduled consumer unit. Recorded in `docs/pending.md`. **UPDATE (2026-09-16, the user's gate answer to open question 4 = YES):** the engine **will** eventually own a durable corpus — a durability **design** unit is now on the roadmap (`docs/decisions.md` `ENGINE-DURABLE-CORPUS-DIRECTION`, **direction only / NOT ACTIVE**; `docs/pending.md` GR-7 row). It is **not authorized to start** (the go-ahead covered **U0 + U1**, both now **landed** — U1 on 2026-09-16, its spec-gate review EMPTY — with **U2/U3 AUTHORIZED (registers authored); U4/U5 HELD**; see `docs/specs/gnosis-gr-inbound-review.md`) and stays gated on the consumer's `SINGLE-WRITER-STORE`/O-8 amendment. |
| **GR-8 — enrichment/traversal routes + a text-generation seam** | **this repo (Gnosis engine), PARKED with trigger — 2026-09-16, gate-1 review (`docs/specs/gnosis-gr-inbound-review.md`); go-ahead = U0 + U1 landed (2026-09-16); spec gate OPEN; U2 LANDED (2026-09-17); **U3 LANDED-GREEN (2026-09-17)**; U4/U5 HELD** | Inbound request from the Astrographer shell. **PARKED.** The routes are genuinely absent (`declare_community`/`update_community_summary`/`resolve_entities`/`merge_facts` are implemented but **unrouted**), but the **text-generation seam is already ruled OUT** (`docs/specs/f4-llm-enrichment-integration.md:139-149`; `docs/pending.md` F4-LLM = SPECULATIVE — the engine adds no text-generation provider), and under §4.2.8 / `RESOLVE-ENTITIES-AUTHORITATIVE-OVERWRITE` an LLM host as caller would be **authoritative** and could silently supersede a human declaration while `AUTHORSHIP-SOURCE-PROPERTY` (the only mechanism recording who won) is **unlanded**. **Trigger:** `AUTHORSHIP-SOURCE-PROPERTY` landed **code-bearing** + an accepted text-gen-seam design + a consumer freeze (F4-LLM un-parked). Recorded in `docs/pending.md`. |
| **FS-3 vs FS-13/14/15 reconciled engine-side (explicit-leg errors / fusion degrades) + the `POST /rag/query` request/response body (envelope-strict, bare-`RagResult`) — the canonical contract should state both** | Auspicion Suite top-level `docs/specs/gnosis.md` (owner: the canonical contract; engine-side pin landed by **U1**, 2026-09-16) | **U1 has now pinned the engine-side resolution** (contract-only; the code-bearing units U2–U5 remain **HELD**) — ***SUPERSEDED (2026-09-16) as to U2/U3: U2 and U3 ARE AUTHORIZED (their typed registers are authored in `docs/specs/p2-gnosis-server.md` §9.5.1/§9.5.2), so only their *code* is owed; U4/U5 remain HELD.** The historical clause read "the code-bearing units U2–U5 remain **HELD**", written before the user authorized U2/U3*: (a) **FS-3 vs FS-13/14/15** — the engine reads FS-13/14/15 as the **explicit-leg** case only: a request that names a leg (`mode=vector`) whose leg is unavailable (no vector index built) ⇒ `VectorIndexUnavailable` ⇒ **503** (with an index built but no provider wired ⇒ `EmbeddingUnavailable` ⇒ 503; the index check precedes the provider check, `src/store/mod.rs:4411-4417`), while a **fusion** request (`mode=hybrid`, and `mode=flat`) **degrades a failing/absent leg to empty** and succeeds (§4.5.1's graceful degradation, `src/store/mod.rs:4449-4454`, `:4489-4513`); FS-15 is effectively **unreachable on the `ragQuery` surface** because the lexical "index" is a live shard scan (`LexicalIndexUnavailable` surfaces only via the `bm25_search` API), and invalid **option values** stay FS-3 (400 `validation_error`) — a different condition from an unbuilt index. Pinned in `docs/specs/engine-wire-contract.md` §16 and `docs/specs/p2-gnosis-server.md` §5.9. **Please state this in the canonical contract** (FS-13/14/15 and §4.5.1 wording) so the consumer docs, the engine specs and the shell agree; this supersedes the "scheduled for U1" scheduling clause of the FS-13/14/15 row above (that row itself is left untouched). (b) **The `POST /rag/query` wire** — the request body is **envelope-strict** (`{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{…}}`; a **bare body is rejected**, with no back-compat tolerance — GR-1's tolerance request was **REFUSED**), the payload is the camelCase `ragQuery` options object (`query`, `mode`, `topK`, `wikiId`, `filters`, `maxHops`, `expand`, `maxParentContext`, `multiQuery`, `compression`, `hyde`, `binaryFirstPass`, `binaryCandidatePool`, `subTaskDag`; **unknown/extra keys tolerated**; absent `mode` ⇒ `flat`; an unrecognized `mode` ⇒ `ValidationError` ⇒ **400 `validation_error`**, case-insensitive over the four modes, never a silent default and never a decode-level 422), and the **response payload is the bare `RagResult` body** — **not** the SSE `{"type":"result","result":…}` chunk wrapper. A request-decode failure returns the NEW-2 **transport-level** JSON body `{"code","message"}` (400/422, never 502, never a new §11 row). Pinned in `docs/specs/engine-wire-contract.md` §4.5/§7.1 and `docs/specs/p2-gnosis-server.md` §5.3–§5.5. **Please reconcile the canonical §4.6.1 signatures + the consumer's client spec to this shape** so the upstream/consumer docs agree with the engine's pinned wire. |

| **The durable-authority fork — the question the inbound `GRQ` set does not file** | **Astrographer shell / the user (a consumer-side ruling; the engine's side is recorded in `docs/decisions.md` `ENGINE-DURABLE-CORPUS-DIRECTION` + `docs/pending.md`'s GR-7 row)** | The `GRQ-1..GRQ-11` set assumes that `GN-1` ("engine owns document CRUD") implies **the engine owns the bytes on disk**, and files a durable store (`GRQ-1`), an ingest route (`GRQ-5`), a record-copy route (`GRQ-6`), a coherence marker (`GRQ-4`) and a change feed (`GRQ-10`) on that assumption. **It never asks the fork:** *(A)* the engine becomes the durable **authority** (then `GRQ-1/5/6` are required, the shell becomes a consumer, and the consumer's **`SINGLE-WRITER-STORE`/O-8 amendment is a precondition** — a durable engine store while the shell still holds the authoritative JSON store is a **two-authority window**), or *(B)* the shell **keeps** the authoritative store and the engine stays a rebuildable session store with an explicit **export/import** seam — in which case **`GRQ-1/GRQ-5/GRQ-6` are unnecessary as filed**, `GRQ-3`'s field means "cache" not "durable", and `ENGINE-DURABLE-CORPUS-DIRECTION` (plus this repo's answer to the 2026-09-16 open question 4) must be **reversed**. **This repo's record:** `docs/specs/gnosis-grq-inbound-review.md` §7(i) + §13 Q1 (the gate recommends **(A), coupled to the O-8 amendment, not schedulable before it**). **The engine does not adjudicate this** — it is the consumer's/user's ruling, and until it is recorded **no durability work may start** (the gate's condition 2; R-1's re-placement cannot be written either). |
| **`GRQ-1..GRQ-11` — the index row for the inbound P2 prerequisite set (2026-09-21)** | **this repo (Gnosis engine) — a GATE RECORD, not a patch; the set's own items remain the consumer's asks** | **STATUS (2026-09-22, the go-ahead — record §14 POST-RECORD UPDATE 1): the fork is answered **(A) the engine becomes the durable authority**; **U5 IS AUTHORIZED** (Q3=B); the **durability design unit (D-D1) + the GRQ-3 rider stay NOT authorized** (Q2=B — they wait on the consumer's `SINGLE-WRITER-STORE`/O-8 amendment being recorded); **U4 remains HELD** (needs `SHELL-2`); the three docs-only asks are **accepted** (Q4) and the §6(c) marker is **refused** (Q5).** The engine-side index row the set's own filing convention points at (the consumer's `docs/HANDOFF.md` row `GNOSIS-ENGINE-PREREQUISITES` is consumer-side). **Where each request landed in this repo:** GRQ-1/2/11 + §6(c) → the **durability design unit** (scope only; **not authorized**; §8 of the record); GRQ-3 → a **rider inside that unit**; GRQ-4 → **REFUSED standing** (cited, not re-opened: `docs/decisions.md` `GNOSIS-CHANGE-CURSOR`, `docs/pending.md` GR-4's row); GRQ-5/6 → **PARKED on GR-6a/6b's unchanged trigger** (`docs/pending.md`; `{files:[paths]}` stays REJECTED); GRQ-7 → **already fixed (U2)** — the engine-side re-run of its four-mode repro is owed **consumer-side**, not by a unit here; GRQ-8 → a **documented refusal** (translation duty = the consumer's); GRQ-9 → a **docs appendix** (its decode-body half already landed) + the engine-side stale-RBAC-clause reconcile (filed in `docs/defects.md` §OPEN); GRQ-10 → **U4** (HELD). **No new `docs/defects.md` row is filed for any GRQ** (they are handoffs), **no ACTIVE decision row is added**, and **no canonical `docs/specs/gnosis.md` clause is edited here** — the §13 reconciles (persistence-home wording; the `durability` field's semantics) go upstream as asks once the fork is answered. |

### U1 — upstream reconcile addendum (2026-09-16; extends the row above it)

**Why this block exists.** The row above it ("FS-3 vs FS-13/14/15 reconciled engine-side …") was written
at U1's **first** pass. The spec-gate reviewer's remand found that (a) its ask names only FS-13/14/15 +
§4.5.1 + the query body, while U1 also lands work the canonical contract has never seen; and (b) the
U1-first-pass text itself was wider than the canonical sentences it names. This block is the **current**
ask (the row above stays as written; its "Please state this in the canonical contract" clause is
**subsumed here**), and it is the row that the **older FS-13/14/15 row's SUPERSEDED marker** points at.
**(REMAND-2, N3 — row 51's exact status, stated so it is not read two ways.)** Row 51 (whose own text
reads "**U1 has now pinned** the engine-side resolution …", i.e. it carries **post-U1** content despite
being the pre-remand pass's row) is **superseded-by-this-addendum for its canonical-ask half**: its
"Please state this in the canonical contract" clause names only "FS-13/14/15 and §4.5.1 wording" — a
**strict subset** of this addendum's six items — so the **six-item list below is the current, complete
ask**, and row 51's narrower version is **not** a live second ask to satisfy. Row 51 is **not deleted**
and its **query-body half lives on**: the request/response shape (envelope-strict + bare `RagResult`) is
stated in row 51's part (b) — **not** in this addendum, and it is **not** one of the six items below —
just as the closing paragraph records.

**The upstream ask — the canonical `docs/specs/gnosis.md` should state all of the following:**

1. **FS-13/14/15 + §4.5.1** — the explicit-leg-only reading (a leg named explicitly errors; a fused leg
   degrades to empty), as pinned in `docs/specs/engine-wire-contract.md` §16 / `p2` §5.9.
2. **§4.6.1's `ragQuery` throw column (`docs/specs/gnosis.md:872`), INCLUDING its `hyde` clause** — the column
   carries no mode qualifier ("…and a vector/hybrid/**hyde** leg requires it"), so it must be reconciled
   to the `mode=vector`-only reading: with `mode=hybrid` **and `hyde: true`**, a
   missing/unreachable provider **degrades the leg to empty (200)** — `hyde` names *what* to embed, not
   a leg to fail on — and `EmbeddingUnavailable` stays reachable only via `mode=vector` with the index
   built. This is required by §6's fail-state completeness rule (`docs/specs/gnosis.md:1052-1055`),
   which makes §4.6.1 an authority table.
3. **FS-3's closed catalogue (`docs/specs/gnosis.md:1027`) — add invalid `expand` (or drop the engine-side pin).**
   The catalogue enumerates invalid `mode` and invalid `compression` but **not** `expand`, while
   `p2` §5.3/§5.4 and F2 §13 pin `expand`'s unrecognized token as **400 `validation_error`**. That 400
   is a **new engine-side state created by U1**, not a canonical restatement: either the canonical
   contract adds it or the engine drops the pin. (The **case-insensitive** matching U1 also pins for the
   three token families is a benign superset of the canonical lowercase tokens and needs no reconcile.)
4. **§4.5.2's `filters` shape (`docs/specs/gnosis.md:692-694`)** — confirm it as the **request** shape and state
   that the store's serde casing (`node_kind`, tuple `target`, PascalCase unit values) is the
   **response/audit** shape; U1 pins the canonical request shape and U2's decoder maps it (a
   pass-through to `QueryAuditFilters` would silently yield an all-`None` filter).
5. **The change cursor + `GET /changes`** — U1 introduces a **new wire-visible surface** the canonical
   contract does not mention: the opaque change cursor (`docs/specs/engine-wire-contract.md` §4.6; the
   engine-side **accessor is U4's**, decision `GNOSIS-CHANGE-CURSOR`) and the `GET /changes` SSE change
   feed (§4.7; route is U4's), with its staleness/reconnect contract and its **no request-side
   as-of-N/point-in-time** rule (a paged read's `cursor` field is a **resume token** only). The
   canonical contract should acknowledge the surface (and the refused `GET /snapshot?revision=…` /
   `stale_revision` alternative) so the consumer docs, the engine specs and the shell agree.
6. **The redefinition of the `subsystems` flag meaning** — U1 pins an `EngineSubsystems` flag as
   "this subsystem's **full query-time capability is wired and functional for the current store**" (not
   "a provider exists"; `engine-wire-contract.md` §9.1, `p2` §5.8; decision
   `SUBSYSTEM-FLAG-CAPABILITY-SEMANTICS`). The canonical `getEngineStatus` row (`docs/specs/gnosis.md:874`) does
   not define the flags' meaning, and the landed engine still emits six hard-coded `true`s (a false
   claim; U3/U5 fix the code) — this is a **redefinition of a canonical field's semantics**, so it needs
   the canonical statement too. Any additive capability-vs-index signal lands on **`HealthReport`**, and
   `EngineSubsystems` (frozen, §4.6.1) **MUST NOT** change shape.

**Please state all six in the canonical contract** so the consumer docs, the engine specs and the shell
agree. **Landing note (2026-09-16): U1 has LANDED** (its spec-gate reviewer loop returned **EMPTY**), so the
engine-side pins the ask below refers to are landed in the engine specs it names — `docs/specs/engine-wire-contract.md`
§16 / §4.6–§4.7 / §9.1 and `docs/specs/p2-gnosis-server.md` §5.3–§5.9 — the six items remain the
**upstream** ask (the canonical `docs/specs/gnosis.md` is untouched, as required) and the code-bearing
units (**U2/U3**) are **AUTHORIZED (2026-09-16 — their typed registers are authored in
`docs/specs/p2-gnosis-server.md` §9.5.1/§9.5.2), while **U4/U5 remain HELD**. **SUPERSEDED (2026-09-17,
post-greens doc-review): U2's code has LANDED REALIZED-GREEN** (`docs/next-steps.md`'s U2 DONE row) — so the
"code is still owed" clause of this sentence now applies to **U3 only**; **U4/U5 remain HELD**. **SUPERSEDED
AGAIN (2026-09-17, the U3 documentation pass): U3's code has LANDED-GREEN** (`get_engine_status`'s read-time
flag derivation, the lib seam `boot_wiring`, the bin boot, and the amended V-8.1/V-8.2 literals — see this
file's §"U3 documentation pass (2026-09-17)" block) — so **no unit's code is owed among U2/U3**; **U4/U5
remain HELD** and are the only unauthorized code-bearing units left. The historical clause read "the *code* is
still owed". The query-body half of the ask (the request/response shape — envelope-strict + bare
`RagResult`) is stated in **row 51's part (b)** — it is **not** one of the six items above — and is
unchanged by this addendum.

**Note (2026-09-16): the block immediately below is the one exception to this file's ownership
rule** — its owner is **this repo**, not another project. It is kept in its own labelled section
(not folded into the upstream table above) so the upstream/canonical-contract reconcile rows stay
unambiguous.

## U2 adversarial pass (2026-09-16) — engine-side package items

**Why this block exists.** Every other row in this file is owned by **another project** (the Auspicion
Suite top-level repo, the provident-ssr foundation, or another suite tool). The rows below are **NOT**:
they are **this repo's (the Gnosis engine's) own deferred work** — the three engine-side
package/foundation items the U2 (query POST contract) **adversarial pass** surfaced. They are recorded
here because the tracker convention for a package/foundation finding is `docs/defects.md` +
`docs/HANDOFF.md` (AGENTS.md, adversarial gate 4), and because they must be **scheduled as their own
units** rather than absorbed — but their **owner is this repo**, and they are **not** an ask of the
canonical contract or of any consumer. **U2 stayed scope-clean:** it deliberately did **not** absorb
any of them (the disposition for each is a *separate spec + register rows*, i.e. its own unit), and
none of the three is a **U2 regression** — `P-1` and `P-2` are **pre-existing** transport behavior, and
`P-3` is **F2 wire-foundation** code that U2 consumes rather than owns. Their full OPEN rows live in
`docs/defects.md` §OPEN (`P-1`/`P-2`/`P-3`; `P-4` is the test-side trip-wire, recorded **FIXED** in the
same pass and **not** an engine item). The upstream/canonical-contract reconcile rows above are
untouched by this block.

**Proofread-pass additions (2026-09-17) — the two rows the U2 documentation audit filed, annotated with their
2026-09-16 supervisor dispositions (`P-5` remains OPEN; `P-6` is CLOSED — and the audit's lower-severity
companion was filed as `P-7` and is CLOSED docs-side).**
The U2 documentation audit (proofreader) filed **`P-5`** and **`P-6`** in `docs/defects.md` §OPEN; neither is a
U2 regression and neither is an upstream/canonical-contract ask:

- **`P-5` (transport / the SSE branch):** the **pre-existing** pre-stream `StoreError` `error` frame on
  `GET /rag/stream` is rendered `text/plain; charset=utf-8` (`src/bin/gnosis_server.rs:272-281` — the
  `:264-281` citation this bullet carries is stale, the U3 landing having moved the branch) while the newly
  pinned SSE media type is **`text/event-stream`** for every SSE response (`p2` §5.4's F-2 bullet / §5.6 / §10,
  F2 §4.4) — the U2 renderer already emits the pinned header (now `:100-115`; the `:99-114` citation this
  bullet carried is stale, the U3 landing having shifted the fn), so only that older branch diverges.
  **Owner: the transport unit / the SSE branch.**
- **`P-6` — RESOLVED 2026-09-16: the pinned clause STANDS and is now IMPLEMENTED (defect row closed; no
  longer an open item).** Filing record (proofread pass, 2026-09-17): the pinned **wrongly-typed-`n` member**
  clause (`p2` §5.3's `multiQuery` cell + two-layer bullet, §9.5.1/§9.5.4/§10/§11, F2 §4.5/§13 — a wrongly-typed
  `n` ⇒ the field is `None`) was **not implemented** by the landed decoder (`src/wire/query.rs:183-190` kept the
  object and defaulted `n`), and the executed property layer asserted the landed reading, so no test caught it;
  it was reported rather than silently resolved, with the two options named (implement the clause **or** narrow
  it by a pinned-rule amendment) left to the supervisor's disposition. **Supervisor disposition (2026-09-16):
  the clause is CORRECT and is NOT narrowed — the decoder is implemented to it.** The decoder
  (`object_option` and its member-type arms, `src/wire/query.rs:178-202`) now checks **every documented member**
  (`enabled` a bool, `n` a u64
  for `multiQuery`; `enabled` a bool for `subTaskDag`), so `{"multiQuery":{"enabled":true,"n":"3"}}` ⇒
  `multi_query == None` while an **omitted** `n` inside a well-formed object still defaults to `3`. The code
  change and the TestWriter's aligned probe (`tests/props_gnosis_server.rs` — `object_option_is_well_formed` /
  `absorbed_expectation` must check `n` too, and the row doc comment at `:1170-1172` is corrected) land in the
  **same U2 pass**; the spec text was **not** weakened, no register row moved, and **no item is owed here
  anymore** (the row is FIXED/CLOSED in `docs/defects.md`). **Owner at close: the Implementer (code) + the
  TestWriter (probe), one U2 pass** — reopen only as a test-side row if that probe does not land with the code.
- **`P-7` (filed with `P-6`'s report, RESOLVED 2026-09-16 — docs-only).** The numeric cells that read "a
  **number** out of `1..=50`/`1..=5` ⇒ FS-3 400" were **readable** as claiming a 400 for a **non-`u64`** JSON
  number (`topK:-1`, `topK:10.5`), which the decoder's `as_u64` reads treat as **absent ⇒ the documented
  default** (`src/wire/query.rs:119`, `:126`, `:128`, `:140`); **no test pinned that input either way**.
  **Supervisor disposition (2026-09-16): the WORDING was corrected, no behavior change** — every such cell
  (`p2` §5.3's per-key table + fail-state table + wrongly-typed bullet, §10, and F2 §4.5/§13/§16) now states the
  **two distinct halves** (type ⇒ absent ⇒ default; a present **u64 out of range** ⇒ 400 `validation_error`).
  Recorded as FIXED/CLOSED in `docs/defects.md`; **no code was touched and nothing is owed** — a future unit
  that wanted a non-`u64` number to be a 400 would be making a **rule change** (its own spec-amendment unit +
  register row), not fixing this row.

**Post-greens doc-review (2026-09-16, docs-only).** The documentation reviewer reconciled this file's U2-bearing
rows against the landed tree in the same pass: the block's status line ("no engine code lands before U2/U3"), the
GR-6/GR-7/GR-8 owner cells ("U2/U3 AUTHORIZED (registers authored)") and the U1 addendum's landing note were
annotated **U2 LANDED (2026-09-17) / U3 LANDED-GREEN (2026-09-17) / U4/U5 HELD**, the GR-9/RBAC row's
`src/bin/gnosis_server.rs:73-123` citation was re-pointed to the landed decode/dispatch path (`:121-201`), and
the `P-6`/`P-7` dispositions above were confirmed CLOSED while **`P-5` remains OPEN**. Full record:
`archive/reviews/2026-09-16-u2-query-post-contract-doc-review.md`. No source, test or cargo file was touched; the
upstream/canonical-contract reconcile rows above are untouched.

| Item | Owner | Detail |
| --- | --- | --- |
| **U2 adversarial pass — the three engine-side package items (`P-1`, `P-2`, `P-3`), deferred as this repo's own work** | **this repo (Gnosis engine) — one later unit each; NOT another project, NOT a canonical-contract ask** | **(P-1, transport)** Unbounded request-body buffering on **every** route: `src/bin/gnosis_server.rs:191` (`crud_handler`) and `:206` (`rag_query_handler`) take `body: String` with **no** `DefaultBodyLimit`/length cap anywhere in the bin, and `src/wire/envelope.rs:41-58` materializes the whole JSON tree (with `serde_json/preserve_order` enabled, `Cargo.toml:11`) before any decode decision — a loopback-reachable process can drive **O(body) heap**; the only bound today is the OS/process limit. **Pre-existing, not a U2 regression**; currently **documented-but-bounded** by the accepted loopback residual risk (`docs/specs/gnosis-gr-inbound-review.md` §Residual risk register). *(P-2, transport)* The SSE route's `Query<HashMap<String, String>>` extractor (`src/bin/gnosis_server.rs:247`) rejects a malformed query string in **axum's own plain-text 400 before the handler runs**, so §5.5's "one shared rendering … every route that can fail to decode a request" (`docs/specs/p2-gnosis-server.md` §5.5) has an **unlisted hole** on the SSE route; **pre-existing**, and unreachable from the pinned cases. *(P-3, F2 wire foundation)* `validate_rag_result` (`src/wire/decode.rs:63-71`) checks only `engine != "gnosis"` and `blocked_by ⇒ graph trace`, so a `RagTrace::Vector(TraceDescriptor { mode: Flat, … })` or citations contradicting `results` **validate successfully** — which makes the `P-TP-4` register prose ("trace present and **mode-consistent**", `docs/specs/p2-gnosis-server.md` §9.5.1) describe **more than the validator enforces** (a prose-vs-validator gap, recorded explicitly). **Disposition (all three):** OPEN, deferred to **a later transport unit** — each with its own spec + register rows, and **not** an upstream ask. **Status note (2026-09-16):** these three are **still OPEN**; the two proofread rows are **homed in the block above** and are **not** rows of this table — **`P-5` stays OPEN** (same transport/SSE class and the same owner: the transport unit / the SSE branch), while **`P-6` was CLOSED 2026-09-16** (the clause was ruled correct and the decoder implemented to it, same U2 pass) and the audit's companion numeric-wording row **`P-7` was CLOSED docs-side 2026-09-16** (wording corrected, no behavior change).'s decode-failure contract changed, which touches that route's contract) and to **the F2 wire foundation** (P-3 is a validator strengthening — new `ValidationFailure` classes + their rendering decisions); each needs **its own spec + register rows**, which is exactly why the U2 unit **deliberately did not absorb them**. **Revisit triggers:** P-1 — any non-loopback bind, an engine-owned large-payload surface (GR-6/GR-7), or the transport unit's scheduling; P-2 — the transport unit that owns §5.5's rendering, a consumer report of a plain-text 400 from `GET /rag/stream`, or U4 re-touching the SSE extractor; P-3 — the F2 validator-strengthening unit being scheduled, or any consumer relying on mode-consistency/citation integrity of an emitted result. Full rows: `docs/defects.md` §OPEN. |

## U3 documentation pass (2026-09-17) — engine-side deferred items

**Why this block exists (same rule as the U2 block above).** Every other row in this file is owned by **another
project** (the Auspicion Suite top-level repo, the provident-ssr foundation, or another suite tool). The rows
below are **NOT**: they are **this repo's (the Gnosis engine's) own deferred work**, surfaced by the **U3
(status honesty) documentation pass** — and by the U3 **live-scenario battery** it audited. They are recorded
here because the tracker convention for a package/foundation finding is `docs/defects.md` +
`docs/HANDOFF.md` (AGENTS.md, adversarial gate 4) and because each must be **scheduled as its own unit** rather
than absorbed. **Their owner is this repo**, and they are **not** an ask of the canonical contract or of any
consumer. **U3 stayed scope-clean:** it deliberately did **not** absorb either item — each needs its own spec +
register rows — and **neither is a U3 regression**: every pinned U3 row passes (five typed rows HELD,
`P-IM-7` 25 / `P-IM-8` 26 / `P-IM-9` 26 / `P-SM-5` 25 / `P-SM-6` 25 = 127 executed ≤ 400; blind set 13/13;
live battery 9/9 rows live, including `R-L2`). Their full OPEN rows live in `docs/defects.md` §OPEN (**P-8**,
**P-9**); the upstream/canonical-contract reconcile rows above are untouched by this block.

| Item | Owner | Detail |
| --- | --- | --- |
| **U3 documentation pass — the two engine-side items (`P-8`, `P-9`), deferred as this repo's own work** | **this repo (Gnosis engine) — one later unit each; NOT another project, NOT a canonical-contract ask** | **(P-8, transport/status liveness)** `GET /engine/status` is a **boot-time snapshot**: after a boot whose provider was reachable and then **became unreachable**, the surface still reports `state:"Ready"`, `embedding:true`, `lastError:null` — byte-identical to the healthy read — because nothing re-evaluates `EngineState` after `main()`'s probe (`src/store/mod.rs:4193-4232` derives `state` from the stored value; the boot, `src/bin/gnosis_server.rs:378-408`, sets it once). So the "honest signal in that case is the `state`/`last_error` pair" that `docs/specs/p2-gnosis-server.md` §9.5.2's `P-IM-8` prose promises is **never delivered for post-boot loss**. **Proven by U3's own live battery** (2026-09-17). **Pre-existing in kind and NOT a U3 regression** — `P-IM-8` pins `embedding` as the **wired capability**, and the wired provider is still wired, so every pinned U3 row passes; the gap is the **liveness** of `state`/`last_error`, which no U3 row claims. **Disposition:** OPEN, deferred to **a later transport/status unit** — either a re-probe on the status read/a bounded interval (with the pinned `Degraded` + `lastError` transition), or an **explicitly documented staleness window** (the cheaper, honest option). **Cross-filed** in `docs/defects.md` §OPEN as **`P-8`**. *(**P-9, codec)** `src/wire/codecs.rs:94-99`'s `expect("RagResult is Serialize")` is a **reachable panic path** for a non-finite `score` (`serde_json` rejects `NaN`/`±Inf`); reachable only through score production (`gnosis_eval` seeds a `VectorIndex` directly). Pre-existing, out of U3's scope, **not** a U3 regression. **Disposition:** OPEN — owner = the F2 wire foundation / the transport unit (map the failure to a §11 outcome, or reject non-finite scores at the leg boundary); **cross-filed** in `docs/defects.md` §OPEN as **`P-9`**.) |

**U3 LANDED-GREEN (2026-09-17) — the status clause in this file's U2/U3 lines now reads LANDED.** U3's code landed
(the read-time flag derivation in `get_engine_status`, the lib seam `boot_wiring`, the bin boot that writes **no**
flag mask, and the V-8.1/V-8.2 golden literals amended in the same unit), so every clause that read "U3
AUTHORIZED (code owed)" is annotated **U3 LANDED-GREEN (2026-09-17)** with its historical text retained; **U4/U5
remain HELD** (**SUPERSEDED for U5 by the 2026-09-22 blocks below: U5 was AUTHORIZED on 2026-09-22 and has since
LANDED-GREEN with all eight gates run — `docs/next-steps.md` §DONE's U5 row; **U4 alone remains HELD** and needs
the consumer-side `SHELL-2`**). Verified: **`cargo test` 604 passed / 0 failed** (serial `-- --test-threads=1`), green **with and
without** `GNOSIS_SERVER_OLLAMA_URL` set (the suite is hermetic); `cargo fmt --check` exit 0;
`cargo clippy --all-targets` 0 warnings; `cargo build` clean; **U3 blind-greens 13/13**, **U3 live battery 9/9
rows live** (incl. `R-L2`), and the five U3 property rows HELD (127 executed ≤ 400). The U3 DONE row and the
review record are **not** written here — the supervisor and the documentation-review gate (gate 8) own those.
**UPDATE (2026-09-17, the U3 post-greens documentation review, gate 8): both are now written** — the U3 DONE
row is `docs/next-steps.md` §DONE's first data row and the review record is
`archive/reviews/2026-09-17-u3-status-honesty-doc-review.md`; that pass re-verified `P-8`/`P-9` as **OPEN**
against the tree, confirmed the `P-6`/`P-7` closures and the `P-5` **OPEN** disposition, and left the U4/U5
clauses unchanged.

**U5 LANDED-GREEN (2026-09-22) — the unit's gate chain is complete; the status clauses above now read LANDED,
and the unit's OPEN residues are this repo's own work (not upstream asks).** U5 (the boot vector-index build, the
one authorized unit of the `GRQ` go-ahead) has landed: `build_boot_vector_index` (body
`src/store/mod.rs:5369-5414`, name re-exported `src/lib.rs:126`), the bin's boot wiring
(`src/bin/gnosis_server.rs:381-437`; the build call at `:414`), and the comment-only reconciliation at the
flag-derivation site (`src/store/mod.rs:4210-4215`). Verified: **`cargo test` 655 passed / 0 failed** (serial;
604 baseline → +24 U5 in-crate → +27 blind), hermetic with and without `GNOSIS_SERVER_OLLAMA_URL`; the §9.5.5
property layer **314 executed / 355 caps ≤ 400** (8 rows HELD); the blind set
`tests/blind_u5_boot_vector_index_greens.rs` **27/27 GREEN**; the live battery row **`R-L3` (i)/(ii)/(iv) PASSED
live** with **(iii)** and **(v) PARKED** (reasons re-confirmed live) and **`R-L2`'s amended `vector:true` cell
re-verified live**; `fmt --check` exit 0, `clippy --all-targets` 0 warnings, `build` clean. **The
failed-`Reachable`-build branch is lib-level only — it is NOT live-verified** (the bin's boot store is empty, so
a reachable provider's build makes zero `embed` calls). **U5's six OPEN rows live in `docs/defects.md` §OPEN and
are deferred as this repo's own work:** `U5-ADV-1` (the boot index is frozen — a post-boot write makes
`mode=vector` a silent 200 with missing hits while `vector` stays `true`; live-confirmed; a follow-up **"U5
honesty/freshness"** unit is being opened for it), `U5-ADV-2` (pre-bind build unbounded + un-timed),
`U5-ADV-3` (`Degraded` + `embedding:true` + a `lastError` blaming embedding), `U5-ADV-4` (the build's `Err`
discarded), `U5-ADV-5` (the lock-`unwrap` class on a pre-bind path) and **`P-9`'s fired trigger** (a non-finite
provider vector reaches `encode_result`'s `expect`). **None is an upstream/canonical ask**; the PBT audit's
**T1–T10** negative-generator list is an undischarged **TestWriter** obligation. Doc-review record:
`archive/reviews/2026-09-22-u5-boot-vector-index-doc-review.md`; the U5 DONE row is `docs/next-steps.md` §DONE's
first data row. **U4 remains HELD** (it needs the consumer-side `SHELL-2`).

## Round 1 — 2026-09-09

The §4.1 document store landed; the only cross-project item surfaced is the §4.4.3
crosslink publish-gate contract ambiguity above. The suite `docs/defects.md`
GAP-1/GAP-2 (multi-query, three-way lexical fusion) remain the engine-owning gaps
already re-pointed to Gnosis.
