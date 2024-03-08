pub mod binance_spot_limit;
pub mod logging;

use super::*;
use crate::database::{Recorder, RecorderResult, Uniquer};

pub type Item = Promise;
pub type PromiseIdentifier = String;

#[rustfmt::skip]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Promise {
    pub identifier:     PromiseIdentifier,
    pub owner:          PersonIdentifier,
    pub category:       PromiseCategory,
    pub running:        PromiseRunning,
    pub total_runs:     u64,
    pub interval:       u64,
    pub timeout:        u64,
    pub max_concurrent: usize,
}

impl Item {
    #[rustfmt::skip]
    fn with_sqlite_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        let result = Self {
            identifier:     row.get(0)?,
            owner:          row.get(1)?,
            category:       row.get(2)?,
            running:        row.get(3)?,
            total_runs:     row.get(4)?,
            interval:       row.get(5)?,
            timeout:        row.get(6)?,
            max_concurrent: row.get(7)?,
        };

        Ok(result)
    }

    pub fn with_binance_spot_limit(owner: PersonIdentifier, interval: u64) -> Self {
        Self {
            identifier: unique::uuid(),
            owner,
            category: PromiseCategory::BinanceSpotLimit,
            running: PromiseRunning::Stopped,
            total_runs: 0,
            interval,
            timeout: 1800,
            max_concurrent: 1,
        }
    }
}

impl Uniquer for Item {
    type Sign = PromiseIdentifier;

    fn identifier(&self) -> &Self::Sign {
        &self.identifier
    }
}

#[rustfmt::skip]
pub trait Interface {
    fn insert(&self, item: &Item)
        -> impl Future<Output = RecorderResult<()>> + Send;

    fn select_all(&self)
        -> impl Future<Output = RecorderResult<Vec<Item>>> + Send;
    
    fn select_all_by_running(&self, running: &PromiseRunning)
        -> impl Future<Output = RecorderResult<Vec<Item>>> + Send;

    fn select_one_by_identifier(&self, identifier: &PromiseIdentifier) 
        -> impl Future<Output = RecorderResult<Option<Item>>> + Send;

    fn select_one_by_identifier_and_owner(&self, identifier: &PromiseIdentifier, owner: &PersonIdentifier) 
        -> impl Future<Output = RecorderResult<Option<Item>>> + Send;

    fn select_one_by_identifier_owner_category(&self, identifier: &PromiseIdentifier, owner: &PersonIdentifier, category: &PromiseCategory) 
        -> impl Future<Output = RecorderResult<Option<Item>>> + Send;
    
    fn select_all_by_owner_category(&self, owner: &PersonIdentifier, category: &PromiseCategory) 
        -> impl Future<Output = RecorderResult<Vec<Item>>> + Send;

    fn update_running_by_identifier_and_owner(&self, identifier: &PromiseIdentifier, owner: &PersonIdentifier, running: &PromiseRunning) 
        -> impl Future<Output = RecorderResult<()>> + Send;
}

impl Interface for Recorder<Item> {
    async fn insert(&self, item: &Item) -> RecorderResult<()> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "INSERT INTO Promise (identifier, owner, category, running, total_runs, interval, timeout, max_concurrent) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)";

                let conn = c.lock()?;
                let mut stmt = conn.prepare(statement)?;
                stmt.execute((
                    &item.identifier,
                    &item.owner,
                    &item.category,
                    &item.running,
                    &item.total_runs,
                    &item.interval,
                    &item.timeout,
                    &item.max_concurrent,
                ))?;

                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn select_all(&self) -> RecorderResult<Vec<Item>> {
        match self {
            Self::Sqlite(c) => {
                let statement = "SELECT identifier, owner, category, running, total_runs, interval, timeout, max_concurrent FROM Promise";

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

    async fn select_all_by_running(&self, running: &PromiseRunning) -> RecorderResult<Vec<Item>> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "SELECT identifier, owner, category, running, total_runs, interval, timeout, max_concurrent FROM Promise WHERE running = ?1";

                let conn = c.lock()?;
                let mut statement = conn.prepare(statement)?;
                let items_iter =
                    statement.query_map(params![running], |row| Item::with_sqlite_row(row))?;

                let mut result = Vec::with_capacity(32);
                for item in items_iter {
                    result.push(item?)
                }

                Ok(result)
            }
            _ => Ok(vec![]),
        }
    }

