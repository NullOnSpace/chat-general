use axum::{
    extract::State,
    Json,
};
use serde_json::json;
use validator::Validate;

use crate::api::dto::*;
use crate::api::AuthorizationHeader;
use crate::auth::{AuthProvider, PasswordHasher};
use crate::domain::User;
use crate::error::{AppError, AppResult};
use crate::api::AppState;

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> AppResult<Json<UserResponse>> {
    if let Err(e) = req.validate() {
        return Err(AppError::Validation(e.to_string()));
    }
    
    if req.password.len() < 6 {
        return Err(AppError::Validation("Password must be at least 6 characters".to_string()));
    }
    
    let password_hasher = PasswordHasher::new();
    let password_hash = password_hasher.hash(&req.password)?;
    
    let user = User::new(req.username.clone(), req.email.clone(), password_hash)
        .with_display_name(req.display_name.unwrap_or_default());
    
    let user = state.user_store.create(user).await?;
    
    Ok(Json(UserResponse::from(user)))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if req.username.is_empty() || req.password.is_empty() {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }
    
    let user = state.user_store.get_by_username(&req.username).await
        .ok_or_else(|| AppError::Unauthorized("User not found".to_string()))?;
    
    let password_hasher = PasswordHasher::new();
    let is_valid = password_hasher.verify(&req.password, &user.password_hash)?;
    
    if !is_valid {
        return Err(AppError::Unauthorized("Invalid password".to_string()));
    }
    
    let roles = vec!["user".to_string()];
    let token_pair = state.jwt_provider.generate_tokens_for_user(
        &user.id,
        &user.username,
        &roles,
    )?;
    
    Ok(Json(json!({
        "access_token": token_pair.access_token,
        "refresh_token": token_pair.refresh_token,
        "token_type": token_pair.token_type,
        "expires_in": token_pair.expires_in,
        "user": {
            "id": user.id.to_string(),
            "username": user.username,
            "email": user.email,
            "display_name": user.display_name
        }
    })))
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshTokenRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let token_pair = state.jwt_provider
        .refresh_token(&req.refresh_token)
        .await
        .map_err(|_| AppError::Unauthorized("Invalid refresh token".to_string()))?;
    
    Ok(Json(json!({
        "access_token": token_pair.access_token,
        "refresh_token": token_pair.refresh_token,
        "token_type": token_pair.token_type,
        "expires_in": token_pair.expires_in
    })))
}

pub async fn logout() -> AppResult<Json<SuccessResponse>> {
    Ok(Json(SuccessResponse {
        success: true,
        message: "Logged out successfully".to_string(),
    }))
}

pub async fn get_current_user(
    State(state): State<AppState>,
    axum_extra::extract::TypedHeader(headers): axum_extra::extract::TypedHeader<AuthorizationHeader>,
) -> AppResult<Json<UserResponse>> {
    let token = headers.token();
    
    let claims = state.jwt_provider
        .validate_token(token)
        .await
        .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;
    
    let user = state.user_store.get_by_id(&claims.user_id.to_string()).await
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
    
    Ok(Json(UserResponse::from(user)))
}

pub async fn get_user_devices() -> AppResult<Json<serde_json::Value>> {
    Ok(Json(json!({
        "devices": []
    })))
}

pub async fn search_users(
    State(state): State<AppState>,
    axum_extra::extract::TypedHeader(headers): axum_extra::extract::TypedHeader<AuthorizationHeader>,
    axum::extract::Query(params): axum::extract::Query<SearchUsersQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let token = headers.token();
    
    state.jwt_provider
        .validate_token(token)
        .await
        .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;
    
    let query = params.q.unwrap_or_default();
    let users: Vec<serde_json::Value> = state.user_store.search(&query).await
        .into_iter()
        .map(|u| json!({
            "id": u.id.to_string(),
            "username": u.username,
            "email": u.email,
            "display_name": u.display_name
        }))
        .collect();
    
    Ok(Json(json!({
        "users": users
    })))
}
