use std::str::FromStr;

use rust_binance::noun::Decimal;
use rust_binance::spot::client::{SpotClient, SpotClientOption};
use rust_binance::strategy::limit::{Limit, LimitPosition};
use rust_binance::strategy::{Exchanger, Range, Strategy};

use crate::database::collection::{
    BinanceSecret, BinanceSecretInterface, BinanceSpot, BinanceSpotInterface,
    PromiseBinanceSpotLimit, PromiseBinanceSpotLimitInterface,
};

use super::*;

impl Process for PromiseBinanceSpotLimit {
    fn create(self, state: Arc<State>) -> PromiseProcess<()> {
        let item = Arc::new(self);
        let result = move || -> PinFuture<()> {
            let state = state.clone();
            let item = item.clone();
            let process = async move {
                if let Err(e) = process(&state, &item).await {
                    let database = state.database();
                    let promise_id = item.promise.clone();
                    let promise_owner = item.owner.clone();

                    insert_error(database, promise_id, promise_owner, e.to_string())
                        .await
                        .unwrap();

                    if let ProcessError::Person(_) = e {
                        database
                            .promise
                            .update_running_by_identifier_and_owner(
                                &item.promise,
                                &item.owner,
                                &crate::database::collection::PromiseRunning::Stopped,
                            )
                            .await
                            .unwrap();

                        panic!()
                    }
                }
            };

            Box::pin(process)
        };

        Box::new(result)
    }
}

async fn process(state: &State, item: &PromiseBinanceSpotLimit) -> ProcessResult<()> {
    let database = state.database();
    let owner = item.owner.clone();
    let promise_id = item.promise.clone();
    let symbol = item.symbol.clone();

    let secret = select_binance_secret_from_database(database, &owner).await?;
    let spot = select_binance_spot_from_database(database, &owner, &symbol).await?;

    let positions = LimitPosition::new(
        to_decimal(&item.investment)?,
        Range(
            to_decimal(&item.buying_low)?,
            to_decimal(&item.buying_high)?,
        ),
        Range(
            to_decimal(&item.selling_low)?,
            to_decimal(&item.selling_high)?,
        ),
        Some(to_decimal(&item.position)?),
    );

    let client = SpotClient::new(
        secret.api_key,
        secret.secret_key,
        spot.to_spot(),
        Some(SpotClientOption {
            is_production: true,
        }),
    );
    let client = Arc::new(client);
    let price = client.spawn_price();
    let buy = client.spawn_buy();
    let sell = client.spawn_sell();

    let limit = Limit::with_positions(vec![positions]);

    match limit.trap(&price, &buy, &sell).await {
        Err(e) => return Err(ProcessError::Pursue(e.to_string())),
        Ok(_v) => {}
    }

    let position = limit.positions().get(0).unwrap();

    database
        .promise_binance_spot_limit
        .replace_by_promise(&PromiseBinanceSpotLimit::with_limit_position(
            promise_id, owner, symbol, position,
        ))
        .await?;

    Ok(())
}

fn to_decimal(value: &String) -> ProcessResult<Decimal> {
    match Decimal::from_str(&value) {
        Ok(v) => Ok(v),
        Err(e) => Err(ProcessError::Database(e.to_string())),
    }
}

async fn select_binance_secret_from_database(
    database: &Database,
    owner: &PersonIdentifier,
) -> ProcessResult<BinanceSecret> {
    let secret = database
        .binance_secret
        .select_one_spot_by_owner(&owner)
        .await?;
    match secret {
        Some(v) => Ok(v),
        None => return Err(ProcessError::Person(String::from("Secret not found"))),
    }
}

async fn select_binance_spot_from_database(
    database: &Database,
    owner: &PersonIdentifier,
    symbol: &String,
) -> ProcessResult<BinanceSpot> {
    let spot = database
        .binance_spot
        .select_one_by_owner_and_symbol(&owner, &symbol)
        .await?;
    match spot {
        Some(v) => Ok(v),
        None => return Err(ProcessError::Person(String::from("Spot not found"))),
    }
}

impl PromiseBinanceSpotLimit {
    fn with_limit_position(
        promise_identifier: PromiseIdentifier,
        owner: PersonIdentifier,
        symbol: String,
        value: &LimitPosition,
    ) -> Self {
        let position = {
            let position_quantity = value.position.lock().unwrap();
            match *position_quantity {
                Some(v) => v,
                None => Decimal::ZERO,
            }
        };

        Self {
            promise: promise_identifier,
            owner,
            symbol,
            buying_low: value.buying.low().to_string(),
            buying_high: value.buying.high().to_string(),
            selling_low: value.selling.low().to_string(),
            selling_high: value.selling.high().to_string(),
            investment: value.investment.to_string(),
            position: position.to_string(),
        }
    }
}
