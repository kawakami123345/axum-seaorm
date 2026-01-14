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
use openidconnect::{
    AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointMaybeSet,
    EndpointNotSet, EndpointSet, HttpRequest, HttpResponse, IssuerUrl, Nonce, RedirectUrl, Scope,
    TokenResponse,
    core::{CoreClient, CoreProviderMetadata, CoreResponseType},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

use crate::AppState;

const NONCE_COOKIE_NAME: &str = "auth_nonce";
const STATE_COOKIE_NAME: &str = "auth_state";
const SESSION_COOKIE_NAME: &str = "session";
const CSRF_COOKIE_NAME: &str = "csrf_token";
const CSRF_HEADER_NAME: &str = "X-CSRF-Token";

pub type CustomOidcClient = CoreClient<
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;

pub fn auth_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", get(login))
        .route("/callback", get(callback))
        .route("/logout", get(logout))
}

pub async fn create_oidc_client(
    http_client: &reqwest::Client,
) -> anyhow::Result<CustomOidcClient> {
    let client_id =
        ClientId::new(std::env::var("OIDC_CLIENT_ID").context("OIDC_CLIENT_ID must be set")?);
    let client_secret = ClientSecret::new(
        std::env::var("OIDC_CLIENT_SECRET").context("OIDC_CLIENT_SECRET must be set")?,
    );
    let issuer_url =
        IssuerUrl::new(std::env::var("OIDC_ISSUER_URL").context("OIDC_ISSUER_URL must be set")?)?;

    let provider_metadata = CoreProviderMetadata::discover_async(
        issuer_url,
        http_client,
    )
    .await
    .context("Failed to discover OIDC provider metadata")?;

    let app_url = std::env::var("APP_URL").context("APP_URL must be set")?;
    let redirect_url = format!("{}/callback", app_url);

    let client =
        CoreClient::from_provider_metadata(provider_metadata, client_id, Some(client_secret))
            .set_redirect_uri(RedirectUrl::new(redirect_url)?);

    Ok(client)
}

pub async fn async_http_client(
    client: &reqwest::Client,
    request: HttpRequest,
) -> Result<HttpResponse, Box<dyn std::error::Error + Send + Sync>> {
    let (parts, body) = request.into_parts();
    let url = parts.uri.to_string();

    let mut builder = client.request(parts.method, url);
    builder = builder.headers(parts.headers);
    builder = builder.body(body);

    let response = builder.send().await?;

    let status = response.status();
    let headers = response.headers().clone();
    let body = response.bytes().await?.to_vec();

    let mut resp = HttpResponse::new(body);
    *resp.status_mut() = status;
    *resp.headers_mut() = headers;

    Ok(resp)
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

#[derive(Serialize, Deserialize, Clone)]
pub struct UserSession {
    pub sub: String,
    pub roles: Vec<String>,
}

/// 認証ミドルウェア: セッションCookieがない場合は 401 Unauthorized を返す
pub async fn require_auth(
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
    mut request: axum::extract::Request,
    next: Next,
) -> Response {
    // セッションCookieが存在するか確認
    if let Some(cookie) = cookies.private(&state.cookie_key).get(SESSION_COOKIE_NAME) {
        if let Ok(session) = serde_json::from_str::<UserSession>(cookie.value()) {
            request.extensions_mut().insert(session);
            return next.run(request).await;
        }
    }
    (axum::http::StatusCode::UNAUTHORIZED, "Unauthorized").into_response()
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
) -> Response {
    let client = &state.oidc_client;

    let nonce_str = if let Some(cookie) = cookies.private(&state.cookie_key).get(NONCE_COOKIE_NAME)
    {
        cookie.value().to_string()
    } else {
        return Redirect::to("/login?error=missing_nonce").into_response();
    };

    let state_str = if let Some(cookie) = cookies.private(&state.cookie_key).get(STATE_COOKIE_NAME)
    {
        cookie.value().to_string()
    } else {
        return Redirect::to("/login?error=missing_state").into_response();
    };

    if params.state != state_str {
        return Redirect::to("/login?error=invalid_state").into_response();
    }

    let http_client = &state.http_client;

    let token_response = match client.exchange_code(AuthorizationCode::new(params.code)) {
        Ok(request) => request.request_async(http_client).await,
        Err(e) => {
            error!(error = ?e, "Failed to create token exchange request");
            return Redirect::to("/login?error=token_exchange_request_failed").into_response();
        }
    };

    match token_response {
        Ok(token_response) => {
            let id_token = token_response.id_token();
            match id_token {
                Some(id_token) => {
                    let nonce = Nonce::new(nonce_str);
                    let claims = id_token.claims(&client.id_token_verifier(), &nonce);

                    match claims {
                        Ok(claims) => {
                            let sub = claims.subject().to_string();
                            info!(sub = %sub, "User login successful");

                            // rolesの取得ロジック
                            let roles = if let Ok(id_token_str) = serde_json::to_string(id_token) {
                                extract_roles(&id_token_str)
                            } else {
                                vec![]
                            };

                            let user_session = UserSession {
                                sub: sub.clone(),
                                roles,
                            };
                            let session_json = serde_json::to_string(&user_session).unwrap();

                            let mut session_cookie = Cookie::new(SESSION_COOKIE_NAME, session_json);
                            session_cookie.set_path("/");
                            session_cookie.set_http_only(true);
                            session_cookie.set_same_site(SameSite::Lax);
                            #[cfg(not(debug_assertions))]
                            session_cookie.set_secure(true);
                            session_cookie.set_max_age(Some(
                                time::Duration::hours(24).try_into().unwrap_or_else(|_| {
                                    time::Duration::seconds(86400).try_into().unwrap()
                                }),
                            ));

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

                            let frontend_url = std::env::var("FRONTEND_URL")
                                .unwrap_or_else(|_| "http://localhost:5173".to_string());
                            Redirect::to(&frontend_url).into_response()
                        }
                        Err(e) => {
                            error!(error = ?e, "ID token validation failed");
                            Redirect::to("/login?error=invalid_token").into_response()
                        }
                    }
                }
                None => Redirect::to("/login?error=no_id_token").into_response(),
            }
        }
        Err(e) => {
            error!(error = ?e, "Token exchange failed");
            Redirect::to("/login?error=token_exchange_failed").into_response()
        }
    }
}

/// ログアウト: セッションCookieを削除し、Keycloakからもログアウトする
pub async fn logout(State(state): State<Arc<AppState>>, cookies: Cookies) -> impl IntoResponse {
    let mut cookie = Cookie::new(SESSION_COOKIE_NAME, "");
    cookie.set_path("/");
    cookie.set_max_age(Some(time::Duration::seconds(0).try_into().unwrap()));

    cookies.private(&state.cookie_key).remove(cookie);

    // Logout logic
    let client_id = match std::env::var("OIDC_CLIENT_ID") {
        Ok(v) => v,
        Err(_) => {
            error!("OIDC_CLIENT_ID must be set");
            return Redirect::to("/login?error=internal_configuration_error").into_response();
        }
    };
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
