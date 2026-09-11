// src/main/retrieval.ts — Unit E: the RAG index + retrieval module
// (docs/specs/unit-e-rag-index.md). Pure, deterministic, no Electron — operates
// on the RagStore INTERFACE (Unit A §5.4), never the concrete JSON store.
//
// Scope: tokenization + the lexical index (§5.1), the interface-swappable
// Embedder + the lexical (BM25) implementation (§5.2), selection (§5.3), graph
// traversal for context assembly (§5.4), the retrieval entry point (§5.5), the
// retrieval engine (§5.6), and the rag.query MCP tool handler (§5.7).
//
// Determinism: no network egress, no randomness. BM25 with fixed defaults
// (k1=1.2, b=0.75); tie-breaking by node id (lexicographic ascending). Same
// query + same store → same result.
import type { RagNode, RagEdge, RagStore } from './rag-store.js'
import { computeDocumentSubgraph } from './traversal.js'
import type { StoreResultInput } from './merge-store-results.js'

// ---------------------------------------------------------------------------
// §5.1 Tokenization + the lexical index
// ---------------------------------------------------------------------------

/** The fixed default stopword set (a module constant). Configurable via
 *  LexicalEmbedderOptions.stopwords. */
export const DEFAULT_STOPWORDS: ReadonlySet<string> = new Set([
  'a', 'an', 'and', 'are', 'as', 'at', 'be', 'been', 'being', 'but', 'by',
  'can', 'did', 'do', 'does', 'for', 'from', 'had', 'has', 'have', 'he', 'her',
  'here', 'hers', 'him', 'his', 'how', 'i', 'if', 'in', 'into', 'is', 'it',
  'its', 'me', 'my', 'no', 'not', 'of', 'on', 'or', 'our', 'ours', 'she',
  'so', 'than', 'that', 'the', 'their', 'theirs', 'them', 'then', 'these',
  'they', 'this', 'those', 'to', 'too', 'us', 'was', 'we', 'were', 'what',
  'when', 'where', 'which', 'while', 'who', 'whom', 'why', 'will', 'with',
  'would', 'you', 'your', 'yours',
])

/** Tokenize text for lexical retrieval. Deterministic: lowercase, split on
 *  non-alphanumeric runs, drop empty tokens, drop stopwords. F6 — the split
 *  boundary is UNICODE-aware (`\p{L}` letters / `\p{N}` numbers, `u` flag), so
 *  non-ASCII letters/numbers are kept as token characters rather than split
 *  into boundaries. ASCII behavior is unchanged (a-z/0-9 are `\p{L}`/`\p{N}`). */
export function tokenize(text: string): string[] {
  if (typeof text !== 'string') throw new Error('tokenize: text must be a string')
  return tokenizeWithStopwords(text, DEFAULT_STOPWORDS)
}

function tokenizeWithStopwords(text: string, stopwords: ReadonlySet<string>): string[] {
  return text
    .toLowerCase()
    .split(/[^\p{L}\p{N}]+/u)
    // Drop empty tokens and stopwords (§5.1 — a stopword is dropped regardless
    // of length; 'a'/'i' are in DEFAULT_STOPWORDS and are dropped too).
    .filter((t) => t !== '' && !stopwords.has(t))
}

/** Unit Q — return a node's FULL searchable text: content + the content of
 *  every inline child (in order), space-joined after dropping empty strings.
 *  Pure + deterministic. Reads the Unit M `children?: RagNodeChild[]` field. */
export function nodeText(node: RagNode): string {
  if (node === null || node === undefined) throw new Error('nodeText: node required')
  return [node.content, ...(node.children ?? []).map((c) => c.content)]
    .filter((s) => s !== '')
    .join(' ')
}

/** The lexical index — the maintained term/document statistics over the RAG
 *  node content. */
export interface LexicalIndex {
  /** The indexed RAG node ids, in insertion order. */
  nodeIds: string[]
  /** Term frequencies per node: nodeId → term → count. */
  termFrequencies: Map<string, Map<string, number>>
  /** Document frequencies: term → number of indexed nodes containing it. */
  documentFrequencies: Map<string, number>
  /** The number of indexed nodes. */
  documentCount: number
  /** The average document length (in tokens) across indexed nodes. */
  averageDocumentLength: number
}

/** Build the index from a node list (boot). */
export function createLexicalIndex(nodes: RagNode[]): LexicalIndex {
  if (nodes === null || nodes === undefined) throw new Error('createLexicalIndex: nodes required')
  const nodeIds: string[] = []
  const termFrequencies = new Map<string, Map<string, number>>()
  const documentFrequencies = new Map<string, number>()
  let totalTokens = 0
  for (const node of nodes) {
    const tf = new Map<string, number>()
    for (const t of tokenize(nodeText(node))) tf.set(t, (tf.get(t) ?? 0) + 1)
    termFrequencies.set(node.id, tf)
    nodeIds.push(node.id)
    for (const [t, c] of tf) {
      documentFrequencies.set(t, (documentFrequencies.get(t) ?? 0) + 1)
      totalTokens += c
    }
  }
  const documentCount = nodes.length
  return {
    nodeIds,
    termFrequencies,
    documentFrequencies,
    documentCount,
    averageDocumentLength: documentCount > 0 ? totalTokens / documentCount : 0,
  }
}

function recomputeAverageDocumentLength(index: LexicalIndex): void {
  let total = 0
  for (const tf of index.termFrequencies.values()) {
    for (const c of tf.values()) total += c
  }
  index.averageDocumentLength = index.documentCount > 0 ? total / index.documentCount : 0
}

/** Incremental content update: re-tokenize the node's new content, replace its
 *  term frequencies, recompute document frequencies for the changed terms, and
 *  recompute the average document length. If the node is NOT in the index, it
 *  is added (same as addToLexicalIndex). */
export function updateLexicalIndex(index: LexicalIndex, node: RagNode): void {
  if (index === null || index === undefined || node === null || node === undefined) {
    throw new Error('lexical index: index/node required')
  }
  if (!index.nodeIds.includes(node.id)) {
    addToLexicalIndex(index, node)
    return
  }
  const oldTF = index.termFrequencies.get(node.id) ?? new Map<string, number>()
  const newTF = new Map<string, number>()
  for (const t of tokenize(nodeText(node))) newTF.set(t, (newTF.get(t) ?? 0) + 1)
  // decrement DF for terms removed from this node
  for (const t of oldTF.keys()) {
    if (!newTF.has(t)) {
      const df = (index.documentFrequencies.get(t) ?? 0) - 1
      if (df <= 0) index.documentFrequencies.delete(t)
      else index.documentFrequencies.set(t, df)
    }
  }
  // increment DF for terms newly added to this node
  for (const t of newTF.keys()) {
    if (!oldTF.has(t)) index.documentFrequencies.set(t, (index.documentFrequencies.get(t) ?? 0) + 1)
  }
  index.termFrequencies.set(node.id, newTF)
  recomputeAverageDocumentLength(index)
}

/** Incremental add: tokenize the node, add its term frequencies, increment the
 *  document frequencies for its terms, increment documentCount, recompute the
 *  average document length. If the node IS already in the index, it is updated
 *  (same as updateLexicalIndex). */
export function addToLexicalIndex(index: LexicalIndex, node: RagNode): void {
  if (index === null || index === undefined || node === null || node === undefined) {
    throw new Error('lexical index: index/node required')
  }
  if (index.nodeIds.includes(node.id)) {
    updateLexicalIndex(index, node)
    return
  }
  const tf = new Map<string, number>()
  for (const t of tokenize(nodeText(node))) tf.set(t, (tf.get(t) ?? 0) + 1)
  index.termFrequencies.set(node.id, tf)
  index.nodeIds.push(node.id)
  for (const t of tf.keys()) index.documentFrequencies.set(t, (index.documentFrequencies.get(t) ?? 0) + 1)
  index.documentCount++
  recomputeAverageDocumentLength(index)
}

