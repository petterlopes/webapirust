mod support;

use std::sync::Arc;

use cucumber::{given, then, when, World as _};
use webrust::application::services::auth_service::{AuthService, AuthSession};
use webrust::application::services::user_service::UserService;
use webrust::domain::repositories::user_repository::UserRepository;
use webrust::shared::error::AppError;
use webrust::shared::security::token::JwtManager;

use support::InMemoryUserRepository;

#[derive(Default, cucumber::World)]
pub struct AppWorld {
    #[world(skip)]
    user_service: Option<UserService>,
    #[world(skip)]
    auth_service: Option<AuthService>,
    #[world(skip)]
    last_auth_session: Option<AuthSession>,
    #[world(skip)]
    last_error: Option<AppError>,
}

impl std::fmt::Debug for AppWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppWorld")
            .field("has_user_service", &self.user_service.is_some())
            .field("has_auth_service", &self.auth_service.is_some())
            .field("last_auth_session", &self.last_auth_session)
            .field("last_error", &self.last_error)
            .finish()
    }
}

impl AppWorld {
    fn ensure_services(&mut self) {
        if self.user_service.is_some() && self.auth_service.is_some() {
            return;
        }

        let repository: Arc<dyn UserRepository> = Arc::new(InMemoryUserRepository::new());
        let user_service = UserService::new(repository.clone());
        let jwt_manager = JwtManager::new("test-secret", 60);
        let auth_service = AuthService::new(repository, jwt_manager);

        self.user_service = Some(user_service);
        self.auth_service = Some(auth_service);
    }

    fn user_service(&mut self) -> &mut UserService {
        self.ensure_services();
        self.user_service
            .as_mut()
            .expect("user service should be initialised")
    }

    fn auth_service(&mut self) -> &mut AuthService {
        self.ensure_services();
        self.auth_service
            .as_mut()
            .expect("auth service should be initialised")
    }

    fn clear_results(&mut self) {
        self.last_auth_session = None;
        self.last_error = None;
    }
}

#[given(
    regex = r#"an admin account "(?P<name>[^"]+)" with email "(?P<email>[^"]+)" and password "(?P<password>[^"]+)""#
)]
async fn an_admin_account(world: &mut AppWorld, name: String, email: String, password: String) {
    world.clear_results();
    let _ = world
        .user_service()
        .ensure_admin_account(&name, &email, &password)
        .await
        .expect("failed to ensure admin account");
}

#[when(
    regex = r#"I authenticate with email "(?P<email>[^"]+)" and password "(?P<password>[^"]+)""#
)]
async fn i_authenticate(world: &mut AppWorld, email: String, password: String) {
    match world.auth_service().authenticate(&email, &password).await {
        Ok(session) => {
            world.last_auth_session = Some(session);
            world.last_error = None;
        }
        Err(err) => {
            world.last_auth_session = None;
            world.last_error = Some(err);
        }
    }
}

#[then("the authentication succeeds")]
async fn authentication_succeeds(world: &mut AppWorld) {
    assert!(
        world.last_auth_session.is_some(),
        "expected successful authentication, got error: {:?}",
        world.last_error
    );
}

#[then(regex = r#"the returned user role is "(?P<role>[^"]+)""#)]
async fn returned_user_role(world: &mut AppWorld, role: String) {
    let session = world
        .last_auth_session
        .as_ref()
        .expect("authentication session should be present");
    assert_eq!(session.user.role.as_str(), role);
}

#[then("the access token is issued")]
async fn access_token_issued(world: &mut AppWorld) {
    let session = world
        .last_auth_session
        .as_ref()
        .expect("expected session to be present");
    assert!(!session.token.is_empty(), "token must not be empty");
}

#[then(regex = r#"the authentication fails with message "(?P<message>[^"]+)""#)]
async fn authentication_fails(world: &mut AppWorld, message: String) {
    let err = world
        .last_error
        .as_ref()
        .expect("expected authentication to fail");
    assert!(
        err.to_string().contains(&message),
        "expected error to contain '{message}', got '{}'",
        err
    );
}

#[then("no access token is issued")]
async fn no_access_token_issued(world: &mut AppWorld) {
    assert!(
        world.last_auth_session.is_none(),
        "expected authentication to fail"
    );
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    AppWorld::run("tests/features").await;
}
