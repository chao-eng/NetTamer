//! Alert rule engine: matching, threshold evaluation, cooldowns, persistence.

pub mod engine;
pub mod matcher;

pub use engine::Engine;
