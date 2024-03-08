use super::*;

pub mod post {
    use super::*;

    use self::database::collection::{BinanceSpotBuyingOrder, BinanceSpotBuyingOrderInterface};

    #[derive(Debug, Clone, Deserialize)]
    pub struct RequestPayload {
        symbol: String,
        investment: Amount,
        production: bool,
    }

    type ResponseBody = SpotBuying;

    #[instrument(skip(state), name = "POST Binance Spot Buy")]
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
        let result = client.buy(&price, &payload.investment).await?;

        if payload.production {
            let SpotBuying {
                price,
                quantity,
                spent,
                quantity_after_commission,
                timestamp,
            } = result.clone();
            let order = BinanceSpotBuyingOrder {
                owner: owner.to_string(),
                symbol,
                price: price.to_string(),
                quantity: quantity.to_string(),
                spent: spent.to_string(),
                quantity_after_commission: quantity_after_commission.to_string(),
                timestamp,
                ..Default::default()
            };

            database.binance_spot_buying_order.insert(&order).await?;
        }

        Ok(Response::ok(result))
    }
}