    async fn select_one_by_identifier(
        &self,
        identifier: &PromiseIdentifier,
    ) -> RecorderResult<Option<Item>> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "SELECT identifier, owner, category, running, total_runs, interval, timeout, max_concurrent FROM Promise WHERE identifier = :identifier";

                let conn = c.lock()?;
                let mut statement = conn.prepare(statement)?;
                let items_iter = statement.query_map(&[(":identifier", identifier)], |row| {
                    Item::with_sqlite_row(row)
                })?;

                let result = match items_iter.into_iter().nth(0) {
                    Some(item) => Some(item?),
                    None => None,
                };
                Ok(result)
            }
            _ => Ok(None),
        }
    }

    async fn select_one_by_identifier_and_owner(
        &self,
        identifier: &PromiseIdentifier,
        owner: &PersonIdentifier,
    ) -> RecorderResult<Option<Item>> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "SELECT identifier, owner, category, running, total_runs, interval, timeout, max_concurrent FROM Promise WHERE identifier = :identifier AND owner = :owner";

                let conn = c.lock()?;
                let mut statement = conn.prepare(statement)?;
                let items_iter = statement
                    .query_map(&[(":identifier", identifier), (":owner", owner)], |row| {
                        Item::with_sqlite_row(row)
                    })?;

                let result = match items_iter.into_iter().nth(0) {
                    Some(v) => Some(v?),
                    None => None,
                };

                Ok(result)
            }
            _ => Ok(None),
        }
    }

    async fn select_one_by_identifier_owner_category(
        &self,
        identifier: &PromiseIdentifier,
        owner: &PersonIdentifier,
        category: &PromiseCategory,
    ) -> RecorderResult<Option<Item>> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "SELECT identifier, owner, category, running, total_runs, interval, timeout, max_concurrent FROM Promise WHERE identifier = ?1 AND owner = ?2 AND category = ?3";

                let conn = c.lock()?;
                let mut statement = conn.prepare(statement)?;
                let items_iter = statement
                    .query_map(params![identifier, owner, category], |row| {
                        Item::with_sqlite_row(row)
                    })?;

                let result = match items_iter.into_iter().nth(0) {
                    Some(v) => Some(v?),
                    None => None,
                };

                Ok(result)
            }
            _ => Ok(None),
        }
    }

    async fn select_all_by_owner_category(
        &self,
        owner: &PersonIdentifier,
        category: &PromiseCategory,
    ) -> RecorderResult<Vec<Item>> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "SELECT identifier, owner, category, running, total_runs, interval, timeout, max_concurrent FROM Promise WHERE owner = ?1 AND category = ?2";

                let conn = c.lock()?;
                let mut statement = conn.prepare(statement)?;
                let items_iter = statement
                    .query_map(params![owner, category], |row| Item::with_sqlite_row(row))?;

                let mut result = Vec::with_capacity(32);
                for item in items_iter {
                    result.push(item?)
                }

                Ok(result)
            }
            _ => Ok(vec![]),
        }
    }

    async fn update_running_by_identifier_and_owner(
        &self,
        identifier: &PromiseIdentifier,
        owner: &PersonIdentifier,
        running: &PromiseRunning,
    ) -> RecorderResult<()> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "UPDATE Promise SET running = ?1 WHERE identifier = ?2 AND owner = ?3";

                let conn = c.lock()?;
                let mut statement = conn.prepare(statement)?;

                statement.execute(params![running, identifier, owner])?;

                Ok(())
            }
            _ => Ok(()),
        }
    }
}

// ===== Rusqlite SQL converted =====
#[rustfmt::skip]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PromiseCategory {
    BinanceSpotLimit = 1
}

impl FromSql for PromiseCategory {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let num = value.as_i64()?;
        let result = match num {
            1 => Self::BinanceSpotLimit,
            _ => return Err(FromSqlError::InvalidType),
        };
        Ok(result)
    }
}

impl ToSql for PromiseCategory {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(*self as u8))
    }
}

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PromiseRunning {
    Stopped = 0,
    Running = 1,
}

impl FromSql for PromiseRunning {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let num = value.as_i64()?;
        let result = match num {
            0 => Self::Stopped,
            1 => Self::Running,
            _ => return Err(FromSqlError::InvalidType),
        };
        Ok(result)
    }
}

impl ToSql for PromiseRunning {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(*self as u8))
    }
}