/** Incremental remove: remove the node's term frequencies, decrement the
 *  document frequencies for its terms, decrement documentCount, recompute the
 *  average document length. If the node is NOT in the index, it is a no-op. */
export function removeFromLexicalIndex(index: LexicalIndex, nodeId: string): void {
  if (index === null || index === undefined || typeof nodeId !== 'string') {
    throw new Error('lexical index: index/nodeId required')
  }
  const pos = index.nodeIds.indexOf(nodeId)
  if (pos === -1) return // no-op
  const tf = index.termFrequencies.get(nodeId)
  if (tf) {
    for (const t of tf.keys()) {
      const df = (index.documentFrequencies.get(t) ?? 0) - 1
      if (df <= 0) index.documentFrequencies.delete(t)
      else index.documentFrequencies.set(t, df)
    }
  }
  index.termFrequencies.delete(nodeId)
  index.nodeIds.splice(pos, 1)
  index.documentCount--
  recomputeAverageDocumentLength(index)
}

// ---------------------------------------------------------------------------
// §5.2 The Embedder interface + the lexical (BM25) implementation
// ---------------------------------------------------------------------------

/** A scored RAG node. */
export interface ScoredNode {
  nodeId: string
  score: number
}

/** The semantic placement decision — which existing RAG node/edge a new section
 *  attaches to (the "strays from the topic" re-scoping — review §9.3). */
export type PlacementDecision =
  | { ok: true; targetNodeId: string; edgeKind: 'parent-child' | 'doc-child' | 'next-section'; score: number }
  | { ok: false; reason: 'no-match' | 'empty-content' }

/** The interface-swappable scoring engine. The lexical-first implementation
 *  (BM25/tf-idf) is the v1 default; vector embeddings (Unit F) are a drop-in
 *  behind the SAME interface. The Embedder owns the SEMANTIC PLACEMENT
 *  decision. Deterministic (no network egress, no randomness).
 *
 *  **ASYNC (Unit F amendment, 2026-08-27):** the interface is ASYNC — `score`
 *  and `place` return Promises. A vector embedder (Unit F) must compute the
 *  query embedding via an async provider call, so the interface is async to
 *  fit a network-backed embedder. The lexical embedder wraps its synchronous
 *  computation in a resolved promise. The optional `onStoreChanged?` lifecycle
 *  hook lets a stateful embedder (e.g. the vector embedder's vector index)
 *  reconcile its own state on a store change. */
export interface Embedder {
  /** Score all RAG nodes against a query. Returns a ranked list (highest score
   *  first). Deterministic. ASYNC. */
  score(query: string, nodes: RagNode[]): Promise<ScoredNode[]>
  /** The semantic placement decision: given a new section's content, which
   *  existing RAG node/edge it attaches to. ASYNC. */
  place(content: string, nodes: RagNode[], edges: RagEdge[]): Promise<PlacementDecision>
  /** Optional lifecycle hook: reconcile the embedder's own state (e.g. a vector
   *  index) on a store change. The retrieval engine calls it (if present) after
   *  its own index reconciliation (§5.6). ASYNC. */
  onStoreChanged?(kind: 'content' | 'structural', nodeIds: string[], edgeIds: string[]): Promise<void>
  /** RELEASE U-H5 — OPTIONAL forward-only hook: an embedder's resource-release
   *  (e.g. a future vector embedder releasing its memoizer/session). NOT
   *  required — no current embedder implements it (absent = a no-op). Called
   *  by the retrieval engine's `teardown()`. */
  teardown?(): Promise<void>
}

export interface LexicalEmbedderOptions {
  /** BM25 k1 (default 1.2). */
  k1?: number
  /** BM25 b (default 0.75). */
  b?: number
  /** The stopword set (default DEFAULT_STOPWORDS). */
  stopwords?: ReadonlySet<string>
}

/** The placement minimum score — a best score at or below this is `no-match`.
 *  Fixed constant (default 0). */
export const PLACEMENT_MIN_SCORE = 0

/** Module-private marker: the `LexicalIndex` a `createLexicalEmbedder`-created
 *  `Embedder` scores against. The retrieval engine (F2) reads it so it can
 *  share the SAME maintained index with the lexical (v1 default) embedder —
 *  `onStoreChanged` then updates the index the embedder actually references. A
 *  non-lexical drop-in embedder (a vector embedder, Unit F) has no such marker
 *  and the engine maintains its own index (a no-op for scoring — such an
 *  embedder computes scores from the live node content). */
const LEXICAL_INDEX = Symbol('lexical-index')
type LexicalEmbedder = Embedder & { [LEXICAL_INDEX]: LexicalIndex }

/** The lexical-first (BM25) implementation. Holds a reference to the
 *  LexicalIndex (maintained by the retrieval engine — §5.6). */
export function createLexicalEmbedder(index: LexicalIndex, opts?: LexicalEmbedderOptions): Embedder {
  if (index === null || index === undefined) throw new Error('createLexicalEmbedder: index required')
  const k1 = opts?.k1 ?? 1.2
  const b = opts?.b ?? 0.75
  const stopwords = opts?.stopwords ?? DEFAULT_STOPWORDS

  function score(query: string, nodes: RagNode[]): Promise<ScoredNode[]> {
    if (typeof query !== 'string' || nodes === null || nodes === undefined) {
      return Promise.reject(new Error('embedder score: query/nodes required'))
    }
    const qTokens = tokenizeWithStopwords(query, stopwords)
    const N = index.documentCount
    const avgdl = index.averageDocumentLength
    const scored: ScoredNode[] = nodes.map((node) => {
      const tf = index.termFrequencies.get(node.id)
      const docLen = tf ? [...tf.values()].reduce((s, c) => s + c, 0) : 0
      let s = 0
      for (const t of qTokens) {
        const df = index.documentFrequencies.get(t) ?? 0
        if (df === 0) continue
        const idf = Math.log(1 + (N - df + 0.5) / (df + 0.5))
        const tfT = tf?.get(t) ?? 0
        if (tfT === 0) continue
        const lenRatio = avgdl === 0 ? 0 : docLen / avgdl
        const denom = tfT + k1 * (1 - b + b * lenRatio)
        s += idf * ((tfT * (k1 + 1)) / denom)
      }
      return { nodeId: node.id, score: s }
    })
    scored.sort((a, b) => b.score - a.score || (a.nodeId < b.nodeId ? -1 : a.nodeId > b.nodeId ? 1 : 0))
    return Promise.resolve(scored)
  }

  function place(content: string, nodes: RagNode[], edges: RagEdge[]): Promise<PlacementDecision> {
    if (typeof content !== 'string' || nodes === null || nodes === undefined || edges === null || edges === undefined) {
      return Promise.reject(new Error('embedder place: content/nodes/edges required'))
    }
    if (content.trim() === '') return Promise.resolve({ ok: false, reason: 'empty-content' })
    return score(content, nodes).then((scored) => {
      const best = scored[0]
      if (!best || best.score <= PLACEMENT_MIN_SCORE) return { ok: false, reason: 'no-match' }
      const bestNode = nodes.find((n) => n.id === best.nodeId)
      let edgeKind: 'parent-child' | 'doc-child' | 'next-section'
      if (bestNode && (bestNode.type === 'ul' || bestNode.type === 'ol' || bestNode.type === 'div')) {
        edgeKind = 'doc-child'
      } else if (bestNode && (bestNode.type.startsWith('h') || bestNode.type === 'p')) {
        edgeKind = 'next-section'
      } else {
        edgeKind = 'parent-child'
      }
      return { ok: true, targetNodeId: best.nodeId, edgeKind, score: best.score }
    })
  }

  const embedder: Embedder = { score, place }
  // F2 — tag the lexical embedder with the index it references, so the
  // retrieval engine can share it (the engine maintains the index the embedder
  // scores against). The public `Embedder` shape is unchanged.
  ;(embedder as LexicalEmbedder)[LEXICAL_INDEX] = index
  return embedder
}

