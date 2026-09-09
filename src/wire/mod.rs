//! §7.2 F2 — Engine wire contract (`src/wire/`).
//!
//! Mechanism-agnostic codecs + validation + single-event SSE + health +
//! versioned envelope layer for the Astrographrer-shell → Gnosis-engine proxy
//! seam. Contract: `docs/specs/engine-wire-contract.md` (GREEN — 360 tests).

pub mod codecs;
pub mod decode;
pub mod envelope;
pub mod error;
pub mod sse;
pub mod status;

pub use self::decode::{DecodeError, ValidationFailure};
pub use self::envelope::Envelope;
pub use self::status::HealthReport;
