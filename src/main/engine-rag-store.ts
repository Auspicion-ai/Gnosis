// src/main/engine-rag-store.ts — Unit GN: the Gnosis-engine wire client
// (docs/specs/unit-gn-engine-integration.md). The `createEngineRagStore` proxy
// over the F2 wire contract — the retrieval trio + health (ragQuery /
// ragStream / getEngineStatus + health), decode-then-validate, the §11
// HTTP-status rendering (all 21 rows, ConflictError = 409), the SSE client for
// ragStream (single-shot), READY observation via getEngineStatus, and the D2
// engine-absent = UNAVAILABLE behavior. The module is a decode-only wire client
// (it consumes wire responses; there is no encode surface).
//
// The proxy is a distinct surface from the Astrographer `RagStore` CRUD
// interface (Unit A §5.4) — it implements the retrieval trio + health ONLY.

import type { BlockedByEntry, RagQueryFilters } from './retrieval.js'

// ---------------------------------------------------------------------------
// The §11 HTTP-status map (all 21 rows — the conformance target).
// ---------------------------------------------------------------------------

export type EngineErrorCode =
  | 'not_found'
  | 'wiki_not_found'
  | 'validation_error'
  | 'conflict'
  | 'doc_in_use'
  | 'invalid_state'
  | 'unresolved_reference'
  | 'engine_unavailable'
  | 'engine_error'
  | 'trace_unavailable'
  | 'hop_limit_exceeded'
  | 'cycle_detected'
  | 'embedding_unavailable'
  | 'vector_index_unavailable'
  | 'lexical_index_unavailable'
  | 'reranker_unavailable'
  | 'compression_failed'
  | 'hyde_generation_failed'
  | 'multi_query_expansion_failed'
  | 'community_not_found'
  | 'sub_task_dag_failed'

/** The §11 map — total over the 21 codes. ConflictError = 409 (mandated). */
export const ENGINE_HTTP_STATUS: Record<EngineErrorCode, number> = {
  not_found: 404,
  wiki_not_found: 404,
  validation_error: 400,
  conflict: 409,
  doc_in_use: 409,
  invalid_state: 409,
  unresolved_reference: 422,
  engine_unavailable: 503,
  engine_error: 502,
  trace_unavailable: 502,
  hop_limit_exceeded: 422,
  cycle_detected: 409,
  embedding_unavailable: 503,
  vector_index_unavailable: 503,
  lexical_index_unavailable: 503,
  reranker_unavailable: 503,
  compression_failed: 500,
  hyde_generation_failed: 500,
  multi_query_expansion_failed: 500,
  community_not_found: 404,
  sub_task_dag_failed: 500,
}

/** Pinned endpoint paths (the shell's transport decision). */
export const ENGINE_ENDPOINTS = {
  ragQuery: '/rag/query',
  ragStream: '/rag/stream',
  engineStatus: '/engine/status',
} as const

// ---------------------------------------------------------------------------
// The typed error model.
// ---------------------------------------------------------------------------

export class EngineWireError extends Error {
  constructor(
    readonly code: EngineErrorCode,
    readonly httpStatus: number,
    message: string,
  ) {
    super(message)
    this.name = 'EngineWireError'
  }
}

export class EngineUnavailable extends EngineWireError {
  constructor(
    readonly cause:
      | 'connection-refused'
      | 'engine-not-spawned'
      | 'not-ready'
      | 'unavailable-state',
    message: string,
  ) {
    super('engine_unavailable', 503, message)
    this.name = 'EngineUnavailable'
  }
}

export class EngineError extends EngineWireError {
  constructor(message: string) {
    super('engine_error', 502, message)
    this.name = 'EngineError'
  }
}

export class TraceUnavailable extends EngineWireError {
  constructor(message: string) {
    super('trace_unavailable', 502, message)
    this.name = 'TraceUnavailable'
  }
}

export class ConflictError extends EngineWireError {
  constructor(message: string) {
    super('conflict', 409, message)
    this.name = 'ConflictError'
  }
}

/** Translate a wire code + message to the typed error. A known code → the
 *  matching EngineWireError (with the §11 httpStatus). An UNKNOWN code →
 *  EngineError (502). PURE + DETERMINISTIC. */
export function wireCodeToError(code: string, message: string): EngineWireError {
  const status = ENGINE_HTTP_STATUS[code as EngineErrorCode]
  if (status === undefined) return new EngineError(message)
  switch (code) {
    case 'engine_unavailable':
      return new EngineUnavailable('unavailable-state', message)
    case 'conflict':
      return new ConflictError(message)
    case 'engine_error':
      return new EngineError(message)
    case 'trace_unavailable':
      return new TraceUnavailable(message)
    default:
      return new EngineWireError(code as EngineErrorCode, status, message)
  }
}