// ---------------------------------------------------------------------------
// §5.3 Selection (score all, take top-k)
// ---------------------------------------------------------------------------

/** Select the top-k scored RAG nodes. Deterministic. ASYNC (Unit F amendment —
 *  awaits the embedder's async `score`). */
export async function selectTopK(embedder: Embedder, query: string, nodes: RagNode[], k: number): Promise<ScoredNode[]> {
  if (embedder === null || embedder === undefined || typeof query !== 'string' || nodes === null || nodes === undefined) {
    throw new Error('selectTopK: embedder/query/nodes/k required')
  }
  // A non-integer k (1.5) or NaN is a non-positive-integer k → the required error.
  if (typeof k !== 'number' || !Number.isInteger(k)) {
    throw new Error('selectTopK: embedder/query/nodes/k required')
  }
  if (k < 1) throw new Error('selectTopK: k must be a positive integer')
  const scored = await embedder.score(query, nodes)
  scored.sort((a, b) => b.score - a.score || (a.nodeId < b.nodeId ? -1 : a.nodeId > b.nodeId ? 1 : 0))
  return scored.slice(0, k)
}

// ---------------------------------------------------------------------------
// §5.4 Graph traversal for context assembly (bounded)
// ---------------------------------------------------------------------------

/** The coarse line→node map (first-class assembly output): each RAG object in
 *  the context → its line range in the rendered markdown. */
export interface LineNodeMap {
  ranges: Array<{ nodeId: string; startLine: number; endLine: number }>
}

export interface AssemblyOptions {
  /** The maximum number of RAG nodes in the assembled context (default 50). */
  maxNodes: number
  /** The maximum traversal depth from a seed node (default 3). */
  maxDepth: number
}

export interface AssemblyResult {
  /** The assembled context RAG nodes (bounded by maxNodes), in visit order. */
  context: RagNode[]
  /** The rendered markdown of the assembled context. */
  markdown: string
  /** The coarse line→node map (first-class assembly output): each RAG object in
   *  the context → its line range in `markdown`. */
  lineMap: LineNodeMap
  /** Traversal census. */
  traversal: { visited: string[]; depth: number; nodeCount: number }
}

/** Unit Q — render a node's full inline markdown: content + each inline child's
 *  markdown, concatenated DIRECTLY (no auto-inserted separator). A node WITHOUT
 *  children returns content unchanged. Pure + deterministic. */
function renderInlineText(n: RagNode): string {
  let text = n.content
  for (const c of n.children ?? []) {
    // F1 — skip empty-content children (consistent with nodeText's empty-string
    // filter): a `strong` child with content '' must not render `****`, an `em`
    // `**`, an `a` `[]()`, an `img` `![]()`.
    if (c.content === '') continue
    switch (c.type) {
      case 'strong': text += `**${c.content}**`; break
      case 'em':     text += `*${c.content}*`;   break
      // F2 — coerce href/src to string: a non-string value (e.g. `{}`) must not
      // coerce to garbage like `[object Object]`; it renders the empty-URL form.
      case 'a':      text += `[${c.content}](${typeof c.props?.href === 'string' ? c.props.href : ''})`; break
      case 'img':    text += `![${c.content}](${typeof c.props?.src === 'string' ? c.props.src : ''})`; break
    }
  }
  return text
}

function renderNode(n: RagNode): string {
  switch (n.type) {
    case 'h1': return `# ${renderInlineText(n)}`
    case 'h2': return `## ${renderInlineText(n)}`
    case 'h3': return `### ${renderInlineText(n)}`
    case 'h4': return `#### ${renderInlineText(n)}`
    case 'h5': return `##### ${renderInlineText(n)}`
    case 'h6': return `###### ${renderInlineText(n)}`
    case 'li': return `- ${renderInlineText(n)}`
    case 'blockquote': return `> ${renderInlineText(n)}`
    case 'pre': return `\`\`\`\n${renderInlineText(n)}\n\`\`\``
    case 'code': return `\`${renderInlineText(n)}\``
    default: return renderInlineText(n)
  }
}

function buildMarkdown(context: RagNode[]): { markdown: string; ranges: LineNodeMap['ranges'] } {
  const lines: string[] = []
  const ranges: LineNodeMap['ranges'] = []
  for (const n of context) {
    const startLine = lines.length
    const nodeLines = renderNode(n).split('\n')
    lines.push(...nodeLines)
    ranges.push({ nodeId: n.id, startLine, endLine: lines.length - 1 })
    lines.push('')
  }
  while (lines.length > 0 && lines[lines.length - 1] === '') lines.pop()
  return { markdown: lines.join('\n'), ranges }
}

/** Assemble the relevant document context by graph traversal from the top-k
 *  seed nodes. Bounded by maxNodes/maxDepth. Deterministic. */
export function assembleContext(store: RagStore, topK: ScoredNode[], opts: AssemblyOptions): AssemblyResult {
  if (store === null || store === undefined || topK === null || topK === undefined || opts === null || opts === undefined) {
    throw new Error('assembleContext: store/topK/opts required')
  }
  // F4 — `maxNodes` must be a positive INTEGER and `maxDepth` a non-negative
  // INTEGER. A non-numeric / non-integer / fractional bound is rejected (the
  // documented §5.9 fail-state) rather than silently mis-bounding the walk.
  if (
    !Number.isInteger(opts.maxNodes) || opts.maxNodes < 1 ||
    !Number.isInteger(opts.maxDepth) || opts.maxDepth < 0
  ) {
    throw new Error('assembleContext: maxNodes/maxDepth invalid')
  }
  const maxNodes = opts.maxNodes
  const maxDepth = opts.maxDepth
  const edges = store.listEdges()
  const visited = new Set<string>()
  const context: RagNode[] = []
  const visitOrder: string[] = []

  const addNode = (id: string): void => {
    if (visited.has(id) || context.length >= maxNodes) return
    const node = store.getNode(id)
    if (!node) return
    visited.add(id)
    context.push(node)
    visitOrder.push(id)
  }

  // Seeds in rank order (highest score first).
  for (const s of topK) addNode(s.nodeId)

  const neighbors = (nodeId: string): string[] => {
    const out = new Set<string>()
    for (const e of edges) {
      if (e.kind === 'next-section') {
        if (e.source === nodeId) out.add(e.target)
        if (e.target === nodeId) out.add(e.source) // backward: the node whose next-section targets this node
      } else if (e.kind === 'parent-child' || e.kind === 'doc-child') {
        if (e.source === nodeId) out.add(e.target)
        if (e.target === nodeId) out.add(e.source)
      } else if (e.kind === 'doc-head' || e.kind === 'doc-end') {
        if (e.source === nodeId) out.add(e.target) // anchor the document: include the head/end node
      }
    }
    return [...out].sort()
  }

  // BFS: level 0 = the seeds. At each level, expand each node's neighbors (in
  // sorted-by-node-id order), adding them if not already visited and if
  // maxNodes is not exceeded. Stop when the current level's expansion would
  // exceed maxNodes, or when maxDepth levels have been processed.
  let depth = 0
  let currentLevel = visitOrder.slice()
  while (currentLevel.length > 0 && depth < maxDepth) {
    const nextLevel: string[] = []
    for (const nodeId of currentLevel) {
      for (const nb of neighbors(nodeId)) {
        if (!visited.has(nb) && context.length < maxNodes) {
          addNode(nb)
          if (visited.has(nb)) nextLevel.push(nb)
        }
      }
    }
    currentLevel = nextLevel
    if (nextLevel.length > 0) depth++
  }

  const { markdown, ranges } = buildMarkdown(context)
  return {
    context,
    markdown,
    lineMap: { ranges },
    traversal: { visited: visitOrder, depth, nodeCount: context.length },
  }
}

