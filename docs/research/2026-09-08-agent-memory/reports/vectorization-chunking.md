# Per-exchange chunking produces many smaller, more precise vectors preserving per-message relationships

## Summary

Per-exchange chunking is a vectorization strategy for conversational data in which a long conversation is not embedded as one monolithic vector (or as fixed-size token windows that cut across turns), but is instead split into discrete chunks aligned to conversational boundaries — typically one chunk per user/assistant exchange (a question–answer pair or a single turn). Each exchange is embedded independently, producing many smaller, more precise vectors. This directly addresses two failure modes of long-vector embedding: (1) long-vector blurriness — when a very long text is compressed into a single fixed-dimension embedding, the model must average/summarize the entire content, so individual details, entities, and the semantic identity of each message get diluted and the vector becomes a low-resolution "blur" of the whole; and (2) relation-loss — when a conversation is chunked by arbitrary token windows, the natural relationship between a user's question and the assistant's answer (and the turn-by-turn dependency chain) is severed, so retrieval returns fragments that lack the context needed to be meaningful. By chunking per exchange, each vector is high-resolution for its own turn, and the exchange boundary preserves the question→answer relationship inside a single retrievable unit. The tradeoff is a larger number of vectors (higher storage and index cost) and the loss of cross-exchange context within a single vector, which must be recovered through retrieval-time strategies such as returning neighboring chunks, hybrid lexical+dense fusion, or hierarchical/aggregation approaches. This is the standard, well-supported approach in conversational-memory RAG systems and is the recommended remedy for the long-vector and relation-loss limitations identified in the parent topics.

## State of the art

The current state of the art for embedding long or conversational content centers on three competing strategies, and per-exchange chunking sits at the intersection of them.