// ---------------------------------------------------------------------------
// The wire shapes (types).
// ---------------------------------------------------------------------------

/** The versioned envelope (the wire shape §3.1 of the guide / §4.1 of the wire
 *  contract). `payload` is the canonical chunk/result/error JSON. */
export interface Envelope {
  schemaVersion: number
  idFormat: string
  payload: unknown
}

/** The canonical chunk (the wire shape §3.2 of the guide / §4.2 of the wire
 *  contract). A `result` chunk carries the proxy-specific `EngineRagResult`. */
export type RagChunk =
  | { type: 'result'; result: EngineRagResult }
  | { type: 'done' }
  | { type: 'error'; code: string; message: string }

/** The health report (the wire shape §3.5 of the guide / §9 of the wire
 *  contract). `state` is the PascalCase `EngineState`; `lastError` is `Some`
 *  exactly when the engine is `Degraded` (a faithful projection). */
export interface HealthReport {
  schemaVersion: number
  idFormat: string
  state: 'Ready' | 'Starting' | 'Degraded' | 'Unavailable'
  version: string
  subsystems: {
    store: boolean
    graph: boolean
    lexical: boolean
    vector: boolean
    embedding: boolean
    reranker: boolean
  }
  lastError: string | null
}

/** The injectable SSE client. `subscribe` opens a connection to `url` and
 *  dispatches parsed single-event frames to `onEvent`; `onError` fires on a
 *  transport failure; `onClose` fires when the connection closes. Returns a
 *  handle whose `close()` tears the connection down cleanly. */
export interface SseClient {
  subscribe(
    url: string,
    handlers: {
      onEvent: (event: string, data: string) => void
      onError: (err: unknown) => void
      onClose: () => void
    },
  ): { close(): void }
}

/** The proxy-specific result — the wire body mapped to a shell-consumable
 *  shape. This is NOT the full Astrographer `RagResult`: the wire body carries
 *  none of the local fan-out's `ranked`/`context`/`markdown`/`lineMap`/`k`
 *  fields, and the wire `trace` covers all four engine modes. */
export interface EngineRagResult {
  query: string
  results: EngineRagResultItem[]
  engine: string
  citations: Array<{ documentId: string; nodeId: string }>
  trace: EngineRagTrace
  blockedBy?: BlockedByEntry[]
}

/** The per-result item (the wire `results[]` element, snake→camel mapped). */
export interface EngineRagResultItem {
  documentId: string
  nodeId: string
  score: number
  snippet: string
  source: 'local' | 'zodiac'
  parent?: { documentId: string; title: string; snippet: string; stale: boolean }
  stale?: boolean
}

/** The proxy-specific trace — a discriminated union over ALL FOUR wire trace
 *  variants (the Astrographer `RagTrace` is flat/graph ONLY). */
export type EngineRagTrace =
  | EngineFlatTrace
  | EngineVectorTrace
  | EngineGraphTrace
  | EngineHybridTrace

export interface EngineFlatTrace {
  mode: 'flat'
  engine: string
  topK: number
  source: 'local' | 'zodiac'
}

export interface EngineVectorTrace {
  mode: 'vector'
  engine: string
  topK: number
  source: 'local' | 'zodiac'
}

export interface EngineGraphTrace {
  mode: 'graph'
  entries: EngineGraphTraceEntry[]
}

export interface EngineHybridTrace {
  mode: 'hybrid'
  engine: string
  legs: Array<'graph' | 'vector' | 'lexical'>
  topK: number
  source: 'local' | 'zodiac'
}

export interface EngineGraphTraceEntry {
  from: { documentId: string; nodeId: string }
  to: { documentId: string; nodeId: string }
  edge: 'link' | 'embed' | 'crosslink'
  state: 'FRESH' | 'RESOLVED' | 'STALE' | 'BROKEN'
}

/** The engine's bind address — LOOPBACK-ONLY by contract. */
export interface EngineRagStoreOptions {
  baseUrl: string
  requestTimeoutMs?: number
  readyPollIntervalMs?: number
  readyPollMaxAttempts?: number
  auth?: {
    token?: string
    /** TLS options (ca/cert/key) for the loopback connection. ACCEPTED but not
     *  yet applied: the standard `fetch` API does not accept a custom
     *  CA/cert/key without an https agent, and the injectable `fetch` is the
     *  transport seam. A future https-agent-backed transport may apply these. */
    tls?: { ca?: string; cert?: string; key?: string }
  }
  fetch?: typeof fetch
  sse?: SseClient
}

/** The retrieval-trio query options (proxy-specific — passes through the
 *  engine's FULL mode set, incl. the vector/hybrid modes). */
