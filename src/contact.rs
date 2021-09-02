use crate::{DbPool, Env};
use anyhow::Result;
use axum::{extract::Extension, response::IntoResponse, Json};
use lettre::{
    transport::smtp::authentication::Credentials, AsyncSmtpTransport, AsyncTransport, Message,
    Tokio1Executor,
};
use reqwest::StatusCode;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
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

    if std::env::var("LOCAL").is_err() {
        let credentials = Credentials::new(env.smtp_user.clone(), env.smtp_pass.clone());
        let mailer: AsyncSmtpTransport<Tokio1Executor> =
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&env.relay)?
                .credentials(credentials)
                .build();

        match mailer.send(email).await {
            Ok(_) => tracing::info!("Email successfully sent"),
            Err(e) => eprintln!("Could not send email: {}", e),
        }
    }

    Ok(())
}

pub async fn contact_me(
    contact: Json<ContactMe>,
    Extension(db): Extension<DbPool>,
    Extension(env): Extension<Env>,
) -> impl IntoResponse {
    if contact.whoami.to_lowercase() != env.whoami.to_lowercase() {
        return StatusCode::BAD_REQUEST;
    }

    return match sqlx::query!(
        "insert into contact (name, sender, message) values ($1, $2, $3)",
        contact.name,
        contact.from,
        contact.message
    )
    .execute(&db)
    .await
    {
        Ok(_) => {
            if let Err(e) = email_me(&env, contact.0.clone()).await {
                tracing::error!("{}", e);
                return StatusCode::INTERNAL_SERVER_ERROR;
            }

            StatusCode::OK
        }
        Err(e) => {
            tracing::error!("{}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    };
}
