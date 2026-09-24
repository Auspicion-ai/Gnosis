//! §7.2 F2 — Engine wire contract (`src/wire/`).
//!
//! Mechanism-agnostic codecs + validation + single-event SSE + health +
//! versioned envelope layer for the Astrographrer-shell → Gnosis-engine proxy
//! seam. Contract: `docs/specs/engine-wire-contract.md` (GREEN — 360 tests).

pub mod codecs;
pub mod crud;
pub mod decode;
pub mod envelope;
pub mod error;
// §7.2 P2 U2 — the shared `ragQuery` decode seam (RED-stage stub).
pub mod query;
pub mod sse;
pub mod status;

pub use self::crud::{CrudMethod, CrudResponseError, CrudResult, CrudValidationFailure};
pub use self::decode::{DecodeError, ValidationFailure};
pub use self::envelope::Envelope;
pub use self::query::{
    decode_query_request, encode_result_checked, request_decode_code, request_decode_message,
    resolve_compression_mode, resolve_expand_mode, resolve_query_mode, QueryDecodeError, QueryPath,
    SseParams,
};
pub use self::status::HealthReport;