export interface EngineRagQueryOptions {
  topK?: number
  mode?: 'flat' | 'graph' | 'vector' | 'hybrid'
  maxHops?: number
  expand?: 'none' | 'parent'
  maxParentContext?: number
  filters?: RagQueryFilters
}

/** The proxy surface — the retrieval trio + health + READY observation. */
export interface EngineRagStore {
  ragQuery(query: string, opts?: EngineRagQueryOptions): Promise<EngineRagResult>
  ragStream(query: string, opts?: EngineRagQueryOptions): AsyncIterable<RagChunk>
  getEngineStatus(): Promise<HealthReport>
  health(): Promise<HealthReport>
  waitForReady(opts?: {
    pollIntervalMs?: number
    maxAttempts?: number
  }): Promise<HealthReport>
}

// ---------------------------------------------------------------------------
// The decoder (decode-then-validate).
// ---------------------------------------------------------------------------

/** Decode the envelope from a response body string. Throws EngineError on a
 *  malformed envelope (missing schemaVersion/idFormat/payload, or a wrong
 *  field type). */
export function decodeEnvelope(json: string): Envelope {
  let parsed: unknown
  try {
    parsed = JSON.parse(json)
  } catch {
    throw new EngineError('malformed envelope: invalid JSON')
  }
  if (!parsed || typeof parsed !== 'object') {
    throw new EngineError('malformed envelope: not an object')
  }
  const obj = parsed as Record<string, unknown>
  if (typeof obj.schemaVersion !== 'number') {
    throw new EngineError('malformed envelope: schemaVersion')
  }
  if (obj.schemaVersion !== 1) {
    throw new EngineError(`unsupported schemaVersion: ${obj.schemaVersion}`)
  }
  if (typeof obj.idFormat !== 'string') {
    throw new EngineError('malformed envelope: idFormat')
  }
  if (obj.idFormat !== 'opaque-string-v1') {
    throw new EngineError(`unknown idFormat: ${obj.idFormat}`)
  }
  if (!('payload' in obj)) {
    throw new EngineError('malformed envelope: payload')
  }
  return {
    schemaVersion: obj.schemaVersion,
    idFormat: obj.idFormat,
    payload: obj.payload,
  }
}

function decodeTrace(trace: unknown): EngineRagTrace {
  if (!trace || typeof trace !== 'object') {
    throw new EngineError('malformed trace')
  }
  const t = trace as Record<string, any>
  if ('Flat' in t) {
    const f = t.Flat
    if (
      !f ||
      typeof f.mode !== 'string' ||
      f.mode !== 'Flat' ||
      typeof f.engine !== 'string' ||
      typeof f.top_k !== 'number' ||
      typeof f.source !== 'string'
    ) {
      throw new EngineError('malformed flat trace')
    }
    const source = f.source.toLowerCase()
    if (source !== 'local' && source !== 'zodiac') {
      throw new EngineError('malformed flat trace: source')
    }
    return { mode: 'flat', engine: f.engine, topK: f.top_k, source }
  }
  if ('Vector' in t) {
    const v = t.Vector
    if (
      !v ||
      typeof v.mode !== 'string' ||
      v.mode !== 'Vector' ||
      typeof v.engine !== 'string' ||
      typeof v.top_k !== 'number' ||
      typeof v.source !== 'string'
    ) {
      throw new EngineError('malformed vector trace')
    }
    const source = v.source.toLowerCase()
    if (source !== 'local' && source !== 'zodiac') {
      throw new EngineError('malformed vector trace: source')
    }
    return { mode: 'vector', engine: v.engine, topK: v.top_k, source }
  }
  if ('Graph' in t) {
    const entries = t.Graph
    if (!Array.isArray(entries)) {
      throw new EngineError('malformed graph trace')
    }
    return {
      mode: 'graph',
      entries: entries.map((e: any) => {
        if (!e || typeof e !== 'object') {
          throw new EngineError('malformed graph trace entry')
        }
        const from = e.from
        const to = e.to
        if (
          !from ||
          typeof from !== 'object' ||
          typeof from.document_id !== 'string' ||
          typeof from.node_id !== 'string' ||
          !to ||
          typeof to !== 'object' ||
          typeof to.document_id !== 'string' ||
          typeof to.node_id !== 'string'
        ) {
          throw new EngineError('malformed graph trace entry')
        }
        if (e.edge !== 'link' && e.edge !== 'embed' && e.edge !== 'crosslink') {
          throw new EngineError('malformed graph trace entry: edge')
        }
        if (
          e.state !== 'FRESH' &&
          e.state !== 'RESOLVED' &&
          e.state !== 'STALE' &&
          e.state !== 'BROKEN'
        ) {
          throw new EngineError('malformed graph trace entry: state')
        }
        return {
          from: { documentId: from.document_id, nodeId: from.node_id },
          to: { documentId: to.document_id, nodeId: to.node_id },
          edge: e.edge,
          state: e.state,
        }
      }),
    }
  }
  if ('Hybrid' in t) {
    const h = t.Hybrid
    if (
      !h ||
      typeof h.mode !== 'string' ||
      h.mode !== 'Hybrid' ||
      typeof h.engine !== 'string' ||
      !Array.isArray(h.legs) ||
      typeof h.top_k !== 'number' ||
      typeof h.source !== 'string'
    ) {
      throw new EngineError('malformed hybrid trace')
    }
    const source = h.source.toLowerCase()
    if (source !== 'local' && source !== 'zodiac') {
      throw new EngineError('malformed hybrid trace: source')
    }
    for (const leg of h.legs) {
      if (leg !== 'graph' && leg !== 'vector' && leg !== 'lexical') {
        throw new EngineError('malformed hybrid trace: legs')
      }
    }
    return { mode: 'hybrid', engine: h.engine, legs: h.legs, topK: h.top_k, source }
  }
  throw new EngineError('unknown trace variant')
}

