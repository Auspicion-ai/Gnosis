// tests/unit-a1-crud-routing-proxy.test.ts — Unit A1: the document-CRUD routing
// proxy — the `createEngineCrudRagStore` proxy (the 11 §4.1 document-CRUD wire
// client) over the frozen document-CRUD wire (P1a).
// (docs/specs/unit-a1-crud-routing-proxy.md §5.8 happy-path states + §5.9
// fail-states + §5.2 golden-vector conformance + §5.4 HTTP-status rendering +
// NEW-2 + RBAC caller + §5.5 transport + §5.6 READY observation + D2
// engine-absent; the frozen wire shapes + golden vectors V-10..V-12 + §7
// endpoint paths + §8 RBAC caller + §10 decode-then-validate in
// ../Gnosis/docs/specs/p1a-document-crud-wire.md).
//
// The RED set for the NEW module `src/main/engine-crud-rag-store.ts`:
//   - the factory `createEngineCrudRagStore(opts)` + the 11-method proxy surface
//   - the encode+decode functions: encodeCrudRequest, decodeCrudRequest,
//     decodeCrudResponse, validateCrudResult
//   - the constant ENGINE_CRUD_ENDPOINTS (11 paths)
//   - the typed surface: EngineCrudRagStoreOptions, EngineCrudRagStore,
//     CrudMethod, CrudRequestArgs, CrudResult, CrudValidationFailure, Document,
//     DocumentList, Wiki, and the 11 per-method args types
//   - the §5.7 PBT register is executed in the sibling
//     tests/props-a1-crud-routing-proxy.test.ts
//
// Convention: follows the sibling `tests/unit-gn-engine-integration.test.ts`
// (vitest node environment, `.js` import suffix for the main-process ESM
// module). The shared error model / status map / decodeEnvelope are imported
// from `src/main/engine-rag-store.js` (Unit GN) — A1 imports, never redefines.
//
// RED: `src/main/engine-crud-rag-store.ts` does NOT exist yet, so the static
// import fails to resolve — the ENTIRE test set is the red set for the
// not-yet-implemented module (the file fails to load).
import { describe, it, expect } from 'vitest'
import {
  createEngineCrudRagStore,
  encodeCrudRequest,
  decodeCrudRequest,
  decodeCrudResponse,
  validateCrudResult,
  ENGINE_CRUD_ENDPOINTS,
  type EngineCrudRagStore,
  type CrudMethod,
  type CrudRequestArgs,
  type CrudResult,
  type Document,
  type DocumentList,
  type Wiki,
} from '../src/main/engine-crud-rag-store.js'
import {
  EngineWireError,
  EngineUnavailable,
  EngineError,
  TraceUnavailable,
  ConflictError,
  ENGINE_HTTP_STATUS,
  ENGINE_ENDPOINTS,
  decodeEnvelope,
  type EngineErrorCode,
} from '../src/main/engine-rag-store.js'

// ---------------------------------------------------------------------------
// Golden-vector constants (P1a §9 — the byte-exact conformance targets).
// ---------------------------------------------------------------------------

const V10_ENVELOPE =
  '{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"method":"createDocument","args":{"caller":"user:alice","wikiId":"w1","body":{"title":"Getting Started","tags":["guide"],"author":"alice"}}}}'

const V11_ENVELOPE =
  '{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"method":"createDocument","result":{"document_id":"d1","wiki_id":"w1","revision":0,"state":"Draft","graph":{"nodes":[],"edges":[]},"title":"Getting Started","created_at":"2026-09-09T00:00:00Z","updated_at":"2026-09-09T00:00:00Z","tags":["guide"],"author":"alice"}}}'

const V12_ENVELOPE =
  '{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"method":"updateDocument","error":{"code":"conflict","message":"optimistic-concurrency conflict: stale base revision"}}}'

// ---------------------------------------------------------------------------
// The READY health report (the gate the fake fetch serves).
// ---------------------------------------------------------------------------

const READY_REPORT = {
  schemaVersion: 1,
  idFormat: 'opaque-string-v1',
  state: 'Ready',
  version: '…',
  subsystems: {
    store: true,
    graph: true,
    lexical: true,
    vector: true,
    embedding: true,
    reranker: true,
  },
  lastError: null,
}

// ---------------------------------------------------------------------------
// The wire bodies + typed results (the §5.2 snake→camel projection).
// ---------------------------------------------------------------------------

const DOC_WIRE = {
  document_id: 'd1',
  wiki_id: 'w1',
  revision: 0,
  state: 'Draft',
  graph: { nodes: [], edges: [] },
  title: 'Getting Started',
  created_at: '2026-09-09T00:00:00Z',
  updated_at: '2026-09-09T00:00:00Z',
  tags: ['guide'],
  author: 'alice',
}

const DOC_TYPED: Document = {
  documentId: 'd1',
  wikiId: 'w1',
  revision: 0,
  state: 'Draft',
  graph: { nodes: [], edges: [] },
  title: 'Getting Started',
  createdAt: '2026-09-09T00:00:00Z',
  updatedAt: '2026-09-09T00:00:00Z',
  tags: ['guide'],
  author: 'alice',
}

const WIKI_WIRE = { wiki_id: 'w1', name: 'My Wiki' }
const WIKI_TYPED: Wiki = { wikiId: 'w1', name: 'My Wiki' }

const DOCLIST_WIRE = { items: [DOC_WIRE], total: 1, page: 1, page_size: 20 }
const DOCLIST_TYPED: DocumentList = {
  items: [DOC_TYPED],
  total: 1,
  page: 1,
  pageSize: 20,
}

// ---------------------------------------------------------------------------
// The §11 HTTP-status map (all 21 rows — the conformance target).
// ---------------------------------------------------------------------------

const ALL_21_CODES: EngineErrorCode[] = [
  'not_found',
  'wiki_not_found',
  'validation_error',
  'conflict',
  'doc_in_use',
  'invalid_state',
  'unresolved_reference',
  'engine_unavailable',
  'engine_error',
  'trace_unavailable',
  'hop_limit_exceeded',
  'cycle_detected',
  'embedding_unavailable',
  'vector_index_unavailable',
  'lexical_index_unavailable',
  'reranker_unavailable',
  'compression_failed',
  'hyde_generation_failed',
  'multi_query_expansion_failed',
  'community_not_found',
  'sub_task_dag_failed',
]

