// src/main/engine-crud-rag-store.ts — Unit A1: the document-CRUD routing proxy
// (docs/specs/unit-a1-crud-routing-proxy.md). The `createEngineCrudRagStore`
// proxy routes the 11 §4.1 document-CRUD methods over the frozen document-CRUD
// wire (P1a). It is a SIBLING to `engine-rag-store.ts` (Unit GN) and IMPORTS the
// shared error model, the §11 HTTP-status map, and the decode helpers from it
// (never redefined). Unlike Unit GN (a decode-only client), A1 is an
// encode+decode client: it ENCODES the request envelope (method + args) and
// DECODES the response envelope (result or error). The golden vectors V-10..V-12
// (P1a §9) are the byte-exact conformance targets.
//
// The proxy is a distinct surface from the Astrographer `RagStore` CRUD
// interface (Unit A §5.4) and from the retrieval-trio proxy (Unit GN).

import {
  EngineWireError,
  EngineUnavailable,
  EngineError,
  TraceUnavailable,
  ConflictError,
  wireCodeToError,
  ENGINE_HTTP_STATUS,
  decodeEnvelope,
  decodeHealthReport,
  ENGINE_ENDPOINTS,
  type Envelope,
  type HealthReport,
  type EngineErrorCode,
  type EngineRagStoreOptions,
} from './engine-rag-store.js'

// ---------------------------------------------------------------------------
// The options + wire-shape types (§5.1).
// ---------------------------------------------------------------------------

/** The engine's bind address — LOOPBACK-ONLY by contract. Mirrors the
 *  EngineRagStoreOptions shape (imported from engine-rag-store.ts) EXCEPT the
 *  `sse` field (A1 is a request/response CRUD client with no SSE surface) and
 *  adds the P4 retry option. */
export interface EngineCrudRagStoreOptions {
  /** The engine's base URL, e.g. `http://127.0.0.1:PORT`. MUST resolve to a
   *  loopback address (127.0.0.1 or ::1). A non-loopback baseUrl throws at
   *  construction (LOOPBACK-AUTH-TLS). */
  baseUrl: string
  /** Per-request timeout in ms (default 10_000). */
  requestTimeoutMs?: number
  /** READY-poll interval in ms (default 500). */
  readyPollIntervalMs?: number
  /** Max READY-poll attempts before EngineUnavailable (default 60). */
  readyPollMaxAttempts?: number
  /** Shell-owned auth/TLS (D4 carve-out). */
  auth?: {
    /** Bearer token sent as `Authorization: Bearer <token>`. */
    token?: string
    /** TLS options (ca/cert/key) for the loopback connection. */
    tls?: { ca?: string; cert?: string; key?: string }
  }
  /** Injectable HTTP client (default global fetch). */
  fetch?: typeof fetch
  /** P4 idempotency/retry for mutating creates (createDocument/createWiki).
   *  Bounded, opt-in, retry-on-EngineUnavailable-only. */
  retry?: {
    /** Max retries for a mutating create on EngineUnavailable (default 0 = off). */
    maxRetries?: number
    /** Retry backoff base in ms (default 100). */
    backoffMs?: number
  }
}

/** The serde-frozen request bodies (P1a §4.2 — snake_case keys preserved
 *  verbatim under the `body` key; `null` for absent `Option` fields). */
export interface CreateDocumentRequest {
  title: string
  tags: string[] | null
  author: string | null
}
export interface UpdateDocumentRequest {
  baseRevision: number
  graph: Graph
  title: string | null
  tags: string[] | null
}
export interface ListDocumentsFilter {
  state: 'Draft' | 'Published' | 'Archived' | null
  tag: string | null
  page: number | null
  pageSize: number | null
}
/** The Graph serde body — embedded verbatim (the client does NOT re-case the
 *  graph internals; the node/edge inner shapes are not pinned by P1a and are
 *  passed through opaque). */
export interface Graph { nodes: GraphNode[]; edges: GraphEdge[] }
export interface GraphNode { [key: string]: unknown }
export interface GraphEdge { [key: string]: unknown }

