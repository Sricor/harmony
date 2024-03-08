use super::*;

pub mod post {
    use super::*;

    use self::database::collection::PersonInterface;

    const INCORRECT_INFO: &str = "incorrect name or password";

    #[derive(Debug, Deserialize)]
    pub struct RequestPayload {
        name: Option<String>,
        password: Option<String>,
    }

    #[derive(Debug, Serialize)]
    pub struct ResponseBody {
        claim: String,
    }

    #[instrument(skip(state), name = "POST Person Verify")]
    pub async fn request(
        claim: Option<Claim>,

        TractState(state): TractState<AppState>,
        Json(payload): Json<RequestPayload>,
    ) -> ResponseResult<ResponseBody> {
        // Claim expiration extension
        if let Some(claim) = claim {
            let response = Response::ok(ResponseBody {
                claim: Claim::approve(state.secret(), claim.subject().clone())?,
            });

            return Ok(response);
        }

        // Reissue
        let name = require_request_payload(payload.name)?;
        let password = require_request_payload(payload.password)?;
        let identifier = state
            .database()
            .person
            .select_identifier_by_name_and_password(&name, &password)
            .await?;

        match identifier {
            Some(identifier) => {
                let response = Response::ok(ResponseBody {
                    claim: Claim::approve(state.secret(), identifier)?,
                });

                Ok(response)
            }

            None => {
                let response = Response::fobidden(String::from(INCORRECT_INFO));

                Err(response)
            }
        }
    }
}