const EXPECTED_STATUS: Record<EngineErrorCode, number> = {
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

// ---------------------------------------------------------------------------
// The method sets + a representative args per method (for the pure surface).
// ---------------------------------------------------------------------------

const CRUD_METHODS: CrudMethod[] = [
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

const MUTATING_METHODS: CrudMethod[] = [
  'createDocument',
  'updateDocument',
  'deleteDocument',
  'publishDocument',
  'unpublishDocument',
  'archiveDocument',
  'createWiki',
]

const READ_ONLY_METHODS: CrudMethod[] = [
  'getDocument',
  'listDocuments',
  'getWiki',
  'listWikis',
]

const ALL_METHOD_ARGS: Record<CrudMethod, CrudRequestArgs> = {
  createDocument: {
    caller: 'user:alice',
    wikiId: 'w1',
    body: { title: 't', tags: null, author: null },
  },
  getDocument: { documentId: 'd1' },
  updateDocument: {
    caller: 'user:alice',
    documentId: 'd1',
    body: { baseRevision: 0, graph: { nodes: [], edges: [] }, title: null, tags: null },
  },
  deleteDocument: { caller: 'user:alice', documentId: 'd1' },
  publishDocument: { caller: 'user:alice', documentId: 'd1' },
  unpublishDocument: { caller: 'user:alice', documentId: 'd1' },
  archiveDocument: { caller: 'user:alice', documentId: 'd1' },
  listDocuments: {
    wikiId: 'w1',
    body: { state: null, tag: null, page: null, pageSize: null },
  },
  createWiki: { caller: 'user:alice', name: 'My Wiki' },
  getWiki: { wikiId: 'w1' },
  listWikis: {},
}

// ---------------------------------------------------------------------------
// Test helpers — fake fetch + the READY-gate-then-CRUD transport.
// ---------------------------------------------------------------------------

type FetchHandler = (
  method: string,
  url: string,
  init?: RequestInit,
) => Response | Promise<Response>

function fakeFetch(handler: FetchHandler): typeof fetch {
  return (async (input: any, init?: RequestInit) => {
    const url = typeof input === 'string' ? input : (input as Request).url
    const method = (init?.method || 'GET').toUpperCase()
    return handler(method, url, init)
  }) as typeof fetch
}

function jsonResponse(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'content-type': 'application/json' },
  })
}

function refusedError(): Error {
  const e: any = new Error('connect ECONNREFUSED 127.0.0.1:8080')
  e.code = 'ECONNREFUSED'
  return e
}

function notSpawnedError(): Error {
  const e: any = new Error('getaddrinfo ENOTFOUND 127.0.0.1:8080')
  e.code = 'ENOTFOUND'
  return e
}

interface CrudCall {
  method: string
  url: string
  body: unknown
}

/** Wrap a CRUD response payload in the full versioned envelope. */
function crudResponseEnvelope(payload: unknown): unknown {
  return { schemaVersion: 1, idFormat: 'opaque-string-v1', payload }
}

/** A fake fetch that serves a READY gate then a single CRUD endpoint response,
 *  capturing the CRUD request (verb, url, JSON request body). */
function readyGateThenCrud(
  crudHandler: (method: string, url: string, body: unknown) => Response,
  calls: CrudCall[] = [],
): typeof fetch {
  return fakeFetch((method, url, init) => {
    if (method === 'GET' && url.endsWith(ENGINE_ENDPOINTS.engineStatus)) {
      return jsonResponse(READY_REPORT)
    }
    const body = init?.body ? JSON.parse(init.body as string) : undefined
    calls.push({ method, url, body })
    return crudHandler(method, url, body)
  })
}

/** A store whose CRUD endpoint always returns the given response. */
function crudStoreWithResponse(response: Response): EngineCrudRagStore {
  return createEngineCrudRagStore({
    baseUrl: 'http://127.0.0.1:8080',
    fetch: readyGateThenCrud(() => response),
  })
}

/** Await a promise and return the rejection (throws if it resolves). */
async function captureError(p: Promise<unknown>): Promise<unknown> {
  try {
    await p
  } catch (e) {
    return e
  }
  throw new Error('expected the promise to reject')
}

// ---------------------------------------------------------------------------
// §5.8 happy-path states (1, 14) + §5.9 fail-states (1–3) + §5.10 census.
// ---------------------------------------------------------------------------

describe('createEngineCrudRagStore — factory + proxy surface', () => {
  it('§5.8-1 factory happy: returns a proxy; no network I/O at construction', () => {
    let fetchCalled = false
    const fetchImpl = (async () => {
      fetchCalled = true
      throw new Error('should not be called')
    }) as typeof fetch
    const store = createEngineCrudRagStore({
      baseUrl: 'http://127.0.0.1:8080',
      fetch: fetchImpl,
    })
    expect(store).toBeTruthy()
    for (const m of CRUD_METHODS) {
      expect(typeof (store as any)[m]).toBe('function')
    }
    expect(fetchCalled).toBe(false)
  })

  it('§5.8-14 loopback enforcement: 127.0.0.1 / ::1 / localhost OK; non-loopback throws', () => {
    expect(() =>
      createEngineCrudRagStore({ baseUrl: 'http://127.0.0.1:8080' }),
    ).not.toThrow()
    expect(() =>
      createEngineCrudRagStore({ baseUrl: 'http://[::1]:8080' }),
    ).not.toThrow()
    expect(() =>
      createEngineCrudRagStore({ baseUrl: 'http://localhost:8080' }),
    ).not.toThrow()
  })

  it('§5.9-1 fail: null/undefined opts throws', () => {
    expect(() => createEngineCrudRagStore(null as any)).toThrow(
      'engine crud rag store: opts required',
    )
    expect(() => createEngineCrudRagStore(undefined as any)).toThrow(
      'engine crud rag store: opts required',
    )
  })

  it('§5.9-2 fail: missing/empty baseUrl throws', () => {
    expect(() => createEngineCrudRagStore({} as any)).toThrow(
      'engine crud rag store: baseUrl required',
    )
    expect(() => createEngineCrudRagStore({ baseUrl: '' } as any)).toThrow(
      'engine crud rag store: baseUrl required',
    )
  })

  it('§5.9-3 fail: non-loopback baseUrl throws', () => {
    expect(() =>
      createEngineCrudRagStore({ baseUrl: 'http://192.168.1.10:8080' }),
    ).toThrow('engine crud rag store: baseUrl must be loopback')
    expect(() =>
      createEngineCrudRagStore({ baseUrl: 'http://example.com:8080' }),
    ).toThrow('engine crud rag store: baseUrl must be loopback')
  })

  it('§5.10 census: ENGINE_CRUD_ENDPOINTS pins the 11 paths', () => {
    expect(ENGINE_CRUD_ENDPOINTS).toEqual({
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
    })
  })
})

