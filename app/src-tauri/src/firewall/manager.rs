//! Persistence-aware firewall manager. Mirrors rule changes into both the
//! [`FirewallTable`] (runtime) and the SQLite store (durable).

use std::sync::Arc;

use crate::firewall::FirewallTable;
use crate::models::{Error, FirewallRule};
use crate::store::{Db, FirewallStore};

pub struct Manager {
    db: Db,
    table: Arc<FirewallTable>,
}

impl Manager {
    pub fn new(db: Db, table: Arc<FirewallTable>) -> Self {
        Manager { db, table }
    }

    /// Upsert a firewall rule: persist it and update the live table.
    pub fn apply(&self, rule: FirewallRule) -> Result<(), Error> {
        self.db.save_firewall_rule(&rule)?;
        self.table.apply_rule(rule);
        Ok(())
    }

    /// Remove a firewall rule from the store and the live table.
    pub fn remove(&self, id: &str) -> Result<(), Error> {
        self.db.delete_firewall_rule(id)?;
        self.table.remove_rule(id);
        Ok(())
    }

    /// List persisted firewall rules.
    pub fn list(&self) -> Result<Vec<FirewallRule>, Error> {
        self.db.list_firewall_rules()
    }

    /// Load all active persisted rules from SQLite into the live table.
    pub fn load(&self) -> Result<(), Error> {
        let list = self.db.list_firewall_rules()?;
        for r in list {
            if r.active {
                self.table.apply_rule(r);
            }
        }
        Ok(())
    }
}
