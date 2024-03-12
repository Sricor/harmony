use rust_binance::{
    noun::{Amount, Price, Quantity},
    spot::client::{SpotClient, SpotClientOption},
    strategy::{AmountPoint, Exchanger, PricePoint, QuantityPoint, Strategy},
};

use crate::database::{
    collection::{BinanceSecret, BinanceSpot, PersonIdentifier, PromiseProcessBinanceSpotLimit},
    Database,
};

use super::*;

impl Process for PromiseProcessBinanceSpotLimit {
    fn create(
        self,
        state: Arc<State>,
        logger: Sender<(PromiseLoggingLevel, String)>,
        owner: PersonIdentifier,
        promise: PromiseIdentifier,
    ) -> ClosureFuture<()> {
        let item = Arc::new(self);

        let result = move || -> PinFuture<()> {
            let state = state.clone();
            let item = item.clone();
            let promise = promise.clone();
            let owner = owner.clone();
            let logger = logger.clone();
            let process = async move {
                let database = state.database();
                let result = process(&state, &logger, &owner, &promise, &item).await;

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
    logger: &Sender<(PromiseLoggingLevel, String)>,
    owner: &PersonIdentifier,
    _promise: &PromiseIdentifier,
    item: &PromiseProcessBinanceSpotLimit,
) -> SchedulingResult<()> {
    let database = state.database();
    let symbol = &item.symbol;


    let secret = select_binance_secret_from_database(database, logger, &owner).await?;
    let spot = select_binance_spot_from_database(database, logger, &owner, symbol).await?;

    let client = SpotClient::new(
        secret.api_key,
        secret.secret_key,
        spot.to_spot(),
        Some(SpotClientOption {
            is_production: true,
        }),
    );
    let client = Arc::new(client);
    let buy = point_expand_spawn_buy(client.spawn_buy(), logger.clone());
    let sell = point_expand_spawn_sell(client.spawn_sell(), logger.clone());
    let price = point_expand_spawn_price(client.spawn_price(), logger.clone());

    match item.limit.trap(&price, &buy, &sell).await {
        Err(e) => return Err(SchedulingError::Pursue(e.to_string())),
        Ok(_) => {}
    }

    Ok(())
}

fn point_expand_spawn_price<T>(
    source: T,
    logger: Sender<(PromiseLoggingLevel, String)>,
) -> impl Fn() -> PinFuture<Result<PricePoint, Box<dyn Error + Send + Sync>>>
where
    T: Fn() -> PinFuture<Result<PricePoint, Box<dyn Error + Send + Sync>>>,
{
    move || {
        let logger = logger.clone();
        let source_future = source();
        let f = async move {
            let result = source_future.await;

            if let Err(e) = &result {
                let logging = (
                    PromiseLoggingLevel::Error,
                    format!("price error {}", e.to_string()),
                );

                logger
                    .send_timeout(logging, Duration::from_secs(5))
                    .await
                    .unwrap();
            };

            result
        };

        Box::pin(f)
    }
}


fn point_expand_spawn_buy<T>(
    source: T,
    logger: Sender<(PromiseLoggingLevel, String)>,
) -> impl Fn(Price, Amount) -> PinFuture<Result<QuantityPoint, Box<dyn Error + Send + Sync>>>
where
    T: Fn(Price, Amount) -> PinFuture<Result<QuantityPoint, Box<dyn Error + Send + Sync>>>,
{
    move |price: Price, amount: Amount| {
        let logger = logger.clone();
        let source_future = source(price, amount);
        let f = async move {
            let result = source_future.await;

            let logging = match &result {
                Ok(v) => (PromiseLoggingLevel::Info, format!("buying {:?}", v)),
                Err(e) => (
                    PromiseLoggingLevel::Error,
                    format!(
                        "buying price: {} amount: {} error: {}",
                        price,
                        amount,
                        e.to_string()
                    ),
                ),
            };

            logger
                .send_timeout(logging, Duration::from_secs(5))
                .await
                .unwrap();

            result
        };

        Box::pin(f)
    }
}

fn point_expand_spawn_sell<T>(
    source: T,
    logger: Sender<(PromiseLoggingLevel, String)>,
) -> impl Fn(Price, Quantity) -> PinFuture<Result<AmountPoint, Box<dyn Error + Send + Sync>>>
where
    T: Fn(Price, Quantity) -> PinFuture<Result<AmountPoint, Box<dyn Error + Send + Sync>>>,
{
    move |price: Price, quantity: Quantity| {
        let logger = logger.clone();
        let source_future = source(price, quantity);
        let f = async move {
            let result = source_future.await;

            let logging = match &result {
                Ok(v) => (PromiseLoggingLevel::Info, format!("selling {:?}", v)),
                Err(e) => (
                    PromiseLoggingLevel::Error,
                    format!(
                        "selling price: {} quantity: {} error: {}",
                        price,
                        quantity,
                        e.to_string()
                    ),
                ),
            };

            logger
                .send_timeout(logging, Duration::from_secs(5))
                .await
                .unwrap();

            result
        };

        Box::pin(f)
    }
}

async fn select_binance_secret_from_database(
    database: &Database,
    logger: &Sender<(PromiseLoggingLevel, String)>,
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
            let message = String::from("binance secret not found");
            logger
                .send((PromiseLoggingLevel::Error, message.clone()))
                .await?;

            return Err(SchedulingError::Person(message));
        }
    }
}

async fn select_binance_spot_from_database(
    database: &Database,
    logger: &Sender<(PromiseLoggingLevel, String)>,
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
            let message = String::from("binance spot not found");
            logger
                .send((PromiseLoggingLevel::Error, message.clone()))
                .await?;

            return Err(SchedulingError::Person(message));
        }
    }
}
