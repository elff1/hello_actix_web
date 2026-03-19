use actix_web::{HttpResponse, web};
use chrono::Utc;
use rand::{distr::Alphanumeric, prelude::*};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    configuration::ApplicaionSettings,
    constants::SUBSCRIPTION_TOKEN_LENGTH,
    domain::{
        NewSubScriber, PersistedSubscriber, PersistedSubscriptionTokens, SubscriberEmail,
        SubscriberName,
    },
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
    skip(form, application, db_connection_pool, email_client),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    application: web::Data<ApplicaionSettings>,
    db_connection_pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> HttpResponse {
    let Ok(new_subscriber) = form.0.try_into() else {
        return HttpResponse::BadRequest().finish();
    };

    let Ok(mut transaction) = db_connection_pool.begin().await else {
        return HttpResponse::InternalServerError().finish();
    };

    let Ok(subscriber) = insert_subscriber(&mut transaction, new_subscriber).await else {
        return HttpResponse::InternalServerError().finish();
    };

    let Ok(token) = store_token(&mut transaction, &subscriber).await else {
        return HttpResponse::InternalServerError().finish();
    };

    if transaction.commit().await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    if send_confirmation_email(&email_client, &application, &subscriber, &token)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

// status: pending_confirmation, confirmed
#[tracing::instrument(
    name = "Saving new subscriber details in the DB",
    skip(transaction, new_subscriber)
)]
pub async fn insert_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    new_subscriber: NewSubScriber,
) -> Result<PersistedSubscriber, sqlx::Error> {
    let uuid = Uuid::new_v4();
    let time = Utc::now();
    let status = "pending_confirmation".to_string();

    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        uuid,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        time,
        status
    )
    .execute(&mut **transaction)
    .await
    .map_err(|e| {
        // TODO: handle duplicate email error, which returns HTTP 500 now
        tracing::error!("Failed to excute query: {:?}", e);
        e
    })?;

    Ok(PersistedSubscriber {
        id: uuid,
        email: new_subscriber.email,
        name: new_subscriber.name,
        subscribed_at: time,
        status,
    })
}

#[tracing::instrument(
    name = "Saving subscription token in the DB",
    skip(transaction, subscriber)
)]
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber: &PersistedSubscriber,
) -> Result<PersistedSubscriptionTokens, sqlx::Error> {
    let token = generate_subscription_token();

    sqlx::query!(
        r#"
        INSERT INTO subscription_tokens (subscription_token, subscriber_id)
        VALUES ($1, $2)
        "#,
        token,
        subscriber.id,
    )
    .execute(&mut **transaction)
    .await
    .map_err(|e| {
        tracing::error!("Failed to excute query: {:?}", e);
        e
    })?;

    Ok(PersistedSubscriptionTokens {
        subscriber_id: subscriber.id,
        token,
    })
}

#[tracing::instrument(
    name = "Sending a confirmation eamil to the new subscriber",
    skip(email_client, application, subscriber, token)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    application: &ApplicaionSettings,
    subscriber: &PersistedSubscriber,
    token: &PersistedSubscriptionTokens,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        application.base_url, token.token
    );
    let html_body = format!(
        "<p>Please <a href=\"{}\">click</a> to confirm your subscription.</p>",
        confirmation_link
    );
    let text_body = format!(
        "Please visit {} to confirm your subscription.",
        confirmation_link
    );

    email_client
        .send_email(&subscriber.email, "Welcome!", &html_body, &text_body)
        .await
}

fn generate_subscription_token() -> String {
    let mut rng = rand::rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(SUBSCRIPTION_TOKEN_LENGTH)
        .collect()
}
