use axum::http::header::AUTHORIZATION;
use axum::http::HeaderValue;

pub struct AuthorizationHeader(pub String);

impl axum_extra::headers::Header for AuthorizationHeader {
    fn name() -> &'static axum::http::HeaderName {
        &AUTHORIZATION
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, axum_extra::headers::Error>
    where
        I: Iterator<Item = &'i HeaderValue>,
    {
        let value = values
            .next()
            .ok_or_else(axum_extra::headers::Error::invalid)?;
        let str = value
            .to_str()
            .map_err(|_| axum_extra::headers::Error::invalid())?;
        Ok(AuthorizationHeader(str.to_owned()))
    }

    fn encode<E: Extend<HeaderValue>>(&self, _values: &mut E) {
        unimplemented!()
    }
}

impl AuthorizationHeader {
    pub fn token(&self) -> &str {
        self.0.trim_start_matches("Bearer ")
    }
}
