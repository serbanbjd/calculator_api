//! Calculator API: a stateless HTTP JSON calculator (see docs/prd/PRD.md).
//!
//! `calc` is the pure maths domain; `api` is the HTTP layer on top of it
//! (ADR-0004).

mod api;

pub use api::app;
