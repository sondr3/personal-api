use crate::Env;
use anyhow::Result;
use lettre::{
    transport::smtp::authentication::Credentials, AsyncSmtpTransport, AsyncTransport, Message,
    Tokio1Executor,
};
use rocket::{http::Status, serde::json::Json, serde::Deserialize, State};
use sqlx::{Pool, Postgres};

#[derive(Deserialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct ContactMe {
    name: String,
    from: String,
    message: String,
    whoami: String,
}

async fn email_me(env: &Env, message: ContactMe) -> Result<()> {
    let email = Message::builder()
        .from(format!("{} <{}>", "noreply", env.contact_email).parse()?)
        .reply_to(format!("{} <{}>", message.name, message.from).parse()?)
        .to(format!("Sondre Nilsen <{}>", env.email).parse()?)
        .subject("New contact request")
        .body(message.message)?;

    let credentials = Credentials::new(env.smtp_user.clone(), env.smtp_pass.clone());
    let mailer: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&env.relay)?
            .credentials(credentials)
            .build();

    match mailer.send(email).await {
        Ok(_) => info!("Email successfully sent"),
        Err(e) => eprintln!("Could not send email: {}", e),
    }

    Ok(())
}

#[post("/contact", data = "<contact>")]
pub async fn contact_me(
    db: &State<Pool<Postgres>>,
    env: &State<Env>,
    contact: Json<ContactMe>,
) -> Status {
    if contact.whoami.to_lowercase() != env.whoami.to_lowercase() {
        return Status::BadRequest;
    }

    return match sqlx::query!(
        "insert into contact (name, sender, message) values ($1, $2, $3)",
        contact.name,
        contact.from,
        contact.message
    )
    .execute(&**db)
    .await
    {
        Ok(_) => {
            if let Err(e) = email_me(env, contact.clone()).await {
                error!("{}", e);
                return Status::InternalServerError;
            }

            Status::Ok
        }
        Err(e) => {
            error!("{}", e);
            Status::InternalServerError
        }
    };
}
