# Familiar — knowledge retrieval + optional memory store

**Edge:** Familiar → Gnosis (`docs/specs/gnosis.md` §5.7).

## Nature

Familiar (the "smart assistant" agent harness) gains graph-aware retrieval and
document creation over Gnosis's knowledge graph. **Optionally**, Familiar can
use Gnosis (via Astrographer) as its **memory store** (decision
**FAMILIAR-MEMORY-STORE-INTEGRATION**, `docs/integration-matrix.md` edge #42).

## Mechanism

- **Knowledge / research (always):** Familiar's F1/F4 edges route through the
  shell's RAG surface, which proxies Gnosis (§5.7). The agent-memory retrieval
  surface (`docs/specs/gnosis.md` §4.5.4) serves Familiar's knowledge retrieval.
- **Memory store (optional):** when enabled, Familiar's memory store uses Gnosis
  as its backing: the facts table maps to Gnosis's `fact` nodes (§4.3.1), the
  profile summary to the derived profile summary (§4.5.4), the candidate-fact
  pipeline to the candidate-fact + deterministic validation (§4.3.2a), and
  vector search over memory to the RAG `vector` mode (§4.5.1). The **local**
  memory store remains the default; the Gnosis backing is opt-in (D2).

## Value

Familiar gains graph-aware retrieval + document creation, and (optionally) a
scalable, graph-backed memory store with Gnosis's fact/citation/consistency/
retrieval machinery.

## Fail-states

The engine is optional (D2): when Gnosis is unreachable, Familiar's knowledge
operations fail while the assistant core continues; the optional Gnosis-backed
memory store falls back to the local default (not an error).
