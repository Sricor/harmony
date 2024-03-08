use std::{
    error::Error,
    fmt::Display,
    sync::{Arc, Mutex, PoisonError},
};

use rusqlite::{Connection, Error as RusqliteError};

use super::{collection::*, model};

pub struct Database {
    pub person: Recorder<Person>,
    pub binance_secret: Recorder<BinanceSecret>,
    pub binance_spot: Recorder<BinanceSpot>,
    pub binance_spot_buying_order: Recorder<BinanceSpotBuyingOrder>,
    pub binance_spot_selling_order: Recorder<BinanceSpotSellingOrder>,
    pub promise: Recorder<Promise>,
    pub promise_logging: Recorder<PromiseLogging>,
    pub promise_binance_spot_limit: Recorder<PromiseBinanceSpotLimit>,
}

impl Database {
    pub async fn new(url: &str) -> Self {
        let conn = Connection::open(url).unwrap();
        let conn: Arc<Mutex<Connection>> = Arc::new(Mutex::new(conn));
        Self::create_sqlite_table(&conn).await;

        Self {
            person: Recorder::with_sqlite(conn.clone()),
            binance_secret: Recorder::with_sqlite(conn.clone()),
            binance_spot: Recorder::with_sqlite(conn.clone()),
            binance_spot_buying_order: Recorder::with_sqlite(conn.clone()),
            binance_spot_selling_order: Recorder::with_sqlite(conn.clone()),
            promise: Recorder::with_sqlite(conn.clone()),
            promise_logging: Recorder::with_sqlite(conn.clone()),
            promise_binance_spot_limit: Recorder::with_sqlite(conn.clone()),
        }
    }

    async fn create_sqlite_table(conn: &Mutex<Connection>) {
        let conn = conn.lock().unwrap();
        model::sqlite_table_inventory().iter().for_each(|e| {
            conn.execute(e, ()).unwrap();
        });
    }
}

trait Sqlite {
    fn with_sqlite(conn: Arc<Mutex<Connection>>) -> Self;
}

pub enum Recorder<T> {
    Sqlite(Arc<Mutex<Connection>>),

    // TODO
    Mongo(T),
}

impl<T> Sqlite for Recorder<T> {
    fn with_sqlite(conn: Arc<Mutex<Connection>>) -> Self {
        Self::Sqlite(conn)
    }
}

#[derive(Debug)]
pub enum RecorderError {
    Sqlite(String),
    Mutex(String),
}

impl Error for RecorderError {}

impl Display for RecorderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::Sqlite(e) => e,
            Self::Mutex(e) => e,
        };

        write!(f, "{}", message)
    }
}

impl From<RusqliteError> for RecorderError {
    fn from(value: RusqliteError) -> Self {
        RecorderError::Sqlite(value.to_string())
    }
}

impl<T> From<PoisonError<T>> for RecorderError {
    fn from(value: PoisonError<T>) -> Self {
        Self::Mutex(value.to_string())
    }
}

pub type RecorderResult<T> = Result<T, RecorderError>;

pub trait Uniquer {
    type Sign;

    fn identifier(&self) -> &Self::Sign;
}