function mapResultItem(item: any): EngineRagResultItem {
  if (!item || typeof item !== 'object') {
    throw new EngineError('malformed result item')
  }
  if (
    typeof item.document_id !== 'string' ||
    typeof item.node_id !== 'string' ||
    typeof item.score !== 'number' ||
    typeof item.snippet !== 'string' ||
    typeof item.source !== 'string'
  ) {
    throw new EngineError('malformed result item')
  }
  const source = item.source.toLowerCase()
  if (source !== 'local' && source !== 'zodiac') {
    throw new EngineError('malformed result item: source')
  }
  const mapped: EngineRagResultItem = {
    documentId: item.document_id,
    nodeId: item.node_id,
    score: item.score,
    snippet: item.snippet,
    source,
  }
  if (item.parent != null) {
    if (
      !item.parent ||
      typeof item.parent !== 'object' ||
      typeof item.parent.document_id !== 'string' ||
      typeof item.parent.title !== 'string' ||
      typeof item.parent.snippet !== 'string' ||
      typeof item.parent.stale !== 'boolean'
    ) {
      throw new EngineError('malformed result item: parent')
    }
    mapped.parent = {
      documentId: item.parent.document_id,
      title: item.parent.title,
      snippet: item.parent.snippet,
      stale: item.parent.stale,
    }
  }
  if (item.stale != null) {
    if (typeof item.stale !== 'boolean') {
      throw new EngineError('malformed result item: stale')
    }
    mapped.stale = item.stale
  }
  return mapped
}

/** Decode a RagResult body (the envelope payload) + decode-then-validate.
 *  Maps the wire body to the proxy-specific EngineRagResult shape. Throws
 *  TraceUnavailable / EngineError per the precedence below. */
export function decodeRagResult(payload: unknown): EngineRagResult {
  if (!payload || typeof payload !== 'object') {
    throw new EngineError('malformed result body')
  }
  const body = payload as Record<string, any>
  // trace-key-presence is checked FIRST (wire contract §7/§12 V-9).
  if (!('trace' in body)) {
    throw new TraceUnavailable('result body missing trace')
  }
  // Structural validation.
  if (typeof body.query !== 'string') {
    throw new EngineError('malformed result body: query')
  }
  if (!Array.isArray(body.results)) {
    throw new EngineError('malformed result body: results')
  }
  if (typeof body.engine !== 'string') {
    throw new EngineError('malformed result body: engine')
  }
  if (!Array.isArray(body.citations)) {
    throw new EngineError('malformed result body: citations')
  }
  // engine == "gnosis" validation.
  if (body.engine !== 'gnosis') {
    throw new EngineError('invalid engine')
  }
  const trace = decodeTrace(body.trace)
  // blocked_by ⇒ RagTrace::Graph validation.
  if (body.blocked_by != null && trace.mode !== 'graph') {
    throw new EngineError('blocked_by requires a graph trace')
  }
  const result: EngineRagResult = {
    query: body.query,
    results: body.results.map(mapResultItem),
    engine: body.engine,
    citations: body.citations.map((c: any) => {
      if (
        !Array.isArray(c) ||
        c.length !== 2 ||
        typeof c[0] !== 'string' ||
        typeof c[1] !== 'string'
      ) {
        throw new EngineError('malformed result body: citations')
      }
      return { documentId: c[0], nodeId: c[1] }
    }),
    trace,
  }
  if (body.blocked_by != null) {
    if (!Array.isArray(body.blocked_by)) {
      throw new EngineError('malformed result body: blocked_by')
    }
    result.blockedBy = body.blocked_by.map((b: any) => {
      if (
        !b ||
        typeof b !== 'object' ||
        typeof b.document_id !== 'string' ||
        typeof b.node_id !== 'string' ||
        typeof b.state !== 'string'
      ) {
        throw new EngineError('malformed result body: blocked_by')
      }
      return {
        documentId: b.document_id,
        nodeId: b.node_id,
        state: b.state,
      }
    })
  }
  return result
}

