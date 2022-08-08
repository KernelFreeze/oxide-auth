use crate::WebError;
use axum::{
    response::{IntoResponse, Response},
    http::{
        StatusCode,
        header::{self, HeaderValue, HeaderName},
        HeaderMap,
    },
};
use oxide_auth::frontends::dev::{WebResponse, Url};

#[derive(Default, Clone, Debug)]
/// Type implementing `WebResponse` and `IntoResponse` for use in route handlers
pub struct OAuthResponse {
    status: StatusCode,
    headers: HeaderMap,
    body: Option<String>,
}

impl OAuthResponse {
    /// Adds a header to the response
    pub fn header(
        &mut self, key: impl TryInto<HeaderName>, value: impl TryInto<HeaderValue>,
    ) -> Result<(), WebError> {
        let name = key.try_into().map_err(|_| WebError::EncodeResponse)?;
        let value = value.try_into().map_err(|_| WebError::EncodeResponse)?;
        self.headers.append(name, value);
        Ok(())
    }

    /// Sets the `StatusCode` of the response
    pub fn status(&mut self, status: impl TryInto<StatusCode>) -> Result<(), WebError> {
        self.status = status.try_into().map_err(|_| WebError::EncodeResponse)?;
        Ok(())
    }

    /// Set the `ContentType` header on a response
    pub fn content_type(mut self, content_type: &str) -> Result<Self, WebError> {
        self.header(header::CONTENT_TYPE, content_type)?;
        Ok(self)
    }

    /// Set the body for the response
    pub fn body(mut self, body: String) -> Self {
        self.body = Some(body);
        self
    }
}

impl WebResponse for OAuthResponse {
    type Error = WebError;

    fn ok(&mut self) -> Result<(), Self::Error> {
        self.status(StatusCode::OK)?;
        Ok(())
    }

    fn redirect(&mut self, url: Url) -> Result<(), Self::Error> {
        self.status(StatusCode::FOUND)?;
        self.header(header::LOCATION, url.as_str())?;
        Ok(())
    }

    fn client_error(&mut self) -> Result<(), Self::Error> {
        self.status(StatusCode::BAD_REQUEST)?;
        Ok(())
    }

    fn unauthorized(&mut self, kind: &str) -> Result<(), Self::Error> {
        self.status(StatusCode::UNAUTHORIZED)?;
        self.header(header::WWW_AUTHENTICATE, kind)?;
        Ok(())
    }

    fn body_text(&mut self, text: &str) -> Result<(), Self::Error> {
        self.body = Some(text.to_owned());
        self.header(header::CONTENT_TYPE, HeaderValue::from_static("text/plain"))?;
        Ok(())
    }

    fn body_json(&mut self, json: &str) -> Result<(), Self::Error> {
        self.body = Some(json.to_owned());
        self.header(header::CONTENT_TYPE, HeaderValue::from_static("application/json"))?;
        Ok(())
    }
}

impl IntoResponse for OAuthResponse {
    fn into_response(self) -> Response {
        (self.status, self.headers, self.body.unwrap_or_default()).into_response()
    }
}

impl From<Response<String>> for OAuthResponse {
    fn from(response: Response<String>) -> Self {
        let (parts, body) = response.into_parts();
        Self {
            status: parts.status,
            headers: parts.headers,
            body: Some(body),
        }
    }
}
