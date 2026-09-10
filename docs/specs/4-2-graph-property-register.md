# §4.2 Knowledge Graph — Property-Based-Testing (PBT) Property Register

- **Unit**: §4.2 Knowledge graph (`src/store/mod.rs`; the `Store` graph surface
  re-exported from `src/lib.rs`). Exercises `tests/graph_integration.rs`.
- **Spec**: `docs/specs/gnosis.md` §4.2.1–§4.2.9 (state/fail-state contract);
  concurrency plan `docs/research/gnosis-data-structures-concurrency-plan.md`.
- **Date**: 2025-XX-XX
- **Role**: `role_spec_writer` — author of this **property register** (artifact #1
  of the §4.2 PBT gate).

---

## PBT-gate contract (mandatory)

This register is the **PBT-gate contract** for the §4.2 unit. Three artifacts are
required per code-bearing unit; this file is artifact #1 (the **typed property
register**).

1. **This register** (spec_writer) — the typed properties below.
2. **Executed property layer** (TestWriter) — runs under `cargo test` with a
   **deterministic pinned seed**, **≤100 generated cases per row** and **≤400
   total across the unit's property layer**, **stop-after-5** (≤5 distinct
   held/broken counterexamples per row). Each row is recorded **held** or
   **broken** with its **strategy-id**.
3. **Read-only PBT audit** (adversarial reviewer) — over-strength reasoning per
   row, generator-coverage audit, prose counterexamples, negative-generator
   requests. The reviewer **never runs generators**.

The property layer must be deterministic: the pinned seed is fixed; generated
cases are reproducible across runs. Every row below is an **invariant TRUE of the
current GREEN implementation** (implied by the already-green §4.2 example suite
and its GRAPH-OWNS-RELATION-AND-MERGE / HIGH-* / CRITICAL-* regression tests) —
**not** a pending feature.

### Property taxonomy

- **P-IM** — *input-model* invariant: a contract over **legal inputs** /
  validation — return-shape correctness, input parameter honored on legal input.
- **P-SM** — *state-model* invariant: a contract over **graph state / transitions**
  — a structural predicate the graph state always satisfies.
- **P-TP** — *transform / preservation* invariant: an operation is **preserving —
  idempotent / no-clobber / authoritative** under repetition or unrelated changes.

**Scope.** The current GREEN `declare_community` does **not** validate that the
declared member nodes exist (only that the node set is non-empty and the summary
is non-empty). A "community membership validity" row is therefore **excluded**
(not TRUE of the current implementation). All rows below have been refined
against the actual public API and the GREEN behavior.

---

## Property table

`Property-id | Class | Invariant | Strategy-id | Observable-as-property`

