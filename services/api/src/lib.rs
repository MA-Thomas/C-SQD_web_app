//! Reusable API service boundaries.
//!
//! The binary remains the composition root; this library target exposes
//! repository code to integration tests without starting the HTTP server.

pub mod repositories;
