// src/lib.rs
//! Firework: un framework unopinionated basado en std y tokio.
//!
//! Este framework pretende mantener un tiempo de compilación rápido y ser fácilmente extensible.
//! Aquí se reexportan los módulos públicos.

pub mod error;
pub mod route;
pub mod router;
pub mod request;
pub mod response;
pub mod middleware;
pub mod server;
pub mod macros;
