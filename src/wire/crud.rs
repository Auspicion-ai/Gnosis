//! §7.2 P1a — Document-CRUD wire contract (`src/wire/crud.rs`).
//!
//! The 11 §4.1 document-CRUD request/response wire shapes + codecs + validation
//! + endpoint constants. Extends the F2 wire layer (`docs/specs/engine-wire-contract.md`);
//! every CRUD request/response is wrapped in the F2 `Envelope`
//! (`CURRENT_SCHEMA_VERSION=1`, `idFormat:"opaque-string-v1"`). The serde-frozen
//! store bodies are embedded verbatim ("body via serde, don't re-case").
//! Contract: `docs/specs/p1a-document-crud-wire.md`.

use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::store::{
    CreateDocumentRequest, DocState, Document, DocumentId, DocumentList, ListDocumentsFilter,
    StoreError, UpdateDocumentRequest, Wiki, WikiId,
};
use crate::wire::decode::DecodeError;
use crate::wire::envelope::{Envelope, CURRENT_SCHEMA_VERSION, ID_FORMAT_OPAQUE_STRING_V1};
use crate::wire::error::from_wire;

/// The typed CRUD method discriminator (§4.2). Serialized as the camelCase
/// strings of §4.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CrudMethod {
    CreateDocument,
    GetDocument,
    UpdateDocument,
    DeleteDocument,
    PublishDocument,
    UnpublishDocument,
    ArchiveDocument,
    ListDocuments,
    CreateWiki,
    GetWiki,
    ListWikis,
}

impl CrudMethod {
    /// The camelCase wire `"method"` value (§4.2).
    pub fn method_str(&self) -> &'static str {
        use CrudMethod::*;
        match self {
            CreateDocument => "createDocument",
            GetDocument => "getDocument",
            UpdateDocument => "updateDocument",
            DeleteDocument => "deleteDocument",
            PublishDocument => "publishDocument",
            UnpublishDocument => "unpublishDocument",
            ArchiveDocument => "archiveDocument",
            ListDocuments => "listDocuments",
            CreateWiki => "createWiki",
            GetWiki => "getWiki",
            ListWikis => "listWikis",
        }
    }

    /// Reverse lookup of the camelCase wire `"method"` value → the variant.
    fn from_str(s: &str) -> Option<CrudMethod> {
        use CrudMethod::*;
        Some(match s {
            "createDocument" => CreateDocument,
            "getDocument" => GetDocument,
            "updateDocument" => UpdateDocument,
            "deleteDocument" => DeleteDocument,
            "publishDocument" => PublishDocument,
            "unpublishDocument" => UnpublishDocument,
            "archiveDocument" => ArchiveDocument,
            "listDocuments" => ListDocuments,
            "createWiki" => CreateWiki,
            "getWiki" => GetWiki,
            "listWikis" => ListWikis,
            _ => return None,
        })
    }
}

/// The typed per-method request-args surface (§4.3). Each variant serializes to
/// the flat `args` object of §4.2 (camelCase top-level fields; the serde-frozen
/// `body` embedded verbatim).
#[derive(Debug, Clone, PartialEq)]
pub enum CrudRequestArgs {
    CreateDocument {
        caller: String,
        wiki_id: WikiId,
        body: CreateDocumentRequest,
    },
    GetDocument {
        document_id: DocumentId,
    },
    UpdateDocument {
        caller: String,
        document_id: DocumentId,
        body: UpdateDocumentRequest,
    },
    DeleteDocument {
        caller: String,
        document_id: DocumentId,
    },
    PublishDocument {
        caller: String,
        document_id: DocumentId,
    },
    UnpublishDocument {
        caller: String,
        document_id: DocumentId,
    },
    ArchiveDocument {
        caller: String,
        document_id: DocumentId,
    },
    ListDocuments {
        wiki_id: WikiId,
        body: ListDocumentsFilter,
    },
    CreateWiki {
        caller: String,
        name: String,
    },
    GetWiki {
        wiki_id: WikiId,
    },
    ListWikis,
}