/** The typed request-args (mirroring P1a `CrudRequestArgs` — camelCase top-level
 *  fields; `caller` present ONLY on the 7 mutating methods). */
export interface CreateDocumentArgs { caller: string; wikiId: string; body: CreateDocumentRequest }
export interface GetDocumentArgs { documentId: string }
export interface UpdateDocumentArgs { caller: string; documentId: string; body: UpdateDocumentRequest }
export interface DeleteDocumentArgs { caller: string; documentId: string }
export interface PublishDocumentArgs { caller: string; documentId: string }
export interface UnpublishDocumentArgs { caller: string; documentId: string }
export interface ArchiveDocumentArgs { caller: string; documentId: string }
export interface ListDocumentsArgs { wikiId: string; body: ListDocumentsFilter }
export interface CreateWikiArgs { caller: string; name: string }
export interface GetWikiArgs { wikiId: string }
export interface ListWikisArgs { /* empty */ }

/** The typed result types (mirroring the P1a serde bodies, snake→camel mapped). */
export interface Document {
  documentId: string
  wikiId: string
  revision: number
  state: 'Draft' | 'Published' | 'Archived'
  graph: Graph
  title: string
  createdAt: string
  updatedAt: string
  tags: string[]
  author: string | null
}
export interface DocumentList {
  items: Document[]
  total: number
  page: number
  pageSize: number
}
export interface Wiki { wikiId: string; name: string }

/** The method discriminator (the 11 camelCase wire `"method"` values). */
export type CrudMethod =
  | 'createDocument' | 'getDocument' | 'updateDocument' | 'deleteDocument'
  | 'publishDocument' | 'unpublishDocument' | 'archiveDocument'
  | 'listDocuments' | 'createWiki' | 'getWiki' | 'listWikis'

/** The typed request-args union. */
export type CrudRequestArgs =
  | CreateDocumentArgs | GetDocumentArgs | UpdateDocumentArgs | DeleteDocumentArgs
  | PublishDocumentArgs | UnpublishDocumentArgs | ArchiveDocumentArgs
  | ListDocumentsArgs | CreateWikiArgs | GetWikiArgs | ListWikisArgs

/** The typed result union (discriminated by method). */
export type CrudResult =
  | { method: 'createDocument'; result: Document }
  | { method: 'getDocument'; result: Document }
  | { method: 'updateDocument'; result: Document }
  | { method: 'deleteDocument'; result: null }
  | { method: 'publishDocument'; result: Document }
  | { method: 'unpublishDocument'; result: Document }
  | { method: 'archiveDocument'; result: Document }
  | { method: 'listDocuments'; result: DocumentList }
  | { method: 'createWiki'; result: Wiki }
  | { method: 'getWiki'; result: Wiki }
  | { method: 'listWikis'; result: Wiki[] }

/** The CRUD-specific validation failure (mirroring P1a `CrudValidationFailure`). */
export type CrudValidationFailure =
  | { kind: 'UnexpectedRevision'; expected: number; actual: number }
  | { kind: 'UnexpectedState'; expected: 'Draft' | 'Published' | 'Archived'; actual: 'Draft' | 'Published' | 'Archived' }
  | { kind: 'InvalidPagination'; page: number; pageSize: number }
  | { kind: 'UnexpectedVoid' }

/** The proxy surface — the 11 document-CRUD methods. Each issues the REST call
 *  to the pinned endpoint path with the request envelope body, then decodes
 *  the response envelope. Each returns a Promise of the typed result. Throws a
 *  typed EngineWireError on any fail-state. ASYNC. */
