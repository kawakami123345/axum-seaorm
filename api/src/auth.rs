use anyhow::Context;
use axum::{
    Json, Router,
    extract::{Query, State, rejection::QueryRejection},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
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
    url::Url,
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

use crate::error::AppError;

#[derive(Clone)]
pub struct AuthState {
    pub sessions: Arc<usecase::session::Service>,
    pub oidc_client: AppClient,
    pub cookie_key: tower_cookies::Key,
    pub http_client: reqwest::Client,
    pub redirects: AuthRedirects,
}

#[derive(Clone)]
pub struct AuthRedirects {
    frontend: Url,
    login: Url,
    logout: Url,
}

impl AuthRedirects {
    pub fn new(frontend: &str, logout: &str) -> anyhow::Result<Self> {
        let frontend = Url::parse(frontend).context("Invalid FRONTEND_URL")?;
        let mut login = frontend.clone();
        login.set_query(None);
        login.set_fragment(None);
        login
            .path_segments_mut()
            .map_err(|_| anyhow::anyhow!("Invalid FRONTEND_URL"))?
            .pop_if_empty()
            .push("login");
        Ok(Self {
            frontend,
            login,
            logout: Url::parse(logout).context("Invalid OIDC_LOGOUT_URL")?,
        })
    }

    fn login_error(&self, reason: &str) -> Response {
        let mut url = self.login.clone();
        url.query_pairs_mut().append_pair("error", reason);
        Redirect::to(url.as_str()).into_response()
    }
}

const NONCE_COOKIE_NAME: &str = "auth_nonce";
const STATE_COOKIE_NAME: &str = "auth_state";
const SESSION_COOKIE_NAME: &str = "session";
const CSRF_COOKIE_NAME: &str = "csrf_token";
const CSRF_HEADER_NAME: &str = "X-CSRF-Token";

pub fn auth_router() -> Router<AuthState> {
    Router::new()
        .route("/login", get(login))
        .route("/callback", get(callback))
        .route("/logout", post(logout))
}

pub async fn create_oidc_client(http_client: &reqwest::Client) -> anyhow::Result<AppClient> {
    let client_id =
        ClientId::new(std::env::var("OIDC_CLIENT_ID").context("OIDC_CLIENT_ID must be set")?);
    let client_secret = ClientSecret::new(
        std::env::var("OIDC_CLIENT_SECRET").context("OIDC_CLIENT_SECRET must be set")?,
    );
    let issuer_url =
        IssuerUrl::new(std::env::var("OIDC_ISSUER_URL").context("OIDC_ISSUER_URL must be set")?)?;
    let app_url = std::env::var("APP_URL").context("APP_URL must be set")?;
    let callback = RedirectUrl::new(format!("{}/callback", app_url.trim_end_matches('/')))?;

    let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, http_client)
        .await
        .context("Failed to discover OIDC provider metadata")?;

    let client =
        CoreClient::from_provider_metadata(provider_metadata, client_id, Some(client_secret));

    Ok(client.set_redirect_uri(callback))
}

