use crate::Env;
use rocket::http::Status;
use rocket::{serde::json::Json, serde::Deserialize, State};
use sqlx::{Pool, Sqlite};

#[derive(Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct ContactMe {
    from: String,
    message: String,
    whoami: String,
}

#[post("/contact", data = "<contact>")]
pub async fn contact_me(
    db: &State<Pool<Sqlite>>,
    env: &State<Env>,
    contact: Json<ContactMe>,
) -> Status {
    if contact.whoami.to_lowercase() != env.whoami.to_lowercase() {
        return Status::BadRequest;
    }

    return match sqlx::query!(
        "insert into contact (sender, message) values (?, ?)",
        contact.from,
        contact.message
    )
    .execute(&**db)
    .await
    {
        Ok(_) => Status::Ok,
        Err(e) => {
            error!("{}", e);
            Status::InternalServerError
        }
    };
}
