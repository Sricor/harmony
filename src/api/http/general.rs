use std::sync::Arc;

use delay::delay::Delay;

use crate::database::Database;

use super::claim::Secret;

// ===== App State =====
pub type AppState = Arc<State>;

pub struct State {
    database: Database,

    // Claim Secret
    secret: Secret,

    // Delay
    delay: Delay,
}

impl State {
    pub fn new(database: Database, secret: Secret) -> Self {
        Self {
            database,
            secret,
            delay: Delay::new(),
        }
    }

    pub fn database(&self) -> &Database {
        &self.database
    }

    pub fn secret(&self) -> &Secret {
        &self.secret
    }

    pub fn delay(&self) -> &Delay {
        &self.delay
    }

    pub fn timestamp_millis() -> i64 {
        use chrono::Utc;
        Utc::now().timestamp_millis()
    }
}