// ---------------------------------------------------------------------------
// §5.8 happy-path states (2–12) — each method's request → typed result, with
// the §5.5 transport (verb + path + request envelope in the JSON request body).
// ---------------------------------------------------------------------------

describe('the 11 CRUD methods — happy paths (§5.8-2..12)', () => {
  it('§5.8-2 happy: createDocument POST /documents → typed Document', async () => {
    const calls: CrudCall[] = []
    const store = createEngineCrudRagStore({
      baseUrl: 'http://127.0.0.1:8080',
      fetch: readyGateThenCrud((method, url) => {
        expect(method).toBe('POST')
        expect(url).toBe('http://127.0.0.1:8080/documents')
        return jsonResponse(
          crudResponseEnvelope({ method: 'createDocument', result: DOC_WIRE }),
        )
      }, calls),
    })
    const doc = await store.createDocument({
      caller: 'user:alice',
      wikiId: 'w1',
      body: { title: 'Getting Started', tags: ['guide'], author: 'alice' },
    })
    expect(doc).toEqual(DOC_TYPED)
    expect(calls.length).toBe(1)
    expect(calls[0].body).toEqual({
      schemaVersion: 1,
      idFormat: 'opaque-string-v1',
      payload: {
        method: 'createDocument',
        args: {
          caller: 'user:alice',
          wikiId: 'w1',
          body: { title: 'Getting Started', tags: ['guide'], author: 'alice' },
        },
      },
    })
  })

  it('§5.8-3 happy: getDocument GET /documents/:id → typed Document', async () => {
    const calls: CrudCall[] = []
    const store = createEngineCrudRagStore({
      baseUrl: 'http://127.0.0.1:8080',
      fetch: readyGateThenCrud((method, url) => {
        expect(method).toBe('GET')
        expect(url).toBe('http://127.0.0.1:8080/documents/d1')
        return jsonResponse(
          crudResponseEnvelope({ method: 'getDocument', result: DOC_WIRE }),
        )
      }, calls),
    })
    const doc = await store.getDocument({ documentId: 'd1' })
    expect(doc).toEqual(DOC_TYPED)
    expect(calls.length).toBe(1)
    expect(calls[0].body).toEqual({
      schemaVersion: 1,
      idFormat: 'opaque-string-v1',
      payload: { method: 'getDocument', args: { documentId: 'd1' } },
    })
  })

  it('§5.8-4 happy: updateDocument POST /documents/:id/update → typed Document', async () => {
    const calls: CrudCall[] = []
    const store = createEngineCrudRagStore({
      baseUrl: 'http://127.0.0.1:8080',
      fetch: readyGateThenCrud((method, url) => {
        expect(method).toBe('POST')
        expect(url).toBe('http://127.0.0.1:8080/documents/d1/update')
        return jsonResponse(
          crudResponseEnvelope({ method: 'updateDocument', result: DOC_WIRE }),
        )
      }, calls),
    })
    const doc = await store.updateDocument({
      caller: 'user:alice',
      documentId: 'd1',
      body: {
        baseRevision: 0,
        graph: { nodes: [], edges: [] },
        title: 'Getting Started',
        tags: ['guide'],
      },
    })
    expect(doc).toEqual(DOC_TYPED)
    expect(calls.length).toBe(1)
    expect(calls[0].body).toEqual({
      schemaVersion: 1,
      idFormat: 'opaque-string-v1',
      payload: {
        method: 'updateDocument',
        args: {
          caller: 'user:alice',
          documentId: 'd1',
          body: {
            base_revision: 0,
            graph: { nodes: [], edges: [] },
            title: 'Getting Started',
            tags: ['guide'],
          },
        },
      },
    })
  })

  it('§5.8-5 happy: deleteDocument DELETE /documents/:id → void', async () => {
    const calls: CrudCall[] = []
    const store = createEngineCrudRagStore({
      baseUrl: 'http://127.0.0.1:8080',
      fetch: readyGateThenCrud((method, url) => {
        expect(method).toBe('DELETE')
        expect(url).toBe('http://127.0.0.1:8080/documents/d1')
        return jsonResponse(
          crudResponseEnvelope({ method: 'deleteDocument', result: null }),
        )
      }, calls),
    })
    const result = await store.deleteDocument({
      caller: 'user:alice',
      documentId: 'd1',
    })
    expect(result).toBeUndefined()
    expect(calls.length).toBe(1)
    expect(calls[0].body).toEqual({
      schemaVersion: 1,
      idFormat: 'opaque-string-v1',
      payload: {
        method: 'deleteDocument',
        args: { caller: 'user:alice', documentId: 'd1' },
      },
    })
  })

  it('§5.8-6 happy: publishDocument POST /documents/:id/publish → typed Document (Published)', async () => {
    const calls: CrudCall[] = []
    const store = createEngineCrudRagStore({
      baseUrl: 'http://127.0.0.1:8080',
      fetch: readyGateThenCrud((method, url) => {
        expect(method).toBe('POST')
        expect(url).toBe('http://127.0.0.1:8080/documents/d1/publish')
        return jsonResponse(
          crudResponseEnvelope({
            method: 'publishDocument',
            result: { ...DOC_WIRE, state: 'Published' },
          }),
        )
      }, calls),
    })
    const doc = await store.publishDocument({
      caller: 'user:alice',
      documentId: 'd1',
    })
    expect(doc).toEqual({ ...DOC_TYPED, state: 'Published' })
    expect(calls.length).toBe(1)
    expect(calls[0].body).toEqual({
      schemaVersion: 1,
      idFormat: 'opaque-string-v1',
      payload: {
        method: 'publishDocument',
        args: { caller: 'user:alice', documentId: 'd1' },
      },
    })
  })

  it('§5.8-7 happy: unpublishDocument POST /documents/:id/unpublish → typed Document (Draft)', async () => {
    const calls: CrudCall[] = []
    const store = createEngineCrudRagStore({
      baseUrl: 'http://127.0.0.1:8080',
      fetch: readyGateThenCrud((method, url) => {
        expect(method).toBe('POST')
        expect(url).toBe('http://127.0.0.1:8080/documents/d1/unpublish')
        return jsonResponse(
          crudResponseEnvelope({
            method: 'unpublishDocument',
            result: { ...DOC_WIRE, state: 'Draft' },
          }),
        )
      }, calls),
    })
    const doc = await store.unpublishDocument({
      caller: 'user:alice',
      documentId: 'd1',
    })
    expect(doc).toEqual({ ...DOC_TYPED, state: 'Draft' })
    expect(calls.length).toBe(1)
    expect(calls[0].body).toEqual({
      schemaVersion: 1,
      idFormat: 'opaque-string-v1',
      payload: {
        method: 'unpublishDocument',
        args: { caller: 'user:alice', documentId: 'd1' },
      },
    })
  })

  it('§5.8-8 happy: archiveDocument POST /documents/:id/archive → typed Document (Archived)', async () => {
    const calls: CrudCall[] = []
    const store = createEngineCrudRagStore({
      baseUrl: 'http://127.0.0.1:8080',
      fetch: readyGateThenCrud((method, url) => {
        expect(method).toBe('POST')
        expect(url).toBe('http://127.0.0.1:8080/documents/d1/archive')
        return jsonResponse(
          crudResponseEnvelope({
            method: 'archiveDocument',
            result: { ...DOC_WIRE, state: 'Archived' },
          }),
        )
      }, calls),
    })
    const doc = await store.archiveDocument({
      caller: 'user:alice',
      documentId: 'd1',
    })
    expect(doc).toEqual({ ...DOC_TYPED, state: 'Archived' })
    expect(calls.length).toBe(1)
    expect(calls[0].body).toEqual({
      schemaVersion: 1,
      idFormat: 'opaque-string-v1',
      payload: {
        method: 'archiveDocument',
        args: { caller: 'user:alice', documentId: 'd1' },
      },
    })
  })

  it('§5.8-9 happy: listDocuments GET /documents → typed DocumentList', async () => {
    const calls: CrudCall[] = []
    const store = createEngineCrudRagStore({
      baseUrl: 'http://127.0.0.1:8080',
      fetch: readyGateThenCrud((method, url) => {
        expect(method).toBe('GET')
        expect(url).toBe('http://127.0.0.1:8080/documents')
        return jsonResponse(
          crudResponseEnvelope({ method: 'listDocuments', result: DOCLIST_WIRE }),
        )
      }, calls),
    })
    const list = await store.listDocuments({
      wikiId: 'w1',
      body: { state: null, tag: null, page: 1, pageSize: 20 },
    })
    expect(list).toEqual(DOCLIST_TYPED)
    expect(calls.length).toBe(1)
    expect(calls[0].body).toEqual({
      schemaVersion: 1,
      idFormat: 'opaque-string-v1',
      payload: {
        method: 'listDocuments',
        args: {
          wikiId: 'w1',
          body: { state: null, tag: null, page: 1, page_size: 20 },
        },
      },
    })
  })

  it('§5.8-10 happy: createWiki POST /wikis → typed Wiki', async () => {
    const calls: CrudCall[] = []
    const store = createEngineCrudRagStore({
      baseUrl: 'http://127.0.0.1:8080',
      fetch: readyGateThenCrud((method, url) => {
        expect(method).toBe('POST')
        expect(url).toBe('http://127.0.0.1:8080/wikis')
        return jsonResponse(
          crudResponseEnvelope({ method: 'createWiki', result: WIKI_WIRE }),
        )
      }, calls),
    })
    const wiki = await store.createWiki({ caller: 'user:alice', name: 'My Wiki' })
    expect(wiki).toEqual(WIKI_TYPED)
    expect(calls.length).toBe(1)
    expect(calls[0].body).toEqual({
      schemaVersion: 1,
      idFormat: 'opaque-string-v1',
      payload: {
        method: 'createWiki',
        args: { caller: 'user:alice', name: 'My Wiki' },
      },
    })
  })

  it('§5.8-11 happy: getWiki GET /wikis/:id → typed Wiki', async () => {
    const calls: CrudCall[] = []
    const store = createEngineCrudRagStore({
      baseUrl: 'http://127.0.0.1:8080',
      fetch: readyGateThenCrud((method, url) => {
        expect(method).toBe('GET')
        expect(url).toBe('http://127.0.0.1:8080/wikis/w1')
        return jsonResponse(
          crudResponseEnvelope({ method: 'getWiki', result: WIKI_WIRE }),
        )
      }, calls),
    })
    const wiki = await store.getWiki({ wikiId: 'w1' })
    expect(wiki).toEqual(WIKI_TYPED)
    expect(calls.length).toBe(1)
    expect(calls[0].body).toEqual({
      schemaVersion: 1,
      idFormat: 'opaque-string-v1',
      payload: { method: 'getWiki', args: { wikiId: 'w1' } },
    })
  })

  it('§5.8-12 happy: listWikis GET /wikis → typed Wiki[]', async () => {
    const calls: CrudCall[] = []
    const store = createEngineCrudRagStore({
      baseUrl: 'http://127.0.0.1:8080',
      fetch: readyGateThenCrud((method, url) => {
        expect(method).toBe('GET')
        expect(url).toBe('http://127.0.0.1:8080/wikis')
        return jsonResponse(
          crudResponseEnvelope({ method: 'listWikis', result: [WIKI_WIRE] }),
        )
      }, calls),
    })
    const wikis = await store.listWikis({})
    expect(wikis).toEqual([WIKI_TYPED])
    expect(calls.length).toBe(1)
    expect(calls[0].body).toEqual({
      schemaVersion: 1,
      idFormat: 'opaque-string-v1',
      payload: { method: 'listWikis', args: {} },
    })
  })
})

