use utoipa::openapi;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::application::dtos::auth::{AuthenticatedUserDto, LoginRequestDto, LoginResponseDto};
use crate::application::dtos::user::{CreateUserDto, UpdateUserDto, UserResponseDto};
use crate::shared::error::ErrorResponse;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::presentation::http::controllers::auth_controller::login,
        crate::presentation::http::controllers::users_controller::create_user,
        crate::presentation::http::controllers::users_controller::list_users,
        crate::presentation::http::controllers::users_controller::get_user,
        crate::presentation::http::controllers::users_controller::update_user,
        crate::presentation::http::controllers::users_controller::delete_user
    ),
    components(
        schemas(
            AuthenticatedUserDto,
            LoginRequestDto,
            LoginResponseDto,
            CreateUserDto,
            UpdateUserDto,
            UserResponseDto,
            ErrorResponse
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Auth", description = "Authentication operations"),
        (name = "Users", description = "User management")
    )
)]
pub struct ApiDoc;

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearerAuth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}
