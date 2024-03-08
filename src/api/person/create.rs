use super::*;

pub mod post {
    use super::*;

    use self::database::{
        collection::{Person, PersonInterface},
        Uniquer,
    };

    const PERSON_EXISTS: &str = "person already exists";

    #[derive(Debug, Deserialize)]
    pub struct RequestPayload {
        name: String,
        password: String,
    }

    #[derive(Debug, Serialize)]
    pub struct ResponseBody {
        claim: String,
    }

    #[instrument(skip(state), name = "POST Person Create")]
    pub async fn request(
        TractState(state): TractState<AppState>,
        Json(payload): Json<RequestPayload>,
    ) -> ResponseResult<ResponseBody> {
        let database = state.database();

        let item = database.person.select_one_by_name(&payload.name).await?;
        if item.is_some() {
            return Err(Response::fobidden(String::from(PERSON_EXISTS)));
        }

        let person = Person::with_normal(payload.name, payload.password);
        let identifier = person.identifier();

        database.person.insert(&person).await?;

        let response = Response::ok(ResponseBody {
            claim: Claim::approve(state.secret(), identifier.clone())?,
        });

        Ok(response)
    }
}