| Property-id | Class | Invariant | Strategy-id | Observable-as-property |
|---|---|---|---|---|
| **P-IM-1** | IM | Triple return-shape consistency (GRAPH-OWNS-RELATION-AND-MERGE): for legal `addTriple`, the returned `Triple` mirrors the (subject, relation, object) arguments exactly, its `relation_type` equals `relation`, and its `created_at` is non-empty. | `gen_triple` | For every generated legal triple `addTriple(s, r, o, w)`, the returned `Triple` satisfies `subject == s ∧ object == o ∧ relation == r ∧ relation_type == r ∧ !created_at.is_empty()`. |
| **P-IM-2** | IM | `relationType` filter honored: `getTriples` with a legal non-empty `relation_type` returns only triples whose `relation` equals that type (and which touch the queried node). | `gen_filtered_triples` | For every generated triple set and queried node `n` with `relation_type = Some(rt)`, every element `t` of `getTriples(n, filter)` satisfies `t.relation == rt`, and `n ∈ {t.subject, t.object}` per direction. |
| **P-SM-1** | SM | Adjacency consistency: the graph state reports each edge exactly once per endpoint — `edges_from(v)` is precisely `{e : e.source == v}` and `edges_to(v)` precisely `{e : e.target == v}` for every node `v` of the document. | `gen_adjacency` | For every node `v` in the generated graph, `edges_from(v)` contains every edge with `source == v` and none with `source != v`; `edges_to(v)` symmetrically by target. |
| **P-SM-2** | SM | Reference-graph acyclicity / topological order: a **successful** `resolveReferences` returns a topologically ordered, non-redundant list — each resolved node appears exactly once, `resolution_order` values are a strict 1..N permutation, and every reference that depends on another resolved node has its target at a strictly smaller `resolution_order`. | `gen_reference_chain` | For the returned `Vec<ResolvedFact>`, `n == len`, the `resolution_order`s are exactly `{1..=n}` in the result sequence, and for every resolved `Reference` node whose `target` is also resolved, `order(target) < order(node)`. |
| **P-TP-1** | TP | `setReferenceState` is a **targeted** no-clobber mutation: only the matched reference edge(s)' `state` changes; the document's node set and every other edge (identity **and** state) are preserved, and the revision advances by exactly one. | `gen_reference_state` | Let `G_before = getDocument(doc).graph` and `G_after` after `setReferenceState`. Then `G_after.nodes == G_before.nodes`, every edge except the matched reference edge has identical source/target/kind/state in `G_before` and `G_after`, the matched reference edge's `state == Some(requested)`, and `revision_after == revision_before + 1`. |
| **P-TP-2** | TP | `resolveEntities` **same-call idempotent + authoritative-overwrite convergence** (decision RESOLVE-ENTITIES-AUTHORITATIVE-OVERWRITE): re-applying the **identical** `(entity_ids, canonical_id)` call leaves the durable alias→canonical mapping unchanged — re-applying the identical call does **not** flip an alias's canonical. Each call is an **authoritative overwrite**: it re-stabilizes the durable map to an **acyclic, flat** alias graph — the chosen canonical is never itself an alias (its prior entry is dropped) and every alias in the requested set points **directly** to the canonical, with any prior alias that pointed to a now-re-aliased node re-pointed via path-compression. A later manual call supersedes an earlier one (manual-vs-manual last-write-wins); the §4.2.9.2 "never overwritten by an **automatic** computation" invariant is preserved. **Overlap convergence is now GUARANTEED** — no residual alias, no cycle. | `gen_entities` | After `resolveEntities(ids, {canonical_id: c})`, snapshot `entityAliasCanonical(a)` for every alias `a`; re-apply the identical call, then re-read — every recorded alias still maps to the same canonical, and the two `ResolutionResult`s (merged + aliases) are equal. For an overlapping re-resolution with a **different** canonical, the map fully converges (no residual alias, no cycle): the canonical is never an alias and every alias **in the requested set** (and any path-compressed prior alias) points directly to it. *(The map may legitimately hold multiple roots from disjoint resolutions — "every alias points directly to the canonical" is scoped to the requested set + path-compressed aliases, not the whole map.)* |
| **P-TP-3** | TP | `mergeFacts` idempotent / union-stable: merging an already-merged set (or its post-merge superset) with the same canonical key is a no-op — the persisted fact's `value` is unchanged and its `citations` equal the deduped union of the merged facts' citations (no drift, no dupes). | `gen_facts` | `mergeFacts(keys, {canonical_key: c})`; then `getFact(w, c)` after one application vs. after two (and after merging `keys ∪ {c}`) returns the identical `value`, and its `citations` set equals the deduped union of the merged facts' citation sets in both reads. |
| **P-TP-4** | TP | Manual-override authority: a manually declared community's `summary` is never overwritten by an automatic computation/member-touching operation; it changes only via an explicit `updateCommunitySummary`. *(This is an override-authority **smoke check** — the manual `summary` persists and only an explicit `updateCommunitySummary` changes it — because automatic community-summary derivation is PARKED (`docs/pending.md` F4). Duplicate member node ids are stored **verbatim** (`node_ids.to_vec()`), never de-duplicated.)* | `gen_community` | `declareCommunity(members, {summary, w})`; apply a member-touching operation (an `addTriple` whose `subject` is a member); `getCommunity(id).summary` still equals the manual `summary`. After `updateCommunitySummary(id, new)`, `getCommunity(id).summary == new`. |

---

## Generator-coverage note (per row)

The TestWriter's `strategy-id` generators should also probe the following boundary
and adversarial input shapes (in addition to the nominal case):

