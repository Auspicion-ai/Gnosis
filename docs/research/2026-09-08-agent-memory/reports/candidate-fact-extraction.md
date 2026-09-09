# Candidate fact: lightweight LLM proposes candidate facts, then deterministic TypeScript validation confirms/rejects them

- **runId:** research-candidate-fact-extraction
- **topicId:** candidate-fact-extraction
- **completeness:** complete
- **residualInjectionMarkers:** 0

## Summary

This topic specifies a two-stage fact-ingestion pipeline that feeds a memory-facts-table. Stage 1 is a lightweight LLM that proposes candidate facts in a structured, schema-constrained form (typically JSON emitted via constrained decoding or a structured-output API). Stage 2 is a deterministic TypeScript validation layer — not another LLM — that confirms or rejects each candidate against hard, rule-based checks before the fact is committed to the facts table. The design intent is to separate the generative, fuzzy step (LLM) from the verifiable, reproducible step (code): the LLM is allowed to be creative and propose, while TypeScript validation is the source of truth for what actually gets stored. This mirrors the industry pattern of "LLM proposes, deterministic code disposes," which is the dominant guardrail against hallucinated or malformed facts.

Key validation dimensions include: (a) schema conformance (shape, required fields, types) via a runtime schema library such as Zod; (b) semantic/domain rules (value ranges, allowed enums, cross-field consistency, referential integrity against existing table rows); (c) provenance and grounding checks (does the candidate cite a source, is the source present, is the claim verifiable); and (d) deduplication/conflict resolution against the existing facts table (is this a new fact, a duplicate, or a contradiction of a stored fact). Rejected candidates are either dropped, parked for human review, or fed back to the LLM for a repair/regeneration pass. The pattern is deliberately "lightweight LLM" — a small, cheap, fast model is sufficient because the hard correctness burden is carried by the deterministic validator, not the model. This is a well-established, low-risk architecture that directly supports the memory-facts-table dependency and is implementable in TypeScript with standard tooling (Zod, JSON Schema, a facts-table store).

## State of the art