// ---------------------------------------------------------------------------
// §5.2 golden-vector conformance (V-10..V-12 — the byte-exact targets).
// ---------------------------------------------------------------------------

describe('golden-vector conformance (§5.2 / P1a §9)', () => {
  it('V-10: encodeCrudRequest emits the exact bytes of V-10', () => {
    const env = encodeCrudRequest('createDocument', {
      caller: 'user:alice',
      wikiId: 'w1',
      body: { title: 'Getting Started', tags: ['guide'], author: 'alice' },
    })
    expect(JSON.stringify(env)).toBe(V10_ENVELOPE)
  })

  it('V-11: decodeCrudResponse(decodeEnvelope(V-11).payload) → pinned typed Document', () => {
    const env = decodeEnvelope(V11_ENVELOPE)
    const result = decodeCrudResponse(env.payload)
    expect(result).toEqual({ method: 'createDocument', result: DOC_TYPED })
  })

  it('V-12: decodeCrudResponse(decodeEnvelope(V-12).payload) throws ConflictError (409)', () => {
    const env = decodeEnvelope(V12_ENVELOPE)
    expect(() => decodeCrudResponse(env.payload)).toThrow(ConflictError)
    try {
      decodeCrudResponse(env.payload)
    } catch (e) {
      expect(e).toBeInstanceOf(ConflictError)
      expect((e as ConflictError).code).toBe('conflict')
      expect((e as ConflictError).httpStatus).toBe(409)
    }
  })
})

