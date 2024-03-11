pub mod logging;

use super::*;
use crate::database::{Recorder, RecorderResult, Uniquer};

pub type Item = Promise;
pub type PromiseIdentifier = String;
pub type PromiseProcess = String;

#[rustfmt::skip]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Promise {
    pub identifier:     PromiseIdentifier,
    pub owner:          PersonIdentifier,
    pub process:        PromiseProcess,
    pub category:       PromiseProcessCategory,
    pub status:         PromiseProcessStatus,
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
            status:         row.get(3)?,
            process:        row.get(4)?,
            total_runs:     row.get(5)?,
            interval:       row.get(6)?,
            timeout:        row.get(7)?,
            max_concurrent: row.get(8)?,
        };

        Ok(result)
    }

    pub fn with_process<T>(
        owner: String,
        category: PromiseProcessCategory,
        process: &T,
    ) -> RecorderResult<Self>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        let result = Self {
            identifier: unique::uuid(),
            owner,
            category,
            status: PromiseProcessStatus::Stopped,
            process: Self::serde_process(process)?,
            total_runs: 0,
            interval: 3,
            timeout: 1800,
            max_concurrent: 1,
        };

        Ok(result)
    }

    pub fn process<T>(&self) -> RecorderResult<T>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        let result = serde_json::from_str(&self.process)?;

        Ok(result)
    }

    pub fn serde_process<T>(value: &T) -> RecorderResult<String>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        let result = serde_json::to_string(value)?;

        Ok(result)
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

    fn select_all_by_status(&self, status: &PromiseProcessStatus)
        -> impl Future<Output = RecorderResult<Vec<Item>>> + Send;

    fn select_one_by_identifier(&self, identifier: &PromiseIdentifier) 
        -> impl Future<Output = RecorderResult<Option<Item>>> + Send;

    fn select_one_by_identifier_owner(&self, identifier: &PromiseIdentifier, owner: &PersonIdentifier) 
        -> impl Future<Output = RecorderResult<Option<Item>>> + Send;

    fn select_one_by_identifier_owner_category(&self, identifier: &PromiseIdentifier, owner: &PersonIdentifier, category: &PromiseProcessCategory) 
        -> impl Future<Output = RecorderResult<Option<Item>>> + Send;
    
    fn select_all_by_owner_category(&self, owner: &PersonIdentifier, category: &PromiseProcessCategory) 
        -> impl Future<Output = RecorderResult<Vec<Item>>> + Send;

    fn update_process_by_identifier_category(&self, process: &String, identifier: &PromiseIdentifier, category: &PromiseProcessCategory)
        -> impl Future<Output = RecorderResult<()>> + Send;
       
    fn update_status_by_identifier(&self, status: &PromiseProcessStatus, identifier: &PromiseIdentifier) 
        -> impl Future<Output = RecorderResult<()>> + Send;
    
    fn update_one_total_runs_by_identifier(&self, identifier: &PromiseIdentifier)
        -> impl Future<Output = RecorderResult<()>> + Send;
}

impl Interface for Recorder<Item> {
    async fn insert(&self, item: &Item) -> RecorderResult<()> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "INSERT INTO Promise (identifier, owner, category, status, process, total_runs, interval, timeout, max_concurrent) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)";