- **P-IM-1 `gen_triple`** — nominal: distinct subject/object nodes, varied
  non-empty relations (short, long, whitespace, Unicode). Boundaries: `subject`
  and `object` in the **same** document and across **two** documents
  (`cross_wiki: true`); a self-loop (`subject == object`); a single-node document;
  relation strings with leading/trailing whitespace (still non-empty). Adversarial:
  relation that collides with an existing triple on the same endpoints but a
  **different** relation string; many triples on the same node.
- **P-IM-2 `gen_filtered_triples`** — nominal: several relations on a node,
  varied `relation_type`. Boundaries: a relation_type that matches zero triples
  (must return empty — filter only, no error); every direction (`Out`/`In`/`Both`);
  a filter `relation_type` equal to only **some** of a node's triples. Adversarial:
  a node that is both subject of one triple and object of another (`Both` must
  return both); two triples with the **same** `relation` on different endpoints.
- **P-SM-1 `gen_adjacency`** — nominal: a node with several outgoing and several
  incoming edges across all `EdgeKind`s. Boundaries: a node with zero edges
  (`edges_from == edges_to == []`); a self-loop (`edges_from` and `edges_to` each
  include it once); the head/end spine plus random edges; a node removed by a
  `updateDocument` (edges_from/edges_to on the retained sibling still correct).
  Adversarial: a graph where a `Relation` edge's source or target node is later removed — the edge must be **pruned** (it touches a **REMOVED** endpoint); an edge between two **RETAINED** nodes is **retained** (§4.2.7.5).
- **P-SM-2 `gen_reference_chain`** — nominal: linear chain, branched/diamond chain
  (a fact shared by two roots). Boundaries: single fact root (leaf only); a
  `reference` directly to a `fact`; max chain depth exactly at the permitted
  `max_hops` (and at `max_hops - 1`); multiple roots sharing a downstream fact.
  Adversarial: a chain with a self-pointing `reference` (excluded — must be a
  **successful**-path generator; `CycleDetected` is a `StoreError`, not an
  invariant, so the generator must only emit **acyclic** subgraphs); a root that is
  a plain `Content` node (resolves to its own value, `order` still monotonic); the
  empty root set.