// ---------------------------------------------------------------------------
// §5.4 HTTP-status rendering (fail-state 12 — the total 21-code translation).
// ---------------------------------------------------------------------------

describe('HTTP-status rendering (§5.4 / fail-state 12)', () => {
  it('§5.9-12 fail: total 21-code wire-code translation', () => {
    for (const code of ALL_21_CODES) {
      let thrown: unknown
      try {
        decodeCrudResponse({ method: 'getDocument', error: { code, message: 'm' } })
      } catch (e) {
        thrown = e
      }
      expect(thrown).toBeInstanceOf(EngineWireError)
      const err = thrown as EngineWireError
      expect(err.code).toBe(code)
      expect(err.httpStatus).toBe(EXPECTED_STATUS[code])
    }
  })

  it('§5.9-12 fail: representative 10 codes → matching typed error via the proxy', async () => {
    const cases: Array<[string, { code: string; httpStatus: number; cause?: string }]> = [
      ['not_found', { code: 'not_found', httpStatus: 404 }],
      ['wiki_not_found', { code: 'wiki_not_found', httpStatus: 404 }],
      ['validation_error', { code: 'validation_error', httpStatus: 400 }],
      ['conflict', { code: 'conflict', httpStatus: 409 }],
      ['doc_in_use', { code: 'doc_in_use', httpStatus: 409 }],
      ['invalid_state', { code: 'invalid_state', httpStatus: 409 }],
      ['unresolved_reference', { code: 'unresolved_reference', httpStatus: 422 }],
      ['engine_unavailable', { code: 'engine_unavailable', httpStatus: 503, cause: 'unavailable-state' }],
      ['engine_error', { code: 'engine_error', httpStatus: 502 }],
      ['trace_unavailable', { code: 'trace_unavailable', httpStatus: 502 }],
    ]
    for (const [code, expected] of cases) {
      const store = crudStoreWithResponse(
        jsonResponse(
          crudResponseEnvelope({ method: 'getDocument', error: { code, message: 'm' } }),
        ),
      )
      const err = await captureError(store.getDocument({ documentId: 'd1' }))
      expect(err).toBeInstanceOf(EngineWireError)
      expect(err).toMatchObject(expected)
    }
  })
})

// ---------------------------------------------------------------------------
// §5.4 RBAC caller threading (happy state 16).
// ---------------------------------------------------------------------------

describe('RBAC caller threading (§5.4 / happy 16)', () => {
  it('§5.8-16 happy: mutating requests carry caller; read-only requests carry no caller', () => {
    for (const method of MUTATING_METHODS) {
      const env = encodeCrudRequest(method, ALL_METHOD_ARGS[method])
      const args = env.payload.args as Record<string, unknown>
      expect(args.caller).toBe('user:alice')
    }
    for (const method of READ_ONLY_METHODS) {
      const env = encodeCrudRequest(method, ALL_METHOD_ARGS[method])
      const args = env.payload.args as Record<string, unknown>
      expect('caller' in args).toBe(false)
    }
  })
})

// ---------------------------------------------------------------------------
// §5.3 decode-then-validate surface (the pure functions).
// ---------------------------------------------------------------------------

describe('decode-then-validate surface (§5.3)', () => {
  it('validateCrudResult: valid results → null; invalid → the failure variant', () => {
    // Valid (well-formed, invariants hold).
    expect(
      validateCrudResult('createDocument', { method: 'createDocument', result: DOC_TYPED }),
    ).toBeNull()
    expect(
      validateCrudResult('getDocument', { method: 'getDocument', result: DOC_TYPED }),
    ).toBeNull()
    expect(
      validateCrudResult('updateDocument', { method: 'updateDocument', result: DOC_TYPED }),
    ).toBeNull()
    expect(
      validateCrudResult('deleteDocument', { method: 'deleteDocument', result: null }),
    ).toBeNull()
    expect(
      validateCrudResult('publishDocument', {
        method: 'publishDocument',
        result: { ...DOC_TYPED, state: 'Published' },
      }),
    ).toBeNull()
    expect(
      validateCrudResult('unpublishDocument', {
        method: 'unpublishDocument',
        result: { ...DOC_TYPED, state: 'Draft' },
      }),
    ).toBeNull()
    expect(
      validateCrudResult('archiveDocument', {
        method: 'archiveDocument',
        result: { ...DOC_TYPED, state: 'Archived' },
      }),
    ).toBeNull()
    expect(
      validateCrudResult('listDocuments', { method: 'listDocuments', result: DOCLIST_TYPED }),
    ).toBeNull()
    expect(
      validateCrudResult('createWiki', { method: 'createWiki', result: WIKI_TYPED }),
    ).toBeNull()
    expect(
      validateCrudResult('getWiki', { method: 'getWiki', result: WIKI_TYPED }),
    ).toBeNull()
    expect(
      validateCrudResult('listWikis', { method: 'listWikis', result: [WIKI_TYPED] }),
    ).toBeNull()

    // Invalid (a CRUD-specific invariant fails).
    expect(
      validateCrudResult('createDocument', {
        method: 'createDocument',
        result: { ...DOC_TYPED, revision: 1 },
      }),
    ).toMatchObject({ kind: 'UnexpectedRevision' })
    expect(
      validateCrudResult('createDocument', {
        method: 'createDocument',
        result: { ...DOC_TYPED, state: 'Published' },
      }),
    ).toMatchObject({ kind: 'UnexpectedState' })
    expect(
      validateCrudResult('publishDocument', {
        method: 'publishDocument',
        result: { ...DOC_TYPED, state: 'Draft' },
      }),
    ).toMatchObject({ kind: 'UnexpectedState' })
    expect(
      validateCrudResult('unpublishDocument', {
        method: 'unpublishDocument',
        result: { ...DOC_TYPED, state: 'Published' },
      }),
    ).toMatchObject({ kind: 'UnexpectedState' })
    expect(
      validateCrudResult('archiveDocument', {
        method: 'archiveDocument',
        result: { ...DOC_TYPED, state: 'Draft' },
      }),
    ).toMatchObject({ kind: 'UnexpectedState' })
    expect(
      validateCrudResult('listDocuments', {
        method: 'listDocuments',
        result: { ...DOCLIST_TYPED, page: 0 },
      }),
    ).toMatchObject({ kind: 'InvalidPagination' })
    expect(
      validateCrudResult('listDocuments', {
        method: 'listDocuments',
        result: { ...DOCLIST_TYPED, pageSize: 0 },
      }),
    ).toMatchObject({ kind: 'InvalidPagination' })
    expect(
      validateCrudResult('listDocuments', {
        method: 'listDocuments',
        result: { ...DOCLIST_TYPED, pageSize: 101 },
      }),
    ).toMatchObject({ kind: 'InvalidPagination' })
    expect(
      validateCrudResult('deleteDocument', {
        method: 'deleteDocument',
        result: DOC_TYPED,
      }),
    ).toMatchObject({ kind: 'UnexpectedVoid' })
  })

  it('decodeCrudRequest is the inverse of encodeCrudRequest (representative)', () => {
    for (const method of CRUD_METHODS) {
      const args = ALL_METHOD_ARGS[method]
      const decoded = decodeCrudRequest(encodeCrudRequest(method, args))
      expect(decoded.method).toBe(method)
      expect(JSON.stringify(decoded.args)).toBe(JSON.stringify(args))
    }
  })

  it('encodeCrudRequest is total over the 11 methods (NEW-2 awareness: never an unknown method)', () => {
    for (const method of CRUD_METHODS) {
      const env = encodeCrudRequest(method, ALL_METHOD_ARGS[method])
      expect(env.schemaVersion).toBe(1)
      expect(env.idFormat).toBe('opaque-string-v1')
      expect(env.payload).toMatchObject({ method })
      expect(env.payload).toHaveProperty('args')
    }
  })
})

