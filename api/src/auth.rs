use anyhow::Context;
use axum::{
    Router,
    body::Body,
    extract::{Query, State},
    http::Request,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use openidconnect::{
    AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret, CsrfToken, HttpRequest,
    HttpResponse, IssuerUrl, Nonce, RedirectUrl, Scope, TokenResponse,
    core::{CoreClient, CoreProviderMetadata, CoreResponseType},
    reqwest::Error as OidcReqwestError,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

use crate::AppState;

const SESSION_COOKIE_NAME: &str = "session";

pub fn auth_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", get(login))
        .route("/callback", get(callback))
        .route("/logout", get(logout))
}

pub async fn create_oidc_client() -> anyhow::Result<CoreClient> {
    let client_id =
        ClientId::new(std::env::var("OIDC_CLIENT_ID").context("OIDC_CLIENT_ID must be set")?);
    let client_secret = ClientSecret::new(
        std::env::var("OIDC_CLIENT_SECRET").context("OIDC_CLIENT_SECRET must be set")?,
    );
    let issuer_url =
        IssuerUrl::new(std::env::var("OIDC_ISSUER_URL").context("OIDC_ISSUER_URL must be set")?)?;

    // Use custom http client to allow self-signed certs (localhost)
    let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, &async_http_client)
        .await
        .context("Failed to discover OIDC provider metadata")?;

    let client =
        CoreClient::from_provider_metadata(provider_metadata, client_id, Some(client_secret))
            .set_redirect_uri(RedirectUrl::new(
                "http://localhost:3000/callback".to_string(),
            )?);

    Ok(client)
}

// Custom HTTP client that ignores SSL verification (for dev/localhost)
// Handles conversion between http 0.2 (openidconnect) and http 1.0 (reqwest 0.12)
pub async fn async_http_client(
    request: HttpRequest,
) -> Result<HttpResponse, OidcReqwestError<reqwest::Error>> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .connect_timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| OidcReqwestError::Other(e.to_string()))?;

    // Convert Method (http 0.2 -> 1.0 via string)
    let method = reqwest::Method::from_bytes(request.method.as_str().as_bytes())
        .map_err(|e| OidcReqwestError::Other(e.to_string()))?;

    // Convert URL to string to avoid version mismatch
    let url = request.url.to_string();

    let mut builder = client.request(method, url).body(request.body);

    for (name, value) in &request.headers {
        if let Ok(n) = name.as_str().parse::<reqwest::header::HeaderName>() {
            builder = builder.header(n, value.as_bytes());
        }
    }

    let response = builder.send().await.map_err(OidcReqwestError::Reqwest)?;

    let status_u16 = response.status().as_u16();
    let status_code = openidconnect::http::StatusCode::from_u16(status_u16)
        .map_err(|e| OidcReqwestError::Other(e.to_string()))?;

    let mut headers = openidconnect::http::HeaderMap::new();
    for (name, value) in response.headers() {
        if let Ok(n) = openidconnect::http::HeaderName::from_bytes(name.as_str().as_bytes()) {
            if let Ok(v) = openidconnect::http::HeaderValue::from_bytes(value.as_bytes()) {
                headers.append(n, v);
            }
        }
    }

    let body = response
        .bytes()
        .await
        .map_err(OidcReqwestError::Reqwest)?
        .to_vec();

    Ok(HttpResponse {
        status_code,
        headers,
        body,
    })
}

#[derive(Debug, Deserialize)]
struct AuthRequest {
    code: String,
    state: String, // unused but required by OIDC params
}

#[derive(Debug, Serialize, Deserialize)]
struct UserInfo {
    sub: String,
    name: Option<String>,
    email: Option<String>,
}

/// 認証ミドルウェア: セッションCookieがない場合は /login へリダイレクト
pub async fn require_auth(jar: CookieJar, request: Request<Body>, next: Next) -> Response {
    // セッションCookieが存在するか確認
    if jar.get(SESSION_COOKIE_NAME).is_some() {
        next.run(request).await
    } else {
        Redirect::to("/login").into_response()
    }
}

async fn login(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let client = &state.oidc_client;

    let (auth_url, _csrf_token, _nonce) = client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .url();

    // In a real app, store csrf_token and nonce...

    Redirect::to(auth_url.as_str())
}

async fn callback(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Query(params): Query<AuthRequest>,
) -> impl IntoResponse {
    let client = &state.oidc_client;

    let token_response = client
        .exchange_code(AuthorizationCode::new(params.code))
        .request_async(async_http_client)
        .await;

    match token_response {
        Ok(token_response) => {
            let id_token = token_response.id_token();
            match id_token {
                Some(id_token) => {
                    let claims = id_token.claims(&client.id_token_verifier(), &Nonce::new_random());
                    println!("User login: {:?}", claims);

                    // セッションCookieを設定
                    let session_cookie = Cookie::build((SESSION_COOKIE_NAME, "authenticated"))
                        .path("/")
                        .http_only(true)
                        .same_site(SameSite::Lax)
                        .max_age(time::Duration::hours(24))
                        .build();

                    (jar.add(session_cookie), Redirect::to("/"))
                }
                None => (jar, Redirect::to("/login?error=no_id_token")),
            }
        }
        Err(e) => {
            println!("Token exchange failed: {:?}", e);
            (jar, Redirect::to("/login?error=token_exchange_failed"))
        }
    }
}

/// ログアウト: セッションCookieを削除
pub async fn logout(jar: CookieJar) -> impl IntoResponse {
    let cookie = Cookie::build((SESSION_COOKIE_NAME, ""))
        .path("/")
        .max_age(time::Duration::seconds(0))
        .build();
    (jar.remove(cookie), Redirect::to("/login"))
}