/** Decode a chunk payload (the envelope payload or the SSE data line).
 *  Returns the RagChunk. Throws EngineError on a malformed chunk. */
export function decodeChunk(payload: unknown): RagChunk {
  if (!payload || typeof payload !== 'object') {
    throw new EngineError('malformed chunk')
  }
  const c = payload as Record<string, any>
  if (c.type === 'done') {
    // Wire contract §13: the done chunk is exactly {"type":"done"} — no extra
    // payload is accepted.
    if (Object.keys(c).length !== 1) {
      throw new EngineError('malformed done chunk')
    }
    return { type: 'done' }
  }
  if (c.type === 'error') {
    if (typeof c.code !== 'string' || typeof c.message !== 'string') {
      throw new EngineError('malformed error chunk')
    }
    return { type: 'error', code: c.code, message: c.message }
  }
  if (c.type === 'result') {
    return { type: 'result', result: decodeRagResult(c.result) }
  }
  throw new EngineError('unknown chunk type')
}

/** Decode a standalone error payload ({code,message}). Returns the typed
 *  EngineWireError (an unknown code → EngineError, 502). Call sites throw the
 *  returned error. */
export function decodeError(payload: unknown): EngineWireError {
  if (!payload || typeof payload !== 'object') {
    throw new EngineError('malformed error')
  }
  const e = payload as Record<string, any>
  if (typeof e.code !== 'string' || typeof e.message !== 'string') {
    throw new EngineError('malformed error')
  }
  return wireCodeToError(e.code, e.message)
}

/** Decode a health report payload. Returns the HealthReport. Throws
 *  EngineError on a malformed report. */
export function decodeHealthReport(payload: unknown): HealthReport {
  if (!payload || typeof payload !== 'object') {
    throw new EngineError('malformed health report')
  }
  const h = payload as Record<string, any>
  if (typeof h.schemaVersion !== 'number') {
    throw new EngineError('malformed health report: schemaVersion')
  }
  if (h.schemaVersion !== 1) {
    throw new EngineError(`unsupported schemaVersion: ${h.schemaVersion}`)
  }
  if (typeof h.idFormat !== 'string') {
    throw new EngineError('malformed health report: idFormat')
  }
  if (h.idFormat !== 'opaque-string-v1') {
    throw new EngineError(`unknown idFormat: ${h.idFormat}`)
  }
  if (
    typeof h.state !== 'string' ||
    !['Ready', 'Starting', 'Degraded', 'Unavailable'].includes(h.state)
  ) {
    throw new EngineError('malformed health report: state')
  }
  if (typeof h.version !== 'string') {
    throw new EngineError('malformed health report: version')
  }
  if (!h.subsystems || typeof h.subsystems !== 'object') {
    throw new EngineError('malformed health report: subsystems')
  }
  const subsystems: HealthReport['subsystems'] = {
    store: false,
    graph: false,
    lexical: false,
    vector: false,
    embedding: false,
    reranker: false,
  }
  for (const key of ['store', 'graph', 'lexical', 'vector', 'embedding', 'reranker'] as const) {
    if (typeof h.subsystems[key] !== 'boolean') {
      throw new EngineError('malformed health report: subsystems')
    }
    subsystems[key] = h.subsystems[key]
  }
  if (h.lastError != null && typeof h.lastError !== 'string') {
    throw new EngineError('malformed health report: lastError')
  }
  // Faithfulness (P-SM-1): lastError is Some exactly when the engine is
  // Degraded — a Ready/Starting/Unavailable report must not invent one, and a
  // Degraded report must not drop it.
  if (h.state === 'Degraded' && h.lastError == null) {
    throw new EngineError('malformed health report: lastError')
  }
  if (h.state !== 'Degraded' && h.lastError != null) {
    throw new EngineError('malformed health report: lastError')
  }
  return {
    schemaVersion: h.schemaVersion,
    idFormat: h.idFormat,
    state: h.state as HealthReport['state'],
    version: h.version,
    subsystems,
    lastError: h.lastError ?? null,
  }
}

// ---------------------------------------------------------------------------
// The SSE frame parser.
// ---------------------------------------------------------------------------

/** Parse a single SSE frame. Returns the event type + data line. Throws
 *  EngineError on a malformed frame (no event:/data: lines, or trailing
 *  non-blank content). */