// ---------------------------------------------------------------------------
// §5.5 The retrieval entry point
// ---------------------------------------------------------------------------

export interface RetrievalOptions {
  /** The top-k to select (default 5). */
  k?: number
  /** The context assembly bound (default 50). */
  maxNodes?: number
  /** The context assembly depth bound (default 3). */
  maxDepth?: number
}

export interface RetrievalResult {
  query: string
  /** The top-k ranked RAG nodes (highest score first). */
  ranked: ScoredNode[]
  /** The assembled context RAG nodes (bounded). */
  context: RagNode[]
  /** The rendered markdown of the assembled context. */
  markdown: string
  /** The coarse line→node map (first-class assembly output). */
  lineMap: LineNodeMap
  /** The k used. */
  k: number
}

/** The retrieval entry point: select top-k, then assemble context by graph
 *  traversal. Deterministic. ASYNC (Unit F amendment — awaits the embedder's
 *  async `score` via `selectTopK`). */
export async function retrieve(
  store: RagStore,
  embedder: Embedder,
  index: LexicalIndex,
  query: string,
  opts: RetrievalOptions,
): Promise<RetrievalResult> {
  if (
    store === null || store === undefined ||
    embedder === null || embedder === undefined ||
    index === null || index === undefined ||
    typeof query !== 'string' ||
    opts === null || opts === undefined
  ) {
    throw new Error('retrieve: store/embedder/index/query/opts required')
  }
  if (query.trim() === '') throw new Error('retrieve: query must be a non-empty string')
  // F4 — the zero-token (stopword-only) check is LEXICAL-specific: a lexical
  // embedder cannot score a query that tokenizes to zero tokens (the documented
  // empty-query fail-state — Unit E F5). A vector embedder CAN embed a
  // stopword-only query (e.g. 'the'), so the check is gated on the lexical
  // embedder (detected via the LEXICAL_INDEX marker).
  const isLexical = (embedder as { [LEXICAL_INDEX]?: unknown })[LEXICAL_INDEX] !== undefined
  if (isLexical && tokenize(query).length === 0) throw new Error('retrieve: query must be a non-empty string')
  const k = opts.k ?? 5
  if (typeof k !== 'number' || !Number.isInteger(k) || k < 1) {
    throw new Error('retrieve: k must be a positive integer')
  }
  const maxNodes = opts.maxNodes ?? 50
  const maxDepth = opts.maxDepth ?? 3
  // The ranked result excludes irrelevant (score ≤ 0) nodes — a retrieval
  // returns only nodes that actually match the query. (selectTopK itself still
  // returns the full scored list including score-0 nodes when k exceeds the
  // node count — §5.3 test 12.)
  const ranked = (await selectTopK(embedder, query, store.listNodes(), k)).filter((s) => s.score > 0)
  const assembled = assembleContext(store, ranked, { maxNodes, maxDepth })
  return {
    query,
    ranked,
    context: assembled.context,
    markdown: assembled.markdown,
    lineMap: assembled.lineMap,
    k,
  }
}

// ---------------------------------------------------------------------------
// §5.6 The retrieval engine (index lifecycle + MCP/UI routing)
// ---------------------------------------------------------------------------

export interface RetrievalEngine {
  /** Run a retrieval query. Returns the extended `RagResult` (the preserved
   *  ranked/context/markdown/lineMap/k fields + the Unit X results/engine/
   *  citations/trace/blockedBy surface). ASYNC (Unit F amendment — awaits the
   *  embedder's async `score`). The existing `{ k }` calls still work (mapped
   *  to `topK`). */
  query(
    query: string,
    opts?: {
      k?: number
      mode?: 'flat' | 'graph'
      maxHops?: number
      expand?: 'none' | 'parent'
      maxParentContext?: number
      filters?: RagQueryFilters
    },
  ): Promise<RagResult>
  /** Update the index on a store change (content or structural). ASYNC (Unit F
   *  amendment — forwards to the embedder's `onStoreChanged` hook, if present). */
  onStoreChanged(kind: 'content' | 'structural', nodeIds: string[], edgeIds: string[]): Promise<void>
  /** W1 (§5.12) — atomic ONE-WAY embedder promotion: replaces the active
   *  embedder binding so the SAME engine instance serves lexical pre-swap and
   *  vector post-swap. Every query observes exactly ONE embedder (an in-flight
   *  pre-swap query completes on the OLD embedder). Throws on a second call
   *  (one-way) or a null/undefined embedder. */
  setEmbedder(embedder: Embedder): void
  /** RELEASE U-H5 — tear the engine down: mark STOPPED (query/onStoreChanged/
   *  setEmbedder now throw `retrieval engine: torn down`), forward to
   *  `activeEmbedder.teardown?.()`, resolve with `undefined`. IDEMPOTENT.
   *  NO-FAIL. Does NOT cancel an in-flight query (the U-H4 drain first). */
  teardown(): Promise<void>
  /** DRAIN-SEAM U-H5 (A-P2-2 check) — the count of currently-unsettled
   *  `query()` calls (incremented on entry, decremented on settle). The U-H4
   *  drain-then-teardown reads it and awaits 0 BEFORE tearing down. A stopped
   *  engine reports its residual unresolved count (truthful). */
  inFlight(): number
}

/** Create the retrieval engine. Builds the index from the store on
 *  construction; maintains it on store changes. F2 — the passed `embedder` IS
 *  used (the interface-swappable seam). For a lexical embedder the engine
 *  shares the SAME index the embedder references, so `onStoreChanged` keeps the
 *  index the embedder scores against consistent (the tests observe the index
 *  via the engine's query). For any other drop-in embedder (no exposed index)
 *  the engine maintains its own index. */