1. Long-context embedding models and Late Chunking. Modern embedding models (e.g., jina-embeddings-v3, and the LongEmbed family) accept long contexts, but research shows that naively embedding a long text as one vector degrades retrieval. [Late Chunking](https://arxiv.org/pdf/2409.04701) (Jina AI) is the leading technique: embed the full long context once with a long-context model, then pool the token-level vectors into chunk-level vectors using the model's own attention, so each chunk vector is "contextualized" by the surrounding text. This preserves cross-chunk context while still producing per-chunk vectors. [LongEmbed](https://arxiv.org/html/2404.12096v3) benchmarks long-context retrieval and shows that naive single-vector embedding of long inputs loses precision. The [jina-ai/late-chunking](https://github.com/jina-ai/late-chunking) repo provides a reference implementation.

2. Chunking-strategy research. [A Systematic Investigation of Document Chunking Strategies and Embedding Sensitivity](https://arxiv.org/html/2603.06976) empirically compares chunking strategies (fixed-size, semantic, recursive, etc.) and shows that chunk granularity materially affects embedding quality and retrieval. The general finding: smaller, semantically-coherent chunks yield higher-precision vectors, at the cost of more vectors and more index entries. [Elasticsearch's chunking-strategies guide](https://www.elastic.co/search-labs/blog/chunking-strategies-elasticsearch) is the practical industry reference for choosing chunk size and overlap.

3. Conversational-memory retrieval. For chat/agent memory specifically, the field has converged on per-exchange or per-turn chunking as the default. [Session-as-RAG](https://insiderllm.com/pdfs/session-as-rag-local-ai-memory.pdf) and the [cx-conversation-embedding-pipeline](https://skills.lc/rulebase-co/rulebase-skills/rulebase-co-rulebase-skills-cx-conversation-embedding-pipeline) skill both chunk conversations by exchange/turn and embed each exchange separately. [Fidelity Before Structure: Verbatim Chunks Beat Lossy Artifact Extraction in Long-Conversation LLM Memory](https://arxiv.org/html/2601.00821) finds that verbatim, boundary-aligned chunks (i.e., per-exchange) outperform lossy summarization for long-conversation memory. [Training-Free Lexical–Dense Fusion for Conversational-Memory Retrieval](https://arxiv.org/html/2606.04194v1) and [Bounded Conversational Memory with Hybrid Retrieval](https://aclanthology.org/2026.sigdial-1.55.pdf) address the cross-exchange context gap by fusing lexical and dense signals at retrieval time rather than by enlarging the chunk. [Beyond RAG for Agent Memory: Retrieval by Decoupling and Aggregation](https://arxiv.org/html/2602.02007) and [On Memory Construction and Retrieval for Personalized Conversational Agents](https://arxiv.org/pdf/2502.05589) explore aggregation of per-turn vectors to reconstruct higher-level memory.

The consensus: per-exchange chunking is the recommended baseline for conversational data because it maximizes per-vector precision and preserves the question→answer relationship, and the remaining limitation (loss of cross-exchange context) is handled at retrieval time (neighbor return, hybrid fusion, aggregation) rather than by enlarging the chunk.

## Key concepts glossary

- **Long-vector blurriness**: The degradation that occurs when a long text is compressed into a single fixed-dimension embedding vector. The model must average/summarize all content, so individual details, entities, and per-message identity are diluted; the vector becomes a low-resolution "blur" of the whole conversation. This is the problem the parent topic `vectorization-long-vector` describes.
- **Relation-loss**: The loss of the natural relationship between messages when a conversation is chunked by arbitrary token windows (or embedded whole). A user's question and the assistant's answer get separated into different vectors, so retrieval returns fragments lacking the context needed to be meaningful. This is the problem the parent topic `vectorization-relation-loss` describes.
- **Per-exchange chunking**: Splitting a conversation into chunks aligned to conversational boundaries — typically one chunk per user/assistant exchange (a question–answer pair or a single turn) — and embedding each chunk independently. Produces many smaller, more precise vectors.
- **Exchange / turn**: A single conversational unit, usually a user message plus the assistant's response (a question–answer pair), or a single message. The natural boundary for conversational chunking.
- **Embedding vector**: A fixed-dimension dense representation of a text produced by an embedding model; used for semantic similarity search.
- **Late Chunking**: A technique (Jina AI) that embeds a long context once with a long-context model, then pools token-level vectors into chunk-level vectors using the model's attention, so each chunk vector is contextualized by surrounding text. An alternative to naive per-exchange independent embedding.
- **Hybrid lexical–dense fusion**: Combining sparse (lexical/BM25) and dense (embedding) retrieval signals at query time to recover cross-exchange context that per-exchange chunking loses.
- **Neighbor return / context windowing**: Returning the chunks adjacent to a retrieved chunk so the surrounding conversation context is available to the LLM.
- **Aggregation / decoupling**: Reconstructing higher-level memory by aggregating multiple per-turn vectors (e.g., clustering or summarizing groups of exchanges).

## Implementation guide

To implement per-exchange chunking for a conversational vector store:

1. Segment the conversation into exchanges. Parse the message stream and group each user message with its assistant response into one exchange unit. If a turn is very long, sub-split it by semantic or sentence boundaries, but keep the exchange as the primary retrieval unit. Preserve metadata: conversation id, exchange index, timestamps, speaker roles.

2. Embed each exchange independently. Call the embedding model once per exchange to produce one vector per exchange. This yields N vectors for N exchanges — many smaller, more precise vectors rather than one blurred vector for the whole conversation.

3. Store with rich metadata. Persist each vector with its exchange id, conversation id, and the raw text. This enables neighbor-return and filtering at retrieval time.

4. Index for retrieval. Use a vector index (e.g., pgvector, Qdrant, Milvus, RedisVL) with the exchange as the indexed unit. Consider a secondary lexical index (BM25) over the same exchanges for hybrid fusion.

5. Recover cross-exchange context at retrieval time. Because each vector only "knows" its own exchange, add one or more of: (a) return the K nearest exchanges plus their immediate neighbors (context windowing); (b) hybrid lexical–dense fusion to catch keyword matches across exchanges; (c) an aggregation layer that clusters or summarizes related exchanges into higher-level memory units.

6. Tune chunk granularity. If exchanges are too small (single short messages), precision per vector is high but context is thin; if too large, blurriness returns. Use the chunking-strategy findings (Elasticsearch guide, systematic chunking study) to set a minimum/maximum token range per exchange and split oversized exchanges.

7. Evaluate. Measure retrieval precision/recall against a conversational benchmark (e.g., LongEmbed-style or a custom Q&A-over-conversation set). Compare per-exchange chunking against whole-conversation embedding and against fixed-window chunking to confirm the precision and relation-preservation gains.

8. Consider Late Chunking as an enhancement. If a long-context embedding model is available, Late Chunking can give per-exchange vectors that are also contextualized by the surrounding conversation, mitigating the cross-exchange context loss while keeping per-exchange granularity.

## Per-project application notes

The projects list is non-empty (Familiar, Astrographer, Incanter, Horoscope, Solomon, Augur), so per-project notes are provided rather than N/A.

- **Familiar** (conversational agent / memory): Per-exchange chunking is the core memory strategy. Each user/assistant exchange becomes a retrievable memory unit, preserving the question→answer relationship. Use neighbor-return and hybrid fusion to recover cross-exchange context. This directly fixes the long-vector blurriness and relation-loss that would otherwise degrade long-session recall.
- **Astrographer** (data/relationship mapping): Apply per-exchange chunking to conversation-derived relationship data so each exchange's entities and relationships are captured at high resolution rather than blurred into a whole-conversation vector. Aggregate per-exchange vectors to build relationship graphs.
- **Incanter** (tool/automation): When embedding tool-call histories or instruction exchanges, chunk per exchange so each tool invocation and its result stays a coherent retrievable unit, preserving the call→result relationship.
- **Horoscope** (personalized content): Chunk per user exchange to keep each user's stated preferences/context in a precise, individually retrievable vector, avoiding blurring across a long history.
- **Solomon** (advisory/decision): Per-exchange chunking preserves the question→answer and decision→rationale relationships in advisory conversations, so retrieval returns coherent advice units rather than fragmented or blurred text.
- **Augur** (prediction/forecast): Chunk per exchange to keep each forecast request and its outcome as a precise retrievable pair, enabling accurate retrieval of past predictions and their results.

All six projects share the same integration pattern: segment by exchange → embed per exchange → store with metadata → recover cross-exchange context at retrieval time (neighbor return + hybrid fusion + optional aggregation).

## Sources

- https://arxiv.org/pdf/2409.04701
- https://arxiv.org/html/2404.12096v3
- https://github.com/jina-ai/late-chunking
- https://arxiv.org/html/2603.06976
- https://www.elastic.co/search-labs/blog/chunking-strategies-elasticsearch
- https://insiderllm.com/pdfs/session-as-rag-local-ai-memory.pdf
- https://skills.lc/rulebase-co/rulebase-skills/rulebase-co-rulebase-skills-cx-conversation-embedding-pipeline
- https://arxiv.org/html/2601.00821
- https://arxiv.org/html/2606.04194v1
- https://aclanthology.org/2026.sigdial-1.55.pdf
- https://arxiv.org/html/2602.02007
- https://arxiv.org/pdf/2502.05589
