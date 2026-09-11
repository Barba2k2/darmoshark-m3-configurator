//! Protocol and transport for the Darmoshark M3 (Attack Shark M3) mouse.
//!
//! The `dms` wire protocol was reverse engineered from the vendor's WebHID
//! bundle and validated against real hardware; `PROTOCOL.md` is the spec.
//!
//! - `protocol`: every report id, opcode and limit. Constants live only here.
//! - `packets`: pure builders, validating ranges before anything reaches the
//!   hardware. They never touch a device.
//! - `replies`: pure decoders for what the mouse and receiver answer.
//! - `transport`: the only module that talks to `hidapi`.
//! - `configurator`: builders + transport composed into high-level operations.
//! - `profile`: the vendor's capability definition for the M3, offline.

// Constants follow the project naming rule: camelCase, never SCREAMING_CASE.
#![allow(non_upper_case_globals)]

pub mod configurator;
pub mod error;
pub mod packets;
pub mod profile;
pub mod protocol;
pub mod replies;
pub mod transport;