export function createRetrieval(store: RagStore, embedder: Embedder, opts?: RetrievalOptions): RetrievalEngine {
  if (store === null || store === undefined || embedder === null || embedder === undefined) {
    throw new Error('createRetrieval: store/embedder required')
  }
  // F2 — the passed `embedder` IS used (the interface-swappable seam: a vector
  // embedder is a drop-in behind the same interface). For the lexical (v1
  // default) embedder the engine SHARES the index the embedder references
  // (LEXICAL_INDEX) so `onStoreChanged` keeps the index the embedder scores
  // against consistent. For any other embedder (no exposed index) the engine
  // maintains its own index (a no-op for scoring — such an embedder derives
  // scores from the live node content passed to `score`).
  const shared = (embedder as { [LEXICAL_INDEX]?: LexicalIndex })[LEXICAL_INDEX]
  const index = shared ?? createLexicalIndex(store.listNodes())
  const maxNodes = opts?.maxNodes ?? 50
  const maxDepth = opts?.maxDepth ?? 3
  // W1 (§5.12) — the ACTIVE embedder binding, read by `query` and by the
  // `onStoreChanged` hook forward at each call. `setEmbedder` reassigns it
  // (a single assignment): an in-flight pre-swap query holds the OLD embedder
  // (passed by value into `retrieve`), post-swap queries observe the new one.
  // The engine's OWN lexical `index` maintenance is UNCHANGED and continues in
  // both phases.
  let activeEmbedder = embedder
  let promoted = false
  // U-H5 — the STOPPED latch + the unsettled-query counter (the drain seam).
  let stopped = false
  let inFlightCount = 0

  return {
    query(
      query: string,
      qopts?: {
        k?: number
        mode?: 'flat' | 'graph'
        maxHops?: number
        expand?: 'none' | 'parent'
        maxParentContext?: number
        filters?: RagQueryFilters
      },
    ): Promise<RagResult> {
      // U-H5 — a NEW query on a torn-down engine fails loud (F4). A query that
      // entered BEFORE teardown (already past this check) is NOT cancelled and
      // settles on the OLD embedder (F8).
      if (stopped) throw new Error('retrieval engine: torn down')
      // DRAIN-SEAM — increment on entry, decrement on settle (resolve OR reject).
      inFlightCount++
      // Unit X — the extended retrieval entry point. Uses the MAINTAINED
      // `index`/`activeEmbedder` (F1 — no per-call rebuild) and returns the
      // extended `RagResult`. The existing `{ k }` calls map to `topK`.
      // `ragQuery` captures `activeEmbedder` by value here, so an in-flight
      // pre-teardown query keeps scoring against the OLD embedder (F8).
      return ragQuery(store, activeEmbedder, index, query, {
        topK: qopts?.k,
        mode: qopts?.mode,
        maxHops: qopts?.maxHops,
        expand: qopts?.expand,
        maxParentContext: qopts?.maxParentContext,
        filters: qopts?.filters,
      }).then(
        (result) => { inFlightCount--; return result },
        (err) => { inFlightCount--; throw err },
      )
    },
    async onStoreChanged(kind: 'content' | 'structural', nodeIds: string[], edgeIds: string[]): Promise<void> {
      // U-H5 — a torn-down engine no longer serves store changes (F5).
      if (stopped) throw new Error('retrieval engine: torn down')
      if (nodeIds === null || nodeIds === undefined) throw new Error('onStoreChanged: nodeIds required')
      // edgeIds is accepted and ignored for index purposes (edges are not indexed).
      for (const nodeId of nodeIds) {
        const node = store.getNode(nodeId)
        if (node) {
          if (index.nodeIds.includes(nodeId)) updateLexicalIndex(index, node)
          else addToLexicalIndex(index, node)
        } else if (index.nodeIds.includes(nodeId)) {
          removeFromLexicalIndex(index, nodeId)
        }
      }
      // Unit F amendment — forward to the embedder's onStoreChanged hook (if
      // present) so a stateful embedder (e.g. the vector embedder's vector
      // index) reconciles its own state on the same store change.
      await activeEmbedder.onStoreChanged?.(kind, nodeIds, edgeIds)
    },
    // W1 (§5.12) — atomic ONE-WAY embedder promotion. A second call throws
    // (one-way); a null/undefined embedder throws and consumes nothing.
    setEmbedder(next: Embedder): void {
      // U-H5 — a torn-down engine can no longer be promoted (F6).
      if (stopped) throw new Error('retrieval engine: torn down')
      if (next === null || next === undefined) throw new Error('retrieval engine: embedder required')
      // F-W1-3 (RCA-3) — the STRUCTURAL guard: a present-but-invalid embedder
      // (e.g. `{} as Embedder`) must be rejected with the SAME pinned message
      // WITHOUT consuming the one-way latch (promoted stays false) — otherwise
      // the promotion latches a broken embedder and the first query throws
      // TypeError with no recovery.
      if (typeof next.score !== 'function' || typeof next.place !== 'function') {
        throw new Error('retrieval engine: embedder required')
      }
      if (promoted) throw new Error('retrieval engine: embedder promotion is one-way (already promoted)')
      activeEmbedder = next
      promoted = true
    },
    // ---- teardown + the drain seam (Unit U-H5) -----------------------------
    // Mark STOPPED (revoke future service calls), forward the optional
    // embedder teardown hook, resolve `undefined`. IDEMPOTENT and NO-FAIL: an
    // embedder hook that rejects is swallowed (best-effort release). Does NOT
    // cancel an in-flight query (F8 — the U-H4 drain guarantees idleness).
    async teardown(): Promise<void> {
      if (stopped) return undefined // idempotent NO-OP
      stopped = true
      try {
        await activeEmbedder.teardown?.()
      } catch {
        // no-fail — the STOPPED state is the binding contract.
      }
      return undefined
    },
    // The unsettled-query count (resolve OR reject decrements).
    inFlight(): number {
      return inFlightCount
    },
  }
}

// ---------------------------------------------------------------------------
// Unit X — RAG-surface provenance + multi-hop traversal
// (docs/specs/unit-x-rag-provenance-traversal.md §5.2–§5.6)
// ---------------------------------------------------------------------------

/** The snippet cap — a node's `content` is truncated to this length for the
 *  `snippet` field (default 200). */
export const SNIPPET_MAX_LENGTH = 200

/** The engine id — the current Astrographer value is 'local' (the contract's
 *  'incanter' is the Auspicion Suite's framing — F4). */
export const RAG_ENGINE_ID = 'local'

/** The pinned filters shape (A2). */
export interface RagQueryFilters {
  nodeKind?: 'content' | 'fact' | 'reference'
  edgeType?: 'link' | 'embed'
  target?: { documentId: string; nodeId: string }
  state?: 'FRESH' | 'RESOLVED' | 'STALE' | 'BROKEN'
}

/** The per-result item (the contract's `results` array element). */
export interface RagResultItem {
  documentId: string
  nodeId: string
  score: number
  snippet: string
  source: 'local' | 'incanter' | 'zodiac'
  parent?: { documentId: string; title: string; snippet: string; stale: boolean }
  /** ADDITIVE — the producing store's registry name. Present ONLY in a
   *  qualified (`stores:"all"`) result (A-F1). */
  store?: string
}

/** The flat-mode trace (A1). */
export interface FlatTrace {
  mode: 'flat'
  engine: string
  topK: number
  source: 'local' | 'incanter' | 'zodiac'
}

/** The graph-mode trace entry (A1). */
export interface GraphTraceEntry {
  from: { documentId: string; nodeId: string }
  to: { documentId: string; nodeId: string }
  edge: 'link' | 'embed'
  state: 'FRESH' | 'RESOLVED' | 'STALE' | 'BROKEN'
  /** ADDITIVE — type-shape uniformity only; NEVER stamped by the qualified FLAT
   *  builder (the fan-out is FLAT-only, D4). */
  store?: string
}

/** The trace — per-mode (A1). */
export type RagTrace = FlatTrace | GraphTraceEntry[]

/** The blocked-by entry (graph mode, empty result — a valid state, not an
 *  error). */
export interface BlockedByEntry {
  documentId: string
  nodeId: string
  state: 'BROKEN' | 'STALE'
  /** ADDITIVE — type-shape uniformity only; NEVER stamped by the qualified FLAT
   *  builder (the fan-out is FLAT-only, D4). */
  store?: string
}

/** The extended RAG query options (A1 + A2). */
export interface RagQueryOptions {
  wikiId?: string
  topK?: number
  filters?: RagQueryFilters
  mode?: 'flat' | 'graph'
  maxHops?: number
  expand?: 'none' | 'parent'
  maxParentContext?: number
}

