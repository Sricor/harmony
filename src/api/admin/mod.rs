use self::database::{
    collection::{Person, PersonIdentifier, PersonInterface},
    Database,
};

use super::*;

pub mod person;

async fn require_administrator_role(
    database: &Database,
    identifier: PersonIdentifier,
) -> Result<Person, Response<()>> {
    let person = database
        .person
        .select_one_by_identifier(&identifier)
        .await?;

    match person {
        Some(person) => {
            if person.role as u8 == 2 {
                return Ok(person);
            }

            Err(response_access_denied())
        }

        None => Err(response_access_denied()),
    }
}

fn response_access_denied() -> Response<()> {
    Response::fobidden(String::from("Access denied"))
}
