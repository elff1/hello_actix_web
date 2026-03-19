use actix_web::{HttpResponse, web};
use sqlx::PgPool;

use crate::domain::PersistedSubscriptionTokens;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(
    name = "Confirm a pending subscriber",
    skip(parameters, db_connection_pool)
    fields(token = %parameters.subscription_token)
)]
pub async fn confirm(
    parameters: web::Query<Parameters>,
    db_connection_pool: web::Data<PgPool>,
) -> HttpResponse {
    let token = parameters.0.subscription_token;

    let persisted_token = match query_token(&db_connection_pool, token).await {
        Ok(Some(t)) => t,
        Ok(None) => return HttpResponse::NotFound().finish(),
        _ => return HttpResponse::InternalServerError().finish(),
    };

    match update_subscriber(&db_connection_pool, &persisted_token).await {
        Ok(1) => HttpResponse::Ok().finish(),
        _ => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Finding subscription token in the DB",
    skip(db_connection_pool, token)
)]
pub async fn query_token(
    db_connection_pool: &PgPool,
    token: String,
) -> Result<Option<PersistedSubscriptionTokens>, sqlx::Error> {
    Ok(sqlx::query!(
        r#"
            SELECT subscriber_id
            FROM subscription_tokens
            WHERE subscription_token = $1
            "#,
        token,
    )
    .fetch_optional(db_connection_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to excute query: {:?}", e);
        e
    })?
    .map(|record| PersistedSubscriptionTokens {
        subscriber_id: record.subscriber_id,
        token,
    }))
}

#[tracing::instrument(
    name = "Updating subscriber status in the DB",
    skip(db_connection_pool, persisted_token)
)]
pub async fn update_subscriber(
    db_connection_pool: &PgPool,
    persisted_token: &PersistedSubscriptionTokens,
) -> Result<u64, sqlx::Error> {
    Ok(sqlx::query!(
        r#"
        UPDATE subscriptions
        SET status = 'confirmed'
        WHERE id = $1
        "#,
        persisted_token.subscriber_id
    )
    .execute(db_connection_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to excute query: {:?}", e);
        e
    })?
    .rows_affected())
}
