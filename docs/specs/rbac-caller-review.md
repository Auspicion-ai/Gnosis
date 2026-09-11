# RBAC caller threading — PROPOSAL-REVIEW record

- **Unit:** the §8 RBAC `caller` threading on the document-CRUD mutating methods
  (the OPEN row in `docs/defects.md` + `docs/HANDOFF.md`).
- **Gate:** proposal-review gate (validity ∥ critique → architecture →
  change-analysis).
- **Date:** 2026-09-10.
- **Verdict:** **REJECT the proposal as written (Option A — engine-side RBAC
  enforcement); PROCEED with Option C (re-scope RBAC enforcement to the shell).**
  Contingent on the user's go-ahead (GIVEN 2026-09-10).
- **Status:** proposal approved by review (as amended to Option C); the Option C
  re-scope is a **documentation/decision change, NOT a code change** — no `src/`
  or `tests/` edits.

## What the proposal asked

Add a `caller` param to the 7 mutating `RagStore` methods (`create_document`,
`update_document`, `delete_document`, `publish_document`, `unpublish_document`,
`archive_document`, `create_wiki`) and enforce edit authority **engine-side** (a
caller without edit access → an RBAC denial error). This was framed as the
engine-side half of the RBAC enforcement the Astrographer shell's A2 unit wired
on the shell side.

## The review findings (all read-only passes)

**Validity (VALID-WITH-AMENDMENTS).** The proposal names a real, recorded gap and
a coherent deliverable, and it fits the P1a `caller` shape + the A1/A2 threading.
Must-fix before it could land: (1) the engine's authority-mapping source is
unspecified; (2) the denial `StoreError` variant + §11 status is unspecified;
(3) the proposal reverses the P2 §8 re-scope; (4) the trait-change blast radius
is unenumerated. Also: the read-only surface is ungated; the A1/A2 client impact;
the missing `GNOSIS-RBAC-EDIT-ENFORCEMENT` decision record; the missing roadmap
spec; the canonical-contract silence on RBAC; `createWiki` authority semantics.

**Critique (UNSOUND-AS-WRITTEN).** The central premise — "enforce edit authority
engine-side" — has **no authority-mapping source**: the engine has no notion of
who may edit; the only "who may edit" mapping lives in the shell's boot-time
`AuthorityStore` (`callerId → credential`, from operator settings). Without a
mapping source, engine-side "enforcement" reduces to a non-empty-string check
(security theater). Also: reverses the just-landed P2 §8 re-scope + `P-SM-3`;
contradicts the C5 boundary classification (RBAC → SHELL); breaks the frozen
21-row §11 map / `P-IM-3` enumeration with a new denial variant; ~209-call-site
blast radius; redundant with the shell's caller-side deny; `caller` ambiguous
(credential vs identity); the 7-method scope is unjustified (graph/fact/
consistency mutating methods also mutate); missing-caller (400) vs
unauthorized-caller (403) conflated; idempotency dedup bypasses the engine check;
`createWiki` authority granularity; read-only surface silently ungated; PBT/test
blast radius unaccounted; pure-backend policy separation.

**Architecture (REJECT Option A → recommend Option C).** The engine has no
identity concept and no authority-mapping source, and it binds loopback-only
behind the shell's auth/TLS policy — so engine-side enforcement is both
unimplementable today (no mapping) and redundant (the shell already denies
caller-side). The C5 classification (`astrographer-engine-shell-boundary.md`
line 87: "RBAC to selective versions → SHELL. The authorization gate at the access
boundary → SHELL (a policy/interface concern)") is the project's own, deliberate
answer to "where does RBAC live," and it says SHELL. The sound resolution is to
amend the conflicting `GNOSIS-RBAC-EDIT-ENFORCEMENT` decision to align with C5,
not to force the engine to hold policy it has no business holding. Runner-up:
Option B (a caller-aware decorator) only if a future non-shell direct client
mandates engine-side enforcement.

**Change-analysis (PROCEED with Option C).** Option A is unsatisfiable as written
(no authority-mapping source) and contradicts the recorded architecture. Option C
resolves the recorded gap by re-scoping the enforcement to the shell, where the
authority mapping + the fail-closed A2 deny already live. Residual risk (no
engine-side enforcement) is acceptable under the loopback-only + local-shell-only
trust model; the trigger condition for revisiting is a future non-shell direct
client (mitigation: Option B's decorator).

## The Option C change list (landed in this pass)

1. **Reconcile `GNOSIS-RBAC-EDIT-ENFORCEMENT`** — record it in `docs/decisions.md`
   in its amended, shell-aligned form (RBAC → SHELL). The decision was previously
   only *referenced* (in `defects.md`/`HANDOFF.md`), never recorded.
2. **Close the `defects.md` + `HANDOFF.md` OPEN rows** as "re-scoped to the shell"
   (the shell's A2 fail-closed deny is the enforcement).
3. **Add a note to P2 spec §8** that the engine-side enforcement is deliberately
   NOT implemented (the shell owns it).
4. **Canonical-contract note** — a HANDOFF reconcile request to the Auspicion
   Suite top-level spec (RBAC is a shell concern; the canonical contract has no
   RBAC section today).

## Residual risk (accepted)

Under Option C the engine has **no engine-side RBAC enforcement**. Acceptable
today: the engine binds loopback-only behind the shell's auth/TLS policy (P2
decision), and the only client is the local shell. The shell's A2 deny is
fail-closed (a caller with no edit authority is denied before any proxy call).
The residual risk becomes material only if a non-shell direct client is ever
introduced; the recorded mitigation is Option B (a caller-aware decorator at the
access boundary).
