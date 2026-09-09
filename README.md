# Gnosis

**Gnosis** is the **production graph/vector engine** of the Auspicion Suite — a
fast, multithread-capable (Rust) backend that owns the document store, the
knowledge graph, fact/citation tracking, consistency enforcement, and the
RAG/agent-memory retrieval stack.

Gnosis is a **pure backend** to the **Astrographer** Electron shell (its
front-end). It has **no MCP surface and no GUI surface**; Astrographer proxies
Gnosis's API over the `RagStore` + query/stream/engine-status seam. Gnosis is the
production replacement for the archived **Incanter** prototype.

## Documentation

| Doc | Purpose |
| --- | --- |
| `docs/specs/gnosis.md` | **The canonical behavior contract** (copied from the Auspicion Suite top-level architecture repo). The compile-horizon-review spec covering the document store (§4.1), knowledge graph (§4.2), fact/citation tracking (§4.3), consistency enforcement (§4.4), RAG/agent-memory retrieval (§4.5), and all query modes/surfaces (§4.6). |
| `docs/research/` | The relevant research notes + web-grounded reports that informed the engine (Advanced-RAG, agent-memory, graph-RAG). |
| `docs/integrations/` | The integration-surface documentation (Astrographer, Zodiac, Solomon, Familiar, Astral, Emerald, Firmament). |
| `docs/next-steps.md` | The work queue. |
| `docs/decisions.md` · `docs/pending.md` · `docs/defects.md` · `docs/HANDOFF.md` | The active trackers. |

## Build

```sh
cargo build
cargo test
```

## The canonical contract

`docs/specs/gnosis.md` is the single source of truth for Gnosis's behavior. A
TestWriter derives every state and fail-state from it. It is **not** a Gnosis-
local invention — it is the suite-level contract the Gnosis project must honor;
changes are reviewed through the Auspicion Suite proposal-review gate.

## License

AGPL-3.0 (open-source first, per Auspicion Suite design constraint D1).
