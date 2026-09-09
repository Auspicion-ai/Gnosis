//! §7.2 F2 — the versioned envelope `{schemaVersion, idFormat, payload}`.
//! Contract: `docs/specs/engine-wire-contract.md` §4.1 / §10.
//!
//! **RED-stage stub skeleton** — the struct/consts are real (serde derives make
//! them usable), but the helper bodies (`to_json`/`from_json`/`with_payload`)
//! are placeholders the Implementer fills.

use serde::{Deserialize, Serialize};

use crate::wire::codecs::WireCodecError;
use crate::wire::decode::DecodeError;

/// The wire envelope. snake_case **Rust** fields; camelCase on the wire via
/// `rename_all` (`schemaVersion` / `idFormat`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Envelope {
    pub schema_version: u32,
    pub id_format: String,
    pub payload: serde_json::Value,
}

/// The current schema version (all §12 golden vectors use `schemaVersion:1`).
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// The only implemented `id_format` ("opaque strings" — RFC-4122 deferred).
pub const ID_FORMAT_OPAQUE_STRING_V1: &str = "opaque-string-v1";

/// Returns `CURRENT_SCHEMA_VERSION`.
pub fn current_schema_version() -> u32 {
    CURRENT_SCHEMA_VERSION
}

impl Envelope {
    /// Serialize to compact JSON (serde_json::to_string); never fails for our
    /// payload types.
    pub fn to_json(&self) -> Result<String, WireCodecError> {
        let _ = self;
        todo!("RED-stage stub: Envelope::to_json")
    }

    /// Parse + shape-check a JSON string; `InvalidJson` on a parse error,
    /// `InvalidEnvelope` on a missing/typed-weak top-level field. Does **not**
    /// itself validate `schema_version`/`id_format` against the constants.
    pub fn from_json(s: &str) -> Result<Envelope, DecodeError> {
        let _ = s;
        todo!("RED-stage stub: Envelope::from_json")
    }

    /// Canonical envelope: `schema_version = 1`, `id_format = "opaque-string-v1"`.
    pub fn with_payload(payload: serde_json::Value) -> Envelope {
        let _ = payload;
        todo!("RED-stage stub: Envelope::with_payload")
    }
}
