use serde::{Deserialize, Serialize};
use std::future::Future;

use crate::database::{Recorder, RecorderResult};

pub type Item = CryptocurrencyPrice;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptocurrencyPrice {
    pub symbol: String,
    pub price: String,
    pub timestamp: i64,
}

impl Item {
    fn with_sqlite_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        let result = Self {
            symbol: row.get(0)?,
            price: row.get(1)?,
            timestamp: row.get(2)?,
        };

        Ok(result)
    }
}

#[rustfmt::skip]
pub trait Interface {
    fn insert(&self, item: &Item)
        -> impl Future<Output = RecorderResult<()>> + Send;

    fn select_latest_by_symbol(&self, symbol: &String)
        -> impl Future<Output = RecorderResult<Option<Item>>> + Send;
}

impl Interface for Recorder<Item> {
    async fn insert(&self, item: &Item) -> RecorderResult<()> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "INSERT INTO CryptocurrencyPrice (symbol, price, timestamp) VALUES (?1, ?2, ?3)";

                let conn = c.lock()?;

                let mut stmt = conn.prepare(statement)?;
                stmt.execute((&item.symbol, &item.price, &item.timestamp))?;

                Ok(())
            }
            _ => todo!(),
        }
    }

    async fn select_latest_by_symbol(&self, symbol: &String) -> RecorderResult<Option<Item>> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "SELECT symbol, price, timestamp FROM CryptocurrencyPrice WHERE symbol = ?1 ORDER BY timestamp DESC LIMIT 1";

                let conn = c.lock()?;
                let mut statement = conn.prepare(statement)?;
                let rows = statement.query_map(&[symbol], |row| Item::with_sqlite_row(row))?;

                let person = rows.into_iter().nth(0);

                let result = match person {
                    Some(v) => Some(v?),
                    None => None,
                };

                Ok(result)
            }
            _ => todo!(),
        }
    }
}