export function parseSseFrame(frame: string): { event: string; data: string } {
  if (typeof frame !== 'string') {
    throw new EngineError('malformed SSE frame')
  }
  const lines = frame.split('\n')
  if (lines.length < 3) {
    throw new EngineError('malformed SSE frame')
  }
  if (lines[lines.length - 1] !== '' || lines[lines.length - 2] !== '') {
    throw new EngineError('malformed SSE frame')
  }
  const body = lines.slice(0, -2)
  if (body.length !== 2) {
    throw new EngineError('malformed SSE frame')
  }
  if (!body[0].startsWith('event:')) {
    throw new EngineError('malformed SSE frame: missing event line')
  }
  if (!body[1].startsWith('data:')) {
    throw new EngineError('malformed SSE frame: missing data line')
  }
  const event = body[0].slice('event:'.length).trim()
  const data = body[1].slice('data:'.length).trim()
  if (!event || !data) {
    throw new EngineError('malformed SSE frame')
  }
  return { event, data }
}

/** Decode a single SSE frame to a RagChunk. Enforces the event/data type
 *  match. Throws EngineError on a mismatch or a malformed data line. */
export function decodeSseChunk(frame: string): RagChunk {
  const { event, data } = parseSseFrame(frame)
  let parsed: unknown
  try {
    parsed = JSON.parse(data)
  } catch {
    throw new EngineError('unparseable SSE data')
  }
  if (!parsed || typeof parsed !== 'object') {
    throw new EngineError('malformed SSE data')
  }
  const type = (parsed as Record<string, unknown>).type
  if (type !== event) {
    throw new EngineError('SSE event/data type mismatch')
  }
  return decodeChunk(parsed)
}

// ---------------------------------------------------------------------------
// The factory + proxy.
// ---------------------------------------------------------------------------

function isLoopbackHost(host: string): boolean {
  if (host === '127.0.0.1' || host === '::1') return true
  // IPv4-mapped IPv6 loopback (::ffff:127.0.0.1 and ::ffff:7f00:1).
  if (host.startsWith('::ffff:')) {
    const v4 = host.slice('::ffff:'.length)
    if (v4 === '127.0.0.1') return true
    const parts = v4.split('.')
    if (parts.length === 4 && parts[0] === '127') return true
  }
  if (host === 'localhost') {
    // §5.8-14: localhost is accepted when it resolves to loopback. The factory
    // is synchronous (no network I/O at construction), so DNS resolution is not
    // performed here; localhost is accepted as the loopback alias per the
    // pinned happy state.
    return true
  }
  return false
}

