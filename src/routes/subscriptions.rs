use actix_web::{HttpResponse, web};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{NewSubScriber, SubscriberEmail, SubscriberName};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, db_connection_pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    db_connection_pool: web::Data<PgPool>,
) -> HttpResponse {
    let new_subscriber = match (
        SubscriberName::parse(form.0.name),
        SubscriberEmail::parse(form.0.email),
    ) {
        (Ok(name), Ok(email)) => NewSubScriber { name, email },
        (_, _) => return HttpResponse::BadRequest().finish(),
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
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(db_connection_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to excute query: {:?}", e);
        e
    })?;

    Ok(())
}
