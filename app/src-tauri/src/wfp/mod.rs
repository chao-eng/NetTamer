//! Windows Filtering Platform (WFP) integration:
//! Native kernel-level process network blocking via ALE and Transport layers.

pub mod engine;

pub use engine::WfpEngine;
