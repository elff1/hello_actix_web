use actix_web::{HttpResponse, web};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::{NewSubScriber, SubscriberEmail, SubscriberName},
    email_client::EmailClient,
};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubScriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        match (
            SubscriberName::parse(value.name),
            SubscriberEmail::parse(value.email),
        ) {
            (Ok(name), Ok(email)) => Ok(NewSubScriber { name, email }),
            (Err(str), _) | (_, Err(str)) => Err(str),
        }
    }
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, db_connection_pool, _email_client),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    db_connection_pool: web::Data<PgPool>,
    _email_client: web::Data<EmailClient>,
) -> HttpResponse {
    let Ok(new_subscriber) = form.0.try_into() else {
        return HttpResponse::BadRequest().finish();
    };

    match insert_subscriber(&db_connection_pool, &new_subscriber).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the DB",
    skip(new_subscriber, db_connection_pool)
)]
pub async fn insert_subscriber(
    db_connection_pool: &PgPool,
    new_subscriber: &NewSubScriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'confirmed')
        "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(db_connection_pool)
    .await
    .map_err(|e| {
        // TODO: handle duplicate email error, which returns HTTP 500 now
        tracing::error!("Failed to excute query: {:?}", e);
        e
    })?;

    Ok(())
}
