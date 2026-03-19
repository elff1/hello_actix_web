use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{SubscriberEmail, SubscriberName};

pub struct PersistedSubscriber {
    pub id: Uuid,
    pub email: SubscriberEmail,
    pub name: SubscriberName,
    pub subscribed_at: DateTime<Utc>,
    pub status: String,
}

pub struct PersistedSubscriptionTokens {
    pub subscriber_id: Uuid,
    pub token: String,
}
