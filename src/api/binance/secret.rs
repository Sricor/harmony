use self::database::collection::{BinanceSecret, BinanceSecretInterface, BinanceSecretPurview};

use super::*;

pub mod get {
    use super::*;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Secret {
        purview: u8,
        api_key: String,
        secret_key: String,
    }
    #[instrument(skip(state), name = "GET Binance Secret")]
    pub async fn request(
        claim: Claim,

        TractState(state): TractState<AppState>,
    ) -> ResponseResult<Vec<Secret>> {
        let database = state.database();
        let items = database
            .binance_secret
            .select_all_by_owner(&claim.subject())
            .await?;
        let items = items
            .into_iter()
            .map(|e| Secret {
                purview: e.purview as u8,
                api_key: e.api_key,
                secret_key: e.secret_key,
            })
            .collect();
        let response = Response::ok(items);

        Ok(response)
    }
}

pub mod post {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Secret {
        purview: u8,
        api_key: String,
        secret_key: String,
    }

    #[instrument(skip(state), name = "POST Binance Secret")]
    pub async fn request(
        claim: Claim,

        TractState(state): TractState<AppState>,
        Json(payload): Json<Secret>,
    ) -> ResponseResult<()> {
        let database = state.database();
        let owner = claim.subject();

        let insert_item = match payload.purview {
            1 => {
                if database
                    .binance_secret
                    .select_one_by_owner_and_purview(owner, &BinanceSecretPurview::Read)
                    .await?
                    .is_none()
                {
                    Some(BinanceSecret::with_read(
                        owner.clone(),
                        payload.api_key,
                        payload.secret_key,
                    ))
                } else {
                    None
                }
            }
            2 => {
                if database
                    .binance_secret
                    .select_one_by_owner_and_purview(owner, &BinanceSecretPurview::Spot)
                    .await?
                    .is_none()
                {
                    Some(BinanceSecret::with_spot(
                        owner.clone(),
                        payload.api_key,
                        payload.secret_key,
                    ))
                } else {
                    None
                }
            }
            _ => {
                return Err(Response::incompatible(String::from(
                    "Purview parameter error",
                )))
            }
        };

        let response = if let Some(item) = insert_item {
            database.binance_secret.insert(&item).await?;
            Response::ok(())
        } else {
            Response::incompatible(String::from(
                "The corresponding purview secret already exists",
            ))
        };

        Ok(response)
    }
}
