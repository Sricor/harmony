use self::database::collection::{BinanceSpotSellingOrder, BinanceSpotSellingOrderInterface};

use super::*;

pub mod post {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RequestPayload {
        symbol: String,
        quantity: Quantity,
        production: bool,
    }

    type ResponseBody = SpotSelling;

    #[instrument(skip(state), name = "POST Binance Spot Sell")]
    pub async fn request(
        claim: Claim,

        TractState(state): TractState<AppState>,
        Json(payload): Json<RequestPayload>,
    ) -> ResponseResult<ResponseBody> {
        let database = state.database();
        let owner = claim.subject();
        let spot = select_spot_by_owner_symbol(database, owner, &payload.symbol)
            .await?
            .to_spot();
        let symbol = spot.symbol.clone();

        let client = if payload.production {
            spot_client_production(database, owner.clone(), spot).await?
        } else {
            spot_client_experiment(spot).await?
        };

        let price = client.price().await?;
        let result = client.sell(&price, &payload.quantity).await?;

        if payload.production {
            let SpotSelling {
                price,
                quantity,
                income,
                income_after_commission,
                timestamp,
            } = result.clone();

            let order = BinanceSpotSellingOrder {
                owner: owner.to_string(),
                symbol,
                price: price.to_string(),
                quantity: quantity.to_string(),
                income: income.to_string(),
                income_after_commission: income_after_commission.to_string(),
                timestamp,
                ..Default::default()
            };

            database.binance_spot_selling_order.insert(order).await?;
        }

        Ok(Response::ok(result))
    }
}
