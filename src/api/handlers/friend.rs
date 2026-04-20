use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::api::dto::*;
use crate::api::AppState;
use crate::api::AuthorizationHeader;
use crate::auth::AuthProvider;
use crate::domain::{FriendRequestId, UserId};
use crate::error::{AppError, AppResult};

pub async fn get_friends(
    State(state): State<AppState>,
    axum_extra::extract::TypedHeader(headers): axum_extra::extract::TypedHeader<AuthorizationHeader>,
) -> AppResult<Json<serde_json::Value>> {
    let user_id = get_current_user_id(&state, headers.token()).await?;
    let friends = state.friend_service.get_friends(&user_id).await?;

    let response: Vec<FriendshipResponse> = friends.into_iter().map(FriendshipResponse::from).collect();

    Ok(Json(json!({
        "friends": response
    })))
}

pub async fn delete_friend(
    State(state): State<AppState>,
    axum_extra::extract::TypedHeader(headers): axum_extra::extract::TypedHeader<AuthorizationHeader>,
    Path(friend_id): Path<String>,
) -> AppResult<Json<SuccessResponse>> {
    let user_id = get_current_user_id(&state, headers.token()).await?;
    let friend_uuid = Uuid::parse_str(&friend_id)
        .map_err(|_| AppError::Validation("Invalid friend ID".into()))?;
    let friend_user_id = UserId::from(friend_uuid);

    state.friend_service.remove_friend(&user_id, &friend_user_id).await?;

    Ok(Json(SuccessResponse {
        success: true,
        message: "Friend removed successfully".to_string(),
    }))
}

pub async fn get_pending_requests(
    State(state): State<AppState>,
    axum_extra::extract::TypedHeader(headers): axum_extra::extract::TypedHeader<AuthorizationHeader>,
) -> AppResult<Json<serde_json::Value>> {
    let user_id = get_current_user_id(&state, headers.token()).await?;
    let requests = state.friend_service.get_pending_requests(&user_id).await?;

    let response: Vec<FriendRequestResponse> = requests.into_iter().map(FriendRequestResponse::from).collect();

    Ok(Json(json!({
        "requests": response
    })))
}

pub async fn get_sent_requests(
    State(state): State<AppState>,
    axum_extra::extract::TypedHeader(headers): axum_extra::extract::TypedHeader<AuthorizationHeader>,
) -> AppResult<Json<serde_json::Value>> {
    let user_id = get_current_user_id(&state, headers.token()).await?;
    let requests = state.friend_service.get_sent_requests(&user_id).await?;

    let response: Vec<FriendRequestResponse> = requests.into_iter().map(FriendRequestResponse::from).collect();

    Ok(Json(json!({
        "requests": response
    })))
}

pub async fn send_friend_request(
    State(state): State<AppState>,
    axum_extra::extract::TypedHeader(headers): axum_extra::extract::TypedHeader<AuthorizationHeader>,
    Json(req): Json<SendFriendRequest>,
) -> AppResult<Json<FriendRequestResponse>> {
    let from_user_id = get_current_user_id(&state, headers.token()).await?;
    let to_uuid = Uuid::parse_str(&req.to_user_id)
        .map_err(|_| AppError::Validation("Invalid user ID".into()))?;
    let to_user_id = UserId::from(to_uuid);

    if from_user_id == to_user_id {
        return Err(AppError::Validation("Cannot send friend request to yourself".into()));
    }

    let is_friend = state.friend_service.is_friend(&from_user_id, &to_user_id).await?;
    if is_friend {
        return Err(AppError::Validation("Already friends with this user".into()));
    }

    let has_pending = state.friend_service.has_pending_request(&from_user_id, &to_user_id).await?;
    if has_pending {
        return Err(AppError::Validation("Friend request already pending".into()));
    }

    let request = state.friend_service
        .send_request(from_user_id, to_user_id, req.message)
        .await?;

    Ok(Json(FriendRequestResponse::from(request)))
}

pub async fn accept_friend_request(
    State(state): State<AppState>,
    axum_extra::extract::TypedHeader(headers): axum_extra::extract::TypedHeader<AuthorizationHeader>,
    Path(request_id): Path<String>,
) -> AppResult<Json<FriendshipResponse>> {
    let _user_id = get_current_user_id(&state, headers.token()).await?;
    let request_uuid = Uuid::parse_str(&request_id)
        .map_err(|_| AppError::Validation("Invalid request ID".into()))?;
    let friend_request_id = FriendRequestId::from(request_uuid);

    let friendship = state.friend_service.accept_request(&friend_request_id).await?;

    Ok(Json(FriendshipResponse::from(friendship)))
}

pub async fn reject_friend_request(
    State(state): State<AppState>,
    axum_extra::extract::TypedHeader(headers): axum_extra::extract::TypedHeader<AuthorizationHeader>,
    Path(request_id): Path<String>,
) -> AppResult<Json<SuccessResponse>> {
    let _user_id = get_current_user_id(&state, headers.token()).await?;
    let request_uuid = Uuid::parse_str(&request_id)
        .map_err(|_| AppError::Validation("Invalid request ID".into()))?;
    let friend_request_id = FriendRequestId::from(request_uuid);

    state.friend_service.reject_request(&friend_request_id).await?;

    Ok(Json(SuccessResponse {
        success: true,
        message: "Friend request rejected".to_string(),
    }))
}

async fn get_current_user_id(state: &AppState, token: &str) -> AppResult<UserId> {
    let claims = state.jwt_provider
        .validate_token(token)
        .await
        .map_err(|_| AppError::Unauthorized("Invalid token".into()))?;
    Ok(claims.user_id)
}