                let conn = c.lock()?;
                let mut stmt = conn.prepare(statement)?;
                stmt.execute((
                    &item.identifier,
                    &item.owner,
                    &item.category,
                    &item.status,
                    &item.process,
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
                let statement = "SELECT identifier, owner, category, status, process, total_runs, interval, timeout, max_concurrent FROM Promise";

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

    async fn select_all_by_status(
        &self,
        status: &PromiseProcessStatus,
    ) -> RecorderResult<Vec<Item>> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "SELECT identifier, owner, category, status, process, total_runs, interval, timeout, max_concurrent FROM Promise WHERE status = ?1";

                let conn = c.lock()?;
                let mut statement = conn.prepare(statement)?;
                let items_iter =
                    statement.query_map(params![status], |row| Item::with_sqlite_row(row))?;

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
                    "SELECT identifier, owner, category, status, process, total_runs, interval, timeout, max_concurrent FROM Promise WHERE identifier = :identifier";

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

    async fn select_one_by_identifier_owner(
        &self,
        identifier: &PromiseIdentifier,
        owner: &PersonIdentifier,
    ) -> RecorderResult<Option<Item>> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "SELECT identifier, owner, category, status, process, total_runs, interval, timeout, max_concurrent FROM Promise WHERE identifier = :identifier AND owner = :owner";

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
        category: &PromiseProcessCategory,
    ) -> RecorderResult<Option<Item>> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "SELECT identifier, owner, category, status, process, total_runs, interval, timeout, max_concurrent FROM Promise WHERE identifier = ?1 AND owner = ?2 AND category = ?3";

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
        category: &PromiseProcessCategory,
    ) -> RecorderResult<Vec<Item>> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "SELECT identifier, owner, category, status, process, total_runs, interval, timeout, max_concurrent FROM Promise WHERE owner = ?1 AND category = ?2";

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

    async fn update_process_by_identifier_category(
        &self,
        process: &String,
        identifier: &PromiseIdentifier,
        category: &PromiseProcessCategory,
    ) -> RecorderResult<()> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "UPDATE Promise SET process = ?1 WHERE identifier = ?2 AND category =?3";

                let conn = c.lock()?;
                let mut statement = conn.prepare(statement)?;

                statement.execute(params![process, identifier, category])?;

                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn update_status_by_identifier(
        &self,
        status: &PromiseProcessStatus,
        identifier: &PromiseIdentifier,
    ) -> RecorderResult<()> {
        match self {
            Self::Sqlite(c) => {
                let statement = "UPDATE Promise SET status = ?1 WHERE identifier = ?2";

                let conn = c.lock()?;
                let mut statement = conn.prepare(statement)?;

                statement.execute(params![status, identifier])?;

                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn update_one_total_runs_by_identifier(
        &self,
        identifier: &PromiseIdentifier,
    ) -> RecorderResult<()> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "UPDATE Promise SET total_runs = total_runs + 1 WHERE identifier = ?1";

                let conn = c.lock()?;
                let mut statement = conn.prepare(statement)?;

                statement.execute(params![identifier])?;

                Ok(())
            }
            _ => Ok(()),
        }
    }
}

// ===== Rusqlite SQL converted =====
#[rustfmt::skip]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PromiseProcessCategory {
    BinanceSpotLimit = 1
}

impl FromSql for PromiseProcessCategory {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let num = value.as_i64()?;
        let result = match num {
            1 => Self::BinanceSpotLimit,
            _ => return Err(FromSqlError::InvalidType),
        };
        Ok(result)
    }
}

impl ToSql for PromiseProcessCategory {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(*self as u8))
    }
}

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PromiseProcessStatus {
    Stopped = 0,
    Running = 1,
}

impl FromSql for PromiseProcessStatus {
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

impl ToSql for PromiseProcessStatus {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(*self as u8))
    }
}

pub mod process {
    use super::*;

    pub use self::binance_spot_limit::Process as PromiseProcessBinanceSpotLimit;

    mod binance_spot_limit {
        use rust_binance::strategy::limit::Limit;

        use super::*;

        #[derive(Serialize, Deserialize)]
        pub struct Process {
            pub symbol: String,
            pub limit: Limit,
        }

        impl Promise {
            pub fn with_process_binance_spot_limit(
                owner: String,
                process: &Process,
            ) -> RecorderResult<Self> {
                Self::with_process::<Process>(
                    owner,
                    PromiseProcessCategory::BinanceSpotLimit,
                    &process,
                )
            }

            pub fn process_binance_spot_limit(&self) -> Option<Process> {
                self.process().ok()
            }
        }
    }
}