// ---------------------------------------------------------------------------
// §5.5 transport — the :id substitution + URL-encoding.
// ---------------------------------------------------------------------------

describe('transport (§5.5)', () => {
  it(':id substitution URL-encodes the documentId/wikiId', async () => {
    const calls: CrudCall[] = []
    const store = createEngineCrudRagStore({
      baseUrl: 'http://127.0.0.1:8080',
      fetch: readyGateThenCrud((method, url) => {
        expect(method).toBe('GET')
        expect(url).toBe('http://127.0.0.1:8080/documents/a%20b%2Fc')
        return jsonResponse(
          crudResponseEnvelope({ method: 'getDocument', result: DOC_WIRE }),
        )
      }, calls),
    })
    await store.getDocument({ documentId: 'a b/c' })
    expect(calls.length).toBe(1)
  })
})

// ---------------------------------------------------------------------------
// §5.9 fail-states (4–18) + the P4 retry.
// ---------------------------------------------------------------------------

describe('fail-states (§5.9)', () => {
  it('§5.9-4 fail: observed state not Ready → EngineUnavailable (not-ready / unavailable-state)', async () => {
    for (const [state, cause, lastError] of [
      ['Starting', 'not-ready', null],
      // A VALID Degraded report (spec §5.6: lastError is Some exactly when
      // Degraded) — the gate must see Degraded → EngineUnavailable, not
      // EngineError from the faithfulness check.
      ['Degraded', 'not-ready', 'a non-core subsystem (embedding/reranker) is unavailable'],
      ['Unavailable', 'unavailable-state', null],
    ] as const) {
      let crudCalled = false
      const store = createEngineCrudRagStore({
        baseUrl: 'http://127.0.0.1:8080',
        fetch: fakeFetch((method, url) => {
          if (method === 'GET' && url.endsWith(ENGINE_ENDPOINTS.engineStatus)) {
            return jsonResponse({ ...READY_REPORT, state, lastError })
          }
          crudCalled = true
          throw new Error(`unexpected ${method} ${url}`)
        }),
      })
      const err = await captureError(store.getDocument({ documentId: 'd1' }))
      expect(err).toBeInstanceOf(EngineUnavailable)
      expect(err).toMatchObject({
        code: 'engine_unavailable',
        httpStatus: 503,
        cause,
      })
      expect(crudCalled).toBe(false)
    }
  })

  it('§5.9-5 fail: connection-refused → EngineUnavailable (503, connection-refused)', async () => {
    const store = createEngineCrudRagStore({
      baseUrl: 'http://127.0.0.1:8080',
      fetch: fakeFetch(() => {
        throw refusedError()
      }),
    })
    const err = await captureError(store.getDocument({ documentId: 'd1' }))
    expect(err).toBeInstanceOf(EngineUnavailable)
    expect(err).toMatchObject({
      code: 'engine_unavailable',
      httpStatus: 503,
      cause: 'connection-refused',
    })
  })

  it('§5.9-6 fail: engine-not-spawned → EngineUnavailable (503, engine-not-spawned)', async () => {
    const store = createEngineCrudRagStore({
      baseUrl: 'http://127.0.0.1:8080',
      fetch: fakeFetch(() => {
        throw notSpawnedError()
      }),
    })
    const err = await captureError(store.getDocument({ documentId: 'd1' }))
    expect(err).toBeInstanceOf(EngineUnavailable)
    expect(err).toMatchObject({
      code: 'engine_unavailable',
      httpStatus: 503,
      cause: 'engine-not-spawned',
    })
  })

  it('§5.9-7 fail: malformed response envelope → EngineError (502)', async () => {
    // Missing schemaVersion/idFormat/payload.
    const store1 = crudStoreWithResponse(jsonResponse({ payload: {} }))
    const err1 = await captureError(store1.getDocument({ documentId: 'd1' }))
    expect(err1).toBeInstanceOf(EngineError)
    expect(err1).toMatchObject({ code: 'engine_error', httpStatus: 502 })
    // Invalid JSON body.
    const store2 = crudStoreWithResponse(
      new Response('not-json', {
        status: 200,
        headers: { 'content-type': 'application/json' },
      }),
    )
    const err2 = await captureError(store2.getDocument({ documentId: 'd1' }))
    expect(err2).toBeInstanceOf(EngineError)
    expect(err2).toMatchObject({ code: 'engine_error', httpStatus: 502 })
  })

  it('§5.9-8 fail: unknown schemaVersion (99) → EngineError (502)', async () => {
    const store = crudStoreWithResponse(
      jsonResponse({
        schemaVersion: 99,
        idFormat: 'opaque-string-v1',
        payload: { method: 'getDocument', result: DOC_WIRE },
      }),
    )
    const err = await captureError(store.getDocument({ documentId: 'd1' }))
    expect(err).toBeInstanceOf(EngineError)
    expect(err).toMatchObject({ code: 'engine_error', httpStatus: 502 })
  })

  it('§5.9-9 fail: unknown idFormat ("uuid-v4") → EngineError (502)', async () => {
    const store = crudStoreWithResponse(
      jsonResponse({
        schemaVersion: 1,
        idFormat: 'uuid-v4',
        payload: { method: 'getDocument', result: DOC_WIRE },
      }),
    )
    const err = await captureError(store.getDocument({ documentId: 'd1' }))
    expect(err).toBeInstanceOf(EngineError)
    expect(err).toMatchObject({ code: 'engine_error', httpStatus: 502 })
  })

  it('§5.9-10 fail: missing/unknown method discriminator → EngineError (502)', async () => {
    const missing = crudStoreWithResponse(
      jsonResponse(crudResponseEnvelope({ result: DOC_WIRE })),
    )
    const err1 = await captureError(missing.getDocument({ documentId: 'd1' }))
    expect(err1).toBeInstanceOf(EngineError)
    expect(err1).toMatchObject({ code: 'engine_error', httpStatus: 502 })

    const unknown = crudStoreWithResponse(
      jsonResponse(crudResponseEnvelope({ method: 'bogus', result: DOC_WIRE })),
    )
    const err2 = await captureError(unknown.getDocument({ documentId: 'd1' }))
    expect(err2).toBeInstanceOf(EngineError)
    expect(err2).toMatchObject({ code: 'engine_error', httpStatus: 502 })
  })

  it('§5.9-11 fail: both result and error, or neither → EngineError (502)', async () => {
    const both = crudStoreWithResponse(
      jsonResponse(
        crudResponseEnvelope({
          method: 'getDocument',
          result: DOC_WIRE,
          error: { code: 'conflict', message: 'x' },
        }),
      ),
    )
    const err1 = await captureError(both.getDocument({ documentId: 'd1' }))
    expect(err1).toBeInstanceOf(EngineError)
    expect(err1).toMatchObject({ code: 'engine_error', httpStatus: 502 })

    const neither = crudStoreWithResponse(
      jsonResponse(crudResponseEnvelope({ method: 'getDocument' })),
    )
    const err2 = await captureError(neither.getDocument({ documentId: 'd1' }))
    expect(err2).toBeInstanceOf(EngineError)
    expect(err2).toMatchObject({ code: 'engine_error', httpStatus: 502 })
  })

  it('§5.9-13 fail: unknown wire code → EngineError (502)', async () => {
    const store = crudStoreWithResponse(
      jsonResponse(
        crudResponseEnvelope({
          method: 'getDocument',
          error: { code: 'some_foreign_code', message: 'x' },
        }),
      ),
    )
    const err = await captureError(store.getDocument({ documentId: 'd1' }))
    expect(err).toBeInstanceOf(EngineError)
    expect(err).toMatchObject({ code: 'engine_error', httpStatus: 502 })
  })

  it('§5.9-14 fail: malformed result body (wrong field types) → EngineError (502)', async () => {
    const store = crudStoreWithResponse(
      jsonResponse(
        crudResponseEnvelope({ method: 'getDocument', result: { document_id: 123 } }),
      ),
    )
    const err = await captureError(store.getDocument({ documentId: 'd1' }))
    expect(err).toBeInstanceOf(EngineError)
    expect(err).toMatchObject({ code: 'engine_error', httpStatus: 502 })
  })

  it('§5.9-15a fail: createDocument revision != 0 → EngineError (UnexpectedRevision)', async () => {
    const store = crudStoreWithResponse(
      jsonResponse(
        crudResponseEnvelope({
          method: 'createDocument',
          result: { ...DOC_WIRE, revision: 1 },
        }),
      ),
    )
    const err = await captureError(
      store.createDocument({
        caller: 'user:alice',
        wikiId: 'w1',
        body: { title: 't', tags: null, author: null },
      }),
    )
    expect(err).toBeInstanceOf(EngineError)
    expect(err).toMatchObject({ code: 'engine_error', httpStatus: 502 })
    expect((err as Error).message).toMatch(/unexpected revision/i)
  })

  it('§5.9-15b fail: createDocument state != Draft → EngineError (UnexpectedState)', async () => {
    const store = crudStoreWithResponse(
      jsonResponse(
        crudResponseEnvelope({
          method: 'createDocument',
          result: { ...DOC_WIRE, state: 'Published' },
        }),
      ),
    )
    const err = await captureError(
      store.createDocument({
        caller: 'user:alice',
        wikiId: 'w1',
        body: { title: 't', tags: null, author: null },
      }),
    )
    expect(err).toBeInstanceOf(EngineError)
    expect(err).toMatchObject({ code: 'engine_error', httpStatus: 502 })
    expect((err as Error).message).toMatch(/unexpected state/i)
  })

  it('§5.9-15c fail: publishDocument state != Published → EngineError (UnexpectedState)', async () => {
    const store = crudStoreWithResponse(
      jsonResponse(
        crudResponseEnvelope({
          method: 'publishDocument',
          result: { ...DOC_WIRE, state: 'Draft' },
        }),
      ),
    )
    const err = await captureError(
      store.publishDocument({ caller: 'user:alice', documentId: 'd1' }),
    )
    expect(err).toBeInstanceOf(EngineError)
    expect(err).toMatchObject({ code: 'engine_error', httpStatus: 502 })
    expect((err as Error).message).toMatch(/unexpected state/i)
  })

  it('§5.9-15d fail: unpublishDocument state != Draft → EngineError (UnexpectedState)', async () => {
    const store = crudStoreWithResponse(
      jsonResponse(
        crudResponseEnvelope({
          method: 'unpublishDocument',
          result: { ...DOC_WIRE, state: 'Published' },
        }),
      ),
    )
    const err = await captureError(
      store.unpublishDocument({ caller: 'user:alice', documentId: 'd1' }),
    )
    expect(err).toBeInstanceOf(EngineError)
    expect(err).toMatchObject({ code: 'engine_error', httpStatus: 502 })
    expect((err as Error).message).toMatch(/unexpected state/i)
  })

  it('§5.9-15e fail: archiveDocument state != Archived → EngineError (UnexpectedState)', async () => {
    const store = crudStoreWithResponse(
      jsonResponse(
        crudResponseEnvelope({
          method: 'archiveDocument',
          result: { ...DOC_WIRE, state: 'Draft' },
        }),
      ),
    )
    const err = await captureError(
      store.archiveDocument({ caller: 'user:alice', documentId: 'd1' }),
    )
    expect(err).toBeInstanceOf(EngineError)
    expect(err).toMatchObject({ code: 'engine_error', httpStatus: 502 })
    expect((err as Error).message).toMatch(/unexpected state/i)
  })

  it('§5.9-15f fail: listDocuments invalid pagination → EngineError (InvalidPagination)', async () => {
    for (const result of [
      { items: [], total: 0, page: 0, page_size: 20 },
      { items: [], total: 0, page: 1, page_size: 0 },
      { items: [], total: 0, page: 1, page_size: 101 },
    ]) {
      const store = crudStoreWithResponse(
        jsonResponse(crudResponseEnvelope({ method: 'listDocuments', result })),
      )
      const err = await captureError(
        store.listDocuments({
          wikiId: 'w1',
          body: { state: null, tag: null, page: 1, pageSize: 20 },
        }),
      )
      expect(err).toBeInstanceOf(EngineError)
      expect(err).toMatchObject({ code: 'engine_error', httpStatus: 502 })
      expect((err as Error).message).toMatch(/invalid pagination/i)
    }
  })

  it('§5.9-15g fail: deleteDocument non-null result → EngineError (UnexpectedVoid)', async () => {
    const store = crudStoreWithResponse(
      jsonResponse(
        crudResponseEnvelope({ method: 'deleteDocument', result: DOC_WIRE }),
      ),
    )
    const err = await captureError(
      store.deleteDocument({ caller: 'user:alice', documentId: 'd1' }),
    )
    expect(err).toBeInstanceOf(EngineError)
    expect(err).toMatchObject({ code: 'engine_error', httpStatus: 502 })
    expect((err as Error).message).toMatch(/unexpected void/i)
  })

  it('§5.9-16 fail: method/result mismatch → EngineError (502)', async () => {
    const store = crudStoreWithResponse(
      jsonResponse(crudResponseEnvelope({ method: 'getWiki', result: DOC_WIRE })),
    )
    const err = await captureError(store.getWiki({ wikiId: 'w1' }))
    expect(err).toBeInstanceOf(EngineError)
    expect(err).toMatchObject({ code: 'engine_error', httpStatus: 502 })
  })

  it('§5.9-17 fail: non-2xx response with a non-envelope body → EngineError (502)', async () => {
    const store = crudStoreWithResponse(
      new Response(JSON.stringify({ error: 'bad request' }), {
        status: 400,
        headers: { 'content-type': 'application/json' },
      }),
    )
    const err = await captureError(store.getDocument({ documentId: 'd1' }))
    expect(err).toBeInstanceOf(EngineError)
    expect(err).toMatchObject({ code: 'engine_error', httpStatus: 502 })
  })

  it('§5.9-18 fail: P4 retry exhausted → EngineUnavailable (503)', async () => {
    let crudCalls = 0
    const store = createEngineCrudRagStore({
      baseUrl: 'http://127.0.0.1:8080',
      retry: { maxRetries: 2, backoffMs: 1 },
      fetch: fakeFetch((method, url) => {
        if (method === 'GET' && url.endsWith(ENGINE_ENDPOINTS.engineStatus)) {
          return jsonResponse(READY_REPORT)
        }
        crudCalls++
        throw refusedError()
      }),
    })
    const err = await captureError(
      store.createDocument({
        caller: 'user:alice',
        wikiId: 'w1',
        body: { title: 't', tags: null, author: null },
      }),
    )
    expect(err).toBeInstanceOf(EngineUnavailable)
    expect(err).toMatchObject({
      code: 'engine_unavailable',
      httpStatus: 503,
      cause: 'connection-refused',
    })
    // 1 initial attempt + maxRetries retries.
    expect(crudCalls).toBe(3)
  })

  it('P4 retry happy: createDocument retries on EngineUnavailable and succeeds', async () => {
    let crudCalls = 0
    const store = createEngineCrudRagStore({
      baseUrl: 'http://127.0.0.1:8080',
      retry: { maxRetries: 1, backoffMs: 1 },
      fetch: fakeFetch((method, url) => {
        if (method === 'GET' && url.endsWith(ENGINE_ENDPOINTS.engineStatus)) {
          return jsonResponse(READY_REPORT)
        }
        crudCalls++
        if (crudCalls === 1) throw refusedError()
        return jsonResponse(
          crudResponseEnvelope({ method: 'createDocument', result: DOC_WIRE }),
        )
      }),
    })
    const doc = await store.createDocument({
      caller: 'user:alice',
      wikiId: 'w1',
      body: { title: 'Getting Started', tags: ['guide'], author: 'alice' },
    })
    expect(doc).toEqual(DOC_TYPED)
    expect(crudCalls).toBe(2)
  })

  it('P4 retry: a non-EngineUnavailable error is NOT retried', async () => {
    let crudCalls = 0
    const store = createEngineCrudRagStore({
      baseUrl: 'http://127.0.0.1:8080',
      retry: { maxRetries: 2, backoffMs: 1 },
      fetch: fakeFetch((method, url) => {
        if (method === 'GET' && url.endsWith(ENGINE_ENDPOINTS.engineStatus)) {
          return jsonResponse(READY_REPORT)
        }
        crudCalls++
        return jsonResponse(
          crudResponseEnvelope({
            method: 'createDocument',
            error: { code: 'conflict', message: 'x' },
          }),
        )
      }),
    })
    const err = await captureError(
      store.createDocument({
        caller: 'user:alice',
        wikiId: 'w1',
        body: { title: 't', tags: null, author: null },
      }),
    )
    expect(err).toBeInstanceOf(ConflictError)
    expect(crudCalls).toBe(1)
  })
})
