mod client;
mod model;

// =====   Database Client   =====
pub use client::Database;
pub use client::{Recorder, RecorderResult, Uniquer};

// ===== Database Collection =====
#[rustfmt::skip]
pub mod collection {
    use super::model;

    pub use model::person::{
        PersonIdentifier,
        Item      as Person,
        Interface as PersonInterface
    };

    pub use model::promise::{
        PromiseIdentifier,
        PromiseProcessCategory,
        PromiseProcessStatus,
        process::PromiseProcessBinanceSpotLimit,
        Item      as Promise,
        Interface as PromiseInterface
    };

    pub use model::promise::logging::{
        PromiseLoggingLevel,
        Item      as PromiseLogging,
        Interface as PromiseLoggingInterface
    };

    // ===== Cryptocurrency =====
    pub use model::cyptocurrency::price::{
        Item      as CyptocurrencyPrice,
        Interface as CyptocurrencyPriceInterface
    };

    // ===== Binance =====
    pub use model::binance::secret::{
        BinanceSecretPurview,
        Item      as BinanceSecret,
        Interface as BinanceSecretInterface
    };

    pub use model::binance::spot::{
        Item      as BinanceSpot,
        Interface as BinanceSpotInterface
    };


    pub use model::binance::spot_buying_order::{
        Item      as BinanceSpotBuyingOrder,
        Interface as BinanceSpotBuyingOrderInterface
    };

    pub use model::binance::spot_selling_order::{
        Item      as BinanceSpotSellingOrder,
        Interface as BinanceSpotSellingOrderInterface
    };
}

pub mod error {
    use super::client;

    pub use client::RecorderError;
}
