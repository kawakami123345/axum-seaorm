use std::{
    collections::HashMap,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use async_trait::async_trait;
use axum::{
    Extension, Json,
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;
use tower_cookies::{CookieManagerLayer, cookie::CookieJar};
use usecase::{
    UserContext,
    session::{Repository, Session},
};
use uuid::Uuid;

use super::*;

#[derive(Default)]
struct MemoryRepository {
    sessions: Mutex<HashMap<Uuid, Session>>,
    fail_reads: AtomicBool,
    fail_revokes: AtomicBool,
}

#[async_trait]
impl Repository for MemoryRepository {
    async fn create(&self, session: Session) -> anyhow::Result<()> {
        self.sessions.lock().unwrap().insert(session.id, session);
        Ok(())
    }

    async fn find(&self, id: Uuid) -> anyhow::Result<Option<Session>> {
        anyhow::ensure!(
            !self.fail_reads.load(Ordering::SeqCst),
            "storage unavailable"
        );
        Ok(self.sessions.lock().unwrap().get(&id).cloned())
    }

    async fn revoke(&self, id: Uuid) -> anyhow::Result<()> {
        anyhow::ensure!(
            !self.fail_revokes.load(Ordering::SeqCst),
            "storage unavailable"
        );
        self.sessions.lock().unwrap().remove(&id);
        Ok(())
    }
}

fn fixture() -> (AuthState, Arc<MemoryRepository>) {
    let repo = Arc::new(MemoryRepository::default());
    let metadata: CoreProviderMetadata = serde_json::from_value(json!({
        "issuer": "http://localhost/issuer",
        "authorization_endpoint": "http://localhost/authorize",
        "token_endpoint": "http://localhost/token",
        "jwks_uri": "http://localhost/jwks",
        "response_types_supported": ["code"],
        "subject_types_supported": ["public"],
        "id_token_signing_alg_values_supported": ["RS256"]
    }))
    .unwrap();
    let state = AuthState {
        sessions: Arc::new(usecase::session::Service::new(repo.clone())),
        oidc_client: CoreClient::from_provider_metadata(
            metadata,
            ClientId::new("test".into()),
            None,
        ),
        cookie_key: tower_cookies::Key::generate(),
        http_client: reqwest::Client::new(),
    };
    (state, repo)
}

fn app(state: AuthState) -> Router {
    Router::new()
        .route(
            "/protected",
            get(|Extension(user): Extension<UserContext>| async move { Json(user) }),
        )
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            require_auth,
        ))
        .merge(auth_router())
        .layer(CookieManagerLayer::new())
        .with_state(state)
}

fn cookie(state: &AuthState, value: String) -> String {
    let mut jar = CookieJar::new();
    jar.private_mut(&state.cookie_key)
        .add(Cookie::new(SESSION_COOKIE_NAME, value));
    jar.get(SESSION_COOKIE_NAME).unwrap().to_string()
}

async fn get_response(app: Router, path: &str, cookie: &str) -> Response {
    app.oneshot(
        Request::builder()
            .uri(path)
            .header("Cookie", cookie)
            .body(Body::empty())
            .unwrap(),
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn active_server_session_authenticates_its_user() {
    let (state, _) = fixture();
    let user_id = Uuid::new_v4();
    let session = state
        .sessions
        .create(UserContext::new(user_id, vec!["admin".into()]))
        .await
        .unwrap();
    let saved_cookie = cookie(&state, session.id.to_string());
    let response = get_response(app(state), "/protected", &saved_cookie).await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 4096)
        .await
        .unwrap();
    let user: UserContext = serde_json::from_slice(&body).unwrap();
    assert_eq!(*user.user_id(), user_id);
    assert!(user.is_admin());
}

#[tokio::test]
async fn missing_unknown_tampered_and_legacy_sessions_are_rejected() {
    let (state, _) = fixture();
    let unknown = cookie(&state, Uuid::new_v4().to_string());
    let legacy = cookie(
        &state,
        serde_json::to_string(&UserContext::new(Uuid::new_v4(), vec![])).unwrap(),
    );
    let tampered = format!("{unknown}X");
    for saved_cookie in ["", &unknown, &legacy, &tampered] {
        assert_eq!(
            get_response(app(state.clone()), "/protected", saved_cookie)
                .await
                .status(),
            StatusCode::UNAUTHORIZED
        );
    }
}

#[tokio::test]
async fn server_expiry_is_enforced_when_the_client_keeps_sending_the_cookie() {
    let (state, repo) = fixture();
    let session = state
        .sessions
        .create(UserContext::new(Uuid::new_v4(), vec![]))
        .await
        .unwrap();
    let saved_cookie = cookie(&state, session.id.to_string());
    repo.sessions
        .lock()
        .unwrap()
        .get_mut(&session.id)
        .unwrap()
        .expires_at = time::OffsetDateTime::now_utc();

    let response = get_response(app(state), "/protected", &saved_cookie).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert!(
        response
            .headers()
            .get("set-cookie")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("Max-Age=0")
    );
}

#[tokio::test]
async fn logout_revokes_copied_cookies_and_keeps_other_sessions_active() {
    let (state, _) = fixture();
    let user = UserContext::new(Uuid::new_v4(), vec![]);
    let session = state.sessions.create(user.clone()).await.unwrap();
    let other_session = state.sessions.create(user).await.unwrap();
    let saved_cookie = cookie(&state, session.id.to_string());
    let other_cookie = cookie(&state, other_session.id.to_string());
    let router = app(state);

    let response = get_response(router.clone(), "/logout", &saved_cookie).await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert!(
        response
            .headers()
            .get("set-cookie")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("Max-Age=0")
    );
    assert_eq!(
        get_response(router.clone(), "/protected", &saved_cookie)
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        get_response(router, "/protected", &other_cookie)
            .await
            .status(),
        StatusCode::OK
    );
}

#[tokio::test]
async fn storage_failures_are_server_errors_and_failed_logout_can_be_retried() {
    let (state, repo) = fixture();
    let session = state
        .sessions
        .create(UserContext::new(Uuid::new_v4(), vec![]))
        .await
        .unwrap();
    let saved_cookie = cookie(&state, session.id.to_string());
    let router = app(state);

    repo.fail_reads.store(true, Ordering::SeqCst);
    assert_eq!(
        get_response(router.clone(), "/protected", &saved_cookie)
            .await
            .status(),
        StatusCode::INTERNAL_SERVER_ERROR
    );
    repo.fail_reads.store(false, Ordering::SeqCst);

    repo.fail_revokes.store(true, Ordering::SeqCst);
    let response = get_response(router.clone(), "/logout", &saved_cookie).await;
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response.headers().get("set-cookie").is_none());
    assert_eq!(
        get_response(router.clone(), "/protected", &saved_cookie)
            .await
            .status(),
        StatusCode::OK
    );

    repo.fail_revokes.store(false, Ordering::SeqCst);
    assert_eq!(
        get_response(router.clone(), "/logout", &saved_cookie)
            .await
            .status(),
        StatusCode::SEE_OTHER
    );
    assert_eq!(
        get_response(router, "/protected", &saved_cookie)
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
}
