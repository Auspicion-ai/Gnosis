# Authorship-source property + future locks — DESIGN REVIEW

- **Status:** **DESIGN-RECORDED (2026-09-09)** — a design decision, **doc/design-only
  now** (the code-bearing TDD unit is deferred until the F4-LLM integration or a
  concrete lock use case unparks).
- **Decision:** `AUTHORSHIP-SOURCE-PROPERTY` (`docs/decisions.md`).
- **Author role:** change-analysis (design pass).

## What this is

The user requested a **property tracking authorship source** on Gnosis's enrichment
surfaces, so **future locks on changes** can be enforced — e.g. agents cannot
override human input; junior staff cannot override changes made by senior staff
roles. This design records the property shape, where it lives, the authority
ordering, the lock policy, and how it interacts with the manual-override-
authoritative invariant (§4.2.8.4).

## The design

### 1. The property shape — structured `{source, authority}`

```rust
/// Who/what authored the enrichment (§4.2.8 attribution dimension, mirrors
/// `Document.author` / `QueryAuditEntry.requester`).
pub enum AuthorshipSource { Human, Agent, LlmHost, System }

/// The comparable authority rank — the lock's comparison key. Total order:
/// System < Agent < LlmHost < HumanJunior < HumanSenior.
pub enum AuthorityRank { System, Agent, LlmHost, HumanJunior, HumanSenior }

pub struct Authorship { pub source: AuthorshipSource, pub authority: AuthorityRank }
```

- **`authority`** is a total-ordered enum — the lock's comparison key (a free string
  like `Document.author` is NOT comparable, so it is the wrong precedent for a lock).
- **`source`** is the attribution/audit dimension (who: human vs agent vs LLM-host).
- The authority is **presented by the caller** (like `requester`), not derived by
  the engine — the trust boundary is the authenticated shell / LLM host.

### 2. Where it lives — persisted on the record, set from a per-call parameter

A lock needs the **prior** author's authority to compare against, so the
authoritative copy must be **persisted** (a transient per-call value is
insufficient).

| Surface | Persisted location | Per-call input |
| --- | --- | --- |
| `declareCommunity` | `Community.authorship` | `DeclareCommunityOptions.authorship: Option<Authorship>` |
| `updateCommunitySummary` | overwrites `Community.authorship` (subject to lock) | new `authorship` parameter |
| `resolveEntities` | new `entity_authorship: RwLock<HashMap<(DocumentId,NodeId), Authorship>>` keyed by canonical | `ResolveEntitiesOptions.authorship` |
| `mergeFacts` | `Fact.authorship` | `MergeFactsOptions.authorship` |

### 3. The lock policy — opt-in, default-OFF, engine-enforced

- **Ordering:** `System < Agent < LlmHost < HumanJunior < HumanSenior`.
- **Rule:** a change may override a prior change only if `authority(new) >=
  authority(prior)`; otherwise the engine rejects with `InsufficientAuthority`.
- **Enforced in the engine** (the single place holding both the prior record's
  persisted authority and the new call's authority) — authoritative + D4 parity.
- **Opt-in / default-OFF** — the current contract + 387 tests pin these surfaces as
  unconditional authoritative overwrites (manual-vs-manual last-write-wins, per
  `RESOLVE-ENTITIES-AUTHORITATIVE-OVERWRITE`); the lock is a new enforcement layer
  that defaults OFF (preserving current behavior).
- **`InsufficientAuthority` is RESERVED** (per `RESERVED-ERRVARIANTS-DISCIPLINE`)
  until the lock actually lands — not fabricated into a throw-path now.

### 4. Interaction with §4.2.8.4 — refines + strengthens, core unchanged

- **Refines:** an agent/LLM-host is still a **manual caller** in the §4.2.8.4 sense
  (it calls the manual-override surface, not an engine-internal automatic pass), so
  the invariant's "automatic computation" category is unchanged.
- **Strengthens:** the lock adds a new guarantee layered on top — a human's
  declaration can't be overridden by a lower-authority agent/LLM-host (when the lock
  is on). §4.2.8.4 protects against automatic computation; the lock protects against
  lower-authority *callers*. Orthogonal and compose.

### 5. Backward compatibility — additive in contract, code-bearing, doc-only now

- **Additive:** `authorship: Option<Authorship>` on records/options, defaulting to a
  sensible value; the lock is opt-in/default-OFF. No existing behavior changes.
- **But code-bearing:** changes `Community`/`Fact` (both `Serialize`/`Deserialize`,
  re-exported from `src/lib.rs`), the options structs, the `RagStore` trait
  signatures, and the **387 green tests** that construct these types literally.
- **Therefore doc/design-only now:** the F4-LLM integration is SPECULATIVE and the
  lock is a future feature. The code-bearing TDD unit (TestWriter red set from the
  spec, PBT register + property layer + audit per `PBT-GATE-MANDATORY`) lands when
  the F4-LLM integration or a concrete lock use case unparks.

## The F4-LLM integration

The LLM host presents `{source: LlmHost, authority: LlmHost}` on each enrichment
call; a future engine lock prevents it from overriding a human's higher-authority
declaration. The F4-LLM spec's core principle ("LLM proposes, Gnosis's manual
override stays authoritative") is preserved and refined — the LLM host is still a
manual caller, but a lower-authority one.

## Cross-references

- `docs/decisions.md` `AUTHORSHIP-SOURCE-PROPERTY`, `RESOLVE-ENTITIES-AUTHORITATIVE-OVERWRITE`,
  `RESERVED-ERRVARIANTS-DISCIPLINE`.
- `docs/specs/f4-llm-enrichment-integration.md` (the future-integration spec this
  design feeds).
- `docs/specs/gnosis.md` §4.2.8/§4.2.9/§4.2.8.4.
- `docs/pending.md` (SPECULATIVE row).