export interface EngineCrudRagStore {
  createDocument(args: CreateDocumentArgs): Promise<Document>
  getDocument(args: GetDocumentArgs): Promise<Document>
  updateDocument(args: UpdateDocumentArgs): Promise<Document>
  deleteDocument(args: DeleteDocumentArgs): Promise<void>
  publishDocument(args: PublishDocumentArgs): Promise<Document>
  unpublishDocument(args: UnpublishDocumentArgs): Promise<Document>
  archiveDocument(args: ArchiveDocumentArgs): Promise<Document>
  listDocuments(args: ListDocumentsArgs): Promise<DocumentList>
  createWiki(args: CreateWikiArgs): Promise<Wiki>
  getWiki(args: GetWikiArgs): Promise<Wiki>
  listWikis(args: ListWikisArgs): Promise<Wiki[]>
}

// ---------------------------------------------------------------------------
// The pinned endpoint paths (P1a §7, H4 — the shell does NOT invent them).
// ---------------------------------------------------------------------------

export const ENGINE_CRUD_ENDPOINTS = {
  createDocument: 'POST /documents',
  getDocument: 'GET /documents/:id',
  updateDocument: 'POST /documents/:id/update',
  deleteDocument: 'DELETE /documents/:id',
  publishDocument: 'POST /documents/:id/publish',
  unpublishDocument: 'POST /documents/:id/unpublish',
  archiveDocument: 'POST /documents/:id/archive',
  listDocuments: 'GET /documents',
  createWiki: 'POST /wikis',
  getWiki: 'GET /wikis/:id',
  listWikis: 'GET /wikis',
} as const

// ---------------------------------------------------------------------------
// The encode surface (the request envelope).
// ---------------------------------------------------------------------------

const CRUD_METHODS: readonly CrudMethod[] = [
  'createDocument',
  'getDocument',
  'updateDocument',
  'deleteDocument',
  'publishDocument',
  'unpublishDocument',
  'archiveDocument',
  'listDocuments',
  'createWiki',
  'getWiki',
  'listWikis',
]

function isCrudMethod(v: string): v is CrudMethod {
  return (CRUD_METHODS as readonly string[]).includes(v)
}

/** Re-case the serde-frozen body from the typed camelCase form to the wire
 *  snake_case form (P1a §4.2). Only the fields whose wire key differs are
 *  re-cased; the rest pass through verbatim. */
function encodeBody(method: CrudMethod, body: unknown): unknown {
  if (method === 'updateDocument') {
    const b = body as UpdateDocumentRequest
    return { base_revision: b.baseRevision, graph: b.graph, title: b.title, tags: b.tags }
  }
  if (method === 'listDocuments') {
    const b = body as ListDocumentsFilter
    return { state: b.state, tag: b.tag, page: b.page, page_size: b.pageSize }
  }
  return body
}

/** Build the wire `args` object for a method (camelCase top-level fields;
 *  `caller` present only on the 7 mutating methods). */
function encodeArgs(method: CrudMethod, args: CrudRequestArgs): Record<string, unknown> {
  switch (method) {
    case 'createDocument': {
      const a = args as CreateDocumentArgs
      return { caller: a.caller, wikiId: a.wikiId, body: encodeBody(method, a.body) }
    }
    case 'getDocument': {
      const a = args as GetDocumentArgs
      return { documentId: a.documentId }
    }
    case 'updateDocument': {
      const a = args as UpdateDocumentArgs
      return { caller: a.caller, documentId: a.documentId, body: encodeBody(method, a.body) }
    }
    case 'deleteDocument': {
      const a = args as DeleteDocumentArgs
      return { caller: a.caller, documentId: a.documentId }
    }
    case 'publishDocument': {
      const a = args as PublishDocumentArgs
      return { caller: a.caller, documentId: a.documentId }
    }
    case 'unpublishDocument': {
      const a = args as UnpublishDocumentArgs
      return { caller: a.caller, documentId: a.documentId }
    }
    case 'archiveDocument': {
      const a = args as ArchiveDocumentArgs
      return { caller: a.caller, documentId: a.documentId }
    }
    case 'listDocuments': {
      const a = args as ListDocumentsArgs
      return { wikiId: a.wikiId, body: encodeBody(method, a.body) }
    }
    case 'createWiki': {
      const a = args as CreateWikiArgs
      return { caller: a.caller, name: a.name }
    }
    case 'getWiki': {
      const a = args as GetWikiArgs
      return { wikiId: a.wikiId }
    }
    case 'listWikis':
      return {}
  }
}

