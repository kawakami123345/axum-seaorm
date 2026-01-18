use anyhow::Context;
use axum::{
    Router,
    extract::{Query, State},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
    routing::get,
};

use tower_cookies::{Cookie, Cookies, cookie::SameSite};

use base64::Engine;
use openidconnect::core::{
    CoreAuthDisplay, CoreAuthPrompt, CoreErrorResponseType, CoreGenderClaim, CoreJsonWebKey,
    CoreJweContentEncryptionAlgorithm, CoreRevocableToken, CoreTokenIntrospectionResponse,
    CoreTokenResponse,
};
use openidconnect::{
    AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    EmptyAdditionalClaims, EndpointMaybeSet, EndpointNotSet, EndpointSet, IssuerUrl, Nonce,
    RedirectUrl, RevocationErrorResponseType, Scope, StandardErrorResponse, TokenResponse,
    core::{CoreClient, CoreProviderMetadata, CoreResponseType},
};

pub type AppClient = openidconnect::Client<
    EmptyAdditionalClaims,
    CoreAuthDisplay,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJsonWebKey,
    CoreAuthPrompt,
    StandardErrorResponse<CoreErrorResponseType>,
    CoreTokenResponse,
    CoreTokenIntrospectionResponse,
    CoreRevocableToken,
    StandardErrorResponse<RevocationErrorResponseType>,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;
use serde::Deserialize;
use std::sync::Arc;
use tracing::{error, info};

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

pub async fn create_oidc_client(http_client: &reqwest::Client) -> anyhow::Result<AppClient> {
    let client_id =
        ClientId::new(std::env::var("OIDC_CLIENT_ID").context("OIDC_CLIENT_ID must be set")?);
    let client_secret = ClientSecret::new(
        std::env::var("OIDC_CLIENT_SECRET").context("OIDC_CLIENT_SECRET must be set")?,
    );
    let issuer_url =
        IssuerUrl::new(std::env::var("OIDC_ISSUER_URL").context("OIDC_ISSUER_URL must be set")?)?;

    let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, http_client)
        .await
        .context("Failed to discover OIDC provider metadata")?;

    let client =
        CoreClient::from_provider_metadata(provider_metadata, client_id, Some(client_secret));

    Ok(client)
}

#[derive(Debug, Deserialize)]
struct AuthRequest {
    code: String,
    state: String,
}

#[derive(Debug, Deserialize)]
struct RoleClaims {
    #[serde(default)]
    roles: Vec<String>,
}

/// 認証ミドルウェア: セッションCookieがない場合は 401 Unauthorized を返す
pub async fn require_auth(
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
    mut request: axum::extract::Request,
    next: Next,
) -> Response {
    // セッションCookieが存在するか確認
    if let Some(cookie) = cookies.private(&state.cookie_key).get(SESSION_COOKIE_NAME)
        && let Ok(session) = serde_json::from_str::<usecase::UserContext>(cookie.value())
    {
        request.extensions_mut().insert(session);
        return next.run(request).await;
    }
    (axum::http::StatusCode::UNAUTHORIZED, "Unauthorized").into_response()
}

