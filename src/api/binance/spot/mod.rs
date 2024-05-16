pub mod order;

pub mod price {
    pub mod get_price {
        pub const PATH: &str = "/binance/spot/price";

        use binance::types::{Symbol, SymbolPrice};
        use serde::{Deserialize, Serialize};

        use crate::api::http::request::Query;
        use crate::api::http::response::{Response, ResponseResult};
        use crate::api::http::trip::Trip;
        use crate::services::binance::client;

        #[derive(Debug, Serialize, Deserialize)]
        pub struct Params {
            symbol: Option<Symbol>,
        }

        type Reply = Vec<SymbolPrice>;

        #[tracing::instrument(skip(_c))]
        pub async fn handler(_c: Trip, Query(q): Query<Params>) -> ResponseResult<Reply> {
            let client = client()?;

            let result = match q.symbol {
                Some(v) => {
                    vec![client.price(&v).await?]
                }
                None => client.prices(None).await?,
            };

            Ok(Response::ok(result))
        }
    }
}

pub mod asset {
    pub mod post_asset {
        pub const PATH: &str = "/binance/spot/asset";

        use binance::types::{Asset, UserAsset};
        use serde::{Deserialize, Serialize};

        use crate::api::http::request::Json;
        use crate::api::http::response::{Response, ResponseResult};
        use crate::api::http::trip::Trip;
        use crate::services::binance::client_with_sign;

        #[derive(Debug, Serialize, Deserialize)]
        pub struct Payload {
            api_key: String,
            secret_key: String,
            asset: Option<Asset>,
        }

        pub type Reply = Vec<UserAsset>;

        #[tracing::instrument(skip(_c))]
        pub async fn handler(_c: Trip, Json(p): Json<Payload>) -> ResponseResult<Reply> {
            let client = client_with_sign(p.api_key, p.secret_key)?;

            let asset = match &p.asset {
                Some(v) => Some(v),
                None => None,
            };

            let result = client.user_asset(asset, Some(false), None).await?;

            Ok(Response::ok(result))
        }
    }
}
