//! Persistence-aware throttle manager. Mirrors policy changes into both the
//! [`ThrottleTable`] (runtime) and the SQLite store (durable).

use std::sync::Arc;

use crate::models::{Error, Policy};
use crate::store::{Db, ThrottleStore};
use crate::throttle::ThrottleTable;

pub struct Manager {
    db: Db,
    table: Arc<ThrottleTable>,
}

impl Manager {
    pub fn new(db: Db, table: Arc<ThrottleTable>) -> Self {
        Manager { db, table }
    }

    /// Upsert a policy: persist it and update the live bucket.
    pub fn apply(&self, p: Policy) -> Result<(), Error> {
        self.db.save_policy(&p)?;
        self.table.apply_policy(p);
        Ok(())
    }

    /// Remove a policy from the store and the live table.
    pub fn remove(&self, id: &str) -> Result<(), Error> {
        self.db.delete_policy(id)?;
        self.table.remove_policy(id);
        Ok(())
    }

    /// List persisted policies.
    pub fn list(&self) -> Result<Vec<Policy>, Error> {
        self.db.list_policies()
    }

    /// Load all active persisted policies from SQLite into the live table.
    pub fn load(&self) -> Result<(), Error> {
        let list = self.db.list_policies()?;
        for p in list {
            if p.active {
                self.table.apply_policy(p);
            }
        }
        Ok(())
    }
}
