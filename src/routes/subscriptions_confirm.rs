use actix_web::{HttpResponse, http::StatusCode, web};
use anyhow::Context;
use sqlx::PgPool;

use crate::{domain::PersistedSubscriptionTokens, helper::*};

#[derive(thiserror::Error)]
pub enum ConfirmError {
    #[error("invalid token")]
    InvalidToken,
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for ConfirmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl actix_web::ResponseError for ConfirmError {
    fn status_code(&self) -> StatusCode {
        match self {
            ConfirmError::InvalidToken => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

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
) -> Result<HttpResponse, ConfirmError> {
    let token = parameters.0.subscription_token;

    let persisted_token = query_token(&db_connection_pool, token)
        .await
        .context("failed to query token in the DB")?
        .ok_or(ConfirmError::InvalidToken)?;

    match update_subscriber(&db_connection_pool, &persisted_token)
        .await
        .context("failed to update subscriber status in the DB")?
    {
        1 => (),
        n => tracing::warn!("[{n}] rows affected, not ONE"),
    }

    Ok(HttpResponse::Ok().finish())
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
