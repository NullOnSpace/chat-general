use async_trait::async_trait;

use crate::domain::Message;
use crate::session::Session;
use crate::error::AppResult;

#[derive(Debug, Clone)]
pub enum HandlerAction {
    Continue,
    Modify(Message),
    Reject(String),
    Respond(Message),
}

#[async_trait]
pub trait MessageHandler: Send + Sync {
    async fn on_message(
        &self,
        message: &Message,
        session: &Session,
    ) -> AppResult<HandlerAction>;
}

pub struct HandlerChain {
    handlers: Vec<Box<dyn MessageHandler>>,
}

impl HandlerChain {
    pub fn new() -> Self {
        Self { handlers: Vec::new() }
    }

    pub fn add(mut self, handler: Box<dyn MessageHandler>) -> Self {
        self.handlers.push(handler);
        self
    }

    pub async fn process(&self, mut message: Message, session: &Session) -> AppResult<Message> {
        for handler in &self.handlers {
            match handler.on_message(&message, session).await? {
                HandlerAction::Continue => continue,
                HandlerAction::Modify(modified) => message = modified,
                HandlerAction::Reject(reason) => {
                    return Err(crate::error::AppError::Validation(reason));
                }
                HandlerAction::Respond(response) => {
                    return Ok(response);
                }
            }
        }
        Ok(message)
    }
}

impl Default for HandlerChain {
    fn default() -> Self {
        Self::new()
    }
}

pub struct LoggingHandler;

#[async_trait]
impl MessageHandler for LoggingHandler {
    async fn on_message(
        &self,
        message: &Message,
        _session: &Session,
    ) -> AppResult<HandlerAction> {
        tracing::info!(
            "Message from {} to conversation {}: {}",
            message.sender_id,
            message.conversation_id,
            message.content.chars().take(50).collect::<String>()
        );
        Ok(HandlerAction::Continue)
    }
}

pub struct ContentFilterHandler {
    blocked_words: Vec<String>,
}

impl ContentFilterHandler {
    pub fn new(blocked_words: Vec<String>) -> Self {
        Self { blocked_words }
    }
}

#[async_trait]
impl MessageHandler for ContentFilterHandler {
    async fn on_message(
        &self,
        message: &Message,
        _session: &Session,
    ) -> AppResult<HandlerAction> {
        let content_lower = message.content.to_lowercase();
        for word in &self.blocked_words {
            if content_lower.contains(&word.to_lowercase()) {
                return Ok(HandlerAction::Reject(format!(
                    "Message contains blocked word: {}",
                    word
                )));
            }
        }
        Ok(HandlerAction::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ConversationId, UserId};

    #[tokio::test]
    async fn test_handler_chain() {
        let chain = HandlerChain::new()
            .add(Box::new(LoggingHandler));
        
        let conv_id = ConversationId::new();
        let user_id = UserId::new();
        let device_id = crate::domain::DeviceId::new();
        let session = Session::new(user_id, device_id);
        let message = Message::text(conv_id, user_id, "Test message".to_string());
        
        let result = chain.process(message, &session).await.unwrap();
        assert_eq!(result.content, "Test message");
    }

    #[tokio::test]
    async fn test_content_filter() {
        let filter = ContentFilterHandler::new(vec!["spam".to_string()]);
        
        let conv_id = ConversationId::new();
        let user_id = UserId::new();
        let device_id = crate::domain::DeviceId::new();
        let session = Session::new(user_id, device_id);
        
        let good_message = Message::text(conv_id, user_id, "Hello world".to_string());
        let result = filter.on_message(&good_message, &session).await.unwrap();
        assert!(matches!(result, HandlerAction::Continue));
        
        let bad_message = Message::text(conv_id, user_id, "This is spam!".to_string());
        let result = filter.on_message(&bad_message, &session).await.unwrap();
        assert!(matches!(result, HandlerAction::Reject(_)));
    }
}
