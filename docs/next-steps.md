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

## OPEN

| Unit | Status | Notes |
| --- | --- | --- |
| **The doc-store + knowledge-graph + fact/citation + consistency core** (spec §4.1–§4.4) | ready to delegate | The document store (§4.1), knowledge graph §4.2 (nodes/edges/properties, subject-relation model, entity resolution, manual overrides), fact/citation (§4.3), and consistency enforcement (§4.4) form the core engine. Delegate per unit (RCA-5): each its own TestWriter-red → Implementer-green → adversarial → blind-greens → doc-review cycle, from the spec ALONE. |
| **The RAG/agent-memory retrieval stack** (spec §4.5–§4.6) | ready to delegate (second) | The query modes (`flat`/`graph`/`vector`/`hybrid`), the retrieval stack (lexical BM25, reranking, multi-query, compression, HyDE, sub-task DAG), multiple vector fields, the agent-memory surface, and the query/stream/engine-status API. Delegate per unit after the core lands. |
| **The IPC/HTTP engine seam** (spec §4.6.1, §5.1) | pending | The exact IPC/process transport between the shell and Gnosis (the `RagStore` + query/stream/engine-status seam) — F2 in the spec. Design and delegate once the core API exists. |

## DONE

_(none yet — the scaffold is in place; implementation units are pending.)_
