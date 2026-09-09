# Mem0: extracting information from chat logs

- **runId**: mem0-chat-extraction
- **topicId**: mem0-chat-extraction
- **completeness**: complete
- **residualInjectionMarkers**: 0

## Summary

Mem0 is an open-source, self-hostable "memory layer" for LLM applications that extracts, stores, and retrieves structured long-term memory from conversational data. Its core value proposition is turning raw chat logs into durable, queryable facts about users, entities, and preferences. The extraction pipeline is two-phase: (1) an LLM-driven extraction step that reads a conversation (or a batch of messages) and produces candidate memory facts, and (2) a validation/update step that reconciles those candidates against existing stored memories — adding new facts, updating changed ones, and deleting stale ones. This is exposed through the `add()` operation (which accepts either a single message or a list of messages plus an optional user_id/agent_id/run_id context) and the `search()`/`get_all()` retrieval operations. Mem0 is designed to be model-agnostic (works with OpenAI, Anthropic, Ollama, and other providers), vector-store-agnostic (Qdrant, Chroma, pgvector, etc.), and deployable fully locally — which aligns directly with the Auspicion Suite's "if it can be local, it should be local" constraint. It is licensed under Apache 2.0 (the core open-source library), with a separate managed cloud offering. For the Auspicion Suite, Mem0's chat-log extraction is the foundational mechanism by which each tool can build persistent user/entity memory from its own conversational surface, and — critically — a shared memory layer becomes the interconnection point between tools (constraint D3).

## State of the art

Mem0 (mem0ai/mem0) is currently one of the leading open-source memory-layer frameworks for LLM agents. Its architecture is documented in the official docs (docs.mem0.ai/core-concepts/how-it-works) and in a peer-reviewed-style paper "Mem0: Building Production-Ready AI Agents with Scalable Long-Term Memory" (arXiv 2504.19413). The current state of the art centers on a few ideas: (1) a two-phase extraction-and-update pipeline rather than naive vector embedding of whole transcripts; (2) memory as a first-class, versioned store with add/update/delete semantics rather than a raw retrieval-augmented-generation (RAG) index; (3) model-agnostic and vector-store-agnostic pluggability so the same memory layer runs against local models (Ollama) and local vector stores; and (4) a managed cloud tier layered on top of the open-source core. The extraction step is LLM-driven: given a conversation, the model produces candidate facts (e.g. "user prefers dark mode", "user's company is Acme"), and the update step compares candidates to existing memories to decide add/update/delete. This is materially different from older approaches that embedded entire chat transcripts and relied on similarity search, which produced noisy, non-deduplicated, non-updated memory. The field is converging on "extract structured facts, then reconcile" as the correct pattern, and Mem0 is the reference implementation of that pattern. For a local-first suite, the relevant state of the art is that Mem0 can run entirely on-premise with a local LLM and a local vector store, satisfying data-residency requirements (HIPAA-style constraints) without any cloud dependency.

## Key concepts glossary

- **Memory layer**: A persistent store that sits between an LLM and its application, holding structured facts about users/entities that survive across sessions. Mem0 is the reference open-source implementation.
- **Two-phase extraction**: Mem0's core pipeline. Phase 1 extracts candidate memory facts from a conversation via an LLM; Phase 2 reconciles those candidates against existing memories (add new, update changed, delete stale). This is the mechanism behind "extracting information from chat logs."
- **add() operation**: The Mem0 API call that ingests memory. It accepts a single message string or a list of messages, plus optional context (user_id, agent_id, run_id). This is the primary entry point for chat-log extraction.
- **search() / get_all()**: Retrieval operations. search() does semantic retrieval of relevant memories for a query; get_all() returns the full memory store for a given user/agent.
- **user_id / agent_id / run_id**: Scoping keys. Memories are namespaced per user and per agent, and run_id groups memories from a single run/session — essential for multi-tenant and multi-tool deployments.
- **Vector store**: The underlying index (Qdrant, Chroma, pgvector, etc.) used for semantic retrieval. Mem0 is vector-store-agnostic.
- **Model-agnostic**: Mem0 works with OpenAI, Anthropic, Ollama, and other providers, so the extraction LLM can be a local model.
- **Memory evaluation**: Mem0 ships an evaluation harness (docs.mem0.ai/core-concepts/memory-evaluation) to score extraction quality (precision/recall of extracted facts), which is relevant to validating a chat-extraction pipeline.
- **Apache 2.0**: The license of the open-source Mem0 core library (distinct from the managed cloud offering).

## Implementation guide

To use Mem0 for extracting information from chat logs in a local-first deployment: (1) Install the open-source library (`pip install mem0ai` or the relevant SDK) and configure a local vector store (e.g. Chroma or Qdrant) and a local LLM provider (e.g. Ollama) so no data leaves the machine — satisfying the suite's local-first constraint. (2) Feed chat logs through the `add()` operation: pass the conversation as a list of messages (each with a role and content) and scope it with `user_id` and `agent_id`. Mem0's two-phase pipeline extracts candidate facts and reconciles them against existing memory, so repeated conversations accumulate and update a durable profile rather than duplicating facts. (3) Retrieve with `search()` for context injection into prompts, or `get_all()` for a full profile dump. (4) Use `run_id` to group memories from a single session/run for traceability. (5) Validate extraction quality with Mem0's memory-evaluation harness before relying on the extracted facts. Key design decisions for the suite: keep the extraction LLM and vector store local (constraint D2); expose the same add/search/get_all operations through both the GUI and MCP endpoints (constraint D4); and treat the shared memory store as the interconnection fabric between tools (constraint D3). Because Mem0's core is Apache 2.0 (not AGPL), the suite must decide whether to vendor it as a dependency or wrap it behind its own AGPL service layer — a licensing decision to record in docs/decisions.md.

## Per-project application notes

- **Familiar**: Use Mem0 to extract persistent user identity and preference memory from chat logs — names, roles, communication style, recurring requests. The `user_id` scoping maps directly to Familiar's per-user profile model; a local vector store keeps PII on-premise (D2).
- **Astrographer**: Extract entity and relationship facts from conversational logs about people, places, and organizations, building a durable knowledge graph of entities the user discusses. `agent_id` scoping keeps Astrographer's entity memory separate from other tools' memory.
- **Incanter**: Use Mem0 to remember the user's recurring spell/incantation patterns, preferred output formats, and past invocations so repeated requests are served from memory rather than recomputed. `run_id` groups memories per invocation session for traceability.
- **Horoscope**: Extract the user's birth data, sign, and recurring life-context facts (relationships, career, mood) from chat logs to personalize horoscope output across sessions. Local storage is essential given the sensitive personal data involved (D2).
- **Solomon**: Extract the user's recurring decision contexts, preferences, and past rulings from chat logs so advisory output is consistent and context-aware over time. A shared memory store lets Solomon reference facts learned by other tools (D3).
- **Augur**: Extract predictive/forecast-relevant facts and the user's confidence preferences from chat logs, building a longitudinal record that improves forecast calibration. The shared memory layer is the interconnection point where Augur's learned facts become available to the rest of the suite (D3).

## Sources

- https://docs.mem0.ai/core-concepts/how-it-works
- https://github.com/mem0ai/mem0
- https://arxiv.org/html/2504.19413v1
- https://docs.mem0.ai/api-reference/memory/add-memories
- https://docs.mem0.ai/core-concepts/memory-evaluation
- https://docs.mem0.ai/open-source/overview
