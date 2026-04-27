use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;

use crate::api::AppState;
use crate::auth::{AuthProvider, AuthUser};
use crate::domain::UserId;
use crate::error::{AppError, AuthError};

pub struct CurrentUser {
    pub user_id: UserId,
    pub username: String,
    pub roles: Vec<String>,
    pub jti: Option<String>,
}

impl CurrentUser {
    pub fn from_auth_user(auth_user: AuthUser) -> Self {
        Self {
            user_id: auth_user.user_id,
            username: auth_user.username,
            roles: auth_user.roles,
            jti: auth_user.jti,
        }
    }
}

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| AppError::Auth(AuthError::InvalidToken))?;

        let auth_user = state
            .jwt_provider
            .validate_token(bearer.token())
            .await
            .map_err(|_| AppError::Auth(AuthError::InvalidToken))?;

        Ok(CurrentUser::from_auth_user(auth_user))
    }
}
