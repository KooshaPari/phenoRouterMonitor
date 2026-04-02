//! Analytics traits

use crate::event::AnalyticsEvent;

pub trait Trackable {
    fn event_type(&self) -> &str;
    fn to_event(&self) -> AnalyticsEvent;
}

impl<T: Trackable> From<T> for AnalyticsEvent {
    fn from(value: T) -> Self {
        value.to_event()
    }
}
