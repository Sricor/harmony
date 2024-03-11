use rust_binance::{
    spot::client::{SpotClient, SpotClientOption},
    strategy::{Exchanger, Strategy},
};

use crate::database::{
    collection::{
        BinanceSecret, BinanceSpot, PersonIdentifier, PromiseLogging, PromiseLoggingInterface,
        PromiseProcessBinanceSpotLimit,
    },
    Database,
};

use super::*;

impl Process for PromiseProcessBinanceSpotLimit {
    fn create(
        self,
        state: Arc<State>,
        owner: PersonIdentifier,
        promise: PromiseIdentifier,
    ) -> ClosureFuture<()> {
        let item = Arc::new(self);
        let result = move || -> PinFuture<()> {
            let state = state.clone();
            let item = item.clone();
            let promise = promise.clone();
            let owner = owner.clone();
            let process = async move {
                let database = state.database();
                let result = process(&state, &owner, &promise, &item).await;

                database
                    .promise
                    .update_one_total_runs_by_identifier(&promise)
                    .await
                    .unwrap();

                database
                    .promise
                    .update_process_by_identifier_category(
                        &Promise::serde_process(&*item).unwrap(),
                        &promise,
                        &PromiseProcessCategory::BinanceSpotLimit,
                    )
                    .await
                    .unwrap();

                if let Err(e) = result {
                    if let SchedulingError::Person(_) = &e {
                        state.delay().remove(&promise).await.unwrap();
                        database
                            .promise
                            .update_status_by_identifier(&PromiseProcessStatus::Stopped, &promise)
                            .await
                            .unwrap();
                    }

                    let logging = PromiseLogging::with_error(promise, owner.clone(), e.to_string());

                    database.promise_logging.insert(&logging).await.unwrap();
                }
            };

            Box::pin(process)
        };

        Box::new(result)
    }
}

async fn process(
    state: &State,
    owner: &PersonIdentifier,
    _promise: &PromiseIdentifier,
    item: &PromiseProcessBinanceSpotLimit,
) -> SchedulingResult<()> {
    let database = state.database();
    let symbol = &item.symbol;

    let secret = select_binance_secret_from_database(database, &owner).await?;
    let spot = select_binance_spot_from_database(database, &owner, symbol).await?;

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
    let sell = client.spawn_sell();
    let buy = client.spawn_buy();

    match item.limit.trap(&price, &buy, &sell).await {
        Err(e) => return Err(SchedulingError::Pursue(e.to_string())),
        Ok(_) => {}
    }

    Ok(())
}

async fn select_binance_secret_from_database(
    database: &Database,
    owner: &PersonIdentifier,
) -> SchedulingResult<BinanceSecret> {
    use crate::database::collection::BinanceSecretInterface;

    let secret = database
        .binance_secret
        .select_one_spot_by_owner(&owner)
        .await?;

    match secret {
        Some(v) => Ok(v),
        None => {
            return Err(SchedulingError::Person(String::from(
                "binance secret not found",
            )))
        }
    }
}

async fn select_binance_spot_from_database(
    database: &Database,
    owner: &PersonIdentifier,
    symbol: &String,
) -> SchedulingResult<BinanceSpot> {
    use crate::database::collection::BinanceSpotInterface;
    let spot = database
        .binance_spot
        .select_one_by_owner_and_symbol(&owner, &symbol)
        .await?;

    match spot {
        Some(v) => Ok(v),
        None => {
            return Err(SchedulingError::Person(String::from(
                "binance spot ot found",
            )))
        }
    }
}