/** The extended RAG result (A1 + A2). The existing RetrievalResult fields
 *  (ranked/context/markdown/lineMap/k) are PRESERVED — a backward-compatible
 *  additive extension. */
export interface RagResult {
  query: string
  /** The contract's per-result items (A1/A2). */
  results: RagResultItem[]
  /** The engine identifier. The current Astrographer value is 'local'. */
  engine: string
  /** The deduplicated grounding set (A1). */
  citations: Array<{ documentId: string; nodeId: string; store?: string }>
  /** The per-mode trace (A1). */
  trace: RagTrace
  /** Present ONLY in graph mode when the traversal resolves no target (A2). */
  blockedBy?: BlockedByEntry[]
  // ---- the preserved existing surface (backward-compatible) ----
  ranked: ScoredNode[]
  context: RagNode[]
  markdown: string
  lineMap: LineNodeMap
  k: number
  /** D2/A-F1 — per-store context blocks. Present ONLY in a qualified
   *  (`stores:"all"`) result; the top-level context/markdown/lineMap stay the
   *  default store's block. Absent (undefined) in a single-store result. */
  storeContexts?: StoreContextBlock[]
}

/** The graph-mode walk options (§5.4). */
export interface WalkOptions {
  maxHops: number
  filters?: RagQueryFilters
}

/** The graph-mode walk result (§5.4). */
export interface WalkResult {
  /** The resolved target nodes the traversal reached (deduped by
   *  (documentId, nodeId)). */
  targets: RagResultItem[]
  /** The ordered reference→fact path walked (the graph-mode trace). */
  trace: GraphTraceEntry[]
  /** Present when the traversal resolves no target (a valid state, not an
   *  error). */
  blockedBy?: BlockedByEntry[]
}

/** The node's `content` truncated to `SNIPPET_MAX_LENGTH` (200); empty content
 *  → `''`. Deterministic. */
function snippetOf(node: RagNode): string {
  return node.content.slice(0, SNIPPET_MAX_LENGTH)
}

/** The owning document id for a node (the first of `documentIdsForNode`,
 *  sorted ascending), or `''` when the node belongs to no document. */
function docForNode(store: RagStore, nodeId: string): string {
  return documentIdsForNode(store, nodeId)[0] ?? ''
}

/** Derive the document id(s) for a node: the document(s) whose docNodeIds (via
 *  computeDocumentSubgraph) include the node. Returns [] if the node belongs to
 *  no document. Deterministic (sorted ascending). */
export function documentIdsForNode(store: RagStore, nodeId: string): string[] {
  if (store == null || typeof nodeId !== 'string') {
    throw new Error('documentIdsForNode: store/nodeId required')
  }
  const docIds: string[] = []
  const seen = new Set<string>()
  for (const e of store.edgesByKind('doc-head')) {
    for (const d of e.documentIds ?? []) {
      if (seen.has(d)) continue
      seen.add(d)
      const { docNodeIds } = computeDocumentSubgraph(store, d)
      if (docNodeIds.has(nodeId)) docIds.push(d)
    }
  }
  return docIds.sort()
}

/** Build the deduplicated grounding set from the result items. Order is by
 *  first appearance in the result set; duplicates (same documentId+nodeId) are
 *  removed. */
export function buildCitations(items: RagResultItem[]): Array<{ documentId: string; nodeId: string }> {
  if (items == null) throw new Error('buildCitations: items required')
  const out: Array<{ documentId: string; nodeId: string }> = []
  const seen = new Set<string>()
  for (const item of items) {
    // F-X-5 — skip a null element (a malformed result item must not throw an
    // unpinned TypeError; it is simply not cited).
    if (item == null) continue
    const key = `${item.documentId}\u0000${item.nodeId}`
    if (seen.has(key)) continue
    seen.add(key)
    out.push({ documentId: item.documentId, nodeId: item.nodeId })
  }
  return out
}

/** Build the flat-mode trace. */
export function buildFlatTrace(engine: string, topK: number, source: 'local' | 'incanter' | 'zodiac'): FlatTrace {
  if (typeof engine !== 'string' || typeof source !== 'string' || typeof topK !== 'number' || !Number.isInteger(topK) || topK < 1) {
    throw new Error('buildFlatTrace: engine/topK/source required')
  }
  return { mode: 'flat', engine, topK, source }
}

/** True when an edge passes the walk's edge filters (edgeType/state/target). */
function edgePassesFilters(store: RagStore, e: RagEdge, filters: RagQueryFilters | undefined): boolean {
  if (!filters) return true
  if (filters.edgeType !== undefined && e.edgeType !== filters.edgeType) return false
  if (filters.state !== undefined && e.state !== filters.state) return false
  if (filters.target !== undefined && (e.target !== filters.target.nodeId || docForNode(store, e.target) !== filters.target.documentId)) return false
  return true
}

/** Walk the reference→fact graph from the seed nodes, hop-limited, resolving
 *  through FRESH/RESOLVED and surfacing BROKEN/STALE. Deterministic. */
export function walkReferenceGraph(store: RagStore, seeds: RagResultItem[], opts: WalkOptions): WalkResult {
  if (store == null || seeds == null || opts == null) {
    throw new Error('walkReferenceGraph: store/seeds/opts required')
  }
  const maxHops = opts.maxHops
  if (typeof maxHops !== 'number' || !Number.isInteger(maxHops) || maxHops < 1 || maxHops > 5) {
    throw new Error('walkReferenceGraph: maxHops must be an integer in [1, 5]')
  }
  const filters = opts.filters

  const targets: RagResultItem[] = []
  const trace: GraphTraceEntry[] = []
  const blockedBy: BlockedByEntry[] = []
  const seenTargets = new Set<string>()

  const addTarget = (item: RagResultItem): void => {
    const key = `${item.documentId}\u0000${item.nodeId}`
    if (seenTargets.has(key)) return
    seenTargets.add(key)
    targets.push(item)
  }

  for (const seed of seeds) {
    const path = [seed.nodeId]
    const pathSet = new Set<string>([seed.nodeId])
    let current = seed.nodeId
    let hops = 0
    let resolved = false

    // A seed that is itself a fact node is a resolved target.
    const seedNode = store.getNode(current)
    if (seedNode && seedNode.nodeKind === 'fact') {
      if (filters?.nodeKind === undefined || filters.nodeKind === 'fact') {
        addTarget(seed)
        resolved = true
      }
    }

    while (true) {
      // F-X-4 (refined) — traverse a crosslink-kind edge whose target is a
      // `reference` OR `fact` node (a `content` target is NON-traversable).
      // A `reference` target is a WAYPOINT (not a resolved target) — this is
      // what makes `HopLimitExceeded` (a reference chain longer than maxHops)
      // and `CycleDetected` (a reference→fact→reference cycle) reachable.
      const edges = store.edgesFrom(current).filter(
        (e) => e.kind === 'crosslink' && edgePassesFilters(store, e, filters) && (store.getNode(e.target)?.nodeKind === 'fact' || store.getNode(e.target)?.nodeKind === 'reference'),
      )
      if (edges.length === 0) {
        // No traversable edge — the walk cannot proceed. If no target was
        // resolved, the traversal is blocked (a valid state, not an error).
        // F-X-2 — `blockedBy` is present ONLY when the traversal resolves NO
        // target (spec §5.4/§5.2): guard on `targets.length === 0`, not the
        // per-seed `resolved` flag.
        if (targets.length === 0) {
          blockedBy.push({ documentId: docForNode(store, current), nodeId: current, state: 'BROKEN' })
        }
        break
      }
      const e = edges[0]
      const state = e.state ?? 'RESOLVED'
      const edgeType = e.edgeType ?? 'link'
      if (state === 'BROKEN' || state === 'STALE') {
        // A BROKEN/STALE edge is NOT traversed; it is recorded in the trace.
        trace.push({ from: { documentId: docForNode(store, e.source), nodeId: e.source }, to: { documentId: docForNode(store, e.target), nodeId: e.target }, edge: edgeType, state })
        // F-X-2 — `blockedBy` is present ONLY when the traversal resolves NO
        // target (spec §5.4/§5.2): a BROKEN/STALE edge encountered AFTER a
        // target was already resolved must NOT populate `blockedBy`.
        if (targets.length === 0) {
          blockedBy.push({ documentId: docForNode(store, e.source), nodeId: e.source, state })
        }
        break
      }
      // F-X-1 — `HopLimitExceeded` fires ONLY when the walk exceeds `maxHops`
      // AND no target was resolved (spec §5.4: "exceeds maxHops WITHOUT
      // resolving a target"). A chain of exactly `maxHops` hops that resolves a
      // target must NOT throw on the next iteration.
      if (hops > maxHops && !resolved) throw new Error('walkReferenceGraph: HopLimitExceeded')
      hops++
      trace.push({ from: { documentId: docForNode(store, e.source), nodeId: e.source }, to: { documentId: docForNode(store, e.target), nodeId: e.target }, edge: edgeType, state })
      const target = e.target
      if (pathSet.has(target)) throw new Error('walkReferenceGraph: CycleDetected')
      path.push(target)
      pathSet.add(target)
      current = target
      const targetNode = store.getNode(target)
      if (targetNode && targetNode.nodeKind === 'fact') {
        if (filters?.nodeKind === undefined || filters.nodeKind === 'fact') {
          addTarget({ documentId: docForNode(store, target), nodeId: target, score: seed.score, snippet: snippetOf(targetNode), source: 'local' as const })
          resolved = true
        }
      }
    }
  }

  return { targets, trace, blockedBy: blockedBy.length > 0 ? blockedBy : undefined }
}

