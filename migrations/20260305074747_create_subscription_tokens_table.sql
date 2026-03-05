CREATE TABLE subscription_tokens (
    subscription_token TEXT NOT NULL,
    PRIMARY KEY (subscription_token),
    subscriber_id uuid NOT NULL REFERENCES subscriptions (id) ON DELETE CASCADE
);
