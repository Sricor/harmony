use super::*;

use self::database::collection::{Person, PersonInterface};

#[instrument(skip(state), name = "POST Admin Person")]
pub async fn post(
    claim: Claim,

    TractState(state): TractState<AppState>,
) -> ResponseResult<Vec<Person>> {
    require_administrator_role(state.database(), claim.subject().clone()).await?;

    let person_lits = state.database().person.select_all().await?;

    Ok(Response::ok(person_lits))
}