- **P-TP-1 `gen_reference_state`** — nominal: a doc with one `Link` edge plus
  unrelated `DocHead`/`DocEnd`/`DocChild` edges. Boundaries: each of
  `FRESH/STALE/RESOLVED/BROKEN`; a doc with **multiple** reference edges — assert
  only the matched one changes; a `Crosslink` and an `Embed` reference edge
  (both stateable); a doc where the matched source+target identifies **one specific**
  edge among several. Adversarial: two reference edges sharing the same source node
  but different targets (only the matched target's edge changes); repeated
  `setReferenceState` on the same edge (last write wins, others untouched).
- **P-TP-2 `gen_entities`** — nominal: a set of ≥2 content nodes, explicit
  canonical. Boundaries: `canonical_id: None` (deterministic first-entity choice);
  a single-entity set (no aliases — result empty `merged`/`aliases`); a set where
  the alias is itself an entity of a prior resolution. Adversarial: re-running the
  **same** call after an intervening unrelated resolution must not flip any recorded
  alias; a set whose canonical is reordered in the `entity_ids` argument (the
  explicitly-chosen canonical still dominates).
- **P-TP-3 `gen_facts`** — nominal: two+ same-`value` facts with **disjoint**
  citation sets (union grows), and with **overlapping** citation sets (union
  dedupes). Boundaries: a single fact key (merge of one — union is its own
  citations); a merge where one citation appears in many facts (dedup to one);
  re-merging `keys ∪ {canonical}`. Adversarial: a **second `merge_facts` with a
  larger key set** (a broader union reflects the newest set); ensure conflicting
  `value`s are never generated here (→ `ConflictError` is a fail-state, excluded).
- **P-TP-4 `gen_community`** — nominal: one community, then a triple on a member
  node. Boundaries: a community whose members span **two** documents; a community
  of exactly one member; an `addTriple` on a non-member node (nothing should touch
  the summary) as well as on a member; then an explicit `updateCommunitySummary`.
  Adversarial: two communities, only one of whose members is touched — only that
  community's summary is at stake, and neither summary may change automatically;
  member set repeated with a duplicate node id — `declare_community` stores
  `node_ids` **verbatim** and does **not** de-duplicate, so `members` may contain
  duplicate ids (asserted as stored).

---

## API notes for the TestWriter

Exact public API used by the rows (traits `RagStore` impl `Store`; types
re-exported from `src/lib.rs`):

- **Triples** — `add_triple(&(DocumentId, NodeId), &str, &(DocumentId, NodeId), &WikiId) -> Result<Triple, StoreError>`;
  `get_triples(&(DocumentId, NodeId), &GetTriplesFilter) -> Result<Vec<Triple>, _>`;
  `query_triples(&TriplePattern, &QueryTriplesOptions) -> Result<Vec<Triple>, _>`.
  `Triple { subject, relation, object, relation_type, created_at }`.
  The graph owns triples as `Relation` edges (source=subject, target=object)
  carrying `relation_type` (**GR-OWNS-RELATION-AND-MERGE**); `edges_for_document`
  exposes them. Legal-input guards: unknown wiki / doc → `WikiNotFound` /
  `DocumentNotFound`; empty relation / unknown node → `ValidationError`;
  `query_triples` limit must be `1..=100`.
- **Adjacency** — `edges_from`, `edges_to`, `edges_by_kind`, `edges_for_document`,
  `doc_head_for_document` (see §4.2.5). `edges_from(v)` returns every edge with
  `source == v`; `edges_to(v)` every edge with `target == v`.
- **Resolution** — `resolve_references(&[(DocumentId,NodeId)], &ResolveOptions)
  -> Result<Vec<ResolvedFact>, _>`; `ResolvedFact { node, value,
  resolution_order }` (1-based topological order; failure → `CycleDetected` /
  `HopLimitExceeded` — **not** invariants, so generators emit only acyclic
  subgraphs and in-range `max_hops ∈ 1..=5`).
- **Reference state** — `set_reference_state(&DocumentId, &NodeId source, &NodeId target, &str) -> Result<Edge, _>`.
  Accepts `FRESH/STALE/RESOLVED/BROKEN` (case-insensitive); parses to
  `ReferenceState { Fresh, Stale, Resolved, Broken }`. Matches a `Link`/`Embed`/
  `Crosslink` edge by `(source.1, target.1)`.
- **Entity resolution** — `resolve_entities(&[(DocumentId,NodeId)], &ResolveEntitiesOptions) -> Result<ResolutionResult, _>`.
  `ResolutionResult { merged: Vec<EntityPair>, aliases: Vec<Alias>, canonical_id }`.
  Records a **durable** alias→canonical map read via
  `entity_alias_canonical(&(DocumentId,NodeId)) -> Result<Option<(DocumentId,NodeId)>, _>`.
- **Fact merging** — `create_fact(&WikiId, &DocumentId, &str key, &str value,
  &[(DocumentId,NodeId)]) -> Result<Fact, _>`; `merge_facts(&[String], &MergeFactsOptions) -> Result<Fact, _>`;
  `get_fact(&WikiId, &str) -> Result<Fact, _>`.
  `MergeFactsOptions { canonical_key: String, wiki_id: WikiId }`. A merge **persists** to
  the fact store (union citations + refreshed `updated_at`). Conflicting `value`s →
  `ConflictError`, and missing key → `DocumentNotFound` — both fail-states, so the
  `gen_facts` generator must only emit **all-equal-value** fact sets.
- **Communities** — `declare_community(&[(DocumentId,NodeId)], &DeclareCommunityOptions) -> Result<Community, _>`,
  `get_community`, `list_communities`, `update_community_summary`. `Community {
  community_id, members, summary, wiki_id }`. The declared summary is authoritative
  and is never auto-overwritten (automatic derivation is parked).

**Generator constraints (fail-states are NOT invariants).** Every `strategy-id`
must generate only **legal inputs** as its nominal domain; boundary/probe shapes
must stay on the legal side of the §4.2 guards, because a row that drives a
`StoreError` produces no value on which to assert the invariant. The adversarial
reviewer may separately request **negative** generators that confirm the
`ValidationError`/`DocumentNotFound`/`WikiNotFound`/`ConflictError`/
`CycleDetected`/`HopLimitExceeded` shapes — those are a §4.2 fail-state concern
(documented in `docs/specs/gnosis.md` §4.2/§6), **outside** this register.
