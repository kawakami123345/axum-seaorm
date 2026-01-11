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

const NONCE_COOKIE_NAME: &str = "auth_nonce";
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

/// 認証ミドルウェア: セッションCookieがない場合は 401 Unauthorized を返す
pub async fn require_auth(jar: CookieJar, request: Request<Body>, next: Next) -> Response {
    // セッションCookieが存在するか確認
    if jar.get(SESSION_COOKIE_NAME).is_some() {
        next.run(request).await
    } else {
        (axum::http::StatusCode::UNAUTHORIZED, "Unauthorized").into_response()
    }
}

async fn login(State(state): State<Arc<AppState>>, jar: CookieJar) -> impl IntoResponse {
    let client = &state.oidc_client;

    let (auth_url, _csrf_token, nonce) = client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_extra_param("prompt", "login")
        .url();

    let nonce_cookie = Cookie::build((NONCE_COOKIE_NAME, nonce.secret().to_string()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::minutes(10))
        .build();

    (jar.add(nonce_cookie), Redirect::to(auth_url.as_str()))
}

async fn callback(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Query(params): Query<AuthRequest>,
) -> impl IntoResponse {
    let client = &state.oidc_client;

    let nonce_str = if let Some(cookie) = jar.get(NONCE_COOKIE_NAME) {
        cookie.value().to_string()
    } else {
        return (jar, Redirect::to("/login?error=missing_nonce")).into_response();
    };

    let token_response = client
        .exchange_code(AuthorizationCode::new(params.code))
        .request_async(async_http_client)
        .await;

    match token_response {
        Ok(token_response) => {
            let id_token = token_response.id_token();
            match id_token {
                Some(id_token) => {
                    let nonce = Nonce::new(nonce_str);
                    let claims = id_token.claims(&client.id_token_verifier(), &nonce);

                    match claims {
                        Ok(claims) => {
                            println!("User login: {:?}", claims);

                            // セッションCookieを設定
                            let session_cookie =
                                Cookie::build((SESSION_COOKIE_NAME, "authenticated"))
                                    .path("/")
                                    .http_only(true)
                                    .same_site(SameSite::Lax)
                                    .max_age(time::Duration::hours(24))
                                    .build();

                            // Nonceクッキーを削除
                            let nonce_cookie = Cookie::build((NONCE_COOKIE_NAME, ""))
                                .path("/")
                                .max_age(time::Duration::seconds(0))
                                .build();

                            (jar.add(session_cookie).add(nonce_cookie), Redirect::to("/"))
                                .into_response()
                        }
                        Err(e) => {
                            println!("User login: Err({:?})", e);
                            (jar, Redirect::to("/login?error=invalid_token")).into_response()
                        }
                    }
                }
                None => (jar, Redirect::to("/login?error=no_id_token")).into_response(),
            }
        }
        Err(e) => {
            println!("Token exchange failed: {:?}", e);
            (jar, Redirect::to("/login?error=token_exchange_failed")).into_response()
        }
    }
}

/// ログアウト: セッションCookieを削除し、Keycloakからもログアウトする
pub async fn logout(jar: CookieJar) -> impl IntoResponse {
    let cookie = Cookie::build((SESSION_COOKIE_NAME, ""))
        .path("/")
        .max_age(time::Duration::seconds(0))
        .build();

    // Keycloak logout logic
    let client_id = std::env::var("OIDC_CLIENT_ID").unwrap_or("rust_web".to_string());
    let issuer = std::env::var("OIDC_ISSUER_URL")
        .unwrap_or_else(|_| "http://localhost:8080/realms/rust-web-realm".to_string());

    // Construct Keycloak logout URL directly
    let mut url =
        openidconnect::url::Url::parse(&format!("{}/protocol/openid-connect/logout", issuer))
            .expect("Failed to parse logout url");

    url.query_pairs_mut()
        .append_pair("post_logout_redirect_uri", "http://localhost:3000/login")
        .append_pair("client_id", &client_id);

    (jar.remove(cookie), Redirect::to(url.as_str()))
}
