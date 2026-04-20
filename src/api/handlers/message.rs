use axum::{extract::State, Json};
use garde::Validate;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::api::dto::*;
use crate::api::AppState;
use crate::api::AuthorizationHeader;
use crate::auth::AuthProvider;
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
    axum_extra::extract::TypedHeader(headers): axum_extra::extract::TypedHeader<
        AuthorizationHeader,
    >,
) -> AppResult<Json<serde_json::Value>> {
    let _user_id = get_current_user_id(&state, headers.token()).await?;

    Ok(Json(json!({
        "conversations": []
    })))
}

pub async fn create_conversation(
    State(state): State<AppState>,
    axum_extra::extract::TypedHeader(headers): axum_extra::extract::TypedHeader<
        AuthorizationHeader,
    >,
    Json(req): Json<CreateConversationRequest>,
) -> AppResult<Json<ConversationResponse>> {
    let user_id = get_current_user_id(&state, headers.token()).await?;

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
        let is_friend = state.friend_service.is_friend(&user_id, &friend_id).await?;
        if !is_friend {
            return Err(AppError::Validation(
                "Can only create direct conversation with friends".into(),
            ));
        }

        let conv = Conversation::new_direct(user_id, friend_id);
        return Ok(Json(ConversationResponse::from(conv)));
    }

    let conv = Conversation::new_group(participants);
    Ok(Json(ConversationResponse::from(conv)))
}

pub async fn get_conversation(
    State(state): State<AppState>,
    axum_extra::extract::TypedHeader(headers): axum_extra::extract::TypedHeader<
        AuthorizationHeader,
    >,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let _user_id = get_current_user_id(&state, headers.token()).await?;

    Ok(Json(json!({
        "id": id,
        "conversation_type": "direct"
    })))
}

pub async fn get_messages(
    State(state): State<AppState>,
    axum_extra::extract::TypedHeader(headers): axum_extra::extract::TypedHeader<
        AuthorizationHeader,
    >,
    axum::extract::Path(conv_id): axum::extract::Path<String>,
    axum::extract::Query(query): axum::extract::Query<GetMessagesQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let _user_id = get_current_user_id(&state, headers.token()).await?;

    let conversation_id = ConversationId::from(conv_id);
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
    axum_extra::extract::TypedHeader(headers): axum_extra::extract::TypedHeader<
        AuthorizationHeader,
    >,
    Json(req): Json<SendMessageRequest>,
) -> AppResult<Json<MessageResponse>> {
    let user_id = get_current_user_id(&state, headers.token()).await?;

    if let Err(e) = req.validate() {
        return Err(AppError::Validation(e.to_string()));
    }

    if req.content.trim().is_empty() {
        return Err(AppError::Validation(
            "Message content cannot be empty".into(),
        ));
    }

    let conversation_id = ConversationId::from(req.conversation_id);

    let message = Message::text(conversation_id, user_id, req.content);
    let stored = state.message_store.store(&message).await?;

    Ok(Json(MessageResponse::from(stored)))
}

async fn get_current_user_id(state: &AppState, token: &str) -> AppResult<UserId> {
    let claims = state
        .jwt_provider
        .validate_token(token)
        .await
        .map_err(|_| AppError::Unauthorized("Invalid token".into()))?;
    Ok(claims.user_id)
}