The "LLM proposes, deterministic validation confirms/rejects" pattern is the current best practice for grounding LLM output and is widely documented. The dominant implementation approach is schema-constrained structured output: modern LLM APIs (OpenAI, Anthropic, Google) support structured-output modes that force the model to emit JSON conforming to a supplied JSON Schema, and TypeScript projects pair this with a runtime validator such as Zod (which can generate JSON Schema from a Zod schema). See TanStack AI's Structured Outputs overview (https://tanstack.com/ai/latest/docs/structured-outputs/overview) and Kyle Kitlinski's guide to getting structured output from LLMs with Zod (https://kkit.dev/blog/structured-output-llms-zod-typescript). Validation is layered: schema-level validation (Zod) is the first gate, then domain/rule-level validation, then grounding/provenance checks. Hallucination detection is an active research area — OpenAI Guardrails provides a hallucination detection check (https://openai.github.io/openai-guardrails-python/ref/checks/hallucination_detection/), and the Chain-of-Verification paper (https://aclanthology.org/2024.findings-acl.212.pdf) shows that having the model verify its own claims reduces hallucination. For knowledge-graph/fact extraction specifically, recent work emphasizes provenance tracking and anchor-constrained extraction: the Grounded Knowledge Graph Extraction paper (https://doi.org/10.3390/computers15030178) and ODKE+ (https://www.arxiv.org/pdf/2509.04696) both couple LLM extraction with deterministic grounding constraints. The "facts vs inferences" provenance pipeline (https://dev.to/hexisteme/how-to-make-an-ai-research-agent-label-facts-vs-inferences-a-deterministic-provenance-pipeline-5dfn) is directly on-topic: it labels extracted claims as facts or inferences using deterministic rules rather than trusting the model. The state of the art therefore converges on: (1) constrained/structured LLM output, (2) runtime schema validation (Zod), (3) deterministic domain + grounding + dedup rules, and (4) a repair-or-reject feedback loop. The "lightweight LLM" choice is well-supported: because the validator carries correctness, a small model suffices, which is cheaper and faster than a large model doing the same job unvalidated.

## Key concepts glossary

- **Candidate fact:** A fact proposed by the LLM in stage 1, not yet committed; it is a proposal pending deterministic confirmation.
- **Deterministic validation:** Rule-based, reproducible checks written in TypeScript (no LLM involved) that confirm or reject a candidate; the same input always yields the same verdict.
- **Memory-facts-table:** The target store (the dependency of this topic) that holds confirmed facts; the validator is the gatekeeper that decides what enters it.
- **Schema conformance:** Checking that a candidate matches a declared shape — required fields, types, enums — typically via a runtime schema library (Zod).
- **Grounding/provenance:** Verifying that a candidate is backed by a cited source and that the claim is traceable to that source, rather than invented.
- **Deduplication/conflict resolution:** Comparing a candidate against existing table rows to detect duplicates or contradictions before insertion.
- **Repair/regeneration loop:** Feeding a rejected candidate back to the LLM with the validation error so it can propose a corrected version, or parking it for human review.
- **Structured output / constrained decoding:** LLM API features that force the model to emit output conforming to a supplied JSON Schema, reducing malformed candidates at the source.
- **Lightweight LLM:** A small, cheap, fast model; sufficient here because the correctness burden is carried by the deterministic validator rather than the model.

## Implementation guide

Implement the two-stage pipeline in TypeScript as follows.

**Stage 1 — Candidate proposal:**
1. Define a Zod schema for a candidate fact (e.g. `{ id, subject, predicate, object, confidence?, sourceRef?, extractedAt }`).
2. Call a lightweight LLM with a prompt that asks it to extract candidate facts from an input (text/document/event) and return them as JSON conforming to the schema; use the LLM's structured-output mode with the JSON Schema generated from the Zod schema (zod-to-json-schema) to minimize malformed output.
3. Parse the LLM response with the Zod schema; a parse failure is itself a rejection (or a retry).

**Stage 2 — Deterministic validation:**
4. Run a chain of pure, testable validator functions, each returning pass/fail plus a reason: (a) schemaValidator (Zod safeParse), (b) domainValidator (enums, value ranges, cross-field consistency, required-source presence), (c) groundingValidator (sourceRef exists and is resolvable; claim is present in the source), (d) dedupValidator (query the facts table for an identical or contradictory row).
5. Combine verdicts: if all pass, commit the fact to the memory-facts-table; if any fail, reject and either drop, park for human review, or send the candidate + error back to the LLM for one repair pass.
6. Keep every validator a pure function with no side effects so the pipeline is unit-testable and deterministic.
7. Record provenance on every committed fact (source, extraction timestamp, validator version) so the table is auditable.

Recommended libraries: Zod for schema validation, zod-to-json-schema for structured-output integration, and a simple typed store (SQLite/Postgres or an in-memory table) for the facts table. The pipeline should be exposed both as a library function and, per the suite's MCP-GUI parity constraint, as an MCP endpoint and a UI action.

## Per-project application notes

- **Familiar:** Use the candidate-fact pipeline to extract relationship and preference facts about people from conversations/contacts; deterministic validation confirms identity fields (names, roles, contact details) and dedups against the existing people facts table before committing.
- **Astrographer:** Apply the pipeline to extract celestial/observation facts (coordinates, magnitudes, timings) from observation logs or imported data; deterministic validation enforces numeric ranges and coordinate-format rules so only well-formed astronomical facts enter the table.
- **Incanter:** Use it to extract spell/ritual/incantation facts from source texts; validation confirms the structured components (name, components, effects, constraints) and rejects malformed or ungrounded entries.
- **Horoscope:** Extract astrological facts (signs, house positions, transit dates) from ephemeris data or text; deterministic validation enforces date/range and sign-enum rules before committing to the facts table.
- **Solomon:** Use the pipeline to extract judgment/decision facts from case material or rulings; validation confirms the structured decision fields and grounds each fact in its cited source before storage.
- **Augur:** Extract predictive/omen facts from readings or inputs; deterministic validation confirms the structured prediction fields and dedups/conflicts against existing stored predictions before committing.

## Sources

- https://tanstack.com/ai/latest/docs/structured-outputs/overview
- https://kkit.dev/blog/structured-output-llms-zod-typescript
- https://openai.github.io/openai-guardrails-python/ref/checks/hallucination_detection/
- https://aclanthology.org/2024.findings-acl.212.pdf
- https://doi.org/10.3390/computers15030178
- https://www.arxiv.org/pdf/2509.04696
- https://dev.to/hexisteme/how-to-make-an-ai-research-agent-label-facts-vs-inferences-a-deterministic-provenance-pipeline-5dfn
- https://learnbackend.com/ai-engineering/prompting-and-structured-output/validating-llm-output/
- https://mepritam.dev/references/constraining-llms-structured-outputs-zod-jsonschema/