async fn login(State(state): State<Arc<AppState>>, cookies: Cookies) -> impl IntoResponse {
    let client = &state.oidc_client;

    let app_url = std::env::var("APP_URL").expect("APP_URL must be set");
    let redirect_url =
        RedirectUrl::new(format!("{}/callback", app_url)).expect("Invalid redirect URL");

    let client = client.clone().set_redirect_uri(redirect_url);

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

    let max_age = time::Duration::minutes(10);

    let mut nonce_cookie = Cookie::new(NONCE_COOKIE_NAME, nonce.secret().to_string());
    nonce_cookie.set_path("/");
    nonce_cookie.set_http_only(true);
    nonce_cookie.set_same_site(SameSite::Lax);
    #[cfg(not(debug_assertions))]
    nonce_cookie.set_secure(true);
    nonce_cookie.set_max_age(Some(max_age));

    let mut state_cookie = Cookie::new(STATE_COOKIE_NAME, csrf_token.secret().to_string());
    state_cookie.set_path("/");
    state_cookie.set_http_only(true);
    state_cookie.set_same_site(SameSite::Lax);
    #[cfg(not(debug_assertions))]
    state_cookie.set_secure(true);
    state_cookie.set_max_age(Some(max_age));

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
    let cookie_key = &state.cookie_key;

    // 1. Validate Cookies & State
    let Some(nonce_cookie) = cookies.private(cookie_key).get(NONCE_COOKIE_NAME) else {
        return Redirect::to("/login?error=missing_nonce").into_response();
    };
    let nonce_str = nonce_cookie.value().to_string();

    let Some(state_cookie) = cookies.private(cookie_key).get(STATE_COOKIE_NAME) else {
        return Redirect::to("/login?error=missing_state").into_response();
    };
    let state_str = state_cookie.value().to_string();

    if params.state != state_str {
        return Redirect::to("/login?error=invalid_state").into_response();
    }

    // 2. Exchange Code for Token
    let app_url = std::env::var("APP_URL").expect("APP_URL must be set");
    let redirect_url =
        RedirectUrl::new(format!("{}/callback", app_url)).expect("Invalid redirect URL");
    let client = client.clone().set_redirect_uri(redirect_url);

    let token_request = match client.exchange_code(AuthorizationCode::new(params.code)) {
        Ok(r) => r,
        Err(e) => {
            error!(error = ?e, "Failed to create token request");
            return Redirect::to("/login?error=internal_configuration_error").into_response();
        }
    };

    let token_response = match token_request.request_async(&state.http_client).await {
        Ok(r) => r,
        Err(e) => {
            error!(error = ?e, "Token exchange failed");
            return Redirect::to("/login?error=token_exchange_failed").into_response();
        }
    };

    // 3. Verify ID Token & Claims
    let Some(id_token) = token_response.id_token() else {
        return Redirect::to("/login?error=no_id_token").into_response();
    };

    let claims = match id_token.claims(&client.id_token_verifier(), &Nonce::new(nonce_str)) {
        Ok(c) => c,
        Err(e) => {
            error!(error = ?e, "ID token validation failed");
            return Redirect::to("/login?error=invalid_token").into_response();
        }
    };

    let sub = claims.subject().to_string();
    info!(sub = %sub, "User login successful");

    // 4. Create Session
    let roles = if let Ok(id_token_str) = serde_json::to_string(id_token) {
        extract_roles(&id_token_str)
    } else {
        vec![]
    };

    let user_session = usecase::UserContext::new(sub.parse().unwrap(), roles);
    let session_json = serde_json::to_string(&user_session).unwrap();

    let mut session_cookie = Cookie::new(SESSION_COOKIE_NAME, session_json);
    session_cookie.set_path("/");
    session_cookie.set_http_only(true);
    session_cookie.set_same_site(SameSite::Lax);
    #[cfg(not(debug_assertions))]
    session_cookie.set_secure(true);
    session_cookie.set_max_age(Some(time::Duration::hours(24)));

    cookies.private(cookie_key).add(session_cookie);

    // 5. Cleanup & Redirect
    let mut delete_nonce = Cookie::new(NONCE_COOKIE_NAME, "");
    delete_nonce.set_path("/");
    delete_nonce.set_max_age(Some(time::Duration::ZERO));
    cookies.private(cookie_key).remove(delete_nonce);

    let mut delete_state = Cookie::new(STATE_COOKIE_NAME, "");
    delete_state.set_path("/");
    delete_state.set_max_age(Some(time::Duration::ZERO));
    cookies.private(cookie_key).remove(delete_state);

    let frontend_url =
        std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());
    Redirect::to(&frontend_url).into_response()
}

/// ログアウト: セッションCookieを削除し、Keycloakからもログアウトする
pub async fn logout(State(state): State<Arc<AppState>>, cookies: Cookies) -> impl IntoResponse {
    let mut cookie = Cookie::new(SESSION_COOKIE_NAME, "");
    cookie.set_path("/");
    cookie.set_max_age(Some(time::Duration::seconds(0)));

    cookies.private(&state.cookie_key).remove(cookie);

    // Logout logic
    let client_id = state.oidc_client.client_id().to_string();
    let logout_url_base = match std::env::var("OIDC_LOGOUT_URL") {
        Ok(v) => v,
        Err(_) => {
            error!("OIDC_LOGOUT_URL must be set");
            return Redirect::to("/login?error=internal_configuration_error").into_response();
        }
    };
    let frontend_url =
        std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());

    let mut url = match openidconnect::url::Url::parse(&logout_url_base) {
        Ok(u) => u,
        Err(e) => {
            error!(error = ?e, "Failed to parse logout url");
            return Redirect::to("/login?error=internal_configuration_error").into_response();
        }
    };

    url.query_pairs_mut()
        .append_pair(
            "post_logout_redirect_uri",
            &format!("{}/login", frontend_url),
        )
        .append_pair("client_id", &client_id);

    Redirect::to(url.as_str()).into_response()
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
        #[cfg(not(debug_assertions))]
        cookie.set_secure(true);
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

fn extract_roles(id_token_str: &str) -> Vec<String> {
    // 前後の引用符を外す (JSON文字列として来る可能性があるため)
    let jwt_str = id_token_str.trim_matches('"');
    let parts: Vec<&str> = jwt_str.split('.').collect();
    if parts.len() != 3 {
        return vec![];
    }
    let payload = parts[1];

    // Base64デコード
    // URL_SAFE_NO_PAD を優先するが、失敗した場合は URL_SAFE (パディングあり) を試す
    let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let decoded = engine
        .decode(payload)
        .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(payload))
        .unwrap_or_default();

    if let Ok(claims) = serde_json::from_slice::<RoleClaims>(&decoded) {
        claims.roles
    } else {
        vec![]
    }
}