/** Encode a CRUD request envelope (method + args). Total over the 11 methods —
 *  never emits an unknown method or a malformed envelope. PURE. */
export function encodeCrudRequest(method: CrudMethod, args: CrudRequestArgs): Envelope {
  return {
    schemaVersion: 1,
    idFormat: 'opaque-string-v1',
    payload: { method, args: encodeArgs(method, args) },
  }
}

/** Re-case the wire body back to the typed camelCase form (the inverse of
 *  encodeBody). */
function decodeBody(method: CrudMethod, body: unknown): unknown {
  if (method === 'updateDocument') {
    const b = body as Record<string, unknown>
    return {
      baseRevision: b.base_revision,
      graph: b.graph,
      title: b.title,
      tags: b.tags,
    }
  }
  if (method === 'listDocuments') {
    const b = body as Record<string, unknown>
    return { state: b.state, tag: b.tag, page: b.page, pageSize: b.page_size }
  }
  return body
}

/** Decode the wire `args` object back to the typed args (the inverse of
 *  encodeArgs). */
function decodeArgs(method: CrudMethod, args: Record<string, unknown>): CrudRequestArgs {
  switch (method) {
    case 'createDocument':
      return {
        caller: args.caller as string,
        wikiId: args.wikiId as string,
        body: decodeBody(method, args.body) as CreateDocumentRequest,
      }
    case 'getDocument':
      return { documentId: args.documentId as string }
    case 'updateDocument':
      return {
        caller: args.caller as string,
        documentId: args.documentId as string,
        body: decodeBody(method, args.body) as UpdateDocumentRequest,
      }
    case 'deleteDocument':
      return { caller: args.caller as string, documentId: args.documentId as string }
    case 'publishDocument':
      return { caller: args.caller as string, documentId: args.documentId as string }
    case 'unpublishDocument':
      return { caller: args.caller as string, documentId: args.documentId as string }
    case 'archiveDocument':
      return { caller: args.caller as string, documentId: args.documentId as string }
    case 'listDocuments':
      return {
        wikiId: args.wikiId as string,
        body: decodeBody(method, args.body) as ListDocumentsFilter,
      }
    case 'createWiki':
      return { caller: args.caller as string, name: args.name as string }
    case 'getWiki':
      return { wikiId: args.wikiId as string }
    case 'listWikis':
      return {}
  }
}

/** Decode a CRUD request envelope back to (method, args). The inverse of
 *  encodeCrudRequest. PURE. */
export function decodeCrudRequest(env: Envelope): { method: CrudMethod; args: CrudRequestArgs } {
  const payload = env.payload as { method: CrudMethod; args: Record<string, unknown> }
  return { method: payload.method, args: decodeArgs(payload.method, payload.args) }
}

// ---------------------------------------------------------------------------
// The decode surface (the response envelope) — decode-then-validate (§5.3).
// ---------------------------------------------------------------------------

function decodeDocument(body: unknown): Document {
  if (!body || typeof body !== 'object') {
    throw new EngineError('malformed document')
  }
  const b = body as Record<string, any>
  // Accept both the snake_case wire keys (P1a §4.3) and the camelCase typed
  // keys (the §5.7 PBT register feeds typed results to decodeCrudResponse).
  const documentId = b.document_id ?? b.documentId
  const wikiId = b.wiki_id ?? b.wikiId
  const revision = b.revision
  const state = b.state
  const graph = b.graph
  const title = b.title
  const createdAt = b.created_at ?? b.createdAt
  const updatedAt = b.updated_at ?? b.updatedAt
  const tags = b.tags
  const author = b.author
  if (
    typeof documentId !== 'string' ||
    typeof wikiId !== 'string' ||
    typeof revision !== 'number' ||
    typeof state !== 'string' ||
    !['Draft', 'Published', 'Archived'].includes(state) ||
    !graph ||
    typeof graph !== 'object' ||
    typeof title !== 'string' ||
    typeof createdAt !== 'string' ||
    typeof updatedAt !== 'string' ||
    !Array.isArray(tags) ||
    (author != null && typeof author !== 'string')
  ) {
    throw new EngineError('malformed document')
  }
  return {
    documentId,
    wikiId,
    revision,
    state: state as Document['state'],
    graph: graph as Graph,
    title,
    createdAt,
    updatedAt,
    tags,
    author: author ?? null,
  }
}

