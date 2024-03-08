use super::*;

use crate::database::{
    model::{person::PersonIdentifier, time},
    Recorder, RecorderResult,
};

pub type Item = BinanceSpotBuyingOrder;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceSpotBuyingOrder {
    pub owner: PersonIdentifier,

    pub symbol: String,

    /// Buying price
    pub price: String,

    /// Buying quantity
    pub quantity: String,

    /// Amount spent on buying
    pub spent: String,

    /// Buying quantity after commission, also the actual quantity held
    pub quantity_after_commission: String,

    pub timestamp: i64,
}

impl Item {
    fn with_sqlite_row(row: &Row<'_>) -> rusqlite::Result<Item> {
        let result = Item {
            owner: row.get(0)?,
            symbol: row.get(1)?,
            price: row.get(2)?,
            quantity: row.get(3)?,
            spent: row.get(4)?,
            quantity_after_commission: row.get(5)?,
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
            spent: String::default(),
            quantity_after_commission: String::default(),
            timestamp: time::timestamp_millis(),
        }
    }
}

#[rustfmt::skip]
pub trait Interface {
    fn insert(&self, item: &Item)
        -> impl Future<Output = RecorderResult<()>> + Send;

    fn select_all(&self)
        -> impl Future<Output = RecorderResult<Vec<Item>>> + Send;

    fn select_all_by_owner(&self, owner: &PersonIdentifier) 
        -> impl Future<Output = RecorderResult<Vec<Item>>> + Send;
}

impl Interface for Recorder<Item> {
    async fn insert(&self, item: &Item) -> RecorderResult<()> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "INSERT INTO BinanceSpotBuyingOrder (owner, symbol, price, quantity, spent, quantity_after_commission, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)";

                let conn = c.lock()?;
                let mut stmt = conn.prepare(statement)?;
                stmt.execute((
                    &item.owner,
                    &item.symbol,
                    &item.price,
                    &item.quantity,
                    &item.spent,
                    &item.quantity_after_commission,
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
                let statement = "SELECT owner, symbol, price, quantity, spent, quantity_after_commission, timestamp FROM BinanceSpotBuyingOrder";

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
                    "SELECT owner, symbol, price, quantity, spent, quantity_after_commission, timestamp FROM BinanceSpotBuyingOrder WHERE owner = :owner";

                let conn = c.lock()?;
                let mut statement = conn.prepare(statement)?;

                let items_iter =
                    statement.query_map(&[(":owner", owner)], |row| Item::with_sqlite_row(row))?;

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
