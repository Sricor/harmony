use super::*;

pub mod post {
    use crate::service::promise::Process;

    use self::database::{
        collection::{
            Promise, PromiseBinanceSpotLimit, PromiseBinanceSpotLimitInterface, PromiseIdentifier,
            PromiseInterface, PromiseRunning,
        },
        Uniquer,
    };

    use super::*;
    #[derive(Debug, Clone, Deserialize)]
    pub struct RequestPayload {
        interval: u64,
        symbol: String,
        buying_low: Price,
        buying_high: Price,
        selling_low: Price,
        selling_high: Price,
        investment: Amount,
        position: Quantity,
    }

    pub type ResponseBody = PromiseIdentifier;

    #[instrument(skip(state), name = "POST Binance Spot Limit")]
    pub async fn request(
        claim: Claim,

        TractState(state): TractState<AppState>,
        Json(payload): Json<RequestPayload>,
    ) -> ResponseResult<ResponseBody> {
        let owner = claim.subject().clone();
        let database = state.database();

        let mut promise = Promise::with_binance_spot_limit(owner.clone(), payload.interval);
        let process = PromiseBinanceSpotLimit {
            owner,
            symbol: payload.symbol,
            promise: promise.identifier().clone(),
            buying_low: payload.buying_low.to_string(),
            buying_high: payload.buying_high.to_string(),
            selling_low: payload.selling_low.to_string(),
            selling_high: payload.selling_high.to_string(),
            investment: payload.investment.to_string(),
            position: payload.position.to_string(),
        };

        promise.running = PromiseRunning::Running;
        database.promise.insert(&promise).await?;
        database.promise_binance_spot_limit.insert(&process).await?;

        let task = promise.make(process.create(state.clone()));

        match state.delay().insert(task).await {
            Ok(v) => Ok(Response::ok(v)),
            Err(e) => {
                // TODO: fallback database promise
                Err(Response::incompatible(String::from(e.to_string())))
            }
        }
    }
}

pub mod get {
    use self::database::{
        collection::{
            Promise, PromiseBinanceSpotLimit, PromiseBinanceSpotLimitInterface, PromiseCategory,
            PromiseIdentifier, PromiseInterface, PromiseLogging, PromiseLoggingInterface,
        },
        Uniquer,
    };

    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RequestPayload {
        identifier: PromiseIdentifier,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ResponseBody {
        promise: Promise,
        limit: PromiseBinanceSpotLimit,
        logging: Vec<PromiseLogging>,
    }

    #[instrument(skip(state), name = "GET Binance Spot Limit")]
    pub async fn request(
        claim: Claim,

        TractState(state): TractState<AppState>,
    ) -> ResponseResult<Vec<ResponseBody>> {
        let owner = claim.subject().clone();
        let database = state.database();

        let promises = database
            .promise
            .select_all_by_owner_category(&owner, &PromiseCategory::BinanceSpotLimit)
            .await?;

        if promises.is_empty() {
            return Ok(Response::ok(vec![]));
        }

        let mut result = Vec::with_capacity(promises.len());
        for p in promises.into_iter() {
            let promise_item = database
                .promise_binance_spot_limit
                .select_one_by_promise_and_owner(p.identifier(), &owner)
                .await?;
            let promise_item = match promise_item {
                Some(v) => v,
                None => return Err(Response::incompatible(String::from("Something error"))),
            };

            let promise_logging = database
                .promise_logging
                .select_all_by_promise_and_owner(p.identifier(), &owner)
                .await?;

            result.push(ResponseBody {
                promise: p,
                limit: promise_item,
                logging: promise_logging,
            })
        }

        let response = Response::ok(result);

        Ok(response)
    }
}

pub mod delete {
    use self::database::{
        collection::{PromiseIdentifier, PromiseInterface, PromiseRunning},
        Uniquer,
    };

    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RequestPayload {
        identifier: PromiseIdentifier,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ResponseBody();

    #[instrument(skip(state), name = "DELETE Binance Spot Limit")]
    pub async fn request(
        claim: Claim,

        TractState(state): TractState<AppState>,
        Json(payload): Json<RequestPayload>,
    ) -> ResponseResult<ResponseBody> {
        let owner = claim.subject().clone();
        let database = state.database();
        let delay = state.delay();

        let promise = database
            .promise
            .select_one_by_identifier_and_owner(&payload.identifier, &owner)
            .await?;

        let promise = match promise {
            Some(v) => v,
            None => return Err(Response::incompatible(String::from("Promise Not Found"))),
        };

        let promise_identifier = promise.identifier();

        let _ = delay.remove(&promise_identifier).await;
        database
            .promise
            .update_running_by_identifier_and_owner(
                &promise_identifier,
                &owner,
                &PromiseRunning::Stopped,
            )
            .await?;

        let response = Response::ok(ResponseBody());

        Ok(response)
    }
}