function decodeWiki(body: unknown): Wiki {
  if (!body || typeof body !== 'object') {
    throw new EngineError('malformed wiki')
  }
  const b = body as Record<string, any>
  const wikiId = b.wiki_id ?? b.wikiId
  const name = b.name
  if (typeof wikiId !== 'string' || typeof name !== 'string') {
    throw new EngineError('malformed wiki')
  }
  return { wikiId, name }
}

function decodeDocumentList(body: unknown): DocumentList {
  if (!body || typeof body !== 'object') {
    throw new EngineError('malformed document list')
  }
  const b = body as Record<string, any>
  const pageSize = b.page_size ?? b.pageSize
  if (
    !Array.isArray(b.items) ||
    typeof b.total !== 'number' ||
    typeof b.page !== 'number' ||
    typeof pageSize !== 'number'
  ) {
    throw new EngineError('malformed document list')
  }
  return {
    items: b.items.map(decodeDocument),
    total: b.total,
    page: b.page,
    pageSize,
  }
}

/** Decode the serde-frozen result body to the typed result for the method. A
 *  malformed body (wrong field types) → EngineError (502). */
function decodeResult(method: CrudMethod, resultBody: unknown): unknown {
  switch (method) {
    case 'createDocument':
    case 'getDocument':
    case 'updateDocument':
    case 'publishDocument':
    case 'unpublishDocument':
    case 'archiveDocument':
      return decodeDocument(resultBody)
    case 'deleteDocument':
      // The typed result is void (null); a non-null result is surfaced as a
      // validation failure (UnexpectedVoid), not a malformed body.
      return resultBody
    case 'listDocuments':
      return decodeDocumentList(resultBody)
    case 'createWiki':
    case 'getWiki':
      return decodeWiki(resultBody)
    case 'listWikis':
      if (!Array.isArray(resultBody)) throw new EngineError('malformed wikis')
      return resultBody.map(decodeWiki)
  }
}

/** Validate a decoded CRUD result against the CRUD-specific invariant for the
 *  method. Returns the CrudValidationFailure, or null if valid. PURE. */
export function validateCrudResult(
  method: CrudMethod,
  result: CrudResult,
): CrudValidationFailure | null {
  switch (method) {
    case 'createDocument': {
      const r = result.result as Document
      if (r.revision !== 0) {
        return { kind: 'UnexpectedRevision', expected: 0, actual: r.revision }
      }
      if (r.state !== 'Draft') {
        return { kind: 'UnexpectedState', expected: 'Draft', actual: r.state }
      }
      return null
    }
    case 'getDocument':
    case 'updateDocument':
      return null
    case 'deleteDocument':
      if (result.result !== null) return { kind: 'UnexpectedVoid' }
      return null
    case 'publishDocument': {
      const r = result.result as Document
      if (r.state !== 'Published') {
        return { kind: 'UnexpectedState', expected: 'Published', actual: r.state }
      }
      return null
    }
    case 'unpublishDocument': {
      const r = result.result as Document
      if (r.state !== 'Draft') {
        return { kind: 'UnexpectedState', expected: 'Draft', actual: r.state }
      }
      return null
    }
    case 'archiveDocument': {
      const r = result.result as Document
      if (r.state !== 'Archived') {
        return { kind: 'UnexpectedState', expected: 'Archived', actual: r.state }
      }
      return null
    }
    case 'listDocuments': {
      const r = result.result as DocumentList
      if (r.page < 1 || r.pageSize < 1 || r.pageSize > 100) {
        return { kind: 'InvalidPagination', page: r.page, pageSize: r.pageSize }
      }
      return null
    }
    case 'createWiki':
    case 'getWiki':
    case 'listWikis':
      return null
  }
}

