use self::time::timestamp_millis;

use super::*;
use crate::database::{Recorder, RecorderResult};

pub type Item = PromiseLogging;

#[rustfmt::skip]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromiseLogging {
    pub promise:   PromiseIdentifier,
    pub owner:     PersonIdentifier,
    pub level:     PromiseLoggingLevel,
    pub message:   String,
    pub timestamp: i64,
}

impl Item {
    #[rustfmt::skip]
    fn with_sqlite_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        let result = Self {
            promise:   row.get(0)?,
            owner:     row.get(1)?,
            level:     row.get(2)?,
            message:   row.get(3)?,
            timestamp: row.get(4)?,
        };

        Ok(result)
    }

    pub fn with_info(promise: PromiseIdentifier, owner: PersonIdentifier, message: String) -> Self {
        Self {
            promise,
            owner,
            level: PromiseLoggingLevel::Info,
            message,
            timestamp: timestamp_millis(),
        }
    }

    pub fn with_error(
        promise: PromiseIdentifier,
        owner: PersonIdentifier,
        message: String,
    ) -> Self {
        Self {
            promise,
            owner,
            level: PromiseLoggingLevel::Error,
            message,
            timestamp: timestamp_millis(),
        }
    }
}

#[rustfmt::skip]
pub trait Interface {
    fn insert(&self, item: &Item)
        -> impl Future<Output = RecorderResult<()>> + Send;

    fn insert_by_promise_owner_level_message(&self, promise: &PromiseIdentifier, owner: &PersonIdentifier, level:&PromiseLoggingLevel, message: &String)
        -> impl Future<Output = RecorderResult<()>> + Send;
    
    fn select_all(&self)
        -> impl Future<Output = RecorderResult<Vec<Item>>> + Send;

    fn select_all_by_promise_and_owner(&self, promise: &PromiseIdentifier, owner: &PersonIdentifier) 
        -> impl Future<Output = RecorderResult<Vec<Item>>> + Send;
}

impl Interface for Recorder<Item> {
    async fn insert(&self, item: &Item) -> RecorderResult<()> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "INSERT INTO PromiseLogging (promise, owner, level, message, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)";

                let conn = c.lock()?;
                let mut stmt = conn.prepare(statement)?;
                stmt.execute((
                    &item.promise,
                    &item.owner,
                    &item.level,
                    &item.message,
                    &item.timestamp,
                ))?;

                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn insert_by_promise_owner_level_message(
        &self,
        promise: &PromiseIdentifier,
        owner: &PersonIdentifier,
        level: &PromiseLoggingLevel,
        message: &String,
    ) -> RecorderResult<()> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "INSERT INTO PromiseLogging (promise, owner, level, message, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)";

                let conn = c.lock()?;
                let mut stmt = conn.prepare(statement)?;
                stmt.execute((promise, owner, level, message, timestamp_millis()))?;

                Ok(())
            }
            _ => todo!(),
        }
    }

    async fn select_all(&self) -> RecorderResult<Vec<Item>> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "SELECT promise, owner, level, message, timestamp FROM PromiseLogging";

                let conn = c.lock()?;
                let mut statement = conn.prepare(statement)?;
                let items_iter = statement.query_map([], |row| Item::with_sqlite_row(row))?;

                let mut result = Vec::with_capacity(32);
                for item in items_iter {
                    result.push(item?)
                }

                Ok(result)
            }
            _ => Ok(vec![]),
        }
    }

    async fn select_all_by_promise_and_owner(
        &self,
        promise: &PromiseIdentifier,
        owner: &PersonIdentifier,
    ) -> RecorderResult<Vec<Item>> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "SELECT promise, owner, level, message, timestamp FROM PromiseLogging WHERE promise = :promise AND owner = :owner";

                let conn = c.lock()?;
                let mut statement = conn.prepare(statement)?;
                let items_iter = statement
                    .query_map(&[(":promise", promise), (":owner", owner)], |row| {
                        Item::with_sqlite_row(row)
                    })?;

                let mut result = Vec::with_capacity(32);
                for item in items_iter {
                    result.push(item?)
                }
                Ok(result)
            }
            _ => Ok(vec![]),
        }
    }
}

// ===== Rusqlite SQL converted =====
#[rustfmt::skip]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PromiseLoggingLevel {
    Fatal = 1,
    Error = 2,
    Warn  = 3,
    Info  = 4,
    Debug = 5,
    Trace = 6
}

impl FromSql for PromiseLoggingLevel {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let num = value.as_i64()?;
        let result = match num {
            1 => Self::Fatal,
            2 => Self::Error,
            3 => Self::Warn,
            4 => Self::Info,
            5 => Self::Debug,
            6 => Self::Trace,
            _ => return Err(FromSqlError::InvalidType),
        };
        Ok(result)
    }
}

impl ToSql for PromiseLoggingLevel {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(*self as u8))
    }
}