/** The parent Document/node-cluster context for a retrieved child node, or
 *  `undefined` when the child has no owning document. Stale-propagating: a
 *  child's `parent.stale` is `true` if ANY incoming `reference`→`fact` edge to
 *  the child has `state: 'STALE'`. */
function parentFor(store: RagStore, item: RagResultItem): { documentId: string; title: string; snippet: string; stale: boolean } | undefined {
  const docIds = documentIdsForNode(store, item.nodeId)
  if (docIds.length === 0) return undefined
  const documentId = docIds[0]
  const headEdge = store.edgesByKind('doc-head').find((e) => (e.documentIds ?? []).includes(documentId))
  const headNode = headEdge ? store.getNode(headEdge.source) : undefined
  const title = headNode ? headNode.content : ''
  const snippet = headNode ? headNode.content.slice(0, SNIPPET_MAX_LENGTH) : ''
  const stale = store.edgesTo(item.nodeId).some(
    (e) => e.kind === 'crosslink' && e.state === 'STALE' && store.getNode(e.source)?.nodeKind === 'reference',
  )
  return { documentId, title, snippet, stale }
}

/** Expand the top-capped result items with their parent Document/node-cluster
 *  context. Capped by maxParentContext; stale-propagating. Deterministic. */
export function expandParentContext(store: RagStore, items: RagResultItem[], maxParentContext: number): RagResultItem[] {
  if (store == null || items == null) throw new Error('expandParentContext: store/items required')
  if (typeof maxParentContext !== 'number' || !Number.isInteger(maxParentContext) || maxParentContext < 1) {
    throw new Error('expandParentContext: maxParentContext must be a positive integer')
  }
  return items.map((item, i) => {
    // F-X-5 — skip a null element (a malformed result item must not throw an
    // unpinned TypeError; it is returned unchanged).
    if (item == null) return item
    if (i >= maxParentContext) return item
    const parent = parentFor(store, item)
    if (!parent) return item
    return { ...item, parent }
  })
}

/** Validate the pinned filters shape (§5.2). A malformed filters object throws
 *  `Error('ragQuery: filters malformed')`. */
function validateFilters(filters: RagQueryFilters): void {
  if (filters === null || typeof filters !== 'object' || Array.isArray(filters)) {
    throw new Error('ragQuery: filters malformed')
  }
  if (filters.nodeKind !== undefined && !['content', 'fact', 'reference'].includes(filters.nodeKind)) {
    throw new Error('ragQuery: filters malformed')
  }
  if (filters.edgeType !== undefined && !['link', 'embed'].includes(filters.edgeType)) {
    throw new Error('ragQuery: filters malformed')
  }
  if (filters.state !== undefined && !['FRESH', 'RESOLVED', 'STALE', 'BROKEN'].includes(filters.state)) {
    throw new Error('ragQuery: filters malformed')
  }
  if (filters.target !== undefined) {
    if (filters.target === null || typeof filters.target !== 'object' || Array.isArray(filters.target)) {
      throw new Error('ragQuery: filters malformed')
    }
    if (typeof filters.target.documentId !== 'string' || typeof filters.target.nodeId !== 'string') {
      throw new Error('ragQuery: filters malformed')
    }
  }
}

/** The extended retrieval entry point (A1 + A2). Selects the top-k, then either
 *  (flat mode) assembles the context + builds the flat trace, or (graph mode)
 *  walks the reference→fact graph + builds the graph trace. Applies the filters
 *  and the parent-context expansion. ASYNC. */
