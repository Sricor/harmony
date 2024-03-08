use self::database::collection::{
    BinanceSpotBuyingOrder, BinanceSpotBuyingOrderInterface, BinanceSpotSellingOrder,
    BinanceSpotSellingOrderInterface,
};

use super::*;

pub mod get {
    use super::*;

    #[derive(Debug, Serialize)]
    pub struct ResponseBody {
        buying: Vec<BinanceSpotBuyingOrder>,
        selling: Vec<BinanceSpotSellingOrder>,
    }

    #[instrument(skip(state), name = "GET Binance Spot Order")]
    pub async fn request(
        claim: Claim,

        TractState(state): TractState<AppState>,
    ) -> ResponseResult<ResponseBody> {
        let database = state.database();
        let owner = claim.subject();

        let buying_orders = database
            .binance_spot_buying_order
            .select_all_by_owner(&owner)
            .await?;
        let selling_orders = database
            .binance_spot_selling_order
            .select_all_by_owner(&owner)
            .await?;

        let result = ResponseBody {
            buying: buying_orders,
            selling: selling_orders,
        };

        Ok(Response::ok(result))
    }
}
