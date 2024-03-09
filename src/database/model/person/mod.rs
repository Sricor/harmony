use super::*;

use crate::database::{Recorder, RecorderResult, Uniquer};

pub type Item = Person;
pub type PersonIdentifier = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub identifier: PersonIdentifier,

    pub name: String,
    pub role: PersonRole,
    pub password: String,
}

impl Person {
    pub fn new(name: String, role: PersonRole, password: String) -> Self {
        Self {
            identifier: unique::uuid(),
            name,
            role,
            password,
        }
    }

    pub fn with_normal(name: String, password: String) -> Self {
        Self {
            identifier: unique::uuid(),
            name,
            role: PersonRole::Normal,
            password,
        }
    }

    fn with_sqlite_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        let result = Self {
            identifier: row.get(0)?,
            name: row.get(1)?,
            role: row.get(2)?,
            password: row.get(3)?,
        };

        Ok(result)
    }
}

impl Uniquer for Person {
    type Sign = PersonIdentifier;

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

    fn select_one_by_identifier(&self, identifier: &PersonIdentifier) 
        -> impl Future<Output = RecorderResult<Option<Item>>> + Send;
        
    fn select_one_by_name(&self, name: &String) 
        -> impl Future<Output = RecorderResult<Option<Item>>> + Send;

    fn select_identifier_by_name_and_password(&self, name: &String, password: &String)
        -> impl Future<Output = RecorderResult<Option<PersonIdentifier>>> + Send;
}

impl Interface for Recorder<Item> {
    async fn insert(&self, item: &Item) -> RecorderResult<()> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "INSERT INTO Person (identifier, name, role, password) VALUES (?1, ?2, ?3, ?4)";

                let conn = c.lock()?;
                let mut stmt = conn.prepare(statement)?;
                stmt.execute((&item.identifier, &item.name, &item.role, &item.password))?;

                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn select_all(&self) -> RecorderResult<Vec<Item>> {
        match self {
            Self::Sqlite(c) => {
                let statement = "SELECT identifier, name, role, password FROM Person";

                let conn = c.lock()?;
                let mut statement = conn.prepare(statement)?;
                let person_iter = statement.query_map([], |row| {
                    Ok(Person {
                        identifier: row.get(0)?,
                        name: row.get(1)?,
                        role: row.get(2)?,
                        password: row.get(3)?,
                    })
                })?;
                let mut result = Vec::with_capacity(32);
                for person in person_iter {
                    result.push(person?)
                }

                Ok(result)
            }
            _ => Ok(vec![]),
        }
    }

    async fn select_one_by_identifier(
        &self,
        identifier: &PersonIdentifier,
    ) -> RecorderResult<Option<Item>> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "SELECT identifier, name, role, password FROM Person WHERE identifier = :identifier";

                let conn = c.lock()?;
                let mut statement = conn.prepare(statement)?;

                let rows = statement.query_map(&[(":identifier", identifier)], |row| {
                    Ok(Item {
                        identifier: row.get(0)?,
                        name: row.get(1)?,
                        role: row.get(2)?,
                        password: row.get(3)?,
                    })
                })?;

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

    async fn select_one_by_name(&self, name: &PersonIdentifier) -> RecorderResult<Option<Item>> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "SELECT identifier, name, role, password FROM Person WHERE name = :name";

                let conn = c.lock()?;
                let mut statement = conn.prepare(statement)?;

                let rows =
                    statement.query_map(&[(":name", name)], |row| Item::with_sqlite_row(row))?;

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

    async fn select_identifier_by_name_and_password(
        &self,
        name: &String,
        password: &String,
    ) -> RecorderResult<Option<PersonIdentifier>> {
        match self {
            Self::Sqlite(c) => {
                let statement =
                    "SELECT identifier FROM Person WHERE name = :name AND password = :password";

                let conn = c.lock()?;
                let mut statement = conn.prepare(statement)?;

                let rows = statement
                    .query_map(&[(":name", &name), (":password", &password)], |row| {
                        row.get(0)
                    })?;

                let person_identifier = rows.into_iter().nth(0);

                let result = match person_identifier {
                    Some(v) => Some(v?),
                    None => None,
                };
                Ok(result)
            }

            // TODO
            Self::Mongo(_c) => Ok(None),
        }
    }
}

// ===== Rusqlite SQL converted =====
#[rustfmt::skip]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PersonRole {
    Normal        = 1,
    Administrator = 2
}

impl FromSql for PersonRole {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let num = value.as_i64()?;
        let result: PersonRole = unsafe { std::mem::transmute(num as u8) };
        Ok(result)
    }
}

impl ToSql for PersonRole {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(*self as u8))
    }
}