function assertLoopback(baseUrl: string): void {
  let url: URL
  try {
    url = new URL(baseUrl)
  } catch {
    throw new Error('engine rag store: baseUrl must be loopback')
  }
  const host = url.hostname.replace(/^\[|\]$/g, '')
  if (isLoopbackHost(host)) return
  throw new Error('engine rag store: baseUrl must be loopback')
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

function transportError(err: unknown): unknown {
  if (err instanceof EngineWireError) return err
  // The transport code lives either directly on the error (`err.code`, the
  // shape the unit tests inject) or nested on the fetch/undici `cause`
  // (`err.cause.code`, the real transport shape — a `TypeError: fetch failed`
  // whose `cause` is the underlying socket error). Read both so the D2
  // engine-absent split (connection-refused vs engine-not-spawned) is
  // preserved regardless of which shape the transport produces.
  const code =
    err && typeof err === 'object'
      ? (err as any).code ?? (err as any).cause?.code
      : undefined
  if (code === 'ECONNREFUSED') {
    return new EngineUnavailable('connection-refused', String((err as any).message))
  }
  if (code === 'ENOTFOUND') {
    return new EngineUnavailable('engine-not-spawned', String((err as any).message))
  }
  // Any other transport failure (ECONNRESET, timeout, generic) → a typed
  // EngineWireError (H6): §5.1 requires a typed EngineWireError on any
  // fail-state, never a raw Error.
  return new EngineWireError('engine_unavailable', 503, String(err))
}

/** Create the proxy. Throws on a null/undefined opts, a non-loopback baseUrl,
 *  or a missing baseUrl. Does NOT contact the engine at construction. */
export function createEngineRagStore(opts: EngineRagStoreOptions): EngineRagStore {
  if (opts == null) {
    throw new Error('engine rag store: opts required')
  }
  if (typeof opts.baseUrl !== 'string' || opts.baseUrl === '') {
    throw new Error('engine rag store: baseUrl required')
  }
  assertLoopback(opts.baseUrl)
  const baseUrl = opts.baseUrl.replace(/\/+$/, '')
  const fetchImpl = opts.fetch ?? globalThis.fetch
  const sse = opts.sse ?? defaultSseClient
  const readyPollIntervalMs = opts.readyPollIntervalMs ?? 500
  const readyPollMaxAttempts = opts.readyPollMaxAttempts ?? 60
  const requestTimeoutMs = opts.requestTimeoutMs ?? 10_000
  const auth = opts.auth

  const headers = (): Record<string, string> => {
    const h: Record<string, string> = { 'content-type': 'application/json' }
    if (auth?.token) h['authorization'] = `Bearer ${auth.token}`
    return h
  }

  // H4: apply the per-request timeout to every fetch. On timeout the fetch is
  // aborted and a typed EngineWireError is thrown (never a raw AbortError).
  const fetchWithTimeout = async (url: string, init: RequestInit): Promise<Response> => {
    const controller = new AbortController()
    const timer = setTimeout(() => controller.abort(), requestTimeoutMs)
    try {
      return await fetchImpl(url, { ...init, signal: controller.signal })
    } catch (err) {
      if (controller.signal.aborted) {
        throw new EngineUnavailable(
          'connection-refused',
          `request timed out after ${requestTimeoutMs}ms`,
        )
      }
      throw transportError(err)
    } finally {
      clearTimeout(timer)
    }
  }

  const getEngineStatus = async (): Promise<HealthReport> => {
    const url = `${baseUrl}${ENGINE_ENDPOINTS.engineStatus}`
    let res: Response
    try {
      res = await fetchWithTimeout(url, { method: 'GET', headers: headers() })
    } catch (err) {
      throw transportError(err)
    }
    const text = await res.clone().text()
    let payload: unknown
    try {
      payload = JSON.parse(text)
    } catch {
      throw new EngineError('malformed health report')
    }
    return decodeHealthReport(payload)
  }

  const decodeQueryPayload = (payload: unknown): EngineRagResult => {
    if (payload && typeof payload === 'object' && 'type' in (payload as any)) {
      const c = payload as Record<string, any>
      if (c.type === 'result') return decodeRagResult(c.result)
      if (c.type === 'error') throw wireCodeToError(c.code, c.message)
      throw new EngineError(`unexpected chunk type: ${c.type}`)
    }
    if (
      payload &&
      typeof payload === 'object' &&
      'code' in (payload as any) &&
      'message' in (payload as any)
    ) {
      throw decodeError(payload)
    }
    return decodeRagResult(payload)
  }

  const ragQuery = async (
    query: string,
    opts?: EngineRagQueryOptions,
  ): Promise<EngineRagResult> => {
    // H12: the query must always be present in the wire body — an undefined
    // query would be dropped by JSON.stringify, so reject it up front.
    if (typeof query !== 'string') {
      throw new EngineError('ragQuery: query must be a string')
    }
    // Fresh READY gate per RAG call.
    const report = await getEngineStatus()
    if (report.state !== 'Ready') {
      const cause =
        report.state === 'Unavailable' ? 'unavailable-state' : 'not-ready'
      throw new EngineUnavailable(cause, `engine not ready: ${report.state}`)
    }
    const url = `${baseUrl}${ENGINE_ENDPOINTS.ragQuery}`
    let res: Response
    try {
      res = await fetchWithTimeout(url, {
        method: 'POST',
        headers: headers(),
        body: JSON.stringify({ query, ...opts }),
      })
    } catch (err) {
      throw transportError(err)
    }
    const text = await res.clone().text()
    const env = decodeEnvelope(text)
    return decodeQueryPayload(env.payload)
  }

  const buildStreamUrl = (query: string, opts?: EngineRagQueryOptions): string => {
    const params = new URLSearchParams()
    params.set('query', query)
    if (opts?.topK !== undefined) params.set('topK', String(opts.topK))
    if (opts?.mode !== undefined) params.set('mode', opts.mode)
    if (opts?.maxHops !== undefined) params.set('maxHops', String(opts.maxHops))
    if (opts?.expand !== undefined) params.set('expand', opts.expand)
    if (opts?.maxParentContext !== undefined) {
      params.set('maxParentContext', String(opts.maxParentContext))
    }
    if (opts?.filters !== undefined) {
      params.set('filters', JSON.stringify(opts.filters))
    }
    return `${baseUrl}${ENGINE_ENDPOINTS.ragStream}?${params.toString()}`
  }

  const ragStream = (
    query: string,
    opts?: EngineRagQueryOptions,
  ): AsyncIterable<RagChunk> => {
    return {
      [Symbol.asyncIterator]() {
        let subscribed = false
        let handle: { close(): void } | null = null
        let gateStarted = false
        let gateError: unknown = null
        let gateResolved = false
        const queue: RagChunk[] = []
        let streamDone = false
        let doneReceived = false
        let streamError: unknown = null
        let waiters: Array<() => void> = []

        const wake = () => {
          const ws = waiters
          waiters = []
          for (const w of ws) w()
        }

        const startGate = () => {
          if (gateStarted) return
          gateStarted = true
          getEngineStatus().then(
            (report) => {
              if (report.state !== 'Ready') {
                const cause =
                  report.state === 'Unavailable' ? 'unavailable-state' : 'not-ready'
                gateError = new EngineUnavailable(cause, `engine not ready: ${report.state}`)
              }
              gateResolved = true
              wake()
            },
            (err) => {
              gateError = err
              gateResolved = true
              wake()
            },
          )
        }

        const subscribe = () => {
          if (subscribed) return
          subscribed = true
          const url = buildStreamUrl(query, opts)
          handle = sse.subscribe(url, {
            onEvent: (event, data) => {
              try {
                const chunk = decodeSseChunk(`event: ${event}\ndata: ${data}\n\n`)
                if (chunk.type === 'done') doneReceived = true
                queue.push(chunk)
              } catch (e) {
                streamError = e
              }
              wake()
            },
            onError: (err) => {
              streamError = new EngineUnavailable(
                'connection-refused',
                `sse transport error: ${String(err)}`,
              )
              wake()
            },
            onClose: () => {
              // H10: a clean close is only one that follows a `done` chunk. A
              // dropped connection before `done` that fires onClose (rather than
              // onError) is a premature drop → EngineUnavailable, not a clean end.
              if (doneReceived) {
                streamDone = true
              } else {
                streamError = new EngineUnavailable(
                  'connection-refused',
                  'sse connection closed before done',
                )
              }
              wake()
            },
          })
        }

        return {
          next: async (): Promise<IteratorResult<RagChunk>> => {
            if (!subscribed) {
              subscribe()
              startGate()
            }
            for (;;) {
              if (gateError) {
                // H9: the READY gate precedes proceeding — on a gate failure the
                // SSE connection is torn down (no RAG data proceeds) and the
                // typed gate error surfaces.
                if (handle) handle.close()
                throw gateError
              }
              if (!gateResolved) {
                await new Promise<void>((res) => waiters.push(res))
                continue
              }
              if (queue.length > 0) {
                const chunk = queue.shift()!
                if (chunk.type === 'done') streamDone = true
                return { value: chunk, done: false }
              }
              if (streamError) throw streamError
              if (streamDone) {
                if (handle) handle.close()
                return { value: undefined, done: true }
              }
              await new Promise<void>((res) => waiters.push(res))
            }
          },
          return: async (): Promise<IteratorResult<RagChunk>> => {
            if (handle) handle.close()
            return { value: undefined, done: true }
          },
        }
      },
    }
  }

  const waitForReady = async (opts?: {
    pollIntervalMs?: number
    maxAttempts?: number
  }): Promise<HealthReport> => {
    const pollIntervalMs = opts?.pollIntervalMs ?? readyPollIntervalMs
    const maxAttempts = opts?.maxAttempts ?? readyPollMaxAttempts
    for (let i = 0; i < maxAttempts; i++) {
      const report = await getEngineStatus()
      if (report.state === 'Ready') return report
      if (report.state === 'Unavailable') {
        throw new EngineUnavailable('unavailable-state', 'engine unavailable')
      }
      if (i < maxAttempts - 1) await sleep(pollIntervalMs)
    }
    throw new EngineUnavailable('not-ready', 'engine not ready within maxAttempts')
  }

  return {
    ragQuery,
    ragStream,
    getEngineStatus,
    health: getEngineStatus,
    waitForReady,
  }
}

// ---------------------------------------------------------------------------
// The default SSE client (EventSource-based). Never exercised by the unit
// tests (they inject a fake SSE client); provided for the real transport.
// ---------------------------------------------------------------------------

const defaultSseClient: SseClient = {
  subscribe(url, handlers) {
    const es = new EventSource(url)
    // H3: the wire frames use NAMED events (`event: result`/`event: done`/
    // `event: error`), which EventSource dispatches to addEventListener
    // listeners — never to `onmessage` (that only fires for the default
    // "message" event). Register the named listeners and pass the correct
    // event type through to decodeSseChunk.
    es.addEventListener('result', (e) =>
      handlers.onEvent('result', (e as MessageEvent).data),
    )
    es.addEventListener('done', (e) =>
      handlers.onEvent('done', (e as MessageEvent).data),
    )
    es.addEventListener('error', (e) => {
      const me = e as MessageEvent
      // A server-sent `event: error` chunk carries a data line; a connection
      // error is a bare Event with no data. Route each accordingly.
      if (typeof me.data === 'string' && me.data.length > 0) {
        handlers.onEvent('error', me.data)
      } else {
        handlers.onError(e)
      }
    })
    return {
      close() {
        es.close()
      },
    }
  },
}
