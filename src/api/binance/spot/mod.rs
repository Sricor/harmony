pub mod buy;
pub mod limit;
pub mod order;
pub mod predict;
pub mod sell;

use self::database::{
    collection::{
        BinanceSecretInterface, BinanceSecretPurview, BinanceSpot, BinanceSpotInterface,
        PersonIdentifier,
    },
    Database,
};

use super::*;

async fn spot_client_experiment(spot: Spot) -> ResponseContextResult<SpotClient> {
    let result = SpotClient::new(String::from("NULL"), String::from("NULL"), spot, None);

    Ok(result)
}

async fn spot_client_production(
    database: &Database,
    owner: PersonIdentifier,
    spot: Spot,
) -> ResponseContextResult<SpotClient> {
    let secret = database
        .binance_secret
        .select_one_by_owner_and_purview(&owner, &BinanceSecretPurview::Spot)
        .await?
        .ok_or(Response::fobidden(String::from(
            "Corresponding purview secret not matched",
        )))?;

    let result = SpotClient::new(
        secret.api_key,
        secret.secret_key,
        spot,
        Some(SpotClientOption {
            is_production: true,
        }),
    );

    Ok(result)
}

impl<T> From<SpotClientError> for Response<T> {
    fn from(err: SpotClientError) -> Self {
        Self::incompatible(err.to_string())
    }
}

impl BinanceSpot {
    pub fn to_spot(self) -> Spot {
        use std::str::FromStr;

        Spot {
            symbol: self.symbol,
            transaction_quantity_precision: self.transaction_quantity_precision,
            quantity_precision: self.quantity_precision,
            amount_precision: self.amount_precision,
            buying_commission: Decimal::from_str(&self.buying_commission).unwrap(),
            selling_commission: Decimal::from_str(&self.selling_commission).unwrap(),
            minimum_transaction_amount: Decimal::from_str(&self.minimum_transaction_amount)
                .unwrap(),
        }
    }
}

async fn select_spot_by_owner_symbol(
    database: &Database,
    owner: &String,
    symbol: &String,
) -> ResponseContextResult<BinanceSpot> {
    let spot = database
        .binance_spot
        .select_one_by_owner_and_symbol(owner, symbol)
        .await?;

    match spot {
        Some(v) => Ok(v),
        None => return Err(Response::incompatible(format!("{} Not Found", symbol))),
    }
}

pub mod get {
    use super::*;

    #[instrument(skip(state), name = "GET Binance Spot")]
    pub async fn request(
        claim: Claim,

        TractState(state): TractState<AppState>,
    ) -> ResponseResult<Vec<BinanceSpot>> {
        let database = state.database();
        let owner = claim.subject();

        let result = database.binance_spot.select_all_by_owner(owner).await?;

        Ok(Response::ok(result))
    }
}

pub mod post {
    use super::*;

    #[instrument(skip(state), name = "POST Binance Spot")]
    pub async fn request(
        claim: Claim,

        TractState(state): TractState<AppState>,
        Json(payload): Json<Spot>,
    ) -> ResponseResult<()> {
        let database = state.database();
        let owner = claim.subject();

        let item = BinanceSpot {
            owner: owner.clone(),
            symbol: payload.symbol,
            transaction_quantity_precision: payload.transaction_quantity_precision,
            quantity_precision: payload.quantity_precision,
            amount_precision: payload.amount_precision,
            buying_commission: payload.buying_commission.to_string(),
            selling_commission: payload.selling_commission.to_string(),
            minimum_transaction_amount: payload.minimum_transaction_amount.to_string(),
        };

        database.binance_spot.insert(&item).await?;

        Ok(Response::ok(()))
    }
}
