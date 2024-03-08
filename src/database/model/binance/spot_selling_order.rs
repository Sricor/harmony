use super::*;

use crate::database::{
    model::{person::PersonIdentifier, time},
    Recorder, RecorderResult,
};

pub type Item = BinanceSpotSellingOrder;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceSpotSellingOrder {
    pub owner: PersonIdentifier,

    pub symbol: String,

    /// Selling price
    pub price: String,

    /// Selling quantity
    pub quantity: String,

    /// Income gained after selling
    pub income: String,

    /// Income gained after commission selling, also the actual income recorded
    pub income_after_commission: String,

    pub timestamp: i64,
}

impl Item {
    fn with_sqlite_row(row: &Row<'_>) -> rusqlite::Result<Item> {
        let result = Item {
            owner: row.get(0)?,
            symbol: row.get(1)?,
            price: row.get(2)?,
            quantity: row.get(3)?,
            income: row.get(4)?,
            income_after_commission: row.get(5)?,
            timestamp: row.get(6)?,
        };

        Ok(result)
    }
}

impl Default for Item {
    fn default() -> Self {
        Self {
            owner: String::default(),
            symbol: String::default(),
            price: String::default(),
            quantity: String::default(),
            income: String::default(),
            income_after_commission: String::default(),
            timestamp: time::timestamp_millis(),
        }
    }
}

#[rustfmt::skip]
pub trait Interface {
    fn insert(&self, item: Item)
        -> impl Future<Output = RecorderResult<()>> + Send;

    fn select_all(&self)
        -> impl Future<Output = RecorderResult<Vec<Item>>> + Send;

    fn select_all_by_owner(&self, owner: &PersonIdentifier) 
        -> impl Future<Output = RecorderResult<Vec<Item>>> + Send;
}

impl Interface for Recorder<Item> {
    async fn insert(&self, item: Item) -> RecorderResult<()> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "INSERT INTO BinanceSpotSellingOrder (owner, symbol, price, quantity, income, income_after_commission, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)";

                let conn = c.lock()?;
                let mut stmt = conn.prepare(statement)?;
                stmt.execute((
                    &item.owner,
                    &item.symbol,
                    &item.price,
                    &item.quantity,
                    &item.income,
                    &item.income_after_commission,
                    &item.timestamp,
                ))?;

                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn select_all(&self) -> RecorderResult<Vec<Item>> {
        match self {
            Self::Sqlite(c) => {
                let statement = "SELECT owner, symbol, price, quantity, income, income_after_commission, timestamp FROM BinanceSpotSellingOrder";

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

    async fn select_all_by_owner(&self, owner: &PersonIdentifier) -> RecorderResult<Vec<Item>> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "SELECT owner, symbol, price, quantity, income, income_after_commission, timestamp FROM BinanceSpotSellingOrder WHERE owner = :owner";

                let conn = c.lock()?;
                let mut statement = conn.prepare(statement)?;

                let items_iter =
                    statement.query_map(&[(":owner", &owner)], |row| Item::with_sqlite_row(row))?;

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
