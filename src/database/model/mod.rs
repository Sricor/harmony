pub mod binance;
pub mod cyptocurrency;
pub mod person;
pub mod promise;

use std::future::Future;

use rusqlite::params;
use rusqlite::{
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef},
    Row, ToSql,
};
use serde::{Deserialize, Serialize};

pub use self::person::PersonIdentifier;

mod time {
    use chrono::prelude::Utc;

    pub(super) fn timestamp_millis() -> i64 {
        let now = Utc::now();

        now.timestamp_millis()
    }
}

mod unique {
    use uuid::Uuid;

    pub(super) fn uuid() -> String {
        Uuid::new_v4().to_string()
    }
}

// ===== Person =====
const SQLITE_TABLE_PERSON: &str = "
CREATE TABLE IF NOT EXISTS Person
(
    serial      INTEGER PRIMARY KEY NOT NULL,
    identifier  TEXT                NOT NULL  UNIQUE,
    name        TEXT                NOT NULL  UNIQUE,
    role        INTEGER             NOT NULL,
    password    TEXT                NOT NULL
)";

// ===== Promise =====

// Enum Category
// 1. Binance Spot Limit
const SQLITE_TABLE_PROMISE: &str = "
CREATE TABLE IF NOT EXISTS Promise
(
    serial          INTEGER PRIMARY KEY NOT NULL,
    identifier      TEXT                NOT NULL  UNIQUE,
    owner           TEXT                NOT NULL,
    category        INTEGER             NOT NULL,
    running         INTEGER             NOT NULL,
    total_runs      INTEGER             NOT NULL,
    interval        INTEGER             NOT NULL,
    timeout         INTEGER             NOT NULL,
    max_concurrent  INTEGER             NOT NULL,
    FOREIGN KEY(owner) REFERENCES Person(identifier) ON DELETE CASCADE ON UPDATE CASCADE
)";

const SQLITE_TABLE_PROMISE_LOGGING: &str = "
CREATE TABLE IF NOT EXISTS PromiseLogging
(
    serial          INTEGER PRIMARY KEY NOT NULL,
    promise         TEXT                NOT NULL,
    owner           TEXT                NOT NULL,
    message         TEXT                NOT NULL,
    level           INTEGER             NOT NULL,
    timestamp       INTEGER             NOT NULL,
    FOREIGN KEY(owner) REFERENCES Person(identifier) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY(promise)  REFERENCES Promise(identifier) ON DELETE CASCADE ON UPDATE CASCADE
)";

const SQLITE_TABLE_PROMISE_BINANCE_SPOT_LIMIT: &str = "
CREATE TABLE IF NOT EXISTS PromiseBinanceSpotLimit
(
    serial          INTEGER PRIMARY KEY NOT NULL,
    promise         TEXT                NOT NULL,
    owner           TEXT                NOT NULL,
    symbol          TEXT                NOT NULL,
    buying_low      TEXT                NOT NULL,
    buying_high     TEXT                NOT NULL,
    selling_low     TEXT                NOT NULL,
    selling_high    TEXT                NOT NULL,
    investment      TEXT                NOT NULL,
    position        TEXT                NOT NULL,
    FOREIGN KEY(owner) REFERENCES Person(identifier) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY(promise)  REFERENCES Promise(identifier) ON DELETE CASCADE ON UPDATE CASCADE
)";

// ===== Cryptocurrency
const SQLITE_TABLE_CRYPTOCURRENCY_PRICE: &str = "
CREATE TABLE IF NOT EXISTS CryptocurrencyPrice
(
    serial      INTEGER PRIMARY KEY NOT NULL,
    symbol      TEXT                NOT NULL,
    price       TEXT                NOT NULL,
    timestamp   INTEGER             NOT NULL,
)";

