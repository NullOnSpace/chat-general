use axum::{extract::State, Json};
use garde::Validate;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::api::dto::*;
use crate::api::extractor::CurrentUser;
use crate::api::AppState;
use crate::domain::{Conversation, ConversationId, Message, UserId};
use crate::error::{AppError, AppResult};
use crate::message::MessageStore;

#[derive(Debug, Deserialize)]
pub struct GetMessagesQuery {
    pub limit: Option<i64>,
    pub before: Option<String>,
}

pub async fn get_conversations(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> AppResult<Json<serde_json::Value>> {
    let conversations = state
        .message_store
        .get_user_conversations(&current_user.user_id)
        .await?;

    let response: Vec<ConversationResponse> = conversations
        .into_iter()
        .map(ConversationResponse::from)
        .collect();

    Ok(Json(json!({
        "conversations": response
    })))
}

pub async fn create_conversation(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(req): Json<CreateConversationRequest>,
) -> AppResult<Json<ConversationResponse>> {
    if req.participant_ids.is_empty() {
        return Err(AppError::Validation(
            "At least one participant is required".into(),
        ));
    }

    let participants: Vec<UserId> = req
        .participant_ids
        .iter()
        .filter_map(|id| Uuid::parse_str(id).ok())
        .map(UserId::from)
        .collect();

    if participants.is_empty() {
        return Err(AppError::Validation("Invalid participant IDs".into()));
    }

    if participants.len() == 1 {
        let friend_id = participants[0];
        let is_friend = state
            .friend_service
            .is_friend(&current_user.user_id, &friend_id)
            .await?;
        if !is_friend {
            return Err(AppError::Validation(
                "Can only create direct conversation with friends".into(),
            ));
        }

        let conv = Conversation::new_direct(current_user.user_id, friend_id);
        let all_participants = vec![current_user.user_id, friend_id];
        state
            .message_store
            .add_conversation_participants(conv.id, all_participants)
            .await;
        state.message_store.save_conversation(conv.clone()).await?;
        return Ok(Json(ConversationResponse::from(conv)));
    }

    let mut all_participants = participants.clone();
    all_participants.push(current_user.user_id);
    let conv = Conversation::new_group(participants);
    state
        .message_store
        .add_conversation_participants(conv.id, all_participants)
        .await;
    state.message_store.save_conversation(conv.clone()).await?;
    Ok(Json(ConversationResponse::from(conv)))
}

pub async fn get_conversation(
    _current_user: CurrentUser,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(json!({
        "id": id,
        "conversation_type": "direct"
    })))
}

pub async fn get_messages(
    State(state): State<AppState>,
    _current_user: CurrentUser,
    axum::extract::Path(conv_id): axum::extract::Path<String>,
    axum::extract::Query(query): axum::extract::Query<GetMessagesQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let conversation_id = ConversationId::try_from(conv_id)
        .map_err(|_| AppError::Validation("Invalid conversation ID".into()))?;
    let limit = query.limit.unwrap_or(50);

    let before = query.before.and_then(|b| {
        chrono::DateTime::parse_from_rfc3339(&b)
            .ok()
            .map(|d| d.with_timezone(&chrono::Utc))
    });

    let messages = state
        .message_store
        .get_history(&conversation_id, before, limit)
        .await?;

    let response: Vec<MessageResponse> = messages.into_iter().map(MessageResponse::from).collect();

    Ok(Json(json!({
        "messages": response
    })))
}

pub async fn send_message(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(req): Json<SendMessageRequest>,
) -> AppResult<Json<MessageResponse>> {
    if let Err(e) = req.validate() {
        return Err(AppError::Validation(e.to_string()));
    }

    if req.content.trim().is_empty() {
        return Err(AppError::Validation(
            "Message content cannot be empty".into(),
        ));
    }

    let conversation_id = ConversationId::try_from(req.conversation_id)
        .map_err(|_| AppError::Validation("Invalid conversation ID".into()))?;

    let message = Message::text(conversation_id, current_user.user_id, req.content);
    let stored = state.message_store.store(&message).await?;

    Ok(Json(MessageResponse::from(stored)))
}
