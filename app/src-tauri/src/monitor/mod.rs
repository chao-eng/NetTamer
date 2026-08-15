//! Per-process rate aggregation and EWMA smoothing.

pub mod aggregator;
pub mod ewma;

pub use aggregator::Aggregator;
pub use ewma::ewma;
