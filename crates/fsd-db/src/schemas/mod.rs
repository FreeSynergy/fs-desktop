//! Schema definitions for the other FSN program databases.
//!
//! These modules define the SQL schemas that each program will use when they
//! initialize their own database. The schemas are centralized here so they
//! can be referenced consistently across the codebase.

pub mod browser;
pub mod bus;
pub mod conductor;
pub mod core;
pub mod lenses;
pub mod store;
