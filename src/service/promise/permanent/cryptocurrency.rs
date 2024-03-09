use std::str::FromStr;

use rust_binance::noun::Decimal;
use rust_binance::spot::client::{SpotClient, SpotClientOption};
use rust_binance::spot::Spot;
use rust_binance::strategy::limit::{Limit, LimitPosition};
use rust_binance::strategy::{Exchanger, Range, Strategy};

use crate::database::collection::CyptocurrencyPriceInterface;

use super::*;

pub struct PromiseCryptoCurrency {
    client: SpotClient
}

impl Default for PromiseCryptoCurrency {
    fn default() -> Self {
        let spot = Spot {
            symbol: String::from("BTCUSDT"),
            transaction_quantity_precision: 5,
            quantity_precision: 8,
            amount_precision: 8,
            buying_commission: Decimal::from_str("0.001").unwrap(),
            selling_commission: Decimal::from_str("0.001").unwrap(),
            minimum_transaction_amount: Decimal::from(5)
        };

        Self {
            client: SpotClient::new("".into(), "".into(), spot, None)
        }
    }
}


impl Process for PromiseCryptoCurrency {
    fn create(self, state: Arc<State>) -> PromiseProcess<()> {
        let item = Arc::new(self);
        let result = move || -> PinFuture<()> {
            let state = state.clone();
            let item = item.clone();
            let process = async move {

                // TODO
                // if let Err(e) = process(&state, &item).await {
                    
                // }
            };

            Box::pin(process)
        };

        Box::new(result)
    }
}