function crudValidationMessage(method: CrudMethod, failure: CrudValidationFailure): string {
  switch (failure.kind) {
    case 'UnexpectedRevision':
      return `${method}: unexpected revision (expected ${failure.expected}, got ${failure.actual})`
    case 'UnexpectedState':
      return `${method}: unexpected state (expected ${failure.expected}, got ${failure.actual})`
    case 'InvalidPagination':
      return `${method}: invalid pagination (page ${failure.page}, pageSize ${failure.pageSize})`
    case 'UnexpectedVoid':
      return `${method}: unexpected void`
  }
}

/** Decode a CRUD response envelope PAYLOAD ({method, result|error}) to the
 *  typed CrudResult. The parameter is the envelope's `payload` object — NOT the
 *  full versioned envelope. Call sites decode the full envelope via
 *  `decodeEnvelope` (imported from engine-rag-store.ts) FIRST, then pass
 *  `env.payload` here. Throws a typed EngineWireError on any fail-state
 *  (decode-then-validate, §5.3). PURE. */
export function decodeCrudResponse(payload: unknown): CrudResult {
  if (!payload || typeof payload !== 'object') {
    throw new EngineError('malformed crud response: not an object')
  }
  const p = payload as Record<string, unknown>
  if (typeof p.method !== 'string' || !isCrudMethod(p.method)) {
    throw new EngineError('malformed crud response: method')
  }
  const method = p.method
  const hasResult = 'result' in p
  const hasError = 'error' in p
  if (hasResult === hasError) {
    throw new EngineError('malformed crud response: exactly one of result/error')
  }
  if (hasError) {
    const e = p.error as Record<string, unknown>
    if (
      !e ||
      typeof e !== 'object' ||
      typeof e.code !== 'string' ||
      typeof e.message !== 'string'
    ) {
      throw new EngineError('malformed crud response: error')
    }
    throw wireCodeToError(e.code, e.message)
  }
  const result = decodeResult(method, p.result)
  const crudResult = { method, result } as CrudResult
  const failure = validateCrudResult(method, crudResult)
  if (failure) {
    throw new EngineError(crudValidationMessage(method, failure))
  }
  return crudResult
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
    // §5.8-14: localhost is accepted as the loopback alias (the factory is
    // synchronous — no DNS resolution is performed at construction).
    return true
  }
  return false
}