#[derive(Debug, Deserialize)]
struct AuthRequest {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RoleClaims {
    #[serde(default)]
    roles: Vec<String>,
}

/// 認証ミドルウェア: セッションCookieがない場合は 401 Unauthorized を返す
pub async fn require_auth(
    State(state): State<AuthState>,
    cookies: Cookies,
    mut request: axum::extract::Request,
    next: Next,
) -> Response {
    if let Some(id) = session_id(&cookies, &state.cookie_key) {
        match state.sessions.authenticate(id).await {
            Ok(Some(user)) => {
                request.extensions_mut().insert(user);
                return next.run(request).await;
            }
            Ok(None) => {}
            Err(error) => return AppError(error).into_response(),
        }
    }
    clear_session_cookie(&cookies, &state.cookie_key);
    (axum::http::StatusCode::UNAUTHORIZED, "Unauthorized").into_response()
}

fn session_id(cookies: &Cookies, key: &tower_cookies::Key) -> Option<uuid::Uuid> {
    cookies
        .private(key)
        .get(SESSION_COOKIE_NAME)?
        .value()
        .parse()
        .ok()
}

fn clear_session_cookie(cookies: &Cookies, key: &tower_cookies::Key) {
    let mut cookie = Cookie::new(SESSION_COOKIE_NAME, "");
    cookie.set_path("/");
    cookies.private(key).remove(cookie);
}

async fn login(State(state): State<AuthState>, cookies: Cookies) -> impl IntoResponse {
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
    State(state): State<AuthState>,
    cookies: Cookies,
    params: Result<Query<AuthRequest>, QueryRejection>,
) -> impl IntoResponse {
    let client = &state.oidc_client;
    let cookie_key = &state.cookie_key;
    let Ok(Query(params)) = params else {
        return state.redirects.login_error("invalid_response");
    };

    // 1. Validate Cookies & State
    let Some(nonce_cookie) = cookies.private(cookie_key).get(NONCE_COOKIE_NAME) else {
        return state.redirects.login_error("missing_nonce");
    };
    let nonce_str = nonce_cookie.value().to_string();

    let Some(state_cookie) = cookies.private(cookie_key).get(STATE_COOKIE_NAME) else {
        return state.redirects.login_error("missing_state");
    };
    let state_str = state_cookie.value().to_string();

    if params.state.as_deref() != Some(state_str.as_str()) {
        return state.redirects.login_error("invalid_state");
    }
    if let Some(error) = params.error {
        return state.redirects.login_error(if error == "access_denied" {
            "access_denied"
        } else {
            "provider_error"
        });
    }

    // 2. Exchange Code for Token
    let Some(code) = params.code else {
        return state.redirects.login_error("invalid_response");
    };
    let token_request = match client.exchange_code(AuthorizationCode::new(code)) {
        Ok(r) => r,
        Err(e) => {
            error!(error = ?e, "Failed to create token request");
            return state.redirects.login_error("internal_configuration_error");
        }
    };

    let token_response = match token_request.request_async(&state.http_client).await {
        Ok(r) => r,
        Err(e) => {
            error!(error = ?e, "Token exchange failed");
            return state.redirects.login_error("token_exchange_failed");
        }
    };

    // 3. Verify ID Token & Claims
    let Some(id_token) = token_response.id_token() else {
        return state.redirects.login_error("no_id_token");
    };

    let claims = match id_token.claims(&client.id_token_verifier(), &Nonce::new(nonce_str)) {
        Ok(c) => c,
        Err(e) => {
            error!(error = ?e, "ID token validation failed");
            return state.redirects.login_error("invalid_token");
        }
    };

    let user_id = match claims.subject().as_str().parse::<uuid::Uuid>() {
        Ok(user_id) if !user_id.is_nil() => user_id,
        Ok(_) => return state.redirects.login_error("invalid_subject"),
        Err(error) => {
            error!(error = ?error, "OIDC subject is not a UUID");
            return state.redirects.login_error("invalid_subject");
        }
    };

    // 4. Create Session
    let roles = if let Ok(id_token_str) = serde_json::to_string(id_token) {
        extract_roles(&id_token_str)
    } else {
        vec![]
    };

    // Re-authentication replaces the current browser session, including its
    // server-side record, so a saved copy of the previous cookie stays revoked.
    if let Some(id) = session_id(&cookies, cookie_key)
        && let Err(error) = state.sessions.revoke(id).await
    {
        error!(%error, "Failed to replace session");
        return state.redirects.login_error("session_error");
    }
    let session = match state
        .sessions
        .create(usecase::UserContext::new(user_id, roles))
        .await
    {
        Ok(session) => session,
        Err(error) => {
            error!(%error, "Failed to create session");
            return state.redirects.login_error("session_error");
        }
    };

    let mut session_cookie = Cookie::new(SESSION_COOKIE_NAME, session.id.to_string());
    session_cookie.set_path("/");
    session_cookie.set_http_only(true);
    session_cookie.set_same_site(SameSite::Lax);
    #[cfg(not(debug_assertions))]
    session_cookie.set_secure(true);
    session_cookie.set_max_age(Some(usecase::session::SESSION_LIFETIME));
    session_cookie.set_expires(session.expires_at);

    cookies.private(cookie_key).add(session_cookie);
    info!(sub = %user_id, "User login successful");

    // 5. Cleanup & Redirect
    let mut delete_nonce = Cookie::new(NONCE_COOKIE_NAME, "");
    delete_nonce.set_path("/");
    delete_nonce.set_max_age(Some(time::Duration::ZERO));
    cookies.private(cookie_key).remove(delete_nonce);

    let mut delete_state = Cookie::new(STATE_COOKIE_NAME, "");
    delete_state.set_path("/");
    delete_state.set_max_age(Some(time::Duration::ZERO));
    cookies.private(cookie_key).remove(delete_state);

    Redirect::to(state.redirects.frontend.as_str()).into_response()
}

/// ログアウト: セッションCookieを削除し、Keycloakからもログアウトする
pub async fn logout(State(state): State<AuthState>, cookies: Cookies) -> impl IntoResponse {
    if let Some(id) = session_id(&cookies, &state.cookie_key)
        && let Err(error) = state.sessions.revoke(id).await
    {
        return AppError(error).into_response();
    }
    clear_session_cookie(&cookies, &state.cookie_key);

    let mut url = state.redirects.logout;
    url.query_pairs_mut()
        .append_pair("post_logout_redirect_uri", state.redirects.login.as_str())
        .append_pair("client_id", state.oidc_client.client_id().as_str());
    Json(serde_json::json!({ "redirect_url": url.as_str() })).into_response()
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

#[cfg(test)]
mod tests;