// ===== Binance =====
const SQLITE_TABLE_BINANCE_SECRET: &str = "
CREATE TABLE IF NOT EXISTS BinanceSecret
(
    serial      INTEGER PRIMARY KEY NOT NULL,
    owner       TEXT                NOT NULL,
    purview     INTEGER             NOT NULL,
    api_key     TEXT                NOT NULL,
    secret_key  TEXT                NOT NULL,
    FOREIGN KEY(owner) REFERENCES Person(identifier) ON DELETE CASCADE ON UPDATE CASCADE
)";

const SQLITE_TABLE_BINANCE_SPOT: &str = "
CREATE TABLE IF NOT EXISTS BinanceSpot
(
    serial                          INTEGER PRIMARY KEY NOT NULL,
    owner                           TEXT                NOT NULL,
    symbol                          TEXT                NOT NULL,
    transaction_quantity_precision  INTEGER             NOT NULL,
    quantity_precision              INTEGER             NOT NULL,
    amount_precision                INTEGER             NOT NULL,
    buying_commission               TEXT                NOT NULL,
    selling_commission              TEXT                NOT NULL,
    minimum_transaction_amount      TEXT                NOT NULL,
    FOREIGN KEY(owner) REFERENCES Person(identifier) ON DELETE CASCADE ON UPDATE CASCADE
)";

const SQLITE_TABLE_BINANCE_SPOT_BUYING_ORDER: &str = "
CREATE TABLE IF NOT EXISTS BinanceSpotBuyingOrder
(
    serial                     INTEGER PRIMARY KEY NOT NULL,
    owner                      TEXT                NOT NULL,
    symbol                     TEXT                NOT NULL,
    price                      TEXT                NOT NULL,
    quantity                   TEXT                NOT NULL,
    spent                      TEXT                NOT NULL,
    quantity_after_commission  TEXT                NOT NULL,
    timestamp                  INTEGER             NOT NULL,
    FOREIGN KEY(owner) REFERENCES Person(identifier) ON DELETE CASCADE ON UPDATE CASCADE
)";

const SQLITE_TABLE_BINANCE_SPOT_SELLING_ORDER: &str = "
CREATE TABLE IF NOT EXISTS BinanceSpotSellingOrder
(
    serial                   INTEGER PRIMARY KEY NOT NULL,
    owner                    TEXT                NOT NULL,
    symbol                   TEXT                NOT NULL,
    price                    TEXT                NOT NULL,
    quantity                 TEXT                NOT NULL,
    income                   TEXT                NOT NULL,
    income_after_commission  TEXT                NOT NULL,
    timestamp                INTEGER             NOT NULL,
    FOREIGN KEY(owner) REFERENCES Person(identifier) ON DELETE CASCADE ON UPDATE CASCADE
)";

pub fn sqlite_table_normal() -> Vec<&'static str> {
    vec![
        SQLITE_TABLE_PERSON,
        SQLITE_TABLE_PROMISE,
        SQLITE_TABLE_PROMISE_LOGGING,
        SQLITE_TABLE_PROMISE_BINANCE_SPOT_LIMIT,
        SQLITE_TABLE_CRYPTOCURRENCY_PRICE,
        SQLITE_TABLE_BINANCE_SECRET,
        SQLITE_TABLE_BINANCE_SPOT,
        SQLITE_TABLE_BINANCE_SPOT_BUYING_ORDER,
        SQLITE_TABLE_BINANCE_SPOT_SELLING_ORDER,
    ]
}

// ===== Cryptocurrency =====
const SQLITE_TABLE_CRYPTOCURRENCY_PRICE: &str = "
CREATE TABLE IF NOT EXISTS CryptocurrencyPrice
(
    serial      INTEGER PRIMARY KEY NOT NULL,
    symbol      TEXT                NOT NULL,
    price       TEXT                NOT NULL,
    timestamp   INTEGER             NOT NULL,
)";

pub fn sqlite_table_inventory_cryptocurrency() -> Vec<&'static str> {
    vec![SQLITE_TABLE_CRYPTOCURRENCY_PRICE]
}
