use axum::{
    extract::State,
    Json,
};
use serde_json::json;
use validator::Validate;

use crate::api::dto::*;
use crate::api::AuthorizationHeader;
use crate::auth::AuthProvider;
use crate::domain::{GroupId, UserId};
use crate::error::{AppError, AppResult};
use crate::api::AppState;

pub async fn create_group(
    State(state): State<AppState>,
    axum_extra::extract::TypedHeader(headers): axum_extra::extract::TypedHeader<AuthorizationHeader>,
    Json(req): Json<CreateGroupRequest>,
) -> AppResult<Json<GroupResponse>> {
    let owner_id = get_current_user_id(&state, headers.token()).await?;
    
    if let Err(e) = req.validate() {
        return Err(AppError::Validation(e.to_string()));
    }
    
    if req.name.trim().is_empty() {
        return Err(AppError::Validation("Group name cannot be empty".into()));
    }
    
    let group = state.group_service.create_group(req.name, owner_id).await?;
    
    Ok(Json(GroupResponse::from(group)))
}

pub async fn get_group(
    State(state): State<AppState>,
    axum_extra::extract::TypedHeader(headers): axum_extra::extract::TypedHeader<AuthorizationHeader>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let _user_id = get_current_user_id(&state, headers.token()).await?;
    
    let group_id = GroupId::parse(&id)
        .map_err(|_| AppError::NotFound("Invalid group ID".to_string()))?;
    
    let group = state.group_service.get_group(&group_id).await?
        .ok_or_else(|| AppError::NotFound("Group not found".to_string()))?;
    
    Ok(Json(json!({
        "id": group.id.to_string(),
        "name": group.name,
        "description": group.description,
        "owner_id": group.owner_id.to_string(),
        "member_count": group.member_count()
    })))
}

pub async fn add_member(
    State(state): State<AppState>,
    axum_extra::extract::TypedHeader(headers): axum_extra::extract::TypedHeader<AuthorizationHeader>,
    axum::extract::Path(group_id): axum::extract::Path<String>,
    Json(req): Json<AddGroupMemberRequest>,
) -> AppResult<Json<SuccessResponse>> {
    let _user_id = get_current_user_id(&state, headers.token()).await?;
    
    let gid = GroupId::parse(&group_id)
        .map_err(|_| AppError::NotFound("Invalid group ID".to_string()))?;
    
    let member_id = UserId::parse(&req.user_id)
        .map_err(|_| AppError::Validation("Invalid user ID".to_string()))?;
    
    state.group_service.add_member(&gid, member_id).await?;
    
    Ok(Json(SuccessResponse {
        success: true,
        message: "Member added".to_string(),
    }))
}

pub async fn remove_member(
    State(state): State<AppState>,
    axum_extra::extract::TypedHeader(headers): axum_extra::extract::TypedHeader<AuthorizationHeader>,
    axum::extract::Path((group_id, user_id)): axum::extract::Path<(String, String)>,
) -> AppResult<Json<SuccessResponse>> {
    let _user_id = get_current_user_id(&state, headers.token()).await?;
    
    let gid = GroupId::parse(&group_id)
        .map_err(|_| AppError::NotFound("Invalid group ID".to_string()))?;
    
    let member_id = UserId::parse(&user_id)
        .map_err(|_| AppError::NotFound("Invalid user ID".to_string()))?;
    
    state.group_service.remove_member(&gid, &member_id).await?;
    
    Ok(Json(SuccessResponse {
        success: true,
        message: "Member removed".to_string(),
    }))
}

pub async fn get_user_groups(
    State(state): State<AppState>,
    axum_extra::extract::TypedHeader(headers): axum_extra::extract::TypedHeader<AuthorizationHeader>,
) -> AppResult<Json<serde_json::Value>> {
    let user_id = get_current_user_id(&state, headers.token()).await?;
    
    let groups = state.group_service.get_user_groups(&user_id).await?;
    
    let groups_json: Vec<serde_json::Value> = groups
        .into_iter()
        .map(|g| json!({
            "id": g.id.to_string(),
            "name": g.name,
            "description": g.description,
            "owner_id": g.owner_id.to_string(),
            "member_count": g.member_count()
        }))
        .collect();
    
    Ok(Json(json!({
        "groups": groups_json
    })))
}

pub async fn get_group_members(
    State(state): State<AppState>,
    axum_extra::extract::TypedHeader(headers): axum_extra::extract::TypedHeader<AuthorizationHeader>,
    axum::extract::Path(group_id): axum::extract::Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let _user_id = get_current_user_id(&state, headers.token()).await?;
    
    let gid = GroupId::parse(&group_id)
        .map_err(|_| AppError::NotFound("Invalid group ID".to_string()))?;
    
    let group = state.group_service.get_group(&gid).await?
        .ok_or_else(|| AppError::NotFound("Group not found".to_string()))?;
    
    let members_json: Vec<serde_json::Value> = group.members
        .into_iter()
        .map(|m| json!({
            "user_id": m.user_id.to_string(),
            "role": m.role.to_string(),
            "joined_at": m.joined_at.to_rfc3339()
        }))
        .collect();
    
    Ok(Json(json!({
        "members": members_json
    })))
}

async fn get_current_user_id(state: &AppState, token: &str) -> AppResult<UserId> {
    let claims = state.jwt_provider
        .validate_token(token)
        .await
        .map_err(|_| AppError::Unauthorized("Invalid token".into()))?;
    Ok(claims.user_id)
}