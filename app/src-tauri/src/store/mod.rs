//! SQLite persistence layer (rusqlite + r2d2 connection pool).

pub mod migrations;
pub mod store;

pub use store::{AlertStore, ConfigStore, Db, ThrottleStore};
