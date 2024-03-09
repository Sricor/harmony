use std::{
    error::Error,
    fmt::Display,
    sync::{Arc, Mutex, PoisonError},
};

use rusqlite::{Connection, Error as RusqliteError};

use super::{collection::*, model};

pub struct DatabaseOptions {
    normal_name: &'static str,
    crypto_name: &'static str,
}

impl Default for DatabaseOptions {
    fn default() -> Self {
        Self {
            normal_name: "harmony.db",
            crypto_name: "cryptocurrency.db",
        }
    }
}

pub struct Database {
    pub person: Recorder<Person>,
    pub binance_secret: Recorder<BinanceSecret>,
    pub binance_spot: Recorder<BinanceSpot>,
    pub binance_spot_buying_order: Recorder<BinanceSpotBuyingOrder>,
    pub binance_spot_selling_order: Recorder<BinanceSpotSellingOrder>,
    pub promise: Recorder<Promise>,
    pub promise_logging: Recorder<PromiseLogging>,
    pub promise_binance_spot_limit: Recorder<PromiseBinanceSpotLimit>,

    pub cryptocurrency: Recorder<CyptocurrencyPrice>,
}

impl Database {
    pub fn new(options: Option<DatabaseOptions>) -> Self {
        let options = options.unwrap_or_default();
        let normal = Self::create_sqlite_normal(&options.normal_name);
        let crypto = Self::create_sqlite_cryptocurrency(&options.crypto_name);

        Self {
            person: Recorder::with_sqlite(normal.clone()),
            binance_secret: Recorder::with_sqlite(normal.clone()),
            binance_spot: Recorder::with_sqlite(normal.clone()),
            binance_spot_buying_order: Recorder::with_sqlite(normal.clone()),
            binance_spot_selling_order: Recorder::with_sqlite(normal.clone()),
            promise: Recorder::with_sqlite(normal.clone()),
            promise_logging: Recorder::with_sqlite(normal.clone()),
            promise_binance_spot_limit: Recorder::with_sqlite(normal.clone()),

            cryptocurrency: Recorder::with_sqlite(crypto.clone()),
        }
    }

    fn create_sqlite_normal(url: &str) -> Arc<Mutex<Connection>> {
        let conn = Connection::open(url).unwrap();
        model::sqlite_table_normal().iter().for_each(|e| {
            conn.execute(e, ()).unwrap();
        });

        Arc::new(Mutex::new(conn))
    }

    fn create_sqlite_cryptocurrency(url: &str) -> Arc<Mutex<Connection>> {
        let conn = Connection::open(url).unwrap();
        model::sqlite_table_inventory_cryptocurrency()
            .iter()
            .for_each(|e| {
                conn.execute(e, ()).unwrap();
            });

        Arc::new(Mutex::new(conn))
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
