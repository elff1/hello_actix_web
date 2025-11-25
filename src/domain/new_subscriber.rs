use crate::domain::{SubscriberEmail, SubscriberName};

pub struct NewSubScriber {
    pub name: SubscriberName,
    pub email: SubscriberEmail,
}