function assertLoopback(baseUrl: string): void {
  let url: URL
  try {
    url = new URL(baseUrl)
  } catch {
    throw new Error('engine crud rag store: baseUrl must be loopback')
  }
  const host = url.hostname.replace(/^\[|\]$/g, '')
  if (isLoopbackHost(host)) return
  throw new Error('engine crud rag store: baseUrl must be loopback')
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

function transportError(err: unknown): unknown {
  if (err instanceof EngineWireError) return err
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
  return new EngineWireError('engine_unavailable', 503, String(err))
}

/** Create the proxy. Throws on a null/undefined opts, a non-loopback baseUrl,
 *  or a missing baseUrl. Does NOT contact the engine at construction. */
export function createEngineCrudRagStore(opts: EngineCrudRagStoreOptions): EngineCrudRagStore {
  if (opts == null) {
    throw new Error('engine crud rag store: opts required')
  }
  if (typeof opts.baseUrl !== 'string' || opts.baseUrl === '') {
    throw new Error('engine crud rag store: baseUrl required')
  }
  assertLoopback(opts.baseUrl)
  const baseUrl = opts.baseUrl.replace(/\/+$/, '')
  const fetchImpl = opts.fetch ?? globalThis.fetch
  const requestTimeoutMs = opts.requestTimeoutMs ?? 10_000
  const auth = opts.auth
  const retry = opts.retry

  const headers = (): Record<string, string> => {
    const h: Record<string, string> = { 'content-type': 'application/json' }
    if (auth?.token) h['authorization'] = `Bearer ${auth.token}`
    return h
  }

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

  /** Issue a fresh READY gate, then the REST call to the pinned endpoint path
   *  with the request envelope in the JSON request body, then decode the
   *  response envelope. */
  const crudCall = async <T>(
    method: CrudMethod,
    args: CrudRequestArgs,
    id: string | undefined,
  ): Promise<T> => {
    const report = await getEngineStatus()
    if (report.state !== 'Ready') {
      const cause =
        report.state === 'Unavailable' ? 'unavailable-state' : 'not-ready'
      throw new EngineUnavailable(cause, `engine not ready: ${report.state}`)
    }
    const endpoint = ENGINE_CRUD_ENDPOINTS[method]
    const space = endpoint.indexOf(' ')
    const verb = endpoint.slice(0, space)
    const path = endpoint.slice(space + 1)
    const urlPath = id !== undefined ? path.replace(':id', encodeURIComponent(id)) : path
    const url = `${baseUrl}${urlPath}`
    const env = encodeCrudRequest(method, args)
    let res: Response
    try {
      res = await fetchWithTimeout(url, {
        method: verb,
        headers: headers(),
        body: JSON.stringify(env),
      })
    } catch (err) {
      throw transportError(err)
    }
    const text = await res.clone().text()
    const decoded = decodeEnvelope(text)
    const crudResult = decodeCrudResponse(decoded.payload)
    return crudResult.result as T
  }

  /** P4 retry (mutating creates only): retry on EngineUnavailable (503) up to
   *  maxRetries times with backoffMs between attempts. Each retry re-issues a
   *  fresh READY gate + the same request envelope (crudCall does both). A
   *  non-EngineUnavailable error is NOT retried. */
  const withRetry = async <T>(
    method: CrudMethod,
    args: CrudRequestArgs,
    id: string | undefined,
  ): Promise<T> => {
    const maxRetries = retry?.maxRetries ?? 0
    const backoffMs = retry?.backoffMs ?? 100
    if (maxRetries <= 0) return crudCall<T>(method, args, id)
    let lastErr: unknown
    for (let i = 0; i <= maxRetries; i++) {
      try {
        return await crudCall<T>(method, args, id)
      } catch (err) {
        if (!(err instanceof EngineUnavailable)) throw err
        lastErr = err
        if (i < maxRetries) await sleep(backoffMs)
      }
    }
    throw lastErr
  }

  return {
    createDocument: (args: CreateDocumentArgs) =>
      withRetry<Document>('createDocument', args, undefined),
    getDocument: (args: GetDocumentArgs) =>
      crudCall<Document>('getDocument', args, args.documentId),
    updateDocument: (args: UpdateDocumentArgs) =>
      crudCall<Document>('updateDocument', args, args.documentId),
    deleteDocument: async (args: DeleteDocumentArgs): Promise<void> => {
      await crudCall<null>('deleteDocument', args, args.documentId)
    },
    publishDocument: (args: PublishDocumentArgs) =>
      crudCall<Document>('publishDocument', args, args.documentId),
    unpublishDocument: (args: UnpublishDocumentArgs) =>
      crudCall<Document>('unpublishDocument', args, args.documentId),
    archiveDocument: (args: ArchiveDocumentArgs) =>
      crudCall<Document>('archiveDocument', args, args.documentId),
    listDocuments: (args: ListDocumentsArgs) =>
      crudCall<DocumentList>('listDocuments', args, undefined),
    createWiki: (args: CreateWikiArgs) =>
      withRetry<Wiki>('createWiki', args, undefined),
    getWiki: (args: GetWikiArgs) =>
      crudCall<Wiki>('getWiki', args, args.wikiId),
    listWikis: (args: ListWikisArgs) =>
      crudCall<Wiki[]>('listWikis', args, undefined),
  }
}
