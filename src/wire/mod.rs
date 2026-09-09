//! §7.2 F2 — Engine wire contract (`src/wire/`).
//!
//! Mechanism-agnostic codecs + validation + single-event SSE + health +
//! versioned envelope layer for the Astrographrer-shell → Gnosis-engine proxy
//! seam. Contract: `docs/specs/engine-wire-contract.md`.
//!
//! **RED-stage stub skeleton.** This module's function bodies are `todo!()` /
//! `Err` placeholders inserted by the TestWriter so the crate compiles and the
//! conformance + property tests fail red. The F2 Implementer replaces the
//! bodies with the real logic (matching these exact signatures) to go green.

pub mod codecs;
pub mod decode;
pub mod envelope;
pub mod error;
pub mod sse;
pub mod status;

pub use self::decode::{DecodeError, ValidationFailure};
pub use self::envelope::Envelope;
pub use self::status::HealthReport;