impl Serialize for CrudRequestArgs {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        match self {
            CrudRequestArgs::CreateDocument {
                caller,
                wiki_id,
                body,
            } => {
                map.serialize_entry("caller", caller)?;
                map.serialize_entry("wikiId", wiki_id)?;
                map.serialize_entry("body", body)?;
            }
            CrudRequestArgs::GetDocument { document_id } => {
                map.serialize_entry("documentId", document_id)?;
            }
            CrudRequestArgs::UpdateDocument {
                caller,
                document_id,
                body,
            } => {
                map.serialize_entry("caller", caller)?;
                map.serialize_entry("documentId", document_id)?;
                map.serialize_entry("body", body)?;
            }
            CrudRequestArgs::DeleteDocument { caller, document_id } => {
                map.serialize_entry("caller", caller)?;
                map.serialize_entry("documentId", document_id)?;
            }
            CrudRequestArgs::PublishDocument { caller, document_id } => {
                map.serialize_entry("caller", caller)?;
                map.serialize_entry("documentId", document_id)?;
            }
            CrudRequestArgs::UnpublishDocument { caller, document_id } => {
                map.serialize_entry("caller", caller)?;
                map.serialize_entry("documentId", document_id)?;
            }
            CrudRequestArgs::ArchiveDocument { caller, document_id } => {
                map.serialize_entry("caller", caller)?;
                map.serialize_entry("documentId", document_id)?;
            }
            CrudRequestArgs::ListDocuments { wiki_id, body } => {
                map.serialize_entry("wikiId", wiki_id)?;
                map.serialize_entry("body", body)?;
            }
            CrudRequestArgs::CreateWiki { caller, name } => {
                map.serialize_entry("caller", caller)?;
                map.serialize_entry("name", name)?;
            }
            CrudRequestArgs::GetWiki { wiki_id } => {
                map.serialize_entry("wikiId", wiki_id)?;
            }
            CrudRequestArgs::ListWikis => {}
        }
        map.end()
    }
}

/// The typed CRUD result surface (§4.3).
#[derive(Debug, Clone, PartialEq)]
pub enum CrudResult {
    Document(Document),
    DeleteDocument,
    DocumentList(DocumentList),
    Wiki(Wiki),
    WikiList(Vec<Wiki>),
}

/// A CRUD response decode/validation failure (§10).
#[derive(Debug, Clone, PartialEq)]
pub enum CrudResponseError {
    /// The response carried a `StoreError` (the non-chunk codec body).
    Store(StoreError),
    /// The response body was malformed (→ `EngineError`/502).
    Decode(DecodeError),
    /// A well-formed body failed a CRUD-specific invariant (→ `EngineError`/502).
    Validation(CrudValidationFailure),
}

/// A CRUD-specific validation failure (§10).
#[derive(Debug, Clone, PartialEq)]
pub enum CrudValidationFailure {
    /// createDocument must yield revision == 0.
    UnexpectedRevision { expected: u64, actual: u64 },
    /// publish/unpublish/archive must yield the documented state.
    UnexpectedState { expected: DocState, actual: DocState },
    /// listDocuments must yield page >= 1, 1 <= page_size <= 100.
    InvalidPagination { page: u64, page_size: u64 },
    /// deleteDocument must yield void (null result).
    UnexpectedVoid,
}

// ---------------------------------------------------------------------------
// Per-variant args deserialization helpers (the flat `args` object of §4.2).
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct CreateDocumentArgs {
    caller: String,
    #[serde(rename = "wikiId")]
    wiki_id: WikiId,
    body: CreateDocumentRequest,
}

#[derive(Deserialize)]
struct GetDocumentArgs {
    #[serde(rename = "documentId")]
    document_id: DocumentId,
}

#[derive(Deserialize)]
struct UpdateDocumentArgs {
    caller: String,
    #[serde(rename = "documentId")]
    document_id: DocumentId,
    body: UpdateDocumentRequest,
}

#[derive(Deserialize)]
struct IdArgs {
    caller: String,
    #[serde(rename = "documentId")]
    document_id: DocumentId,
}

#[derive(Deserialize)]
struct ListDocumentsArgs {
    #[serde(rename = "wikiId")]
    wiki_id: WikiId,
    body: ListDocumentsFilter,
}

#[derive(Deserialize)]
struct CreateWikiArgs {
    caller: String,
    name: String,
}

#[derive(Deserialize)]
struct WikiIdArgs {
    #[serde(rename = "wikiId")]
    wiki_id: WikiId,
}

// ---------------------------------------------------------------------------
// Request codecs
// ---------------------------------------------------------------------------

/// Encode a CRUD request into a request envelope (§4.2).
pub fn encode_crud_request(method: CrudMethod, args: CrudRequestArgs) -> Envelope {
    let payload = serde_json::json!({
        "method": method.method_str(),
        "args": args,
    });
    Envelope::with_payload(payload)
}

