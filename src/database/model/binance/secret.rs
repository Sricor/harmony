use super::*;

use crate::database::{model::person::PersonIdentifier, Recorder, RecorderResult};

pub type Item = BinanceSecret;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceSecret {
    pub owner: PersonIdentifier,
    pub purview: BinanceSecretPurview,
    pub api_key: String,
    pub secret_key: String,
}

impl Item {
    pub fn with_read(owner: PersonIdentifier, api_key: String, secret_key: String) -> Self {
        Self {
            owner,
            purview: BinanceSecretPurview::Read,
            api_key,
            secret_key,
        }
    }

    pub fn with_spot(owner: PersonIdentifier, api_key: String, secret_key: String) -> Self {
        Self {
            owner,
            purview: BinanceSecretPurview::Spot,
            api_key,
            secret_key,
        }
    }

    fn with_sqlite_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        let result = Self {
            owner: row.get(0)?,
            purview: row.get(1)?,
            api_key: row.get(2)?,
            secret_key: row.get(3)?,
        };

        Ok(result)
    }
}

#[rustfmt::skip]
pub trait Interface {
    fn insert(&self, item: &Item)
        -> impl Future<Output = RecorderResult<()>> + Send;

    fn select_one_by_owner_and_purview(&self, owner: &PersonIdentifier, purview: &BinanceSecretPurview)
        -> impl Future<Output = RecorderResult<Option<Item>>> + Send;

    fn select_one_spot_by_owner(&self, owner: &PersonIdentifier)
        -> impl Future<Output = RecorderResult<Option<Item>>> + Send;

    fn select_all_by_owner(&self, owner: &PersonIdentifier) 
        -> impl Future<Output = RecorderResult<Vec<Item>>> + Send;
}

impl Interface for Recorder<Item> {
    async fn insert(&self, item: &Item) -> RecorderResult<()> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "INSERT INTO BinanceSecret (owner, purview, api_key, secret_key) VALUES (?1, ?2, ?3, ?4)";

                let conn = c.lock()?;

                let mut stmt = conn.prepare(statement)?;
                stmt.execute((&item.owner, &item.purview, &item.api_key, &item.secret_key))?;

                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn select_one_by_owner_and_purview(
        &self,
        owner: &PersonIdentifier,
        purview: &BinanceSecretPurview,
    ) -> RecorderResult<Option<Item>> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "SELECT owner, purview, api_key, secret_key FROM BinanceSecret WHERE owner = :owner AND purview = :purview";

                let conn = c.lock()?;
                let mut statement = conn.prepare(statement)?;
                let rows = statement.query_map(
                    &[(":owner", owner), (":purview", &purview.to_string())],
                    |row| Item::with_sqlite_row(row),
                )?;

                let person = rows.into_iter().nth(0);

                let result = match person {
                    Some(v) => Some(v?),
                    None => None,
                };

                Ok(result)
            }
            _ => Ok(None),
        }
    }

    async fn select_one_spot_by_owner(
        &self,
        owner: &PersonIdentifier,
    ) -> RecorderResult<Option<Item>> {
        self.select_one_by_owner_and_purview(owner, &BinanceSecretPurview::Spot)
            .await
    }

    async fn select_all_by_owner(&self, owner: &PersonIdentifier) -> RecorderResult<Vec<Item>> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "SELECT owner, purview, api_key, secret_key FROM BinanceSecret WHERE owner = :owner";

                let conn = c.lock()?;
                let mut statement = conn.prepare(statement)?;

                let item_iter =
                    statement.query_map(&[(":owner", &owner)], |row| Item::with_sqlite_row(row))?;

                let mut result = Vec::with_capacity(32);
                for item in item_iter {
                    result.push(item?)
                }

                Ok(result)
            }
            _ => Ok(vec![]),
        }
    }
}

// ===== Rusqlite SQL converted =====
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BinanceSecretPurview {
    Read = 1,
    Spot = 2,
}

impl BinanceSecretPurview {
    fn to_string(&self) -> String {
        (*self as u8).to_string()
    }
}

impl FromSql for BinanceSecretPurview {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let num = value.as_i64()?;
        let result = match num {
            1 => BinanceSecretPurview::Read,
            2 => BinanceSecretPurview::Spot,
            _ => return Err(FromSqlError::InvalidType),
        };
        Ok(result)
    }
}

impl ToSql for BinanceSecretPurview {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(*self as u8))
    }
}