export async function ragQuery(
  store: RagStore,
  embedder: Embedder,
  index: LexicalIndex,
  query: string,
  opts: RagQueryOptions,
): Promise<RagResult> {
  if (store == null || embedder == null || index == null || typeof query !== 'string' || opts == null) {
    throw new Error('ragQuery: store/embedder/index/query/opts required')
  }
  if (query.trim() === '') throw new Error('ragQuery: query must be a non-empty string')

  const topK = opts.topK ?? 5
  if (typeof topK !== 'number' || !Number.isInteger(topK) || topK < 1 || topK > 50) {
    throw new Error('ragQuery: topK must be an integer in [1, 50]')
  }

  const mode = opts.mode ?? 'flat'
  if (mode !== 'flat' && mode !== 'graph') throw new Error('ragQuery: mode must be "flat" or "graph"')

  const maxHops = opts.maxHops ?? 3
  if (typeof maxHops !== 'number' || !Number.isInteger(maxHops) || maxHops < 1 || maxHops > 5) {
    throw new Error('ragQuery: maxHops must be an integer in [1, 5]')
  }

  const expand = opts.expand ?? 'none'
  if (expand !== 'none' && expand !== 'parent') throw new Error('ragQuery: expand must be "none" or "parent"')

  const maxParentContext = opts.maxParentContext ?? 5
  if (typeof maxParentContext !== 'number' || !Number.isInteger(maxParentContext) || maxParentContext < 1) {
    throw new Error('ragQuery: maxParentContext must be a positive integer')
  }

  if (opts.filters !== undefined) validateFilters(opts.filters)

  const ranked = (await selectTopK(embedder, query, store.listNodes(), topK)).filter((s) => s.score > 0)

  let results: RagResultItem[]
  let trace: RagTrace
  let blockedBy: BlockedByEntry[] | undefined
  let context: RagNode[]
  let markdown: string
  let lineMap: LineNodeMap

  if (mode === 'flat') {
    const assembled = assembleContext(store, ranked, { maxNodes: 50, maxDepth: 3 })
    context = assembled.context
    markdown = assembled.markdown
    lineMap = assembled.lineMap
    results = ranked.map((s) => {
      const node = store.getNode(s.nodeId)
      return {
        documentId: docForNode(store, s.nodeId),
        nodeId: s.nodeId,
        score: s.score,
        snippet: node ? snippetOf(node) : '',
        source: 'local' as const,
      }
    })
    if (opts.filters?.nodeKind !== undefined) {
      results = results.filter((r) => store.getNode(r.nodeId)?.nodeKind === opts.filters!.nodeKind)
    }
    trace = buildFlatTrace(RAG_ENGINE_ID, topK, 'local')
  } else {
    const seeds = ranked.map((s) => {
      const node = store.getNode(s.nodeId)
      return {
        documentId: docForNode(store, s.nodeId),
        nodeId: s.nodeId,
        score: s.score,
        snippet: node ? snippetOf(node) : '',
        source: 'local' as const,
      }
    })
    const walk = walkReferenceGraph(store, seeds, { maxHops, filters: opts.filters })
    results = walk.targets
    trace = walk.trace
    blockedBy = walk.blockedBy
    const targetScored = walk.targets.map((t) => ({ nodeId: t.nodeId, score: t.score }))
    const assembled = assembleContext(store, targetScored, { maxNodes: 50, maxDepth: 3 })
    context = assembled.context
    markdown = assembled.markdown
    lineMap = assembled.lineMap
  }

  if (expand === 'parent') {
    results = expandParentContext(store, results, maxParentContext)
  }

  const citations = buildCitations(results)

  // TraceUnavailable (FS-16) — a defensive fail-state: a valid result always
  // carries a trace.
  if (trace == null) throw new Error('ragQuery: TraceUnavailable — result has no trace')

  return {
    query,
    results,
    engine: RAG_ENGINE_ID,
    citations,
    trace,
    blockedBy,
    ranked,
    context,
    markdown,
    lineMap,
    k: topK,
  }
}

// ---------------------------------------------------------------------------
// Unit F2 — result qualification + `storeContexts` (cross-store fan-out)
// (docs/specs/unit-f2-result-qualification.md §5.3–§5.7)
// ---------------------------------------------------------------------------

/** One per-store context block (D2). */
export interface StoreContextBlock {
  store: string // the producing store's registry name (no `<name>:` prefix)
  context: RagNode[]
  markdown: string
  lineMap: LineNodeMap
}

/** Qualify a merged fan-out result. PURE + DETERMINISTIC — never mutates
 *  `merged` or the per-store results/items.
 *
 *  A-F1 gate — when `opts.qualified` is FALSE, returns `merged` UNCHANGED
 *  (byte-equal, no `store`/`storeContexts`), preserving the Phase-1 A4
 *  single-store contract. `stores` is NOT read. When TRUE, stamps the
 *  per-item/per-entry `store` (D3) and builds `storeContexts` (D2) from the
 *  per-store results. */
export function qualifyStoreResult(
  merged: RagResult,
  stores: Array<StoreResultInput>,
  opts: { qualified: boolean },
): RagResult {
  if (merged === null || merged === undefined) {
    throw new Error('qualifyStoreResult: merged result required')
  }
  if (opts === null || opts === undefined) {
    throw new Error('qualifyStoreResult: opts required')
  }
  if (typeof opts.qualified !== 'boolean') {
    throw new Error('qualifyStoreResult: qualified must be a boolean')
  }
  if (!opts.qualified) {
    return merged
  }

  // `qualified` === true — validate the store inputs.
  if (stores === null || stores === undefined || !Array.isArray(stores)) {
    throw new Error('qualifyStoreResult: stores required when qualified')
  }

  // ---- validation pass (§5.6) ----
  // A store with a valid (non-null) result must carry context/markdown/lineMap
  // (needed for its storeContexts block). A null/undefined-result store is
  // SKIPPED (D6 — no block, no attribution, never a throw).
  for (let i = 0; i < stores.length; i++) {
    const entry: StoreResultInput = stores[i]
    if (entry === null || entry === undefined) {
      throw new Error('qualifyStoreResult: store entry required')
    }
    if (typeof entry.name !== 'string' || entry.name.trim().length === 0) {
      throw new Error('qualifyStoreResult: store name must be a non-empty string')
    }
    const result = entry.result
    if (result !== null && result !== undefined) {
      // F-F2-1: a non-null `result` MUST carry a `results` array. An absent/
      // undefined (or non-array) `results` would otherwise survive the §5.6
      // pass and crash later with an unpinned `entry.result.results is not
      // iterable` — assert it here with the pinned message instead.
      if (!Array.isArray(result.results)) {
        throw new Error('qualifyStoreResult: store results must be an array')
      }
      if (
        result.context === null ||
        result.context === undefined ||
        result.markdown === null ||
        result.markdown === undefined ||
        result.lineMap === null ||
        result.lineMap === undefined
      ) {
        throw new Error(
          'qualifyStoreResult: store result must have context, markdown and lineMap',
        )
      }
    }
  }

  // ---- attribute every merged item (§5.3.1) — reference identity against the
  // per-store results.items arrays (mergeStoreResults interleaves BY REFERENCE,
  // U-F1 §5.2, so reference identity is exact) ----
  const itemOwner = new Map<RagResultItem, string>()
  for (const entry of stores) {
    if (entry && entry.result) {
      const name = entry.name
      for (const it of entry.result.results) {
        if (it !== null && it !== undefined) {
          if (!itemOwner.has(it)) itemOwner.set(it, name)
        }
      }
    }
  }
  const ownerName = (item: RagResultItem): string => {
    const n = itemOwner.get(item)
    if (n === undefined) {
      throw new Error('qualifyStoreResult: unattributable result item')
    }
    return n
  }

  // ---- spread-copy the items with `store` stamped (§5.3.1) ----
  // F-F2-2: skip a `null`/`undefined` element in `merged.results` (a sparse-array
  // hole or an explicit null) rather than silently spreading it to `{}` and then
  // mischaracterizing it as `unattributable result item` — consistent with U-F1's
  // null skip (merge-store-results F-F1-3).
  const results = merged.results
    .filter((item) => item !== null && item !== undefined)
    .map((item) => ({ ...item, store: ownerName(item) }))

  // ---- re-derive citations with the first-appearance store (§5.3.2) ----
  const citations: RagResult['citations'] = []
  const seen = new Set<string>()
  for (const item of results) {
    const key = `${item.documentId}\u0000${item.nodeId}`
    if (seen.has(key)) continue
    seen.add(key)
    citations.push({ documentId: item.documentId, nodeId: item.nodeId, store: item.store })
  }

  // ---- build `storeContexts` (§5.3.3, D2) — one block per VALID store in
  // the `stores` array order; the null/undefined-result stores are skipped ----
  const storeContexts: StoreContextBlock[] = []
  for (const entry of stores) {
    if (entry && entry.result) {
      storeContexts.push({
        store: entry.name,
        context: entry.result.context,
        markdown: entry.result.markdown,
        lineMap: entry.result.lineMap,
      })
    }
  }

  // ---- the qualified result: passthrough of the preserved surface; the
  // top-level block stays the default store's block (D2); no `blockedBy`
  // (FLAT-only, D4) ----
  return {
    query: merged.query,
    results,
    engine: merged.engine,
    citations,
    trace: merged.trace,
    ranked: merged.ranked,
    context: merged.context,
    markdown: merged.markdown,
    lineMap: merged.lineMap,
    k: merged.k,
    storeContexts,
  }
}
