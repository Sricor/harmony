use super::*;

pub mod post {
    use rust_binance::strategy::{
        limit::{Limit, LimitPosition},
        Range,
    };

    use crate::service::promise::Scheduling;

    use super::*;

    use self::database::collection::{
        Promise, PromiseIdentifier, PromiseInterface, PromiseProcessBinanceSpotLimit,
        PromiseProcessStatus,
    };

    #[derive(Debug, Clone, Deserialize)]
    pub struct Position {
        buying_low: Price,
        buying_high: Price,
        selling_low: Price,
        selling_high: Price,
        investment: Amount,
        position: Quantity,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct RequestPayload {
        symbol: String,
        positions: Vec<Position>,
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

        let mut positions = Vec::with_capacity(payload.positions.len());
        for p in payload.positions.into_iter() {
            let item = LimitPosition::new(
                p.investment,
                Range(p.buying_low, p.buying_high),
                Range(p.selling_low, p.selling_high),
                Some(p.position),
            );

            positions.push(item)
        }

        let mut promise = Promise::with_process_binance_spot_limit(
            owner,
            &PromiseProcessBinanceSpotLimit {
                symbol: payload.symbol,
                limit: Limit::with_positions(positions),
            },
        )?;
        promise.status = PromiseProcessStatus::Running;

        database.promise.insert(&promise).await?;

        let task = promise
            .make::<PromiseProcessBinanceSpotLimit>(state.clone())
            .unwrap();

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
            Promise, PromiseIdentifier, PromiseInterface, PromiseLogging, PromiseLoggingInterface,
            PromiseProcessCategory,
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
            .select_all_by_owner_category(&owner, &PromiseProcessCategory::BinanceSpotLimit)
            .await?;

        if promises.is_empty() {
            return Ok(Response::ok(vec![]));
        }

        let mut result = Vec::with_capacity(promises.len());
        for promise in promises.into_iter() {
            let promise_logging = database
                .promise_logging
                .select_all_by_promise_and_owner(promise.identifier(), &owner)
                .await?;

            result.push(ResponseBody {
                promise,
                logging: promise_logging,
            })
        }

        let response = Response::ok(result);

        Ok(response)
    }
}

pub mod delete {
    use self::database::{
        collection::{
            PromiseIdentifier, PromiseInterface, PromiseProcessCategory, PromiseProcessStatus,
        },
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
            .select_one_by_identifier_owner_category(
                &payload.identifier,
                &owner,
                &PromiseProcessCategory::BinanceSpotLimit,
            )
            .await?;

        let promise = match promise {
            Some(v) => v,
            None => return Err(Response::incompatible(String::from("Promise Not Found"))),
        };

        let promise_identifier = promise.identifier();
        let _ = delay.remove(&promise_identifier).await;

        database
            .promise
            .update_status_by_identifier(&PromiseProcessStatus::Stopped, &promise_identifier)
            .await?;

        let response = Response::ok(ResponseBody());

        Ok(response)
    }
}