/// Decode a CRUD request envelope → `(method, args)` (§4.2 / §6.2).
pub fn decode_crud_request(env: &Envelope) -> Result<(CrudMethod, CrudRequestArgs), DecodeError> {
    if env.schema_version != CURRENT_SCHEMA_VERSION {
        return Err(DecodeError::UnsupportedSchemaVersion(env.schema_version));
    }
    if env.id_format != ID_FORMAT_OPAQUE_STRING_V1 {
        return Err(DecodeError::UnknownIdFormat(env.id_format.clone()));
    }
    let payload = env.payload.as_object().ok_or_else(|| {
        DecodeError::InvalidEnvelope("CRUD request payload must be a JSON object".to_string())
    })?;
    let method_str = payload
        .get("method")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            DecodeError::InvalidEnvelope("CRUD request missing string \"method\"".to_string())
        })?;
    let method = CrudMethod::from_str(method_str)
        .ok_or_else(|| DecodeError::UnknownMethod(method_str.to_string()))?;
    let args = payload
        .get("args")
        .ok_or_else(|| DecodeError::InvalidEnvelope("CRUD request missing \"args\"".to_string()))?;
    let args = decode_args(&method, args)?;
    Ok((method, args))
}

fn decode_args(method: &CrudMethod, args: &Value) -> Result<CrudRequestArgs, DecodeError> {
    use CrudMethod::*;
    let invalid = |e: serde_json::Error| DecodeError::InvalidEnvelope(e.to_string());
    match method {
        CreateDocument => {
            let a: CreateDocumentArgs = serde_json::from_value(args.clone()).map_err(invalid)?;
            Ok(CrudRequestArgs::CreateDocument {
                caller: a.caller,
                wiki_id: a.wiki_id,
                body: a.body,
            })
        }
        GetDocument => {
            let a: GetDocumentArgs = serde_json::from_value(args.clone()).map_err(invalid)?;
            Ok(CrudRequestArgs::GetDocument {
                document_id: a.document_id,
            })
        }
        UpdateDocument => {
            let a: UpdateDocumentArgs = serde_json::from_value(args.clone()).map_err(invalid)?;
            Ok(CrudRequestArgs::UpdateDocument {
                caller: a.caller,
                document_id: a.document_id,
                body: a.body,
            })
        }
        DeleteDocument => {
            let a: IdArgs = serde_json::from_value(args.clone()).map_err(invalid)?;
            Ok(CrudRequestArgs::DeleteDocument {
                caller: a.caller,
                document_id: a.document_id,
            })
        }
        PublishDocument => {
            let a: IdArgs = serde_json::from_value(args.clone()).map_err(invalid)?;
            Ok(CrudRequestArgs::PublishDocument {
                caller: a.caller,
                document_id: a.document_id,
            })
        }
        UnpublishDocument => {
            let a: IdArgs = serde_json::from_value(args.clone()).map_err(invalid)?;
            Ok(CrudRequestArgs::UnpublishDocument {
                caller: a.caller,
                document_id: a.document_id,
            })
        }
        ArchiveDocument => {
            let a: IdArgs = serde_json::from_value(args.clone()).map_err(invalid)?;
            Ok(CrudRequestArgs::ArchiveDocument {
                caller: a.caller,
                document_id: a.document_id,
            })
        }
        ListDocuments => {
            let a: ListDocumentsArgs = serde_json::from_value(args.clone()).map_err(invalid)?;
            Ok(CrudRequestArgs::ListDocuments {
                wiki_id: a.wiki_id,
                body: a.body,
            })
        }
        CreateWiki => {
            let a: CreateWikiArgs = serde_json::from_value(args.clone()).map_err(invalid)?;
            Ok(CrudRequestArgs::CreateWiki {
                caller: a.caller,
                name: a.name,
            })
        }
        GetWiki => {
            let a: WikiIdArgs = serde_json::from_value(args.clone()).map_err(invalid)?;
            Ok(CrudRequestArgs::GetWiki { wiki_id: a.wiki_id })
        }
        ListWikis => {
            // The args object must be an empty object `{}`.
            if args.as_object().map(|o| o.is_empty()).unwrap_or(false) {
                Ok(CrudRequestArgs::ListWikis)
            } else {
                Err(DecodeError::InvalidEnvelope(
                    "listWikis args must be an empty object".to_string(),
                ))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Response codecs
// ---------------------------------------------------------------------------

/// Encode a well-formed CRUD result into a response envelope (§4.3).
pub fn encode_crud_response(method: CrudMethod, result: CrudResult) -> Envelope {
    let result_value = match &result {
        CrudResult::DeleteDocument => Value::Null,
        CrudResult::Document(d) => serde_json::to_value(d).expect("Document serializes"),
        CrudResult::DocumentList(l) => serde_json::to_value(l).expect("DocumentList serializes"),
        CrudResult::Wiki(w) => serde_json::to_value(w).expect("Wiki serializes"),
        CrudResult::WikiList(w) => serde_json::to_value(w).expect("WikiList serializes"),
    };
    let payload = serde_json::json!({
        "method": method.method_str(),
        "result": result_value,
    });
    Envelope::with_payload(payload)
}

/// Encode a `StoreError` into a CRUD error envelope (§4.3 / §4.4).
pub fn encode_crud_error(method: CrudMethod, err: &StoreError) -> Envelope {
    let payload = serde_json::json!({
        "method": method.method_str(),
        "error": {
            "code": err.wire_code(),
            "message": format!("{}", err),
        },
    });
    Envelope::with_payload(payload)
}

/// Decode a CRUD response envelope → the result, or a `CrudResponseError`
/// (§4.3 / §10).
pub fn decode_crud_response(env: &Envelope) -> Result<CrudResult, CrudResponseError> {
    if env.schema_version != CURRENT_SCHEMA_VERSION {
        return Err(CrudResponseError::Decode(DecodeError::UnsupportedSchemaVersion(
            env.schema_version,
        )));
    }
    if env.id_format != ID_FORMAT_OPAQUE_STRING_V1 {
        return Err(CrudResponseError::Decode(DecodeError::UnknownIdFormat(
            env.id_format.clone(),
        )));
    }
    let payload = env.payload.as_object().ok_or_else(|| {
        CrudResponseError::Decode(DecodeError::InvalidEnvelope(
            "CRUD response payload must be a JSON object".to_string(),
        ))
    })?;
    let method_str = payload
        .get("method")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            CrudResponseError::Decode(DecodeError::InvalidEnvelope(
                "CRUD response missing string \"method\"".to_string(),
            ))
        })?;
    let method = CrudMethod::from_str(method_str).ok_or_else(|| {
        CrudResponseError::Decode(DecodeError::UnknownMethod(method_str.to_string()))
    })?;

    // An `"error"` field carries the non-chunk codec body.
    if let Some(err) = payload.get("error") {
        let code = err
            .get("code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                CrudResponseError::Decode(DecodeError::InvalidEnvelope(
                    "CRUD error missing string \"code\"".to_string(),
                ))
            })?;
        let message = err.get("message").and_then(|v| v.as_str());
        let store_err = from_wire(code, message).ok_or_else(|| {
            CrudResponseError::Decode(DecodeError::UnknownCode(code.to_string()))
        })?;
        return Err(CrudResponseError::Store(store_err));
    }

    let result = payload.get("result").ok_or_else(|| {
        CrudResponseError::Decode(DecodeError::InvalidEnvelope(
            "CRUD response missing \"result\"".to_string(),
        ))
    })?;
    let crud_result = decode_result(&method, result)?;
    validate_crud_result(&method, &crud_result).map_err(CrudResponseError::Validation)?;
    Ok(crud_result)
}

