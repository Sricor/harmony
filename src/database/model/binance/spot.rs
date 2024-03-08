use super::*;

use crate::database::{model::person::PersonIdentifier, Recorder, RecorderResult};

pub type Item = BinanceSpot;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceSpot {
    pub owner: PersonIdentifier,
    pub symbol: String,
    pub transaction_quantity_precision: u32,
    pub quantity_precision: u32,
    pub amount_precision: u32,
    pub buying_commission: String,
    pub selling_commission: String,
    pub minimum_transaction_amount: String,
}

impl Item {
    fn with_sqlite_row(row: &Row<'_>) -> rusqlite::Result<Item> {
        let result = Item {
            owner: row.get(0)?,
            symbol: row.get(1)?,
            transaction_quantity_precision: row.get(2)?,
            quantity_precision: row.get(3)?,
            amount_precision: row.get(4)?,
            buying_commission: row.get(5)?,
            selling_commission: row.get(6)?,
            minimum_transaction_amount: row.get(7)?,
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

    fn select_all_by_owner(&self, owner: &PersonIdentifier) 
        -> impl Future<Output = RecorderResult<Vec<Item>>> + Send;
    
    fn select_one_by_owner_and_symbol(&self, owner: &PersonIdentifier, symbol: &String) 
        -> impl Future<Output = RecorderResult<Option<Item>>> + Send;
}

impl Interface for Recorder<Item> {
    async fn insert(&self, item: &Item) -> RecorderResult<()> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "INSERT INTO BinanceSpot (owner, symbol, transaction_quantity_precision, quantity_precision, amount_precision, buying_commission, selling_commission, minimum_transaction_amount) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)";

                let conn = c.lock()?;
                let mut stmt = conn.prepare(statement)?;
                stmt.execute((
                    &item.owner,
                    &item.symbol,
                    &item.transaction_quantity_precision,
                    &item.quantity_precision,
                    &item.amount_precision,
                    &item.buying_commission,
                    &item.selling_commission,
                    &item.minimum_transaction_amount,
                ))?;

                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn select_all(&self) -> RecorderResult<Vec<Item>> {
        match self {
            Self::Sqlite(c) => {
                let statement = "SELECT owner, symbol, transaction_quantity_precision, quantity_precision, amount_precision, buying_commission, selling_commission, minimum_transaction_amount FROM BinanceSpot";

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
                    "SELECT owner, symbol, transaction_quantity_precision, quantity_precision, amount_precision, buying_commission, selling_commission, minimum_transaction_amount FROM BinanceSpot WHERE owner = :owner";

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

    async fn select_one_by_owner_and_symbol(
        &self,
        owner: &PersonIdentifier,
        symbol: &String,
    ) -> RecorderResult<Option<Item>> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "SELECT owner, symbol, transaction_quantity_precision, quantity_precision, amount_precision, buying_commission, selling_commission, minimum_transaction_amount FROM BinanceSpot WHERE owner = :owner AND symbol = :symbol";

                let conn = c.lock()?;
                let mut statement = conn.prepare(statement)?;

                let items_iter = statement
                    .query_map(&[(":owner", owner), (":symbol", symbol)], |row| {
                        Item::with_sqlite_row(row)
                    })?;

                let person = items_iter.into_iter().nth(0);

                let result = match person {
                    Some(v) => Some(v?),
                    None => None,
                };

                Ok(result)
            }
            _ => Ok(None),
        }
    }
}
