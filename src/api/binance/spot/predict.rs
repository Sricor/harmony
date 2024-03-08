use super::*;

pub mod post {
    use super::*;

    #[derive(Debug, Deserialize)]
    pub struct SpotTrading {
        investment: Amount,
        buying_price: Price,
        selling_price: Price,
    }

    #[derive(Debug, Deserialize)]
    pub struct RequestPayload {
        symbol: String,
        trading: SpotTrading,
    }

    pub type ResponseBody = Vec<SpotTransaction>;

    #[instrument(skip(state), name = "POST Binance Spot")]
    pub async fn request(
        claim: Claim,

        TractState(state): TractState<AppState>,
        Json(payload): Json<RequestPayload>,
    ) -> ResponseResult<ResponseBody> {
        let database = state.database();
        let owner = claim.subject();

        let binance_spot = database
            .binance_spot
            .select_one_by_owner_and_symbol(owner, &payload.symbol)
            .await?;
        let binance_spot = match binance_spot {
            Some(v) => v,
            None => {
                return Err(Response::incompatible(String::from(
                    "You have not provided a spot yet",
                )))
            }
        };
        let buying_amount = payload.trading.investment;
        let buying_price = payload.trading.buying_price;
        let selling_price = payload.trading.selling_price;

        let client = spot_client_experiment(binance_spot.to_spot()).await?;
        let spot_buying = client.buy(&buying_price, &buying_amount).await;
        let spot_buying = match spot_buying {
            Ok(v) => v,
            Err(e) => {
                let response = Response::incompatible(e.to_string());
                return Err(response);
            }
        };

        let spot_selling = client
            .sell(&selling_price, &spot_buying.quantity_after_commission)
            .await;
        let spot_selling = match spot_selling {
            Ok(v) => v,
            Err(e) => {
                let response = Response::incompatible(e.to_string());
                return Err(response);
            }
        };

        let transaction = SpotTransaction::new(spot_buying, spot_selling);
        let response = Response::ok(vec![transaction]);

        Ok(response)
    }
}
