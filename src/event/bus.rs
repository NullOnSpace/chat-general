use async_trait::async_trait;

use super::Event;
use crate::error::AppResult;

#[async_trait]
pub trait EventSubscriber: Send + Sync {
    async fn on_event(&self, event: &Event) -> AppResult<()>;
}

pub struct EventBus {
    subscribers: Vec<Box<dyn EventSubscriber>>,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            subscribers: Vec::new(),
        }
    }

    pub fn subscribe(mut self, subscriber: Box<dyn EventSubscriber>) -> Self {
        self.subscribers.push(subscriber);
        self
    }

    pub async fn publish(&self, event: Event) -> AppResult<()> {
        for subscriber in &self.subscribers {
            subscriber.on_event(&event).await?;
        }
        Ok(())
    }
}

pub struct LoggingSubscriber;

#[async_trait]
impl EventSubscriber for LoggingSubscriber {
    async fn on_event(&self, event: &Event) -> AppResult<()> {
        tracing::info!("Event: {:?}", event.event_type());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::UserId;

    #[tokio::test]
    async fn test_event_bus() {
        let bus = EventBus::new()
            .subscribe(Box::new(LoggingSubscriber));
        
        let event = Event::UserOnline {
            user_id: UserId::new(),
            device_id: "test".to_string(),
        };
        
        bus.publish(event).await.unwrap();
    }
}