fn decode_result(method: &CrudMethod, result: &Value) -> Result<CrudResult, CrudResponseError> {
    use CrudMethod::*;
    let invalid = |e: serde_json::Error| {
        CrudResponseError::Decode(DecodeError::InvalidJson(e.to_string()))
    };
    match method {
        CreateDocument
        | GetDocument
        | UpdateDocument
        | PublishDocument
        | UnpublishDocument
        | ArchiveDocument => {
            let d: Document = serde_json::from_value(result.clone()).map_err(invalid)?;
            Ok(CrudResult::Document(d))
        }
        DeleteDocument => {
            if result.is_null() {
                Ok(CrudResult::DeleteDocument)
            } else {
                Err(CrudResponseError::Decode(DecodeError::InvalidEnvelope(
                    "deleteDocument result must be null".to_string(),
                )))
            }
        }
        ListDocuments => {
            let l: DocumentList = serde_json::from_value(result.clone()).map_err(invalid)?;
            Ok(CrudResult::DocumentList(l))
        }
        CreateWiki | GetWiki => {
            let w: Wiki = serde_json::from_value(result.clone()).map_err(invalid)?;
            Ok(CrudResult::Wiki(w))
        }
        ListWikis => {
            let w: Vec<Wiki> = serde_json::from_value(result.clone()).map_err(invalid)?;
            Ok(CrudResult::WikiList(w))
        }
    }
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

/// Validate a CRUD result against the §10 per-method invariants.
pub fn validate_crud_result(
    method: &CrudMethod,
    result: &CrudResult,
) -> Result<(), CrudValidationFailure> {
    use CrudMethod::*;
    match method {
        CreateDocument => {
            let d = match result {
                CrudResult::Document(d) => d,
                _ => return Err(CrudValidationFailure::UnexpectedVoid),
            };
            if d.revision != 0 {
                return Err(CrudValidationFailure::UnexpectedRevision {
                    expected: 0,
                    actual: d.revision,
                });
            }
            if d.state != DocState::Draft {
                return Err(CrudValidationFailure::UnexpectedState {
                    expected: DocState::Draft,
                    actual: d.state,
                });
            }
            Ok(())
        }
        GetDocument | UpdateDocument => Ok(()),
        DeleteDocument => {
            if !matches!(result, CrudResult::DeleteDocument) {
                return Err(CrudValidationFailure::UnexpectedVoid);
            }
            Ok(())
        }
        PublishDocument => {
            let d = match result {
                CrudResult::Document(d) => d,
                _ => return Err(CrudValidationFailure::UnexpectedVoid),
            };
            if d.state != DocState::Published {
                return Err(CrudValidationFailure::UnexpectedState {
                    expected: DocState::Published,
                    actual: d.state,
                });
            }
            Ok(())
        }
        UnpublishDocument => {
            let d = match result {
                CrudResult::Document(d) => d,
                _ => return Err(CrudValidationFailure::UnexpectedVoid),
            };
            if d.state != DocState::Draft {
                return Err(CrudValidationFailure::UnexpectedState {
                    expected: DocState::Draft,
                    actual: d.state,
                });
            }
            Ok(())
        }
        ArchiveDocument => {
            let d = match result {
                CrudResult::Document(d) => d,
                _ => return Err(CrudValidationFailure::UnexpectedVoid),
            };
            if d.state != DocState::Archived {
                return Err(CrudValidationFailure::UnexpectedState {
                    expected: DocState::Archived,
                    actual: d.state,
                });
            }
            Ok(())
        }
        ListDocuments => {
            let l = match result {
                CrudResult::DocumentList(l) => l,
                _ => return Err(CrudValidationFailure::UnexpectedVoid),
            };
            if l.page < 1 || l.page_size < 1 || l.page_size > 100 {
                return Err(CrudValidationFailure::InvalidPagination {
                    page: l.page,
                    page_size: l.page_size,
                });
            }
            Ok(())
        }
        CreateWiki | GetWiki | ListWikis => Ok(()),
    }
}

// ---------------------------------------------------------------------------
// Endpoint-path ownership (H4, §7)
// ---------------------------------------------------------------------------

/// The 11-row bijective endpoint-path table (§7).
pub const ENGINE_ENDPOINTS: &[(&str, CrudMethod)] = &[
    ("POST /documents", CrudMethod::CreateDocument),
    ("GET /documents/:id", CrudMethod::GetDocument),
    ("POST /documents/:id/update", CrudMethod::UpdateDocument),
    ("DELETE /documents/:id", CrudMethod::DeleteDocument),
    ("POST /documents/:id/publish", CrudMethod::PublishDocument),
    ("POST /documents/:id/unpublish", CrudMethod::UnpublishDocument),
    ("POST /documents/:id/archive", CrudMethod::ArchiveDocument),
    ("GET /documents", CrudMethod::ListDocuments),
    ("POST /wikis", CrudMethod::CreateWiki),
    ("GET /wikis/:id", CrudMethod::GetWiki),
    ("GET /wikis", CrudMethod::ListWikis),
];

pub const ENDPOINT_CREATE_DOCUMENT: &str = "POST /documents";
pub const ENDPOINT_GET_DOCUMENT: &str = "GET /documents/:id";
pub const ENDPOINT_UPDATE_DOCUMENT: &str = "POST /documents/:id/update";
pub const ENDPOINT_DELETE_DOCUMENT: &str = "DELETE /documents/:id";
pub const ENDPOINT_PUBLISH_DOCUMENT: &str = "POST /documents/:id/publish";
pub const ENDPOINT_UNPUBLISH_DOCUMENT: &str = "POST /documents/:id/unpublish";
pub const ENDPOINT_ARCHIVE_DOCUMENT: &str = "POST /documents/:id/archive";
pub const ENDPOINT_LIST_DOCUMENTS: &str = "GET /documents";
pub const ENDPOINT_CREATE_WIKI: &str = "POST /wikis";
pub const ENDPOINT_GET_WIKI: &str = "GET /wikis/:id";
pub const ENDPOINT_LIST_WIKIS: &str = "GET /wikis";
