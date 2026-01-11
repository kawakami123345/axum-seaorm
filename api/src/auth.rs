use anyhow::Context;
use axum::{
    Router,
    extract::{Query, State},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
    routing::get,
};

use tower_cookies::{Cookie, Cookies, cookie::SameSite};

use openidconnect::{
    AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret, CsrfToken, HttpRequest,
    HttpResponse, IssuerUrl, Nonce, RedirectUrl, Scope, TokenResponse,
    core::{CoreClient, CoreProviderMetadata, CoreResponseType},
    reqwest::Error as OidcReqwestError,
};
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;

use crate::AppState;

const NONCE_COOKIE_NAME: &str = "auth_nonce";
const STATE_COOKIE_NAME: &str = "auth_state";
const SESSION_COOKIE_NAME: &str = "session";
const CSRF_COOKIE_NAME: &str = "csrf_token";
const CSRF_HEADER_NAME: &str = "X-CSRF-Token";

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

    let app_url = std::env::var("APP_URL").context("APP_URL must be set")?;
    let redirect_url = format!("{}/callback", app_url);

    let client =
        CoreClient::from_provider_metadata(provider_metadata, client_id, Some(client_secret))
            .set_redirect_uri(RedirectUrl::new(redirect_url)?);

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

/// 認証ミドルウェア: セッションCookieがない場合は 401 Unauthorized を返す
pub async fn require_auth(
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
    request: axum::extract::Request,
    next: Next,
) -> Response {
    // セッションCookieが存在するか確認
    if cookies
        .private(&state.cookie_key)
        .get(SESSION_COOKIE_NAME)
        .is_some()
    {
        next.run(request).await
    } else {
        (axum::http::StatusCode::UNAUTHORIZED, "Unauthorized").into_response()
    }
}

async fn login(State(state): State<Arc<AppState>>, cookies: Cookies) -> impl IntoResponse {
    let client = &state.oidc_client;

    let (auth_url, csrf_token, nonce) = client
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

    let mut nonce_cookie = Cookie::new(NONCE_COOKIE_NAME, nonce.secret().to_string());
    nonce_cookie.set_path("/");
    nonce_cookie.set_http_only(true);
    nonce_cookie.set_same_site(SameSite::Lax);
    nonce_cookie.set_max_age(Some(time::Duration::minutes(10).try_into().unwrap()));

    let mut state_cookie = Cookie::new(STATE_COOKIE_NAME, csrf_token.secret().to_string());
    state_cookie.set_path("/");
    state_cookie.set_http_only(true);
    state_cookie.set_same_site(SameSite::Lax);
    state_cookie.set_max_age(Some(time::Duration::minutes(10).try_into().unwrap()));

    cookies.private(&state.cookie_key).add(nonce_cookie);
    cookies.private(&state.cookie_key).add(state_cookie);

    Redirect::to(auth_url.as_str())
}

async fn callback(
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
    Query(params): Query<AuthRequest>,
) -> impl IntoResponse {
    let client = &state.oidc_client;

    let nonce_str = if let Some(cookie) = cookies.private(&state.cookie_key).get(NONCE_COOKIE_NAME)
    {
        cookie.value().to_string()
    } else {
        return Redirect::to("/login?error=missing_nonce");
    };

    let state_str = if let Some(cookie) = cookies.private(&state.cookie_key).get(STATE_COOKIE_NAME)
    {
        cookie.value().to_string()
    } else {
        return Redirect::to("/login?error=missing_state");
    };

    if params.state != state_str {
        return Redirect::to("/login?error=invalid_state");
    }

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
                            let mut session_cookie =
                                Cookie::new(SESSION_COOKIE_NAME, "authenticated");
                            session_cookie.set_path("/");
                            session_cookie.set_http_only(true);
                            session_cookie.set_same_site(SameSite::Lax);
                            session_cookie
                                .set_max_age(Some(time::Duration::hours(24).try_into().unwrap()));

                            cookies.private(&state.cookie_key).add(session_cookie);

                            // クッキーを削除
                            let mut delete_cookie = Cookie::new(NONCE_COOKIE_NAME, "");
                            delete_cookie.set_path("/");
                            delete_cookie
                                .set_max_age(Some(time::Duration::seconds(0).try_into().unwrap()));
                            cookies
                                .private(&state.cookie_key)
                                .remove(delete_cookie.clone());

                            delete_cookie.set_name(STATE_COOKIE_NAME);
                            cookies.private(&state.cookie_key).remove(delete_cookie);

                            let frontend_url =
                                std::env::var("FRONTEND_URL").unwrap_or_else(|_| "/".to_string());
                            Redirect::to(&frontend_url)
                        }
                        Err(e) => {
                            println!("User login: Err({:?})", e);
                            Redirect::to("/login?error=invalid_token")
                        }
                    }
                }
                None => Redirect::to("/login?error=no_id_token"),
            }
        }
        Err(e) => {
            println!("Token exchange failed: {:?}", e);
            Redirect::to("/login?error=token_exchange_failed")
        }
    }
}

/// ログアウト: セッションCookieを削除し、Keycloakからもログアウトする
pub async fn logout(State(state): State<Arc<AppState>>, cookies: Cookies) -> impl IntoResponse {
    let mut cookie = Cookie::new(SESSION_COOKIE_NAME, "");
    cookie.set_path("/");
    cookie.set_max_age(Some(time::Duration::seconds(0).try_into().unwrap()));

    cookies.private(&state.cookie_key).remove(cookie);

    // Keycloak logout logic
    let client_id = std::env::var("OIDC_CLIENT_ID").expect("OIDC_CLIENT_ID must be set");
    let issuer = std::env::var("OIDC_ISSUER_URL").expect("OIDC_ISSUER_URL must be set");
    let frontend_url =
        std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());

    // Construct Keycloak logout URL directly
    let mut url =
        openidconnect::url::Url::parse(&format!("{}/protocol/openid-connect/logout", issuer))
            .expect("Failed to parse logout url");

    url.query_pairs_mut()
        .append_pair(
            "post_logout_redirect_uri",
            &format!("{}/login", frontend_url),
        )
        .append_pair("client_id", &client_id);

    Redirect::to(url.as_str())
}

/// CSRF ミドルウェア: ミューテートするリクエスト (POST, PUT, DELETE, PATCH) に対して CSRF トークンの検証を行う
pub async fn csrf_layer(cookies: Cookies, request: axum::extract::Request, next: Next) -> Response {
    let csrf_cookie_val = if let Some(cookie) = cookies.get(CSRF_COOKIE_NAME) {
        cookie.value().to_string()
    } else {
        // トークンがない場合は新しく生成して設定
        let new_token = uuid::Uuid::new_v4().to_string();
        let mut cookie = Cookie::new(CSRF_COOKIE_NAME, new_token.clone());
        cookie.set_path("/");
        cookie.set_http_only(false); // フロントエンドから読み取れるようにする
        cookie.set_same_site(SameSite::Lax);
        cookies.add(cookie);
        new_token
    };

    let method = request.method();
    if method == axum::http::Method::POST
        || method == axum::http::Method::PUT
        || method == axum::http::Method::DELETE
        || method == axum::http::Method::PATCH
    {
        let csrf_header = request
            .headers()
            .get(CSRF_HEADER_NAME)
            .and_then(|h| h.to_str().ok());

        if csrf_header != Some(&csrf_cookie_val) {
            return (axum::http::StatusCode::FORBIDDEN, "Invalid CSRF token").into_response();
        }
    }

    next.run(request).await
}
