use super::*;
use crate::database::{Recorder, RecorderResult};

pub type Item = PromiseBinanceSpotLimit;

#[rustfmt::skip]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromiseBinanceSpotLimit {
    pub promise:      PromiseIdentifier,
    pub owner:        PersonIdentifier,

    // Symbol
    pub symbol:       String,
    // Selling Range
    pub buying_low:   String,
    pub buying_high:  String,

    // Buying Range
    pub selling_low:  String,
    pub selling_high: String,

    // Investment amount
    pub investment:   String,

    // Quantity
    pub position:     String,
}

impl Item {
    #[rustfmt::skip]
    fn with_sqlite_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        let result = Self {
            promise:     row.get(0)?,
            owner:       row.get(1)?,
            symbol:      row.get(2)?,
            buying_low:  row.get(3)?,
            buying_high: row.get(4)?,
            selling_low: row.get(5)?,
            selling_high:row.get(6)?,
            investment:  row.get(7)?,
            position:    row.get(8)?,
        };

        Ok(result)
    }
}

#[rustfmt::skip]
pub trait Interface {
    fn insert(&self, item: &Item)
        -> impl Future<Output = RecorderResult<()>> + Send;

    fn select_all(&self)
        -> impl Future<Output = RecorderResult<Vec<Item>>> + Send;

    fn select_one_by_promise_and_owner(&self, promise: &PromiseIdentifier, owner: &PersonIdentifier) 
        -> impl Future<Output = RecorderResult<Option<Item>>> + Send;
    
    fn replace_by_promise(&self, item: &Item) 
        -> impl Future<Output = RecorderResult<()>> + Send;
    
    fn select_one_by_promise(&self, promise: &PromiseIdentifier) 
        -> impl Future<Output = RecorderResult<Option<Item>>> + Send;
}

impl Interface for Recorder<Item> {
    async fn insert(&self, item: &Item) -> RecorderResult<()> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "INSERT INTO PromiseBinanceSpotLimit (promise, owner, symbol, buying_low, buying_high, selling_low, selling_high, investment, position) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)";

                let conn = c.lock()?;
                let mut stmt = conn.prepare(statement)?;
                stmt.execute((
                    &item.promise,
                    &item.owner,
                    &item.symbol,
                    &item.buying_low,
                    &item.buying_high,
                    &item.selling_low,
                    &item.selling_high,
                    &item.investment,
                    &item.position,
                ))?;

                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn select_all(&self) -> RecorderResult<Vec<Item>> {
        match self {
            Self::Sqlite(c) => {
                let statement = "SELECT promise, owner, symbol, buying_low, buying_high, selling_low, selling_high, investment, position FROM PromiseBinanceSpotLimit";

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

    async fn select_one_by_promise_and_owner(
        &self,
        promise: &PromiseIdentifier,
        owner: &PersonIdentifier,
    ) -> RecorderResult<Option<Item>> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "SELECT promise, owner, symbol, buying_low, buying_high, selling_low, selling_high, investment, position FROM PromiseBinanceSpotLimit WHERE promise = :promise AND owner = :owner";

                let conn = c.lock()?;
                let mut statement = conn.prepare(statement)?;
                let items_iter = statement
                    .query_map(&[(":promise", promise), (":owner", owner)], |row| {
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

    async fn select_one_by_promise(
        &self,
        promise: &PromiseIdentifier,
    ) -> RecorderResult<Option<Item>> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "SELECT promise, owner, symbol, buying_low, buying_high, selling_low, selling_high, investment, position FROM PromiseBinanceSpotLimit WHERE promise = ?1";

                let conn = c.lock()?;
                let mut statement = conn.prepare(statement)?;
                let items_iter =
                    statement.query_map(params![promise], |row| Item::with_sqlite_row(row))?;

                let result = match items_iter.into_iter().nth(0) {
                    Some(item) => Some(item?),
                    None => None,
                };
                Ok(result)
            }
            _ => Ok(None),
        }
    }

    async fn replace_by_promise(&self, item: &Item) -> RecorderResult<()> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "UPDATE PromiseBinanceSpotLimit SET owner = ?1, symbol = ?2, buying_low = ?3, buying_high = ?4, selling_low = ?5, selling_high = ?6, investment = ?7, position = ?8 WHERE promise = ?9";

                let conn = c.lock()?;
                let mut statement = conn.prepare(statement)?;
                statement.execute(params![
                    item.owner,
                    item.symbol,
                    item.buying_low,
                    item.buying_high,
                    item.selling_low,
                    item.selling_high,
                    item.investment,
                    item.position,
                    item.promise
                ])?;

                Ok(())
            }
            _ => Ok(()),
        }
    }
}
